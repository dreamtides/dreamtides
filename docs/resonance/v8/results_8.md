# Results: Agent 8 — Compensated Pair Allocation

## Algorithm (one sentence)

Every post-commitment pack contains pair-matched slots (drawn from the player's
leading archetype-pair subpool), R1-filtered slots, and random slots, with
graduated ramp (picks 1-2 random, 3-5: 1+1+2, 6+: jittered 2+1+1) and a
non-uniform pool distributing more dual-resonance cards to mechanically distant
archetype pairs.

## Set Design Specification (40% Enriched, Compensated)

**Pool Breakdown:**

| Archetype           |  Total  | Home-Only | Dual-Res | Tri-Res | Generic |
| ------------------- | :-----: | :-------: | :------: | :-----: | :-----: |
| Flash (Ze/Em)       |   40    |    18     |    14    |    8    |   --    |
| Blink (Em/Ze)       |   40    |    18     |    14    |    8    |   --    |
| Storm (Em/St)       |   40    |    14     |    18    |    8    |   --    |
| SelfDiscard (St/Em) |   40    |    14     |    18    |    8    |   --    |
| SelfMill (St/Ti)    |   40    |    16     |    16    |    8    |   --    |
| Sacrifice (Ti/St)   |   40    |    16     |    16    |    8    |   --    |
| Warriors (Ti/Ze)    |   40    |    18     |    14    |    8    |   --    |
| Ramp (Ze/Ti)        |   40    |    18     |    14    |    8    |   --    |
| Generic             |   40    |    --     |    --    |   --    |   40    |
| **Total**           | **360** |  **132**  | **124**  | **64**  | **40**  |

"Compensated" = high-overlap pairs get 14 dual-res, low-overlap pairs get 18.
Uniform variant: all 18.

**Symbol Distribution:**

| Type                               | Cards |  %  |
| ---------------------------------- | :---: | :-: |
| 0 symbols (generic)                |  40   | 11% |
| 2 same-res (e.g. Tide/Tide)        |  132  | 37% |
| 2 different-res (e.g. Tide/Zephyr) |  124  | 34% |
| 3 symbols (e.g. Tide/Zephyr/Stone) |  64   | 18% |

**Per-Resonance R1 Pool:** Each resonance has ~80 cards as primary. The
pair-matched subpool per archetype contains 14-22 cards (dual-res + tri-res
contributions).

## Scorecard: Committed Strategy (Primary Target)

| Metric  | Target | Opt (V7) | Opt (40%) | GR (V7) | GR (40%) | Pess (40%) | Host (40%) |
| ------- | ------ | :------: | :-------: | :-----: | :------: | :--------: | :--------: |
| M1      | >= 3   |   5.39   |   5.68    |  4.61   |   4.72   |    4.53    |    4.25    |
| M2      | \<= 2  |   2.54   |   2.43    |  2.27   |   2.18   |    2.11    |    2.02    |
| M3      | >= 2.0 | **2.54** | **2.16**  |  1.62   |   1.45   |    1.40    |    1.29    |
| M4      | >= 0.5 |   1.46   |   1.84    |  2.38   |   2.55   |    2.60    |    2.71    |
| M5      | 5-8    |   8.9    |    7.0    |  13.9   |   11.4   |    12.5    |    13.7    |
| M6      | 60-90% |  79.9%   |   85.8%   |  54.8%  |  61.8%   |   61.9%    |   61.3%    |
| M7      | < 40%  |  15.0%   |   16.7%   |  9.6%   |  13.5%   |   14.7%    |   14.7%    |
| M8      | 5-20%  | 10.8-14% |  11-14%   | 12-13%  |  11-14%  |   11-14%   |   11-14%   |
| M9      | >= 0.8 |   0.51   | **0.82**  |  0.64   | **0.83** |  **0.82**  |  **0.81**  |
| M10 avg | \<= 2  |   4.9    |    3.2    |   9.1   |   6.9    |    7.1     |    8.1     |

V7 = 15% dual-res pool. 40% = 40% Enriched compensated. Bold = passes target.

## Pack Quality Distribution (Committed, Picks 6+)

| Pool         | Fitness     | p10 | p25 | p50 | p75 | p90 |
| ------------ | ----------- | :-: | :-: | :-: | :-: | :-: |
| 40% Enriched | Optimistic  |  1  |  2  |  2  |  3  |  3  |
| 40% Enriched | Graduated   |  0  |  1  |  1  |  2  |  3  |
| 40% Enriched | Pessimistic |  0  |  1  |  1  |  2  |  3  |
| 40% Enriched | Hostile     |  0  |  0  |  1  |  2  |  3  |
| Surge+Floor  | Optimistic  |  1  |  1  |  2  |  3  |  4  |
| Surge+Floor  | Graduated   |  0  |  1  |  2  |  2  |  3  |

## Consecutive Bad Pack Analysis (Committed, S/A < 1.5)

| Pool    | Fitness     | Avg Worst Run | Global Worst |
| ------- | ----------- | :-----------: | :----------: |
| CPA 40% | Optimistic  |      3.2      |      25      |
| CPA 40% | Graduated   |      6.9      |      25      |
| CPA 40% | Pessimistic |      7.1      |      25      |
| CPA 40% | Hostile     |      8.1      |      25      |
| S+F V7  | Optimistic  |      1.7      |      2       |
| S+F V7  | Graduated   |      3.2      |      11      |

The global worst of 25 represents drafts where the pair counters pointed to the
wrong archetype pair throughout.

## Fitness Degradation Curve (Committed, 40% Enriched)

| Fitness     | CPA M3 | S+F M3 (V7) | S+F M3 (40%) | CPA Delta vs S+F V7 |
| ----------- | :----: | :---------: | :----------: | :-----------------: |
| Optimistic  |  2.16  |    2.30     |     2.32     |        -0.15        |
| Graduated   |  1.45  |    1.66     |     1.65     |        -0.21        |
| Pessimistic |  1.40  |    1.48     |     1.49     |        -0.08        |
| Hostile     |  1.29  |    1.32     |     1.34     |        -0.03        |

Gap narrows at lower fitness. Under hostile, CPA is only 0.03 behind S+F.

## Per-Archetype Convergence Table (Committed, Graduated Realistic, 40% Enriched)

| Archetype   | Pair  | M3 (compensated) | M3 (uniform) | Best-Worst Gap |
| ----------- | ----- | :--------------: | :----------: | :------------: |
| Flash       | Ze/Em |       1.40       |     1.52     |                |
| Blink       | Em/Ze |       1.53       |     1.33     |                |
| Storm       | Em/St |       1.44       |     1.48     |                |
| SelfDiscard | St/Em |       1.54       |     1.49     |                |
| SelfMill    | St/Ti |       1.37       |     1.61     |                |
| Sacrifice   | Ti/St |       1.43       |     1.47     |                |
| Warriors    | Ti/Ze |       1.57       |     1.63     |                |
| Ramp        | Ze/Ti |       1.30       |     1.26     |                |
| **Worst**   |       |     **1.30**     |   **1.26**   |                |
| **Best**    |       |     **1.57**     |   **1.63**   |                |
| **Gap**     |       |     **0.27**     |   **0.37**   |                |

Compensated pool narrows the gap from 0.37 to 0.27. Neither reaches 2.0 for any
archetype under Graduated Realistic.

## Parameter Sensitivity (Graduated Realistic, 40% Enriched)

| Parameter          | Variant               |    M3    |  M9  | M10 avg |
| ------------------ | --------------------- | :------: | :--: | :-----: |
| Pair slots (fixed) | 1                     |   1.43   | 0.82 |   5.3   |
|                    | 2                     | **1.64** | 0.83 |   5.4   |
|                    | 3                     |   1.43   | 0.83 |   6.5   |
| Jitter             | None (2+1+1 fixed)    |   1.51   | 0.81 |   6.7   |
|                    | Standard (70/20/10)   | **1.59** | 0.85 |   5.5   |
|                    | Aggressive (40/40/20) |   1.39   | 0.83 |   6.8   |
| Ramp start         | Pick 3                |   1.46   | 0.82 |   --    |
|                    | Pick 4                | **1.54** | 0.84 |   --    |
|                    | Pick 5                |   1.50   | 0.81 |   --    |
|                    | Pick 6                |   1.53   | 0.83 |   --    |

2 pair slots optimal. Standard jitter beats none. Ramp start 4 slightly best.

## Draft Traces (Summary)

**Trace 1 — Warriors committed:** Post-commitment S/A = 1.32. Correct pair
identified but R1 slots delivered many Sacrifice cards. Packs oscillated 0-3
S/A.

**Trace 2 — Power chaser (Flash):** Post-commitment S/A = 2.24. Natural
alignment between power picks and pair counters. Consistent 2-3 S/A.

**Trace 3 — Signal reader (failure):** Post-commitment S/A = 0.40. Pair counters
locked onto wrong archetype, producing 25 consecutive 0-S/A packs. Reveals
structural fragility.

## Baseline Comparison

Under Graduated Realistic fitness with the committed strategy:

| Algorithm   | Pool             |    M3    |    M9    | M10 avg |    M5    |
| ----------- | ---------------- | :------: | :------: | :-----: | :------: |
| **CPA**     | **40% Enriched** | **1.45** | **0.83** | **6.9** | **11.4** |
| Surge+Floor | V7 Standard      |   1.66   |   0.94   |   3.2   |    --    |
| Surge+Floor | 40% Enriched     |   1.65   |   0.94   |   3.3   |    --    |

CPA underperforms Surge+Floor by 0.20 on M3 with worse M10. CPA's M9 (0.83)
passes the target while Surge+Floor's higher M9 (0.94) reflects bimodal
delivery.

## Self-Assessment

**CPA fails its primary M3 objective.** Theoretical M3 was 2.57; actual is 1.45.
The root cause: pair counters misalign with the committed archetype ~40% of the
time. Same-resonance doubled cards (Tide/Tide) increment both Tide-primary pairs
equally, and early random picks can lock onto the wrong pair. This misalignment
also causes M10 to be worse than Surge+Floor -- continuous allocation with
wrong-pair targeting creates bad streaks worse than Surge+Floor's predictable
alternation.

**Two findings survive the failure:** (1) Non-uniform pool distribution narrows
the per-archetype M3 gap from 0.37 to 0.27 -- this pool design insight applies
to any algorithm. (2) Pair-level targeting achieves theoretical 85% precision
when properly aligned, confirming that pair-matching is the right mechanism
class if the alignment problem is solved.

**What would fix it:** Lock the leading pair after pick 5 (never switch). This
would raise alignment from ~60% to ~95%, projecting M3 ~2.1 under Graduated
Realistic. However, this requires either an explicit player commitment or an
irreversible algorithmic lock-in, moving closer to Agent 3 or Agent 6's
territory.
