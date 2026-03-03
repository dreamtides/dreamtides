# V7 Agent 1 Discussion: Baselines Perspective

## Agreed Fitness Models for Simulation

After reviewing all proposals, I advocate converging on three fitness models. Agent 3's Complementary Pair Aware model is theoretically interesting but introduces a variable (pair-aware fitness) that confounds comparison. Agent 6's Designed Cost Separation model embeds a card-design assumption into the fitness model, which tests card design rather than algorithm quality. The shared models should be:

1. **Optimistic (A):** Cross-archetype = 100% A-tier. Resonance-matched S/A = 100%. The V6 baseline. Every simulation must include this for backward compatibility.
2. **Moderate (B):** Cross-archetype = 50% A, 30% B, 20% C. Resonance-matched S/A = 75%. The primary realistic target. All champions must be compared here.
3. **Pessimistic (C):** Cross-archetype = 25% A, 40% B, 35% C. Resonance-matched S/A = 62.5%. Stress test for specialized card pools.

Models D (Severe), Complementary Pair Aware, and Designed Cost Separation should be run as secondary experiments only if time permits. The core comparison is A vs B vs C across all champions.

## Simplicity Ranking (Simplest to Most Complex)

1. **Aspiration Packs (Agent 7)** -- no tokens, no thresholds; just read top-2 resonances and fill 2 slots. Gate logic is the only wrinkle.
2. **Archetype Compass Packs (Agent 5)** -- no tokens, no thresholds; 1 top-resonance slot + 1 rotating neighbor slot. Marginally more complex due to rotation rule.
3. **Surge + Floor (Agent 2)** -- standard Surge Packs plus one guaranteed floor slot on normal packs. Two behaviors to explain.
4. **Pair Surge (Agent 3)** -- Surge Packs with modified 2+1+1 composition. Same token system, different slot fill.
5. **Biased Surge (Agent 4)** -- Surge Packs with weighted random draws. Requires explaining probability weighting.
6. **Surge Packs V6 (Baseline)** -- the known quantity. Tokens, thresholds, spending.
7. **Dual-Counter Surge (Agent 6)** -- Surge Packs plus cost-profile tracking and cost-band filtering. Two tracking systems.

## Scorecard

| Metric | Target | Surge V6 | Surge+Floor (2) | Pair Surge (3) | Biased Surge (4) | Compass (5) | Dual-Counter (6) | Aspiration (7) |
|--------|--------|----------|-----------------|----------------|-------------------|-------------|-------------------|----------------|
| M3: Late S/A (Moderate) | >=2.0 | 1.65 | 1.90 | 1.75 | 1.75 | 1.80 | 1.80 | 1.75 |
| M5: Convergence pick | 5-8 | 5.9 | 4.5 | 5.5 | 5.5 | 4.0 | 5.9 | 4.0 |
| M6: Concentration | 60-90% | 76% | 82% | 74% | 80% | 75% | 76% | 72% |
| M7: Card overlap | <40% | 28% | 30% | 26% | 30% | 25% | 28% | 24% |
| M9: S/A stddev | >=0.8 | 1.42 | 1.0 | 1.30 | 1.10 | 1.20 | 1.40 | 1.15 |
| Simplicity | high | 6 | 5 | 5 | 5 | 8 | 3 | 9 |

Predicted M3 values are for Moderate fitness. All are below 2.0. This is the central finding reaffirmed: no algorithm achieves 2.0 S/A under Moderate fitness through resonance-level targeting alone.

## Critical Cross-Domain Observations

**The convergence of Agents 3, 5, and 7.** Pair Surge, Compass Packs, and Aspiration Packs all fill pack slots from the player's top two resonances. The mechanical differences are minor: Pair Surge embeds dual-resonance targeting inside surge packs only; Compass rotates between neighbors; Aspiration fills both every pack with a gate. These should converge into a single "dual-resonance targeting" champion for simulation, with the three variants tested as parameter settings rather than separate algorithms.

**Should M3 be lowered?** Under Moderate fitness, my baseline analysis showed all algorithms landing at 1.6-1.7. The best champions add 0.1-0.2 above that. I recommend lowering the M3 target to 1.8 for Moderate fitness evaluation, with the understanding that card designers must achieve at least 60% cross-archetype A-tier to push the algorithm past 2.0 in production.

**Agent 2's Surge + Floor is the most targeted fix.** It directly addresses the weakest link (normal packs) identified in my baseline analysis. However, it does not address the disambiguation problem -- it just adds more resonance-matched cards, which still suffer from fitness degradation.

## Final Champion

**Surge Packs V6 (unchanged baseline)**. My role is to provide the reference point. All other champions must beat my baseline under each fitness model.

## Planned Modifications for Simulation

None. Surge Packs V6 is implemented exactly as specified: T=4, S=3, +2/+1 token earning, no floor, no bias. This is the control group.
