# Agent 3 Discussion Output: Soft Slot Targeting

## Simplicity Ranking (Simplest to Most Complex)

1. **Lane Locking (1)** -- two thresholds, binary lock, zero ambiguity
2. **Ratcheting Slot Commitment (6)** -- three thresholds, same binary lock
3. **Surge Packs (7)** -- token counter, single trigger, slot filling
4. **Threshold Auto-Spend (2)** -- token counter, auto-spend, bonus addition
5. **Soft Locks (3)** -- thresholds + probability + split-resonance
6. **Pool Sculpting (4)** -- dual tracking, replacement, reserve
7. **Cascading Enhancement (5)** -- per-card rolls, dual-type pool, cap

## Scorecard (1-10)

| Goal | LL (1) | AS (2) | SL (3) | PS (4) | CE (5) | RS (6) | SP (7) |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | 9 | 7 | 5 | 4 | 3 | 8 | 7 |
| 2. No actions | 10 | 10 | 10 | 10 | 10 | 10 | 10 |
| 3. Not on rails | 3 | 6 | 7 | 7 | 7 | 3 | 7 |
| 4. No forced decks | 6 | 7 | 7 | 7 | 7 | 5 | 7 |
| 5. Flexible | 3 | 5 | 7 | 7 | 5 | 3 | 6 |
| 6. Convergent | 7 | 6 | 5 | 3 | 5 | 8 | 4 |
| 7. Splashable | 4 | 6 | 8 | 7 | 5 | 4 | 7 |
| 8. Open early | 6 | 7 | 7 | 9 | 7 | 5 | 8 |
| 9. Signal reading | 4 | 5 | 6 | 4 | 6 | 4 | 5 |

## Key Discussion Points

**Self-critique:** My champion is the most complex of all seven. Three thresholds, probability per slot, AND dual-resonance targeting on the third lock -- three ideas stacked, not one mechanism. I score myself 5/10 on simplicity.

**Best-of-breed:** My unchampioned Proposal 4 (Split-Resonance Slot Pairs) was the more original contribution. Agents 1, 3, and 6 are all converging on threshold-triggered slot locking variants. The split-resonance insight is the only genuinely differentiating idea in my domain.

**The S/A precision question undermines soft targeting.** If locked resonance slots already deliver 75-100% S/A AND draw from pools of ~80 cards for natural variance, then the 75% probability in soft locks trades convergence power for marginal variance gains. At 75% precision from hard locks: 3 hard locks = 2.25 S/A; 3 soft locks at 75% probability = 0.75 * 2.25 = 1.69 S/A. The softening costs ~0.56 S/A -- a huge penalty.

**Convergence with hard locks + split resonance:** If I drop the probability and go binary:
- 2 locks at primary (~75% S/A each) = 1.5 S/A
- 1 lock at secondary (~50-75% S/A) = 0.5-0.75 S/A
- 1 random slot = 0.25 S/A
- Total: 2.25-2.5 S/A -- comfortably above 2.0.

## Final Champion

**Threshold-Triggered Soft Locks** -- retained but radically simplified.

## Planned Modifications

1. **Drop probability to binary (100% locks).** Probabilistic softening is complexity for insufficient gain.
2. **Keep the split-resonance third lock.** The unique differentiator over Ratcheting.
3. **Revised one-sentence:** "When your top resonance crosses 3 and 6, lock one slot each to that resonance; when it crosses 9, lock a third slot to your second-highest resonance; slot 4 stays random."

This is now a Lane Locking variant with one twist: the third lock targets the second resonance for archetype disambiguation.

## Proposed Symbol Distribution

54 dual-type (15%), 65 mono-1, 140 mono-2, 65 mono-3, 36 generic. Maximize dual-type to support second-resonance identification.
