# Results Agent 3: Escalating Pair Lock

## Algorithm

"Track drafted resonance pairs; slots unlock progressively at thresholds 4/7/10.
From pick 3+, a guaranteed floor slot draws pair-matched. Locked slots draw
pair-matched with 80-90% probability; remaining slots are random."

## Set Design Specification (40% Enriched Pool)

### Pool Breakdown

| Archetype            |  Total  | Home-Only (1 sym) | Cross-Arch (2 sym) | Generic |
| -------------------- | :-----: | :---------------: | :----------------: | :-----: |
| Each of 8 archetypes |   40    |        24         |         16         |   --    |
| Generic              |   40    |        --         |         --         |   40    |
| **Total**            | **360** |      **192**      |      **128**       | **40**  |

### Symbol & Dual-Resonance Distribution

128 dual-res cards (35.6%), 192 single-res (53.3%), 40 generic (11.1%). Each
ordered archetype pair has 16 dual-res cards, all belonging to that pair's home
archetype. When pair-filtering for Warriors (Tide, Zephyr), the 16-card subpool
is 100% Warriors S-tier. Per-resonance R1 pools: 80 cards each. The card
designer creates 74 additional dual-res cards vs V7 baseline (128 vs 54). Cards
need only carry the correct symbol pair; mechanical cross-archetype fitness is
independent.

______________________________________________________________________

## Scorecard: Committed Players, All Conditions

| Metric  |  Opt/V7  | Opt/40%  |  GR/V7   |  GR/40%  | Pess/40% | Host/40%  |     Target     |
| ------- | :------: | :------: | :------: | :------: | :------: | :-------: | :------------: |
| M1      |   5.98   |   5.92   |   4.85   |   4.93   |   4.81   |   4.41    |    >=3 PASS    |
| M2      |   1.20   |   1.18   |   0.84   |   0.88   |   0.84   |   0.70    |   \<=2 PASS    |
| **M3**  | **2.01** | **1.98** | **1.48** | **1.50** | **1.46** | **1.31**  | **>=2.0 FAIL** |
| M4      |   1.49   |   1.52   |   2.02   |   2.00   |   2.04   |   2.19    |   >=0.5 PASS   |
| **M5**  | **11.3** | **11.9** | **17.4** | **16.8** | **17.3** | **19.3**  |  **5-8 FAIL**  |
| M6      |  78.8%   |  78.4%   |  64.1%   |  64.3%   |  63.1%   |   58.8%   | 60-90% PASS\*  |
| M7      |  21.1%   |  21.9%   |  19.4%   |  20.5%   |  21.5%   |   22.2%   |   \<40% PASS   |
| M9      |   1.16   |   1.15   |   1.22   |   1.23   |   1.22   |   1.20    |   >=0.8 PASS   |
| **M10** | **4.53** | **4.53** | **8.84** | **8.77** | **9.32** | **10.28** | **\<=2 FAIL**  |
| Align   |  55.4%   |  56.6%   |  55.1%   |  54.8%   |  56.9%   |   56.6%   |       --       |

M3 passes only at Optimistic/V7 (2.01). All other conditions fail M3, M5, and
M10.

______________________________________________________________________

## The Critical Finding: Pair Alignment Failure

**The algorithm's leading pair matches the player's archetype only 55% of the
time.** This is the root cause of every metric failure. When aligned, Sacrifice
at Graduated Realistic achieves M3=2.23. When misaligned, locked slots deliver
wrong-archetype cards, producing 0 S/A packs.

The alignment problem stems from exploration (picks 1-5): power-based picks
build pair counter credit for whichever pair's dual-res cards appear first. With
8 competing pairs, the correct pair wins only ~55% of the time.

### Per-Archetype M3 (40% Enriched, Graduated Realistic, Committed)

| Archetype    |  M3  | Convergence |     | Archetype |  M3  | Convergence |
| ------------ | :--: | :---------: | --- | --------- | :--: | :---------: |
| Sacrifice    | 2.23 |     9.2     |     | Ramp      | 1.35 |    18.4     |
| Blink        | 1.95 |    12.9     |     | Self-Mill | 1.13 |    20.1     |
| Self-Discard | 1.94 |    12.6     |     | Flash     | 1.11 |    22.3     |
| Warriors     | 1.36 |    15.9     |     | Storm     | 0.97 |    22.8     |

Spread is 2.23 to 0.97 -- unacceptable. Archetypes sharing a primary resonance
(Flash/Ramp, Storm/Blink) suffer because single-res picks credit both,
preventing either from dominating.

______________________________________________________________________

## Pack Quality Distribution (40% Enriched, GR, Committed, Picks 6+)

| P10 | P25 | P50 | P75 | P90 | Mean |
| :-: | :-: | :-: | :-: | :-: | :--: |
|  0  |  0  |  1  |  3  |  3  | 1.50 |

Heavily bimodal: 26.9% of packs have 0 S/A, 25.7% have 3-4 S/A. Reflects aligned
vs misaligned drafts.

## Consecutive Bad Packs (Committed, Picks 6+)

| Fitness (40% Enriched) | Avg Worst Streak | Global Worst |
| :--------------------- | :--------------: | :----------: |
| Optimistic             |       4.53       |      25      |
| Graduated Realistic    |       8.77       |      25      |
| Pessimistic            |       9.32       |      25      |
| Hostile                |      10.28       |      25      |

Global worst of 25 = entire post-commitment draft with \<1.5 S/A. Occurs in
misaligned drafts.

## Fitness Degradation (40% Enriched, Committed)

| Fitness         |  M3  |   Worst Arch    | Drop vs Opt |
| --------------- | :--: | :-------------: | :---------: |
| Optimistic      | 1.98 |   Ramp (1.77)   |     --      |
| Grad. Realistic | 1.50 |  Storm (0.97)   |    -24%     |
| Pessimistic     | 1.46 |  Storm (0.92)   |    -26%     |
| Hostile         | 1.31 | Warriors (0.89) |    -34%     |

______________________________________________________________________

## Parameter Sensitivity (40% Enriched, GR, Committed)

| Variant                           |  M3  |  M5  | M10  | Align |
| --------------------------------- | :--: | :--: | :--: | :---: |
| Thresholds: Conservative (5/9/13) | 1.45 | 16.8 | 8.29 | 59.3% |
| Thresholds: Balanced (4/7/10)     | 1.50 | 16.8 | 8.77 | 54.8% |
| Thresholds: Aggressive (3/5/8)    | 1.64 | 16.5 | 8.41 | 57.8% |
| No floor (removes floor slot)     | 1.54 | 15.9 | 7.78 | 70.1% |
| Floor from pick 5                 | 1.63 | 15.7 | 7.46 | 59.6% |
| No floor + aggressive (3/5/8)     | 1.69 | 15.2 |  --  | 62.6% |

**Removing the guaranteed floor INCREASES alignment from 55% to 70%.** The floor
amplifies wrong-pair lock-in by reinforcing the leading pair before it is
correctly identified.

______________________________________________________________________

## Draft Traces (40% Enriched, Graduated Realistic)

**Trace 1 (Committed Flash, MISALIGNED):** Player commits to Flash (Ze/Em). Pick
1 hits Ramp(Ze/Ti), seeding (Ze,Ti) as leading pair. Floor slot from pick 3
reinforces this wrong pair. By pick 10, (Ze,Ti)=3.5 vs (Ze,Em)=0.5. Algorithm
locks onto Ramp cards. Final: 6/30 S/A (20%).

**Trace 2 (Signal-reader Storm, MISALIGNED):** Signal reader gravitates to Stone
cards, but pair counter locks onto (St,Em)=Self-Discard instead of
(Em,St)=Storm. Locked slots deliver Self-Discard cards. Final: 5/30 S/A (17%).

**Trace 3 (Committed Warriors, ALIGNED):** Pair counter correctly accumulates
(Ti,Ze). First lock at pick 8, second at 11, third at 15. Post-lock packs
deliver 2-3 S/A. This is the success case hidden in aggregate numbers.

______________________________________________________________________

## Baseline Comparison

Agent 1 projects Pair-Escalation on 40% pool at M3=2.08 (Graduated Realistic).
Escalating Pair Lock achieves M3=1.50 on the same pool. The 0.58 gap is
explained by pair alignment: if alignment were 100%, M3 would be ~2.4
(extrapolated from aligned drafts averaging ~2.2).

______________________________________________________________________

## Self-Assessment

**Escalating Pair Lock fails its primary target at all realistic fitness
levels.** The Round 2 analysis correctly identified pair-matching precision
(~100% from correct pair pool) but critically overestimated the algorithm's
ability to converge on the correct pair. The 55% alignment rate means nearly
half of drafts are functionally broken.

**Root cause:** Eight competing pair counters with noisy early signal. The
guaranteed floor slot worsens the problem by amplifying wrong-pair lock-in.

**What would fix it:** (a) Misalignment detection and self-correction, (b)
player input to seed correct pair (violates zero-decision), or (c) a pair
detection strategy that requires strong evidence before committing. None of
these are achievable with parameter tuning alone.

**Passes:** M1, M2, M4, M6, M7, M9 (6/10). **Fails:** M3, M5, M10.
