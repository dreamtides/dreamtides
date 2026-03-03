# Agent 2 Comparison: Threshold Auto-Spend

## Scorecard (1-10, all 7 algorithms x 9 design goals)

| Goal               | LL (1) | TAS (2) | SL (3) | PS (4) | DE (5) | RS (6) | SP (7) |
| ------------------ | :----: | :-----: | :----: | :----: | :----: | :----: | :----: |
| 1. Simple          |   9    |    7    |   5    |   4    |   8    |   8    |   9    |
| 2. No actions      |   10   |   10    |   10   |   10   |   10   |   10   |   10   |
| 3. Not on rails    |   2    |    7    |   6    |   8    |   7    |   3    |   8    |
| 4. No forced decks |   7    |    8    |   7    |   8    |   7    |   6    |   7    |
| 5. Flexible        |   2    |    6    |   5    |   7    |   5    |   3    |   7    |
| 6. Convergent      |   7    |    7    |   8    |   4    |   4    |   8    |   6    |
| 7. Splashable      |   4    |    8    |   5    |   3    |   7    |   7    |   8    |
| 8. Open early      |   7    |    8    |   7    |   8    |   7    |   7    |   8    |
| 9. Signal reading  |   3    |    6    |   5    |   5    |   5    |   3    |   5    |
| **Total**          | **51** | **67**  | **58** | **57** | **60** | **55** | **68** |

**Key scoring disagreements with Agent 1:**

- I score LL lower on flexibility (2) and rails (2) than Agent 1 does. Permanent
  locks at pick 3-4 are the worst railroading of any algorithm. Agent 1 is too
  generous to their own system.
- I score Surge Packs higher on simplicity (9) than most agents. "Accumulate
  tokens, spend at threshold, fill slots" is as clean as slot-locking.
- SL convergent 8: Agent 3's 2.20 S/A at 0.88 stddev is the best
  convergence-with-variance combination. The 75% probability is a feature, not a
  bug.

## Biggest Strength and Weakness Per Strategy

| Algo    | Biggest Strength                                                         | Biggest Weakness                                                              |
| ------- | ------------------------------------------------------------------------ | ----------------------------------------------------------------------------- |
| LL (1)  | Guaranteed floor: every post-lock pack has exactly 2 S/A                 | Zero variance -- 78.5% of packs have exactly 2 S/A cards                      |
| TAS (2) | Only 9/9 pass; balanced across all metrics with no catastrophic failures | 2.01 S/A has no safety margin; statistically indistinguishable from 2.0       |
| SL (3)  | Best variance-convergence combination (2.20 S/A, 0.88 stddev)            | 97% deck concentration for committed players                                  |
| PS (4)  | Invisible to player; most organic feel of all algorithms                 | Splash failure (0.36) is structural and unfixable without ruining convergence |
| DE (5)  | Pack-composition trigger is the most original mechanic in V6             | Champion (thresh=2) fails at 1.32 S/A; the concept does not scale             |
| RS (6)  | Highest raw S/A among algorithms with genuine convergence (2.20)         | Variance 0.69 fails target; mechanically identical to Lane Locking            |
| SP (7)  | Non-permanent tracking + rhythmic surge creates best player experience   | 25% of post-commitment packs have 0 S/A -- painful valleys                    |

## Proposed Improvements

- **LL (1):** Raise thresholds to 5/12 to delay convergence into the 5-8 window.
  Will not fix variance.
- **TAS (2) -- self-improvement:** Switch to C3B2 with a "no spend before pick
  3" gate. This pushes S/A to 2.6 while preserving early openness. Concentration
  rises to 91%, a known tradeoff.
- **SL (3):** The 75% probability is the right design choice. Problem is
  concentration, not convergence. Add a "skip enhancement if last 3 packs were
  enhanced" cooldown.
- **DE (5):** Abandon thresh=2. Thresh=1 with bonus=2 is the real algorithm
  (2.13 S/A). Accept the 63% fire rate.
- **RS (6):** Hybrid: use Ratcheting's 3-lock structure but make the first lock
  a 75% soft lock, second and third hard. Provides early variance with late
  certainty.
- **SP (7):** The T=4/S=3 variant is the right pick. Consider S=2 with a lower
  threshold (T=3) for less extreme bimodality.

## Baseline Comparison

**Does any V6 algorithm clearly beat both baselines?**

My champion (TAS C4B2) beats both on metric breadth: 9/9 vs LL's 6/9 and PW's
7/9 (V4). But raw S/A power ranks last among algorithms that cross 2.0. The
honest answer: TAS is the most well-rounded zero-decision algorithm but not the
most powerful.

Ratcheting Split (RS) and Soft Locks (SL) both beat LL on splash and match it on
S/A. Neither approaches Pack Widening's 3.35 S/A. The zero-decision constraint
creates a hard ceiling around 2.2 S/A for algorithms that also maintain
variance.

## Proposed Best Algorithm

**Threshold Auto-Spend at C3B2 with pick-3 gate.**

"Each drafted symbol earns matching tokens (+2 primary, +1 others); starting
from pick 3, when any counter reaches 3, auto-spend 3 tokens from the highest
counter and add 2 resonance-matched bonus cards to the pack."

This is my own algorithm tuned for power. The pick-3 gate preserves early
openness. C3B2 fires on 95% of post-commitment picks, delivering ~2.6 S/A. The
tradeoff is 91% deck concentration, which I believe is acceptable for a
roguelike where finding your archetype IS the challenge.

I prefer additive injection (bonus cards) over slot-locking because it preserves
pack size variance and never permanently constrains the player. The higher S/A
also creates real headroom above 2.0.

## 15% Dual-Resonance Constraint Impact

The constraint made the problem **just different, not harder**. The critical
revelation from simulation is that primary-resonance pools deliver 100% S/A to
committed players because adjacent archetypes are mutually S/A. This means
single-resonance targeting is sufficient -- the 50% dilution fear from V3/V4 was
wrong at the S/A tier level.

At 10%: No change to algorithm performance. Dual-type cards are decorative for
algorithms. At 20%: Pair-matching supplements could add ~0.15 S/A. Worth
exploring but not transformative.

The real lesson: the constraint forced us to build algorithms that work with
mono-resonance signals, and it turns out mono-resonance signals are better than
we thought. The problem was always in our evaluation model, not the card pool.
