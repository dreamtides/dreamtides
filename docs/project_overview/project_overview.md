# Dreamtides

Technical architecture of the Dreamtides roguelike deckbuilding game. Dreamtides
is a two-player card game similar to Magic: the Gathering. Players materialize
characters (putting them into play) and play one-time events. Characters have
spark, which earns victory points during the judgment phase. Cards cost energy
to play. The first player to 12 points wins.

## Project Architecture

Dreamtides has a split architecture: a Unity3D C# frontend and a Rust backend
("rules engine"). Communication is JSON-based. In production, the Rust code
compiles to a native library linked into Unity via C FFI
(rules_engine/src/plugin/). During development, an Axum HTTP server
(rules_engine/src/dev_server/) serves the same API on localhost:26598.

The protocol has three operations: Connect (load/create battle, get initial
state), PerformAction (submit a player action, processed asynchronously), and
Poll (retrieve results). Actions are processed on a background thread; the
client polls for incremental updates during AI turns and a final update when the
human can act. A response version UUID prevents stale/duplicate actions.

All shared types derive `serde` + `schemars` traits. The `schema_generator`
crate emits JSON Schema, which `quicktype` converts to C# classes
(client/Assets/Dreamtides/Schema/Schema.cs). Run `just schema` to regenerate.

The command protocol aims for a react-style model where each update describes
complete UI state instead of imperative mutations. Some engine paths still mix
snapshot-style updates with more imperative sequencing, and new work should
prefer full-state command updates for consistency.

## Crate Structure

The rules engine has ~34 crates in a Cargo workspace (rules_engine/Cargo.toml),
organized in layers:

- **Layer 0 (foundational):** `core_data` defines IDs (UserId, BattleId,
  BaseCardId), numeric newtypes (Energy, Spark, Points), enums (CardType,
  CardSubtype, PlayerName). Depended on by every other crate. `strings` defines
  all UI text via the RLF macro system (strings/src/strings.rlf.rs).
- **Layer 1 (data):** `ability_data` (Ability/Effect/StandardEffect types),
  `ai_data` (GameAI enum), `tabula_generated` (generated card ID constants).
- **Layer 2 (card data):** `tabula_data` (TOML loading, CardDefinition
  building), `parser` (ability text parser), `quest_state`, `action_data`
  (GameAction/BattleAction enums).
- **Layer 3 (state):** `battle_state` is the central hub — BattleState struct
  plus all card zone storage, triggers, prompts, pending effects. Nearly
  everything above Layer 2 depends on it. Hard constraint: max 128 cards per
  battle, stored as u128 bitsets in CardSet.
- **Layer 4 (logic):** `battle_queries` (read-only state queries, legal actions)
  and `battle_mutations` (state changes — effects, triggers, turn phases, card
  movement). Mutations depend on queries, not vice versa.
- **Layer 5 (UI):** `masonry` (FlexNode tree builder), `ui_components`
  (Component trait, buttons, panels), `display_data` (Command enum, BattleView).
- **Layer 6 (AI):** `ai_uct` (Monte Carlo Tree Search) and `ai_agents` (action
  selection entry point).
- **Layer 7 (assembly):** `game_creation`, `database`, `display` (rendering, 17
  deps), `rules_engine` (top-level facade, 17 deps).

## Card Data Pipeline

Card data lives in TOML files under rules_engine/tabula/ (mirrored in
client/Assets/StreamingAssets/Tabula/). The main file is cards.toml (large — do
not read directly). Each card has an id, name, energy-cost, card-type, optional
subtype/spark, and crucially a `rules-text` field with directive syntax like
`"{energy($e)}: Draw {cards($c)}."` plus a `variables` field like
`"e: 2, c: 1"`.

The tabula card set is shared between client and rules engine via symlinked
paths. The standalone `tv` app is used to inspect and edit tabula TOML data.

The `tabula generate` command (run via `just tabula-generate`) parses all card
text through a multi-stage pipeline:

1. **Lex** (parser/src/lexer/): Lowercases all input, tokenizes into
   Word/Directive/punctuation tokens. Directives are `{...}` blocks.
2. **Resolve variables** (parser/src/variables/): Substitutes directives against
   typed VariableBindings. `{energy}` with binding `e: 2` becomes
   `ResolvedToken::Energy(2)`. Handles RLF function syntax, subtypes, figments,
   modal variants, and bare phrases. Each semantic concept maps to a distinct
   ResolvedToken variant.
3. **Parse** (parser/src/parser/): A chumsky parser combinator converts the
   resolved token stream into an Ability AST. Five ability types: Event,
   Triggered, Activated, Static, Named. Effects resolve to StandardEffect
   variants (DrawCards, DissolveCharacter, GainEnergy, Foresee, Kindle, etc.).
   Multi-paragraph card text is split on double newlines — each block parses as
   a separate ability.
4. **Serialize to JSON**: The parsed abilities are written to
   parsed_abilities.json as a map from card UUID to Vec<Ability>.

At runtime, `Tabula::load()` (tabula_data) reads the pre-parsed JSON alongside
the TOML metadata and builds CardDefinition structs. No re-parsing occurs at
runtime. For display, the serializer (parser/src/serializer/) walks the Ability
tree and calls RLF phrase functions from `strings` to produce
rich-text-formatted rules text with colored keywords and plural-aware counts.

## Battle Execution

### The Action Loop

The engine entry point is engine.rs. `perform_action()` deserializes the battle
from its save file, delegates to `handle_battle_action::execute()`, then
re-serializes and saves. The action loop:

1. Push an undo snapshot, then apply the action via
   `apply_battle_action::execute()`.
2. Check `legal_actions::next_to_act()` — who acts next?
3. If a single auto-executable action exists (PassPriority, StartNextTurn,
   single-choice prompt), execute it without waiting for input. Loop to step 1.
4. If the next player is AI, render incremental updates to the human, run
   `agent_search::select_action()`, loop back.
5. If the next player is human, render final updates and return.

A speculative search optimization pre-computes the AI's likely response on a
background thread while waiting for human input.

### Stack and Priority

When a card is played (PlayCardFromHand or PlayCardFromVoid for Reclaim), energy
is spent and the card moves to the stack. The opponent receives priority to
respond. A single priority pass resolves the top stack card — events execute
their effects then move to the void; characters move to the battlefield. If the
stack still has items, the resolved card's controller receives priority.

### Effect Resolution

After every action, three cleanup passes run in sequence:

1. **Drain pending effects** — the `pending_effects` VecDeque is processed FIFO.
   List effects push their first element for execution and re-queue the
   remainder. The loop halts if a prompt is created or the game ends.
2. **Fire triggers** — the `triggers.events` queue is drained. Each trigger
   checks its listener card is still on the battlefield, matches trigger
   conditions, and executes matched abilities. New triggers from these effects
   are appended to the back of the same queue.
3. **Advance the turn state machine** — see Turn Phases below.

Effects that need player input (target selection, modal choices, card ordering)
push a PromptData onto `battle.prompts`, which halts all three loops. After the
player responds, the cleanup passes resume. This interleaving of effects and
prompts is the core control flow mechanism — effects can chain through triggers
indefinitely, bounded only by the finite number of cards in play.

The Effect enum (ability_data/src/effect.rs) has five variants: single
StandardEffect, effect with options, sequential list, list with shared options,
and modal choice. StandardEffect covers all leaf mutations — DrawCards,
DissolveCharacter, GainEnergy, Foresee, Kindle, Counterspell, etc.

### Turn Phases

The turn state machine (battle_mutations/src/phase_mutations/turn.rs) progresses
through: EndingPhaseFinished → FiringEndOfTurnTriggers → Starting → Judgment →
Dreamwell → Draw → Main → Ending. Each transition runs phase logic, then drains
pending effects and fires triggers before advancing. **Judgment** awards points
if the active player's total spark exceeds the opponent's. **Dreamwell** draws
from a shared deck of special cards providing energy production and bonus
effects. **Draw** gives the active player one card. **Main** is where players
take actions freely.

## Logging

Battle execution emits structured logs for debugging and analysis. The primary
battle log entrypoint is `battle_trace!`
(rules_engine/src/battle_queries/src/macros/battle_trace.rs), with additional
infrastructure in the `logging` crate.

## Animation and Display

### Animation Recording

During mutations, `BattleState::push_animation()` records animation events. Each
AnimationStep captures the BattleAnimation variant (PlayCard, DrawCards,
GainEnergy, ScorePoints, etc.) plus a full snapshot clone of the BattleState at
that moment. This snapshot-per-step approach means the renderer can later
reconstruct exact intermediate game states.

### Rendering Pipeline

The display crate's renderer has two entry points: `connect()` produces a single
full state snapshot; `render_updates()` replays recorded animation steps as
interleaved snapshot-then-VFX command sequences, followed by a final snapshot.

The output is a CommandSequence — a list of sequential ParallelCommandGroups.
Each group contains Commands executed simultaneously. The primary command is
UpdateBattle, which carries a complete BattleView
(display_data/src/battle_view.rs) — the full visual state of the battle
including all card views, player state, interface controls, and targeting
arrows. This "react-style" snapshot approach means the client reconstructs its
entire UI from each update rather than applying deltas. Other commands handle
VFX: FireProjectile, DissolveCard, DisplayEffect, PlayAudioClip, Wait,
DisplayGameMessage, etc.

### Masonry UI

For overlay UI (prompts, panels, messages), Rust code builds FlexNode trees
using a Component trait (rules_engine/src/ui_components/). Components like
BoxComponent, TextComponent, ButtonComponent, and PanelComponent compose into
FlexNode trees with full CSS flexbox styling. These trees are serialized in
UpdateBattle commands. On the client side, a Masonry reconciler
(client/Assets/Dreamtides/Masonry/Reconciler.cs) diffs the new FlexNode tree
against the previous one and applies changes to Unity's UIToolkit VisualElements
— a virtual-DOM pattern.

## Client Architecture

The Unity client uses a service locator pattern rooted in Registry.cs, which
holds serialized references to all services and layout objects. Initialization
is multi-phase: Displayables, then Services, then SceneElements, then a
frame-delayed start callback. Key services: ActionServiceImpl handles Rust
communication, CardService manages card GameObjects, InputService dispatches
mouse/touch events to Displayable objects.

Gameplay currently runs in a single Unity scene. Most game entities are
prefab-backed MonoBehaviour components, with
client/Assets/Dreamtides/Components/Card.cs as the primary runtime card view.

ActionServiceImpl supports two backends: native FFI via Plugin.cs (DllImport
with JSON over a 10MB byte buffer) and HTTP to the dev server. In production,
`Plugin.HasPendingUpdates()` is checked every frame as a cheap gate before
polling. Commands arriving while processing are queued and replayed
sequentially.

Game entities extend Displayable (Layout/Displayable.cs), which provides
lifecycle hooks and mouse event handling (down/drag/up/hover). Cards have two
visual modes: sprite-based for hand/stack and a 3D battlefield representation,
toggled by GameContext. Cards and other objects are positioned via ObjectLayout
subclasses (CurveObjectLayout, PileObjectLayout, ScrollableUserHandLayout, etc.)
which compute positions and animate transitions via DOTween.

Dreamtides uses multiple UI surfaces in parallel: world-space 3D objects, legacy
UGUI canvas content, and Rust-driven UIToolkit overlays through Masonry.

On each UpdateBattle command, ActionServiceImpl updates status displays, action
buttons, screen overlay, and other UI, then delegates to CardService for card
management. CardService diffs the card list against existing Card GameObjects,
assigns each to the correct ObjectLayout based on its Position, then calls
ApplyAllLayouts to animate everything into place. BattleLayout.cs holds
references to all zone layouts (hands, battlefields, decks, voids, stacks,
browser, dreamwell, etc.).

Most movement animations come from automatic ObjectLayout transitions between
snapshots. For bespoke sequences, CardAnimationService coordinates explicit
DOTween timelines (client/Assets/Dreamtides/Services/CardAnimationService.cs).

## AI System

The AI uses Monte Carlo Tree Search with UCT, implemented in `ai_uct`. Each
candidate action gets its own search tree, evaluated in parallel via rayon.
Opponent hidden information is randomized each iteration. The default policy
runs random rollouts to terminal states (heuristic evaluation performed worse).
Iteration counts scale dynamically — first main-phase action gets 1.5x budget,
prompt responses get 0.5x.

## Testing

Tests live in rules_engine/tests/ (never inline mod tests). TestBattle builds
battle configurations; TestSession drives actions through the full engine
pipeline. Tests operate at the _simulated user interface_ level —
`perform_user_action()` exercises the complete path from action dispatch through
rendering. Both user and enemy client views are validated for consistency. The
StateProvider trait (rules_engine/src/state_provider/) is the dependency
injection boundary — production uses file-backed state, tests use in-memory
state.

Client tests also run in Unity editor mode under
client/Assets/Dreamtides/Tests/. New UI behavior should include targeted editor
tests when practical.

## Build and Tooling

All development commands go through `just` (not `cargo` directly):

- `just dev` — run HTTP dev server for Unity development
- `just fmt` — runs style_validator --fix, rlf-fmt, and cargo +nightly fmt
- `just check` / `just clippy` — type checking and linting
- `just test` — all tests (runs tabula-check first for stale generated files)
- `just battle-test <NAME>` — run a specific battle test
- `just review` — the pre-push validation gate: build, clippy, style-validator,
  rlf-lint, all tests, tv checks. Takes ~5 minutes; has scope-aware skipping.
- `just schema` — regenerate C# types from Rust (requires quicktype)
- `just tabula-generate` — regenerate parsed_abilities.json and test_card.rs
- `just tv-dev` — launch the Tauri-based TOML viewer app (separate workspace)

The style_validator binary enforces code ordering rules. rlf_lint/rlf_fmt
validate and format RLF strings. Nightly Rust toolchain is required for fmt.
