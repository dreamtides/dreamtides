# Dreamtides Combat Prototype Design

## Overview

Revise Dreamtides core battle rules to add a positional combat system with
front/back ranks. Characters on the front rank fight opposing characters in the
same column during a new Judgment phase at end of turn. Characters on the back
rank are safe but do nothing during Judgment. This replaces the old spark
comparison scoring system.

## Design Decisions

- Active player only: scoring and dissolving during Judgment happen only for the
  active player's characters
- Back rank characters do nothing during Judgment (no scoring, no fighting)
- Positional indexing: same visual column (your position 3 fights opponent's
  position 3 as rendered on screen)
- Character limit: 16 total (8 per rank), can't play characters when full (no
  abandon/overflow mechanic)
- Spark bonus: removed entirely, spark only comes from characters on the field
- Summoning sickness: characters enter the back rank and can't move to the front
  rank on the turn they are played; eligible to move on the player's next turn
- Characters can only be repositioned during the owning player's main phase
- Kindle: changed from "leftmost character" to "highest-spark character" with
  tiebreaker of oldest character (first played)
- Dawn phase: renamed from Judgment, purely a trigger window (no scoring), like
  MTG's upkeep
- Dissolved triggers fire individually after each position's comparison resolves
  (not batched)
- No automated tests; all validation through manual QA via battle prototype
- Changes are to rules_engine/ (Rust) and scripts/battle_prototype/ (TypeScript)
  only; no C# client changes

## Data Model

### Battlefield Storage

Replace the current `PlayerMap<CardSet<CharacterId>>` battlefield in `AllCards`
with a structured container:

```rust
pub struct Battlefield {
    pub front: [Option<CharacterId>; 8],
    pub back: [Option<CharacterId>; 8],
}
```

Each player gets one `Battlefield`. `AllCards` changes from
`battlefield: PlayerMap<CardSet<CharacterId>>` to
`battlefield: PlayerMap<Battlefield>`.

This is optimized for MCTS performance: fixed-size arrays are cheap to clone
(memcpy), O(1) positional lookup, and Judgment resolution is a tight loop over
8 pairs.

### CharacterState Changes

Add `played_turn: u32` to `CharacterState` for summoning sickness. A character
cannot move to the front rank if `played_turn == current_turn`.

### Removals

- `spark_bonus` field on `BattlePlayerState` — eliminated entirely
- Character-limit abandon logic — replaced by "can't play if full"
- The `spark_total()` query simplifies to sum of character spark values only

### Zone Enum

Unchanged. `Zone::Battlefield` still means "on the battlefield." Rank and
position are determined by looking up the character in the `Battlefield` struct.

## Turn Structure

### New Phase Order

Starting → Dreamwell → Draw → Dawn → Main → Judgment → Ending →
EndingPhaseFinished → FiringEndOfTurnTriggers

Changes from current order:
- Dreamwell moves to first (after Starting)
- Draw stays after Dreamwell
- Dawn (renamed from Judgment) fires start-of-turn triggers, no scoring
- Main unchanged, but now includes repositioning actions
- Judgment is new, resolves front-rank combat at end of turn

### BattleTurnPhase Enum Changes

- Rename existing `Judgment` variant → `Dawn`
- Add new `Judgment` variant after `Main` (before `Ending`)
- Reorder the state machine transitions in `turn.rs`

### First Turn

Draw is still skipped on turn 1. No other phase changes.

## Judgment Phase Resolution

Runs automatically at the end of the active player's turn. No player actions,
no priority, no card playing.

### Algorithm

For each position i from 0 to 7:
1. Let attacker = active player's front rank at position i
2. Let defender = opponent's front rank at position i
3. If both present: compare spark values
   - Attacker spark > defender spark → dissolve defender
   - Defender spark > attacker spark → dissolve attacker
   - Tied → dissolve both
4. If only attacker present (no defender): active player gains victory points
   equal to attacker's spark
5. If only defender present: nothing happens
6. If both empty: nothing happens
7. After each position resolves (including any dissolution), fire Dissolved
   triggers and let them fully resolve before moving to position i+1

Dissolution uses the existing dissolve pathway so all card interactions are
preserved. A Dissolved trigger at position 0 can modify a character at position
5 before that comparison happens.

Check for victory condition after all 8 positions resolve.

## Repositioning Actions

### Human Player (Battle Prototype)

During main phase, drag-and-drop characters between slots:
- Drag to empty slot → move character there
- Drag onto another character → swap positions
- Drag to front rank while summoning sick → rejected with visual feedback
- Drag disabled outside main phase

### AI Actions (Simplified for MCTS)

Three action types to avoid infinite branching:
- **MoveToEmptyFrontSlot(CharacterId)** — move to any unoccupied front-rank
  position (all empty slots are equivalent for scoring)
- **MoveToAttack(CharacterId, CharacterId)** — move attacker to the position
  occupied by defender (face a specific enemy)
- **MoveToBack(CharacterId)** — retreat to back rank; only allowed if this
  character has not already been moved this turn (prevents infinite loops)

The "already moved this turn" flag is per-turn transient state, cleared at start
of next turn.

### BattleAction Variants

New variants added to the BattleAction enum:
- `MoveCharacterToFrontRank(CharacterId, u8)` — move to front rank position
  (used by human drag-and-drop mapped to specific positions)
- `MoveCharacterToBackRank(CharacterId, u8)` — move to back rank position

AI legal action generator wraps these into the simplified actions above.

## Battle Prototype UI

### Layout (Top to Bottom)

1. Enemy player status bar (score, energy, deck, hand counts)
2. Enemy back rank (8 slots)
3. Enemy front rank (8 slots)
4. Judgment line separator
5. Your front rank (8 slots)
6. Your back rank (8 slots)
7. Your player status bar
8. Your hand
9. Action bar (End Turn, Undo, etc.)
10. Battle log

### Rank Rendering

Each rank shows 8 fixed slots. Empty slots are dashed outlines. Occupied slots
show compact card display (name + spark). Characters with summoning sickness get
a visual indicator.

### Position Data from Rust

`ObjectPosition` changes: `OnBattlefield` goes from
`{ player: DisplayPlayer }` to
`{ player: DisplayPlayer, rank: "Front" | "Back", position: u8 }`.

## Keyword & Effect Changes

### Kindle

Changes from "add spark to leftmost character" to "add spark to highest-spark
character." Tiebreaker: oldest character (first played). Affects the parser,
effect application, and target query.

### Trigger Renames

- `Trigger::Judgment` → `Trigger::Dawn` (existing cards trigger at Dawn)
- New `Trigger::Judgment` fires during the new Judgment phase

### Targeting

No predicate changes needed. Cards targeting "a character on the battlefield"
still work since the zone is unchanged. Rank-specific predicates are out of
scope.

### Dissolve/Banish/Abandon

Work unchanged. Dissolving a character vacates its slot
(`Option<CharacterId>` becomes `None`).

## Vanilla Test Characters

Add 6 vanilla characters (no abilities, just spark/cost) to cards.toml and
register them in the Core 11 card list. These provide clean combat testing
without ability noise:

| Name              | Cost | Spark | Copies |
|-------------------|------|-------|--------|
| Duskborne Sentry  | 1    | 1     | 8      |
| Glimmer Scout     | 2    | 2     | 8      |
| Veilward Knight   | 3    | 3     | 6      |
| Embertide Warrior | 4    | 5     | 6      |
| Starforged Titan  | 5    | 7     | 4      |
| Abyssal Colossus  | 7    | 10    | 4      |

## QA Strategy

No integration tests or unit tests. All validation through manual QA using
agent-browser CLI against the battle prototype.

### QA Milestones (at least 15 dedicated QA passes)

1. Rank rendering — 4 ranks display, labels, empty slots
2. Character placement — characters appear in back rank on play
3. Drag-and-drop basics — reposition within ranks, between ranks
4. Summoning sickness — can't drag to front on turn played
5. Judgment resolution visuals — combats resolve, dissolves happen, points scored
6. Full game loop round 1 — play 10+ turns, check turn structure, phase ordering
7. AI behavior round 1 — AI repositions, makes attacks, no loops
8. Card effects with ranks — Kindle, dissolve, materialize interactions
9. UX polish round 1 — layout readability, labels, log clarity, missing info
10. Extended play round 1 — 15+ turns, look for mid-game bugs
11. Extended play round 2 — focus on board-full states, 16 characters
12. Extended play round 3 — focus on judgment edge cases (ties, empty, zero spark)
13. AI behavior round 2 — watch AI tactical choices over many games
14. UX polish round 2 — fix issues from extended play, improve judgment readability
15. Extended play round 4 — final 20+ turn stress test, late-game states

Each QA pass produces a bug report. Bugs are fixed, then next QA pass runs.
