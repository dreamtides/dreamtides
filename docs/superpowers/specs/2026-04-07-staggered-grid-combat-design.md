# Staggered Grid Combat Design

Redesign the Dreamtides battlefield from an 8-column aligned grid to a staggered
5-back/4-front layout inspired by autobattler games. Branch off
`positional-combat-prototype`.

## Grid Topology

Each player has 5 back-row slots and 4 front-row slots (9 total). The rows are
staggered: the back row is offset by half a slot width so each back-row slot
sits between two front-row slots.

```
        F0  F1  F2  F3        ← front (4 slots)
      B0  B1  B2  B3  B4     ← back (5 slots)
  ─────────────────────────────
      B0  B1  B2  B3  B4     ← back (5 slots)
        F0  F1  F2  F3        ← front (4 slots)
```

Maximum 9 characters per player on the battlefield.

## Support Relationships

The stagger creates "support" adjacency between back-row and front-row slots:

| Back slot | Supports front slots |
| --------- | -------------------- |
| B0        | F0                   |
| B1        | F0, F1               |
| B2        | F1, F2               |
| B3        | F2, F3               |
| B4        | F3                   |

Support has no built-in mechanical effect. It is a queryable relationship that
card effects can reference (e.g., "Supported characters get +1 spark" or "When a
character you support is dissolved, draw a card"). The data model must track
these relationships so cards can query them.

## Judgment (Combat Resolution)

Combat pairing is direct 1:1 across 4 lanes (F0 vs F0, F1 vs F1, etc.). The
rules are the same as the current system:

- Non-active player's front-row characters are attackers.
- Active player's front-row characters are blockers.
- Unblocked attacker scores victory points equal to its spark.
- Paired judgment: lower spark is dissolved. Equal spark: both dissolved.

**Key change: characters stay in the front row after judgment.** There is no
return-to-back-rank step. Surviving attackers and blockers remain in their
front-row slot. This means:

- A character that survives blocking will be an attacker on the opponent's next
  turn (assuming it isn't repositioned).
- Front-row presence is persistent. The board evolves incrementally rather than
  resetting each turn.
- The `return_to_back_rank()` logic is removed entirely.

## Positioning

During the Main phase, players can freely rearrange their characters across all
9 slots. This is not limited to one move per turn — the player drags characters
around until satisfied, then ends their turn. Dragging a character onto an
occupied slot swaps the two characters.

Constraints:

- **Summoning sickness:** Characters played this turn cannot be placed in a
  front-row slot. They must remain in the back row until the following turn.
- **Back row is the deployment bottleneck:** Characters always materialize into
  the back row. If all 5 back-row slots are occupied, no new characters can be
  played (even if total characters < 9). The player must move a back-row
  character to the front to free a slot.

## Scoring

No changes to the scoring formula. Unblocked attackers score points equal to
their spark. Victory threshold remains at 25 (tunable via playtesting).

With only 4 lanes, full blocking coverage is easier (4 characters vs. 8 in the
old system). Games will likely be more attritional — players grind through
blockers to open lanes rather than finding unblocked gaps across a wide board.
If pacing is too slow, the victory threshold is the first lever to adjust.

## AI

Use MonteCarloV1. The V2+ heuristic position assignment system is not needed for
this prototype. Update the data structures MonteCarloV1 operates on for the new
slot counts (5 back, 4 front).

## Data Structure Changes

The `Battlefield` struct changes from:

```rust
pub struct Battlefield {
    pub front: [Option<CharacterId>; 8],
    pub back: [Option<CharacterId>; 8],
}
```

to:

```rust
pub struct Battlefield {
    pub front: [Option<CharacterId>; 4],
    pub back: [Option<CharacterId>; 5],
}
```

The character limit changes from 8 to 9. Judgment iterates over positions 0–3
instead of 0–7.

## What Doesn't Change

- Turn structure (Dreamwell → Draw → Dawn → Main → Ending → Judgment)
- Stack resolution
- Energy system
- Card types (Character, Event, Dreamcaller, Dreamsign, Dreamwell)
- Keywords (Dissolve, Banish, Materialize, Prevent, Abandon, Kindle, Foresee,
  Reclaim, Fast, Discover, Copy, Gain Control)
- Triggered, activated, and static abilities
- Targeting system
- Figments
