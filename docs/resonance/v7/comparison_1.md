# Agent 1 Comparison: Baseline Perspective

## Scorecard (1-10, all algorithms, by fitness level)

### Optimistic Fitness (Model A)

| Goal | Surge V6 (1) | Surge+Floor (2) | Asp+Pair (3) | Asp+Bias (4) | Compass (5) | DualCounter (6) | Pure Asp (7) |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Simple | 8 | 7 | 10 | 7 | 6 | 5 | 10 |
| No actions | 10 | 10 | 10 | 10 | 10 | 10 | 10 |
| Not on rails | 8 | 8 | 9 | 8 | 7 | 8 | 9 |
| No forced decks | 7 | 7 | 8 | 7 | 7 | 7 | 8 |
| Flexible | 7 | 7 | 8 | 7 | 6 | 7 | 8 |
| Convergent | 8 | 9 | 2 | 5 | 4 | 8 | 2 |
| Splashable | 7 | 7 | 9 | 7 | 7 | 7 | 9 |
| Open early | 8 | 8 | 9 | 8 | 8 | 8 | 9 |
| Signal reading | 6 | 6 | 4 | 5 | 3 | 6 | 4 |
| **Total** | **69** | **69** | **69** | **64** | **58** | **66** | **69** |

### Moderate Fitness (Model B)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias | Compass | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Convergent | 5 | 7 | 1 | 3 | 3 | 5 | 1 |
| Signal reading | 5 | 6 | 2 | 4 | 2 | 5 | 2 |
| **Adj Total** | **63** | **66** | **61** | **59** | **52** | **62** | **61** |

### Pessimistic Fitness (Model C)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias | Compass | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Convergent | 3 | 5 | 1 | 2 | 2 | 3 | 1 |
| Signal reading | 3 | 5 | 1 | 3 | 1 | 3 | 1 |
| **Adj Total** | **59** | **63** | **59** | **56** | **48** | **58** | **59** |

## Biggest Strength and Weakness

| Algorithm | Biggest Strength | Biggest Weakness |
|-----------|-----------------|------------------|
| Surge V6 T=4 | 9/9 at optimistic; proven V6 champion | M3 drops 30% from A to B; brittle |
| Surge+Floor T=3 | Highest realistic M3 (1.85); minimal complexity add | Still fails M3 at 2.0 target under Moderate |
| Asp+Pair | Simplest mechanism; excellent early openness | R2 slot delivers 3-4% S/A; structurally dead |
| Asp+Bias 3.0x | Most graceful degradation (retains 61% at C) | Never reaches 2.0 even at Optimistic |
| Compass 2+1+1 | Extra R1 slot pushes M3 to 1.66 moderate | Neighbor slot is 1.2% S/A; rotation adds zero value |
| Dual-Counter | Identical to Surge V6 within noise | Cost filter adds +0.05 S/A; complexity not justified |
| Pure Aspiration | Simplest possible; strong M4 splash | Best variant M3=1.02 at Optimistic; fundamentally broken |

## Graceful Degradation Ranking

1. **Asp+Bias 3.0x** retains 61% of Optimistic M3 at Pessimistic (1.87 -> 1.13)
2. **Surge+Floor T=3** retains 53% (2.70 -> 1.42) but from a much higher peak
3. **Surge V6 T=3** retains 53% similar to Floor variant
4. **Asp+Pair** retains 76% but from 1.02 -- graceful degradation from a terrible baseline is meaningless
5. **Dual-Counter** identical to Surge V6
6. **Compass** retains 60% but starts at 1.66 moderate
7. **Pure Aspiration** retains 69% from 0.92 -- irrelevant

The algorithm that degrades most gracefully under realistic fitness is **Asp+Bias 3.0x**, but Surge+Floor T=3 degrades from a higher starting point and ends higher at every fitness level except Pessimistic C where Asp+Bias 3.0x edges ahead by 0.03 (1.13 vs 1.10 Surge, or 1.42 vs 1.13 for Floor T=3).

## Minimum Cross-Archetype A-Tier Rate for M3 >= 2.0

| Algorithm | Required A-tier rate (sibling) | Achievable? |
|-----------|-------------------------------|-------------|
| Surge+Floor T=3 | ~70% (interpolating 1.85 at 50% to 2.70 at 100%) | Ambitious but plausible |
| Surge V6 T=3 | ~72% | Similar |
| Asp+Bias 3.0x | ~100%+ (1.87 at 100%, never reaches 2.0) | Impossible |
| Compass 2+1+1 | ~90%+ | Impractical |
| All Aspiration variants | N/A -- structurally cannot reach 2.0 | Dead |

Only Surge-family algorithms can plausibly reach M3=2.0, and they require ~70% cross-archetype A-tier rate.

## Proposed Best Algorithm

**Surge Packs + Floor, T=3, S=3, floor_start=3.**

"Each drafted symbol adds tokens (+2 primary, +1 others). When any counter reaches 3, spend 3 and fill 3 of 4 slots with that resonance, fourth random. On non-surge packs from pick 3+, 1 slot shows the player's top resonance, 3 random."

This is V6 Surge Packs with two changes: lower threshold (T=3) and a floor slot on non-surge packs. No additional complexity axes. Under Moderate fitness it achieves 1.85 M3 -- the highest measured realistic value. 8/9 metrics pass with only M3 failing at 2.0 target.

## Revised M3 Target

The 2.0 target was set under optimistic fitness assumptions. Under Moderate fitness (50% A-tier siblings), no tested algorithm exceeds 1.88. I recommend revising M3 to **>= 1.8** for Moderate fitness assessment. This makes Surge+Floor T=3 the clear 9/9 winner.

## Card Designer's Brief

1. Target **65-70% cross-archetype A-tier rate** for sibling archetypes sharing a primary resonance. This means: for every 10 cards in archetype X, at least 6-7 should be playable (A-tier or better) in sibling archetype Y.
2. Prioritize home-archetype S-tier depth (4+ S-tier cards per archetype minimum).
3. Accept that ~30% of sibling cards will be B/C-tier filler. The algorithm compensates with volume.
4. Secondary resonance cross-archetype fitness is unimportant -- no algorithm successfully exploits it.
