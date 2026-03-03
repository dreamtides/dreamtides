# Agent 2 Discussion Output: Auto-Widening

## Simplicity Ranking (Simplest to Most Complex)

1. **Ratcheting Slot Commitment (6)** -- three thresholds, lock a slot
2. **Lane Locking (1)** -- two thresholds, same mechanic
3. **Surge Packs (7)** -- accumulate tokens, spend 6, fill 2 slots
4. **Threshold Auto-Spend (2)** -- accumulate tokens, spend 3, add 2 bonus cards
5. **Soft Locks (3)** -- thresholds + probability + split targeting
6. **Pool Sculpting (4)** -- dual tracking + replace 12 + reserve
7. **Cascading Enhancement (5)** -- per-card rolls + dual-type pool + cap

## Scorecard (1-10)

| Goal               | LL (1) | AS (2) | SL (3) | PS (4) | CE (5) | RS (6) | SP (7) |
| ------------------ | :----: | :----: | :----: | :----: | :----: | :----: | :----: |
| 1. Simple          |   9    |   7    |   5    |   4    |   3    |   9    |   7    |
| 2. No actions      |   10   |   10   |   10   |   10   |   10   |   10   |   10   |
| 3. Not on rails    |   3    |   6    |   7    |   7    |   6    |   3    |   7    |
| 4. No forced decks |   6    |   7    |   7    |   7    |   7    |   6    |   7    |
| 5. Flexible        |   3    |   5    |   6    |   7    |   5    |   3    |   6    |
| 6. Convergent      |   7    |   6    |   5    |   3    |   5    |   8    |   4    |
| 7. Splashable      |   4    |   7    |   7    |   7    |   5    |   4    |   7    |
| 8. Open early      |   5    |   7    |   7    |   9    |   7    |   5    |   8    |
| 9. Signal reading  |   4    |   5    |   5    |   4    |   6    |   4    |   5    |

## Key Discussion Points

**Best-of-breed:** My unchampioned Proposal 4 (Momentum Auto-Spend) deserves
reconsideration. The streak mechanic -- consecutive same-resonance triggers add
escalating bonus cards -- directly addresses the "barely crossing 2.0" problem.
Agent 7's Surge Packs is structurally identical to my system but uses slot
replacement instead of card addition. Addition is strictly better because it
does not displace random cards.

**Simplicity audit:** Agent 6's Ratcheting passes cleanly but is functionally
Lane Locking with an extra threshold. My champion is implementable from its
one-sentence but "adds 2 bonus cards" means variable pack size (4 or 6), a
hidden UX concern. Agent 4's Pool Sculpting hides a complexity bomb -- the
reserve is not specified in the one-sentence.

**S/A precision debate:** Agent 1's analysis is critical. If the Tide-primary
pool contains only Warriors-home (S-tier for Warriors) and Sacrifice-home cards
(A-tier for Warriors, since adjacent sharing primary), then locked/bonus
resonance cards are ~75-100% S/A, not ~50%. At 100% precision, my cost 3/bonus 2
auto-spend would deliver 2 S/A bonus cards per trigger -- too high at ~3.0 total
S/A. Something doesn't add up; the real figure is likely 60-80% once individual
card fitness varies. But even at 60%, auto-spend bonus cards are more effective
than my Round 1 math assumed.

## Final Champion

**Threshold Auto-Spend** (cost 3, bonus 2) -- unchanged.

## Planned Modifications

- Add minimum activation threshold: no auto-spend until 2+ non-generic cards
  drafted.
- Cap bonus cards at 2 per pack (explicit).
- Consider switching to Momentum Auto-Spend if simulation shows base version
  barely crosses 2.0.

## Proposed Symbol Distribution

Same as Round 1: 36 generic, 65 mono-1, 146 mono-2, 32 dual-2, 59 mono-3, 22
dual-3. Total dual: 54 (15%). Heavy 2-symbol cards sustain ~3 tokens/pick.
