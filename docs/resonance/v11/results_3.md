# SIM-3 Results: Hybrid A (Graduated Bias + Declining Volume)

**Simulation Agent 3 -- V11 Round 4** | Seed 42 | 1000 drafts x 30 picks x 3
strategies | Graduated Realistic

---

## Scorecard

| Metric | Value | Target | Status |
|--------|------:|--------|--------|
| M1 (early variety) | 0.55 | >= 3.0 | FAIL |
| M2 (early S/A) | 0.64 | <= 2.0 | PASS |
| M3 (picks 6+, S/A/pack) | 0.48 | >= 2.0 | FAIL |
| M4 (off-archetype) | 4.52 | >= 0.5 | PASS |
| M5 (convergence pick) | 24.1 | 5-8 | FAIL |
| M6 (deck concentration) | 38% | 60-90% | FAIL |
| M7 (run overlap) | 11% | < 40% | PASS |
| M8 (arch frequency) | 15% max | < 20% | PASS |
| M9 (S/A stddev) | 0.58 | >= 0.8 | FAIL |
| M10 (consec bad packs) | 16.6 | <= 2 | FAIL |
| M11' (picks 20+, S/A) | 0.46 | >= 2.5 | FAIL |
| M12 (signal - committed) | 0.06 | >= 0.3 | FAIL |

**Passes: 4/12. SIM-3 fails catastrophically on all concentration metrics.**

Even with forced open-lane placement (guaranteeing the committed player's
archetype is uncontested), M3 reaches only **0.89** -- still less than half the
2.0 target.

---

## Per-Archetype M3 Table

| Archetype | M3 (all) | M3 (open lane) | M5 | M6 | M10 | M11' |
|-----------|:--------:|:--------------:|:--:|:--:|:---:|:----:|
| Flash | 0.39 | 0.83 | 25.5 | 32% | 18.1 | 0.36 |
| Blink | 0.49 | 0.85 | 23.3 | 38% | 16.7 | 0.48 |
| Storm | 0.43 | 0.86 | 24.1 | 35% | 17.8 | 0.40 |
| Self-Discard | 0.51 | 0.91 | 23.6 | 40% | 16.4 | 0.49 |
| Self-Mill | 0.52 | 0.88 | 23.6 | 39% | 16.3 | 0.50 |
| Sacrifice | 0.48 | 0.95 | 23.0 | 39% | 16.0 | 0.47 |
| Warriors | 0.53 | 0.93 | 21.7 | 41% | 15.8 | 0.53 |
| Ramp | 0.45 | 0.87 | 25.5 | 36% | 17.4 | 0.41 |

M3 spread (all lanes): 0.138. Spread (forced open): 0.116. High-sibling pairs
(Warriors/Sacrifice at 50%) outperform low-sibling pairs (Flash/Ramp at 25%) by
~0.10 M3 in both modes. The spread is moderate and archetype-fair, but the
absolute values are uniformly too low.

---

## Round-by-Round Pool Composition

| Round Start | Pool | Open Total | AI Total | Open/Lane | AI/Lane | Gradient | S/A in Pool |
|:-----------:|:----:|:----------:|:--------:|:---------:|:-------:|:--------:|:-----------:|
| Round 1 | 120 | 39.0 | 65.0 | 13.0 | 13.0 | 1.00x | 17.7 |
| Round 2 | 131 | 65.9 | 41.3 | 22.0 | 8.3 | 2.66x | 15.9 |
| Round 3 | 119 | 74.6 | 39.2 | 24.9 | 7.8 | 3.17x | 15.5 |

The graduated bias and declining volume produce the intended gradient trajectory:
1.0x at start, 2.66x after the 70-card/1.4x refill, 3.17x after the 48-card/2.0x
refill. Open lanes accumulate from 13 to 25 cards per lane while AI lanes decline
from 13 to 8. The mechanism works as designed.

The problem is not the gradient -- it is the absolute density. Even at 3.17x
gradient, the player's specific archetype is one of three open lanes: ~25 cards
in a 119-card pool = 21% density. In a 5-card pack, expected on-archetype cards
= 1.05. With sibling bonus, total expected S/A ~ 1.3. This is a structural
ceiling, not a tuning gap.

---

## Pack Quality Distribution (picks 6+, committed)

| Percentile | S/A per pack |
|:----------:|:------------:|
| P10 | 0 |
| P25 | 0 |
| P50 | 0 |
| P75 | 1 |
| P90 | 1 |

Forced open lane: P10=0, P25=0, P50=1, P75=1, P90=2.

Even in the best case (player in open lane), the median pack contains 1 S/A card.
75% of packs contain 0-1 S/A. This is because the player's archetype density in
the pool (~21%) means a 5-card pack has only ~1.05 expected on-archetype draws.

---

## Consecutive Bad Pack Analysis

| Max Consec Bad | % of Drafts (all) | % (forced open) |
|:--------------:|:------------------:|:---------------:|
| 0-2 | 0.2% | ~5% |
| 3-5 | 6.4% | ~20% |
| 6-10 | 25.2% | ~35% |
| 11-15 | 22.5% | ~20% |
| 16+ | 45.7% | ~20% |

Average max consecutive bad packs: **16.6** (all lanes), **9.1** (forced open).
Target: <= 2. The system produces long droughts because the base S/A rate per
pack is below 1.0 for most of the draft.

---

## S/A Density Trajectory

```
Pick  1: 0.75                      Pick 16: 0.56
Pick  2: 0.67                      Pick 17: 0.54
Pick  3: 0.65                      Pick 18: 0.50
Pick  4: 0.60                      Pick 19: 0.46
Pick  5: 0.55                      Pick 20: 0.47
Pick  6: 0.51                      Pick 21: 0.60  <-- R3 start
Pick  7: 0.48                      Pick 22: 0.52
Pick  8: 0.43                      Pick 23: 0.52
Pick  9: 0.37                      Pick 24: 0.49
Pick 10: 0.33                      Pick 25: 0.48
Pick 11: 0.49  <-- R2 start        Pick 26: 0.42
Pick 12: 0.58                      Pick 27: 0.44
Pick 13: 0.57                      Pick 28: 0.38
Pick 14: 0.54                      Pick 29: 0.35
Pick 15: 0.55                      Pick 30: 0.33
```

Forced open lane peaks at 1.08 S/A per pack (picks 12-13 in Round 2, picks 21-22
in Round 3). The refill events produce visible bumps but each round's density
declines within the round as the player consumes S/A cards from the pool. The
trajectory never exceeds 1.1 even in the ideal case.

---

## Draft Traces

### Trace 1: Warriors, Committed (Warriors is open lane)

Round 1: 6 S/A picks from 10 (Warriors is open, AIs take Sacrifice/Ramp/Blink).
Round 2: 5 S/A picks from 10 -- picks 11-12 both hit Warriors S/A, but
picks 13, 15, 17, 19, 20 are C/F as the player draws from the larger pool where
Warriors is only 22/131 = 17%. Round 3: 6 S/A picks from 10 despite 28 S/A in
pool, because pack draws spread across all archetypes. Final: 18/30 = 60% S/A.

### Trace 2: Storm, Signal Reader (Storm is AI-covered)

Storm is covered by an AI (open lanes are Blink, Ramp, Sacrifice). The signal
reader commits to the most-available archetype at pick 5 but the SA evaluation
is relative to Storm. Round 1: 1 S/A in 10 picks. Round 2: 5 S/A. Round 3:
2 S/A. Final: 8/30 = 27%. This illustrates the penalty for committing to an
AI-covered archetype -- even signal reading cannot compensate when the SA
evaluation is tied to a contested lane.

---

## Comparison

| Algorithm | M3 | M10 | M11'/M11 | M6 |
|-----------|:--:|:---:|:--------:|:--:|
| V9 Hybrid B | 2.70 | 3.8 | 3.25 | 86% |
| V10 Hybrid X | 0.84 | -- | 0.69 | -- |
| SIM-3 committed (all lanes) | **0.48** | 16.6 | 0.46 | 38% |
| SIM-3 committed (open lane only) | **0.89** | 9.1 | 0.89 | 60% |
| SIM-3 signal-reader | **0.54** | 15.9 | 0.51 | 37% |

SIM-3 falls below V10 Hybrid X (M3 = 0.84) in the all-lanes average. Even the
forced-open-lane configuration (M3 = 0.89) slightly exceeds V10 but is 67%
below V9. The expected SIM-1 baseline (~1.4 predicted M3) and SIM-2 (~2.0)
from the design documents appear to be based on analytical estimates that did not
account for the pack-sampling bottleneck.

---

## Self-Assessment

### Root Cause of Failure

SIM-3 fails for a single structural reason: **pack density is too low relative
to pool size**. In a 120-card pool with 8 archetypes, even after aggressive
refill bias producing 3.17x gradient, the player's specific archetype occupies
~21% of the pool. A 5-card pack samples approximately 1.05 on-archetype cards.
With sibling fitness, total expected S/A reaches ~1.3. The target of M3 >= 2.0
requires fundamentally higher per-pack archetype density.

V9 achieved this through pool contraction (12% per pick), reducing the pool from
360 to ~17 cards by pick 30. By late draft, the player's archetype was 40-60% of
the surviving pool. V11's refill mechanism preserves the pool at 119-131 cards
throughout, preventing this concentration.

### What the Graduated Bias Achieves

The mechanism works correctly at the pool level. Open lanes accumulate a 3.17x
gradient by Round 3. If the pack size were 15-20 instead of 5, M3 would approach
target. Alternatively, if the pool shrank to ~40-50 cards by Round 3 (through
smaller refills or no final refill), the same gradient would yield higher per-pack
density. The graduated bias is sound; it is paired with a pool that is too large
for the pack size.

### What Would Fix It

Three structural changes could rescue SIM-3:

1. **Shrink the Round 3 pool.** Eliminating the Round 2 refill entirely (or
   reducing it to 20 cards) would leave a Round 3 pool of ~70-80 cards. The
   player's open-lane archetype would be ~30% of pool, yielding ~1.5 S/A per
   5-card pack. Still short of 2.0 but approaching viability.

2. **Increase pack size to 8-10.** A 10-card pack from a 119-card pool with 25
   on-archetype cards yields ~2.1 expected S/A. This is a UI change, not a
   mechanism change.

3. **Add within-round contraction.** A mild contraction (5-8% per pick) applied
   within each round would reduce the effective pool during late-round picks
   without conflicting with the refill mechanism. This is V9 contraction
   reintroduced at lower intensity.

### Design Document Prediction Accuracy

The design documents predicted M3 = 2.2-2.6 for Hybrid A. The simulation shows
M3 = 0.89 in the ideal case. The predictions overestimated by approximately 2.5x.
The error appears to stem from computing expected archetype density at the pool
level (25 cards in open lane = high) without modeling the pack-sampling step
(5 cards from 119 = low). Pool-level concentration does not translate to
pack-level concentration without either a small pool or a large pack.
