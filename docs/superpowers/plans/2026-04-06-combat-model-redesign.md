# Combat Model Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Revise the Dreamtides battle system so the non-active player's
front-rank characters attack during the active player's Judgment, with
post-judgment return to back rank, phase reordering (Ending before Judgment),
and an 8-character battlefield limit.

**Architecture:** Modify existing judgment resolution, phase transitions, and
playability checks in the Rust rules engine. Update event generation in the
TypeScript battle prototype. No new crates, modules, or data structures beyond a
single new field on TurnData.

**Tech Stack:** Rust (rules engine), TypeScript/React (battle prototype)

______________________________________________________________________

### Task 1: Change Character Limit to 8

**Files:**

- Modify:
  `rules_engine/src/battle_queries/src/legal_action_queries/can_play_cards.rs:20`

- Modify: `rules_engine/src/battle_state/src/battle_cards/battlefield.rs:21-22`

- [ ] **Step 1: Update CHARACTER_LIMIT constant**

In `rules_engine/src/battle_queries/src/legal_action_queries/can_play_cards.rs`,
change line 20:

```rust
const CHARACTER_LIMIT: usize = 8;
```

- [ ] **Step 2: Update is_full() method**

In `rules_engine/src/battle_state/src/battle_cards/battlefield.rs`, change the
`is_full` method and its doc comment:

```rust
    /// Returns true if the battlefield has 8 or more characters (the maximum).
    pub fn is_full(&self) -> bool {
        self.character_count() >= 8
    }
```

- [ ] **Step 3: Commit**

```bash
git add rules_engine/src/battle_queries/src/legal_action_queries/can_play_cards.rs \
       rules_engine/src/battle_state/src/battle_cards/battlefield.rs
git commit -m "feat: reduce battlefield character limit from 16 to 8"
```

______________________________________________________________________

### Task 2: Reorder Phases — Ending Before Judgment

**Files:**

- Modify: `rules_engine/src/battle_state/src/battle/battle_turn_phase.rs`

- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/turn.rs`

- [ ] **Step 1: Reorder the BattleTurnPhase enum**

In `rules_engine/src/battle_state/src/battle/battle_turn_phase.rs`, swap
`Ending` before `Judgment` so the enum matches the new phase flow:

```rust
#[derive(Debug, Ord, PartialOrd, Hash, EnumSetType, Sequence, Serialize, Deserialize)]
pub enum BattleTurnPhase {
    Starting,
    Dreamwell,
    Draw,
    Dawn,
    Main,
    Ending,
    Judgment,
    EndingPhaseFinished,
    FiringEndOfTurnTriggers,
}
```

- [ ] **Step 2: Update to_ending_phase() to transition to Ending (not
  Judgment)**

In `rules_engine/src/battle_mutations/src/phase_mutations/turn.rs`, change
`to_ending_phase()` (lines 18-26) to set the phase to `Ending` instead of
`Judgment`, and remove the judgment_position initialization (that moves to Step
3):

```rust
/// End the current player's turn.
///
/// Transitions into the Ending phase where the opponent may play fast cards
/// before Judgment resolves.
pub fn to_ending_phase(battle: &mut BattleState) {
    battle.phase = BattleTurnPhase::Ending;
    battle_trace!(
        "Moving to ending phase for player",
        battle,
        player = battle.turn.active_player
    );
}
```

- [ ] **Step 3: Update start_next_turn() to transition to Judgment (not
  EndingPhaseFinished)**

In the same file, change `start_next_turn()` (lines 117-120) to initialize
judgment and transition to the Judgment phase:

```rust
/// Transition from the Ending phase to the Judgment phase.
///
/// Called when the non-active player passes during the Ending phase.
pub fn start_next_turn(battle: &mut BattleState) {
    battle.turn.judgment_position = 0;
    battle.turn.judgment_participants.clear();
    battle.phase = BattleTurnPhase::Judgment;
    battle_trace!(
        "Moving to judgment phase for player",
        battle,
        player = battle.turn.active_player
    );
}
```

Note: `judgment_participants` is added in Task 4 Step 1. If building Task 2
before Task 4, temporarily omit the `judgment_participants.clear()` line and add
it during Task 4.

- [ ] **Step 4: Update Judgment case to transition to EndingPhaseFinished (not
  Ending)**

In the same file, in `run_turn_state_machine_if_no_active_prompts()`, change the
Judgment case (lines 92-108). After judgment finishes, transition to
`EndingPhaseFinished` instead of `Ending`:

```rust
            BattleTurnPhase::Judgment => {
                let player = battle.turn.active_player;
                let source = EffectSource::Game { controller: player };
                let finished = judgment_phase::run(battle, player, source);
                apply_effect::execute_pending_effects_if_no_active_prompt(battle);
                fire_triggers::execute_if_no_active_prompt(battle);
                if finished && battle.prompts.is_empty() {
                    judgment_phase::return_participants_to_back_rank(battle);
                    battle.triggers.push(source, Trigger::Judgment(player));
                    apply_effect::execute_pending_effects_if_no_active_prompt(battle);
                    fire_triggers::execute_if_no_active_prompt(battle);
                    battle.phase = BattleTurnPhase::EndingPhaseFinished;
                    battle_trace!(
                        "Judgment phase complete, moving to ending phase finished for player",
                        battle,
                        player
                    );
                }
            }
```

Note: `judgment_phase::return_participants_to_back_rank` is implemented in Task
4\.

- [ ] **Step 5: Commit**

```bash
git add rules_engine/src/battle_state/src/battle/battle_turn_phase.rs \
       rules_engine/src/battle_mutations/src/phase_mutations/turn.rs
git commit -m "feat: reorder phases so Ending (fast actions) occurs before Judgment"
```

______________________________________________________________________

### Task 3: Flip Judgment Resolution — Non-Active Player Attacks

**Files:**

- Modify:
  `rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs`

- [ ] **Step 1: Swap attacker and defender roles**

In `rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs`,
rewrite the `run()` function. The non-active player's front-rank characters are
now attackers, and the active player's front-rank characters are blockers:

```rust
/// Resolves one column of front-rank combat during the Judgment phase.
///
/// The non-active player's front-rank characters are attackers. The active
/// player's front-rank characters are blockers. Returns true if all 8
/// positions have been processed.
pub fn run(battle: &mut BattleState, player: PlayerName, source: EffectSource) -> bool {
    let position = battle.turn.judgment_position;
    let opponent = player.opponent();
    battle_trace!("Judgment phase resolving position", battle, position, player);

    let attacker_id = battle.cards.battlefield(opponent).front[position as usize];
    let blocker_id = battle.cards.battlefield(player).front[position as usize];

    match (attacker_id, blocker_id) {
        (Some(attacker), Some(blocker)) => {
            battle.turn.judgment_participants.push((opponent, attacker, position));
            battle.turn.judgment_participants.push((player, blocker, position));
            let attacker_spark = battle.cards.spark(opponent, attacker).unwrap_or_default();
            let blocker_spark = battle.cards.spark(player, blocker).unwrap_or_default();
            if attacker_spark > blocker_spark {
                dissolve::execute(battle, source, blocker);
            } else if blocker_spark > attacker_spark {
                dissolve::execute(battle, source, attacker);
            } else {
                dissolve::execute(battle, source, blocker);
                dissolve::execute(battle, source, attacker);
            }
        }
        (Some(attacker), None) => {
            let spark = battle.cards.spark(opponent, attacker).unwrap_or_default();
            points::gain(battle, opponent, source, Points(spark.0), ShouldAnimate::Yes);
        }
        _ => {}
    }

    if position >= 7 {
        true
    } else {
        battle.turn.judgment_position = position + 1;
        false
    }
}
```

Note: `judgment_participants` tracking is added in Tasks 3 and 4. The
`(opponent, attacker, position)` and `(player, blocker, position)` tuples record
the player, character ID, and column for post-judgment back-rank return.

- [ ] **Step 2: Add the import for PlayerName**

At the top of the file, ensure `PlayerName` is imported (it already is via the
`player: PlayerName` parameter, but verify). Also add the `use` for the new
`judgment_participants` field — no new imports needed since `PlayerName` and
`CharacterId` are already in scope via the function signatures.

- [ ] **Step 3: Commit**

```bash
git add rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs
git commit -m "feat: non-active player's front-rank characters now attack during Judgment"
```

______________________________________________________________________

### Task 4: Post-Judgment Return to Back Rank

**Files:**

- Modify: `rules_engine/src/battle_state/src/battle/turn_data.rs`

- Modify:
  `rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs`

- Modify: `rules_engine/src/battle_state/src/battle_cards/battlefield.rs`

- [ ] **Step 1: Add judgment_participants field to TurnData**

In `rules_engine/src/battle_state/src/battle/turn_data.rs`, add the import for
`PlayerName` (already present) and add a new field to `TurnData`:

```rust
use core_data::numerics::TurnId;
use core_data::types::PlayerName;
use serde::{Deserialize, Serialize};

use crate::battle::card_id::CharacterId;

/// Identifies a turn within the game.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TurnData {
    /// Player whose turn it is or was.
    pub active_player: PlayerName,

    /// Identifies the turn.
    ///
    /// Each player's turn gets its own ID, so the first turn of the game is
    /// turn 0 for the starting player and then turn 1 for the next player.
    pub turn_id: TurnId,

    /// Current column position being resolved during the Judgment phase (0-7).
    pub judgment_position: u8,

    /// Characters that have been repositioned this turn, used to prevent
    /// infinite back-and-forth movement by the AI.
    pub moved_this_turn: Vec<CharacterId>,

    /// Characters that participated in a judgment (spark comparison) during the
    /// current Judgment phase. Each entry is (player, character_id, column).
    /// After all columns resolve, surviving participants return to back rank.
    pub judgment_participants: Vec<(PlayerName, CharacterId, u8)>,
}

impl Default for TurnData {
    fn default() -> Self {
        TurnData {
            active_player: PlayerName::One,
            turn_id: TurnId::default(),
            judgment_position: 0,
            moved_this_turn: Vec::new(),
            judgment_participants: Vec::new(),
        }
    }
}
```

- [ ] **Step 2: Add return_to_back_rank_at_column method to Battlefield**

In `rules_engine/src/battle_state/src/battle_cards/battlefield.rs`, add a new
method that moves a character from front rank to back rank, preferring the same
column index:

```rust
    /// Moves a character from the front rank to the back rank, preferring the
    /// same column index. If that back-rank slot is occupied, the character is
    /// placed in the first available back-rank slot instead.
    ///
    /// Returns false if the character is not in the front rank or no back-rank
    /// slot is available.
    pub fn return_to_back_rank(&mut self, id: CharacterId) -> bool {
        let Some(front_col) = self.front.iter().position(|s| *s == Some(id)) else {
            return false;
        };
        self.front[front_col] = None;
        if self.back[front_col].is_none() {
            self.back[front_col] = Some(id);
        } else if let Some(slot) = self.first_empty_back_slot() {
            self.back[slot] = Some(id);
        } else {
            // Should not happen given 8-character limit, but restore front rank
            // position to avoid losing the character.
            self.front[front_col] = Some(id);
            return false;
        }
        true
    }
```

- [ ] **Step 3: Add return_participants_to_back_rank function to
  judgment_phase.rs**

In `rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs`,
add a new public function after the `run()` function:

```rust
/// After all Judgment columns resolve, move surviving participants back to
/// the back rank. Characters that were dissolved during Judgment are already
/// in the Void and are skipped.
pub fn return_participants_to_back_rank(battle: &mut BattleState) {
    let participants: Vec<(PlayerName, CharacterId, u8)> =
        battle.turn.judgment_participants.drain(..).collect();
    for (player, character_id, _column) in participants {
        let bf = battle.cards.battlefield_mut(player);
        if bf.is_in_front_rank(character_id) {
            bf.return_to_back_rank(character_id);
        }
    }
}
```

- [ ] **Step 4: Add the judgment_participants.clear() call in start_next_turn**

If not already added in Task 2 Step 3, ensure
`rules_engine/src/battle_mutations/src/phase_mutations/turn.rs` has
`battle.turn.judgment_participants.clear();` inside `start_next_turn()`.

- [ ] **Step 5: Also clear judgment_participants in the FiringEndOfTurnTriggers
  transition**

In `turn.rs`, inside the `FiringEndOfTurnTriggers` case (around line 52 where
`moved_this_turn.clear()` is called), add:

```rust
battle.turn.judgment_participants.clear();
```

right after `battle.turn.moved_this_turn.clear();` to ensure a clean slate for
the next turn.

- [ ] **Step 6: Commit**

```bash
git add rules_engine/src/battle_state/src/battle/turn_data.rs \
       rules_engine/src/battle_state/src/battle_cards/battlefield.rs \
       rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs \
       rules_engine/src/battle_mutations/src/phase_mutations/turn.rs
git commit -m "feat: surviving judgment participants return to back rank after combat"
```

______________________________________________________________________

### Task 5: Build and Fix Compilation Errors

**Files:** Various (depends on compiler output)

- [ ] **Step 1: Run format**

```bash
cd /Users/dthurn/dreamtides/rules_engine && just fmt
```

- [ ] **Step 2: Run type check**

```bash
cd /Users/dthurn/dreamtides/rules_engine && just check
```

Fix any compilation errors. Common issues:

- Missing imports for `PlayerName` in `judgment_phase.rs` (should already be
  available via function parameter)

- `judgment_participants` field not recognized if Task 4 Step 1 wasn't applied
  before Task 2/3

- `return_participants_to_back_rank` not found if Task 4 Step 3 wasn't applied
  before Task 2 Step 4

- [ ] **Step 3: Run clippy**

```bash
cd /Users/dthurn/dreamtides/rules_engine && just clippy
```

Fix any lint warnings.

- [ ] **Step 4: Run full review**

```bash
cd /Users/dthurn/dreamtides/rules_engine && just review
```

This runs the full gate (format + check + clippy + tests). Allow ~5 minutes.

- [ ] **Step 5: Commit any fixes**

```bash
git add -u
git commit -m "fix: resolve compilation and lint issues from combat model changes"
```

______________________________________________________________________

### Task 6: Update Battle Prototype Event Generation

**Files:**

- Modify: `scripts/battle_prototype/src/state/battle-context.tsx`

- [ ] **Step 1: Update generateEvents() judgment log**

In `scripts/battle_prototype/src/state/battle-context.tsx`, the
`generateEvents()` function (lines 74-196) generates judgment event
descriptions. The current logic treats both players symmetrically — it just says
who defeated whom. The log messages should now reflect that the non-active
player's characters are the attackers.

The current judgment detection uses `turn_number` change as a proxy. The event
messages at lines 96-113 already describe outcomes symmetrically ("Your X
defeated Enemy Y" / "Enemy X defeated Your Y"), which is acceptable. The
"uncontested" messages should be updated to say "attacked unblocked" instead of
"was uncontested":

Replace lines 107-113:

```typescript
      } else if (u) {
        judgmentHadAction = true;
        events.push(`Slot ${slot}: Your ${u.name} (${u.spark} spark) attacked unblocked — scored ${u.spark} points`);
      } else if (e) {
        judgmentHadAction = true;
        events.push(`Slot ${slot}: Enemy ${e.name} (${e.spark} spark) attacked unblocked — scored ${e.spark} points`);
      }
```

- [ ] **Step 2: Verify prototype builds**

```bash
cd /Users/dthurn/dreamtides/scripts/battle_prototype && npm run build
```

- [ ] **Step 3: Commit**

```bash
git add scripts/battle_prototype/src/state/battle-context.tsx
git commit -m "feat: update battle prototype judgment event text for new combat model"
```

______________________________________________________________________

### Task 7: Manual QA

**Files:** None (QA only)

- [ ] **Step 1: Start the dev server**

Start both the rules engine server and the battle prototype dev server. The
rules engine server is typically started with:

```bash
cd /Users/dthurn/dreamtides/rules_engine && cargo run
```

And the prototype with:

```bash
cd /Users/dthurn/dreamtides/scripts/battle_prototype && npm run dev
```

- [ ] **Step 2: Run QA using the qa skill**

Dispatch a QA subagent using the `qa` skill (see `.llms/skills/qa/SKILL.md`) to
test the battle prototype at `http://localhost:5173`. The QA agent should
verify:

01. **Attacker/blocker classification:** Front-rank characters with an enemy
    across are blockers; those without are attackers
02. **Non-active player attacks:** During the active player's Judgment, the
    opponent's attackers attack (not the active player's)
03. **Spark comparisons:** Higher spark wins, equal spark both dissolve,
    attacker does not score when blocked
04. **Unblocked attackers score:** Points equal to spark for unblocked attackers
05. **Post-judgment back rank return:** Surviving participants (winners of spark
    comparisons) return to back rank
06. **Non-participants stay:** Front-rank characters not involved in judgment
    remain in front rank
07. **8-character limit:** Cannot play a 9th character when 8 are on battlefield
08. **Summoning sickness:** Characters played this turn cannot move to front
    rank
09. **Phase ordering:** Ending phase (fast actions) occurs before Judgment — use
    fast-speed dissolve to remove an attacker before Judgment
10. **Repositioning restrictions:** Can only reposition during main phase

- [ ] **Step 3: Fix any bugs found during QA**

Address issues from the QA report and re-run affected tests.

- [ ] **Step 4: Final commit**

```bash
git add -u
git commit -m "fix: address QA findings from combat model redesign"
```
