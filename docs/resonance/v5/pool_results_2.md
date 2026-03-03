# Pool Investigation 2: Rarity x Pair-Escalation Slots

## Summary

Rarity is largely **orthogonal** to Pair-Escalation's core convergence behavior. All 5 models converge at pick 7.2--7.5 and achieve 2.57--2.61 late S/A. The algorithm's pair-tracking mechanism is robust enough that symbol distribution changes caused by rarity correlation do not meaningfully alter pack quality. Where rarity *does* matter is in **draft tension** and **power variance**, both of which are design levers rather than balance problems.

## Models Tested (1200 drafts each, K=6, cap=0.50)

| Model | Description | Passes |
|-------|-------------|--------|
| A: Flat | Uniform power, cosmetic rarity | 6/9 |
| B: Standard TCG | 180C/100U/60R/20L, scaled power | 7/9 |
| C: Roguelike | 120C/100U/90R/50L, scaled power | 6/9 |
| D: Symbol Correlation | Rares have 2-3 symbols (accelerate pairs) | 7/9 |
| E: Inverse Correlation | Rares have 1 symbol (stall pairs) | 7/9 |

All models fail deck concentration (96-97%, too high) and off-archetype C/F (0.46, just below 0.50). These are Pair-Escalation structural issues, not rarity-related.

## Core Finding: Convergence Is Insensitive to Rarity

| Metric | A:Flat | B:TCG | C:Rogue | D:SymCor | E:InvCor |
|--------|--------|-------|---------|----------|----------|
| Late S/A | 2.61 | 2.58 | 2.59 | 2.57 | 2.57 |
| Conv pick | 7.2 | 7.3 | 7.4 | 7.5 | 7.5 |
| Late S/A stddev | 0.97 | 0.98 | 0.98 | 0.99 | 0.99 |
| Card overlap | 3.6% | 6.3% | 4.6% | 3.7% | 5.8% |

The range across all models is just 0.04 S/A and 0.3 picks. This is because the pair-escalation probability (min(count/6, 0.50)) depends on how many 2+ symbol cards you *draft*, not how many exist. All strategies draft roughly the same number of pair-contributing cards because the pool always has ~85% non-generic cards with adequate 2+ symbol representation.

## Where Rarity Matters: Tension and Power

### Draft Tension Rate

Tension = packs where highest-power card and highest-fitness card differ by >= 1.5 power.

| Model | Tension % | Rare-power vs Common-fitness % |
|-------|-----------|-------------------------------|
| A: Flat | 0.0% | 0.0% |
| B: TCG | 33.2% | 65.3% |
| C: Rogue | 33.2% | 73.1% |
| D: SymCor | 27.6% | 70.0% |
| E: InvCor | 38.1% | 66.1% |

Model A has zero tension (uniform power). Models B-E all create meaningful tension (28-38%). **Model E creates the most tension** (38.1%) because rares are powerful but contribute no pairs, creating a genuine "power vs. convergence" dilemma. Model D has the *least* tension among scaled-power models (27.6%) because rares both contribute pairs and have high power, aligning incentives.

### Power Variance

| Model | Avg Power | Power StdDev | Power Gap (chaser - committed) |
|-------|-----------|-------------|-------------------------------|
| A: Flat | 5.13 | 0.069 | 0.18 |
| B: TCG | 5.84 | 0.552 | 1.39 |
| C: Rogue | 6.73 | 0.583 | 1.43 |
| D: SymCor | 6.40 | 0.603 | 1.14 |
| E: InvCor | 5.23 | 0.405 | 1.55 |

Model E shows the largest power gap (1.55) between power-chasers and committed players, confirming that inverse correlation punishes archetype commitment. Model D has the smallest gap (1.14), meaning rarity-symbol alignment lets committed players naturally draft high-power rares.

### Pair Contribution by Rarity

| Model | Commons % | Rares+Legendaries % |
|-------|-----------|---------------------|
| A: Flat | 48.7% | 21.9% |
| B: TCG | 33.1% | 34.9% |
| D: SymCor | 16.3% | 52.3% |
| E: InvCor | 49.0% | 11.6% |

In Model D, rares contribute 52% of all pairs. In Model E, rares contribute only 12%. This 40pp swing drives the tension difference but does *not* significantly alter convergence speed, because the total pair count reaches similar levels regardless (top pair at pick 20: D=15.6, E=15.2).

## Power Chaser Impact

Model E uniquely degrades the power-chaser strategy: their deck concentration drops to 54.9% (vs 64-71% for other models) because chasing rares pulls them away from any archetype. This is a desirable property -- it creates a natural penalty for ignoring archetype signals.

## Recommendation

**Model B (Standard TCG)** is the safest default. It creates meaningful tension (33%), moderate power variance, and doesn't interact with convergence. Model E (inverse correlation) is the most interesting design choice: it creates a genuine "power vs. pairs" dilemma (38% tension) and punishes power-chasing (55% deck concentration), but the effect on convergence is negligible. Model D (positive correlation) reduces tension and makes the draft easier/more linear.

**Key design insight:** Rarity-symbol correlation is a *feel* lever, not a *balance* lever. Pair-Escalation is robust enough that you can choose any rarity model without breaking convergence. Pick the model that creates the draft experience you want: inverse correlation for more tension, positive correlation for smoother drafting, standard TCG for the familiar middle ground.
