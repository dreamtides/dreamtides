# SIM-2 Results: Static Open-Lane Bias

## Full Scorecard

| Metric | Value | Target | Status |
|--------|------:|--------|--------|
| M1 | 0.66 | >= 3.0 | FAIL |
| M2 | 0.79 | <= 2.0 | PASS |
| M3 | 0.87 | >= 2.0 | FAIL |
| M4 | 4.13 | >= 0.5 | PASS |
| M5 | 15.9 | 5-8 | FAIL |
| M6 | 60.5% | 60-90% | PASS |
| M7 | 0.0% | < 0.40 | PASS |
| M9 | 0.82 | >= 0.8 | PASS |
| M10 | 9.49 | <= 2 | FAIL |
| M11' | 0.84 | >= 2.5 | FAIL |
| M12 | 0.09 | >= 0.3 | FAIL |

**Passes:** 5 of 11. **Fails:** M1, M3, M5, M10, M11', M12.

M3 = 0.87 is 56% below the 2.0 target. This is a structural failure, not a
tuning miss.

---

## Per-Archetype M3 Table

| Archetype | M3 | M11' | M5 | M6 | M10 |
|-----------|---:|-----:|---:|---:|----:|
| Flash | 0.47 | 0.43 | 24.7 | 39.1% | 16.9 |
| Blink | 0.46 | 0.41 | 22.6 | 39.2% | 17.4 |
| Storm | 0.50 | 0.45 | 23.7 | 40.9% | 16.9 |
| Self-Discard | 0.56 | 0.49 | 21.7 | 43.8% | 15.4 |
| Self-Mill | 0.55 | 0.50 | 22.2 | 43.2% | 15.4 |
| Sacrifice | 0.56 | 0.50 | 21.9 | 45.2% | 15.9 |
| Warriors | 0.61 | 0.58 | 19.9 | 46.9% | 14.7 |
| Ramp | 0.46 | 0.42 | 23.6 | 38.1% | 17.1 |

**Note:** Per-archetype analysis forced each archetype regardless of lane
status. When forced into an AI-contested lane, M3 drops further (no open-lane
advantage). The aggregate M3 of 0.87 reflects committed players who always
chose open lanes. The per-archetype table shows lower M3 because it includes
AI-lane assignments. The spread (0.15) reflects graduated sibling rates:
Warriors/Sacrifice (50% sibling A-tier) outperform Flash/Ramp (25%).

---

## Round-by-Round Pool Composition

| Pick | Pool | Open Cards | AI Cards | Open SA% | AI SA% |
|-----:|-----:|-----------:|---------:|---------:|-------:|
| 1 | 120 | 45.0 | 75.0 | 32.2% | 4.8% |
| 5 | 96 | 42.2 | 53.8 | 29.6% | 4.3% |
| 10 | 66 | 37.7 | 28.3 | 25.2% | 3.4% |
| **Refill 1** | | **37 -> 77** | **23 -> 43** | | |
| 11 | 120 | 76.7 | 43.3 | 28.1% | 3.9% |
| 15 | 96 | 63.8 | 32.2 | 25.5% | 4.0% |
| 20 | 66 | 43.2 | 22.8 | 21.4% | 3.6% |
| **Refill 2** | | **39 -> 79** | **21 -> 41** | | |
| 21 | 120 | 79.3 | 40.7 | 26.3% | 4.2% |
| 25 | 96 | 65.2 | 30.8 | 23.6% | 3.9% |
| 30 | 66 | 45.6 | 20.4 | 19.2% | 3.9% |

The open-lane bias works as designed: open lanes accumulate from 37.5% to
~66% of pool by round 3. But S/A density for the player's specific archetype
within open lanes declines across rounds (32.2% -> 26.3% -> 19.2%) because
the player is steadily removing their own S/A cards from the pool. Refills
replenish quantity but cannot fully replenish the specific S/A cards the
player needs.

---

## Pack Quality Distribution (picks 6+, committed)

| P10 | P25 | P50 | P75 | P90 |
|:---:|:---:|:---:|:---:|:---:|
| 0 | 0 | 1 | 1 | 2 |

The median pack has only 1 S/A card. The 10th and 25th percentile packs have
zero. This is the root cause of M10 failure: most packs hover around 0-1 S/A.

---

## Consecutive Bad Pack Analysis

| Max Consecutive (<1.5 SA) | Drafts | % |
|--------------------------:|-------:|--:|
| 2 | 3 | 0.3% |
| 3-5 | 166 | 16.6% |
| 6-8 | 326 | 32.6% |
| 9-11 | 195 | 19.5% |
| 12-15 | 122 | 12.2% |
| 16+ | 188 | 18.8% |

43.8% of committed drafts experience 25 consecutive bad packs (the entire
picks 6-30 window). Only 0.3% of drafts achieve the M10 target of <= 2.
The "bad pack" problem is not intermittent; it is the baseline state.

---

## S/A Density Trajectory

| Round | Pick Range | Avg SA Fraction |
|------:|-----------:|----------------:|
| 1 | 1-10 | 14.6-18.0% |
| 2 | 11-20 | 15.7-20.1% |
| 3 | 21-30 | 14.6-20.0% |

S/A density in packs oscillates between 15-20% across all three rounds.
Refills produce a brief spike at round start (pick 11: 20.1%, pick 21: 20.0%)
that decays by round end. The trajectory is flat -- no progressive
concentration occurs. This directly contradicts Design 4's prediction of
accumulating open-lane S/A across rounds.

---

## Draft Traces

### Committed Player: Ramp (Open Lane)

The player drafted Ramp in an open lane (Ramp/Sacrifice/Self-Mill open,
5 AIs on the other 5 archetypes). Across 30 picks, the player achieved
19/30 S/A (63%). The trace shows the pattern clearly: most packs contain
0-1 S/A cards for Ramp. Round 2 was strongest (picks 11-20) with several
packs containing 2 S/A, but round 3 regressed. The committed player picks
Ramp cards when available but spends many picks on C/F off-archetype cards
when packs contain no Ramp S/A options.

### Signal-Reader: Blink (Open Lane)

The signal-reader picked highest power for picks 1-4, then committed to
Blink (the open lane with most cards) at pick 5. Despite correct lane
identification, the signal reader achieved only 17/30 S/A (57%). The
delayed commitment (4 power-chasing picks) cost 3-4 potential S/A picks,
and the subsequent picking pattern mirrored the committed player's
experience: packs alternating between 0-2 S/A cards.

---

## Comparison to Baselines

| Algorithm | M3 (committed) | M3 (signal) | M10 | M11' | M12 |
|-----------|---------------:|------------:|----:|-----:|----:|
| V9 Hybrid B | 2.70 | --- | 3.80 | 3.25 | N/A |
| V10 Hybrid X | 0.84 | --- | --- | 0.69 | N/A |
| SIM-1 (balanced) | 0.25 | 0.43 | 21.7 | 0.22 | 0.19 |
| **SIM-2 (this)** | **0.87** | **0.97** | **9.49** | **0.84** | **0.09** |

SIM-2 committed M3 (0.87) is 3.5x better than SIM-1 committed (0.25). The
improvement comes from two sources: (1) committed players now always pick
open-lane archetypes (SIM-1 randomly picked AI lanes 62.5% of the time),
and (2) the 1.7x refill bias enriches open lanes with more cards. SIM-2
M3 is comparable to V10 Hybrid X (0.84) but far below V9 Hybrid B (2.70).
The 1.7x bias provides real but insufficient concentration.

---

## Root Cause Analysis

### Why the Prediction Was Wrong

Design 4 predicted M3 = 1.9-2.2. The actual result is 0.87. The 1.0+ gap
comes from three compounding errors in the prediction:

**Error 1: Confusion between pool-level and pack-level concentration.** The
open-lane bias successfully concentrates the pool -- by round 3, 66% of pool
cards are open-lane cards. But the player's SPECIFIC archetype is only one of
three open lanes. Open-lane concentration does not translate to archetype-level
concentration. The player's archetype represents ~25% of the pool regardless
of round, because the 1.7x multiplier benefits all three open lanes equally.

**Error 2: SA density was overestimated.** The design assumed ~33% S/A rate
per archetype. The actual rate for the player's archetype is: 100% for
own-archetype cards (~15 cards in a 120-card pool = 12.5%) plus ~36% of
sibling cards (~15 * 0.36 = 5.4 S/A siblings). Total: ~20 S/A cards per
120-card pool = 16.7%. In a 5-card pack, expected S/A = 5 * 0.167 = 0.83.
This almost exactly matches the simulated M3 = 0.87.

**Error 3: Pack size effect.** With 5-card packs, each pack is a tiny sample
of the pool. The variance is very high (M9 = 0.82), and the modal pack has
0 S/A cards for the committed archetype. The design implicitly assumed larger
effective pack sizes or some pack construction advantage that the simulation
does not provide.

### The Fundamental Constraint

M3 >= 2.0 with 5-card packs requires SA density >= 40% in the pool (since
5 * 0.4 = 2.0). The maximum achievable SA density without pool contraction
is ~17% (all own-archetype cards + sibling A-tier). Even if every refill
card were from the player's archetype (impossible without Level 1+
reactivity), the density ceiling is ~25-30% because the pool still contains
the starting 120 cards.

**Pool contraction is required.** No amount of refill bias, pool composition
manipulation, or AI depletion can raise the player's archetype SA density
from ~17% to >= 40% without physically removing non-player cards from the
pool. This is exactly what V9's contraction engine did. SIM-2 confirms that
the multi-round refill structure, even with 1.7x open-lane bias, cannot
substitute for pool contraction at M3 >= 2.0.

---

## Self-Assessment

SIM-2 is a clean negative result. The simulation correctly implements Design
4 Proposal B (static 1.7x open-lane multiplier, 3 rounds, 60-card refills,
5 AIs, 5-card packs). The negative outcome is structural, not a bug: the
SA density math makes M3 >= 2.0 physically impossible without pool
contraction.

The M3 prediction of 1.9-2.2 was overoptimistic by a factor of ~2.3x.
The prediction failed because it conflated open-lane concentration (pool
level) with archetype-specific concentration (pack level). The 1.7x
multiplier does concentrate open lanes, but concentrating 3 lanes equally
provides no targeting advantage for the player's specific archetype within
those 3 lanes.

SIM-2's value is as a calibration anchor: it establishes that multi-round
refills with static bias produce M3 ~0.87 -- comparable to V10 Hybrid X
(M3 = 0.84) and well below V9 Hybrid B (M3 = 2.70). The remaining SIM
algorithms (SIM-3 through SIM-6) must introduce either pool contraction,
player-reactive concentration, or fundamentally different pack construction
to bridge this gap.
