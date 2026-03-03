# V7 Agent 1 Results: Surge Packs V6 Baseline (T=4, S=3)

## Algorithm

Each drafted symbol adds tokens (+2 primary, +1 others); when any counter reaches 4, spend 4 and fill 3 of the next pack's 4 slots with random cards of that resonance, fourth slot random.

## Scorecard

| Metric | Target | Model A (Opt) | Model B (Mod) | Model C (Pess) |
|--------|--------|:---:|:---:|:---:|
| M1 Unique archs early | >= 3 | 4.46 PASS | 3.70 PASS | 3.28 PASS |
| M2 S/A early | <= 2 | 1.89 PASS | 1.43 PASS | 1.21 PASS |
| M3 S/A committed late | >= 2.0 | 2.03 PASS | 1.43 FAIL | 1.09 FAIL |
| M4 Off-arch late | >= 0.5 | 0.70 PASS | 0.90 PASS | 1.09 PASS |
| M5 Convergence pick | 5-8 | 6.2 PASS | 10.8 FAIL | 17.9 FAIL |
| M6 Deck concentration | 60-90% | 75.9% PASS | 66.6% PASS | 58.6% FAIL |
| M7 Card overlap | < 40% | 26.4% PASS | 25.4% PASS | 23.8% PASS |
| M8 Arch frequency | 5-20% | 8.1-17.1% PASS | 7.5-17.5% PASS | 9.0-17.1% PASS |
| M9 S/A stddev | >= 0.8 | 1.42 PASS | 1.23 PASS | 1.10 PASS |
| **Total** | **9/9** | **9/9** | **7/9** | **6/9** |

Resonance-matched S/A precision: A=100.0%, B=74.4%, C=61.6%.

## Fitness Degradation Curve

| Metric | Model A | Model B | Model C | A->B delta | B->C delta |
|--------|---------|---------|---------|-----------|-----------|
| M1 Unique archs | 4.46 | 3.70 | 3.28 | -0.76 | -0.42 |
| M2 S/A early | 1.89 | 1.43 | 1.21 | -0.46 | -0.22 |
| M3 S/A late | 2.03 | 1.43 | 1.09 | **-0.60** | -0.34 |
| M4 Off-arch | 0.70 | 0.90 | 1.09 | +0.20 | +0.19 |
| M5 Convergence | 6.2 | 10.8 | 17.9 | **+4.6** | **+7.1** |
| M6 Concentration | 75.9% | 66.6% | 58.6% | -9.3% | -8.0% |
| M7 Overlap | 26.4% | 25.4% | 23.8% | -1.0 | -1.7 |
| M9 S/A stddev | 1.42 | 1.23 | 1.10 | -0.19 | -0.14 |

Key finding: **M3 drops 0.60 from A to B** (2.03 to 1.43), falling below the 2.0 threshold. M5 degrades catastrophically: convergence pick jumps from 6.2 to 10.8 to 17.9 as fewer packs reach the 2+ S/A threshold needed for the 3-consecutive-pack convergence test. M4 (off-archetype cards) actually increases under worse fitness models because more resonance-matched cards are now B/C-tier. M7 and M9 remain healthy across all models.

## Per-Archetype Convergence Table

| Archetype | Model A Pick | Model B Pick | Model C Pick | N Runs (A/B/C) |
|-----------|:-----------:|:-----------:|:-----------:|:---:|
| Flash | 6.2 | 6.5 | 9.9 | 112/108/86 |
| Blink | 6.3 | 9.0 | 10.5 | 59/69/55 |
| Storm | 6.8 | 10.4 | 11.7 | 93/73/69 |
| Self-Discard | 5.3 | 9.9 | 12.3 | 113/88/50 |
| Self-Mill | 5.5 | 8.9 | 12.1 | 70/58/61 |
| Sacrifice | 5.6 | 9.0 | 13.1 | 101/106/44 |
| Warriors | 5.6 | 9.3 | 10.5 | 57/60/30 |
| Ramp | 6.1 | 10.2 | 11.4 | 54/46/38 |

Under Model A all archetypes converge by pick 5-7. Under Model B, Flash remains fastest (6.5) while Storm and Ramp lag to pick 10+. Under Model C, no archetype converges before pick 10 and several archetypes fail to converge in many runs (fewer N Runs). Archetypes with more S-tier home cards in the pool (Flash, Self-Discard, Sacrifice under A) converge faster because they benefit more from resonance matching.

## Parameter Sensitivity Table (selected)

| Model | T | S | M3 | M5 | Pass |
|-------|---|---|-----|------|------|
| A | 3 | 2 | 2.16 | 3.9 | 8/9 |
| A | 3 | 3 | 2.66 | 3.9 | 6/9 |
| **A** | **4** | **3** | **2.03** | **6.2** | **9/9** |
| A | 4 | 4 | 2.22 | 6.4 | 8/9 |
| A | 5 | 3 | 1.75 | 10.9 | 7/9 |
| B | 3 | 3 | 1.88 | 5.8 | 8/9 |
| **B** | **3** | **4** | **1.88** | **5.4** | **6/9** |
| B | 4 | 3 | 1.42 | 10.8 | 7/9 |
| B | 4 | 4 | 1.55 | 9.5 | 7/9 |
| C | 3 | 3 | 1.49 | 9.1 | 7/9 |
| C | 3 | 4 | 1.48 | 7.2 | 7/9 |
| C | 4 | 3 | 1.13 | 16.9 | 6/9 |

Under Model B the best configuration is T=3/S=3 (1.88 M3, 5.8 convergence, 8/9 pass), not the V6 champion T=4/S=3. Lowering the threshold to 3 fires surges more often, partially compensating for reduced per-slot S/A precision. Under Model C, T=3/S=3 reaches 1.49 M3 with 7/9 pass. No parameter combination achieves M3 >= 2.0 under Model B.

## Draft Traces (Model B, Moderate)

**Trace 1 (committed, Self-Discard):** Early picks are mixed. Commits pick 4. Stone surges fire frequently and deliver Stone-primary cards, but ~25% are B/C for Self-Discard (e.g., Self-Mill cards rated C). Picks 10+ show surge packs with 2-3 Stone cards but only 1-2 are S/A. The player accumulates B-tier filler.

**Trace 2 (power-chaser):** No commitment. Picks highest power across all resonances. Token counters oscillate between resonances without strong direction. Surges fire for different resonances across the draft. No archetype concentration.

**Trace 3 (signal-reader, Ramp):** Zephyr surges fire early (picks 1-5) delivering Flash/Ramp cards. After pick 6 the reader commits to Ramp. Mid-draft shows Tide surges (secondary resonance) delivering B-tier cards. Late draft shows mixed Zephyr/Tide surges. Several picks show all-B packs where every resonance-matched card is from the sibling archetype but rated B under Model B.

## Self-Assessment

**Model A (Optimistic): PASS (9/9).** Matches V6 expectations. M3=2.03 is just above threshold, confirming V6 findings.

**Model B (Moderate): FAIL (7/9).** Fails M3 (1.43 vs 2.0 target) and M5 (10.8 vs 5-8 range). The core problem: resonance-matched slots are only 74.4% S/A, so surge packs deliver ~2.2 resonance-matched S/A cards instead of ~3.0. With the random slot adding ~0.2, total is ~1.4-1.5 per surge pack. Normal packs contribute ~0.9 S/A. The blended result falls well below 2.0. M5 fails because convergence requires 3 consecutive packs with 2+ S/A, which is much harder when the expected value per pack is only 1.4.

**Model C (Pessimistic): FAIL (6/9).** Additionally fails M6 (58.6% vs 60% floor). At 61.6% S/A precision, surge packs barely outperform random packs. The algorithm provides almost no archetype-level assistance. Convergence is essentially broken (pick 17.9).

**Conclusion:** Surge Packs V6 is a strong algorithm under optimistic assumptions but degrades rapidly under realistic fitness. The degradation is structural: the algorithm targets resonance, not archetype, and every reduction in cross-archetype fitness directly reduces S/A delivery. V7 algorithms must either achieve archetype-level targeting or provide enough surplus resonance-level S/A to absorb the fitness penalty. Under Model B, the best Surge Packs variant (T=3/S=3) reaches 1.88 M3 -- close but not sufficient. No parameter tuning alone can bridge the gap to 2.0 under realistic fitness.
