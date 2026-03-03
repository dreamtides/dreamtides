# Results 4: Dual-Threshold Pair Guarantee

## One-Sentence Algorithm

"Track the ordered symbol pair (first, second) of each 2+ symbol card you
draft; at 3 matching picks one pack slot is pair-matched, at 7 matching picks a
second slot is pair-matched, and remaining slots are random."

## Target Scorecard

All metrics at ARCHETYPE level. 999 drafts, 30 picks, 3 strategies.

| Metric | Target | Actual | Pass/Fail |
|--------|--------|--------|-----------|
| Picks 1-5: unique archs with S/A per pack | >= 3 | 6.44 | PASS |
| Picks 1-5: S/A for emerging arch per pack | <= 2 | 1.35 | PASS |
| Picks 6+: S/A for committed arch per pack | >= 2 | 2.22 | PASS |
| Picks 6+: off-arch (C/F) per pack | >= 0.5 | 1.51 | PASS |
| Convergence pick | 5-8 | 11.9 | FAIL |
| Deck concentration | 60-90% | 79.6% | PASS |
| Run-to-run card overlap | < 40% | 8.4% | PASS |
| Archetype freq (max/min) | <20/>5% | 17.1/10.3% | PASS |
| StdDev S/A per pack (picks 6+) | >= 0.8 | 0.95 | PASS |

**8/9 pass. Only convergence pick fails (11.9 vs 5-8).**

## Variance Report

StdDev: **0.95**. Distribution (picks 6+): 0=4.5%, 1=15.7%, 2=40.8%, 3=31.7%,
4=7.4%. Bell curve centered on 2-3. Better than Lane Locking (0.74, fails).

## Per-Archetype Convergence Table

| Archetype | Avg Convergence Pick |
|-----------|---------------------|
| Flash/Tempo/Prison | 10.7 |
| Blink/Flicker | 9.9 |
| Storm/Spellslinger | 9.8 |
| Self-Discard | 11.1 |
| Self-Mill/Reanimator | 10.5 |
| Sacrifice/Abandon | 10.4 |
| Warriors/Midrange | 10.0 |
| Ramp/Spirit Animals | 10.7 |
| **Average** | **10.4** |

All 8 within 1.3-pick window -- excellent balance. Slow because only 2+ symbol
picks contribute pairs. **Pair precision: 100.0%**.

## V3/V4 Comparison

| Metric | DT (3/7) | Lane Lock | Pack Widen | Target |
|--------|----------|-----------|------------|--------|
| Late S/A per pack | **2.22** | **2.61** | 1.35 | >=2 |
| Off-arch per pack | **1.51** | 1.19 | 2.25 | >=0.5 |
| Convergence pick | 11.9 | **6.8** | 18.5 | 5-8 |
| Deck concentration | 79.6% | 86.4% | 77.5% | 60-90% |
| StdDev S/A | **0.95** | 0.74 | 0.94 | >=0.8 |
| **Targets passed** | **8/9** | **8/9** | **5/9** | |

LL converges faster (6.8) but fails variance (0.74). Pack Widening fails at
1.35 S/A, confirming single-resonance dilution. DT's 100% pair precision
eliminates this. DT has better splash (1.51 vs 1.19). LL's higher S/A (2.61)
comes from faster accumulation -- all cards contribute symbols, DT only
counts 2+ symbol picks.

## Symbol Distribution and Sensitivity

Primary: 15/65/20 (1/2/3-sym). 85% of non-generics have 2+ symbols.

| Distribution | Late S/A | Conv Pick | StdDev |
|--------------|----------|-----------|--------|
| 15% 1-sym | 2.22 | 12.0 | 0.94 |
| 30% 1-sym | 2.12 | 13.0 | 0.98 |

Robust: +15% 1-symbol costs only 0.10 S/A and 1 convergence pick.

## Parameter Sensitivity

| Thresholds | Late S/A | Conv Pick | StdDev | Off-Arch |
|-----------|----------|-----------|--------|----------|
| (2, 5) | 2.48 | 9.2 | 0.82 | 1.29 |
| (3, 7) | 2.22 | 11.8 | 0.93 | 1.51 |
| (4, 9) | 1.98 | 14.1 | 1.00 | 1.72 |

(2/5) converges faster with higher S/A but less variance. (4/9) fails S/A.
(3/7) balances all metrics; (2/5) viable if speed prioritized.

## Draft Traces

**Trace 1 -- Early Committer (Warriors):** Picks split between Sacrifice
[Tide/Stone] and Warriors [Tide/Zephyr], delaying accumulation. First slot
pick 13, second pick 20. Final: 26/30 S/A (87%). Pair splitting between
adjacent archetypes is the main convergence bottleneck.

**Trace 2 -- Signal Reader (Storm):** Ember/Zephyr accumulates faster than
Ember/Stone. Guaranteed slots deliver Blink (A-tier for Storm). Player drifts
to Blink naturally. Final: 29/30 S/A (97%). Algorithm follows actual trajectory.

**Trace 3 -- Power Chaser:** Picks Self-Mill [Stone/Tide] early by power.
Thresholds at picks 4 and 10. Final: 22/30 S/A (73%).

## Self-Assessment

| Goal | Score | Justification |
|------|-------|---------------|
| 1. Simple | 9 | Binary thresholds, concrete counts, one-sentence specifies all |
| 2. No actions | 10 | Fully automatic |
| 3. Not on rails | 4 | 50% deterministic pack after thresholds; feels guided |
| 4. No forced decks | 6 | Random slots and pool randomness prevent forcing |
| 5. Flexible | 5 | Dynamic leading pair enables pivots but counts resist |
| 6. Convergent | 7 | 2.22 S/A passes; conv pick (11.9) slow |
| 7. Splashable | 7 | 1.51 off-arch; random slots maintain variety |
| 8. Open early | 8 | First 3+ picks fully random |
| 9. Signal reading | 3 | No pool interaction; signals irrelevant |
