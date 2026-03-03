# Results: Agent 2 — Continuous Surge

## Algorithm

Each drafted card's resonance symbols accumulate counters that set a per-slot
probability of pair-matched targeting, with a 1-slot floor from pick 3,
producing a smooth unimodal quality distribution instead of Surge+Floor's
bimodal alternation.

## Set Design Specification

**Pool Breakdown:** 360 cards total. Each of 8 archetypes: 40 cards (24
single-res + 16 dual-res). Plus 40 generic (no symbols). Dual-res cards carry
(primary, secondary) resonance of their archetype.

**Symbol Distribution:** 40 generic (11%), 192 single-res (53%), 128 dual-res
(36%).

**Per-Resonance Pools:** Each resonance's R1 pool contains 112 cards (50%
home-archetype). Each archetype's pair-matched pool contains 16 cards (~100%
home).

**Cross-Archetype Requirements:** Pair-matched slots bypass sibling fitness;
only R1-fallback and random slots depend on it. Per-pair targets:
Warriors/Sacrifice 50%, SelfDiscard/SelfMill 40%, Blink/Storm 30%, Flash/Ramp
25%.

**Card Designer Guidance:** Create 128 dual-resonance cards (up from V7's 54).
The secondary symbol is a *filtering tag*, not a fitness promise. A Warriors
card with (Tide, Zephyr) need not be playable in Ramp.

______________________________________________________________________

## Scorecard: All Conditions (archetype-committed, 1000 drafts)

| Pool / Fitness           |  M1  |  M2  |  **M3**  |  M4  | M5  | M6  |  M9  | M10avg |
| ------------------------ | :--: | :--: | :------: | :--: | :-: | :-: | :--: | :----: |
| V7 Std / Optimistic      | 4.51 | 2.38 | **3.12** | 0.78 | 1.5 | 93% | 0.75 |  1.3   |
| V7 Std / Grad Realistic  | 3.49 | 2.21 | **2.49** | 1.21 | 3.1 | 86% | 0.76 |  4.0   |
| V7 Std / Pessimistic     | 3.18 | 2.14 | **2.30** | 1.37 | 4.3 | 80% | 0.74 |  5.8   |
| V7 Std / Hostile         | 2.88 | 2.07 | **2.25** | 1.40 | 5.4 | 75% | 0.73 |  6.8   |
| 40% Enr / Optimistic     | 4.50 | 2.38 | **3.10** | 0.79 | 1.7 | 93% | 0.76 |  1.3   |
| 40% Enr / Grad Realistic | 3.47 | 2.23 | **2.48** | 1.23 | 3.1 | 85% | 0.78 |  3.8   |
| 40% Enr / Pessimistic    | 3.18 | 2.14 | **2.43** | 1.27 | 3.9 | 82% | 0.76 |  4.5   |
| 40% Enr / Hostile        | 2.87 | 2.06 | **2.25** | 1.42 | 5.8 | 74% | 0.72 |  6.9   |

M7=3.2% (pass). M8 range=9-18% (pass). M10 worst-case=25 at all fitness levels.

## Pack Quality Distribution (picks 6+, 40% Enriched)

| Fitness        | p10 | p25 | p50 | p75 | p90 |
| -------------- | :-: | :-: | :-: | :-: | :-: |
| Optimistic     |  2  |  3  |  3  |  4  |  4  |
| Grad Realistic |  0  |  2  |  3  |  4  |  4  |
| Pessimistic    |  0  |  1  |  3  |  4  |  4  |
| Hostile        |  0  |  1  |  3  |  4  |  4  |

Distribution is unimodal with median 3 at all fitness levels. However, 10% of
packs have 0 S/A under Graduated Realistic -- probabilistic targeting means
occasional packs where all Bernoulli trials miss.

## Consecutive Bad Pack Analysis

| Pool / Fitness           | Avg Max Consecutive (SA < 1.5) | Worst |
| ------------------------ | :----------------------------: | :---: |
| 40% Enr / Grad Realistic |              3.8               |  25   |
| 40% Enr / Pessimistic    |              4.5               |  25   |
| 40% Enr / Hostile        |              6.9               |  25   |

**M10 fails.** Worst-case of 25 means rare drafts where the player never
converges. Average of 3.8 exceeds target of 2.

## Fitness Degradation Curve (40% Enriched)

| Fitness        | Avg Sibling A-Tier |  M3  | Delta |
| -------------- | :----------------: | :--: | :---: |
| Optimistic     |        100%        | 3.10 |  --   |
| Grad Realistic |        36%         | 2.48 | -20%  |
| Pessimistic    |        21%         | 2.43 | -22%  |
| Hostile        |         8%         | 2.25 | -27%  |

Grad Realistic to Pessimistic drops only 0.05, demonstrating pair-matching's
fitness insulation.

## Per-Archetype Convergence (40% Enriched, Graduated Realistic)

| Archetype   | Sibling A-Tier |  M3  |  M5  | M6  |  M9  |
| ----------- | :------------: | :--: | :--: | :-: | :--: |
| Warriors    |      50%       | 2.60 | 6.3  | 87% | 0.76 |
| Sacrifice   |      50%       | 2.56 | 6.6  | 87% | 0.78 |
| Storm       |      30%       | 2.57 | 7.5  | 84% | 0.74 |
| SelfDiscard |      40%       | 2.38 | 8.4  | 80% | 0.73 |
| Flash       |      25%       | 2.37 | 9.6  | 78% | 0.71 |
| SelfMill    |      40%       | 1.80 | 12.7 | 71% | 0.73 |
| Blink       |      30%       | 1.94 | 12.0 | 74% | 0.74 |
| Ramp        |      25%       | 1.55 | 17.5 | 64% | 0.67 |

**Worst: Ramp at 1.55.** Three archetypes (Ramp, SelfMill, Blink) fall below
2.0. The Warriors-to-Ramp gap is 68%.

## Parameter Sensitivity (40% Enriched, Graduated Realistic)

|  K  | P_max | Decay |  M3  |  M9  | M10avg |
| :-: | :---: | :---: | :--: | :--: | :----: |
|  4  | 0.85  | 0.50  | 2.96 | 0.66 |  2.1   |
|  4  | 0.75  | 0.30  | 2.66 | 0.78 |  3.1   |
|  6  | 0.75  | 0.50  | 2.54 | 0.78 |  3.4   |
|  8  | 0.85  | 0.70  | 2.83 | 0.78 |  3.0   |
|  8  | 0.65  | 0.70  | 2.35 | 0.89 |  3.6   |

M3-M9 tradeoff: P_max=0.85 maximizes M3 but kills M9. P_max=0.65 preserves
variance but drops M3.

## Draft Traces (40% Enriched, Graduated Realistic)

**Early Committer (SelfDiscard):** Counters reach P=0.75 by pick 5.
Post-convergence SA: 2,4,3,3,1,2,4,2,3,4,4,2,3,3,3,2,2,4,4,2,4,4,3,1. Mode=3,
range=[1,4]. M3=2.80.

**Signal Reader (Warriors):** Locks onto Tide/Zephyr by pick 2. SA from pick 6:
3,3,4,3,4,4,3,4,3,2,4,3,2,2,4,3,2,3,3,2,4,3,3,2,3. M3=3.04. Concentration=97.6%.

**Power Chaser (Warriors):** Naturally gravitates toward Warriors via high-power
Tide cards. Occasionally picks off-archetype (Flash, Blink) for power. M3=3.00.
Concentration=62.9%.

## Comparison to Agent 1 Baseline

Agent 1 projected Pair-Escalation on 40% at M3=2.08 (Grad Realistic). Continuous
Surge achieves M3=2.48 -- a +0.40 advantage. Actual Agent 1 simulation results
not yet available.

## Self-Assessment

**Strengths:** M3 exceeds 2.0 at all fitness levels (minimum 2.25 Hostile).
Strong fitness insulation (Grad Realistic to Pessimistic: -0.05). Unimodal
distribution. Very low card overlap (3.2%). Works on V7 Standard pool too
(M3=2.49).

**Weaknesses:** M10 fails (3.8 avg, 25 worst) -- probabilistic targeting
produces occasional droughts. M9 marginal (0.78). M5 too early (3.1).
Per-archetype disparity: Ramp=1.55, Blink=1.94, SelfMill=1.80 all below 2.0.

**Honest assessment:** Continuous Surge achieves its design goal of eliminating
bimodal distribution and delivers strong aggregate M3, but trades Surge+Floor's
predictable alternation for unpredictable probabilistic droughts. The M10
failure is structural: P=0.75 means 0.4% chance of all-random packs, and runs of
2-3 such packs occur regularly. The per-archetype gap (Warriors 2.60 vs Ramp
1.55) is the most concerning finding -- low-overlap archetypes suffer
substantially despite pair-matching. Increasing the floor to 2 slots would fix
M10 but push M5 earlier and reduce M9.
