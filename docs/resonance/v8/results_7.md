# Results Agent 7: CSCT (Commitment-Scaled Continuous Targeting)

## Algorithm Summary

**One-sentence:** The number of pair-matched pack slots scales continuously with
the player's commitment ratio (S/A cards drafted for top archetype divided by
total picks), with no discrete surge/floor modes — just smooth proportional
targeting.

**Variant tested:** Jittered+Bias — multiplier=5, bias=1.5x on random slots
toward primary resonance, 15% per-slot jitter for variance, minimum 1
pair-matched slot from pick 3 onward.

______________________________________________________________________

## Set Design Specification (40% Enriched Pool as Simulated)

### 1. Pool Breakdown by Archetype

| Archetype            |  Total  | Home-Only | Cross-Archetype (dual-res) | Generic |
| -------------------- | :-----: | :-------: | :------------------------: | :-----: |
| Flash (Ze/Em)        |   40    |    22     |             18             |   --    |
| Blink (Em/Ze)        |   40    |    22     |             18             |   --    |
| Storm (Em/St)        |   40    |    22     |             18             |   --    |
| Self-Discard (St/Em) |   40    |    22     |             18             |   --    |
| Self-Mill (St/Ti)    |   40    |    22     |             18             |   --    |
| Sacrifice (Ti/St)    |   40    |    22     |             18             |   --    |
| Warriors (Ti/Ze)     |   40    |    22     |             18             |   --    |
| Ramp (Ze/Ti)         |   40    |    22     |             18             |   --    |
| Generic              |   40    |    --     |             --             |   40    |
| **Total**            | **360** |  **176**  |          **144**           | **40**  |

### 2. Symbol Distribution

|            Symbol Count             | Cards |   %   | Example               |
| :---------------------------------: | :---: | :---: | --------------------- |
|             0 (generic)             |  40   | 11.1% | No resonance          |
|              1 symbol               |  176  | 48.9% | (Tide)                |
| 2 symbols (different, ordered pair) |  ~58  | 16.1% | (Tide, Zephyr)        |
|      3 symbols (pair + splash)      |  ~86  | 23.9% | (Tide, Zephyr, Ember) |

### 3. Dual-Resonance Breakdown

| Type                             | Cards |   %   | Filtering Implications                  |
| -------------------------------- | :---: | :---: | --------------------------------------- |
| No resonance (generic)           |  40   | 11.1% | Not filterable                          |
| Single-resonance                 |  176  | 48.9% | Matches 2 archetypes on R1 filter       |
| Dual+ resonance (pair-matchable) |  144  | 40.0% | Matches 1 archetype pair on pair filter |

### 4. Per-Resonance Pool Sizes

| Resonance | Primary Symbol | Pair-Matched Cards/Archetype |
| --------- | :------------: | :--------------------------: |
| Ember     |       80       |  ~18 per Ember-primary pair  |
| Stone     |       80       |  ~18 per Stone-primary pair  |
| Tide      |       80       |  ~18 per Tide-primary pair   |
| Zephyr    |       80       | ~18 per Zephyr-primary pair  |

### 5. Cross-Archetype Requirements

Of each archetype's 40 cards, 18 (45%) carry dual-resonance symbols. Under
Graduated Realistic fitness, the A-tier rates for sibling are:
Warriors/Sacrifice 50%, Self-Discard/Self-Mill 40%, Blink/Storm 30%, Flash/Ramp
25%.

### 6. Card Designer Guidance

Compared to V7: increase dual-resonance cards from 54 to 144. Every non-generic
card should have its primary resonance; 45% should also carry the secondary. For
low-overlap pairs (Flash/Ramp, Blink/Storm), 10-12 intentional bridge cards per
archetype are needed.

______________________________________________________________________

## Core Results: Committed Strategy Scorecard

| Pool     | Fitness     |  M1  |  M2  |    M3    |  M4  | M5  |  M6   |  M7   |  M9  | M10 |
| -------- | ----------- | :--: | :--: | :------: | :--: | :-: | :---: | :---: | :--: | :-: |
| V7 15%   | Optimistic  | 4.10 | 2.79 | **3.08** | 0.46 | 5.0 | 99.2% | 10.4% | 0.69 |  2  |
| V7 15%   | Grad. Real. | 3.30 | 2.61 | **2.93** | 0.46 | 5.0 | 99.0% | 7.4%  | 0.68 |  3  |
| V7 15%   | Pessimistic | 2.86 | 2.57 | **2.88** | 0.46 | 5.0 | 98.9% | 6.5%  | 0.67 |  3  |
| V7 15%   | Hostile     | 2.62 | 2.54 | **2.86** | 0.46 | 5.0 | 98.8% | 0.66  | 0.66 |  4  |
| 40% Enr. | Optimistic  | 4.12 | 2.78 | **3.07** | 0.47 | 5.0 | 99.1% | 11.7% | 0.68 |  2  |
| 40% Enr. | Grad. Real. | 3.26 | 2.61 | **2.92** | 0.47 | 5.0 | 99.0% | 7.8%  | 0.68 |  2  |
| 40% Enr. | Pessimistic | 2.87 | 2.56 | **2.88** | 0.47 | 5.0 | 98.9% | 6.9%  | 0.67 |  2  |
| 40% Enr. | Hostile     | 2.60 | 2.54 | **2.85** | 0.47 | 5.0 | 98.8% | 8.2%  | 0.67 |  2  |

**Targets:** M1>=3, M2\<=2, M3>=2.0, M4>=0.5, M5 in 5-8, M6 60-90%, M7\<40%,
M9>=0.8, M10\<=2

### Metric Pass/Fail Summary (Graduated Realistic, 40% Enriched)

| Metric | Value | Target |       Status        |
| ------ | :---: | :----: | :-----------------: |
| M1     | 3.26  |  >=3   |        PASS         |
| M2     | 2.61  |  \<=2  |      **FAIL**       |
| M3     | 2.92  | >=2.0  |        PASS         |
| M4     | 0.47  | >=0.5  | **FAIL** (marginal) |
| M5     |  5.0  |  5-8   |        PASS         |
| M6     | 99.0% | 60-90% |      **FAIL**       |
| M7     | 7.8%  | \<40%  |        PASS         |
| M9     | 0.68  | >=0.8  |      **FAIL**       |
| M10    |   2   |  \<=2  |        PASS         |

**Score: 5/9 pass.** M3 performance is outstanding (2.85+ even under Hostile),
but M6 (concentration), M9 (variance), M2 (early openness), and M4 (splash) all
fail.

______________________________________________________________________

## Pack Quality Distribution (Committed, Picks 6+)

| Condition              | P10 | P25 | P50 | P75 | P90 |
| ---------------------- | :-: | :-: | :-: | :-: | :-: |
| 40% Enr. + Grad. Real. |  2  |  3  |  3  |  3  |  4  |
| 40% Enr. + Pessimistic |  2  |  3  |  3  |  3  |  4  |
| 40% Enr. + Hostile     |  2  |  3  |  3  |  3  |  4  |
| V7 15% + Grad. Real.   |  2  |  3  |  3  |  3  |  4  |

The distribution is remarkably tight: the median pack delivers 3 S/A cards. The
10th percentile never drops below 2. This is the smoothness advantage CSCT was
designed for — but it comes at the cost of M9 variance (stddev 0.67-0.68, below
the 0.8 target).

## Consecutive Bad Pack Analysis

| Pool         | Fitness     | Avg Worst Run | Max Worst Run | Avg Run Length |
| ------------ | ----------- | :-----------: | :-----------: | :------------: |
| 40% Enriched | Grad. Real. |      0.5      |       2       |      0.5       |
| 40% Enriched | Pessimistic |      0.5      |       2       |      0.5       |
| 40% Enriched | Hostile     |      0.5      |       2       |      0.5       |
| V7 15%       | Grad. Real. |      0.5      |       3       |      0.5       |
| V7 15%       | Hostile     |      0.5      |       4       |      0.5       |

On the 40% Enriched pool, the worst-case consecutive bad pack streak is 2 across
all fitness levels. The average worst run is 0.5, meaning most drafts have zero
or one bad pack in the entire post-commitment phase.

______________________________________________________________________

## Fitness Degradation Curve (40% Enriched, Uniform Rate)

| Fitness Rate |  M3  | Delta from 100% |
| :----------: | :--: | :-------------: |
|     100%     | 3.07 |       --        |
|     80%      | 3.03 |      -0.04      |
|     60%      | 2.98 |      -0.09      |
|     50%      | 2.97 |      -0.10      |
|     40%      | 2.94 |      -0.13      |
|     30%      | 2.91 |      -0.16      |
|     20%      | 2.89 |      -0.18      |
|     10%      | 2.86 |      -0.21      |
|      0%      | 2.84 |      -0.23      |

The total degradation from Optimistic to Hostile is only 0.23 M3. This is CSCT's
strongest result: **the algorithm is nearly fitness-immune.** The pair-matching
mechanism concentrates draws on home-archetype cards, so sibling fitness barely
matters. Even at 0% fitness (every sibling card is B-tier), M3 remains 2.84.

______________________________________________________________________

## Per-Archetype Convergence (Graduated Realistic, 40% Enriched)

| Archetype    | Avg Conv. Pick |  N  | % Converged by Pick 8 |
| ------------ | :------------: | :-: | :-------------------: |
| Flash        |      5.0       | 167 |         100%          |
| Blink        |      5.0       | 146 |         100%          |
| Storm        |      5.0       | 122 |         100%          |
| Self-Discard |      5.0       | 165 |         100%          |
| Self-Mill    |      5.0       | 52  |         100%          |
| Sacrifice    |      5.0       | 219 |         100%          |
| Warriors     |      5.0       | 43  |         100%          |
| Ramp         |      5.0       | 86  |         100%          |

Convergence is universal by pick 5. This is too fast — the algorithm locks in
immediately, contributing to the M6 concentration problem.

### Per-Archetype M3 (Graduated Realistic, 40% Enriched)

| Archetype    |  M3  |
| ------------ | :--: |
| Flash        | 2.91 |
| Blink        | 2.91 |
| Storm        | 2.88 |
| Self-Discard | 2.94 |
| Self-Mill    | 2.92 |
| Sacrifice    | 2.96 |
| Warriors     | 2.94 |
| Ramp         | 2.89 |

The per-archetype M3 spread is extremely tight: 2.88 (Storm) to 2.96
(Sacrifice), a gap of only 0.08. This is excellent equity across archetypes, far
better than the 0.5+ gap predicted by uniform fitness models.

______________________________________________________________________

## Strategy Comparison (Graduated Realistic, 40% Enriched)

| Strategy      |  M3  |  M4  | M5  |  M6   |  M9  | M10 |
| ------------- | :--: | :--: | :-: | :---: | :--: | :-: |
| Committed     | 2.92 | 0.47 | 5.0 | 99.0% | 0.68 |  2  |
| Power Chaser  | 2.82 | 0.51 | 5.4 | 71.2% | 0.75 | 11  |
| Signal Reader | 2.92 | 0.47 | 5.1 | 98.0% | 0.68 |  2  |

Power chasers get lower M3 (2.82) and much worse M10 (11 consecutive bad packs)
because their commitment ratio fluctuates. Signal readers perform identically to
committed players.

______________________________________________________________________

## Parameter Sensitivity

**Multiplier:** Saturates at mult=4-5 (M3 2.92-2.93). Lower mult (3) reduces M3
to 2.77 but improves M9 to 0.75.

**Bias:** Each +0.5 bias adds ~0.06 M3. At bias=2.0, M3 reaches 2.99. Bias has
no meaningful effect on M9.

**Jitter:** The key variance lever. At 0% jitter, M3=3.23 but M9=0.42 (severe
fail). At 30%, M9=0.82 (passes) but M3 drops to 2.68 and M10 degrades to 4. The
15% jitter point balances M3 (2.93) with M9 (0.67) — close to but not reaching
the 0.8 target.

______________________________________________________________________

## Comparison to Agent 1 Baselines

No Agent 1 results exist yet. Against V7's reported Surge+Floor (T=3) at
Moderate fitness:

| Algorithm          | M3 (Mod.) | M10  |  M6  |  M9  |
| ------------------ | :-------: | :--: | :--: | :--: |
| V7 Surge+Floor     |   1.85    | ~3-4 | ~75% | ~0.9 |
| CSCT (Grad. Real.) |   2.92    |  2   | 99%  | 0.68 |

CSCT delivers +1.07 M3 and better M10, but far worse M6 and M9. The tradeoff is
clear: CSCT's pair-matching is so effective that it eliminates both bad packs
AND good variance.

______________________________________________________________________

## Self-Assessment

**Strengths:**

- M3 is exceptionally strong (2.85+) under all fitness levels including Hostile
  — the algorithm is nearly fitness-immune
- Smoothest delivery of any algorithm: tight pack quality distribution, minimal
  bad streaks (M10 passes)
- Near-perfect per-archetype equity (0.08 M3 spread)
- Fitness degradation is only 0.23 across the full range

**Failures:**

- **M6 (99% concentration) is the critical failure.** The committed player ends
  up with an almost mono-archetype deck. This violates the "not on rails" design
  goal. The algorithm is too effective at finding on-archetype cards and the
  committed strategy always takes them.
- **M9 (0.68 variance) fails.** Pack quality is too consistent — every pack
  looks the same. The jitter mechanism helps but cannot reach 0.8 without
  sacrificing M3 below 2.7.
- **M2 (2.61 early SA) fails.** Early packs already have too many S/A cards
  because the floor kicks in at pick 3 and commitment ratio rises fast.
- **M4 (0.47 off-archetype) marginally fails.** Pair-matching leaves little room
  for off-archetype splash.

**Root Cause:** CSCT's continuous targeting is too aggressive. The commitment
ratio rises quickly (often 1.0 by pick 5), immediately maxing out pair-matched
slots. This creates "on rails" behavior despite no discrete locking mechanism.
The algorithm needs a mechanism to deliberately inject randomness or cap
targeting intensity to preserve M6 and M9.

**What would fix it:** Cap pair-matched slots at 2 instead of 3, reduce
multiplier to 3, or add a hard ceiling where targeting intensity cannot exceed
60% of slots. This would sacrifice ~0.3 M3 (bringing it to ~2.6) while
potentially bringing M6 into range. The algorithm has significant M3 headroom to
trade for other metrics.
