# Agent 7 Results: Aspiration Packs (Pure + 3 Variants)

## One-Sentence Descriptions

**Variant A (Pure Aspiration):** "After each pick, compute top resonance pair (R1, R2); if R2 >= 3 tokens AND R2 >= 50% of R1, one slot shows an R1 card, one R2 card, two random; otherwise all four random."

**Variant B (+ Pair Pref):** Same gate, but the R1 slot prefers cards also carrying R2 as a secondary symbol.

**Variant C (+ Bias):** Same gate, but the two random slots are weighted 2x toward R1's resonance.

**Variant D (+ Floor):** Same gate, but when open: 2 R1 slots + 1 R2 slot + 1 random (instead of 1+1+2).

## Comparison Scorecard

### Optimistic Fitness (Cross-archetype = 100% A-tier)

| Metric | Target | Var A | Var B | Var C | Var D |
|--------|--------|:-----:|:-----:|:-----:|:-----:|
| M1: Early unique archs | >= 3 | 5.20 | 5.20 | 5.06 | 5.13 |
| M2: Early S/A emerging | <= 2 | 0.94 | 0.98 | 1.07 | 1.05 |
| M3: Late S/A committed | >= 2 | **0.92** | **1.02** | **0.97** | **0.95** |
| M4: Off-archetype | >= 0.5 | 3.08 | 2.98 | 3.03 | 3.05 |
| M5: Convergence pick | 5-8 | **22.4** | **20.3** | **16.5** | **18.2** |
| M6: Deck concentration | 0.60-0.90 | 0.66 | 0.72 | 0.66 | 0.66 |
| M7: Run overlap | < 0.40 | 0.08 | 0.08 | 0.08 | 0.08 |
| M8: Archetype freq | 5-20% | PASS | PASS | PASS | PASS |
| M9: S/A stddev | >= 0.8 | 0.83 | 0.82 | 0.89 | 0.86 |
| **Pass count** | | **7/9** | **7/9** | **7/9** | **7/9** |

### Moderate Fitness (50%A/30%B/20%C, S/A = 75%)

| Metric | Target | Var A | Var B | Var C | Var D |
|--------|--------|:-----:|:-----:|:-----:|:-----:|
| M3: Late S/A committed | >= 2 | **0.72** | **0.84** | **0.77** | **0.76** |
| M5: Convergence pick | 5-8 | **27.2** | **26.0** | **22.6** | **24.5** |
| M6: Deck concentration | 0.60-0.90 | **0.56** | 0.64 | **0.56** | **0.56** |
| M9: S/A stddev | >= 0.8 | **0.75** | **0.75** | 0.82 | 0.80 |
| **Pass count** | | **5/9** | **6/9** | **6/9** | **5/9** |

### Pessimistic Fitness (25%A/40%B/35%C, S/A = 62.5%)

| Metric | Target | Var A | Var B | Var C | Var D |
|--------|--------|:-----:|:-----:|:-----:|:-----:|
| M3: Late S/A committed | >= 2 | **0.63** | **0.75** | **0.67** | **0.66** |
| M5: Convergence pick | 5-8 | **28.6** | **27.6** | **25.6** | **26.7** |
| M6: Deck concentration | 0.60-0.90 | **0.50** | **0.59** | **0.50** | **0.51** |
| M9: S/A stddev | >= 0.8 | **0.71** | **0.71** | **0.78** | **0.75** |
| **Pass count** | | **5/9** | **5/9** | **5/9** | **5/9** |

## R2 Slot S/A Breakdown

The R2 slot is the core differentiator of Aspiration Packs. Under all fitness models, it underperforms expectations dramatically:

| Fitness | Variant | S% | A% | B% | C% | **S/A total** |
|---------|---------|:--:|:--:|:--:|:--:|:------------:|
| Optimistic | A | 7.9 | 8.6 | 41.4 | 42.0 | **16.5%** |
| Optimistic | B | 2.4 | 2.2 | 47.1 | 48.4 | **4.5%** |
| Optimistic | C | 6.8 | 8.0 | 44.0 | 41.2 | **14.8%** |
| Optimistic | D | 7.5 | 8.4 | 41.9 | 42.2 | **15.9%** |
| Moderate | A | 6.8 | 2.8 | 44.3 | 46.2 | **9.5%** |
| Moderate | B | 3.0 | 1.3 | 47.2 | 48.5 | **4.3%** |
| Pessimistic | A | 6.2 | 1.6 | 47.4 | 44.9 | **7.7%** |

**Diagnosis:** R2 is the player's second-strongest resonance, which is secondary for their target archetype. But R2's primary pool contains cards from two archetypes that use R2 as their primary resonance -- and these archetypes may share no meaningful overlap with the player's target. For example, a Warriors (Tide/Zephyr) player gets R2=Zephyr. The Zephyr primary pool includes Flash and Ramp cards. Only Ramp shares an adjacency with Warriors (50% chance), and even then fitness depends on the model. The R2 slot is effectively random for archetype convergence purposes: ~85% of R2 cards are B/C-tier.

Variant B's pair-preference filtering actually *worsens* R2 S/A (4.5% vs 16.5%) because it constrains the R1 slot to dual-resonance cards, which are rare (15% cap), and the constraint shifts S-tier cards away from the R1 position.

## Fitness Degradation Curves

| Variant | Optimistic M3 | Moderate M3 | Pessimistic M3 | Drop Opt->Pess |
|---------|:------------:|:-----------:|:--------------:|:--------------:|
| A | 0.92 | 0.72 | 0.63 | -31% |
| B | 1.02 | 0.84 | 0.75 | -26% |
| C | 0.97 | 0.77 | 0.67 | -31% |
| D | 0.95 | 0.76 | 0.66 | -31% |

All variants degrade roughly 30% from optimistic to pessimistic. Variant B degrades slightly less (26%) due to its pair-preference filtering providing marginally more archetype precision. But all variants remain far below the 2.0 target at every fitness level.

The convergence pick (M5) is catastrophically high across all variants and fitness levels, ranging from 16.5 (Variant C, optimistic) to 28.6 (Variant A, pessimistic). The algorithm simply does not concentrate archetype-relevant cards fast enough.

## Best Variant Recommendation

**Variant B (+ Pair Pref)** is the strongest performer, achieving the highest M3 at every fitness level (1.02/0.84/0.75). It benefits from the pair-preference R1 slot filter, which slightly increases archetype precision even though it reduces R2 slot quality. Under moderate fitness, Variant B is the only variant to pass M6 (deck concentration = 0.64).

However, the honest assessment is that **no Aspiration Packs variant is competitive with Surge Packs.** V6 Surge Packs achieved M3 = 2.08 at T=4/S=3 under optimistic fitness. Even under the most generous comparison (Variant B optimistic), Aspiration Packs reaches only 1.02 -- less than half of Surge Packs' performance.

## Per-Archetype Convergence (Variant B, Moderate Fitness)

| Archetype | Convergence Pick |
|-----------|:---------------:|
| Flash | 23.8 |
| Blink | 27.5 |
| Storm | 24.5 |
| Self-Discard | 25.5 |
| Self-Mill | 25.5 |
| Sacrifice | 27.1 |
| Warriors | 27.5 |
| Ramp | 27.0 |

All archetypes converge extremely late (23.8-27.5), far outside the 5-8 target. This means the algorithm never reliably delivers 2+ S/A cards per pack for any archetype.

## Parameter Sensitivity (Moderate Fitness)

| Gate | Variant B Late SA | Aspiration % |
|------|:-:|:-:|
| R2>=2/40% | 0.95 | 64.9% |
| R2>=3/50% | 0.83 | 50.2% |
| R2>=4/60% | 0.75 | 32.3% |

Loosening the gate (R2>=2/40%) increases aspiration frequency to 65% and raises M3 to 0.95, but this still falls short of 2.0 by a wide margin. The fundamental constraint is that aspiration packs target only 2 of 4 slots (or 3 for Variant D), and the targeted slots have low archetype precision.

## Draft Traces (Variant B, Moderate Fitness)

**Trace 1 (Committed Warriors):** 0 aspiration packs fired in 15 picks shown. The player drafts Tide-heavy cards, building R1=Tide rapidly, but R2 (Stone or Zephyr) grows slowly because most Tide cards have mono-Tide symbols. The gate requires R2 >= 50% of R1, which is difficult when R1 grows much faster than R2. When aspiration packs do fire (not shown in first 15), R2=Stone yields Self-Discard/Self-Mill cards that are mostly C-tier for Warriors.

**Trace 2 (Power-Chaser):** 25 aspiration packs fired. The power-chaser's scattered picks happen to build both Zephyr and Stone counters, satisfying the gate early. But without a committed archetype, the aspiration slots show diverse cards with no convergence direction -- every card reads as "?" tier because no target is set.

**Trace 3 (Signal Reader, committed to Ramp):** Only 1 aspiration pack in 15 picks. The signal reader rapidly accumulates Zephyr (R1) but R2 lags far behind. Most packs are fully random. When aspiration packs fire, they show Flash/Ramp (Zephyr) and Self-Mill/Self-Discard (Stone) -- the Stone cards are mostly B/C-tier for Ramp.

## Self-Assessment

Aspiration Packs fails conclusively. The design hypothesis -- that dual-resonance targeting (R1+R2 slots) would provide archetype disambiguation superior to single-resonance Surge Packs -- is falsified by simulation.

**Root cause:** The R2 slot was intended to narrow from 4 candidate archetypes to 1-2 by providing the player's secondary resonance context. In practice, R2's primary pool is dominated by archetypes that share no meaningful mechanical overlap with the player's target. Only ~8-17% of R2 slot cards are S/A-tier (vs. the predicted ~37-50% from the design phase). The design document overestimated R2 precision by 3-4x.

**Structural lesson:** Aspiration Packs target only 2 slots (50% of pack). Surge Packs target 3 slots (75% of pack). This 1.5x slot-count disadvantage compounds with R2's low precision. Surge Packs at T=4/S=3 achieve ~64% surge frequency with 75% slot targeting = ~48% effective targeting. Aspiration Packs at best achieve ~50% aspiration frequency with 37.5% effective targeting (1.5 good slots / 4) = ~19% effective targeting. The mechanism is 2.5x less efficient than Surge Packs.

**Why the predictions were wrong:** The design predicted R2 slot S/A = ~50% under optimistic fitness, but actual measurement shows 16.5%. The error stems from conflating "R2 is the player's secondary resonance" with "R2's primary pool contains the player's archetype's cards." R2's pool contains cards from all archetypes using R2 as primary -- typically 2 archetypes, only one of which is adjacent to the player. The adjacent archetype accounts for 50% of R2 pool, and only S-tier cards from that adjacent archetype (if any) are truly relevant.

| Goal | Score | Notes |
|------|:-----:|-------|
| Simple | 10 | Simplest mechanism tested; one gate, no tokens spent |
| No actions | 10 | Fully passive |
| Not on rails | 9 | Non-permanent tracking, can pivot |
| No forced decks | 8 | Gate prevents premature locking |
| Flexible | 8 | R1/R2 pair shifts with picks |
| Convergent | 2 | 0.84-1.02 S/A, never approaches 2.0 |
| Splashable | 9 | 2-3 random slots always present |
| Open early | 9 | Gate delays activation |
| Signal reading | 4 | Minimal benefit from reading signals |

Aspiration Packs excels on simplicity (score 10) and openness (score 9) but catastrophically fails on convergence (score 2). The design trades convergence for simplicity, and the trade is not worth it. Surge Packs' additional complexity (token spending, surge/non-surge alternation) purchases 2x the convergence power.

**Recommendation:** Aspiration Packs should not advance to the final comparison as a competitive algorithm. Its contribution is as a lower bound -- it demonstrates that purely gated 2-slot targeting without token accumulation is structurally insufficient. Any competitive algorithm needs either 3+ targeted slots (Surge Packs) or a fundamentally different approach to archetype precision.
