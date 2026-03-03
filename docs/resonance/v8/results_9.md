# Results Agent 9: Narrative Gravity

## Algorithm

**One sentence:** After each pick, the drawable pool permanently shrinks by
removing cards whose resonance profile is most distant from the player's
accumulated resonance signature.

**Mechanism:** Maintain a 4-element resonance signature vector updated by
drafted card symbols (+2 primary, +1 secondary). After each pick (starting pick
4), compute dot-product relevance for every pool card; remove the bottom R% of
the pool. Generics receive a protected baseline relevance of 0.5. The
contraction rate is adaptive: slightly faster for committed players (based on
commitment ratio = max_signature / total_signature).

**Champion variant (Ultra-Aggressive):** Ramp phase picks 4-8: remove ~15% per
pick; post-ramp picks 9-30: remove ~12% per pick. Pool contracts from 360 to
roughly 10-15 cards by pick 30.

______________________________________________________________________

## Set Design Specification

### 1. Pool Breakdown by Archetype (40% Enriched)

| Archetype            |  Total  | Home-Only | Cross-Archetype | Generic |
| -------------------- | :-----: | :-------: | :-------------: | :-----: |
| Flash (Ze/Em)        |   40    |    24     |       16        |   --    |
| Blink (Em/Ze)        |   40    |    24     |       16        |   --    |
| Storm (Em/St)        |   40    |    24     |       16        |   --    |
| Self-Discard (St/Em) |   40    |    24     |       16        |   --    |
| Self-Mill (St/Ti)    |   40    |    24     |       16        |   --    |
| Sacrifice (Ti/St)    |   40    |    24     |       16        |   --    |
| Warriors (Ti/Ze)     |   40    |    24     |       16        |   --    |
| Ramp (Ze/Ti)         |   40    |    24     |       16        |   --    |
| Generic              |   40    |    --     |       --        |   40    |
| **Total**            | **360** |  **192**  |     **128**     | **40**  |

### 2. Symbol Distribution

|     Symbol Count      | Cards |   %   | Example        |
| :-------------------: | :---: | :---: | -------------- |
|      0 (generic)      |  40   | 11.1% | No resonance   |
|       1 symbol        |  192  | 53.3% | (Tide)         |
| 2 symbols (different) |  128  | 35.6% | (Tide, Zephyr) |

### 3. Dual-Resonance Breakdown

| Type                          | Cards |   %   | Notes                               |
| ----------------------------- | :---: | :---: | ----------------------------------- |
| Single-resonance              |  192  | 53.3% | Matches 2 archetypes on R1 filter   |
| Dual-resonance (pair-aligned) |  128  | 35.6% | 16 per archetype pair               |
| Generic                       |  40   | 11.1% | Protected by 0.5 baseline relevance |

### 4. Per-Resonance Pool Sizes

| Resonance | As Primary | As Any Symbol |
| --------- | :--------: | :-----------: |
| Ember     |     80     |      160      |
| Stone     |     80     |      160      |
| Tide      |     80     |      160      |
| Zephyr    |     80     |      160      |

### 5. Cross-Archetype Requirements

Of each archetype's 40 cards, 16 (40%) carry dual symbols. Under Graduated
fitness, the A-tier rates per sibling pair are: Warriors/Sacrifice 50%,
Self-Discard/Self-Mill 40%, Blink/Storm 30%, Flash/Ramp 25%. Narrative Gravity
compensates for low fitness by removing B/C-tier sibling cards from the pool.

### 6. What the Card Designer Must Do

Increase dual-resonance cards from V7's 15% to 40%. Each archetype's 16
dual-resonance cards carry (primary, secondary) symbols matching the archetype
pair. The algorithm does not depend on cross-archetype fitness being high -- it
removes unplayable siblings instead.

______________________________________________________________________

## Scorecard

### M3 (S/A per pack, picks 6+) -- Primary Results

| Pool              | Optimistic | Graduated | Pessimistic | Hostile |
| ----------------- | :--------: | :-------: | :---------: | :-----: |
| V7 Standard (15%) |    3.36    |   2.38    |    2.16     |  1.94   |
| 40% Enriched      |    3.39    |   2.75    |    2.59     |  2.49   |

### Full Metrics (Ultra-Aggressive, 40% Enriched, committed)

| Metric               | Optimistic | Graduated | Pessimistic | Hostile | Target |
| -------------------- | :--------: | :-------: | :---------: | :-----: | :----: |
| M1 (early variety)   |    2.98    |   3.01    |    2.99     |  3.00   |  >= 3  |
| M2 (early S/A cap)   |    0.93    |   0.64    |    0.57     |  0.48   | \<= 2  |
| M3 (post-commit S/A) |    3.39    |   2.75    |    2.59     |  2.49   | >= 2.0 |
| M4 (off-archetype)   |    0.61    |   1.25    |    1.41     |  1.51   | >= 0.5 |
| M5 (convergence)     |    7.5     |   10.2    |    11.0     |  11.7   |  5-8   |
| M6 (concentration)   |    0.92    |   0.85    |    0.83     |  0.80   | 60-90% |
| M7 (run variety)     |    5.5%    |   5.3%    |    5.1%     |  4.9%   | < 40%  |
| M9 (pack variance)   |    1.01    |   1.21    |    1.25     |  1.31   | >= 0.8 |
| M10 (consec bad avg) |    1.9     |    3.3    |     3.9     |   4.3   | \<= 2  |

**Pass/Fail summary (Graduated Realistic):** M1 PASS (3.01 >= 3), M2 PASS (0.64
\<= 2), M3 PASS (2.75 >= 2.0), M4 PASS (1.25 >= 0.5), M5 MARGINAL (10.2, target
5-8), M6 MARGINAL (85%, target ceiling 90%), M7 PASS (5.3% < 40%), M9 PASS (1.21
\>= 0.8), M10 FAIL (3.3, target \<= 2).

______________________________________________________________________

## Pack Quality Distribution (picks 6+, committed)

| Condition                  | P10 | P25 | P50 | P75 | P90 |
| -------------------------- | :-: | :-: | :-: | :-: | :-: |
| 40% Enriched / Graduated   |  1  |  2  |  3  |  4  |  4  |
| 40% Enriched / Pessimistic |  0  |  1  |  3  |  4  |  4  |
| V7 Standard / Graduated    |  0  |  1  |  2  |  4  |  4  |
| V7 Standard / Pessimistic  |  0  |  1  |  2  |  4  |  4  |

The distribution is bimodal: early post-commitment packs (picks 6-10) have low
S/A because the pool has not yet contracted sufficiently. Late packs (picks 20+)
are nearly all 4/4 S/A. This bimodality is the source of the M10 failure -- the
quality ramp is monotonic but the early-to-late gradient is steep.

______________________________________________________________________

## Consecutive Bad Pack Analysis

| Condition                  | Avg Consec Bad | Worst Case |
| -------------------------- | :------------: | :--------: |
| 40% Enriched / Graduated   |      3.3       |     25     |
| 40% Enriched / Pessimistic |      3.9       |     25     |
| V7 Standard / Graduated    |      4.2       |     25     |

Worst case of 25 comes from power-chaser drafts where the player never commits
to an archetype; the contraction removes the wrong cards. For committed strategy
only, worst case drops to approximately 12-15.

______________________________________________________________________

## Per-Archetype Convergence (40% Enriched, committed)

| Archetype    | M3 (Grad) | M3 (Pess) | M3 (Host) | M5 (Grad) |
| ------------ | :-------: | :-------: | :-------: | :-------: |
| Flash        |   2.40    |   2.13    |   2.16    |   10.4    |
| Blink        |   2.51    |   2.24    |   2.09    |    9.8    |
| Storm        |   2.94    |   2.85    |   2.79    |   10.7    |
| Self-Discard |   2.53    |   2.35    |   2.23    |   10.1    |
| Self-Mill    |   3.03    |   2.88    |   2.78    |    9.8    |
| Sacrifice    |   2.69    |   2.61    |   2.14    |    9.5    |
| Warriors     |   3.13    |   2.96    |   2.81    |    9.1    |
| Ramp         |   2.92    |   2.82    |   2.82    |   11.1    |

**All 8 archetypes exceed M3 >= 2.0 under Graduated and Pessimistic fitness.**
The gap between best (Warriors 3.13) and worst (Flash 2.40) under Graduated is
0.73 -- significant but both exceed the 2.0 target. Under Hostile, Flash and
Blink dip to 2.16 and 2.09 respectively but still pass.

The per-archetype gap on the V7 15% pool is much worse: Flash drops to M3=1.47
under Graduated. This confirms the 40% Enriched pool is essential for
cross-archetype fairness.

______________________________________________________________________

## Fitness Degradation Curve (40% Enriched, committed)

| Fitness               |  M3  | M6  | M10 avg |
| --------------------- | :--: | :-: | :-----: |
| Optimistic (100%)     | 3.39 | 92% |   1.9   |
| Graduated (36% avg)   | 2.75 | 85% |   3.3   |
| Pessimistic (21% avg) | 2.59 | 83% |   3.9   |
| Hostile (8%)          | 2.49 | 80% |   4.3   |

The degradation from Optimistic to Hostile is only 0.90 S/A (3.39 to 2.49). This
is remarkably flat compared to slot-filling approaches. The reason: pool
contraction removes B/C-tier sibling cards regardless of fitness, so the
surviving pool is always predominantly home-archetype cards.

______________________________________________________________________

## Parameter Sensitivity

**Contraction rate is the dominant parameter.** Sweeping from 2% to 15% per
pick:

| Rate |  M3  | M6  |  M7  | M10 |
| :--: | :--: | :-: | :--: | :-: |
|  2%  | 1.09 | 67% | 8.4% | 7.6 |
|  6%  | 1.99 | 80% | 7.0% | 4.7 |
|  8%  | 2.28 | 83% | 6.2% | 4.0 |
| 12%  | 2.77 | 86% | 5.3% | 3.2 |
| 15%  | 3.00 | 87% | 4.3% | 2.8 |
| 20%  | 2.91 | 83% | 4.2% | 2.7 |

M3 peaks around 15% then declines at 20% (pool exhausts too fast; late packs
draw from tiny pools with poor options). M6 exceeds the 90% ceiling above 15%.
The 12% setting is the best balance point.

______________________________________________________________________

## Baseline Comparison

Agent 1's results are not yet available for direct comparison. Comparing to V7's
published baselines:

| Algorithm                             | M3 (Optimistic) | M3 (Moderate/Grad) | M3 (Pessimistic) |
| ------------------------------------- | :-------------: | :----------------: | :--------------: |
| V7 Surge+Floor (T=3)                  |      2.70       |        1.85        |      ~1.42       |
| Narrative Gravity (Standard, V7 pool) |      2.52       |        1.73        |       1.53       |
| Narrative Gravity (Ultra-Agg, 40%)    |      3.39       |        2.75        |       2.59       |

Under Optimistic fitness, Narrative Gravity (Ultra-Agg) exceeds V7's best by
0.69 S/A. Under Graduated Realistic (harder than V7's Moderate), it exceeds by
0.90 S/A. Under Pessimistic, it exceeds by 1.17 S/A. The advantage grows as
fitness decreases because pool contraction bypasses the sibling fitness problem
entirely.

______________________________________________________________________

## Draft Traces

**Trace 1 -- Warriors, Graduated, committed:** Pool contracts from 360 to 11 by
pick 30. S/A rate jumps from 1/4 at pick 5 to 4/4 by pick 14. Final deck: 28/30
S/A (93%). The draft feels like a funnel: early exploration gives way to a
tight, on-theme pool. Late packs are almost entirely Warriors + Sacrifice cards.

**Trace 2 -- Blink, Graduated, signal-reader:** Player starts with power picks
(Ramp, Self-Mill, Storm) then locks onto Ember at pick 4. Pool contracts to
remove Stone/Tide cards. By pick 17, packs are 4/4 S/A. Final deck: 24/30 S/A
(80%). Signal reading works because the contraction follows the player's actual
draft signals.

**Trace 3 -- Flash, Pessimistic, power-chaser:** The power-chaser never commits
to Flash. Signature drifts randomly. Pool contracts toward whatever the player
happened to draft, which is off-archetype. Final deck: 1/30 S/A (3%). This is
the expected failure mode for power-chasing under pool contraction -- the
algorithm cannot help a player who does not signal intent.

______________________________________________________________________

## Self-Assessment

**Strengths:**

- M3 >= 2.0 across all 8 archetypes under all four fitness models on the 40%
  Enriched pool -- no other V3-V8 algorithm has achieved this.
- Exceptional fitness robustness: only 0.90 S/A degradation from Optimistic to
  Hostile.
- Smooth monotonic quality ramp: packs get strictly better over time with no
  surge/floor alternation.
- M7 is excellent (5% overlap): high run-to-run variety because the pool
  contracts differently each time based on draft choices.
- Simple mechanism: one sentence description, no counters, thresholds, or modes.

**Weaknesses:**

- M5 is late (10.2 vs target 5-8): contraction needs ~10 picks to concentrate
  the pool sufficiently. This is structural -- the pool starts at 360 cards.
- M6 exceeds the 90% ceiling (85-92%): ultra-aggressive contraction forces high
  concentration. The 12% rate sits at 86%, just below the target ceiling. Higher
  rates push above 90%.
- M10 fails (3.3 avg, target \<= 2): the transition from "pool is diverse" to
  "pool is concentrated" creates a multi-pack window (picks 6-10) where S/A is
  still low. This is the fundamental tradeoff of gradual contraction vs. instant
  surge.
- Power-chaser strategy produces terrible results: the algorithm requires the
  player to signal archetype intent through their picks. A pure power-chaser
  gets no benefit.
- The pool contracts to very small sizes late (10-15 cards by pick 30). While
  the sim draws 4-card packs successfully, in a real game this may feel
  limiting.

**Honest assessment:** Narrative Gravity achieves the M3 >= 2.0 target under
every tested fitness model, which is its primary contribution. However, it does
so by essentially filtering the pool down to home-archetype cards, which
produces high concentration (M6 risk) and eliminates splash opportunities in
late packs (M4 passes only because off-archetype cards in early post-commit
packs offset the near-zero splash in late packs). The mechanism is genuinely
novel and simple, but its aggressive contraction creates an "on rails"
late-draft experience that may conflict with Design Goal 2 ("not on rails").
