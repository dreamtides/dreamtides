# Pool Results 5: Lane Locking Threshold Tuning

## Executive Summary

"All threshold pairs perform similarly" was true only for late S/A in
two-threshold configs (range 1.96-2.24). **The real differentiator is lock
timing arc** -- when locks fire determines whether the draft *feels* good.
Three-threshold configs are a separate lever, boosting late S/A by +0.5-0.7.

## Configuration Comparison (Two-Threshold, W2)

| Thresholds | Dist | 1st Lock | 2nd Lock | All 4 | P1% | Late S/A | Conv | Arc |
|---|---|---|---|---|---|---|---|---|
| (2,5) | 25/55/20 | 1.1 | 3.0 | 7.4 | 93% | 1.96 | 4 | POOR |
| (3,8) | 25/55/20 | 1.9 | 4.8 | 12.0 | 29% | 2.05 | 6 | GOOD |
| (4,10) | 25/55/20 | 2.6 | 6.0 | 15.6 | 0% | 2.08 | 6 | EXCELLENT |
| (4,10) | 50/35/15 | 2.7 | 6.4 | 19.6 | 0% | 2.15 | 7 | EXCELLENT |
| (5,12) | 25/55/20 | 3.2 | 7.2 | 19.1 | 0% | 2.11 | 8 | EXCELLENT |
| (5,12) | 50/35/15 | 3.5 | 7.8 | 22.3 | 0% | 2.18 | 8 | EXCELLENT |
| (5,15) | 25/55/20 | 3.3 | 8.4 | 20.3 | 0% | 2.09 | 9 | EXCELLENT |
| (6,15) | 25/55/20 | 3.9 | 8.7 | 21.9 | 0% | 2.11 | 9 | EXCELLENT |
| (8,20) | 25/55/20 | 5.2 | 10.8 | 23.9 | 0% | 2.10 | 11 | GOOD |

## The Ideal Lock Timing Curve

Three acts in 30 picks: **Exploration (picks 1-3)** -- no locks, diverse packs.
**Commitment (picks 4-8)** -- first lock fires, second lock solidifies
archetype, S/A rises from 25% to 50%+. **Refinement (picks 10+)** -- pack
structure mostly set, player optimizes within archetype.

## Recommended: (5,12) with Primary Weight 2

| Property | (3,8) W2 | (4,10) W2 | **(5,12) W2** | (5,15) W2 |
|---|---|---|---|---|
| 1st lock | 2.0 | 2.7 | **3.4** | 3.4 |
| 2nd lock | 5.2 | 6.2 | **7.5** | 8.6 |
| All-4 locked | 14.2 | 17.6 | **20.7** | 20.9 |
| Pick-1 lock% | 24% | 0% | **0%** | 0% |
| Late S/A | 2.08 | 2.12 | **2.15** | 2.12 |
| Conv pick | 6 | 7 | **8** | 9 |

(5,12) W2: zero pick-1 locks, first lock ~pick 3-4, second lock ~pick 7-8,
all-4 locked ~pick 20 (last 10 picks retain open-slot variety). Late S/A 2.15
exceeds the 2.0 target. (4,10) W2 is the faster alternative -- nearly
identical but locks 3 picks sooner.

The current (3,8) W2 is slightly too fast: 19-29% lock on pick 1, second lock
by pick 5 -- before meaningful exploration. It works mechanically but lacks a
distinct exploration phase.

**Why W2 over W3?** W3 makes every card contribute ~4 weighted symbols, so
threshold 5 is reached in 2 picks. W3+high thresholds recreates W2 behavior
with proportionally bigger numbers. W2 keeps mental math simpler.

**Why W2 over W1?** W1 accumulates too slowly and barely distinguishes primary
from secondary, undermining the "symbol order matters" design principle.

## Third Threshold Analysis

Adding threshold 24 to (5,12) boosts late S/A from 2.15 to **2.70** (+0.55).
The third lock converts another open slot to guaranteed resonance-matched.

| Config | Late S/A | Conv | All Locked |
|---|---|---|---|
| T(5,12) W2 | 2.15 | 8 | 20.7 |
| **T(5,12,24) W2** | **2.70** | **8** | **13.5** |

Tradeoff: three thresholds lock all 4 slots by pick 13-14 instead of 20+,
eliminating late-draft open slots. Whether this is acceptable depends on
whether maintaining splash access matters more than stronger convergence.
**Recommend playtesting both (5,12) and (5,12,24).**

## Threshold-Distribution Interaction

50/35/15 (more 1-symbol cards) consistently produces +0.05-0.07 higher late S/A
than 25/55/20 across all thresholds. The effect is small vs. threshold choice
(delta 0.05-0.07 for distribution vs. 0.0-0.19 for thresholds). The two knobs
are largely orthogonal.

**Recommendation:** Keep 25/55/20. The convergence benefit of 50/35/15 is too
small to justify losing the richer cross-archetype signaling that 2-symbol
cards provide. Cards like [Tide, Zephyr] let a Warriors player build both
counts simultaneously, making the draft feel more nuanced.
