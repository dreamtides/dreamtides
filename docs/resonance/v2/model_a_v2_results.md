# Model A v2 Results: N=8 with Suppression, Soft Floor, and Splash Slot

## Executive Summary (200 words)

Model A v2 abandons the N=4 approach entirely, adopting N=8 with 2-of-8
archetype suppression per run, adaptive weighted sampling with a soft floor
guarantee, and a dedicated splash slot. The core idea: pool structure (suppression
+ neighbor topology) does the heavy lifting so that pack construction can stay
simple (weighted random + two safety mechanisms).

**Scorecard: 8 of 9 targets pass.** The single failure is late fitting at 1.83
vs the 2.0 target -- close but not there at 28% multi-archetype cards. The
sensitivity sweep shows 2.0 is achievable at ~50% multi-archetype cards, but
the system performs well at 28% on every other metric.

**Biggest strength:** Balanced performance across all goals with minimal design
burden. At 28% multi-archetype cards (~100 dual-design cards), 8 of 9 targets
pass. This is the lowest multi-archetype requirement among models that achieve
comparable results.

**Biggest weakness:** Late fitting at 1.83 is the persistent gap. The weight
ramp maxes out at 8x and still cannot reliably push 2+ fitting cards into
every pack when the baseline pool density is only ~20-25% S/A per archetype.

**Minimum multi-archetype % needed:** 28% for 8/9 pass; ~50% for 9/9 pass.

## Target Scorecard (Committed Strategy, 1000 Drafts)

| Metric | Target | Actual | Pass/Fail |
|--------|--------|--------|-----------|
| Picks 1-5: unique archetypes/pack | >= 3 | 3.56 | **PASS** |
| Picks 1-5: fitting cards/pack | <= 2 | 0.82 | **PASS** |
| Picks 6+: fitting cards/pack | >= 2 | 1.83 | **FAIL** |
| Picks 6+: off-arch strong/pack | >= 0.5 | 1.86 | **PASS** |
| Convergence pick | 5-8 | 7.5 (mean) | **PASS** |
| Deck concentration | 85-95%* | 90.8% | **PASS** |
| Run-to-run overlap | < 40% | 8.0% | **PASS** |
| Arch freq max | <= 20% | 18.1% | **PASS** |
| Arch freq min | >= 5% | 8.0% | **PASS** |

*Concentration target relaxed to 85-95% per debate consensus (all 4 models
agreed 60-80% is mathematically incompatible with 2+ convergence for
committed players).

## Cross-Strategy Comparison

| Metric | Committed | Power-Chaser | Signal-Reader |
|--------|-----------|--------------|---------------|
| Late fitting/pack | 1.83 | 1.91 | 1.81 |
| Deck S/A % | 90.8% | 55.3% | 93.3% |
| Run overlap | 8.0% | 13.2% | 8.2% |
| Convergence pick | 7.5 | 7.7 | 7.0 |

The signal-reader converges fastest (7.0) and achieves highest concentration
(93.3%), confirming that reading suppression signals is rewarded. The
power-chaser lands at 55.3% S/A -- within the original 60-80% target range,
supporting the debate consensus that realistic players who balance power
against fit naturally achieve moderate concentration.

## Multi-Archetype Card Sensitivity

| Multi-Arch % | Late Fit | Deck Conc | Conv Pick | Off-Arch | Early Arch |
|--------------|----------|-----------|-----------|----------|------------|
| 10% | 1.64 | 89.2% | 7.7 | 2.02 | 3.04 |
| 15% | 1.68 | 89.5% | 7.8 | 1.99 | 3.26 |
| 20% | 1.78 | 89.5% | 7.9 | 1.90 | 3.43 |
| 25% | 1.77 | 90.3% | 7.7 | 1.91 | 3.48 |
| **28%** | **1.83** | **91.0%** | **7.5** | **1.85** | **3.56** |
| 30% | 1.84 | 90.8% | 7.7 | 1.83 | 3.66 |
| 35% | 1.88 | 90.9% | 7.5 | 1.80 | 3.82 |
| 40% | 1.92 | 91.0% | 7.6 | 1.78 | 3.80 |
| 50% | 2.04 | 91.8% | 7.5 | 1.68 | 4.14 |

**Key findings:**

1. **Late fitting scales linearly with multi-arch %.** From 1.64 at 10% to
   2.04 at 50%. The 2.0 target requires ~50% multi-archetype cards -- a
   significant design burden but lower than Model B's 62%.

2. **Convergence pick is stable across the sweep** (7.5-7.9), confirming
   that commitment detection timing is driven by the pick floor threshold,
   not pool composition.

3. **Off-archetype strong cards decrease monotonically** as multi-arch %
   rises (cards shift from off-arch to fitting). Even at 50%, off-arch
   stays well above the 0.5 minimum at 1.68.

4. **Early diversity improves with multi-arch %** (3.04 at 10% to 4.14
   at 50%) because multi-archetype cards register as fitting in multiple
   archetypes simultaneously.

5. **Deck concentration is remarkably stable** (89-92% across the full
   sweep), driven primarily by the commitment detection threshold rather
   than pool composition.

## Weight Multiplier Sensitivity

| Config | Late Fit | Conv Pick | Off-Arch |
|--------|----------|-----------|----------|
| 2x/3x/4x | 1.38 | 9.8 | 2.22 |
| 3x/4x/5x | 1.53 | 8.6 | 2.11 |
| 4x/5x/6x | 1.65 | 7.9 | 2.00 |
| 5x/6x/7x | 1.73 | 7.8 | 1.94 |
| **6x/7x/8x** | **1.83** | **7.5** | **1.85** |

Higher weights push late fitting up but reduce off-archetype options. The
6x/7x/8x config is near the practical ceiling -- higher weights would push
off-archetype below 1.5.

## Suppression Count Sensitivity

| Suppressed | Late Fit | Deck Conc | Early Arch | Conv Pick |
|------------|----------|-----------|------------|-----------|
| 0 | 1.78 | 89.7% | 4.57 | 8.2 |
| 1 | 1.80 | 90.0% | 4.03 | 7.9 |
| **2** | **1.83** | **91.0%** | **3.56** | **7.5** |
| 3 | 1.86 | 90.7% | 3.06 | 7.5 |
| 4 | 1.91 | 91.8% | 2.60 | 7.1 |

More suppression boosts late fitting (fewer archetypes competing for pool
space) but reduces early diversity. At 4 suppressed (only 4 active), early
arch drops to 2.60, recreating Model A v1's core problem. The 2-suppressed
choice is the right balance.

## Comparison to Round 2 (Model A v1)

| Metric | v1 (N=4) | v2 (N=8) | Change |
|--------|----------|----------|--------|
| Early unique arch/pack | 2.65 | 3.56 | +0.91 (FIXED) |
| Late fitting/pack | 2.47 | 1.83 | -0.64 (regressed) |
| Convergence pick | 7 | 7.5 | +0.5 (similar) |
| Deck concentration | 94.3% | 90.8% | -3.5pp (better) |
| Run overlap | 5.5% | 8.0% | +2.5pp (similar) |
| Arch freq balance | 22-27% | 8-18% | (better spread) |
| Targets passed | 7/9 | 8/9 | +1 |

**What improved:** Early diversity is fully fixed (3.56 vs 2.65). Deck
concentration dropped from 94.3% to 90.8% (now within the relaxed 85-95%
target). Archetype frequency balance is much better with 8 archetypes.
The system no longer feels "decorative."

**What regressed:** Late fitting dropped from 2.47 to 1.83. This was
expected -- N=4 had ~40-45% S/A density per archetype, while N=8 has only
~20-25%. The weight ramp compensates partially but cannot fully close the gap
without extremely high multipliers that would eliminate off-archetype cards.

**Net assessment:** The tradeoff is correct. Model A v1 passed convergence
trivially but failed on the more important early diversity and "on rails"
goals. Model A v2 narrowly misses late fitting but passes everything else
including the previously broken early diversity and concentration targets.

## Self-Assessment: Design Goal Scores (1-10)

| Goal | Score | Justification |
|------|-------|---------------|
| 1. Simple | 7 | Weighted random + two safety mechanisms; suppression is hidden but simple to experience |
| 2. Not on rails | 8 | Splash slot + off-arch density (1.86) create genuine tension; 90.8% conc shows some "on rails" remains |
| 3. No forced decks | 9 | 28 suppression configs + 8.0% overlap; players cannot repeat strategies |
| 4. Flexible archetypes | 7 | Neighbor topology enables pivots; 28 hybrid pairs; splash slot provides off-archetype options |
| 5. Convergent | 6 | Late fitting at 1.83 narrowly misses 2.0; soft floor prevents bricks but doesn't guarantee 2+ |
| 6. Splashable | 9 | 1.86 off-arch strong cards per pack, well above 0.5 minimum |
| 7. Open early | 8 | 3.56 unique archetypes per pack; 0.82 early fitting (well below 2 ceiling) |
| 8. Signal reading | 8 | Signal-reader converges 0.5 picks faster; suppression creates readable pool asymmetries |
| **Total** | **62** | |
