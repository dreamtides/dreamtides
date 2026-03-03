# Agent 3 Comparison: Aspiration Postmortem Perspective

## Scorecard (1-10, all algorithms, by fitness level)

### Optimistic Fitness (Model A)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias | Compass | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Simple | 8 | 7 | 10 | 7 | 6 | 5 | 10 |
| No actions | 10 | 10 | 10 | 10 | 10 | 10 | 10 |
| Not on rails | 8 | 8 | 9 | 8 | 7 | 8 | 9 |
| No forced decks | 7 | 7 | 8 | 7 | 7 | 7 | 8 |
| Flexible | 7 | 7 | 8 | 7 | 6 | 7 | 8 |
| Convergent | 8 | 9 | 2 | 6 | 4 | 8 | 2 |
| Splashable | 7 | 8 | 9 | 8 | 7 | 7 | 9 |
| Open early | 8 | 8 | 9 | 8 | 8 | 8 | 9 |
| Signal reading | 6 | 7 | 3 | 5 | 3 | 6 | 3 |
| **Total** | **69** | **71** | **68** | **66** | **58** | **66** | **68** |

### Moderate Fitness (Model B)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias | Compass | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Convergent | 4 | 7 | 1 | 3 | 3 | 4 | 1 |
| Signal reading | 4 | 6 | 1 | 4 | 2 | 4 | 1 |
| **Adj Total** | **63** | **67** | **58** | **60** | **52** | **60** | **58** |

### Pessimistic Fitness (Model C)

| Goal | Surge V6 | Surge+Floor | Asp+Pair | Asp+Bias | Compass | DualCounter | Pure Asp |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Convergent | 3 | 5 | 1 | 2 | 2 | 3 | 1 |
| Signal reading | 3 | 4 | 1 | 3 | 1 | 3 | 1 |
| **Adj Total** | **59** | **63** | **56** | **57** | **48** | **56** | **56** |

## Biggest Strength and Weakness

| Algorithm | Biggest Strength | Biggest Weakness |
|-----------|-----------------|------------------|
| Surge V6 T=4 | Only algorithm achieving 9/9 at any fitness level | T=4 fires too infrequently under moderate fitness |
| Surge+Floor T=3 | Floor eliminates the non-surge "dead zone" problem | Added floor rule is extra complexity for ~0.4 M3 over T=3 alone |
| Asp+Pair | Gate paradox is the most instructive failure of V7 | Committed players lock themselves out of the gate |
| Asp+Bias 3.0x | Demonstrates bias as a valid convergence component | Still 25% below Surge+Floor at Moderate |
| Compass 2+1+1 | Proves that extra R1 slots are what matters, not rotation | Neighbor slot is 1.2% S/A; entire compass concept is dead |
| Dual-Counter | Confirms cost is orthogonal to resonance for archetype disambiguation | +0.05 S/A; indistinguishable from Surge V6 |
| Pure Aspiration | Cleanest falsification: 2 targeted slots cannot compete with 3 | M3=0.72-1.02 across all fitness levels |

## Graceful Degradation: The Honest Answer

**Asp+Bias 3.0x degrades most gracefully in percentage terms** (retains 60% from A to C), but this is the wrong metric. The right question is: "which algorithm stays closest to passing M3 under degraded fitness?" Answer:

| Algorithm | M3 at Model B | Gap to M3=2.0 | Gap to M3=1.8 |
|-----------|:---:|:---:|:---:|
| Surge+Floor T=3 | 1.85 | -0.15 | **+0.05 PASS** |
| Surge V6 T=3 | 1.88 | -0.12 | **+0.08 PASS** |
| DualCounter T=3 | 1.88 | -0.12 | **+0.08 PASS** |
| Compass 2+1+1 | 1.66 | -0.34 | -0.14 |
| Asp+Bias 3.0x | 1.39 | -0.61 | -0.41 |
| Asp+Pair | 0.84 | -1.16 | -0.96 |
| Pure Aspiration | 0.72-0.84 | -1.16+ | -0.96+ |

Only Surge-family algorithms with T=3 are within striking distance. All Aspiration variants are catastrophically far from any reasonable target.

**My honest conclusion as the Aspiration+Pair advocate: Aspiration Packs is dead.** The R2 slot hypothesis was falsified. The dual-resonance pair concept fails because R2's primary pool targets wrong archetypes. I am switching my recommendation to Surge+Floor T=3.

## What Aspiration Taught Us

Three structural lessons from the Aspiration failure:

1. **Slot count dominates slot precision.** Surge's 3 matched slots at 75% S/A (Model B) beat Aspiration's 1 matched slot at 100% + 1 matched slot at 4% S/A. Raw slot count matters more than clever targeting.

2. **Secondary resonance is not an archetype signal.** R2 maps to the player's secondary resonance, but R2-primary cards belong to different archetypes. The resonance-to-archetype mapping is one-to-many, and the secondary resonance amplifies the wrong "many."

3. **Gate conditions that punish commitment are fatal.** The R2 >= 50% * R1 gate closes for committed drafters whose R1 grows much faster than R2. An algorithm that fails for its intended audience is broken.

## Minimum Cross-Archetype A-Tier Rate

| Algorithm | Rate for M3=2.0 | Rate for M3=1.8 |
|-----------|:---:|:---:|
| Surge+Floor T=3 | ~65% | ~50% (current Model B) |
| Surge V6 T=3 | ~63% | ~48% |
| Asp+Bias 3.0x | Impossible | ~90%+ |
| All other Aspiration | Impossible | Impossible |

The Surge family needs approximately 50% A-tier siblings for M3=1.8 and 65% for M3=2.0.

## Proposed Best Algorithm

**Surge Packs + Floor (T=3, S=3, floor_start=3).**

I considered whether the bias component from Agent 4 could be layered onto Surge+Floor. The answer is: it could be applied to the floor slot (bias the 3 random slots on floor packs toward the top resonance), but the improvement would be marginal (~0.05 M3) and adds a parameter. The floor slot already shows the top resonance. Biasing the remaining 3 random slots by 2x would shift them from ~25% on-resonance to ~35%, adding ~0.1 S/A per floor pack. With floor packs comprising ~60% of packs, this adds ~0.06 M3 overall. Not worth the complexity.

## Revised M3 Target

**M3 >= 1.8 under Moderate fitness.** This is the only target that any algorithm can meet. The original 2.0 assumed optimistic fitness. Under realistic card design (50% sibling A-tier), 1.8 is the practical ceiling. If the card designer achieves 65%+ A-tier, the algorithm will exceed 2.0 automatically.

## Card Designer's Brief

1. **The algorithm cannot compensate for weak card design.** Every algorithm's M3 is linearly proportional to sibling A-tier rate. There is no "clever algorithm" that overcomes bad cross-archetype fitness.
2. **Design sibling archetypes as "good stuff" pairs.** For each primary resonance, the two archetypes sharing it should have 60-70% card overlap at A-tier. This means their signature mechanics should be complementary, not orthogonal.
3. **Avoid narrow parasitic mechanics.** Cards that read "only playable in exactly one archetype" reduce sibling A-tier rate. Prefer broadly-playable effects with archetype-specific upside.
4. **Test cards against sibling fitness explicitly.** For every card, ask: "Is this at least A-tier in the sibling archetype?" If fewer than 60% of an archetype's cards pass this test, the draft algorithm will underperform.
