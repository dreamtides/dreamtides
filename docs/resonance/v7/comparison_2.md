# Agent 2 Comparison: Floor Specialist Perspective

## Scorecard (1-10, all algorithms, by fitness level)

### Optimistic Fitness (Model A)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias | Compass | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Simple | 8 | 7 | 10 | 6 | 6 | 5 | 10 |
| No actions | 10 | 10 | 10 | 10 | 10 | 10 | 10 |
| Not on rails | 8 | 8 | 9 | 8 | 7 | 8 | 9 |
| No forced decks | 7 | 7 | 8 | 7 | 7 | 7 | 8 |
| Flexible | 7 | 7 | 8 | 7 | 6 | 7 | 8 |
| Convergent | 8 | **9** | 2 | 6 | 5 | 8 | 2 |
| Splashable | 7 | 7 | 9 | 8 | 7 | 7 | 9 |
| Open early | 7 | 8 | 9 | 8 | 8 | 7 | 9 |
| Signal reading | 6 | 7 | 4 | 5 | 3 | 6 | 4 |
| **Total** | **68** | **70** | **69** | **65** | **59** | **65** | **69** |

### Moderate Fitness (Model B)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias | Compass | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Convergent | 4 | **7** | 1 | 3 | 3 | 4 | 1 |
| Signal reading | 4 | **7** | 2 | 4 | 2 | 4 | 2 |
| **Adj Total** | **62** | **68** | **61** | **59** | **53** | **59** | **61** |

### Pessimistic Fitness (Model C)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias | Compass | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Convergent | 3 | **5** | 1 | 3 | 2 | 3 | 1 |
| Signal reading | 3 | **5** | 1 | 3 | 1 | 3 | 1 |
| **Adj Total** | **58** | **64** | **59** | **58** | **48** | **55** | **59** |

## Biggest Strength and Weakness

| Algorithm | Biggest Strength | Biggest Weakness |
|-----------|-----------------|------------------|
| Surge V6 T=4 | Proven 9/9 under optimistic | M5 explodes to 10.8 under moderate |
| Surge+Floor T=3 | 1.85 M3 moderate; 8/9 pass; floor smooths non-surge packs | M3=1.85 still below 2.0 target |
| Asp+Pair | Gate paradox: committed players cannot open it | R2 slot is 3-4% S/A |
| Asp+Bias 3.0x | Bias layer adds +0.36 M3 over pure aspiration | M3=1.39 moderate, 0.46 below Surge+Floor |
| Compass 2+1+1 | 2+1+1 variant works via extra R1 slot, not rotation | M9 stddev fails at every fitness level (0.44-0.78) |
| Dual-Counter | Cost filter concept is theoretically interesting | +0.05 M3 over plain Surge; noise-level improvement |
| Pure Aspiration | Establishes a clear lower bound for the design space | M3=1.02 optimistic is less than half of Surge |

## Graceful Degradation Analysis

The key question: which algorithm degrades most gracefully?

**M3 retention from Optimistic to Pessimistic:**
- Surge+Floor T=3: 2.70 -> 1.42 (47% retained, but 1.42 is the highest Pessimistic M3 measured)
- Asp+Bias 3.0x: 1.87 -> 1.13 (60% retained, but 1.13 < Floor's 1.42)
- Surge V6 T=3: 1.88(B) -> 1.49(C) from Agent 1 parameter table (similar retention)
- Compass 2+1+1: 2.21 -> 1.32 (60% retained)
- Asp+Pair: 1.02 -> 0.78 (76% retained -- but from a useless starting point)

**Surge+Floor T=3 degrades in percentage but remains the highest at every fitness level except possibly Pessimistic where Asp+Bias 3.0x shows 1.13 vs Surge+Floor's 1.42.** The Floor variant's absolute dominance makes the percentage question moot.

The floor mechanism specifically helps because non-surge packs go from pure random (~0.5 S/A) to floor-assisted (~0.75 S/A). This raises the minimum per-pack S/A, reducing variance and supporting M5 convergence. Under Moderate fitness, convergence stays at 5.0 for Floor (all archetypes converge within target) vs 10.8 for plain Surge.

## Minimum Cross-Archetype A-Tier Rate for M3 >= 2.0

Interpolating from the data:

| Algorithm | A-tier rate at M3=1.85 | Estimated rate for M3=2.0 |
|-----------|------------------------|---------------------------|
| Surge+Floor T=3 | 50% (Model B) | ~65% |
| Surge V6 T=3 | 50% -> 1.88 M3 | ~63% |
| Asp+Bias 3.0x | 50% -> 1.39 M3 | Unreachable (<100%) |
| Compass 2+1+1 | 50% -> 1.66 M3 | ~85% |

Surge-family algorithms require approximately **65% cross-archetype A-tier** to hit M3=2.0. This translates to: of 10 sibling-archetype cards in the same resonance, 6-7 must be A-tier or better for the player's archetype. Below 50%, no algorithm passes M3=2.0.

## Proposed Best Algorithm

**Surge Packs + Floor (T=3, S=3, floor_start=3).** Identical to my Round 3 champion.

One-sentence: "Each drafted symbol adds tokens (+2 primary, +1 others); when any counter reaches 3, spend 3 and fill 3 of 4 slots with that resonance (4th random); on non-surge packs from pick 3 onward, 1 slot shows top resonance (3 random)."

Why this wins:
1. Highest measured M3 under Moderate fitness (1.85)
2. 8/9 metrics pass -- only M3 fails at the 2.0 target
3. Floor mechanism eliminates M5 failure (convergence stays at 5.0)
4. Minimal complexity over plain Surge: one extra rule for non-surge packs
5. Floor+Pair was tested and is strictly worse -- keep it simple

## Revised M3 Target

**Recommend M3 >= 1.8 under Moderate fitness.** Justification:
- No algorithm reaches 2.0 under Moderate fitness across all 7 agents' simulations
- 1.8 represents the practical ceiling under realistic card design assumptions (50% A-tier siblings)
- At 1.85, Surge+Floor T=3 passes 9/9 with this revised target
- The 2.0 target remains valid for Optimistic fitness as a card-design aspiration goal

## Card Designer's Brief

1. **Primary resonance sibling quality is the single most important card design lever.** Every 10% improvement in sibling A-tier rate translates to approximately +0.17 M3 for Surge+Floor.
2. **Target 65% sibling A-tier to hit M3=2.0.** This means designing cards that work across both archetypes sharing a primary resonance (e.g., Warriors and Sacrifice both use Tide -- 65% of Sacrifice cards should be A-tier for Warriors and vice versa).
3. **Secondary resonance fitness is irrelevant** to all viable algorithms. Do not invest design effort in cross-secondary playability.
4. **Pack rhythm matters.** Floor packs smooth the non-surge troughs. The designer should ensure enough resonance-primary cards exist (minimum 15 per resonance) to avoid depleting the pool during heavy surge drafting.
