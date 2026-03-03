# Results Agent 6: Guaranteed Floor + Pair Escalation (GF+PE)

**Algorithm (one sentence):** Every post-commitment pack gets at least 1
pair-matched slot (guaranteed floor), with permanent escalation to 2 slots at
counter threshold T1 and 3 slots at T2; quality never decreases.

## Set Design Specification (Enriched 40% Pool as Implemented)

### 1. Pool Breakdown by Archetype

| Archetype |  Total  | Home-Only | Cross-Archetype (dual/tri) | Generic |
| --------- | :-----: | :-------: | :------------------------: | :-----: |
| Each of 8 |   40    |     4     |   36 (18 dual + 18 tri)    |   --    |
| Generic   |   40    |    --     |             --             |   40    |
| **Total** | **360** |  **32**   |          **288**           | **40**  |

### 2. Symbol Distribution

| Symbol Count | Cards |   %   | Example               |
| :----------: | :---: | :---: | --------------------- |
| 0 (generic)  |  40   | 11.1% | No resonance          |
|   1 symbol   |  32   | 8.9%  | (Tide)                |
| 2 different  |  144  | 40.0% | (Tide, Zephyr)        |
|  3 symbols   |  144  | 40.0% | (Tide, Zephyr, Ember) |

### 3. Dual-Resonance Breakdown

| Type                | Cards |   %   | Filtering                        |
| ------------------- | :---: | :---: | -------------------------------- |
| Generic (0 symbols) |  40   | 11.1% | Not filtered                     |
| Single-resonance    |  32   | 8.9%  | R1 filter only                   |
| Dual-resonance      |  144  | 40.0% | Pair-matchable at ~80% precision |
| Tri-resonance       |  144  | 40.0% | Pair-matchable; R3 adds data     |

### 4. Per-Resonance Pool Sizes

Each resonance as R1: 80 cards (2 archetypes x 40). Each archetype's (R1,R2)
pair-filtered pool: 36 cards (18 dual + 18 tri from home archetype only; sibling
archetype's cards also match the pair filter, adding ~36 more for ~72 total
pair-matchable). Sustains 3 draws/pack over 25 packs.

### 5. Cross-Archetype Requirements

Per the enriched pool: 90% of each archetype's cards carry the archetype pair.
The fitness model determines how many of the sibling's pair-matched cards are
actually A-tier. Under Graduated Realistic: Warriors/Sacrifice 50%,
Self-Disc./Self-Mill 40%, Blink/Storm 30%, Flash/Ramp 25%.

### 6. Card Designer Guidance

Increase dual+ resonance cards from 54 (V7) to 288. Every non-generic card needs
2+ symbols. The designer must ask "which archetype pair does this card serve?"
and assign primary and secondary resonance accordingly.

______________________________________________________________________

## Scorecard: GF+PE (Committed Strategy, T1=4, T2=8, Jitter=20%)

### Enriched 40% Pool

| Metric                       | Target | Optimistic | Grad. Real. | Pessimistic |  Hostile  |
| ---------------------------- | ------ | :--------: | :---------: | :---------: | :-------: |
| M1 (archetypes/pack, early)  | >= 3   |    1.00    |    0.71     |    0.63     |   0.54    |
| M2 (S/A for emerging, early) | \<= 2  |    1.52    |    1.00     |    0.89     |   0.77    |
| M3 (S/A per pack, 6+)        | >= 2.0 |  **2.57**  |    1.72     |    1.58     |   1.34    |
| M4 (off-archetype/pack)      | >= 0.5 |  **1.37**  |  **1.78**   |  **1.92**   | **2.03**  |
| M5 (convergence pick)        | 5-8    |  **4.8**   |   **7.5**   |     8.4     |   12.2    |
| M6 (deck concentration)      | 60-90% |  **92%**   |   **76%**   |   **71%**   |    59%    |
| M7 (run-to-run overlap)      | < 40%  |  **10%**   |   **8%**    |   **7%**    |  **8%**   |
| M8 (arch freq range)         | 5-20%  | **12.5%**  |  **12.5%**  |  **12.5%**  | **12.5%** |
| M9 (stddev S/A)              | >= 0.8 |    0.72    |    0.74     |    0.71     |   0.62    |
| M10 (max consec < 1.5)       | \<= 2  |    2.49    |    6.34     |    7.60     |   11.31   |

### V7 Standard Pool (15% dual-res)

| Metric | Optimistic | Grad. Real. | Pessimistic | Hostile |
| ------ | :--------: | :---------: | :---------: | :-----: |
| M3     |  **2.62**  |    1.83     |    1.67     |  1.46   |
| M5     |  **4.6**   |   **6.7**   |     9.2     |  11.6   |
| M6     |  **93%**   |   **82%**   |   **74%**   | **65%** |
| M9     |    0.72    |    0.74     |    0.69     |  0.62   |
| M10    |    2.03    |    5.11     |    7.72     |  10.28  |

______________________________________________________________________

## Pack Quality Distribution (Enriched 40%, Committed, Picks 6+)

| Percentile | Optimistic | Grad. Real. | Pessimistic | Hostile |
| :--------: | :--------: | :---------: | :---------: | :-----: |
|    10th    |    1.0     |     0.0     |     0.0     |   0.0   |
|    25th    |    2.0     |     1.0     |     0.0     |   0.0   |
|    50th    |    3.0     |     2.0     |     2.0     |   1.0   |
|    75th    |    3.0     |     3.0     |     3.0     |   3.0   |
|    90th    |    4.0     |     3.0     |     3.0     |   3.0   |

The distribution is wide with a significant tail at 0 under realistic fitness
models. When pair-matched cards drawn from the pool happen to be sibling rather
than home archetype, and the fitness roll fails, the "pair-matched" slot
delivers a B/C card despite the structural guarantee. The floor guarantees a
pair-matched *draw*, not a pair-matched *hit*.

______________________________________________________________________

## Consecutive Bad Pack Analysis (Enriched 40%, Committed)

| Fitness             | Avg Streak (< 1.5 S/A) | Worst Streak |
| ------------------- | :--------------------: | :----------: |
| Optimistic          |          2.18          |      25      |
| Graduated Realistic |          4.57          |      25      |
| Pessimistic         |          5.45          |      25      |
| Hostile             |          9.54          |      25      |

The worst-case streak of 25 appearing across all fitness models indicates that
some archetypes (particularly those with low sibling fitness like Flash/Ramp)
can experience extended runs where the pair-matched pool delivers mostly
B/C-tier sibling cards. This is a structural failure of the "guaranteed floor"
concept: the floor guarantees pair-matched *draws*, but pair-matching only
concentrates home-archetype cards at ~50% when the pair pool is evenly split
between home and sibling archetype cards.

______________________________________________________________________

## Per-Archetype Convergence Table (Enriched 40%, Grad. Realistic)

| Archetype                    | Pair Fitness |  M3  | M5  | M6  |  M9  | M10  |
| ---------------------------- | :----------: | :--: | :-: | :-: | :--: | :--: |
| Flash/Tempo (Ze/Em)          |     25%      | 2.03 | 7.0 | 80% | 0.71 | 5.14 |
| Blink/Flicker (Em/Ze)        |     30%      | 2.07 | 6.5 | 82% | 0.73 | 4.59 |
| Storm/Spellslinger (Em/St)   |     30%      | 1.57 | 7.3 | 77% | 0.81 | 6.08 |
| Self-Discard (St/Em)         |     40%      | 2.11 | 6.4 | 83% | 0.70 | 4.66 |
| Self-Mill/Reanimator (St/Ti) |     40%      | 1.19 | 8.9 | 68% | 0.76 | 8.43 |
| Sacrifice/Abandon (Ti/St)    |     50%      | 2.31 | 5.7 | 87% | 0.74 | 3.40 |
| Warriors/Midrange (Ti/Ze)    |     50%      | 1.66 | 6.7 | 79% | 0.81 | 5.77 |
| Ramp/Spirit Animals (Ze/Ti)  |     25%      | 1.13 | 9.7 | 62% | 0.73 | 9.41 |

**Critical finding:** Per-archetype M3 does not track cleanly with pair fitness.
Ramp (25% fitness) scores 1.13, but Flash (also 25%) scores 2.03. Self-Mill
(40%) scores 1.19, while Self-Discard (also 40%) scores 2.11. The asymmetry
arises because the pair-counter scoring system (+2 for R1 match, +1 for R2)
interacts differently with how each archetype's cards distribute across the pair
pool. Some archetypes accumulate pair counters faster than others based on the
pool composition details.

**Worst archetype:** Ramp/Spirit Animals at M3=1.13 under Graduated Realistic.
This is catastrophically below the 2.0 target and represents a fundamental
fairness problem.

______________________________________________________________________

## Fitness Degradation Curve (Enriched 40%, Committed, Overall Average)

| Fitness                   | Avg A-tier |  M3  | M10 avg | Worst Streak |
| ------------------------- | :--------: | :--: | :-----: | :----------: |
| Optimistic (100%)         |    100%    | 2.57 |  2.80   |      25      |
| Graduated Realistic (36%) |    36%     | 1.72 |  6.34   |      25      |
| Pessimistic (21%)         |    21%     | 1.58 |  7.60   |      25      |
| Hostile (8%)              |     8%     | 1.34 |  11.31  |      25      |

Degradation is steep: M3 drops by 0.85 (33%) from Optimistic to Graduated
Realistic. The algorithm loses approximately 0.20 M3 per 15 percentage points of
fitness reduction.

______________________________________________________________________

## Baseline Comparison (Enriched 40%, Committed)

| Metric            | GF+PE | Surge+Floor (T=3) |     Delta     |
| ----------------- | :---: | :---------------: | :-----------: |
| M3 (Optimistic)   | 2.57  |       2.16        |     +0.41     |
| M3 (Grad. Real.)  | 1.72  |       1.50        |     +0.22     |
| M3 (Pessimistic)  | 1.58  |       1.35        |     +0.23     |
| M3 (Hostile)      | 1.34  |       1.18        |     +0.16     |
| M10 (Grad. Real.) | 6.34  |       4.17        | -2.17 (worse) |
| M9 (Grad. Real.)  | 0.74  |       0.92        | -0.18 (worse) |

GF+PE outperforms Surge+Floor on M3 by +0.16 to +0.41 across all fitness levels.
However, GF+PE **performs worse on M10** (consecutive bad packs) than
Surge+Floor, contradicting the design intent. The cause: Surge+Floor's periodic
surges guarantee high-quality packs at regular intervals, breaking bad streaks.
GF+PE's permanent escalation means that once a bad archetype (Ramp, Self-Mill)
reaches Level 3, it stays at Level 3 but continues delivering pair-matched cards
that fail the fitness check, producing sustained mediocrity rather than
alternating peaks and valleys.

______________________________________________________________________

## Parameter Sensitivity (Enriched 40%, Grad. Realistic, Committed)

**Threshold sweep:** T1 and T2 have minimal impact on M3 (range 1.61-1.77 across
all tested values). The dominant factor is fitness, not threshold timing. Best
configuration: T1=4, T2=9 (M3=1.77).

**Jitter sweep (T1=4, T2=8):** Reducing jitter to 0% raises M3 to 1.92 but drops
M9 to 0.58 (fails target). Jitter at 10% achieves M3=1.92, M9=0.67 (still below
0.8). The M9 target of >= 0.8 requires ~25% jitter, which costs ~0.10 M3.

______________________________________________________________________

## Self-Assessment

**GF+PE does not achieve M3 >= 2.0 under Graduated Realistic fitness** for the
committed player strategy (overall average 1.72, enriched pool). It achieves
2.0+ only under Optimistic fitness.

**The "guaranteed floor" fails its design intent.** The floor guarantees
pair-matched draws, not pair-matched S/A cards. Under realistic fitness,
pair-matched sibling cards frequently fail the A-tier check, producing B/C cards
in "guaranteed" slots. This creates worse M10 than Surge+Floor's periodic reset
pattern.

**Per-archetype variance is the fatal flaw.** Ramp (M3=1.13) and Self-Mill
(M3=1.19) experience a fundamentally broken draft under Graduated Realistic,
while Sacrifice (M3=2.31) and Self-Discard (M3=2.11) thrive. The 2:1 ratio
between best and worst archetype represents an unacceptable player experience
gap.

**Signal-reader strategy performs extremely well** (M3=2.64 under Graduated
Realistic) because it naturally converges on the pair with the best available
cards, effectively selecting high-fitness archetypes. This suggests the
algorithm works well when players can adapt but poorly when players commit to a
specific archetype.

**What would push GF+PE to 2.0?** Either (a) per-pair fitness must reach ~45%
average (all pairs at Moderate or above), or (b) the pair pool must more
strongly concentrate home-archetype cards (currently ~50% home vs sibling; if it
were 80% home, precision would be ~90% regardless of fitness). Option (b)
requires pool redesign where each archetype's pair-filtered subpool is dominated
by home cards.
