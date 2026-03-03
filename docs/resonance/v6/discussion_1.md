# Agent 1 Discussion Output: Lane Locking (Reference Baseline)

## Simplicity Ranking (Simplest to Most Complex)

1. **Surge Packs (7)** -- one counter, one trigger
2. **Ratcheting Slot Commitment (6)** -- three thresholds, proven lock mechanic
3. **Lane Locking (1)** -- two thresholds, binary locks
4. **Threshold Auto-Spend (2)** -- tokens + auto-spend + bonus cards
5. **Soft Locks (3)** -- thresholds + probability + split-resonance
6. **Pool Sculpting (4)** -- dual tracking + replacements + reserve
7. **Cascading Enhancement (5)** -- per-card rolls + dual-type pool + cap

## Scorecard (1-10)

| Goal               | LL (1) | AS (2) | SL (3) | PS (4) | CE (5) | RS (6) | SP (7) |
| ------------------ | :----: | :----: | :----: | :----: | :----: | :----: | :----: |
| 1. Simple          |   9    |   7    |   5    |   4    |   3    |   8    |   8    |
| 2. No actions      |   10   |   10   |   10   |   10   |   10   |   10   |   10   |
| 3. Not on rails    |   3    |   6    |   7    |   7    |   7    |   4    |   7    |
| 4. No forced decks |   7    |   7    |   7    |   6    |   7    |   6    |   7    |
| 5. Flexible        |   3    |   5    |   6    |   7    |   5    |   3    |   6    |
| 6. Convergent      |   7    |   6    |   5    |   3    |   6    |   8    |   4    |
| 7. Splashable      |   4    |   6    |   7    |   7    |   6    |   4    |   7    |
| 8. Open early      |   5    |   7    |   7    |   9    |   7    |   5    |   8    |
| 9. Signal reading  |   4    |   5    |   5    |   4    |   5    |   4    |   5    |

## Key Discussion Points

**Best-of-breed:** Agent 3's unchampioned Proposal 4 (Split-Resonance Slot
Pairs) contains the strongest single insight: targeting BOTH top resonances
narrows archetype ambiguity from 4 to 1-2. Agent 7's Proposal 5 (Draft Imprint)
offers non-permanent slot tracking that solves Lane Locking's biggest weakness.

**Simplicity audit:** Agent 5's Cascading Enhancement fails the one-sentence
test -- three independent mechanisms plus hidden rules. Agent 6's Ratcheting is
functionally Lane Locking with a third threshold.

**Critical S/A precision question:** This discussion's most important open
issue. When a slot locks to Tide, the Tide-primary pool contains ONLY
Warriors-home and Sacrifice-home cards. The fitness model says Sacrifice cards
are A-tier for Warriors (adjacent sharing primary). If true, locked slots
deliver ~75-100% S/A, not ~50%. This would mean all agents are underestimating
slot-locking approaches. V3's 2.72 S/A with 2 locks is consistent with ~75%
precision per locked slot. Simulation must resolve this.

## Final Champion

**Lane Locking** (unchanged -- reference baseline).

## Planned Modifications

None. Lane Locking is the reference baseline and must be implemented as
specified for apples-to-apples comparison.

## Proposed Symbol Distribution

Same as Round 1: 36 generic, 80 mono-1, 136 mono-2, 32 dual-2, 48 mono-3, 24
dual-3, 4 trimmed for the 54-cap. Total dual: 54 (15%).
