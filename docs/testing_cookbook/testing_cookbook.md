# Testing Cookbook

Dreamtides tests exercise the full rules engine pipeline from action dispatch
through rendering. Tests live in `rules_engine/tests/` as integration tests
(never inline `mod tests`). The test infrastructure provides a builder-based API
for configuring battles, performing actions, and asserting on both player views
simultaneously.

## Table of Contents

- [Test Infrastructure Overview](#test-infrastructure-overview)
- [TestBattle Builder](#testbattle-builder)
- [TestPlayer Configuration](#testplayer-configuration)
- [TestSession](#testsession)
- [TestClient Query API](#testclient-query-api)
- [TestSessionBattleExtension](#testsessionbattleextension)
- [DebugBattleAction](#debugbattleaction)
- [Test Card Conventions](#test-card-conventions)
- [Common Test Patterns](#common-test-patterns)
- [TestStateProvider](#teststateprovider)
- [Running Tests](#running-tests)
- [Parser Tests vs Battle Tests](#parser-tests-vs-battle-tests)

## Test Infrastructure Overview

The test stack has three layers. TestBattle is the outermost builder that
configures a battle's initial conditions. It produces a TestSession, which
orchestrates actions and routes engine responses to two TestClient instances
(one per player). TestClient stores the current visual state that each player
would see, including cards in every zone, player stats, interface buttons, and
display commands.

Every action updates both clients. The engine produces separate command streams
for the acting player and the opponent, and TestSession routes each to the
correct TestClient. This dual-client model lets tests verify that both
perspectives stay consistent after every action.

## TestBattle Builder

TestBattle is the entry point for battle tests. The builder pattern starts with
`TestBattle::builder()` and ends with `.connect()`, which returns a TestSession.

Builder methods (all optional, chainable):

- **.user(TestPlayer)** / **.enemy(TestPlayer)**: Override default player
  configuration (energy, points, etc.).
- **.enemy_agent(GameAI)**: Use an AI opponent instead of a second human. GameAI
  variants include FirstAvailableAction, RandomAction, and
  MonteCarlo(iterations).
- **.seed(u64)**: Set the RNG seed for deterministic behavior. If omitted, a
  default seed of 314159265358979323 (digits of pi) is used. All tests are
  deterministic.
- **.with_dreamwell(DreamwellCardIdList)**: Override the dreamwell card list.
  Defaults to TestDreamwellNoAbilities, which contains only simple
  energy-producing cards with no abilities.

The `.connect()` method performs several setup steps: connects both players to
the rules engine, moves all cards from both hands to decks (tests start with
empty hands), then applies each player's configured points, energy, produced
energy, and spark bonus via debug actions. The deck override is always
TestDeckName::Vanilla.

## TestPlayer Configuration

TestPlayer configures a single player's starting state. It uses the `bon`
crate's Builder derive with four fields, all with `#[builder(into)]` so raw
integers can be passed directly:

- **energy**: Current energy. Defaults to 99 (effectively unlimited).
- **produced_energy**: Total energy produced. Defaults to 99.
- **points**: Victory points. Defaults to 0.
- **spark_bonus**: Bonus spark. Defaults to 0.

The high default energy means most tests can focus on game mechanics without
worrying about resource constraints. Tests that specifically need to verify
energy costs override this to a precise value.

## TestSession

TestSession is the central orchestrator. It holds the TestStateProvider, both
player IDs (randomly generated UUIDs), the battle ID, two TestClient instances,
response version tracking, and the last command sequences for each player.

Key methods:

- **perform_user_action(action)**: Execute a GameAction as the user (player
  one). Calls the engine synchronously, then applies resulting commands to both
  clients.
- **perform_enemy_action(action)**: Same, but as the enemy (player two).
- **perform_player_action(DisplayPlayer, action)**: Dispatches to the
  appropriate method based on the player argument.
- **client(DisplayPlayer)**: Returns a reference to either user_client or
  enemy_client.

Both perform methods accept anything that implements `Into<GameAction>`,
including BattleAction, DebugBattleAction, and other action types. The engine
call is synchronous (perform_action_blocking), so there is no polling loop in
tests.

## TestClient Query API

TestClient represents what one player sees. After every action, both clients are
updated with the engine's response commands.

### Zone Queries

All zone queries live on `TestClientCards` and return a TestClientCardList.
Cards within each list are sorted by their sorting key:

- **user_hand()** / **enemy_hand()**: Cards in each player's hand.
- **user_battlefield()** / **enemy_battlefield()**: Characters on the field.
- **user_void()** / **enemy_void()**: Discard piles.
- **user_banished()** / **enemy_banished()**: Banished (exiled) cards.
- **user_deck()** / **enemy_deck()**: Library cards.
- **stack_cards()**: All cards currently on the stack.
- **browser_cards()**: Cards shown in a browser/selection UI.

TestClientCardList provides `contains(card_id)`, `len()`, `is_empty()`,
`get(index)` for positional access, and `iter()` for iteration.

### Card Inspection

- **cards.get(id)**: Returns a TestClientCard reference (panics if not found).
- **cards.get_revealed(id)**: Returns the RevealedCardView (panics if not
  revealed).
- **cards.get_cost(id)**: Returns the Energy cost parsed from the display
  string.

RevealedCardView exposes name, cost, spark, card_type, rules_text, is_fast,
outline_color, actions (can_play, on_click, can_select_order), effects, and
info_zoom_data. The TestRevealedCardExtension trait adds numeric_cost() and
numeric_spark() for typed access.

### Player State Queries

TestClientPlayer (accessed via `s.user_client.me` or `s.user_client.opponent`)
provides:

- **score()**: Current victory points.
- **energy()**: Current energy.
- **produced_energy()**: Energy produced this turn.
- **total_spark()**: Total spark across all characters.
- **can_act()**: Whether this player can currently take an action.
- **is_current_turn()**: Whether it's this player's turn.

### Interface Queries

- **legal_actions()**: Collects all available GameAction values from interface
  buttons and card actions into a deterministic list.
- **is_game_over()** / **user_won()**: Check game outcome.
- **interface()**: Returns the InterfaceView with all button states.
- **primary_action_button()** / **secondary_action_button()**: Access specific
  buttons.

TestInterfaceView provides text extraction and substring checking for every
button type: screen_overlay_text/contains, primary_action_button_text/contains,
secondary_action_button_text/contains, increment_button_text/contains,
decrement_button_text/contains, dev_button_text/contains,
undo_button_text/contains.

## TestSessionBattleExtension

This extension trait on TestSession provides the high-level API that most tests
use. Import it via `use test_utils::session::test_session_prelude::*`.

### Card Placement and Play

- **create_and_play(player, card)**: The most common operation. Adds a card to
  the player's hand via debug action, plays it, and optionally clicks a target.
  Accepts a BaseCardId directly or a TestPlayCard with a .target() chain.
  Returns the new card's ClientCardId.
- **add_to_hand(player, BaseCardId)**: Add a card to a player's hand. Returns
  the new card's ID.
- **add_to_battlefield(player, BaseCardId)**: Place a card directly on the
  battlefield (bypasses play).
- **add_to_void(player, BaseCardId)**: Place a card in the void.
- **play_card_from_hand(player, card_id)**: Play a card already in hand by
  looking up its can_play action.
- **play_card_from_void(player, card_id)**: For Reclaim: finds the reclaim token
  card (ID prefixed "V") in hand and plays it.
- **activate_ability(player, character_id, ability_number)**: Play an ability
  token card (ID formatted as "A{character_id}/{ability_number}").

### Interaction

- **click_card(player, target_id)**: Select a card as a target via on_click.
  Includes a helpful panic if only one battlefield card exists (suggesting
  auto-selection already occurred).
- **click_primary_button(player, containing)** /
  **click_secondary_button(player, containing)**: Click a UI button, asserting
  the label contains the expected text.
- **click_increment_button(player)** / **click_decrement_button(player)**: Click
  numeric controls.
- **select_card_order(player, card_id, target)**: For Foresee effects: place a
  card at Deck(position) or into Void.

### Turn Management

- **end_turn_remove_opponent_hand(player)**: End the turn and move the
  opponent's hand back to deck. This prevents random draw effects from adding
  noise to the test state.

### Command Inspection

- **find_command(player, extractor)**: Search the last command sequence for a
  specific command type using a closure. Returns the extracted value.
- **find_all_commands(player, extractor)**: Returns all matching commands.
- **set_next_dreamwell_card(player, id)**: Control which dreamwell card is drawn
  next.

## DebugBattleAction

Debug actions provide test-only state manipulation. They are submitted through
perform_user_action or perform_enemy_action like any other action. Available
variants:

- **DrawCard { player }**: Draw a card.
- **SetEnergy / SetPoints / SetProducedEnergy / SetSparkBonus**: Override player
  stats to specific values.
- **AddCardToHand / AddCardToBattlefield / AddCardToVoid**: Inject a specific
  card (by BaseCardId) into a zone.
- **MoveHandToDeck { player }**: Move all cards from hand to deck.
- **SetCardsRemainingInDeck { player, cards }**: Set deck size by moving excess
  to void. Useful for testing edge cases with small or empty decks.
- **OpponentPlayCard { card }**: Have the opponent play a card with prompt
  choices.
- **OpponentContinue**: Cause the opponent to take a "continue" action.
- **SetNextDreamwellCard { base_card_id }**: Control the next dreamwell draw.

The extension trait methods (add_to_hand, add_to_battlefield, etc.) wrap these
debug actions and additionally return the new card's ClientCardId by diffing
card lists before and after the action.

## Test Card Conventions

Test cards are defined in `rules_engine/tabula/test-cards.toml`. Dreamwell test
cards are in `test-dreamwell.toml`.

**Naming**: Every test card name begins with "Test " followed by a descriptive
Title Case name. Names are intentionally long and descriptive since they serve
as documentation of the card's purpose (e.g. "Test Trigger Gain Spark When
Materialize Another Character").

**Generated constants**: The `just tabula-generate` command reads test TOML
files and generates Rust constants in the `test_card` module with
SCREAMING_SNAKE_CASE names derived from card names. For example, "Test Draw One"
becomes `test_card::TEST_DRAW_ONE`. Tests reference cards exclusively through
these typed constants, never raw UUIDs.

**Card design**: Most test cards have low energy costs (0-4) to keep tests
simple. They are either Character type (with a spark value) or Event type. Modal
cards use the special cost "\*" to indicate mode-dependent pricing.

**Variables and rules text**: Rules text uses the standard directive syntax with
curly braces. Variables are defined as "key: value" pairs. Cards requiring
targeting include a prompts field with human-readable prompt text.

## Common Test Patterns

### Simple Effect Verification

The most basic pattern: create a battle, place precondition cards via debug
actions, play a card with create_and_play, then assert zone contents. For
example, a dissolve test adds an enemy character to the battlefield, plays Test
Dissolve targeting it, then asserts the character moved to the void.

### Targeting with Multiple Targets

When only one valid target exists, the engine auto-selects it. When multiple
targets exist, the test must explicitly call click_card to choose. Tests set up
multiple valid targets, play the targeting card, then call click_card on the
desired target.

### Turn Switching for Opponent-Turn Interactions

To test fast cards or responses during the opponent's turn: set up the user's
board, call end_turn_remove_opponent_hand to switch turns cleanly, have the
enemy play a card (which goes on the stack), assert the user can_act(), then
have the user respond with a fast card or counterspell.

### Stack Interactions and Auto-Resolution

The engine's action loop automatically executes any action when it is the only
legal choice. This auto-resolution is the most common source of confusion in
stack-related tests. The should_auto_execute_action function checks whether
there is exactly one legal action available and, if so, executes it without
waiting for player input. The auto-executed actions include PassPriority (when
passing is the only option), StartNextTurn, single-choice prompt selections, and
single-target character or stack card selections.

This means stack items resolve immediately if the opponent has no cards to
respond with. When a player plays a card onto the stack, the opponent receives
priority. If the opponent has no fast cards or other responses, PassPriority is
their only legal action, so the engine auto-executes it and the stack item
resolves before the perform_action call returns. The test never sees the card
"on the stack" because it has already resolved by the time the action completes.

To keep cards on the stack long enough to inspect or interact with them, the
opponent must have at least one playable response in hand. The standard pattern
is to add an extra card (often a fast card or counterspell) to the opponent's
hand before playing the card under test. This gives the opponent more than one
legal action (play the response or pass), which prevents auto-resolution. The
same principle applies to target selection: if only one valid target exists, the
engine auto-selects it. To force manual target selection via click_card, the
test must ensure at least two valid targets are present.

When testing stack resolution explicitly, tests build up multiple stack items,
verify stack sizes and priority (who can_act via me.can_act() and
opponent.can_act()), then resolve items one at a time with
BattleAction::PassPriority. Each pass resolves the top card (LIFO order). After
each resolution, the resolved card's controller receives priority if the stack
is not empty. Tests can verify arrows between stack cards (showing targeting
relationships) and info_zoom_data (showing targeting icons on card hover). When
a target is removed from the stack before its targeting card resolves, the
targeting card resolves with no effect and any arrows pointing to the removed
card disappear.

### Prompt and Browser Interactions

Cards with prompts (modal choices, foresee, return-from-void) open browser UIs.
Tests verify browser content, click cards to make selections, and submit via
click_primary_button. For foresee, tests use select_card_order with Deck(0),
Deck(1), or Void targets.

### Dual Client Verification

A test helper `assert_clients_identical` verifies that both the user and enemy
client views are consistent: hand sizes match between perspectives, battlefields
match, voids match, and numeric values (energy, produced energy, total spark,
score) agree between views.

### Display Command Verification

Tests use find_command and find_all_commands to verify that specific display
commands were emitted, such as FireProjectile (with correct source/target card
IDs), DissolveCard, DisplayEffect, and UpdateBattle.

### Energy Tracking

Tests that care about energy costs set a specific initial energy via
TestPlayer::builder().energy(N), then assert energy values after each action
using s.user_client.me.energy().

### Activated Ability Testing

Characters with activated abilities get token cards in hand with IDs formatted
as "A{character_id}/{ability_number}". Tests use activate_ability to play these
tokens. Multi-use abilities regenerate the token after use; once-per-turn
abilities consume it. Fast abilities work during the opponent's turn.

### Reclaim Testing

Cards with Reclaim go to the void after initial play, and a reclaim token
appears in hand with ID "V{card_id}" and the reclaim cost. Tests use
play_card_from_void to reclaim, then verify the card moves to the banished zone
(not back to void).

## TestStateProvider

TestStateProvider is the in-memory implementation of the StateProvider trait
used in all tests. It replaces filesystem-backed persistence with
Mutex-protected HashMaps inside an Arc.

Key behaviors:

- **Global Tabula cache**: A static OnceLock caches parsed card data across all
  test instances in the process. The first test that initializes loads all card
  TOML files; subsequent tests clone the Arc pointer. This is a critical
  performance optimization since parsing Tabula data is expensive.
- **should_panic_on_error returns true**: Engine errors immediately panic
  instead of being converted to UI error messages. This ensures tests never
  silently swallow errors. Panics also propagate directly (no catch_unwind) so
  test failures show full stack traces.
- **Default deck**: TestDeckName::Vanilla (production uses Core11).
- **Default dreamwell**: TestDreamwellNoAbilities (simple energy-producing cards
  with no abilities), overridable via with_dreamwell on the builder.

## Running Tests

All test commands use `just` (never raw `cargo`):

- **`just test`**: Full workspace test suite. Runs tabula-check first to ensure
  generated files are current. Sets RUST_MIN_STACK=8388608 (8 MB) for the
  parser's deep Chumsky hierarchy. On Linux/Docker or when LOW_MEMORY is set,
  limits to single-threaded execution.
- **`just battle-test <NAME>`**: Run a specific battle test by name filter.
  Routes through a helper script that verifies at least one test matched the
  filter (prevents silently passing on typos).
- **`just parser-test`**: Run parser tests with the elevated stack size.
- **`just review`**: Full pre-push validation gate including build, clippy,
  style checks, and all tests. Takes approximately 5 minutes.

## Parser Tests vs Battle Tests

Parser tests and battle tests are separate Cargo packages testing fundamentally
different things.

**Parser tests** (`parser_tests` crate) directly invoke the lexer, variable
resolver, and parser to convert ability text strings into Ability AST
structures. They do not use TestSession or the rules engine. A thread-local
cached parser avoids the expensive cost of constructing the Chumsky parser on
every test. The `stacker` crate grows the stack when needed. Tests use
`insta::assert_ron_snapshot!` for inline snapshot testing, with expected outputs
embedded as RON-formatted strings in the test source. Run via
`just parser-test`.

**Battle tests** (`battle_tests` crate) exercise the full engine end-to-end.
They create a TestBattle, connect to the engine, perform game actions, and
assert on the resulting client state. They use the TestSessionBattleExtension
API for high-level operations. Organized into subdirectories: basic_tests (turn
sequences, undo, limits, triggered abilities), effect_tests (dissolve, draw,
counterspell, foresee, etc.), static_ability_tests (reclaim), and property_tests
(determinism). Run via `just battle-test`.

In the review gate, parser tests and battle tests run as separate steps since
parser tests require elevated stack sizes.
