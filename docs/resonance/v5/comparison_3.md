# Comparison 3: Pool Evolution Agent — Round 4 Cross-Comparison

## Scorecard (Strategy x Goal, 1-10)

| Goal | D1: Pair Threshold | D2: Pair-Escalation | D3: Pool Seeding | D4: Dual-Threshold | D5: Hybrid Trigger |
|------|:---:|:---:|:---:|:---:|:---:|
| 1. Simple | 8 | 5 | 8 | 9 | 7 |
| 2. No extra actions | 10 | 10 | 10 | 10 | 10 |
| 3. Not on rails | 8 | 4 | 9 | 5 | 9 |
| 4. No forced decks | 7 | 5 | 9 | 6 | 8 |
| 5. Flexible archetypes | 7 | 4 | 7 | 5 | 8 |
| 6. Convergent (>=2.0) | 2 | 10 | 2 | 7 | 3 |
| 7. Splashable | 8 | 4 | 8 | 7 | 8 |
| 8. Open early | 9 | 7 | 9 | 8 | 9 |
| 9. Signal reading | 2 | 2 | 8 | 3 | 3 |
| **Total** | **61** | **51** | **70** | **60** | **65** |

**Key scoring justifications:**

- **D2 Convergent = 10:** 3.00 S/A is the only result that comfortably exceeds 2.0. But Simple = 5 because `min(count/6, 0.65)` requires formula reasoning. Not-on-rails = 4 and Splashable = 4 because 97% deck concentration and 0.51 off-archetype are near-failures.
- **D4 Convergent = 7:** 2.22 S/A passes 2.0 but convergence pick 11.9 fails the 5-8 target. The (2/5) variant at 2.48 S/A with convergence 9.2 would score 8.
- **D3 Signal reading = 8:** Pool seeding is the only mechanism that interacts with pool composition, making it the only one where reading what's available in the pool matters. All other mechanisms are self-referential (based only on what you've drafted).
- **D5 Convergent = 3:** Standalone 1.52 fails, but D4+D5 hybrid at 2.10 demonstrates value as a layer.
- **Totals are misleading.** D3 scores highest but fails the hardest constraint (convergence). Convergence is a pass/fail gate. D2 and D4 are the only standalones that pass.

## Biggest Strength / Weakness Per Strategy

| Strategy | Biggest Strength | Biggest Weakness |
|----------|-----------------|------------------|
| D1 | Cleanest one-sentence description; most honest parameter sweep | 1.10 S/A — structurally capped at ~1.4 even with aggressive tuning |
| D2 | Raw convergence power (3.00 S/A) | Over-convergence: 97% deck concentration, 0.51 splash — feels like a vending machine |
| D3 | Most natural feel; only mechanism supporting signal reading; best variety (5.7% overlap) | Pool bloat ceiling caps S/A at ~1.2 — cannot anchor a system alone |
| D4 | Best balance across all 9 goals; only strategy passing 8/9 targets | Convergence pick 11.9 — pair counting inherently slower than symbol counting |
| D5 | Best variance (1.23 stddev); most organic pack-to-pack feel | Standalone 1.52 S/A — only viable as a hybrid component |

## Proposed Improvements

**D1:** Use as bonus layer atop D4. At T=2, B=1 with D4's guaranteed slots: projects ~2.6 S/A with intermittent bonuses adding variance.

**D2:** Lower cap to 0.50 (2.61 S/A, 0.71 off-archetype). Still passes convergence but fixes the overcorrection. Alternatively, simplify the one-sentence description — "each slot has a coin-flip chance of being pair-targeted once you've drafted 4+ matching pairs."

**D3:** Cannot standalone. Best role: complement enriching random slots in D4 or D2. Pool seeding rate 3 adds ~+0.12 S/A to random slots while providing signal reading.

**D4:** Lower thresholds to (2/5). This accelerates convergence from 11.9 to ~9.2 while maintaining 2.48 S/A. Count 1-symbol cards as 0.5 toward the top pair's primary resonance to further speed accumulation.

**D5:** The D4+D5 hybrid (2.10 S/A, 1.21 stddev) is D5's best form. The conditional trigger adds organic variance that D4 alone lacks. But simplify: at 64% fire rate, the conditionality is almost vestigial.

## V3/V4 Comparison

Lane Locking baseline varies across simulations (1.74-2.61 S/A) due to different pool constructions. The Round 5 synthesis must normalize this. Qualitative comparison:

**D4 (2/5) vs Lane Locking:** Pair matching achieves comparable convergence (2.48 vs ~2.4) with 2 guaranteed slots vs Lane Locking's 2-3 locked slots — because each pair-matched slot is ~100% archetype-precise vs ~50% for resonance-matched. D4 has better variance (0.82 vs 0.74), better deck concentration (80% vs 86%), but slower convergence (9.2 vs 6.6). The pair-matching breakthrough is confirmed.

**All V5 vs Pack Widening auto-spend:** Pack Widening auto-spend consistently underperforms (0.80-1.96 S/A), confirming single-resonance dilution is fatal. V5's pair-based approach is a clear generation-over-generation improvement.

**Verdict:** V5 D4 is a meaningful improvement over Lane Locking on balance (better variance, flexibility, precision) with an acceptable tradeoff on speed. It clearly beats Pack Widening auto-spend. The zero-decision interface is preserved.

## Best Possible Hybrid

**"Pair-Threshold Guarantee with Pool Enrichment"**

One-sentence: "Track your most-drafted ordered resonance pair; at 2 matching drafts one pack slot is pair-guaranteed, at 5 a second slot is pair-guaranteed; after each pick, 3 cards matching your top pair are added to the pool from a reserve."

This combines D4 (2/5) guaranteed slots with D3 pool seeding (rate 3). The two mechanisms are independent: guaranteed slots provide the convergence floor (2 x 0.95 = 1.90 S/A from targeted slots), pool seeding enriches the 2 random slots (pushing from 0.22 to ~0.28 each = 0.56 total), and together they project ~2.46 S/A.

Why this hybrid over D2: Comparable S/A with clearer mental model, better splash, and signal reading. Why over D4+D5: Simpler (two independent rules vs two interacting mechanisms), and pool seeding adds the unique signal-reading capability no other mechanism provides.

## Pair-Matching Analysis

Pair matching is V5's breakthrough. All 5 agents used ordered pairs, and the results validate the hypothesis: pair precision is 88-100% S-tier for the target archetype, compared to ~50% for single-resonance matching. This means each pair-targeted slot delivers roughly 2x the archetype value of a resonance-targeted slot. D4 needs only 2 guaranteed slots to match Lane Locking's 2-3, and D2 achieves 3.00 S/A (vs Lane Locking's ~2.4) using the same per-slot probability mechanism with pair matching instead of resonance matching.

The pair-matching insight is the single most valuable finding across V3-V5: it eliminates the archetype dilution ceiling that capped all V4 probabilistic approaches at ~1.7 S/A.
