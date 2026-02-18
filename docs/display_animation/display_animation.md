# Display & Animation Patterns

How game state changes become visual sequences on the client. Covers the
animation recording mechanism in mutations, the rendering pipeline that converts
recorded events to commands, the command protocol structure, BattleView
assembly, and client-side command processing.

## Table of Contents

- [Animation Recording](#animation-recording)
- [The BattleAnimation Enum](#the-battleanimation-enum)
- [The Rendering Pipeline](#the-rendering-pipeline)
- [ResponseBuilder API](#responsebuilder-api)
- [CommandSequence and Command Variants](#commandsequence-and-command-variants)
- [BattleView Assembly](#battleview-assembly)
- [React-Style Snapshots vs Imperative Commands](#react-style-snapshots-vs-imperative-commands)
- [Client-Side Command Processing](#client-side-command-processing)
- [Adding a New Animation](#adding-a-new-animation)

## Animation Recording

During battle mutations, the rules engine records animation events alongside
full game state snapshots. The `push_animation()` method on BattleState
(battle_state/src/battle/battle_state.rs) is the core recording mechanism.

Each call to `push_animation()` captures an AnimationStep containing three
things: the BattleAnimation variant describing what happened, the EffectSource
identifying who caused it, and a complete clone of the BattleState at that
moment. This snapshot-per-step approach means the renderer can later reconstruct
exact intermediate game states for each event.

The method takes a closure for the BattleAnimation value rather than a direct
value. This is a performance optimization: the `animations` field on
BattleState is an `Option<AnimationData>`, and when it is `None` (during AI
simulation or testing), the closure never executes and no snapshot is created.
When animation recording is enabled, the method clones the full state, invokes
the closure, and pushes the resulting AnimationStep onto the animation timeline.

Some mutations use a `ShouldAnimate` enum parameter for finer control, allowing
callers to suppress duplicate animations when the same mutation is invoked
multiple times in a single resolution chain.

Animation recording call sites span roughly 20 files across the battle_mutations
crate, covering: card play and resolution, energy gain, spark gain, point
scoring, card drawing, void shuffling, dissolve and counterspell effects, target
selection, modal choices, prompt responses, turn starts, judgment, dreamwell
activation, trigger state changes, and activated abilities.

## The BattleAnimation Enum

The BattleAnimation enum (battle_state/src/battle/battle_animation_data.rs) has
approximately 20 variants representing distinct game events:

- **ActivatedAbility**: A player activates a character's ability, carrying the
  player and ability ID.
- **ApplyTargetedEffect**: A targeted effect (dissolve, counterspell,
  return-to-hand) applies to specific cards.
- **DrawCards**: A player draws one or more cards from their deck.
- **DreamwellActivation**: A card is drawn from the shared dreamwell.
- **GainEnergy**: A player gains energy, with source tracking.
- **GainSpark**: A character gains spark tokens.
- **Judgment**: The judgment phase resolves, carrying the player and new score.
- **MakeChoice**: A player makes a choice from a prompt.
- **PlayCard**: A card is initially played (before it hits the stack).
- **PlayedCard**: A card transitions onto the stack after being played.
- **PreventedEffect**: A targeted effect is prevented (countered).
- **PutCardsFromDeckIntoVoid**: The foresee effect sends cards to the void.
- **ResolveCharacter**: A character resolves from the stack to the battlefield.
- **ScorePoints**: A player scores victory points.
- **SelectModalEffectChoice**: A player picks a branch for a modal effect.
- **SelectedTargetsForCard**: A player confirms targets for a card's effect.
- **SetActiveTriggers**: The set of triggers awaiting resolution changes.
- **ShuffleVoidIntoDeck**: Void cards are shuffled back into the deck.
- **StartTurn**: A new turn begins.

A companion TriggerAnimation struct tracks which triggers are currently active,
storing the trigger's controller, character ID, and ability number for display.

## The Rendering Pipeline

The display crate has two rendering entry points
(display/src/rendering/renderer.rs):

**`renderer::connect()`** produces a single full-state snapshot without
animations. It creates a ResponseBuilder, delegates to `battle_rendering::run()`
to generate the complete BattleView, and returns a CommandSequence. Used when a
player first connects or reconnects to a battle.

**`renderer::render_updates()`** replays recorded animations and then appends a
final snapshot. The flow:

1. Creates a ResponseBuilder with animation enabled and sets `for_animation` to
   true.
2. Iterates through all AnimationSteps stored on the BattleState.
3. For each step, calls `animations::render()`, which pattern-matches on the
   BattleAnimation variant and emits appropriate commands.
4. Stops processing if a GameOver status is detected (later steps are ignored).
5. Resets `for_animation` to false.
6. Calls `battle_rendering::run()` to generate the final state snapshot.
7. Returns the complete CommandSequence with animation commands followed by the
   final snapshot.

The `animations::render()` function (display/src/rendering/animations.rs) uses
several rendering patterns depending on the animation variant:

- **Snapshot-based**: Variants like ActivatedAbility, DreamwellActivation,
  Judgment, and SetActiveTriggers push an intermediate BattleView snapshot so
  the player sees the state at the moment the event occurred. Some add Wait
  commands for viewing time.
- **Card movement**: DrawCards and PutCardsFromDeckIntoVoid construct card view
  data and emit MoveCardsWithCustomAnimation commands with stagger intervals.
- **Projectile-based**: GainEnergy and ScorePoints resolve the EffectSource to a
  game object ID and emit FireProjectile commands with sound and travel time.
- **Effect display**: GainSpark uses a DisplayEffect command with a visual asset
  and duration.
- **Message-based**: MakeChoice and SelectModalEffectChoice show opponent actions
  through DisplayEnemyMessage commands.
- **Conditional**: PlayedCard checks whether the card still exists in the final
  state; if an opponent's card resolved away quickly, a pause is inserted so the
  player can see it.

Each animation variant receives three pieces of state: the snapshot at the
moment the event occurred, the EffectSource, and the actual final game state for
comparison.

## ResponseBuilder API

The ResponseBuilder (display/src/core/response_builder.rs) accumulates commands
during rendering. It tracks which player the response is for, whether animations
are enabled, and maintains a list of pending commands.

Key methods:

- **push(command)**: Adds a single command as its own sequential group.
- **push_battle_view(view)**: Adds an UpdateBattle command carrying the
  BattleView, and attaches all pending commands to execute in parallel with this
  update. Clears the pending queue afterward. This is the primary synchronization
  point between state snapshots and their accompanying visual effects.
- **run_with_next_battle_view(command)**: Queues a command to fire in parallel
  with the next battle view push. Typically called multiple times to accumulate
  projectiles, dissolves, or audio before pushing the snapshot they accompany.
- **is_for_animation()**: Returns whether the current rendering pass is
  producing intermediate animation frames (true during animation replay) or the
  final stable state (false). Controls behavior like arrow suppression and
  preview generation.
- **set_for_animation()**: Toggles the animation flag between passes.
- **should_animate()**: Checks whether the builder was created with animations
  enabled.
- **commands()**: Consumes the builder and returns the finished CommandSequence.

The builder also manages display state per player through a thread-safe
provider, tracks active triggers for display, and provides access to the card
data via Tabula.

## CommandSequence and Command Variants

### Structure

A CommandSequence is a list of ParallelCommandGroups. Groups execute
sequentially (the client waits for one group to finish before starting the
next). Within each group, all commands execute simultaneously.

Helper constructors on CommandSequence:

- `from_command()`: Wraps a single command in its own group.
- `sequential()`: Each command in its own group (one after another).
- `parallel()`: All commands in one group (simultaneous execution).
- `from_vecs()`: Promotes a nested vector structure where each inner vector
  becomes a parallel group.

### Command Variants

The Command enum has 18 variants organized by purpose:

**State updates:**
- **UpdateBattle**: Carries a complete BattleView plus an optional sound. The
  primary command; each instance is a full UI reconstruction point.
- **UpdateQuest**: Updates the quest/overworld view with optional sound.

**Timing:**
- **Wait**: A pure delay for pacing between animation stages.

**Projectiles and effects:**
- **FireProjectile**: Launches a visual projectile between two game objects with
  configurable travel time, sounds, hit effects, and optional target
  repositioning.
- **DisplayEffect**: Shows a visual effect at a target location with duration,
  scale, and optional sound.

**Card operations:**
- **DissolveCard**: Fades a card out (or back in if reversed) using a
  shader-based dissolve effect with configurable color, speed, sound, and delay.
- **MoveCardsWithCustomAnimation**: Moves multiple cards with animation type
  selection (default, drawn-cards position, shop layout, etc.), stagger timing,
  and optional card trails.
- **ShuffleVoidIntoDeck**: Animates the discard pile shuffling back into the
  deck.

**Audio:**
- **PlayAudioClip**: Plays a sound with a pause duration before continuing.

**Messages:**
- **DisplayGameMessage**: Shows UI text like "Your Turn", "Victory", "Defeat".
- **DisplayEnemyMessage**: Shows an opponent status message with duration.
- **DisplayJudgment**: Shows judgment animation with optional new score.

**Special animations:**
- **DisplayDreamwellActivation**: Animates a dreamwell card activation with
  energy updates.
- **PlayStudioAnimation**: Plays cinematic enter/exit sequences for identity
  card reveals.
- **PlayMecanimAnimation**: Triggers Mecanim state machine animations at quest
  locations.

**Visual customization:**
- **SetCardTrail**: Adds a particle trail effect to cards for a duration.
- **UpdateScreenOverlay**: Sets or clears a flexible-node UI overlay.
- **AnchorToScreenPosition**: Anchors a UI element to a screen position with
  optional fade-out.

## BattleView Assembly

The BattleView struct (display_data/src/battle_view.rs) is the complete visual
representation of a battle. It is assembled by `battle_view()` in
battle_rendering.rs from five components: player status, cards, interface
overlay, targeting arrows, and preview state.

### Card Collection

Cards are gathered from multiple sources in a specific order:

- **Zone cards**: All cards in the game state (hand, deck, battlefield, stack,
  void, banished) are converted to CardView objects via card_rendering. Each
  card's visibility is determined by which players can see it (face-up vs
  face-down). Position is calculated based on the viewing player's perspective
  (the user always appears at the bottom).
- **Modal effect cards**: When a modal choice is active, temporary CardView
  objects represent each selectable branch.
- **Stack tokens**: Activated abilities on the stack appear as token cards.
- **Trigger tokens**: Active triggers awaiting resolution appear as token cards,
  positioned based on battlefield visibility.
- **Activated ability tokens**: Characters with usable abilities display those
  abilities as token cards in a dedicated area, showing only multi-use abilities
  or unused single-use abilities.
- **Void tokens**: Cards in the void appear as tokens when relevant.
- **Dreamwell cards**: All dreamwell zone cards, with special positioning for
  cards currently being activated.
- **Identity cards**: Both players' Dreamcaller identity cards always appear in
  the player status area.

Each CardView carries its image, name, cost, spark, type, rules text, available
actions, outline color (green for playable, yellow for selected, white for valid
targets, red for enemies), and optional create/destroy positions for entry/exit
animations.

### Interface View

The InterfaceView is built by interface_rendering and contains action buttons
(primary, secondary, increment/decrement for energy selection, undo, dev), a
screen overlay FlexNode for prompts and panels, and card browser/order selector
state. Buttons are enabled only when legal actions support them.

### Player Views

Each PlayerView carries score, energy, produced energy, total spark, turn
indicator (with phase information), whether the player can act, and whether
victory is imminent.

### Targeting Arrows

The arrow system builds DisplayArrow objects for cards resolving on the stack.
For each stack item, the system queries its displayed targets and draws arrows
from source to target. Arrow colors encode relationships: green for friendly
targets, red for enemy targets, blue for stack-to-stack targeting, and green for
void targets. Arrows are suppressed during animation replay to avoid clutter.

### Battle Preview

The preview system shows expected outcomes when a player is about to confirm a
choice. It has three states: None (clear preview), Pending (hold previous
preview during animations), and Active (show simulated result of confirming the
current prompt).

## React-Style Snapshots vs Imperative Commands

The display system follows a "react-style" philosophy: each UpdateBattle command
carries a complete BattleView, and the client reconstructs its entire UI from
that snapshot rather than applying incremental deltas. This approach means the
client never accumulates drift from missed updates, and any single snapshot is
sufficient to render the full battle state.

This works well for state that changes between turns: card positions, player
stats, interface controls, and targeting arrows all benefit from full-state
reconstruction. The client's CardService diffs the incoming card list against
existing Card GameObjects, creating, updating, or removing cards as needed, then
assigns each to the correct ObjectLayout for positioning.

However, some visual feedback requires imperative commands that cannot be
expressed as state snapshots:

- **Projectile animations** (FireProjectile) need source and target with travel
  time and hit effects.
- **Dissolve effects** (DissolveCard) need shader-driven fade-out/in
  transitions.
- **Audio cues** (PlayAudioClip) must play at specific moments.
- **Transient messages** (DisplayGameMessage, DisplayEnemyMessage) appear
  briefly and disappear.
- **Custom card movement** (MoveCardsWithCustomAnimation) needs stagger timing
  and intermediate positions.

These imperative commands are synchronized with snapshots through the
ResponseBuilder's pending command mechanism: effect commands are queued via
`run_with_next_battle_view()` and then attached to the next `push_battle_view()`
call, ensuring they execute in parallel with their corresponding state update.

## Client-Side Command Processing

### ActionServiceImpl

ActionServiceImpl is the Unity entry point for command processing. It receives
CommandSequence objects from the Rust backend through Connect, PerformAction,
and Poll responses. A command queue serializes processing: if commands arrive
while others are executing, they are enqueued and replayed after the current
batch completes.

The `ApplyCommands()` method iterates through each ParallelCommandGroup
sequentially, launching all commands within a group as coroutines that run
simultaneously, then waiting for all coroutines in that group to complete before
advancing to the next group.

For UpdateBattle commands, ActionServiceImpl directly updates player status
displays, screen overlays, UI buttons, and other interface state, then delegates
card management to CardService.

### CardService Diffing

CardService maintains a dictionary of active Card GameObjects keyed by client
ID. When an UpdateBattle arrives, it diffs the incoming CardView list against
the dictionary:

- Cards with new IDs are instantiated from the appropriate prefab type.
- Existing cards are updated with new visual data.
- Cards no longer present are queued for deletion.

Each card is assigned to the correct ObjectLayout based on its position
(hand, battlefield, deck, void, stack, etc.), and `ApplyAllLayouts()` triggers
position calculation and animation for every layout.

### ObjectLayout System

ObjectLayout subclasses compute card positions within each zone.
StandardObjectLayout is the base for most implementations, providing sorted
object lists and abstract position/rotation/scale calculation methods. Concrete
implementations include PileObjectLayout (stacked deck/void piles),
RectangularObjectLayout (grid displays), UserHandLayout (curved hand fan), and
CenteredObjectLayout (single centered position).

When applying layouts, if a DOTween sequence is provided, cards animate to their
target positions. All layout animations use `Insert(0, ...)` on the sequence,
starting simultaneously within their group. Without a sequence, cards snap to
position immediately.

### VFX Command Handlers

Each VFX command type has its own handler:

- **FireProjectile**: Retrieves source and target objects, adds an anticipatory
  jump animation to the source, creates a projectile that travels to the target
  with hit effects and sound.
- **DissolveCard**: Applies a dissolve shader to all renderers and fades text,
  with optional reversal for undo-like effects.
- **DisplayEffect**: Instantiates a visual effect prefab at the target's display
  position with duration and sound.
- **MoveCardsWithCustomAnimation**: CardAnimationService handles several
  movement patterns including default moves with stagger, drawn-card animation
  (deck to center to hand), and shuffle-void-into-deck (a multi-phase animation
  with dynamic per-card timing).

### DOTween Integration

All client animations use DOTween for tweening. Standard durations are 0.3
seconds for card movement and 0.4 seconds for card flips. Named sequences aid
debugging. Animations from multiple sources (layout updates, custom handlers,
VFX) are composed into sequences using timeline positioning, enabling
deterministic parallel and sequential execution.

Key files: display/src/rendering/renderer.rs,
display/src/rendering/animations.rs, display/src/rendering/battle_rendering.rs,
display/src/core/response_builder.rs, display_data/src/command.rs,
display_data/src/battle_view.rs, battle_state/src/battle/battle_animation_data.rs.

## Adding a New Animation

To add a new visual animation for a game event:

1. **Add a BattleAnimation variant** to the enum in
   battle_state/src/battle/battle_animation_data.rs with whatever data the
   animation needs (player, card IDs, amounts, etc.).

2. **Record the animation in mutation code** by calling
   `battle.push_animation(source, || BattleAnimation::NewVariant { ... })` at
   the appropriate point in the mutation. The closure should construct the
   variant using current state. Place the call after the state change so the
   snapshot reflects the post-mutation state.

3. **Handle the variant in animations::render()** in
   display/src/rendering/animations.rs. Pattern-match the new variant and emit
   commands. Common patterns: push an intermediate snapshot via
   `push_snapshot()`, emit FireProjectile or DisplayEffect for VFX, add Wait for
   pacing, or emit DisplayEnemyMessage for opponent feedback.

4. **Add client-side handling** if the animation uses a new Command variant or
   requires custom animation logic. For existing command types (projectiles,
   dissolves, effects), no client changes are needed. For new Command variants,
   add handling in ActionServiceImpl and any required animation support in
   CardAnimationService.
