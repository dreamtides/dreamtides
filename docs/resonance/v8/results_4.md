# Results 4: GPE-45 Graduated Pair-Escalation

## One-Sentence Algorithm

Each drafted dual-resonance card increments an ordered-pair counter; per-slot
pair-matching probability ramps from min(count/8, 0.35) to min(count/5, 0.55)
across a smoothed pick 11-15 transition, with a guaranteed pair-matched floor
slot from pick 3+ and R1-resonance fallback on non-pair slots.

______________________________________________________________________

## Complete Set Design Specification

### 1. Pool Breakdown by Archetype

| Archetype            |  Total  | Home-Only (1 sym) | Cross-Arch (2 sym) | Generic |
| -------------------- | :-----: | :---------------: | :----------------: | :-----: |
| Each of 8 archetypes |   40    |        24         |         16         |   --    |
| Generic              |   40    |        --         |         --         |   40    |
| **Total**            | **360** |      **192**      |      **128**       | **40**  |

### 2. Symbol Distribution

|        Symbols         | Cards |   %   | Example        |
| :--------------------: | :---: | :---: | -------------- |
|           0            |  40   | 11.1% | No resonance   |
|           1            |  192  | 53.3% | (Tide)         |
| 2 (different, ordered) |  128  | 35.6% | (Tide, Zephyr) |

### 3. Dual-Resonance Breakdown

128 dual-res cards among 320 resonance-bearing = 40%. Each ordered pair maps to
exactly one archetype, so pair filtering achieves ~100% home-archetype
precision.

### 4. Per-Resonance Pool Sizes

Each resonance: 80 cards as primary (2 archetypes x 40). Per ordered pair: 16
dual-res cards. R1 pool per resonance: 80 cards.

### 5. Cross-Archetype Requirements

| Pair                 | Sibling A-Tier (Grad. Realistic) | Of 16 Dual-Res |
| -------------------- | :------------------------------: | :------------: |
| Warriors/Sacrifice   |               50%                |       8        |
| SelfDiscard/SelfMill |               40%                |       6        |
| Blink/Storm          |               30%                |       5        |
| Flash/Ramp           |               25%                |       4        |

### 6. Card Designer Guidance

Create 128 dual-resonance cards (up from V7's 54). Each archetype needs 16 cards
with both primary and secondary symbols in order. Cards can be mechanically
narrow to home archetype; the dual symbol is primarily a filtering signal.

______________________________________________________________________

## Scorecard (Enriched 40% Pool)

| Metric             |   Target   | Optimistic | Grad. Real. | Pessimistic | Hostile  |
| ------------------ | :--------: | :--------: | :---------: | :---------: | :------: |
| M1 (variety)       |    >= 3    |    2.86    |    2.78     |    2.78     |   2.72   |
| M2 (early S/A)     |   \<= 2    |    1.72    |    1.21     |    1.08     |   1.01   |
| **M3 (late S/A)**  | **>= 2.0** |  **2.73**  |  **2.25**   |  **2.21**   | **2.05** |
| M4 (off-arch)      |   >= 0.5   |    1.27    |    1.23     |    1.19     |   1.28   |
| M5 (convergence)   |    5-8     |    11.9    |    12.5     |    12.7     |   13.5   |
| M6 (concentration) |   60-90%   |   68.3%    |    66.7%    |    66.6%    |  63.7%   |
| M7 (overlap)       |   < 40%    |   17.4%    |    17.4%    |    17.8%    |  18.0%   |
| M8 (arch freq)     |   5-20%    |   11-15%   |   11-14%    |   10-14%    |  11-14%  |
| M9 (stddev)        |   >= 0.8   |    0.09    |    0.51     |    0.56     |   0.56   |
| M10 (consec bad)   |   \<= 2    |  7.9 avg   |   8.2 avg   |   8.0 avg   | 8.9 avg  |

**V7 Standard 15% Pool:** M3 = 2.60/2.01/1.77/1.64 across the four fitness
levels.

______________________________________________________________________

## Pack Quality Distribution (Enriched 40%, picks 6+)

| Percentile | Optim. | Grad.R. | Pess. | Hostile |
| :--------: | :----: | :-----: | :---: | :-----: |
|    10th    |   0    |    0    |   0   |    0    |
|    25th    |   0    |    0    |   0   |    0    |
|    50th    |   4    |    3    |   3   |    3    |
|    75th    |   4    |    4    |   4   |    4    |
|    90th    |   4    |    4    |   4   |    4    |

Bimodal distribution: bootstrapping-phase packs (picks 6-10) produce 0-1 S/A;
mature-phase packs (15+) deliver 3-4.

## Consecutive Bad Pack Analysis

| Fitness             | Avg Consec S/A < 1.5 | Worst |
| ------------------- | :------------------: | :---: |
| Graduated Realistic |         8.2          |  25   |
| Pessimistic         |         8.0          |  25   |
| Hostile             |         8.9          |  25   |

Worst-case 25 occurs when no dual-res card is picked early, starving the pair
counter entirely.

______________________________________________________________________

## Fitness Degradation Curve (Enriched 40%)

| Fitness         |  M3  | Degradation | Worst Archetype |
| --------------- | :--: | :---------: | :-------------: |
| Optimistic      | 2.73 |     --      |      2.59       |
| Grad. Realistic | 2.25 |   -17.6%    |      1.92       |
| Pessimistic     | 2.21 |   -19.0%    |      1.74       |
| Hostile         | 2.05 |   -24.9%    |      1.88       |

Only 25% degradation from Optimistic to Hostile, vs. V7 Surge+Floor's ~50%.

## Per-Archetype Convergence (Enriched 40%, Grad. Realistic)

| Archetype           |  M3  | Notes                              |
| ------------------- | :--: | ---------------------------------- |
| Flash (Ze/Em)       | 1.92 | Lowest -- 25% sibling fitness      |
| Ramp (Ze/Ti)        | 1.94 | Low -- shares Zephyr pair weakness |
| Blink (Em/Ze)       | 2.25 |                                    |
| SelfDiscard (St/Em) | 2.25 |                                    |
| SelfMill (St/Ti)    | 2.34 |                                    |
| Sacrifice (Ti/St)   | 2.41 | High -- 50% sibling fitness        |
| Storm (Em/St)       | 2.45 |                                    |
| Warriors (Ti/Ze)    | 2.46 | Highest                            |

______________________________________________________________________

## R1 Fallback Ablation (Enriched 40%, Grad. Realistic)

Without R1 fallback: M3 = 1.52. With R1 fallback: M3 = 2.27. Delta: **+0.75**.
This is the algorithm's most critical component -- converting wasted random
slots into R1-filtered slots at ~62-75% S/A precision.

## Parameter Sensitivity (Enriched 40%)

| Variant                        | Phase1  | Phase2  | M3 (GR)  | M3 (Pess.) |
| ------------------------------ | :-----: | :-----: | :------: | :--------: |
| Conservative (10/0.30, 6/0.50) |   low   |   low   |   2.07   |    2.13    |
| **Champion (8/0.35, 5/0.55)**  | **mid** | **mid** | **2.31** |  **2.17**  |
| Aggressive (6/0.40, 4/0.60)    |  high   |  high   |   2.24   |    2.10    |

Champion outperforms both. Aggressive over-depletes the pair pool.

______________________________________________________________________

## Draft Traces

**Trace 1 (Committed Storm):** First pick is (Ember,Zephyr) Blink card,
establishing wrong pair. R1 fallback still delivers Ember-primary cards; player
picks Storm S-tier. M3=1.88. Shows graceful degradation under pair misalignment.

**Trace 3 (Committed Sacrifice):** Picks (Tide,Stone) pair on pick 3. Counter
rises fast. By pick 10, packs consistently show 3-4 S/A. M3=3.56. Best-case
scenario with fast pair alignment.

______________________________________________________________________

## Baseline Comparison

| Algorithm      | Pool         | M3 (Optim.) | M3 (GR) | M3 (Pess.) |
| -------------- | ------------ | :---------: | :-----: | :--------: |
| V7 Surge+Floor | V7 15%       |    2.70     |  1.85   | ~1.42 est. |
| **GPE-45**     | V7 15%       |    2.60     |  2.01   |    1.77    |
| **GPE-45**     | Enriched 40% |    2.73     |  2.25   |    2.21    |

GPE-45 exceeds Surge+Floor at every realistic fitness level, with dramatically
better robustness.

______________________________________________________________________

## Self-Assessment

**Passes:** M2, M3 (all fitness levels on Enriched pool), M4, M6, M7, M8.

**Fails:** M1 (2.78 vs 3 -- R1 concentration reduces early variety), M5 (12.5 vs
5-8 -- slow pair counter bootstrap), M9 (0.51 vs 0.80 -- too consistent once
ramped), M10 (8.2 avg vs 2 -- bootstrapping creates long weak streaks before
pair-matching activates).

**Honest assessment:** GPE-45 is the first algorithm to clear M3 >= 2.0 under
all four fitness models including Hostile. However, the bootstrapping problem
(M5/M10) is fundamental: pair-escalation requires drafted dual-res cards before
it can help, creating a ~10-pick cold start. The M10 failure is the most serious
-- players experience 8+ consecutive low-quality packs before the system rewards
their commitment. This could be mitigated by increasing the guaranteed floor to
2 slots (at the cost of M1/flexibility) or by seeding the pair counter from
single-symbol picks. The M9 failure (too consistent) may actually be acceptable:
"on rails" packs feel good when they consistently deliver quality.
