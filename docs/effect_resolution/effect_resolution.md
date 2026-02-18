# Effect Resolution & Trigger System

How effects resolve, triggers fire, and prompts interleave during battle
execution. This is the core runtime control flow that determines the order in
which game state changes happen after every player action.

## Table of Contents

- [Post-Action Cleanup Cascade](#post-action-cleanup-cascade)
- [Pending Effects Queue](#pending-effects-queue)
- [Effect Variants and Decomposition](#effect-variants-and-decomposition)
- [Prompt Interleaving](#prompt-interleaving)
- [Prompt Types](#prompt-types)
- [OnSelected: Linking Prompt Responses to Effects](#onselected-linking-prompt-responses-to-effects)
- [Trigger Lifecycle](#trigger-lifecycle)
- [Trigger Matching](#trigger-matching)
- [Trigger Chaining](#trigger-chaining)
- [EffectSource Tracking](#effectsource-tracking)
- [Turn State Machine Integration](#turn-state-machine-integration)
- [Auto-Executed Actions](#auto-executed-actions)
- [Adding a New Triggered Effect](#adding-a-new-triggered-effect)
- [Key Files](#key-files)

## Post-Action Cleanup Cascade

After every battle action is dispatched through `apply_battle_action`, three
cleanup functions run in strict sequence:

- **Pass 1: Drain pending effects.** Pops effects from the front of the
  `pending_effects` VecDeque and applies them one at a time. Stops if a prompt
  is created, the game ends, or the queue empties.
- **Pass 2: Fire triggers.** Pops trigger events from the front of the
  `triggers.events` VecDeque, checks each listener card is still on the
  battlefield, matches against triggered abilities, and fires matching effects.
  Stops on the same conditions as pass 1.
- **Pass 3: Advance the turn state machine.** Loops through turn phase
  transitions (judgment, dreamwell, draw, etc.). Crucially, each phase
  transition re-invokes passes 1 and 2 internally before advancing to the next
  phase. Exits when it reaches Main or Ending phase, or if a prompt is created.

The ordering matters: direct effects resolve before triggers, and both resolve
before the turn advances. If pass 1 creates a prompt, passes 2 and 3 are
effectively skipped because they check `prompts.is_empty()` and bail
immediately. The cascade resumes on the player's next action.

## Pending Effects Queue

The `pending_effects` field on BattleState is a `VecDeque<PendingEffect>`. Each
PendingEffect has four fields:

- **source** (EffectSource): What caused this effect -- a game rule, player
  action, event card, triggered ability, etc. Used to determine who the
  "controller" is for effects that reference "your" vs "enemy."
- **effect** (Effect): The effect to apply, which can be a single
  StandardEffect, a list of effects, or a modal choice.
- **requested_targets** (optional EffectTargets): Pre-selected targets, either a
  single StandardEffectTarget (Character, StackCard, or VoidCardSet) or a queue
  of targets consumed in order during list effect resolution.
- **modal_choice** (optional ModalEffectChoiceIndex): Which mode was selected
  for modal effects.

The drain loop in `apply_effect::execute_pending_effects_if_no_active_prompt`
pops effects from the front (FIFO order). On each iteration it checks three
conditions before proceeding: prompts must be empty, the game must not be over,
and the queue must be non-empty. If any check fails, the loop returns.

## Effect Variants and Decomposition

The Effect enum has five variants that determine how the drain loop processes
each entry:

- **Effect(StandardEffect)**: A single atomic effect. Applied directly via
  `apply_standard_effect::apply`. StandardEffect is a large enum with
  approximately 70 variants covering all leaf mutations (DrawCards,
  DissolveCharacter, GainEnergy, Foresee, Kindle, Counterspell, etc.).

- **WithOptions(EffectWithOptions)**: A single StandardEffect wrapped with
  metadata: an `optional` flag for "you may" effects, an optional `trigger_cost`
  for costs paid on resolution, and an optional `condition` for "if" guards.
  After checking these flags, delegates to the same `apply_standard_effect`
  path.

- **List(Vec\<EffectWithOptions>)**: A sequential list of effects. The drain
  loop removes the first element, applies it immediately, and pushes the
  remainder back to the **front** of the VecDeque via `push_front`. This ensures
  the remaining effects process next, before any other pending effects already
  in the queue. A list like [A, B, C] decomposes as: execute A, push [B, C] to
  front; next iteration: execute B, push [C] to front; next: execute C, done.
  Between each step the loop re-checks for prompts, so if A creates a prompt,
  [B, C] waits at the front until the prompt resolves.

- **ListWithOptions(ListWithOptions)**: Like List, but with shared options
  (trigger_cost, condition) that apply to the entire sequence rather than
  individual elements. Decomposition is identical after checking the shared
  options.

- **Modal(Vec\<ModalEffectChoice>)**: A choice among multiple effects. Each
  ModalEffectChoice carries an energy cost and an inner Effect (which can itself
  be any variant). Requires a `modal_choice` index to be present on the
  PendingEffect. The chosen alternative's inner Effect is executed recursively.
  If no choice is present, the system panics -- modal effects must have their
  choice resolved via a ModalEffect prompt before reaching the drain loop.

## Prompt Interleaving

Prompts are the universal "pause" mechanism. The `prompts` field on BattleState
is a `VecDeque<PromptData>` where the front element is the currently active
prompt. All three cleanup loops (pending effects, triggers, turn state machine)
check `battle.prompts.is_empty()` at the top of every iteration and exit
immediately if any prompt exists.

Prompts are created during effect execution in several ways:

- Effects that need target selection (character, stack card, void card): the
  system checks whether targets can be automatically determined. If not, it
  creates a prompt with the valid target set and pushes the effect (without
  targets) onto the pending_effects queue. The effect waits there until the
  player responds.
- Foresee effects push a SelectDeckCardOrder prompt for the player to reorder
  revealed cards.
- Discard effects push a ChooseHandCards prompt when the player has more cards
  than required.
- Counterspell-unless-pays effects push a Choose prompt giving the opponent the
  option to pay or let their card be countered.
- Modal effects push a ModalEffect prompt for the player to pick a mode.

When the player responds to a prompt, the corresponding BattleAction handler
pops the prompt from the front of the deque and writes the selection back into
the appropriate target (a stack item or pending effect, determined by the
OnSelected field). Then the standard three-pass cascade runs again, finds
prompts empty, and resumes processing.

## Prompt Types

PromptData has four fields: source (EffectSource), player (PlayerName),
prompt_type (PromptType), and configuration (containing an `optional` boolean).

The nine PromptType variants:

- **ChooseCharacter**: Select a character on the battlefield. Contains an
  OnSelected and a CardSet of valid targets. Created when an effect has a
  character target predicate.
- **ChooseStackCard**: Select a card on the stack. Same structure as
  ChooseCharacter but for stack cards. Used by counterspell-type effects.
- **ChooseVoidCard**: Select one or more cards from the void. Multi-select with
  toggle-and-submit interaction. Contains valid cards, currently selected cards,
  and maximum selection count.
- **ChooseHandCards**: Select cards from hand (e.g., for discard). Multi-select
  with toggle-and-submit. Tracks the hand card effect type (currently only
  Discard).
- **Choose**: General pick-one-from-a-list prompt. Each choice has a label, an
  effect to apply, and optional targets. Used for "counterspell unless pays
  cost" and similar binary decisions.
- **ChooseEnergyValue**: Pick an energy amount between a minimum and maximum.
  Used for variable additional costs like "spend one or more energy."
- **ModalEffect**: Choose one of several modal effect alternatives. Each
  alternative has an energy cost and an Effect.
- **ChooseActivatedAbility**: Pick which activated ability to use when a
  character has multiple. Each option shows the ability name and cost.
- **SelectDeckCardOrder**: The Foresee prompt. Reveals top N cards of the deck
  for reordering and optional voiding. Multi-step: the player positions each
  card individually, then submits.

## OnSelected: Linking Prompt Responses to Effects

The OnSelected enum on prompt types determines where the player's selection gets
written back:

- **AddStackTargets(StackItemId)**: Used during the "play card" flow when
  targets are chosen before the card resolves. The selection is appended to the
  stack item's targets list.
- **AddPendingEffectTarget(PendingEffectIndex)**: Used during the
  cleanup/trigger pipeline when an effect is already in the pending_effects
  queue and needs targeting information filled in. The selection is written into
  the pending effect's `requested_targets` field.

The distinction maps to two timing windows: AddStackTargets is for upfront
targeting when a card is played, while AddPendingEffectTarget is for
at-resolution targeting during triggered abilities and "if you do" effects.

During prompt creation, the PendingEffectIndex is calculated as the current
length of the pending_effects queue (the index where the effect is about to be
pushed). When the player responds, the handler uses this index to locate the
correct pending effect and write the target back.

## Trigger Lifecycle

Triggers follow a registration-matching-firing lifecycle.

**Registration**: When a card moves to the battlefield via `move_card`, the
`on_enter_battlefield` function reads the card's AbilityList and iterates over
the precomputed `battlefield_triggers` EnumSet (a bitset of TriggerName values).
For each trigger type, it calls
`battle.triggers.listeners.add_listener(trigger, card_id)`. Stack triggers are
handled similarly via `on_enter_stack` (currently only used for
PlayedCardFromVoid). Registration and deregistration use the same EnumSet,
keeping them automatically symmetric.

**Deregistration**: When a card leaves the battlefield or stack, the
corresponding `on_leave_battlefield` or `on_leave_stack` function removes
listeners for the same set of trigger names. If the card has "banish when leaves
play," the destination zone is overridden to Banished.

**Event queuing**: When a game event occurs (card dissolved, character
materialized, energy gained, etc.), the mutation code calls
`battle.triggers.push(source, trigger)`. This method looks up the TriggerName
for the event, iterates over all cards currently listening for that name, and
creates one TriggerForListener entry per listener card, each with the same
source and trigger but a different listener CardId. All entries are appended to
the back of the events VecDeque.

**Firing**: The `fire_triggers` loop pops entries from the front of the events
queue. For each entry, it first verifies the listener card is still on the
battlefield (it may have been dissolved since the trigger was queued). If so, it
iterates over the card's triggered abilities and calls
`trigger_queries::matches` to check if the queued trigger matches the ability's
TriggerEvent condition. On match, it constructs an EffectSource::Triggered and
delegates to `apply_effect_with_prompt_for_targets::execute`.

**TriggerState** on BattleState contains two fields: `listeners`
(TriggerListeners, which tracks per-trigger-name CardSets of listening cards)
and `events` (VecDeque of TriggerForListener entries).

**TriggerForListener** is a struct with three fields: the EffectSource that
caused the trigger, the listener CardId, and the Trigger event data.

## Trigger Matching

There are two parallel type hierarchies for triggers:

- **Trigger** (runtime): 14 variants representing events that actually happened.
  Each carries contextual data -- for example, Dissolved carries a VoidCardId,
  Materialized carries a CharacterId, GainedEnergy carries a PlayerName and
  Energy amount.

- **TriggerEvent** (ability definition): 18 variants representing conditions
  that card abilities can listen for. Many carry a Predicate for filtering which
  cards match. Some are todo stubs (AbandonCardsInTurn, DrawCardsInTurn,
  PlayCardsInTurn, MaterializeNthThisTurn).

The mapping between them is N-to-M. For example, TriggerEvent::LeavesPlay
matches both Trigger::Banished and Trigger::Dissolved, since both represent a
character leaving play. A single Trigger::PlayedCard can match
TriggerEvent::Play, TriggerEvent::PlayDuringTurn, and
TriggerEvent::OpponentPlays.

The `trigger_queries::matches` function takes the actual Trigger, the ability's
TriggerEvent, the owning card's controller, and the owning card's ID. It uses a
large match on TriggerEvent:

- **Predicate-based events** (Abandon, Banished, Discard, Dissolved, LeavesPlay,
  PutIntoVoid, Materialize, Play, PlayFromHand, OpponentPlays, PlayDuringTurn):
  First verify the Trigger variant matches the event type, then delegate to
  `trigger_predicates::trigger_matches` to evaluate the predicate against the
  triggering card.

- **Player-scoped events** (DrawAllCardsInCopyOfDeck, EndOfYourTurn,
  GainEnergy): Check that the trigger's player matches the owning card's
  controller.

- **Keyword events**: The Keywords variant holds a list of TriggerKeyword values
  (Materialized, Judgment, Dissolved). These are self-referential triggers like
  "when this character is materialized" (checking owning_card_id equals the
  triggering card) or "at judgment" (checking controller matches).

Predicate evaluation happens in `trigger_predicates::trigger_matches`, which
checks ownership-based Predicate variants (Enemy, Another, Your, Any, AnyOther)
and then delegates to `matches_card_predicate` for type-based CardPredicate
variants (Card, Character, Event, CharacterType, NotCharacterType).

## Trigger Chaining

The system achieves flat, non-recursive trigger chaining through queue design.
When `fire_triggers` pops a trigger and fires it, the resulting effect may
mutate game state (dissolve a character, materialize something, gain energy).
Those mutations call `battle.triggers.push()`, which appends new
TriggerForListener entries to the **back** of the same VecDeque. The loop then
continues popping from the front, processing newly-generated triggers after all
previously-queued ones, in strict FIFO order.

There is no recursion -- `fire_triggered_ability` does not call back into the
trigger loop. A single flat loop drains everything, including cascading
triggers. This prevents stack overflow from deeply chained triggers and ensures
deterministic ordering. Termination is guaranteed because the number of cards in
play is finite (max 128 per battle) and each trigger fires based on a state
change that can only happen a bounded number of times.

If a triggered effect creates a prompt (requires player input for targeting),
the loop breaks. The prompt is resolved by the player's next action, which
re-runs the three-pass cascade. The trigger loop picks up where it left off.

## EffectSource Tracking

Every effect carries an EffectSource that identifies what caused it. All seven
variants carry a `controller: PlayerName` field, and the `controller()` method
extracts it uniformly:

- **Game**: Effects from game rules (drawing for turn, judgment, start-of-turn).
  Controller is the active player.
- **Player**: Direct player actions (playing a card from hand or void, resolving
  a character to the battlefield). Controller is the acting player.
- **Dreamwell**: Dreamwell card activation. Also carries the
  `dreamwell_card_id`. Controller is the player whose dreamwell phase is
  running.
- **Event**: An event card's ability resolving from the stack. Carries
  `stack_card_id` and `ability_number`. Controller is the stack card's
  controller (set when the card was played).
- **Activated**: An activated ability of a character. Carries
  `activated_ability_id`. Created both when the ability is first activated (for
  costs) and when it resolves from the stack.
- **Triggered**: A triggered ability of a character. Carries `character_id` and
  `ability_number`. Controller is looked up dynamically via
  `card_properties::controller` at firing time.
- **IfYouDo**: The "if you do" clause of a static ability that grants
  play-from-void. Carries `ability_id`. Created when a card is played from void
  via such a static ability.

Controller propagation determines who is affected by effects that reference
"your" or "enemy." For example, DrawCards uses `source.controller()` to
determine who draws; Predicate::Your queries the battlefield of
`source.controller()`; Predicate::Enemy queries
`source.controller().opponent()`.

EffectSource also provides `card_id()` (returns the card associated with the
source, if any -- None for Game, Player, and Dreamwell) and `ability_number()`
(the specific ability on that card). These are used for "this card" references
and quantity expressions like "for each energy spent on this card."

## Turn State Machine Integration

The turn state machine in `turn::run_turn_state_machine_if_no_active_prompts`
loops through phase transitions while prompts are empty and the game is not
over. The BattleTurnPhase enum progresses: EndingPhaseFinished,
FiringEndOfTurnTriggers, Starting, Judgment, Dreamwell, Draw, Main, Ending.

Each phase transition runs its specific logic (award judgment points, activate
dreamwell, draw a card, etc.), then re-invokes both the pending effects drain
and trigger firing passes before iterating to the next phase. This ensures
effects and triggers from each phase fully resolve before the next phase begins.

The machine exits when it reaches Main phase (player takes actions freely) or
Ending phase (opponent gets a fast-action window). Between Ending and
EndingPhaseFinished, the opponent can play fast cards or activate abilities
before the turn fully transitions.

## Auto-Executed Actions

The outer action loop in `handle_battle_action::execute` wraps the three-pass
cascade with auto-execution logic. After applying an action and running cleanup,
it determines who acts next and computes their legal actions. If there is
exactly one legal action matching certain patterns, it executes automatically
without returning to the caller:

- PassPriority as the only option (nothing to respond with on the stack)
- StartNextTurn as the only option (no fast actions available)
- SelectPromptChoice with exactly one choice (forced single-option prompt)
- SelectCharacterTarget or SelectStackCardTarget with exactly one valid target

This allows chains of trivial decisions to resolve instantly. The loop only
exits when the game ends or a human player faces a non-trivial choice.

## Adding a New Triggered Effect

To add a new triggered effect:

- Add a new variant to the Trigger enum in battle_state/src/triggers/trigger.rs
  and a corresponding TriggerName variant for listener tracking.
- Add a matching TriggerEvent variant in ability_data/src/trigger_event.rs if
  the ability condition needs new semantics.
- Push the new Trigger at the appropriate mutation site (e.g., in move_card.rs,
  a phase mutation, or an effect application function) via
  `battle.triggers.push(source, Trigger::NewVariant(...))`.
- Handle the new TriggerEvent in `trigger_queries::matches` in
  battle_queries/src/card_ability_queries/trigger_queries.rs to define when the
  ability fires.
- Update `watch_for_battlefield_triggers` in
  battle_queries/src/battle_card_queries/card_abilities.rs so AbilityList knows
  to register listeners for the new trigger type.
- The triggered ability's effect goes through the existing
  `apply_effect_with_prompt_for_targets` path and needs no special handling
  unless it introduces a new StandardEffect.

## Key Files

- **battle_mutations/src/effects/apply_effect.rs**: Main pending effects drain
  loop, effect execution entry point, list decomposition logic.
- **battle_mutations/src/effects/apply_standard_effect.rs**: Dispatches
  StandardEffect variants to concrete mutation functions.
- **battle_mutations/src/effects/apply_effect_with_prompt_for_targets.rs**:
  Handles effects that need target prompts before execution. Used by triggered
  and if-you-do effects.
- **battle_mutations/src/phase_mutations/fire_triggers.rs**: Trigger firing loop
  with battlefield presence checks and ability matching.
- **battle_mutations/src/phase_mutations/turn.rs**: Turn state machine with
  per-phase drain/trigger passes.
- **battle_mutations/src/actions/apply_battle_action.rs**: Action dispatch and
  the three-pass cleanup cascade.
- **battle_state/src/battle/battle_state.rs**: BattleState struct with
  pending_effects, triggers, and prompts fields. PendingEffect struct.
- **battle_state/src/triggers/trigger.rs**: Trigger enum (runtime events) and
  TriggerName enum (bitset keys).
- **battle_state/src/triggers/trigger_state.rs**: TriggerState (listeners +
  event queue) and TriggerForListener struct.
- **battle_state/src/triggers/trigger_listeners.rs**: TriggerListeners with
  per-trigger-name CardSet tracking.
- **battle_state/src/prompt_types/prompt_data.rs**: PromptData, PromptType, and
  OnSelected definitions.
- **battle_state/src/core/effect_source.rs**: EffectSource enum and controller
  propagation.
- **ability_data/src/effect.rs**: Effect enum, EffectWithOptions,
  ListWithOptions, ModalEffectChoice.
- **ability_data/src/standard_effect.rs**: StandardEffect enum with ~70 atomic
  effect variants.
- **ability_data/src/trigger_event.rs**: TriggerEvent enum (ability conditions)
  and TriggerKeyword.
- **battle_queries/src/card_ability_queries/trigger_queries.rs**: Trigger
  matching logic and triggering_card_id extraction.
- **battle_queries/src/card_ability_queries/trigger_predicates.rs**: Predicate
  evaluation for trigger conditions.
- **battle_queries/src/card_ability_queries/effect_prompts.rs**: Determines what
  prompts an effect needs and creates them.
- **battle_mutations/src/card_mutations/move_card.rs**: Trigger
  registration/deregistration on zone transitions.
- **rules_engine/src/handle_battle_action.rs**: Top-level action loop with
  auto-execution and AI integration.
