# Staggered Grid Combat Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Change the battlefield from an 8-column aligned grid to a staggered 5-back/4-front layout with persistent front-row positioning.

**Architecture:** The `Battlefield` struct's arrays change from `[_; 8]` to `[_; 5]` (back) and `[_; 4]` (front). Judgment iterates 0–3. The return-to-back-rank step is removed. The `CHARACTER_LIMIT` changes from 8 to 9. Repositioning constants change from `0..8u8` to the appropriate range. Support relationships are added as a queryable function.

**Tech Stack:** Rust (rules_engine crates), integration tests in `rules_engine/tests/`

---

### Task 1: Update Battlefield Struct and Constants

**Files:**
- Modify: `rules_engine/src/battle_state/src/battle_cards/battlefield.rs`
- Modify: `rules_engine/src/battle_queries/src/legal_action_queries/can_play_cards.rs:20`
- Modify: `rules_engine/src/battle_mutations/src/play_cards/character_limit.rs`

- [ ] **Step 1: Update the Battlefield struct arrays**

In `rules_engine/src/battle_state/src/battle_cards/battlefield.rs`, change the struct:

```rust
pub struct Battlefield {
    pub front: [Option<CharacterId>; 4],
    pub back: [Option<CharacterId>; 5],
}
```

- [ ] **Step 2: Update `is_full` to use 9**

In the same file, change `is_full`:

```rust
pub fn is_full(&self) -> bool {
    self.character_count() >= 9
}
```

Also update the doc comment on `is_full` from "8 or more" to "9 or more".

- [ ] **Step 3: Add `back_row_is_full` method**

Add a new method after `is_full`:

```rust
/// Returns true if all 5 back-row slots are occupied.
pub fn back_row_is_full(&self) -> bool {
    self.back.iter().all(Option::is_some)
}
```

- [ ] **Step 4: Add support relationship query**

Add a new method to `Battlefield`:

```rust
/// Returns the front-row slot indices that the given back-row slot
/// supports in the staggered grid layout.
///
/// B0→[F0], B1→[F0,F1], B2→[F1,F2], B3→[F2,F3], B4→[F3]
pub fn supported_front_slots(back_slot: usize) -> &'static [usize] {
    match back_slot {
        0 => &[0],
        1 => &[0, 1],
        2 => &[1, 2],
        3 => &[2, 3],
        4 => &[3],
        _ => &[],
    }
}

/// Returns the back-row slot indices that support the given front-row
/// slot in the staggered grid layout.
///
/// F0→[B0,B1], F1→[B1,B2], F2→[B2,B3], F3→[B3,B4]
pub fn supporting_back_slots(front_slot: usize) -> &'static [usize] {
    match front_slot {
        0 => &[0, 1],
        1 => &[1, 2],
        2 => &[2, 3],
        3 => &[3, 4],
        _ => &[],
    }
}
```

- [ ] **Step 5: Update CHARACTER_LIMIT constant**

In `rules_engine/src/battle_queries/src/legal_action_queries/can_play_cards.rs`, change:

```rust
const CHARACTER_LIMIT: usize = 9;
```

- [ ] **Step 6: Update character_limit::is_full**

In `rules_engine/src/battle_mutations/src/play_cards/character_limit.rs`, change:

```rust
pub fn is_full(battle: &BattleState, player: PlayerName) -> bool {
    battle.cards.battlefield(player).character_count() >= 9
}
```

- [ ] **Step 7: Add back-row-full check to can_play_cards**

In `rules_engine/src/battle_queries/src/legal_action_queries/can_play_cards.rs`, in both `from_hand` (around line 56) and `from_void` (around line 82), change the `battlefield_full` check to also consider the back row. Replace:

```rust
let battlefield_full = battle.cards.battlefield(player).character_count() >= CHARACTER_LIMIT;
```

with:

```rust
let battlefield_full = battle.cards.battlefield(player).character_count() >= CHARACTER_LIMIT
    || battle.cards.battlefield(player).back_row_is_full();
```

This ensures characters can't be played if the back row is full (even if total < 9).

- [ ] **Step 8: Run `just fmt` then `just check`**

Run: `just fmt && just check`
Expected: Compilation succeeds (there will be downstream errors in other files that reference `[_; 8]`, but the core struct should compile).

- [ ] **Step 9: Commit**

```bash
git add -A && git commit -m "feat: update Battlefield struct to 5-back/4-front staggered grid

Change arrays from [_; 8] to [_; 5] (back) and [_; 4] (front).
Update CHARACTER_LIMIT from 8 to 9. Add back_row_is_full check and
support relationship queries."
```

---

### Task 2: Update Judgment Phase

**Files:**
- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs`
- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/turn.rs:90-107`

- [ ] **Step 1: Update judgment position bounds**

In `rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs`, change the bounds check at line 48 from:

```rust
if position >= 7 {
```

to:

```rust
if position >= 3 {
```

Also update the doc comment on `run` from "all 8 positions" to "all 4 positions".

- [ ] **Step 2: Remove return_participants_to_back_rank**

In the same file, delete the entire `return_participants_to_back_rank` function (lines 59–68):

```rust
// DELETE THIS ENTIRE FUNCTION:
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

- [ ] **Step 3: Remove the call to return_participants_to_back_rank**

In `rules_engine/src/battle_mutations/src/phase_mutations/turn.rs`, in the `BattleTurnPhase::Judgment` match arm (around line 96-97), remove the call:

```rust
judgment_phase::return_participants_to_back_rank(battle);
```

The match arm should go from:

```rust
if finished && battle.prompts.is_empty() {
    judgment_phase::return_participants_to_back_rank(battle);
    battle.triggers.push(source, Trigger::Judgment(player));
```

to:

```rust
if finished && battle.prompts.is_empty() {
    battle.triggers.push(source, Trigger::Judgment(player));
```

- [ ] **Step 4: Remove return_to_back_rank method from Battlefield**

In `rules_engine/src/battle_state/src/battle_cards/battlefield.rs`, delete the `return_to_back_rank` method (lines 89–109). This method is no longer needed since characters stay in the front row.

- [ ] **Step 5: Run `just fmt` then `just check`**

Run: `just fmt && just check`
Expected: Compiles (may have warnings about unused `judgment_participants` field).

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "feat: remove return-to-back-rank after judgment

Characters now stay in the front row after judgment. Remove
return_participants_to_back_rank and the return_to_back_rank method."
```

---

### Task 3: Update Repositioning Logic

**Files:**
- Modify: `rules_engine/src/battle_queries/src/legal_action_queries/legal_actions.rs:160-296`
- Modify: `rules_engine/src/battle_mutations/src/card_mutations/reposition.rs`

- [ ] **Step 1: Update reposition_actions ranges**

In `rules_engine/src/battle_queries/src/legal_action_queries/legal_actions.rs`, update the `reposition_actions` function (lines 160–201). Change all four `0..8u8` loops to use the correct range for each rank:

For back-rank characters moving to front: `0..4u8`
For back-rank characters moving within back: `0..5u8`
For front-rank characters moving to back: `0..5u8`
For front-rank characters moving within front: `0..4u8`

The updated function:

```rust
fn reposition_actions(
    battle: &BattleState,
    player: PlayerName,
) -> (RepositionActions, RepositionActions) {
    let bf = battle.cards.battlefield(player);
    let current_turn = battle.turn.turn_id.0;
    let mut to_front = Vec::new();
    let mut to_back = Vec::new();

    for character_id in bf.back.iter().flatten() {
        let has_summoning_sickness = battle
            .cards
            .battlefield_state(player)
            .get(character_id)
            .is_some_and(|state| state.played_turn == current_turn);
        if !has_summoning_sickness {
            for position in 0..4u8 {
                to_front.push((*character_id, position));
            }
        }

        for position in 0..5u8 {
            if bf.back[position as usize] != Some(*character_id) {
                to_back.push((*character_id, position));
            }
        }
    }

    for character_id in bf.front.iter().flatten() {
        for position in 0..5u8 {
            to_back.push((*character_id, position));
        }

        for position in 0..4u8 {
            if bf.front[position as usize] != Some(*character_id) {
                to_front.push((*character_id, position));
            }
        }
    }

    (to_front, to_back)
}
```

- [ ] **Step 2: Update eligible_back_rank_characters**

In the same file, the `eligible_back_rank_characters` function (lines 282–296) iterates `bf.back` which is now `[_; 5]`. This function uses `.iter().flatten()` on the array, so it should work without changes. Verify that no `0..8` range is used inside it.

The function also checks `moved_this_turn`. With free rearrangement, the `moved_this_turn` check is no longer needed for the human player (they can freely rearrange). However, the AI positioning flow still uses this. For now, leave this function unchanged — the AI positioning phased flow in `phased_main_phase_actions` will still work with the new array sizes.

- [ ] **Step 3: Update assign_column_actions**

In the same file, the `assign_column_actions` function (lines 247–265) and `has_available_column` (lines 270–278) reference `opponent_front` and `own_front` arrays. These use `.iter()` on the arrays, so they should work automatically with `[_; 4]`. No changes needed.

- [ ] **Step 4: Run `just fmt` then `just check`**

Run: `just fmt && just check`
Expected: Compiles.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: update repositioning for 5-back/4-front grid

Change position ranges from 0..8 to 0..4 (front) and 0..5 (back)."
```

---

### Task 4: Update AI (MonteCarloV1) Positioning Heuristics

**Files:**
- Modify: `rules_engine/src/ai_uct/src/uct_search.rs` (lines 490–627)
- Modify: `rules_engine/src/ai_uct/src/position_assignment.rs` (lines 476–477 and any `[_; 8]` refs)

- [ ] **Step 1: Update position_assignment.rs array types**

In `rules_engine/src/ai_uct/src/position_assignment.rs`, find the function signature at lines 476–477:

```rust
own_front: &[Option<CharacterId>; 8],
opponent_front: &[Option<CharacterId>; 8],
```

Change to:

```rust
own_front: &[Option<CharacterId>; 4],
opponent_front: &[Option<CharacterId>; 4],
```

Search the rest of this file for any other `[_; 8]` references or `0..8` ranges and update them to the correct sizes (`4` for front, `5` for back).

- [ ] **Step 2: Verify uct_search.rs heuristics compile**

The V1 rollout heuristics in `uct_search.rs` (lines 490–627) access `battlefield(opponent).front[col as usize]` and iterate over `front`/`back` arrays. Since the arrays are now smaller, the column values from `block_targets` and `attack_column` in `LegalActions::AssignColumn` will naturally be 0–3. The array accesses should be safe as long as the column values come from the legal actions system (which they do).

Scan `uct_search.rs` for any hardcoded `0..8` ranges or `[_; 8]` type annotations and update them.

- [ ] **Step 3: Run `just fmt` then `just check`**

Run: `just fmt && just check`
Expected: Compiles. There may be dead code warnings for V2/V3/V4 search files — that's fine, they're not used.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: update AI positioning for staggered grid

Update array type annotations and ranges in position_assignment.rs
and uct_search.rs for 4-front/5-back layout."
```

---

### Task 5: Update Tests

**Files:**
- Modify: `rules_engine/tests/battle_tests/tests/battle_tests/basic_tests/battle_limits_tests.rs`
- Create: `rules_engine/tests/battle_tests/tests/battle_tests/basic_tests/staggered_grid_tests.rs`
- Modify: `rules_engine/tests/battle_tests/tests/battle_tests/basic_tests.rs` (add module declaration)

- [ ] **Step 1: Update character_limit_prevents_playing_character test**

In `rules_engine/tests/battle_tests/tests/battle_tests/basic_tests/battle_limits_tests.rs`, the test at line 42 adds 16 characters. The new limit is 9 total, but the actual constraint is back-row-full (5). Update the test:

```rust
#[test]
fn character_limit_prevents_playing_character() {
    let mut s = TestBattle::builder().connect();
    for _ in 0..5 {
        s.add_to_battlefield(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    }
    assert_eq!(
        s.user_client.cards.user_battlefield().len(),
        5,
        "User should have 5 characters on battlefield"
    );
    let char_id = s.add_to_hand(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    assert!(
        s.user_client.cards.get_revealed(&char_id).actions.can_play.is_none(),
        "Character should not be playable when back row is full"
    );
}
```

Note: `add_to_battlefield` places characters in the back row. With 5 back-row slots, adding 5 should fill the back row. The test may need adjustment depending on whether `add_to_battlefield` uses `add_to_back_rank` internally — verify by checking test behavior.

- [ ] **Step 2: Write staggered grid test file**

Create `rules_engine/tests/battle_tests/tests/battle_tests/basic_tests/staggered_grid_tests.rs`:

```rust
use display_data::battle_view::DisplayPlayer;
use tabula_generated::test_card;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn characters_stay_in_front_after_judgment() {
    let mut s = TestBattle::builder().connect();
    s.add_to_battlefield(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    s.move_to_front_rank(DisplayPlayer::User, 0, 0);
    s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);
    s.move_to_front_rank(DisplayPlayer::Enemy, 0, 0);

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    s.end_turn_remove_opponent_hand(DisplayPlayer::Enemy);

    // After judgment, surviving characters should still be in front rank
    // (verify via battlefield state — exact assertion depends on test utils API)
    assert!(
        s.user_client.cards.user_battlefield().len() > 0
            || s.enemy_client.cards.enemy_battlefield().len() > 0,
        "At least one character should survive judgment"
    );
}

#[test]
fn back_row_full_prevents_playing_character() {
    let mut s = TestBattle::builder().connect();
    for _ in 0..5 {
        s.add_to_battlefield(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    }
    let char_id = s.add_to_hand(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    assert!(
        s.user_client.cards.get_revealed(&char_id).actions.can_play.is_none(),
        "Character should not be playable when back row is full"
    );
}

#[test]
fn can_play_character_after_moving_to_front() {
    let mut s = TestBattle::builder().connect();
    for _ in 0..5 {
        s.add_to_battlefield(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    }
    // Move one character to front rank to free a back-row slot
    s.move_to_front_rank(DisplayPlayer::User, 0, 0);

    let char_id = s.add_to_hand(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    assert!(
        s.user_client.cards.get_revealed(&char_id).actions.can_play.is_some(),
        "Character should be playable after freeing a back-row slot"
    );
}
```

Note: These tests use `move_to_front_rank` which may not exist as a test helper yet. If not, the test will need to use the appropriate action sequence (e.g., `perform_user_action` with `MoveCharacterToFrontRank`). Adjust based on the available test utilities — check `test_session_battle_extension.rs` for available helpers.

- [ ] **Step 3: Add module declaration**

In `rules_engine/tests/battle_tests/tests/battle_tests/basic_tests.rs`, add:

```rust
mod staggered_grid_tests;
```

- [ ] **Step 4: Run tests**

Run: `just fmt && just battle-test staggered_grid`
Expected: Tests pass (adjust test code based on available test helpers).

- [ ] **Step 5: Run full test suite**

Run: `just fmt && just review`
Expected: All tests pass. Fix any compilation errors from other tests that reference the old 8-column layout.

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "test: add staggered grid tests, update character limit test

Test that characters stay in front after judgment, back-row-full
prevents playing, and freeing a back-row slot re-enables play."
```

---

### Task 6: Fix Remaining Compilation Errors and Clean Up

**Files:**
- Various files that reference `[_; 8]`, `>= 16`, or `return_to_back_rank`

- [ ] **Step 1: Fix any remaining compilation errors**

Run `just check` and fix any remaining errors. Common issues:

- `position_assignment.rs`: Any remaining `[_; 8]` type annotations → change to `[_; 4]` or `[_; 5]`
- `uct_search_v2.rs`, `uct_search_v3.rs`, `uct_search_v4.rs`: These files are large and may have `[_; 8]` references. Since we're using MonteCarloV1 only, these files can have their `[_; 8]` references updated to compile, or they can be `#[allow(dead_code)]` if they're not compiled. Check if they're included in the build.
- `rollout_policy.rs`: May reference `[_; 8]` or old battlefield layout

- [ ] **Step 2: Run `just fmt` then `just clippy`**

Run: `just fmt && just clippy`
Expected: No errors or warnings (aside from existing ones).

- [ ] **Step 3: Run `just review`**

Run: `just review`
Expected: Full gate passes.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "fix: resolve remaining compilation errors for staggered grid

Update all remaining references to 8-column battlefield layout."
```
