# Agent 6 Comparison: Dual-Counter Postmortem Perspective

## Scorecard (1-10, all algorithms, by fitness level)

### Optimistic Fitness (Model A)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias 3x | Compass | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Simple | 8 | 7 | 10 | 7 | 6 | 4 | 10 |
| No actions | 10 | 10 | 10 | 10 | 10 | 10 | 10 |
| Not on rails | 8 | 8 | 9 | 8 | 7 | 8 | 9 |
| No forced decks | 7 | 7 | 8 | 7 | 7 | 7 | 8 |
| Flexible | 7 | 7 | 8 | 7 | 6 | 7 | 8 |
| Convergent | 8 | 9 | 2 | 7 | 5 | 8 | 2 |
| Splashable | 7 | 7 | 9 | 7 | 7 | 7 | 9 |
| Open early | 8 | 8 | 9 | 8 | 8 | 7 | 9 |
| Signal reading | 6 | 7 | 3 | 5 | 3 | 6 | 3 |
| **Total** | **69** | **70** | **68** | **66** | **59** | **64** | **68** |

### Moderate Fitness (Model B)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias 3x | Compass | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Convergent | 4 | 7 | 1 | 4 | 3 | 4 | 1 |
| Signal reading | 4 | 6 | 1 | 4 | 2 | 4 | 1 |
| **Adj Total** | **63** | **66** | **58** | **62** | **53** | **58** | **58** |

### Pessimistic Fitness (Model C)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias 3x | Compass | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Convergent | 3 | 5 | 1 | 3 | 2 | 3 | 1 |
| Signal reading | 3 | 4 | 1 | 3 | 1 | 3 | 1 |
| **Adj Total** | **59** | **62** | **56** | **59** | **46** | **54** | **56** |

Note: I rate Dual-Counter lower on "Simple" (4 vs Surge's 8) because the cost-tracking mechanism adds a running average computation, a cost window filter, and a fallback widening rule -- all for +0.05 M3. This is the worst complexity-to-benefit ratio of any V7 algorithm.

## Biggest Strength and Weakness

| Algorithm | Biggest Strength | Biggest Weakness |
|-----------|-----------------|------------------|
| Surge V6 T=4 | Simplicity and proof of concept from V6 | T=4 is the wrong threshold for realistic fitness |
| Surge+Floor T=3 | Highest M3 at moderate (1.85); floor fixes M5 | Still 0.15 below M3=2.0 |
| Asp+Pair | Falsifies dual-resonance targeting conclusively | Gate paradox; R2 at 3% S/A |
| Asp+Bias 3.0x | Bias validated; graceful degradation at C | 0.46 M3 gap to Floor at moderate |
| Compass 2+1+1 | Extra R1 slot carries the variant | Neighbor rotation is zero-value; M9 fails everywhere |
| Dual-Counter | Cleanly isolates cost as a non-signal for archetype | +0.05 M3 for significant complexity cost |
| Pure Aspiration | Proves 2-slot minimum is broken | Dead; M3 never exceeds 1.02 |

## Graceful Degradation: The Cost-Filtering Perspective

My investigation specifically asked: "Does cost filtering improve degradation under worse fitness?" The answer is no. The degradation curves are:

| Algorithm | M3(A) | M3(B) | M3(C) | Total drop |
|-----------|:-----:|:-----:|:-----:|:----------:|
| DualCounter T=4 | 2.02 | 1.41 | 1.13 | -0.89 |
| Surge V6 T=4 | 2.03 | 1.43 | 1.09 | -0.94 |
| **Difference** | **-0.01** | **-0.02** | **+0.04** | **+0.05** |

The cost filter "saves" 0.04 M3 at Pessimistic fitness. This is within noise. Cost filtering does not provide meaningful resilience because:

1. **Archetype cost profiles overlap.** Sacrifice (avg 3.0) and Warriors (avg 3.7) differ by only 0.7 energy. A +/-1 cost window captures both.
2. **The running average is noisy.** Early picks can skew the average by 0.5+ energy, making the filter target the wrong cost band.
3. **Cost is orthogonal to fitness.** Fitness is about whether a sibling card is A-tier for the player's archetype. This depends on mechanical synergy, not mana cost.

**The algorithm that degrades most gracefully is Surge+Floor T=3**, which maintains 1.42 M3 even at Pessimistic -- 0.29 above DualCounter's 1.13 at Pessimistic. The floor mechanism provides a structural minimum that cost filtering cannot match.

## Minimum Cross-Archetype A-Tier Rate

| Algorithm | Rate for M3=1.8 | Rate for M3=2.0 |
|-----------|:---:|:---:|
| Surge+Floor T=3 | 50% | ~65% |
| DualCounter T=3 | ~49% | ~63% |
| Surge V6 T=3 | ~48% | ~63% |
| Asp+Bias 3.0x | ~80% | Impossible |

DualCounter T=3 reaches 1.88 at 50% A-tier (Model B), so it technically needs marginally less than 50% for 1.8. But the difference from Surge V6 T=3 is within measurement error. The cost filter does not meaningfully change the card-design requirement.

## Why Cost Filtering Should Be Abandoned

My honest postmortem:

1. **The hypothesis was reasonable.** Archetypes sharing a primary resonance often differ in cost profile (Flash=low vs Ramp=high). Cost filtering could theoretically disambiguate.

2. **Reality killed it.** Not all sibling pairs have clear cost separation. Warriors/Sacrifice overlap at 3.0-3.7. Storm/Blink overlap at 2.5-3.5. Only Flash/Ramp (2.5 vs 4.8) has clean separation, and one good pair is not enough.

3. **The fallback mechanism defeats the filter.** When the cost window is too narrow, the algorithm widens to +/-2, then drops the filter entirely. This means the filter is either too narrow (missing good cards) or too wide (not filtering anything). There is no sweet spot.

4. **Threshold T=3 is the actual lever.** At T=3, DualCounter achieves 1.88 M3 -- identical to standard Surge at T=3. The improvement comes from surge frequency, not cost precision.

## Proposed Best Algorithm

**Surge Packs + Floor (T=3, S=3, floor_start=3).** I abandon cost filtering and endorse the consensus.

The lesson from Dual-Counter is that secondary signals (cost, secondary resonance, neighbor position) all fail to provide meaningful archetype disambiguation beyond what primary resonance gives. The only lever that works is **more primary-resonance cards per pack** (Surge's 3/4 slots) combined with **more frequent surges** (T=3) and **non-surge baseline quality** (floor slot).

## Revised M3 Target

**M3 >= 1.8 under Moderate fitness.** At this target, Surge+Floor T=3 achieves 9/9. The 2.0 target should be retained only as a card-design quality metric: "If the card designer achieves 65%+ sibling A-tier, the algorithm will deliver M3 >= 2.0."

## Card Designer's Brief

1. **Do not rely on cost profiles for archetype identity.** Cost-based disambiguation failed in simulation. Archetypes must be distinguished by mechanical synergy, not mana cost.
2. **Sibling A-tier rate of 50%+ is the minimum.** Below this, no algorithm passes M3=1.8.
3. **65%+ sibling A-tier enables M3=2.0.** This is the aspirational target.
4. **Design each resonance pair as a "team."** The two archetypes sharing a primary resonance should have complementary mechanics where 60-70% of cards work well in both decks.
5. **If a card is narrow, it must be powerful.** Cards that only work in one archetype should be S-tier in that archetype to justify their B/C-tier rating in the sibling. The algorithm will deliver them; the player gets to discover "this is a signal card for archetype X."
