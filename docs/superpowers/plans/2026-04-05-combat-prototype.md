# Dreamtides Combat Prototype Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add positional front/back rank combat to Dreamtides, replacing spark-comparison scoring with per-column Judgment resolution.

**Architecture:** Battlefield changes from `CardSet<CharacterId>` to a `Battlefield` struct with `[Option<CharacterId>; 8]` arrays for front/back ranks. New Judgment phase at end of turn resolves each column independently. Turn order becomes Dreamwell → Draw → Dawn → Main → Judgment. AI gets simplified repositioning actions to avoid MCTS branching explosion.

**Tech Stack:** Rust (rules_engine), TypeScript/React (battle prototype), TOML (card data)

**Spec:** `docs/superpowers/specs/2026-04-05-combat-prototype-design.md`

**Key constraint:** NO automated tests. All validation through manual QA using `agent-browser` CLI against the battle prototype. Every implementation task is followed by a QA verification pass. QA subagents follow the full adversarial QA protocol: screenshots after every action, pre-committed predictions, invariant tracking, and never rationalizing anomalies.

**QA tool:** `/Users/dthurn/Library/pnpm/agent-browser`

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

## QA Protocol for Every Verification Step

Every "QA VERIFY" step in this plan dispatches a QA subagent that follows this protocol:

1. **Open the battle prototype** at the dev server URL using `agent-browser open`
2. **Take a screenshot and READ it** — visual inspection is primary, not DOM snapshots
3. **Establish invariants** for the feature under test (counts, scores, zone contents)
4. **For each test scenario:**
   - STATE: Write current values
   - PREDICT: Write what should happen
   - ACT: Perform the action
   - SCREENSHOT + READ: Take screenshot after every action
   - MEASURE: Re-check invariants via `agent-browser eval`
   - COMPARE: Predicted vs actual
   - VERDICT: PASS only if all predictions match; otherwise BUG
5. **Never rationalize anomalies** — wrong numbers, garbled text, layout issues are bugs
6. **Write report** to `/tmp/qa-report-taskN.md` with bugs, UX issues, and passed scenarios
7. **Play through extensively** — never just 2-3 turns. Minimum 10 turns for game loop tests.

The QA subagent MUST NOT read source code. It tests the application as a user would.

---

## Task 1: QA Baseline — Current Game State

**Before any code changes, establish a QA baseline of the current game.**

- [ ] **Step 1: Start the battle prototype servers**

Start both the Rust dev server and the TypeScript frontend. Verify the game loads.

- [ ] **Step 2: QA VERIFY — Baseline gameplay**

Dispatch QA subagent:
- URL: battle prototype (default http://localhost:5174 or check scripts)
- Area: Full game — play 5+ turns to establish baseline behavior
- Invariants to track:
  - Score only changes during judgment phase
  - Energy resets each turn
  - Cards drawn each turn
  - Characters appear on battlefield when played
- Document: current turn structure, scoring behavior, UI layout
- Take screenshots of every phase transition
- Write report to `/tmp/qa-report-baseline.md`

Purpose: This baseline lets us compare before/after and catch regressions.

- [ ] **Step 3: Save baseline report and commit**

---

## Task 2: Add Vanilla Characters to Card Pool

**Files:**
- Modify: `client/Assets/StreamingAssets/Tabula/cards.toml` (append after last card)
- Modify: `client/Assets/StreamingAssets/Tabula/card-lists.toml` (insert before `[metadata]`)

- [ ] **Step 1: Add 6 vanilla character cards to cards.toml**

Append 6 cards with no abilities (empty rules-text), card numbers 223-228:

| Name              | ID (UUID)                              | Cost | Spark | Rarity   |
|-------------------|----------------------------------------|------|-------|----------|
| Duskborne Sentry  | a1b2c3d4-1111-4000-8000-000000000001   | 1    | 1     | Common   |
| Glimmer Scout     | a1b2c3d4-2222-4000-8000-000000000002   | 2    | 2     | Common   |
| Veilward Knight   | a1b2c3d4-3333-4000-8000-000000000003   | 3    | 3     | Common   |
| Embertide Warrior | a1b2c3d4-4444-4000-8000-000000000004   | 4    | 5     | Uncommon |
| Starforged Titan  | a1b2c3d4-5555-4000-8000-000000000005   | 5    | 7     | Uncommon |
| Abyssal Colossus  | a1b2c3d4-7777-4000-8000-000000000006   | 7    | 10    | Rare     |

All cards: `card-type = "Character"`, `is-fast = false`, `image-number = 0`, `art-owned = false`, `subtype = ""`, `variables = ""`.

- [ ] **Step 2: Add vanilla characters to the Core 11 card list**

In `card-lists.toml`, insert entries before `[metadata]` with copies: 8, 8, 6, 6, 4, 4 respectively.

- [ ] **Step 3: Regenerate parsed data**

Run: `just tabula-generate`

- [ ] **Step 4: QA VERIFY — Vanilla characters appear in game**

Dispatch QA subagent:
- Start a new game and play several turns
- Verify vanilla characters appear in hand (no abilities, just name + cost + spark)
- Verify they can be played and appear on battlefield
- Verify their spark values display correctly
- Check that they have no rules text (empty)
- Report to `/tmp/qa-report-task2.md`

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: add 6 vanilla characters to Core 11 for combat testing"
```

---

## Task 3: Battlefield Data Model

**Files:**
- Create: `rules_engine/src/battle_state/src/battle_cards/battlefield.rs`
- Modify: `rules_engine/src/battle_state/src/battle_cards/mod.rs`
- Modify: `rules_engine/src/battle_state/src/battle/all_cards.rs`
- Modify: `rules_engine/src/battle_state/src/battle_cards/character_state.rs`

- [ ] **Step 1: Create the Battlefield struct**

Create `rules_engine/src/battle_state/src/battle_cards/battlefield.rs` with:
- `Battlefield { front: [Option<CharacterId>; 8], back: [Option<CharacterId>; 8] }`
- Methods: `character_count()`, `is_full()`, `first_empty_back_slot()`, `first_empty_front_slot()`, `contains()`, `remove()`, `add_to_back_rank()`, `is_in_front_rank()`, `is_in_back_rank()`, `all_characters()`
- Derive Clone, Debug, Serialize, Deserialize. Manual Default impl (arrays of None).

- [ ] **Step 2: Add module declaration**

In `battle_cards/mod.rs`, add `pub mod battlefield;`

- [ ] **Step 3: Add played_turn to CharacterState**

In `character_state.rs`, add `pub played_turn: u32` field with doc comment for summoning sickness.

- [ ] **Step 4: Replace battlefield storage in AllCards**

In `all_cards.rs`:
- Replace `battlefield: PlayerMap<CardSet<CharacterId>>` with `battlefield: PlayerMap<Battlefield>`
- Update `battlefield()` to return `&Battlefield`
- Add `battlefield_mut()` returning `&mut Battlefield`
- Update `all_battlefield_characters()` to use `Battlefield::all_characters()`
- Update `add_to_zone()` Battlefield case to use `add_to_back_rank()`
- Update `remove_from_zone()` Battlefield case to use `battlefield.remove()`
- Update `contains_card()` Battlefield case to use `battlefield.contains()`

- [ ] **Step 5: Fix all compilation errors**

Run `just check` and fix every compilation error. Key call sites:
- `player_properties.rs` — `spark_total()` iterates battlefield
- `legal_actions.rs` — character ability enumeration
- `character_limit.rs` — character count
- `move_card.rs` — `on_enter_battlefield()` and `on_leave_battlefield()`
- `display_data` — ObjectPosition::OnBattlefield construction
- Any query iterating battlefield as a CardSet

Replace CardSet iteration with `battlefield.all_characters()` or appropriate methods.

- [ ] **Step 6: Run `just fmt` then `just review`**

- [ ] **Step 7: QA VERIFY — Game still works after data model change**

Dispatch QA subagent:
- Play 5+ turns to verify no regressions from data model refactor
- Characters should still appear on battlefield when played
- Scoring should still work (old judgment for now)
- No crashes, no missing characters, no incorrect counts
- Report to `/tmp/qa-report-task3.md`

- [ ] **Step 8: Commit**

```bash
git commit -m "feat: replace battlefield CardSet with ranked Battlefield struct"
```

---

## Task 4: Remove Spark Bonus and Old Character Limit

**Files:**
- Modify: `rules_engine/src/battle_state/src/battle_player/battle_player_state.rs`
- Modify: `rules_engine/src/battle_queries/src/battle_player_queries/player_properties.rs`
- Modify: `rules_engine/src/battle_mutations/src/play_cards/character_limit.rs`
- Modify: `rules_engine/src/battle_queries/src/legal_action_queries/legal_actions.rs`

- [ ] **Step 1: Remove spark_bonus from BattlePlayerState**

Delete `pub spark_bonus: Spark` field. Fix all compilation errors.

- [ ] **Step 2: Simplify spark_total()**

Update to only sum character spark values:
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

Change CHARACTER_LIMIT to 16. Replace abandon logic with `is_full()` check. Remove the old `apply()` function. Update legal actions to filter out character cards when battlefield is full.

- [ ] **Step 4: Run `just fmt` then `just review`**

- [ ] **Step 5: QA VERIFY — Scoring and character limits**

Dispatch QA subagent:
- Play 5+ turns, verify scoring still works without spark bonus
- Try to accumulate many characters — should be able to have more than 8 now
- Verify no abandon/overflow happens
- Report to `/tmp/qa-report-task4.md`

- [ ] **Step 6: Commit**

```bash
git commit -m "feat: remove spark bonus and change character limit to 16"
```

---

## Task 5: Phase and Trigger Renames

**Files:**
- Modify: `rules_engine/src/battle_state/src/battle/battle_turn_phase.rs`
- Modify: `rules_engine/src/battle_state/src/triggers/trigger.rs`
- Rename: `judgment_phase.rs` → `dawn_phase.rs` in `phase_mutations/`
- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/turn.rs`
- Modify: All files referencing old Judgment phase/trigger names

- [ ] **Step 1: Update BattleTurnPhase enum**

New order: Starting, Dreamwell, Draw, Dawn, Main, Judgment, Ending, EndingPhaseFinished, FiringEndOfTurnTriggers

- [ ] **Step 2: Rename Trigger::Judgment → Trigger::Dawn, add new Trigger::Judgment**

Update both `Trigger` and `TriggerName` enums. Update the `name()` method.

- [ ] **Step 3: Rename judgment_phase.rs to dawn_phase.rs**

Update module declaration. Remove scoring logic from Dawn — it's now just a trigger window:
```rust
pub fn run(battle: &mut BattleState, player: PlayerName, source: EffectSource) {
    battle.push_animation(source, || BattleAnimation::Dawn { player });
    battle.triggers.push(source, Trigger::Dawn(player));
}
```

- [ ] **Step 4: Reorder turn state machine**

Update `run_turn_state_machine_if_no_active_prompts()`:
```
Starting → Dreamwell
Dreamwell → Draw  
Draw → Dawn
Dawn → Main
Main → Judgment (stub: just transition to Ending for now)
Judgment → Ending
```

- [ ] **Step 5: Fix all compilation errors from renames**

Update every reference to old `BattleTurnPhase::Judgment`, `Trigger::Judgment`, and `judgment_phase` module. Key: card ability definitions referencing `TriggerName::Judgment` → `TriggerName::Dawn`.

- [ ] **Step 6: Run `just fmt` then `just review`**

- [ ] **Step 7: QA VERIFY — Turn structure and Dawn phase**

Dispatch QA subagent:
- Play 5+ turns and observe the turn structure
- Verify phases happen in new order: Dreamwell → Draw → Dawn → Main → End
- Dawn should fire triggers but NOT score points
- No scoring should happen yet (judgment is stubbed)
- Verify the game doesn't crash or hang between phases
- Report to `/tmp/qa-report-task5.md`

- [ ] **Step 8: Commit**

```bash
git commit -m "feat: rename Judgment to Dawn, add new Judgment phase at end of turn"
```

---

## Task 6: ObjectPosition Changes for Rank/Position

**Files:**
- Modify: `rules_engine/src/display_data/src/object_position.rs`
- Modify: `scripts/battle_prototype/src/types/battle.ts`
- Modify: All Rust code constructing `Position::OnBattlefield`

- [ ] **Step 1: Add Rank enum and update Position::OnBattlefield**

In `object_position.rs`:
```rust
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub enum Rank {
    Front,
    Back,
}
```

Change `OnBattlefield(DisplayPlayer)` to `OnBattlefield(DisplayPlayer, Rank, u8)`.

- [ ] **Step 2: Update all construction sites**

Find every `Position::OnBattlefield(player)` and update to include rank and position. Look up the character's rank/position from the Battlefield struct.

- [ ] **Step 3: Update TypeScript types**

In `battle.ts`, change OnBattlefield from `{ OnBattlefield: DisplayPlayer }` to:
```typescript
{ OnBattlefield: { player: DisplayPlayer, rank: "Front" | "Back", position: number } }
```

- [ ] **Step 4: Update BattleScreen.tsx position filtering**

Update `cardsByPosition()` helper and all references that filter by OnBattlefield.

- [ ] **Step 5: Run `just fmt`, `just review`, and `npx tsc --noEmit`**

- [ ] **Step 6: QA VERIFY — Position data correctness**

Dispatch QA subagent:
- Play characters and verify they appear in the UI
- Use `agent-browser eval` to inspect card position data in the DOM/state
- Verify position includes rank ("Back") and slot index
- Verify cards still render in the battlefield area
- Report to `/tmp/qa-report-task6.md`

- [ ] **Step 7: Commit**

```bash
git commit -m "feat: add rank and position to OnBattlefield position data"
```

---

## Task 7: Battle Prototype UI — Rank Rendering

**Files:**
- Modify: `scripts/battle_prototype/src/components/BattleScreen.tsx`
- Modify: `scripts/battle_prototype/src/components/BattlefieldZone.tsx` (rewrite as RankZone)
- Modify: `scripts/battle_prototype/src/components/CardDisplay.tsx`

- [ ] **Step 1: Rewrite BattlefieldZone as RankZone with 8 fixed slots**

Each rank renders 8 fixed-width slots. Empty slots are dashed outlines. Cards placed by their position index from OnBattlefield data.

- [ ] **Step 2: Update BattleScreen layout to 4 ranks**

Replace 2 battlefield zones with 4 rank zones + judgment line separator:
1. Enemy status → Enemy back rank → Enemy front rank → Judgment line → Your front rank → Your back rank → Your status → Hand → Actions → Log

Helper to filter by rank:
```tsx
function cardsByRank(cards: CardView[], player: DisplayPlayer, rank: "Front" | "Back"): CardView[]
```

- [ ] **Step 3: Add judgment line separator**

Gold-colored divider with "⚡ JUDGMENT LINE ⚡" text between enemy front and your front rank.

- [ ] **Step 4: Add summoning sickness visual indicator**

Add `summoning_sick: bool` to card view data from Rust. Show visual indicator (e.g., blue tint or icon) on summoning-sick characters.

- [ ] **Step 5: Verify TypeScript compiles**

- [ ] **Step 6: QA VERIFY — Rank rendering**

Dispatch QA subagent with full QA protocol:
- Verify 4 rank zones display (enemy back, enemy front, your front, your back)
- Each rank shows 8 slots with dashed outlines for empty ones
- Labels visible and correct for each rank
- Judgment line separator visible between front ranks
- Characters appear in correct rank (should be back rank when played)
- Spark values display on battlefield cards
- Layout is readable, not cramped, no overlapping
- All text is readable (no garbled characters)
- Take screenshots of: empty board, 1 character placed, multiple characters
- Report to `/tmp/qa-report-task7.md`

- [ ] **Step 7: Fix bugs from QA report**

- [ ] **Step 8: Commit**

```bash
git commit -m "feat: render four battlefield ranks with judgment line separator"
```

---

## Task 8: Extended Debug Tools for Combat Testing

**Files:**
- Modify: `rules_engine/src/battle_state/src/actions/debug_battle_action.rs`
- Modify: `rules_engine/src/battle_mutations/src/actions/apply_debug_battle_action.rs`
- Modify: `scripts/battle_prototype/src/components/DebugPanel.tsx`

The existing debug system has `AddCardToHand`, `AddCardToBattlefield`, `SetEnergy`,
`DrawCard`, etc. We need combat-specific debug tools for QA.

- [ ] **Step 1: Add new DebugBattleAction variants**

In `debug_battle_action.rs`, add:

```rust
/// Place a character directly into the front rank at a specific position.
/// Bypasses summoning sickness.
AddCardToFrontRank { player: PlayerName, card: BaseCardId, position: u8 },

/// Place a character directly into the back rank at a specific position.
AddCardToBackRank { player: PlayerName, card: BaseCardId, position: u8 },

/// Skip directly to the Judgment phase, resolving all front-rank combat.
SkipToJudgment,
```

- [ ] **Step 2: Implement handlers in apply_debug_battle_action.rs**

For `AddCardToFrontRank` / `AddCardToBackRank`:
1. Create the card via `battle_deck::debug_add_cards()`
2. Move from deck to battlefield using existing `move_card::from_deck_to_battlefield()`
3. Then reposition the character from its default back-rank slot to the target rank/position

For `SkipToJudgment`:
1. Set `battle.phase = BattleTurnPhase::Judgment`
2. The state machine will pick it up on the next tick and call `judgment_phase::run()`

- [ ] **Step 3: Add combat-focused debug buttons to DebugPanel.tsx**

Add these buttons to the DEBUG_BUTTONS array:

```tsx
{
  label: "Add Sentry (1✦) to Enemy Front 0",
  action: { BattleAction: { Debug: {
    AddCardToFrontRank: { player: "Two", card: "a1b2c3d4-1111-4000-8000-000000000001", position: 0 }
  }}}
},
{
  label: "Add Knight (3✦) to Enemy Front 1",
  action: { BattleAction: { Debug: {
    AddCardToFrontRank: { player: "Two", card: "a1b2c3d4-3333-4000-8000-000000000003", position: 1 }
  }}}
},
{
  label: "Add Titan (7✦) to Enemy Front 2",
  action: { BattleAction: { Debug: {
    AddCardToFrontRank: { player: "Two", card: "a1b2c3d4-5555-4000-8000-000000000005", position: 2 }
  }}}
},
{
  label: "Add Colossus (10✦) to Enemy Front 3",
  action: { BattleAction: { Debug: {
    AddCardToFrontRank: { player: "Two", card: "a1b2c3d4-7777-4000-8000-000000000006", position: 3 }
  }}}
},
{
  label: "Add Scout (2✦) to Your Front 0",
  action: { BattleAction: { Debug: {
    AddCardToFrontRank: { player: "One", card: "a1b2c3d4-2222-4000-8000-000000000002", position: 0 }
  }}}
},
{
  label: "Add Warrior (5✦) to Your Front 1",
  action: { BattleAction: { Debug: {
    AddCardToFrontRank: { player: "One", card: "a1b2c3d4-4444-4000-8000-000000000004", position: 1 }
  }}}
},
{
  label: "Add Vanilla to Hand",
  action: { BattleAction: { Debug: {
    AddCardToHand: { player: "One", card: "a1b2c3d4-3333-4000-8000-000000000003" }
  }}}
},
{
  label: "Skip to Judgment",
  action: { BattleAction: { Debug: "SkipToJudgment" } }
},
```

These let QA set up exact board states: specific characters at specific positions
in specific ranks, then trigger judgment to see the outcome.

- [ ] **Step 4: Run `just fmt` then `just review`**

- [ ] **Step 5: QA VERIFY — Debug tools work**

Dispatch QA subagent:
- Open game, open debug panel
- Click "99 Energy" → verify energy display changes
- Click "Add Sentry (1✦) to Enemy Front 0" → verify enemy front rank slot 0 shows a character
- Click "Add Scout (2✦) to Your Front 0" → verify your front rank slot 0 shows a character
- Click "Skip to Judgment" → verify judgment resolves (Scout 2✦ vs Sentry 1✦ → Sentry dissolved)
- Verify score increased by correct amount for uncontested characters
- Test placing characters in multiple positions
- Test adding multiple enemy characters then running judgment
- Report to `/tmp/qa-report-task8.md`

- [ ] **Step 6: Commit**

```bash
git commit -m "feat: add combat-focused debug tools for QA testing

AddCardToFrontRank, AddCardToBackRank, SkipToJudgment debug actions.
Debug panel buttons for placing vanilla characters at specific positions
and triggering judgment phase for combat testing."
```

---

## Task 9: QA — Character Placement Deep Test (Using Debug Tools)

**Dedicated QA milestone — use debug tools to set up exact board states.**

- [ ] **Step 1: QA VERIFY — Character placement and debug tool validation**

Dispatch QA subagent with extended protocol:
- Play 8+ turns, playing characters each turn
- **Use debug tools to test specific scenarios:**
  - "99 Energy" then play multiple vanilla characters in one turn
  - "Add Vanilla to Hand" to get specific characters
  - "Add Sentry to Enemy Front 0" → verify enemy character appears in correct slot
  - Place characters in multiple specific positions using debug buttons
  - "Skip to Judgment" to verify judgment works with debug-placed characters
- **Invariants to track:**
  - Characters played from hand always appear in back rank
  - Debug-placed characters appear in the specified rank/slot
  - Total characters on battlefield matches expected count
  - Spark values on battlefield match card spark
  - Hand count decreases by 1 when a card is played from hand
  - Energy decreases by card cost when played from hand
- Verify enemy AI also places characters in back rank
- Test playing events (should NOT appear on battlefield)
- Take screenshots after every action
- Report to `/tmp/qa-report-task9.md`

- [ ] **Step 2: Fix any bugs found and commit**

---

## Task 10: New Judgment Phase Resolution

**Files:**
- Create: `rules_engine/src/battle_mutations/src/phase_mutations/judgment_phase.rs`
- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/mod.rs`
- Modify: `rules_engine/src/battle_mutations/src/phase_mutations/turn.rs`

- [ ] **Step 1: Create judgment_phase.rs**

Implement the column-by-column combat resolution:

```rust
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
                    dissolve::run(battle, defender, source, ShouldAnimate::Yes);
                    dissolve::run(battle, attacker, source, ShouldAnimate::Yes);
                }
            }
            (Some(attacker), None) => {
                let spark = battle.cards.spark(attacker);
                points::gain(battle, player, source, Points(spark.0), ShouldAnimate::Yes);
            }
            _ => {}
        }
    }
    battle.triggers.push(source, Trigger::Judgment(player));
}
```

Dissolved triggers fire after each dissolution because `dissolve::run()` pushes `Trigger::Dissolved` internally. Verify this — if triggers are queued and processed later, restructure as a state machine processing one position per tick.

- [ ] **Step 2: Wire into turn state machine**

In `turn.rs`, replace the Judgment stub with `judgment_phase::run()`.

- [ ] **Step 3: Run `just fmt` then `just review`**

- [ ] **Step 4: QA VERIFY — Judgment phase resolution**

Dispatch QA subagent with extensive testing:
- Play 10+ turns, manually moving characters to front rank (if repositioning works) or observing AI
- **Scenarios to test:**
  - Two characters facing each other: higher spark wins
  - Two characters with equal spark: both dissolve
  - Uncontested character (no opponent across): scores victory points
  - Empty columns: nothing happens
  - Only defender present (opponent has character, you don't): nothing happens
- **Invariants:**
  - Score only increases by exact spark value of uncontested characters
  - Dissolved characters disappear from battlefield
  - Dissolved character's slot becomes empty
  - Score changes visible in player status bar
- Take screenshots of judgment resolution at every turn
- Verify battle log shows judgment results
- Report to `/tmp/qa-report-task9.md`

- [ ] **Step 5: Fix bugs from QA report**

- [ ] **Step 6: Commit**

```bash
git commit -m "feat: implement Judgment phase with front-rank combat resolution"
```

---

## Task 11: Character Materialization to Back Rank

**Files:**
- Modify: `rules_engine/src/battle_mutations/src/card_mutations/move_card.rs`

- [ ] **Step 1: Update on_enter_battlefield()**

Place character in first available back rank slot via `battlefield.add_to_back_rank()`. Set `played_turn` on CharacterState to current turn number.

- [ ] **Step 2: Update on_leave_battlefield()**

Remove character from Battlefield struct via `battlefield.remove()` instead of CardSet remove.

- [ ] **Step 3: Run `just fmt` then `just review`**

- [ ] **Step 4: QA VERIFY — Materialization targeting**

Dispatch QA subagent:
- Play 5+ turns, verify characters always enter back rank
- Verify effects that materialize from void (Reclaim) also go to back rank
- Verify dissolve removes character from correct slot
- Report to `/tmp/qa-report-task10.md`

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: characters materialize into back rank with played_turn tracking"
```

---

## Task 12: Repositioning Actions

**Files:**
- Modify: `rules_engine/src/battle_state/src/actions/battle_actions.rs`
- Create: `rules_engine/src/battle_mutations/src/card_mutations/reposition.rs`
- Modify: `rules_engine/src/battle_mutations/src/card_mutations/mod.rs`
- Modify: `rules_engine/src/battle_queries/src/legal_action_queries/legal_actions.rs`
- Modify: `rules_engine/src/battle_queries/src/legal_action_queries/legal_actions_data.rs`

- [ ] **Step 1: Add BattleAction variants**

```rust
MoveCharacterToFrontRank(CharacterId, u8),
MoveCharacterToBackRank(CharacterId, u8),
```

- [ ] **Step 2: Create reposition mutation**

`reposition.rs` with `to_front_rank()` and `to_back_rank()` functions. Include swap logic: when target slot is occupied, the occupant moves to the mover's original slot.

- [ ] **Step 3: Add repositioning to legal actions**

In `standard_legal_actions()`, populate reposition actions during Main phase:
- Back-rank characters without summoning sickness → can move to any front slot
- Front-rank characters → can move to any back slot
- Same-rank moves for repositioning within a rank

- [ ] **Step 4: Handle actions in the dispatcher**

Add match arms for `MoveCharacterToFrontRank` and `MoveCharacterToBackRank`.

- [ ] **Step 5: Update LegalActions methods**

Update `all()`, `len()`, `contains()`, `random_action()` to include reposition actions.

- [ ] **Step 6: Run `just fmt` then `just review`**

- [ ] **Step 7: QA VERIFY — Repositioning works**

Dispatch QA subagent:
- Verify characters can be moved from back to front rank (via UI buttons or actions)
- Verify summoning sickness blocks front-rank moves on the turn played
- Verify front-rank characters can retreat to back rank
- Verify position swaps work when moving to occupied slot
- Test: play character, try to move to front rank same turn → should fail
- Test: next turn, move to front rank → should succeed
- Report to `/tmp/qa-report-task11.md`

- [ ] **Step 8: Commit**

```bash
git commit -m "feat: add character repositioning between front and back ranks"
```

---

## Task 13: QA — Summoning Sickness Deep Test

**Dedicated QA milestone.**

- [ ] **Step 1: QA VERIFY — Summoning sickness edge cases**

Dispatch QA subagent:
- Play 10+ turns focusing on summoning sickness:
  - Play character turn N → can't move to front turn N → CAN move turn N+1
  - Play multiple characters same turn → all summoning sick
  - Character materialized from void (Reclaim) → also summoning sick?
  - Summoning sick character CAN be moved within back rank
  - Summoning sick indicator visible and clear
- **Invariants:**
  - No summoning-sick character ever appears in front rank on the turn it was played
  - Summoning sickness clears on the player's next turn
- Take screenshots showing summoning sickness indicator
- Report to `/tmp/qa-report-task12.md`

- [ ] **Step 2: Fix bugs and commit**

---

## Task 14: AI Simplified Actions

**Files:**
- Modify: `rules_engine/src/battle_queries/src/legal_action_queries/legal_actions.rs`
- Modify: `rules_engine/src/battle_queries/src/legal_action_queries/legal_actions_data.rs`
- Modify: `rules_engine/src/battle_state/src/battle/battle_state.rs` or turn data (add moved_this_turn)

- [ ] **Step 1: Add per-turn moved tracking**

Add `moved_this_turn` set (e.g., `Vec<CharacterId>` or `BTreeSet<CharacterId>`) to turn-scoped state. Clear at turn start.

- [ ] **Step 2: Implement AI action reduction**

When generating reposition actions for AI players:
- **MoveToEmptyFrontSlot(CharacterId)**: one action per eligible character (not per empty slot)
- **MoveToAttack(CharacterId, CharacterId)**: one action per (attacker, defender) pair
- **MoveToBack(CharacterId)**: only if character NOT in moved_this_turn

- [ ] **Step 3: Mark characters as moved after repositioning**

- [ ] **Step 4: Clear moved_this_turn at turn start**

- [ ] **Step 5: Run `just fmt` then `just review`**

- [ ] **Step 6: QA VERIFY — AI repositioning behavior**

Dispatch QA subagent:
- Play 10+ turns and observe AI behavior
- AI should move characters to front rank
- AI should attack enemy characters (move to occupied enemy positions)
- AI turns should complete in reasonable time (no infinite loop)
- AI should NOT repeatedly move the same character back and forth
- Report to `/tmp/qa-report-task13.md`

- [ ] **Step 7: Commit**

```bash
git commit -m "feat: simplified AI repositioning actions for MCTS"
```

---

## Task 15: Battle Prototype UI — Drag and Drop

**Files:**
- Modify: `scripts/battle_prototype/src/components/BattleScreen.tsx` or RankZone component
- Modify: `scripts/battle_prototype/src/components/CardDisplay.tsx`

- [ ] **Step 1: Implement HTML5 drag-and-drop**

Characters are `draggable={true}` during main phase. Empty slots and occupied slots are valid drop targets. On drop: dispatch `MoveCharacterToFrontRank` or `MoveCharacterToBackRank` action.

- [ ] **Step 2: Add visual drag feedback**

Highlight valid drop targets (green). Invalid targets (summoning sick to front) get red indicator. Drag cursor changes.

- [ ] **Step 3: Disable drag outside main phase**

Only enable when `can_act` is true and phase is Main.

- [ ] **Step 4: QA VERIFY — Drag and drop**

Dispatch QA subagent:
- Test dragging character from back rank to empty front slot → works
- Test dragging character from front rank to empty back slot → works  
- Test dragging to swap two characters in same rank → works
- Test dragging to swap between ranks → works
- Test dragging summoning-sick character to front → rejected
- Test dragging during opponent's turn → disabled
- Test rapid drag operations → no visual glitches
- Test dragging onto the judgment line → nothing happens
- Report to `/tmp/qa-report-task14.md`

- [ ] **Step 5: Fix bugs and commit**

```bash
git commit -m "feat: drag-and-drop character repositioning in battle prototype"
```

---

## Task 16: QA — Full Game Loop Round 1

**Dedicated QA milestone — play a complete game.**

- [ ] **Step 1: QA VERIFY — Complete game, 10+ turns**

Dispatch QA subagent with full adversarial QA protocol:
- Play a FULL game of 10+ turns against AI
- **Invariants to track continuously:**
  - Turn structure: Dreamwell → Draw → Dawn → Main → Judgment → End
  - Energy resets each turn to produced amount
  - Score only changes during judgment phase
  - Characters enter back rank only
  - Front rank characters fight during judgment
  - Back rank characters do nothing during judgment
  - Total characters per player ≤ 16
- **Test every feature:**
  - Play characters, events
  - Reposition characters via drag-and-drop
  - Observe judgment resolution
  - Observe AI play
  - Check battle log accuracy
- **Visual audit:**
  - All ranks labeled correctly
  - Judgment line visible
  - Score display accurate
  - Card counts match visible cards
- Take screenshots at EVERY phase transition and EVERY judgment resolution
- Report to `/tmp/qa-report-task15.md`

- [ ] **Step 2: Fix all bugs found**

- [ ] **Step 3: Commit fixes**

---

## Task 17: Kindle Effect Update

**Files:**
- Modify: `rules_engine/src/battle_mutations/src/effects/apply_standard_effect.rs`

- [ ] **Step 1: Implement Kindle as highest-spark targeting**

Find Kindle handler (may be `todo!()`). Implement: find character with highest spark, tiebreak by lowest `played_turn` (oldest). Add the Kindle amount to that character's spark.

- [ ] **Step 2: Run `just fmt` then `just review`**

- [ ] **Step 3: QA VERIFY — Kindle effect**

Dispatch QA subagent:
- Play until a Kindle effect triggers (may need specific cards)
- Verify spark goes to the highest-spark character, not leftmost
- If two characters have equal spark, verify it goes to the one played first
- Report to `/tmp/qa-report-task16.md`

- [ ] **Step 4: Commit**

```bash
git commit -m "feat: kindle targets highest-spark character instead of leftmost"
```

---

## Task 18: QA — Card Effects with Ranks

**Dedicated QA milestone — test card abilities interact correctly with rank system.**

- [ ] **Step 1: QA VERIFY — Card effect interactions**

Dispatch QA subagent:
- Play 10+ turns focused on triggering card abilities
- **Test scenarios:**
  - Dissolve effect removes character from correct rank slot
  - Materialize from void places character in back rank
  - Banish removes character from slot
  - Cards targeting "a character on the battlefield" can target both ranks
  - Triggered abilities (Materialized, Dissolved) fire correctly
  - Dawn triggers fire at start of turn
- **Invariants:**
  - After dissolve, slot becomes empty
  - After materialize, character appears in back rank
  - Card counts stay consistent across zones
- Report to `/tmp/qa-report-task17.md`

- [ ] **Step 2: Fix bugs and commit**

---

## Task 19: QA — UX Polish Round 1

**Dedicated QA milestone — focused on usability, not functionality.**

- [ ] **Step 1: QA VERIFY — Full UX audit**

Dispatch QA subagent:
- **Layout and readability:**
  - Are rank labels clear? Can you tell front from back?
  - Is the judgment line visually distinct?
  - Is the layout too tall for the screen? Do you need to scroll?
  - Can you see all 4 ranks simultaneously?
- **Information completeness:**
  - Can you tell which characters will fight during judgment? (column alignment)
  - Are spark values clearly visible on all battlefield characters?
  - Does the battle log explain what happened during judgment?
  - Can you see opponent's rank positions clearly?
- **Summoning sickness:**
  - Is the indicator clear? Can you tell which characters are sick?
  - Does the game communicate WHY you can't drag to front rank?
- **Drag and drop:**
  - Are valid drop targets obvious?
  - Is drag feedback clear?
  - Does the game feel responsive during repositioning?
- **Missing features:**
  - Is there a way to preview judgment matchups before ending turn?
  - Can you tell how many points you'd score from uncontested characters?
- Report to `/tmp/qa-report-task18.md`

- [ ] **Step 2: Implement UX improvements based on findings**

- [ ] **Step 3: Commit improvements**

---

## Task 20: QA — Extended Play Round 1 (15+ turns)

**Dedicated QA milestone — mid-game stress test.**

- [ ] **Step 1: QA VERIFY — 15+ turn game**

Dispatch QA subagent:
- Play a FULL game of 15+ turns
- Focus on mid-game board states with many characters
- **Watch for:**
  - Characters accumulating in both ranks
  - Complex judgment phases with many matchups
  - Score progression accuracy
  - AI making reasonable moves with many pieces
  - Performance — does the UI stay responsive?
  - Battle log getting long — is it still readable?
- Take screenshots of every judgment phase showing full board state
- Report to `/tmp/qa-report-task19.md`

- [ ] **Step 2: Fix bugs and commit**

---

## Task 21: QA — Extended Play Round 2 (Board Full States)

**Dedicated QA milestone — push to 16 characters.**

- [ ] **Step 1: QA VERIFY — Board full scenarios**

Dispatch QA subagent:
- Attempt to fill the battlefield to 16 characters
- **Test scenarios:**
  - Play characters until both ranks are full → can't play more
  - UI communicates why you can't play a character when full
  - Events can still be played when board is full
  - Characters die during judgment → can play more next turn
  - AI handles full board correctly
- **Invariants:**
  - Never more than 16 characters per player
  - Full board = 8 front + 8 back
- Report to `/tmp/qa-report-task20.md`

- [ ] **Step 2: Fix bugs and commit**

---

## Task 22: QA — Extended Play Round 3 (Judgment Edge Cases)

**Dedicated QA milestone — use debug tools to construct exact edge case board states.**

- [ ] **Step 1: QA VERIFY — Judgment edge cases using debug tools**

Dispatch QA subagent:
- **Use debug tools to set up specific scenarios:**
  - **Tie:** Place Veilward Knight (3✦) at Your Front 0, and Knight (3✦) at Enemy Front 0 → Skip to Judgment → both dissolve
  - **Asymmetric:** Place Titan (7✦) at Your Front 0, Sentry (1✦) at Enemy Front 0 → Skip to Judgment → Sentry dissolved
  - **Zero spark:** If possible, create 0-spark character via debug
  - **All 8 positions filled:** Use debug to fill all 8 front positions on both sides → Skip to Judgment → all 8 pairs resolve
  - **Mass scoring:** Fill your front rank with 8 characters, leave enemy front empty → Skip to Judgment → massive point gain (sum of all spark)
  - **Uncontested + contested mix:** Some columns have matchups, some are empty
  - **Empty board judgment:** No front rank characters → Skip to Judgment → nothing happens
- **Invariants:**
  - Ties always dissolve both
  - Uncontested scoring = exact spark value of each uncontested character
  - Higher spark always wins
  - Dissolved triggers fire between each column
- Take screenshot before and after each Skip to Judgment
- Report to `/tmp/qa-report-task22.md`

- [ ] **Step 2: Fix bugs and commit**

---

## Task 23: QA — AI Behavior Round 2

**Dedicated QA milestone — evaluate AI tactical quality.**

- [ ] **Step 1: QA VERIFY — AI tactics across multiple games**

Dispatch QA subagent:
- Play 3+ full games, observing AI closely
- **Evaluate:**
  - Does AI position high-spark characters against low-spark enemies?
  - Does AI leave uncontested characters to score?
  - Does AI retreat valuable characters when threatened?
  - Does AI use events alongside positioning?
  - Are AI turns completing in reasonable time?
  - Does AI avoid obviously bad moves (1-spark vs 10-spark)?
  - Does AI move characters to front rank at all?
  - Does AI ever have a full front rank?
- Report to `/tmp/qa-report-task22.md`

- [ ] **Step 2: Fix bugs and commit**

---

## Task 24: QA — UX Polish Round 2

**Dedicated QA milestone — second UX pass after extended play learnings.**

- [ ] **Step 1: QA VERIFY — Second UX audit**

Dispatch QA subagent:
- Based on all previous extended play:
  - Is judgment phase resolution clear to the player?
  - Does the battle log show enough info about judgment (which character beat which)?
  - Column alignment — can you visually trace which characters are paired?
  - Turn transitions — is it clear when judgment fires vs turn end?
  - Any confusion about front/back rank semantics?
  - Is the drag-and-drop discoverable for a new player?
  - Does the game communicate the risk of placing characters in front rank?
- Report to `/tmp/qa-report-task23.md`

- [ ] **Step 2: Implement improvements and commit**

---

## Task 25: QA — Extended Play Round 4 (20+ turns)

**Dedicated QA milestone — long game stress test.**

- [ ] **Step 1: QA VERIFY — 20+ turn endurance test**

Dispatch QA subagent:
- Play a full 20+ turn game
- Focus on late-game states:
  - Many characters on both sides
  - High scores approaching victory threshold
  - Board refilling after mass judgment dissolutions
  - AI behavior in complex late-game states
- **Watch for:**
  - Performance degradation
  - Memory issues (battle log growing huge)
  - Score overflow or display issues with large numbers
  - Any desynchronization between UI and game state
- Report to `/tmp/qa-report-task24.md`

- [ ] **Step 2: Fix bugs and commit**

---

## Task 26: QA — Extended Play Round 5 (Another Full Game)

**Dedicated QA milestone — fresh game with all fixes applied.**

- [ ] **Step 1: QA VERIFY — Clean full game**

Dispatch QA subagent:
- Play a fresh full game (15+ turns) with all previous fixes applied
- This is a regression check — verify nothing broken by previous fixes
- Track ALL invariants from earlier milestones
- Note any NEW issues that weren't present before
- Report to `/tmp/qa-report-task25.md`

- [ ] **Step 2: Fix bugs and commit**

---

## Task 27: QA — Stress Testing

**Dedicated QA milestone — adversarial interaction testing.**

- [ ] **Step 1: QA VERIFY — Stress scenarios**

Dispatch QA subagent:
- Rapid repeated clicks on action buttons
- Drag and immediately drop in same position
- Drag to invalid locations (outside battlefield, onto enemy back rank)
- Play card then immediately try to drag it
- End turn while drag is in progress
- Undo after repositioning
- Open void viewer / card browser during main phase
- Resize browser window — does layout break?
- Rapid end-turn clicking
- Report to `/tmp/qa-report-task26.md`

- [ ] **Step 2: Fix bugs and commit**

---

## Task 28: Update Battle Rules Documentation

**Files:**
- Modify: `docs/battle_rules/battle_rules.md`

- [ ] **Step 1: Update rules document**

Reflect all changes: two-rank battlefield, new turn structure, judgment phase, summoning sickness, character limit 16, no spark bonus, Kindle targets highest-spark, Dawn phase.

- [ ] **Step 2: Commit**

```bash
git commit -m "docs: update battle rules for combat prototype"
```

---

## Task 29: QA — Final Regression Pass

**Dedicated QA milestone — comprehensive final test.**

- [ ] **Step 1: QA VERIFY — Complete regression**

Dispatch QA subagent:
- Play a complete game start to finish (15+ turns)
- Test EVERY feature in one session:
  - Character placement in back rank
  - Drag-and-drop repositioning (front, back, swap)
  - Summoning sickness
  - Judgment resolution (win, lose, tie, uncontested)
  - Dissolved triggers during judgment
  - Card effects (dissolve, kindle, materialize)
  - Board-full prevention
  - AI behavior
  - Score accumulation and game end
  - Turn phases in correct order
  - Battle log accuracy
  - All UI elements readable and correct
- **Final invariant check:**
  - All invariants from all previous milestones still hold
- Report to `/tmp/qa-report-task28.md`

- [ ] **Step 2: Fix any remaining issues**

- [ ] **Step 3: Run `just fmt` then `just review`**

- [ ] **Step 4: Final commit**

```bash
git commit -m "chore: final cleanup for combat prototype"
```

---

## Task Summary

| # | Task | Type |
|---|------|------|
| 1 | QA Baseline | QA |
| 2 | Vanilla Characters | Code + QA |
| 3 | Battlefield Data Model | Code + QA |
| 4 | Remove Spark Bonus / Char Limit | Code + QA |
| 5 | Phase & Trigger Renames | Code + QA |
| 6 | ObjectPosition Changes | Code + QA |
| 7 | UI Rank Rendering | Code + QA |
| 8 | Extended Debug Tools | Code + QA |
| 9 | Character Placement Deep Test | QA (uses debug tools) |
| 10 | Judgment Phase Resolution | Code + QA |
| 11 | Materialization to Back Rank | Code + QA |
| 12 | Repositioning Actions | Code + QA |
| 13 | Summoning Sickness Deep Test | QA |
| 14 | AI Simplified Actions | Code + QA |
| 15 | Drag and Drop UI | Code + QA |
| 16 | Full Game Loop Round 1 | QA |
| 17 | Kindle Effect | Code + QA |
| 18 | Card Effects with Ranks | QA |
| 19 | UX Polish Round 1 | QA |
| 20 | Extended Play Round 1 (15+ turns) | QA |
| 21 | Extended Play Round 2 (Board Full) | QA |
| 22 | Extended Play Round 3 (Judgment Edge Cases) | QA (uses debug tools) |
| 23 | AI Behavior Round 2 | QA |
| 24 | UX Polish Round 2 | QA |
| 25 | Extended Play Round 4 (20+ turns) | QA |
| 26 | Extended Play Round 5 (Regression) | QA |
| 27 | Stress Testing | QA |
| 28 | Update Documentation | Code |
| 29 | Final Regression Pass | QA |

**29 tasks total.** QA passes: 20 dedicated (Tasks 1, 9, 13, 16, 18-27, 29) plus QA verification in all 13 code tasks = **33 total QA checkpoints.** Debug tools (Task 8) enable precise board state setup for QA scenarios throughout.
