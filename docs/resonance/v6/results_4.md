# Agent 4 Results: Dual-Resonance Pool Sculpting

## One-Sentence Algorithm

After each pick (starting pick 3), replace 18 off-resonance cards in the pool
with 12 matching your top resonance and 6 matching your second resonance from a
recycling reserve, keeping the pool at 360.

## Best Variant Scorecard (D: 18/pick, 67/33 split)

| Metric                        | Value     | Target | Pass?      |
| ----------------------------- | --------- | ------ | ---------- |
| Early unique archetypes (1-5) | 5.13      | >= 3   | Yes        |
| Early SA for emerging (1-5)   | 1.04      | \<= 2  | Yes        |
| Late SA for committed (6+)    | **2.01**  | >= 2   | **Barely** |
| Late C/F per pack (6+)        | 0.36      | >= 0.5 | **No**     |
| Convergence pick              | 8.6       | 5-8    | **No**     |
| Deck concentration            | 77.9%     | 60-90% | Yes        |
| Card overlap                  | 6.5%      | < 40%  | Yes        |
| Archetype freq range          | 9.1-15.7% | 5-20%  | Yes        |
| SA stddev (6+)                | 1.05      | >= 0.8 | Yes        |

## Parameter Sensitivity

| Variant         | Replace | Split     | Late SA  | Conv    | C/F      |
| --------------- | ------- | --------- | -------- | ------- | -------- |
| A: Moderate     | 12      | 50/50     | 1.65     | 10.9    | 0.48     |
| B: Aggressive   | 20      | 50/50     | 1.78     | 9.2     | 0.38     |
| C: Escalating   | 12->24  | 50/50     | 1.75     | 9.8     | 0.39     |
| **D: T1-heavy** | **18**  | **67/33** | **2.01** | **8.6** | **0.36** |
| E: Even         | 18      | 50/50     | 1.76     | 9.5     | 0.41     |
| F: No delay     | 18      | 50/50     | 1.94     | 7.4     | 0.35     |

**Key finding:** The 67/33 T1-heavy split is the only variant crossing 2.0 SA.
Even splits cap at ~1.78 regardless of rate. Primary resonance has 50% archetype
precision vs secondary's 25%, so T1 weighting concentrates archetype-correct
cards.

## Per-Archetype Convergence (Variant D)

| Archetype    | Avg Pick | Conv% |     | Archetype | Avg Pick | Conv% |
| ------------ | -------- | ----- | --- | --------- | -------- | ----- |
| Flash        | 8.7      | 100%  |     | Self-Mill | 8.7      | 100%  |
| Blink        | 8.9      | 100%  |     | Sacrifice | 8.7      | 100%  |
| Storm        | 8.7      | 100%  |     | Warriors  | 8.6      | 100%  |
| Self-Discard | 8.6      | 100%  |     | Ramp      | 8.3      | 100%  |

Tight 8.3-8.9 range. No archetype disadvantaged.

## Pool Composition Over Time (Variant D)

| Pick | T1% | T1+T2% |
| ---- | --- | ------ |
| 5    | 28% | 52%    |
| 10   | 44% | 75%    |
| 15   | 53% | 89%    |
| 20   | 54% | 89%    |

Pool saturates at ~89% T1+T2 by pick 15. The T1-heavy split pushes T1 to 53% (vs
47% at 50/50), the critical difference for crossing 2.0.

## Pack Quality Variance (Variant D, picks 6+)

| SA Count | 0    | 1     | 2     | 3     | 4    |
| -------- | ---- | ----- | ----- | ----- | ---- |
| % Packs  | 7.3% | 24.8% | 35.4% | 24.9% | 7.6% |

StdDev 1.05. ~32% bad packs (0-1 SA), ~32% great packs (3-4 SA).

## Draft Traces

**Trace 1 (Committed, Self-Mill):** Stone from picks 1-2, T2=Tide by pick 4.
Pack at pick 6 showed 3 SA cards. Concentration: 86.7%.

**Trace 2 (Power-Chaser, Self-Discard):** Scattered across Stone/Tide/Ember. T2
oscillated through pick 11. Convergence delayed. Concentration: 60.0%.

**Trace 3 (Signal-Reader, Blink):** Ember locked by pick 2, Zephyr T2 by pick 3.
Many Zephyr cards were Flash/Ramp (not Blink), diluting SA. Concentration:
66.7%.

## Compare to Agent 1 Baselines

Pool Sculpting (D) achieves 2.01 SA vs Lane Locking's 2.72 (V3). Convergence 8.6
vs LL's 6.1. Advantage: natural variance (1.05 stddev), archetype balance (6.5%
overlap). Cost: weaker convergence, failed splash (0.36 vs 0.5).

## Failures

1. **Splash deficit (structural):** As the pool fills with T1+T2 cards (~89% by
   pick 15), off-resonance cards are eliminated. Pool sculpting cannot
   concentrate AND maintain splash simultaneously.

2. **Razor-thin 2.0 crossing.** 2.01 SA is fragile. Small parameter changes drop
   below 2.0. This confirms V4's structural finding: probabilistic pool
   manipulation hits a ceiling near 2.0.

3. **T2 instability.** Power chasers' scattered picks cause T2 to oscillate,
   wasting replacements on contradictory resonances.

## Self-Assessment (1-10)

| Goal            | Score | Notes                              |
| --------------- | ----- | ---------------------------------- |
| Simple          | 5     | Reserve adds hidden complexity     |
| No actions      | 10    | Fully automatic                    |
| Not on rails    | 8     | Random draws maintain choice       |
| No forced decks | 8     | 6.5% overlap excellent             |
| Flexible        | 7     | All 8 archetypes equal             |
| Convergent      | 4     | 2.01 barely crosses; pick 8.6 late |
| Splashable      | 3     | 0.36 C/F fails target              |
| Open early      | 9     | 5.13 unique archetypes             |
| Signal reading  | 5     | Pool is invisible to player        |
