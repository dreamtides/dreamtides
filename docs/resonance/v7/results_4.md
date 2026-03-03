# V7 Agent 4 Results: Aspiration Packs + Biased Random

## One-Sentence Description

"After each pick, compute top resonance pair (R1, R2); if R2 >= 3 tokens AND R2 >= 50% of R1, one slot shows an R1 card, one shows an R2 card, two slots draw from pool weighted 2x toward R1; otherwise all four weighted-random (2x toward R1 once any counter >= 2)."

## Scorecard

### Fitness Model A (Optimistic: 100% cross-archetype A-tier)

| Metric | Target | Asp+Bias 2.0x | Asp+Bias 3.0x | Pure Aspiration | Surge V6 |
|--------|--------|---------------|---------------|-----------------|----------|
| M1 Unique archs early | >= 3 | 5.05 PASS | 4.90 PASS | 5.21 PASS | 4.44 PASS |
| M2 S/A early | <= 2 | 1.52 PASS | 1.75 PASS | 1.21 PASS | 1.91 PASS |
| M3 S/A committed late | >= 2.0 | 1.48 FAIL | 1.87 FAIL | 0.94 FAIL | **2.06 PASS** |
| M4 Off-arch late | >= 0.5 | 1.08 PASS | 0.93 PASS | 1.30 PASS | 0.70 PASS |
| M5 Convergence pick | 5-8 | 13.4 FAIL | **6.8 PASS** | 25.1 FAIL | **6.5 PASS** |
| M6 Deck concentration | 60-90% | 82.9% PASS | 89.5% PASS | 66.4% PASS | 76.9% PASS |
| M7 Card overlap | < 40% | 25.8% PASS | 30.2% PASS | 18.1% PASS | 26.8% PASS |
| M8 Arch freq range | 5-20% | 8-19% PASS | 7-19% PASS | 8-19% PASS | 8-18% PASS |
| M9 S/A stddev | >= 0.8 | 0.96 PASS | 1.00 PASS | 0.83 PASS | 1.41 PASS |
| **Total** | | **7/9** | **8/9** | **7/9** | **9/9** |

### Fitness Model B (Moderate: 50%A/30%B/20%C, S/A = 75%)

| Metric | Target | Asp+Bias 2.0x | Asp+Bias 3.0x | Pure Aspiration | Surge V6 |
|--------|--------|---------------|---------------|-----------------|----------|
| M1 Unique archs early | >= 3 | 4.13 PASS | 4.04 PASS | 4.22 PASS | 3.75 PASS |
| M2 S/A early | <= 2 | 1.21 PASS | 1.37 PASS | 0.95 PASS | 1.44 PASS |
| M3 S/A committed late | >= 2.0 | 1.11 FAIL | 1.39 FAIL | 0.75 FAIL | 1.43 FAIL |
| M4 Off-arch late | >= 0.5 | 1.23 PASS | 1.09 PASS | 1.34 PASS | 0.91 PASS |
| M5 Convergence pick | 5-8 | 22.2 FAIL | 16.0 FAIL | 28.5 FAIL | 10.7 FAIL |
| M6 Deck concentration | 60-90% | 70.9% PASS | 79.1% PASS | 56.7% FAIL | 66.8% PASS |
| M7 Card overlap | < 40% | 24.3% PASS | 29.4% PASS | 18.5% PASS | 25.2% PASS |
| M8 Arch freq range | 5-20% | 8-20% PASS | 8-17% PASS | 9-19% PASS | 7-18% PASS |
| M9 S/A stddev | >= 0.8 | 0.89 PASS | 0.95 PASS | 0.75 FAIL | 1.23 PASS |
| **Total** | | **7/9** | **7/9** | **5/9** | **7/9** |

### Fitness Model C (Pessimistic: 25%A/40%B/35%C, S/A = 62.5%)

| Metric | Target | Asp+Bias 2.0x | Asp+Bias 3.0x | Pure Aspiration | Surge V6 |
|--------|--------|---------------|---------------|-----------------|----------|
| M3 S/A committed late | >= 2.0 | 0.91 FAIL | 1.13 FAIL | 0.65 FAIL | 1.10 FAIL |
| M5 Convergence pick | 5-8 | 26.0 FAIL | 22.0 FAIL | 29.3 FAIL | 17.3 FAIL |
| M6 Deck concentration | 60-90% | 62.6% PASS | 70.0% PASS | 51.2% FAIL | 58.9% FAIL |
| **Total** | | **7/9** | **7/9** | **5/9** | **6/9** |

## Fitness Degradation Curve

M3 (S/A per pack, picks 6+) across fitness models:

| Algorithm | Model A | Model B | Model C | A-to-B drop | A-to-C drop |
|-----------|---------|---------|---------|-------------|-------------|
| Aspiration+Bias 2.0x | 1.48 | 1.11 | 0.91 | -0.37 | -0.57 |
| Aspiration+Bias 3.0x | 1.87 | 1.39 | 1.13 | -0.48 | -0.74 |
| Pure Aspiration | 0.94 | 0.75 | 0.65 | -0.19 | -0.29 |
| Surge V6 (T=4/S=3) | 2.06 | 1.43 | 1.10 | -0.63 | -0.95 |

Aspiration+Bias degrades more gracefully than Surge V6 in absolute terms (0.57 drop vs 0.95 from A to C). However, this is because Aspiration+Bias starts from a lower peak. In relative terms, Surge V6 retains 53% of its Model A performance at Model C, while Aspiration+Bias 2.0x retains 61%. The bias layer adds resilience because biased random draws increase the home-archetype card rate (always S-tier regardless of fitness model), whereas Surge's resonance-matched slots suffer full fitness degradation.

## Per-Archetype Convergence (Model B, Aspiration+Bias 2.0x)

| Archetype | Avg Convergence Pick | N Converged (of ~333) |
|-----------|---------------------|----------------------|
| Flash | 10.8 | 83 |
| Blink | 11.0 | 40 |
| Storm | 12.4 | 25 |
| Self-Discard | 14.8 | 28 |
| Self-Mill | 10.2 | 31 |
| Sacrifice | 13.9 | 31 |
| Warriors | 13.8 | 22 |
| Ramp | 15.7 | 37 |

Only 297 of ~667 non-power-chaser drafts converge at all under Model B. Most convergences are late (pick 10-16). Compare to Surge V6 under Model B: convergence averages pick 10.7 with 609 drafts converging. The dual-resonance structure helps disambiguation but cannot compensate for the base fitness penalty.

## Parameter Sensitivity: Bias Weight (Model B)

| Variant | M3 | M4 | M5 | M6 | M9 | Pass |
|---------|------|------|------|-------|------|------|
| No bias (pure Aspiration) | 0.75 | 1.34 | 28.5 | 56.7% | 0.75 | 5/9 |
| Bias 1.5x | 0.95 | 1.28 | 25.5 | 64.5% | 0.84 | 7/9 |
| Bias 2.0x | 1.11 | 1.23 | 22.2 | 70.9% | 0.89 | 7/9 |
| Bias 3.0x | 1.39 | 1.09 | 16.0 | 79.1% | 0.95 | 7/9 |

The bias layer provides a substantial and monotonic improvement. Each step from no-bias to 3.0x adds approximately +0.2 to M3. The 3.0x variant recovers nearly enough M3 to match Surge V6 at Model B (1.39 vs 1.43) while maintaining better M4 splash (1.09 vs 0.91). However, 3.0x pushes concentration high (79.1%) and reduces M4, approaching the 90% ceiling. Gate threshold sensitivity is minimal: R2>=2/40%, R2>=3/50%, and R2>=4/60% produce nearly identical M3 (1.12, 1.11, 1.10) at Model B, indicating the gate is not the binding constraint.

## Draft Traces (Model B, Aspiration+Bias 2.0x)

**Trace 1: Early Committer (committed strategy, Sacrifice archetype)**
Picks 0-4: Drafts mostly Tide cards by power preference. By pick 4, counters reach [E:3 S:2 T:9 Z:0]. R1=Tide (9), R2=Ember (3). Gate opens at pick 4 (R2=3, 3/9=33% < 50%). Gate actually opens around pick 5-6 when Stone accumulates. Aspiration slots begin delivering Tide and Stone cards. By pick 12, pack S/A fluctuates between 0-2, consistent with Model B degradation. Final deck: 21/30 S/A (70% concentration), with 17S and 4A -- showing that home-archetype cards dominate because sibling A-tiers are rarer under Model B.

**Trace 2: Power Chaser**
Picks cards purely by power. No commitment ever established. Resonance counters become dominated by Zephyr/Tide by pick 8, but the bias toward those resonances provides incidental archetype coherence. The power chaser naturally gravitates toward higher-power cards in the biased pool.

**Trace 3: Signal Reader (Sacrifice archetype)**
Follows resonance signals aggressively. Picks Tide cards from the start, accumulating tokens rapidly. Gate opens early. The R1 (Tide) slot and bias toward Tide produce a stream of Sacrifice and Warriors home cards. Under Model B, many Warriors-home cards with Tide primary are B or C tier for Sacrifice, producing the observed 60% concentration -- lower than Trace 1 despite earlier commitment.

## Comparison to Pure Aspiration (Agent 3/7) and Surge V6 (Agent 1)

**vs Pure Aspiration:** The bias layer is transformative. At Model B, bias 2.0x adds +0.36 M3 (0.75 to 1.11), recovers M6 from 56.7% to 70.9%, and restores M9 above 0.8. Pure Aspiration without bias is not a viable standalone algorithm -- its two guaranteed resonance-matched slots contribute too few S/A cards per pack to drive convergence. The bias converts the two "random" slots from ~22% on-resonance to ~35%, adding approximately +0.13 archetype-level S/A per random slot under Model B.

**vs Surge V6:** Surge V6 outperforms Aspiration+Bias at every fitness level. At Model A: 2.06 vs 1.48 M3. At Model B: 1.43 vs 1.11. At Model C: 1.10 vs 0.91. Surge's advantage stems from its 3-of-4 resonance-filled surge packs firing every ~2 picks for committed players, delivering concentrated resonance spikes that the continuous-but-shallow Aspiration approach cannot match. However, Aspiration+Bias degrades more gracefully (retains 61% at C vs 53% for Surge), has better M4 splash at every level, and converges more evenly across archetypes (no archetype < 22 converged vs Surge's occasional archetype with very few convergences). At Model C, Aspiration+Bias 2.0x passes 7/9 while Surge V6 passes only 6/9 (Surge fails M6 at 58.9%).

**Key finding: Aspiration+Bias 3.0x is the closest competitor to Surge V6.** At Model B it reaches 1.39 M3 vs Surge's 1.43 -- a 0.04 gap. At Model C it reaches 1.13 vs 1.10, actually exceeding Surge. The 3.0x variant is the recommended configuration if Aspiration+Bias is selected over Surge.

## Self-Assessment

**Honest verdict: Aspiration+Bias does not beat Surge V6 as a standalone algorithm.** The pre-simulation predictions of 2.20 (Optimistic) and 1.75 (Moderate) were significantly too optimistic. The actual results (1.48 and 1.11 at 2.0x bias) show that the layered approach -- dual-resonance aspiration slots plus weighted random draws -- underperforms a concentrated surge mechanism.

**Why the prediction was wrong:** The design analysis assumed that two guaranteed resonance-matched slots (R1 + R2) would each deliver ~0.75 S/A under Model B. In practice, the R2 slot delivers significantly less because R2's resonance pool is less concentrated toward the player's specific archetype. Additionally, the biased random slots' improvement is real but smaller than predicted: the 2x weight shifts from ~22% to ~35% on-resonance, but under Model B only 75% of those are S/A, so the per-slot gain is roughly +0.10 rather than the predicted +0.13.

**Where Aspiration+Bias excels:** M4 off-archetype splash (consistently 1.0+ vs Surge's 0.7-0.9), M7 card overlap (lower, meaning more variety), and M6 robustness under pessimistic fitness (passes at C where Surge fails). The dual-resonance structure provides genuine archetype disambiguation. The algorithm is also simpler than Surge -- no token spending, no threshold tracking, just "read your top two resonances."

**The bias layer is confirmed as a valuable component.** It adds +0.36 M3 over pure Aspiration at Model B and should be applied to whichever base algorithm is selected. The 3.0x variant is competitive with Surge V6 under pessimistic fitness and may be the correct choice if the card pool turns out to have highly specialized archetypes.
