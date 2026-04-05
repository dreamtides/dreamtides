# Dreamtides Combat Prototype Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add positional front/back rank combat to Dreamtides, replacing spark-comparison scoring with per-column Judgment resolution.

**Architecture:** Battlefield changes from `CardSet<CharacterId>` to a `Battlefield` struct with `[Option<CharacterId>; 8]` arrays for front/back ranks. New Judgment phase at end of turn resolves each column independently. Turn order becomes Dreamwell → Draw → Dawn → Main → Judgment. AI gets simplified repositioning actions to avoid MCTS branching explosion.

**Tech Stack:** Rust (rules_engine), TypeScript/React (battle prototype), TOML (card data)

**Spec:** `docs/superpowers/specs/2026-04-05-combat-prototype-design.md`

**Key constraint:** NO automated tests. All validation through manual QA using `agent-browser` CLI against the battle prototype. At least 15 milestones are dedicated QA passes.

**Build/run commands:**
- Format: `just fmt`
- Type check: `just check`
- Lint: `just clippy`
- Full gate: `just review` (run in foreground, ~5 min)
- Regenerate from TOML: `just tabula-generate`
- Regenerate C# types: `just schema`
- Start battle prototype dev server: `cd scripts/battle_prototype && npm run dev`
- Start Rust dev server: Check `scripts/battle_prototype/scripts/` for dev server launch script

**Style rules (from CLAUDE.md):**
- Function calls: exactly ONE qualifier (e.g., `move_card::to_destination_zone()`)
- Struct/enum type names: ZERO qualifiers (e.g., `BattleState`, not `battle_state::BattleState`)
- Enum values: ONE qualifier (e.g., `Zone::Battlefield`)
- Imports: `crate::` not `super::`, all at file top, never inside function bodies
- No `pub use`, no direct function imports
- Only module declarations in `mod.rs`/`lib.rs`
- Prefer inline expressions over `let` bindings
- Short doc comments on public items only

---

## Task 1: Add Vanilla Characters to Card Pool

**Files:**
- Modify: `client/Assets/StreamingAssets/Tabula/cards.toml` (append after line 3802)
- Modify: `client/Assets/StreamingAssets/Tabula/card-lists.toml` (insert before line 157)

- [ ] **Step 1: Add 6 vanilla character cards to cards.toml**

Append these cards at the end of the `[[cards]]` entries in `cards.toml` (before the `[metadata]` section if there is one, otherwise at the end). Use card numbers 223-228. Generate unique UUIDs for each.

```toml
[[cards]]
name = "Duskborne Sentry"
id = "a1b2c3d4-1111-4000-8000-000000000001"
energy-cost = 1
rules-text = ""
card-type = "Character"
subtype = ""
is-fast = false
spark = 1
image-number = 0
rarity = "Common"
art-owned = false
card-number = 223
variables = ""

[[cards]]
name = "Glimmer Scout"
id = "a1b2c3d4-2222-4000-8000-000000000002"
energy-cost = 2
rules-text = ""
card-type = "Character"
subtype = ""
is-fast = false
spark = 2
image-number = 0
rarity = "Common"
art-owned = false
card-number = 224
variables = ""

[[cards]]
name = "Veilward Knight"
id = "a1b2c3d4-3333-4000-8000-000000000003"
energy-cost = 3
rules-text = ""
card-type = "Character"
subtype = ""
is-fast = false
spark = 3
image-number = 0
rarity = "Common"
art-owned = false
card-number = 225
variables = ""

[[cards]]
name = "Embertide Warrior"
id = "a1b2c3d4-4444-4000-8000-000000000004"
energy-cost = 4
rules-text = ""
card-type = "Character"
subtype = ""
is-fast = false
spark = 5
image-number = 0
rarity = "Uncommon"
art-owned = false
card-number = 226
variables = ""

[[cards]]
name = "Starforged Titan"
id = "a1b2c3d4-5555-4000-8000-000000000005"
energy-cost = 5
rules-text = ""
card-type = "Character"
subtype = ""
is-fast = false
spark = 7
image-number = 0
rarity = "Uncommon"
art-owned = false
card-number = 227
variables = ""

[[cards]]
name = "Abyssal Colossus"
id = "a1b2c3d4-7777-4000-8000-000000000006"
energy-cost = 7
rules-text = ""
card-type = "Character"
subtype = ""
is-fast = false
spark = 10
image-number = 0
rarity = "Rare"
art-owned = false
card-number = 228
variables = ""
```

- [ ] **Step 2: Add vanilla characters to the Core 11 card list**

In `card-lists.toml`, insert these entries before the `[metadata]` line (line 157):

```toml
[[card-lists]]
list-name = "Core 11"
list-type = "BaseCardId"
card-id = "a1b2c3d4-1111-4000-8000-000000000001"
copies = 8

[[card-lists]]
list-name = "Core 11"
list-type = "BaseCardId"
card-id = "a1b2c3d4-2222-4000-8000-000000000002"
copies = 8

[[card-lists]]
list-name = "Core 11"
list-type = "BaseCardId"
card-id = "a1b2c3d4-3333-4000-8000-000000000003"
copies = 6

[[card-lists]]
list-name = "Core 11"
list-type = "BaseCardId"
card-id = "a1b2c3d4-4444-4000-8000-000000000004"
copies = 6

[[card-lists]]
list-name = "Core 11"
list-type = "BaseCardId"
card-id = "a1b2c3d4-5555-4000-8000-000000000005"
copies = 4

[[card-lists]]
list-name = "Core 11"
list-type = "BaseCardId"
card-id = "a1b2c3d4-7777-4000-8000-000000000006"
copies = 4
```

- [ ] **Step 3: Regenerate parsed data**

Run: `just tabula-generate`

- [ ] **Step 4: Commit**

```bash
git add client/Assets/StreamingAssets/Tabula/cards.toml client/Assets/StreamingAssets/Tabula/card-lists.toml
git commit -m "feat: add 6 vanilla characters to Core 11 for combat testing"
```

---

## Task 2: Battlefield Data Model

**Files:**
- Create: `rules_engine/src/battle_state/src/battle_cards/battlefield.rs`
- Modify: `rules_engine/src/battle_state/src/battle_cards/mod.rs` (add module declaration)
- Modify: `rules_engine/src/battle_state/src/battle/all_cards.rs` (replace battlefield storage)
- Modify: `rules_engine/src/battle_state/src/battle_cards/character_state.rs` (add played_turn)

- [ ] **Step 1: Create the Battlefield struct**

Create `rules_engine/src/battle_state/src/battle_cards/battlefield.rs`:

```rust
use core_data::types::CharacterId;
use serde::{Deserialize, Serialize};

/// Two-rank battlefield for a single player. Front rank characters
/// participate in Judgment; back rank characters are safe but inert.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Battlefield {
    pub front: [Option<CharacterId>; 8],
    pub back: [Option<CharacterId>; 8],
}

impl Default for Battlefield {
    fn default() -> Self {
        Self {
            front: [None; 8],
            back: [None; 8],
        }
    }
}

impl Battlefield {
    /// Returns the total number of characters across both ranks.
    pub fn character_count(&self) -> usize {
        self.front.iter().filter(|s| s.is_some()).count()
            + self.back.iter().filter(|s| s.is_some()).count()
    }

    /// Returns true if both ranks are completely full (16 characters).
    pub fn is_full(&self) -> bool {
        self.character_count() >= 16
    }

    /// Finds the first empty slot in the back rank. Returns the index.
    pub fn first_empty_back_slot(&self) -> Option<usize> {
        self.back.iter().position(|s| s.is_none())
    }

    /// Finds the first empty slot in the front rank. Returns the index.
    pub fn first_empty_front_slot(&self) -> Option<usize> {
        self.front.iter().position(|s| s.is_none())
    }

    /// Returns true if the given character is in any slot.
    pub fn contains(&self, character_id: CharacterId) -> bool {
        self.front.iter().any(|s| *s == Some(character_id))
            || self.back.iter().any(|s| *s == Some(character_id))
    }

    /// Removes a character from whatever slot it occupies. Returns true
    /// if the character was found and removed.
    pub fn remove(&mut self, character_id: CharacterId) -> bool {
        for slot in self.front.iter_mut().chain(self.back.iter_mut()) {
            if *slot == Some(character_id) {
                *slot = None;
                return true;
            }
        }
        false
    }

    /// Adds a character to the first available back rank slot.
    /// Panics if the back rank is full.
    pub fn add_to_back_rank(&mut self, character_id: CharacterId) -> usize {
        let idx = self
            .first_empty_back_slot()
            .expect("Back rank is full");
        self.back[idx] = Some(character_id);
        idx
    }

    /// Returns true if the character is in the front rank.
    pub fn is_in_front_rank(&self, character_id: CharacterId) -> bool {
        self.front.iter().any(|s| *s == Some(character_id))
    }

    /// Returns true if the character is in the back rank.
    pub fn is_in_back_rank(&self, character_id: CharacterId) -> bool {
        self.back.iter().any(|s| *s == Some(character_id))
    }

    /// Returns all CharacterIds on the battlefield (both ranks).
    pub fn all_characters(&self) -> Vec<CharacterId> {
        self.front
            .iter()
            .chain(self.back.iter())
            .filter_map(|s| *s)
            .collect()
    }
}
```

- [ ] **Step 2: Add module declaration**

In `rules_engine/src/battle_state/src/battle_cards/mod.rs`, add:

```rust
pub mod battlefield;
```

- [ ] **Step 3: Add played_turn to CharacterState**

Modify `rules_engine/src/battle_state/src/battle_cards/character_state.rs`:

```rust
use core_data::numerics::Spark;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CharacterState {
    pub spark: Spark,
    /// The turn number on which this character was played.
    /// Used for summoning sickness (cannot move to front rank on this turn).
    pub played_turn: u32,
}
```

- [ ] **Step 4: Replace battlefield storage in AllCards**

In `rules_engine/src/battle_state/src/battle/all_cards.rs`:

1. Add import: `use crate::battle_cards::battlefield::Battlefield;`
2. Replace the `battlefield: PlayerMap<CardSet<CharacterId>>` field (line ~44) with:
   ```rust
   battlefield: PlayerMap<Battlefield>,
   ```
3. Update the `battlefield()` method (lines 125-128) to return `&Battlefield`:
   ```rust
   pub fn battlefield(&self, player: PlayerName) -> &Battlefield {
       self.battlefield.player(player)
   }
   ```
4. Add a `battlefield_mut()` method:
   ```rust
   pub fn battlefield_mut(&mut self, player: PlayerName) -> &mut Battlefield {
       self.battlefield.player_mut(player)
   }
   ```
5. Update `all_battlefield_characters()` (lines 130-133) to use the new Battlefield:
   ```rust
   pub fn all_battlefield_characters(&self) -> Vec<CharacterId> {
       let mut result = self.battlefield.user.all_characters();
       result.extend(self.battlefield.enemy.all_characters());
       result
   }
   ```
6. Update `add_to_zone()` (lines 390-395) for the Battlefield case to use `add_to_back_rank()` instead of CardSet insert
7. Update `remove_from_zone()` (lines 423-426) for the Battlefield case to use `battlefield.remove()` instead of CardSet remove
8. Update `contains_card()` (lines 260-282) for the Battlefield case to use `battlefield.contains()`

Note: This will cause compilation errors in many downstream files that use the old `CardSet<CharacterId>` battlefield API. Those will be fixed in subsequent tasks. The goal of this task is to get the data model in place and fix enough call sites that `just check` passes.

- [ ] **Step 5: Fix all compilation errors**

After changing the battlefield type, run `just check` and fix every compilation error. Key call sites to update:
- `battle_queries/src/battle_player_queries/player_properties.rs` — `spark_total()` iterates battlefield characters
- `battle_queries/src/legal_action_queries/legal_actions.rs` — character ability enumeration
- `battle_mutations/src/play_cards/character_limit.rs` — character count check
- `battle_mutations/src/card_mutations/move_card.rs` — `on_enter_battlefield()` and `on_leave_battlefield()`
- `display_data` — anywhere ObjectPosition::OnBattlefield is constructed
- Any query that iterates `battlefield()` as a CardSet

For each call site, replace CardSet iteration with `battlefield.all_characters()` or the appropriate Battlefield method. The goal is compilation, not final behavior — judgment phase and repositioning come later.

- [ ] **Step 6: Run `just fmt` then `just review`**

Fix any remaining issues until the full gate passes.

- [ ] **Step 7: Commit**

```bash
git commit -m "feat: replace battlefield CardSet with ranked Battlefield struct

Two-rank battlefield with [Option<CharacterId>; 8] arrays for front
and back ranks. Characters enter the back rank by default. Added
played_turn to CharacterState for summoning sickness."
```

---

## Task 3: Remove Spark Bonus and Old Character Limit

**Files:**
- Modify: `rules_engine/src/battle_state/src/battle_player/battle_player_state.rs` (remove spark_bonus)
- Modify: `rules_engine/src/battle_queries/src/battle_player_queries/player_properties.rs` (simplify spark_total)
- Modify: `rules_engine/src/battle_mutations/src/play_cards/character_limit.rs` (new limit logic)

- [ ] **Step 1: Remove spark_bonus from BattlePlayerState**

In `battle_player_state.rs`, remove the `pub spark_bonus: Spark` field (line ~36). Fix any compilation errors from code that reads or writes this field.

- [ ] **Step 2: Simplify spark_total()**

In `player_properties.rs`, update `spark_total()` to only sum character spark values without adding spark_bonus:

```rust
pub fn spark_total(battle: &BattleState, player: PlayerName) -> Spark {
    let mut total = Spark(0);
    for character_id in battle.cards.battlefield(player).all_characters() {
        total += battle.cards.spark(character_id);
    }
    total
}
```

- [ ] **Step 3: Replace character limit logic**

In `character_limit.rs`, change `CHARACTER_LIMIT` from 8 to 16 and replace the abandon logic. The `apply()` function should now simply prevent playing characters when the battlefield is full rather than abandoning the lowest-spark character:

```rust
const CHARACTER_LIMIT: usize = 16;

/// Returns true if the player's battlefield is full and no more
/// characters can be played.
pub fn is_full(battle: &BattleState, player: PlayerName) -> bool {
    battle.cards.battlefield(player).character_count() >= CHARACTER_LIMIT
}
```

Remove the old `apply()` function that did the abandon logic. Update any call site that called `character_limit::apply()` after materializing — it should instead check `character_limit::is_full()` before allowing a character to be played.

- [ ] **Step 4: Update legal action checks**

In `legal_actions.rs`, when building the list of playable cards from hand, filter out character cards if the battlefield is full. Find where `play_card_from_hand` is populated in `standard_legal_actions()` and add a check:

```rust
// Skip character cards if battlefield is full
if character_limit::is_full(battle, player) {
    // Filter to only event cards
}
```

- [ ] **Step 5: Run `just fmt` then `just review`**

- [ ] **Step 6: Commit**

```bash
git commit -m "feat: remove spark bonus and change character limit to 16

Spark bonus eliminated — spark only comes from characters on the field.
Character limit raised to 16 (8 per rank). Playing characters when
full is prevented instead of abandoning lowest-spark character."
```

---

## Task 4: Phase and Trigger Renames

**Files:**
- Modify: `rules_engine/src/battle_state/src/battle/battle_turn_phase.rs`
- Modify: `rules_engine/src/battle_state/src/triggers/trigger.rs`
- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs`
- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/turn.rs`
- Modify: All files referencing `BattleTurnPhase::Judgment` or `Trigger::Judgment`

- [ ] **Step 1: Rename Judgment phase to Dawn, add new Judgment**

In `battle_turn_phase.rs`, update the enum:

```rust
#[derive(Debug, Ord, PartialOrd, Hash, EnumSetType, Sequence, Serialize, Deserialize)]
pub enum BattleTurnPhase {
    Starting,
    Dreamwell,
    Draw,
    Dawn,
    Main,
    Judgment,
    Ending,
    EndingPhaseFinished,
    FiringEndOfTurnTriggers,
}
```

Note the new ordering: Starting → Dreamwell → Draw → Dawn → Main → Judgment → Ending → EndingPhaseFinished → FiringEndOfTurnTriggers.

- [ ] **Step 2: Rename Trigger::Judgment to Trigger::Dawn, add new Trigger::Judgment**

In `trigger.rs`, rename the existing `Judgment(PlayerName)` variant to `Dawn(PlayerName)` and add a new `Judgment(PlayerName)` variant:

```rust
pub enum Trigger {
    Abandonded(VoidCardId),
    Banished(VoidCardId),
    Dawn(PlayerName),
    Discarded(VoidCardId),
    Dissolved(VoidCardId),
    // ... existing variants ...
    Judgment(PlayerName),
    Materialized(CharacterId),
    // ... rest ...
}
```

Update `TriggerName` enum similarly: rename `Judgment` to `Dawn`, add new `Judgment`. Update the `name()` method in the impl block.

- [ ] **Step 3: Rename judgment_phase.rs to dawn_phase.rs**

Rename the file `rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs` to `dawn_phase.rs`. Update the module declaration in `phase_mutations/mod.rs`. Update the function to fire `Trigger::Dawn` instead of `Trigger::Judgment`:

```rust
/// Runs a Dawn phase for the indicated player. This is a trigger
/// window only — no scoring happens here.
pub fn run(battle: &mut BattleState, player: PlayerName, source: EffectSource) {
    battle.push_animation(source, || BattleAnimation::Dawn { player });
    battle.triggers.push(source, Trigger::Dawn(player));
}
```

Note: The old scoring logic (spark comparison, point gain) is removed from Dawn. It will be reimplemented differently in the new Judgment phase (Task 6).

- [ ] **Step 4: Update turn state machine**

In `turn.rs`, reorder the phase transitions to match the new turn structure:

```
Starting → Dreamwell
Dreamwell → Draw
Draw → Dawn
Dawn → Main
Main → Judgment (new!)
Judgment → Ending
```

The `run_turn_state_machine_if_no_active_prompts()` function needs each match arm updated. The new Judgment arm will call a new `judgment_phase::run()` function (to be created in Task 6). For now, have it simply transition to Ending without doing anything.

- [ ] **Step 5: Fix all compilation errors from renames**

Run `just check` and fix every reference to the old `BattleTurnPhase::Judgment`, `Trigger::Judgment`, and `judgment_phase` module. Key files:
- Card ability definitions that reference `TriggerName::Judgment` → change to `TriggerName::Dawn`
- Display code that checks phase
- Any animation code referencing Judgment

- [ ] **Step 6: Run `just fmt` then `just review`**

- [ ] **Step 7: Commit**

```bash
git commit -m "feat: rename Judgment phase to Dawn, add new Judgment phase

Dawn is now a start-of-turn trigger window (like MTG upkeep). New
Judgment phase added at end of turn for front-rank combat resolution.
Turn order: Dreamwell → Draw → Dawn → Main → Judgment → Ending."
```

---

## Task 5: ObjectPosition Changes for Rank/Position

**Files:**
- Modify: `rules_engine/src/display_data/src/object_position.rs`
- Modify: `scripts/battle_prototype/src/types/battle.ts`
- Modify: All Rust code that constructs `Position::OnBattlefield`

- [ ] **Step 1: Add Rank enum and update Position::OnBattlefield**

In `object_position.rs`, add a `Rank` enum and change `OnBattlefield` to include rank and position:

```rust
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub enum Rank {
    Front,
    Back,
}

// In the Position enum, change:
// OnBattlefield(DisplayPlayer),
// to:
OnBattlefield(DisplayPlayer, Rank, u8),
```

Where the `u8` is the slot index (0-7).

- [ ] **Step 2: Update all Position::OnBattlefield construction sites**

Run `just check` and find every place that constructs `Position::OnBattlefield(player)` and update to include rank and position. The primary site is in the display/view builder that converts internal state to display positions. You'll need to look up the character's rank and position from the Battlefield struct to populate these fields.

Key files to check:
- The view builder that creates CardView objects with ObjectPosition
- `move_card.rs` — where positions are assigned after entering battlefield

- [ ] **Step 3: Update TypeScript types**

In `scripts/battle_prototype/src/types/battle.ts`, update the Position type:

```typescript
// Change from:
// { OnBattlefield: DisplayPlayer }
// to:
{ OnBattlefield: { player: DisplayPlayer, rank: "Front" | "Back", position: number } }
```

- [ ] **Step 4: Update BattleScreen.tsx cardsByPosition helper**

The `cardsByPosition()` function at lines 22-32 of `BattleScreen.tsx` filters cards by position. It needs to handle the new OnBattlefield structure to filter by rank and player.

- [ ] **Step 5: Run `just fmt` then `just review` and `cd scripts/battle_prototype && npx tsc --noEmit`**

- [ ] **Step 6: Commit**

```bash
git commit -m "feat: add rank and position to OnBattlefield position data

OnBattlefield now carries DisplayPlayer, Rank (Front/Back), and slot
index (0-7). Updated Rust construction sites and TypeScript types."
```

---

## Task 6: New Judgment Phase Resolution

**Files:**
- Create: `rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs`
- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/turn.rs` (wire in judgment)
- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/mod.rs` (add module)

- [ ] **Step 1: Create new judgment_phase.rs**

Create `rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs`:

```rust
use battle_state::battle::battle_state::BattleState;
use battle_state::core::effect_source::EffectSource;
use battle_state::core::should_animate::ShouldAnimate;
use battle_state::triggers::trigger::Trigger;
use core_data::numerics::Points;
use core_data::types::PlayerName;

use crate::card_mutations::dissolve;
use crate::player_mutations::points;

/// Runs the Judgment phase: for each front-rank position, the active
/// player's character fights the opponent's character in the same
/// column. Uncontested characters score victory points.
pub fn run(battle: &mut BattleState, player: PlayerName, source: EffectSource) {
    let opponent = player.opponent();

    for position in 0..8u8 {
        let attacker_id = battle.cards.battlefield(player).front[position as usize];
        let defender_id = battle.cards.battlefield(opponent).front[position as usize];

        match (attacker_id, defender_id) {
            (Some(attacker), Some(defender)) => {
                let attacker_spark = battle.cards.spark(attacker);
                let defender_spark = battle.cards.spark(defender);

                if attacker_spark > defender_spark {
                    dissolve::run(battle, defender, source, ShouldAnimate::Yes);
                } else if defender_spark > attacker_spark {
                    dissolve::run(battle, attacker, source, ShouldAnimate::Yes);
                } else {
                    // Tie: dissolve both
                    dissolve::run(battle, defender, source, ShouldAnimate::Yes);
                    dissolve::run(battle, attacker, source, ShouldAnimate::Yes);
                }
            }
            (Some(attacker), None) => {
                // Uncontested: score victory points equal to spark
                let spark = battle.cards.spark(attacker);
                let vp = Points(spark.0);
                points::gain(battle, player, source, vp, ShouldAnimate::Yes);
            }
            _ => {
                // Only defender or both empty: nothing happens
            }
        }
    }

    battle.triggers.push(source, Trigger::Judgment(player));
}
```

Note: This implementation fires Dissolved triggers after each dissolution because `dissolve::run()` already pushes the Dissolved trigger internally. The triggers will resolve before moving to the next position because the turn state machine checks for active prompts before continuing.

- [ ] **Step 2: Verify dissolve::run fires triggers correctly**

Read `rules_engine/src/battle_mutations/src/card_mutations/dissolve.rs` (or wherever the dissolve function lives) and confirm it pushes `Trigger::Dissolved`. If it does, the judgment loop will naturally pause after each dissolution to resolve triggers before the state machine advances.

Important: The judgment phase needs to process one position at a time. If the trigger system processes triggers immediately inline, the loop above works. If triggers are queued and processed later, you may need to restructure this as a state machine that tracks the current position index. Check how `dawn_phase::run()` and the trigger system interact — if `run()` completes fully and then triggers fire, you'll need to change the approach to process one position per state machine tick.

- [ ] **Step 3: Wire judgment into the turn state machine**

In `turn.rs`, update the Judgment phase arm to call `judgment_phase::run()`:

```rust
BattleTurnPhase::Judgment => {
    judgment_phase::run(battle, current_player, source);
    battle.phase = BattleTurnPhase::Ending;
}
```

- [ ] **Step 4: Run `just fmt` then `just review`**

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: implement new Judgment phase with front-rank combat

Each front-rank position resolves independently: higher spark wins,
ties dissolve both, uncontested characters score victory points.
Dissolved triggers fire after each comparison."
```

---

## Task 7: Character Materialization to Back Rank

**Files:**
- Modify: `rules_engine/src/battle_mutations/src/card_mutations/move_card.rs`

- [ ] **Step 1: Update on_enter_battlefield()**

In `move_card.rs`, the `on_enter_battlefield()` function (lines ~255-274) is called when a character enters the battlefield. Update it to:
1. Place the character in the first available back rank slot via `battlefield.add_to_back_rank(character_id)`
2. Set `played_turn` on the CharacterState to the current turn number

```rust
fn on_enter_battlefield(battle: &mut BattleState, card_id: CardId, player: PlayerName) -> CharacterId {
    let character_id = CharacterId(card_id);
    let spark = /* get base spark from card data */;
    let current_turn = battle.turn.turn_number;
    battle.cards.battlefield_mut(player).add_to_back_rank(character_id);
    battle.cards.battlefield_state_mut(player).insert(
        character_id,
        CharacterState {
            spark,
            played_turn: current_turn,
        },
    );
    character_id
}
```

Adapt this to match the exact existing code structure — the key changes are using `add_to_back_rank()` instead of CardSet insert, and setting `played_turn`.

- [ ] **Step 2: Update on_leave_battlefield()**

In the `on_leave_battlefield()` function (lines ~276-294), update to remove the character from the Battlefield struct via `battlefield.remove(character_id)` instead of removing from a CardSet.

- [ ] **Step 3: Run `just fmt` then `just review`**

- [ ] **Step 4: Commit**

```bash
git commit -m "feat: characters materialize into back rank with played_turn tracking

Characters now enter the back rank when materialized. CharacterState
tracks the turn they were played for summoning sickness."
```

---

## Task 8: Repositioning Actions

**Files:**
- Modify: `rules_engine/src/battle_state/src/actions/battle_actions.rs`
- Create: `rules_engine/src/battle_mutations/src/card_mutations/reposition.rs`
- Modify: `rules_engine/src/battle_mutations/src/card_mutations/mod.rs`
- Modify: `rules_engine/src/battle_queries/src/legal_action_queries/legal_actions.rs`
- Modify: `rules_engine/src/battle_queries/src/legal_action_queries/legal_actions_data.rs`

- [ ] **Step 1: Add new BattleAction variants**

In `battle_actions.rs`, add two new variants to the `BattleAction` enum:

```rust
MoveCharacterToFrontRank(CharacterId, u8),
MoveCharacterToBackRank(CharacterId, u8),
```

Update the `battle_action_string()` method to format these.

- [ ] **Step 2: Create reposition mutation**

Create `rules_engine/src/battle_mutations/src/card_mutations/reposition.rs`:

```rust
use battle_state::battle::battle_state::BattleState;
use core_data::types::{CharacterId, PlayerName};

/// Moves a character to a specific front rank position.
/// Panics if the character has summoning sickness or the target slot
/// is occupied by another player's character.
pub fn to_front_rank(
    battle: &mut BattleState,
    player: PlayerName,
    character_id: CharacterId,
    position: u8,
) {
    let battlefield = battle.cards.battlefield_mut(player);
    battlefield.remove(character_id);
    let target = &mut battlefield.front[position as usize];
    if let Some(existing) = *target {
        // Swap: move the existing character to where the moving character was
        // Find where the moving character came from and place existing there
        // For simplicity, put existing in first empty back slot
        // Actually: need to track the source slot for proper swap
    }
    *target = Some(character_id);
}

/// Moves a character to a specific back rank position.
pub fn to_back_rank(
    battle: &mut BattleState,
    player: PlayerName,
    character_id: CharacterId,
    position: u8,
) {
    let battlefield = battle.cards.battlefield_mut(player);
    battlefield.remove(character_id);
    let target = &mut battlefield.back[position as usize];
    if let Some(existing) = *target {
        // Swap logic similar to above
    }
    *target = Some(character_id);
}
```

Implement proper swap logic: when moving to an occupied slot, the occupant goes to the slot the mover came from.

Add the module to `card_mutations/mod.rs`.

- [ ] **Step 3: Add repositioning to legal actions**

In `legal_actions.rs`, within the `standard_legal_actions()` function, add repositioning options when the phase is Main and it's the player's turn. Add a new field to `StandardLegalActions`:

In `legal_actions_data.rs`, add to `StandardLegalActions`:
```rust
pub move_character_to_front_rank: Vec<(CharacterId, u8)>,
pub move_character_to_back_rank: Vec<(CharacterId, u8)>,
```

Populate these in `standard_legal_actions()`:
- For each character in back rank that doesn't have summoning sickness, add entries for each empty front rank slot
- For each character in front rank, add entries for each empty back rank slot
- For swaps: add entries for moving to occupied slots too

- [ ] **Step 4: Handle the new actions in the action dispatcher**

Find where `BattleAction` is matched and dispatched (likely in `battle_mutations` somewhere that handles `perform_action` or `handle_action`). Add match arms for:
- `BattleAction::MoveCharacterToFrontRank(id, pos)` → call `reposition::to_front_rank()`
- `BattleAction::MoveCharacterToBackRank(id, pos)` → call `reposition::to_back_rank()`

- [ ] **Step 5: Update LegalActions to include reposition actions**

Update `LegalActions::all()`, `LegalActions::len()`, `LegalActions::contains()`, and `LegalActions::random_action()` methods in `legal_actions_data.rs` to include the new repositioning actions.

- [ ] **Step 6: Run `just fmt` then `just review`**

- [ ] **Step 7: Commit**

```bash
git commit -m "feat: add character repositioning actions for front/back ranks

Players can move characters between ranks during main phase.
Summoning sickness prevents moving to front rank on the turn played.
Includes swap logic when moving to an occupied slot."
```

---

## Task 9: AI Simplified Actions

**Files:**
- Modify: `rules_engine/src/battle_queries/src/legal_action_queries/legal_actions.rs`
- Modify: `rules_engine/src/battle_queries/src/legal_action_queries/legal_actions_data.rs`
- Modify: `rules_engine/src/battle_state/src/battle/battle_state.rs` (add moved_this_turn tracking)

- [ ] **Step 1: Add per-turn moved tracking**

Add a `moved_this_turn: HashSet<CharacterId>` or similar to `BattleState` (or to turn-scoped transient state). This set tracks which characters the AI has already moved this turn, preventing MoveToBack on characters that were already repositioned. Clear it at the start of each turn.

If there's existing turn-scoped transient state, use that. Otherwise, add a field to `TurnData` or a new transient state container.

- [ ] **Step 2: Implement AI action reduction**

In the legal action computation, when generating repositioning actions, check if the current player is an AI player. If so, generate the simplified action set:

- **MoveToEmptyFrontSlot(CharacterId)**: For each back-rank character without summoning sickness, if there's any empty front-rank slot, emit ONE action (not one per empty slot — they're all equivalent)
- **MoveToAttack(CharacterId, CharacterId)**: For each back-rank character without summoning sickness and each enemy character in the opponent's front rank, emit one action to move to that enemy's position
- **MoveToBack(CharacterId)**: For each front-rank character NOT in `moved_this_turn`, emit one action to retreat

These map to the underlying `MoveCharacterToFrontRank`/`MoveCharacterToBackRank` actions. For MoveToEmptyFrontSlot, pick the first empty slot. For MoveToAttack, use the enemy character's position.

- [ ] **Step 3: Mark characters as moved after repositioning**

In the reposition mutation handlers, add the character to `moved_this_turn` after a successful move.

- [ ] **Step 4: Clear moved_this_turn at turn start**

In the turn state machine, clear the `moved_this_turn` set when transitioning to a new turn (Starting phase).

- [ ] **Step 5: Run `just fmt` then `just review`**

- [ ] **Step 6: Commit**

```bash
git commit -m "feat: simplified AI repositioning actions for MCTS

AI gets three action types: MoveToEmptyFrontSlot, MoveToAttack, and
MoveToBack. MoveToBack blocked for already-moved characters to prevent
infinite MCTS loops. All empty front slots treated as equivalent."
```

---

## Task 10: Battle Prototype UI — Rank Rendering

**Files:**
- Modify: `scripts/battle_prototype/src/components/BattleScreen.tsx`
- Modify: `scripts/battle_prototype/src/components/BattlefieldZone.tsx`
- Modify: `scripts/battle_prototype/src/components/CardDisplay.tsx` (summoning sickness indicator)

- [ ] **Step 1: Update BattlefieldZone to render 8 fixed slots**

Rewrite `BattlefieldZone.tsx` to render a fixed grid of 8 slots. Each slot is either empty (dashed outline) or contains a compact card. Cards are placed by their `position` index from the OnBattlefield position data.

```tsx
interface RankZoneProps {
  label: string;
  cards: CardView[];
  onAction: (action: BattleAction) => void;
  disabled: boolean;
}

function RankZone({ label, cards, onAction, disabled }: RankZoneProps) {
  const slots = Array.from({ length: 8 }, (_, i) => {
    const card = cards.find(c => {
      const pos = c.position.position;
      if (typeof pos === 'object' && 'OnBattlefield' in pos) {
        return pos.OnBattlefield.position === i;
      }
      return false;
    });
    return { index: i, card };
  });

  return (
    <div>
      <div className="text-xs text-slate-500 mb-1">{label}</div>
      <div className="flex gap-1 justify-center">
        {slots.map(({ index, card }) => (
          <div key={index} className="w-[100px] h-[60px]">
            {card ? (
              <CardDisplay card={card} compact battlefield onAction={onAction} disabled={disabled} />
            ) : (
              <div className="w-full h-full border border-dashed border-slate-700 rounded" />
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Update BattleScreen layout to 4 ranks**

In `BattleScreen.tsx`, replace the two BattlefieldZone renders with four RankZone renders. Filter cards by rank:

```tsx
// Helper to filter by rank
function cardsByRank(cards: CardView[], player: DisplayPlayer, rank: "Front" | "Back"): CardView[] {
  return cards.filter(c => {
    const pos = c.position.position;
    return typeof pos === 'object'
      && 'OnBattlefield' in pos
      && pos.OnBattlefield.player === player
      && pos.OnBattlefield.rank === rank;
  });
}
```

Layout order (top to bottom):
1. Enemy status
2. Enemy back rank (label: "ENEMY BACK RANK")
3. Enemy front rank (label: "ENEMY FRONT RANK")
4. Judgment line separator (gold border with "⚡ JUDGMENT LINE ⚡" text)
5. Your front rank (label: "YOUR FRONT RANK")
6. Your back rank (label: "YOUR BACK RANK")
7. Your status
8. Hand
9. Action bar
10. Battle log

- [ ] **Step 3: Add summoning sickness visual indicator**

In `CardDisplay.tsx`, if the card has a property indicating summoning sickness (you'll need to add this to CardView or infer it from card data), show a visual indicator like a blue border or an icon overlay. The Rust side should include this info in the CardView — add a `summoning_sick: bool` field to whatever card view type is used.

- [ ] **Step 4: Add judgment line separator component**

Add a styled divider between enemy front rank and your front rank:

```tsx
<div className="flex items-center justify-center gap-2 py-1">
  <div className="flex-1 border-t border-yellow-600/50" />
  <span className="text-xs text-yellow-600">⚡ JUDGMENT LINE ⚡</span>
  <div className="flex-1 border-t border-yellow-600/50" />
</div>
```

- [ ] **Step 5: Verify TypeScript compiles**

Run: `cd scripts/battle_prototype && npx tsc --noEmit`

- [ ] **Step 6: Commit**

```bash
git commit -m "feat: render four battlefield ranks with judgment line separator

Battle screen now shows enemy back rank, enemy front rank, judgment
line, your front rank, your back rank. Each rank has 8 fixed slots."
```

---

## Task 11: QA — Rank Rendering

**QA milestone using agent-browser CLI.**

- [ ] **Step 1: Start the battle prototype**

Start both the Rust dev server and the frontend dev server. Navigate to the battle prototype URL.

- [ ] **Step 2: Run QA pass**

Using the QA skill with agent-browser, verify:
- 4 rank zones display (enemy back, enemy front, your front, your back)
- Each rank shows 8 slots
- Labels are visible and correct
- Judgment line separator is visible between the two front ranks
- Empty slots show as dashed outlines
- Overall layout is readable and not cramped

- [ ] **Step 3: Fix any bugs found**

- [ ] **Step 4: Commit fixes**

---

## Task 12: QA — Character Placement

**QA milestone using agent-browser CLI.**

- [ ] **Step 1: Play characters and verify placement**

Using agent-browser, play the game and verify:
- Characters appear in the back rank when played
- Characters show in the correct slot (not overlapping)
- Spark values display correctly on battlefield cards
- Enemy characters also appear in back rank

- [ ] **Step 2: Fix any bugs found**

- [ ] **Step 3: Commit fixes**

---

## Task 13: Battle Prototype UI — Drag and Drop

**Files:**
- Modify: `scripts/battle_prototype/src/components/BattleScreen.tsx`
- Create: `scripts/battle_prototype/src/components/RankZone.tsx` (if not already extracted)

- [ ] **Step 1: Implement drag-and-drop for character repositioning**

Add HTML5 drag-and-drop to the RankZone component. During the main phase when it's the player's turn:
- Characters are draggable (set `draggable={true}`)
- Empty slots and occupied slots are valid drop targets
- On drop: dispatch the appropriate `MoveCharacterToFrontRank` or `MoveCharacterToBackRank` action
- Summoning-sick characters cannot be dragged to front rank slots (visual feedback on invalid drop)

```tsx
// On drag start, store the character's card ID
const handleDragStart = (e: React.DragEvent, cardId: string) => {
  e.dataTransfer.setData('text/plain', cardId);
};

// On drop, determine target rank and position, dispatch action
const handleDrop = (e: React.DragEvent, targetRank: "Front" | "Back", targetPosition: number) => {
  const cardId = e.dataTransfer.getData('text/plain');
  if (targetRank === "Front") {
    onAction({ MoveCharacterToFrontRank: [cardId, targetPosition] });
  } else {
    onAction({ MoveCharacterToBackRank: [cardId, targetPosition] });
  }
};
```

- [ ] **Step 2: Add drag visual feedback**

When dragging, highlight valid drop targets with a green border. Invalid targets (summoning sick to front) get a red indicator.

- [ ] **Step 3: Disable drag outside main phase**

Only enable dragging when `can_act` is true and the phase is Main.

- [ ] **Step 4: Verify TypeScript compiles and commit**

```bash
git commit -m "feat: drag-and-drop character repositioning in battle prototype

Characters can be dragged between rank slots during main phase.
Summoning-sick characters cannot be dragged to front rank.
Visual feedback for valid/invalid drop targets."
```

---

## Task 14: QA — Drag and Drop Basics

**QA milestone using agent-browser CLI.**

- [ ] **Step 1: Test repositioning interactions**

Using agent-browser, verify:
- Can drag a character from back rank to an empty front rank slot
- Can drag a character from front rank to an empty back rank slot
- Can drag a character to swap with another character in the same rank
- Dragging to a different rank's occupied slot swaps correctly
- Drag is disabled when it's not your turn
- Drag is disabled outside the main phase

- [ ] **Step 2: Fix any bugs found**

- [ ] **Step 3: Commit fixes**

---

## Task 15: QA — Summoning Sickness

**QA milestone using agent-browser CLI.**

- [ ] **Step 1: Test summoning sickness**

Using agent-browser, verify:
- A character played this turn CANNOT be dragged to the front rank
- The summoning-sick character CAN be moved within the back rank
- On the next turn, the character CAN be moved to the front rank
- Visual indicator clearly shows which characters are summoning sick

- [ ] **Step 2: Fix any bugs found**

- [ ] **Step 3: Commit fixes**

---

## Task 16: QA — Judgment Resolution Visuals

**QA milestone using agent-browser CLI.**

- [ ] **Step 1: Test judgment phase resolution**

Using agent-browser, play several turns and verify:
- End turn triggers judgment phase
- Front rank characters fight opposing column characters
- Higher spark character wins (opponent dissolved)
- Lower spark character loses (dissolved)
- Ties dissolve both
- Uncontested characters score victory points
- Battle log shows judgment results clearly
- Score updates correctly after judgment
- Back rank characters are unaffected

- [ ] **Step 2: Fix any bugs found**

- [ ] **Step 3: Commit fixes**

---

## Task 17: QA — Full Game Loop Round 1

**QA milestone using agent-browser CLI.**

- [ ] **Step 1: Play 10+ turns**

Play a full game against the AI for at least 10 turns. Verify:
- Turn structure: Dreamwell → Draw → Dawn → Main → Judgment → End
- Energy resets correctly each turn
- Card drawing works
- Dawn triggers fire (if any cards have them)
- Playing characters and events works
- Judgment phase resolves correctly each turn
- Score accumulates properly
- Game ends when threshold reached

- [ ] **Step 2: Track invariants**

- Total characters on battlefield never exceeds 16 per player
- Score only increases during judgment phase
- Characters only enter back rank
- Phase order is consistent every turn

- [ ] **Step 3: Fix any bugs found**

- [ ] **Step 4: Commit fixes**

---

## Task 18: QA — AI Behavior Round 1

**QA milestone using agent-browser CLI.**

- [ ] **Step 1: Observe AI play**

Play several games and observe:
- AI plays characters
- AI moves characters to front rank (not on turn played)
- AI moves characters to attack enemy characters
- AI doesn't loop infinitely (game doesn't hang on AI turn)
- AI retreats characters sometimes
- AI doesn't move the same character back and forth

- [ ] **Step 2: Fix any bugs found**

- [ ] **Step 3: Commit fixes**

---

## Task 19: Kindle Effect Update

**Files:**
- Modify: `rules_engine/src/battle_mutations/src/effects/apply_standard_effect.rs`
- Modify: Kindle target query (find where "leftmost" logic exists)

- [ ] **Step 1: Implement Kindle as highest-spark targeting**

Find the Kindle effect handler (it may be unimplemented — falling to `todo!()`). Implement it to:
1. Find the character with the highest spark among the player's battlefield characters
2. Tiebreaker: oldest character (lowest `played_turn` value)
3. Add the Kindle amount to that character's spark

```rust
StandardEffect::Kindle { amount } => {
    let player = source_player;
    let characters = battle.cards.battlefield(player).all_characters();
    if let Some(target) = characters
        .iter()
        .max_by_key(|&&c| {
            let spark = battle.cards.spark(c);
            let played_turn = battle.cards.battlefield_state(player).get(&c)
                .map(|s| s.played_turn)
                .unwrap_or(0);
            // Sort by spark descending, then by played_turn ascending (oldest first)
            (spark, std::cmp::Reverse(played_turn))
        })
    {
        let state = battle.cards.battlefield_state_mut(player)
            .get_mut(target)
            .expect("Character must exist");
        state.spark += amount;
    }
}
```

- [ ] **Step 2: Run `just fmt` then `just review`**

- [ ] **Step 3: Commit**

```bash
git commit -m "feat: kindle targets highest-spark character instead of leftmost

Tiebreaker: oldest character (first played). Replaces the old
position-based leftmost targeting."
```

---

## Task 20: QA — Card Effects with Ranks

**QA milestone using agent-browser CLI.**

- [ ] **Step 1: Test card effect interactions**

Using agent-browser, verify:
- Dissolve effects correctly remove characters from their rank slot
- Materialize effects (from void, from deck) place characters in back rank
- Kindle applies to highest-spark character
- Banish removes characters from their slot
- Cards that reference "characters on the battlefield" work for both ranks

- [ ] **Step 2: Fix any bugs found**

- [ ] **Step 3: Commit fixes**

---

## Task 21: QA — UX Polish Round 1

**QA milestone using agent-browser CLI.**

- [ ] **Step 1: Full UX audit**

Review the entire battle prototype for usability:
- Are rank labels clear and readable?
- Is the judgment line visually distinct?
- Can you tell which characters are summoning sick?
- Does the battle log explain judgment results clearly?
- Is the score display accurate?
- Are card spark values visible on battlefield cards?
- Is the overall layout too tall/cramped for the screen?
- Can you tell which characters will fight which during judgment?
- Are empty slots visible enough to understand valid positions?

- [ ] **Step 2: Implement UX improvements based on findings**

Common improvements might include:
- Column highlighting to show which characters face each other
- Judgment phase animation or log entries
- Better summoning sickness indicator
- Score change animations

- [ ] **Step 3: Commit improvements**

---

## Task 22: QA — Extended Play Round 1 (15+ turns)

**QA milestone using agent-browser CLI.**

- [ ] **Step 1: Play an extended game**

Play 15+ turns against the AI. Focus on:
- Mid-game board states with many characters
- Characters accumulating in both ranks
- Multiple judgment phases with complex board states
- Score progression and game ending correctly

- [ ] **Step 2: Log all anomalies**

Track every bug, visual glitch, or unexpected behavior.

- [ ] **Step 3: Fix bugs and commit**

---

## Task 23: QA — Extended Play Round 2 (Board Full States)

**QA milestone using agent-browser CLI.**

- [ ] **Step 1: Test board-full scenarios**

Try to fill the battlefield to 16 characters. Verify:
- Can't play more characters when board is full
- The UI communicates why you can't play a character
- Events can still be played when board is full
- If characters die during judgment, you can play more next turn

- [ ] **Step 2: Fix bugs and commit**

---

## Task 24: QA — Extended Play Round 3 (Judgment Edge Cases)

**QA milestone using agent-browser CLI.**

- [ ] **Step 1: Test judgment edge cases**

Set up and test:
- Tie: two characters with the same spark in the same column → both dissolve
- Zero spark characters in combat
- All 8 front rank positions filled on both sides
- One side has full front rank, other side has empty front rank (mass scoring)
- Dissolved trigger that modifies another character's spark before their column resolves
- Character dissolved by judgment that had a Dissolved trigger

- [ ] **Step 2: Fix bugs and commit**

---

## Task 25: QA — AI Behavior Round 2

**QA milestone using agent-browser CLI.**

- [ ] **Step 1: Watch AI tactics across multiple games**

Play 3+ full games and evaluate:
- Does AI position high-spark characters against enemy low-spark ones?
- Does AI leave uncontested characters to score?
- Does AI retreat valuable characters when threatened?
- Does AI use events effectively alongside positioning?
- Are AI turns completing in reasonable time?
- Does AI avoid obviously bad moves (like putting 1-spark vs 10-spark)?

- [ ] **Step 2: Fix bugs and commit**

---

## Task 26: QA — UX Polish Round 2

**QA milestone using agent-browser CLI.**

- [ ] **Step 1: Second UX pass**

Based on all previous extended play sessions, audit:
- Judgment phase readability: is it clear what happened during judgment?
- Battle log: does it show enough info about judgment results?
- Column alignment: can you visually see which characters are paired?
- Turn transition: is it clear when judgment fires vs when the turn ends?
- Any confusion about front/back rank semantics?

- [ ] **Step 2: Implement improvements and commit**

---

## Task 27: QA — Extended Play Round 4 (20+ Turns Stress Test)

**QA milestone using agent-browser CLI.**

- [ ] **Step 1: Final extended play session**

Play a full 20+ turn game. This is the final comprehensive test. Verify:
- No crashes or hangs
- Score correctly reaches threshold and game ends
- All card effects still work
- Judgment resolves correctly even with complex board states
- AI plays reasonably throughout
- UI remains readable and performant with many characters
- Battle log is useful and accurate

- [ ] **Step 2: Fix any remaining bugs and commit**

---

## Task 28: Update Battle Rules Documentation

**Files:**
- Modify: `docs/battle_rules/battle_rules.md`

- [ ] **Step 1: Update rules document**

Update the battle rules document to reflect all changes:
- Two-rank battlefield (front/back)
- New turn structure (Dreamwell → Draw → Dawn → Main → Judgment)
- Judgment phase resolution (column-by-column combat)
- Summoning sickness
- Character limit of 16
- No spark bonus
- Kindle targets highest-spark character
- Dawn phase (renamed from Judgment, trigger window only)

- [ ] **Step 2: Commit**

```bash
git commit -m "docs: update battle rules for combat prototype

Reflects two-rank battlefield, new judgment phase with front-rank
combat, revised turn structure, and updated keywords."
```

---

## Task 29: QA — Final Regression Pass

**QA milestone using agent-browser CLI.**

- [ ] **Step 1: Full regression test**

Play a complete game from start to finish, testing every feature:
- Character placement in back rank
- Repositioning via drag-and-drop
- Summoning sickness
- Judgment resolution (win/lose/tie/uncontested)
- Dissolved triggers during judgment
- Card effects (dissolve, kindle, materialize, etc.)
- Board-full prevention
- AI behavior
- Score accumulation and game end
- Turn phases in correct order

- [ ] **Step 2: Fix any remaining issues and commit**

- [ ] **Step 3: Run `just fmt` then `just review` one final time**

---

## Task 30: Final Commit and Cleanup

- [ ] **Step 1: Run full review gate**

Run: `just review` (in foreground)

Fix any remaining lint, format, or type errors.

- [ ] **Step 2: Final commit**

```bash
git commit -m "chore: final cleanup for combat prototype"
```
