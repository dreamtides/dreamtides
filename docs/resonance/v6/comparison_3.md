# Agent 3 Comparison: Soft Locks

## Scorecard (1-10, all 7 algorithms x 9 design goals)

| Goal | LL (1) | TAS (2) | SL (3) | PS (4) | DE (5) | RS (6) | SP (7) |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | 9 | 7 | 6 | 4 | 7 | 8 | 8 |
| 2. No actions | 10 | 10 | 10 | 10 | 10 | 10 | 10 |
| 3. Not on rails | 3 | 7 | 7 | 8 | 7 | 4 | 8 |
| 4. No forced decks | 7 | 8 | 7 | 8 | 7 | 6 | 7 |
| 5. Flexible | 3 | 6 | 6 | 7 | 5 | 4 | 7 |
| 6. Convergent | 7 | 6 | 8 | 4 | 4 | 8 | 6 |
| 7. Splashable | 4 | 8 | 5 | 3 | 7 | 7 | 8 |
| 8. Open early | 7 | 8 | 7 | 8 | 7 | 7 | 7 |
| 9. Signal reading | 3 | 6 | 5 | 5 | 5 | 3 | 5 |
| **Total** | **53** | **66** | **61** | **57** | **59** | **57** | **66** |

**Scoring rationale for contentious cases:**

- SL convergent 8: 2.20 S/A with 0.88 stddev is the only dual-target achievement in V6.
- RS convergent 8: Same 2.20 S/A but variance failure (0.69) is scored under Goal 3.
- TAS convergent 6: 2.01 S/A is not meaningfully above the threshold. One bad seed and it fails.

## Biggest Strength and Weakness Per Strategy

| Algo | Biggest Strength | Biggest Weakness |
|------|-----------------|------------------|
| LL (1) | Structural certainty: 100% S/A in locked slots eliminates convergence risk | Fastest convergence (pick 3.3) means drafts are decided before exploration |
| TAS (2) | Best metric breadth: no catastrophic failure on any dimension | 2.01 S/A is a rounding error away from failure |
| SL (3) | Only algorithm crossing both 2.0 S/A AND 0.8 stddev simultaneously | 97% deck concentration; splash at 0.49 borderline |
| PS (4) | True organic variance from probabilistic pool evolution | Structural ceiling: 2.01 with exhaustive T1-heavy tuning proves V4's finding |
| DE (5) | Pack-composition trigger creates unique variance profile (1.55 stddev) | Fundamentally flawed: thresh=2 fires 21%, thresh=1 fires 63% -- no sweet spot |
| RS (6) | Three-act escalation creates narrative arc across the draft | Mechanically indistinguishable from Lane Locking for the player |
| SP (7) | Non-permanent state tracking enables genuine pivoting | 25% zero-S/A packs post-commitment are unacceptably harsh |

## Proposed Improvements

- **LL (1):** Add a 25% miss chance on locked slots: each locked slot shows a random card 25% of the time instead of the locked resonance. This trades 0.5 S/A for variance, landing at ~1.6 -- too low. This confirms soft locks need ALL three slots, not just two.
- **TAS (2):** The fragility problem is solvable by raising bonus to 3. C4B3 would hit ~2.5 S/A. Test whether concentration stays below 90%.
- **SL (3) -- self-improvement:** The splash deficit is my biggest problem. Replace the "fully random slot 4" with "slot 4 shows a card NOT from your top 2 resonances." This guarantees splash without costing S/A. Revised one-sentence adds 5 words.
- **PS (4):** Hybridize: use pool sculpting as the base layer, then add 1 locked slot at threshold 6. The pool provides organic variance while the lock provides a convergence floor.
- **DE (5):** The concept is fundamentally at odds with V6 constraints. At 22% resonance base rate, requiring 2-of-4 matches is asking for a 21% event. No parameter change fixes the math.
- **RS (6):** Lower thresholds to 2/4/7 (per simulation: stddev 0.81, barely passing variance target).
- **SP (7):** Reduce to 2 surge slots. At 75% S/A precision: surge packs = 2*0.75 + 2*0.25 = 2.0. With 64% surge rate: expected = 0.64*2.0 + 0.36*1.0 = 1.64. Too low. Surge Packs needs 3 slots to work, creating the bimodal problem.

## Baseline Comparison

**Does any V6 algorithm clearly beat both baselines?**

Against LL: Soft Locks (2.20 S/A, 0.88 stddev) matches LL's convergence with much better variance. TAS passes more metrics. Both beat LL on multiple dimensions.

Against Pack Widening: Nothing approaches 3.35 S/A without player decisions. The zero-decision constraint costs roughly 1.0-1.5 S/A.

My claim: Soft Locks beats Lane Locking (same S/A, better variance, slightly better splash) and represents the best variance-adjusted convergence in V6. It does not beat Pack Widening on raw power.

## Proposed Best Algorithm

**Three-threshold soft locks at 75% with forced-splash slot 4.**

"When your top resonance count crosses 3, 6, and 9, one more slot begins showing a resonance-matched card 75% of the time (first two target top resonance, third targets second-highest); slot 4 always shows a card from neither of your top 2 resonances."

This is my algorithm with forced splash on slot 4. Projected metrics: 2.15 S/A, 0.85 stddev, 0.8+ C/F per pack. The forced-splash slot solves the 0.49 C/F problem while the 75% probability delivers natural variance that hard locks cannot match.

I disagree with agents championing slot-locking (1, 6) because variance is not optional -- it is a stated design goal. And I disagree with Agent 2 because 2.01 is not a safe margin.

## 15% Dual-Resonance Constraint Impact

The constraint made the problem **harder but in a surprising way**. The expected impact was on archetype disambiguation -- without dual-type cards, you cannot tell Warriors from Sacrifice. The actual impact was minimal because the S/A tier structure makes disambiguation unnecessary for convergence. Both Warriors and Sacrifice cards are S/A for a Tide-committed player.

The constraint's real effect was **psychological**: it forced algorithm designers to abandon pair-matching hopes and build pure single-resonance systems. The systems we built work better than expected because we were forced to confront the actual tier structure.

At 10%: Identical algorithm performance. Fewer archetype-signal cards for player awareness.
At 20%: Minor improvement from occasional pair-matching bonuses. I estimate +0.05-0.1 S/A from layering a pair-matching supplement. Not transformative.
