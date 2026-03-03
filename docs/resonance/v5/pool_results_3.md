# Agent 3: Archetype Breakdown Results

## Summary

Tested 5 archetype distribution models with Pair-Escalation Slots (K=6, C=0.50,
4 slots, 30 picks) across 1200 drafts each plus 1000 bridge-strategy drafts per
model. **Equal + Bridge Cards** performs best, followed by **Pair-Pool-Optimized**
and the equal small-generic baseline. The algorithm's 100% S-tier precision in
pair-matched pools makes pool distribution less critical than it was for
resonance-based algorithms, but bridge cards still provide a meaningful edge.

## Models Tested

| Model | Generic | Archetype Cards | Special |
|-------|---------|-----------------|---------|
| Equal + Small Generic (10%) | 36 | 40 per arch | Baseline, 15/60/25 split |
| Equal + Large Generic (25%) | 90 | 33 per arch | More generics |
| Equal + Bridge Cards | 36 | 34 per arch + 48 bridge | 6 bridge per adj pair |
| Asymmetric Sizes | 36 | 24-55 per arch | 2 large, 2 small |
| Pair-Pool-Optimized | 36 | 40 per arch, 5/70/25 split | Maximize pair pools |

## Key Findings

### 1. Pair-Matched Pools Achieve 100% S-Tier Precision

Unlike resonance-based algorithms where matching on a single resonance yields
only ~50% archetype precision, ordered pair matching achieves 100% S-tier
precision across all models. Every card in a pair-matched pool (e.g., all cards
with ordered pair [Tide, Zephyr]) belongs to exactly one archetype (Warriors).
There are zero A-tier cards in pair pools -- only S-tier. This is the key
structural advantage of Pair-Escalation: the pair-matched slots deliver
guaranteed on-archetype cards when they fire.

### 2. Bridge Cards Produce the Best Overall Performance

The Bridge model achieves the highest late S/A (1.82) and deck concentration
(75.6%). Bridge cards are S-tier for two adjacent archetypes and their ordered
pair places them in one archetype's pair pool, effectively adding 3 extra cards
per archetype's pair-matched pool (from 6 bridges per pair, alternating
ownership). Bridge strategy viability is strongest at 63.1% dual-archetype
coverage (vs 56.4% baseline). Early diversity also peaks at 5.25 unique
archetypes per pack.

### 3. Large Generic Pools Slow Pair Accumulation

The 25% generic model drops late S/A to 1.72, deck concentration to 71.0%, and
delays pair activation to pick 5.0 (vs 4.2-4.6 for others). Generics contribute
zero pairs when drafted, directly slowing escalation. Pair-slot variety drops to
24.6 unique cards (vs 28-30 for others).

### 4. Asymmetric Sizes Create Severe Balance Problems

While overall averages look comparable (1.77 late S/A), the per-archetype
convergence table reveals dramatic imbalance. Large archetypes (Warriors at 55
cards, pair pool 47) converge at pick 9.2, while small archetypes (Ramp at 24
cards, pair pool 17) converge at pick 19.9. The pair pool size directly
determines how often pair-matched slots can fire effectively -- a 17-card pool
means less variety and the same cards appearing repeatedly. For Pair-Escalation
specifically, asymmetric sizes are strongly contraindicated.

### 5. Pair-Pool-Optimized Provides Marginal Improvement

Shifting from 15/60/25 to 5/70/25 increases average pair pool from 31.4 to 35.4
and yields earliest activation (pick 4.2) and highest variety (30.5 unique
cards). But late S/A improves only 0.01 (1.80 vs 1.79). Once P saturates at
0.50 by pick 10, extra pair pool size provides variety but not more frequent
firing. The marginal gain does not justify reduced design flexibility.

### 6. Convergence Gap Remains an Algorithm Issue

All models converge around pick 14 (vs target 5-8), with late S/A at 1.72-1.82
(vs target 2.0). This gap is consistent across all 5 models, confirming it is
an algorithm-parameter issue rather than a pool-distribution issue. The K=6
divisor means P only reaches meaningful levels (>0.33) after drafting 2+ matched
pair cards, which requires picks 4-6 at minimum. The C=0.50 cap limits the
expected pair-matched slots to 2 of 4 even at maximum escalation.

### 7. Generic Dilution Is Moderate but Measurable

Each additional 54 generic cards reduces late S/A by 0.07 and delays pair
activation by 0.4 picks. The effect is smaller than for resonance-based
algorithms because Pair-Escalation only tracks pairs from 2+ symbol cards --
1-sym cards and generics are equally "silent" to the pair counter. Keep generics
at 10%.

## Target Scorecard

| Metric | Target | Best (Bridge) | Baseline (10%) |
|--------|--------|:---:|:---:|
| Early diversity | >= 3 | 5.25 PASS | 4.99 PASS |
| Early S/A | <= 2 | 1.06 PASS | 0.97 PASS |
| Late S/A | >= 2 | 1.82 FAIL | 1.79 FAIL |
| Late off-arch | >= 0.5 | 0.74 PASS | 0.78 PASS |
| Convergence pick | 5-8 | 14.2 FAIL | 14.2 FAIL |
| Deck concentration | 60-90% | 75.6% PASS | 73.6% PASS |
| Run-to-run variety | < 40% | 10.6% PASS | 11.5% PASS |
| SA StdDev (late) | >= 0.8 | 0.91 PASS | 0.90 PASS |

**6/8 targets pass for all models.** The two failures (late S/A and convergence
pick) are algorithm-parameter limitations, not pool-distribution problems.

## Recommendation

Use **equal archetype sizes** (~40 cards each) with **~10% generic cards** (36)
and **6 bridge cards per adjacent pair** (48 total, alternating pair ownership).
This provides:
- Highest late S/A (1.82) and deck concentration (75.6%)
- Strongest bridge strategy support (63.1% dual-S/A packs)
- 100% S-tier precision in all pair-matched pools (avg 32.8 cards per pool)
- Best early diversity (5.25 unique archetypes per pack)
- Balanced per-archetype convergence (13.3-16.3 range vs 9.2-19.9 for asymmetric)

If bridge cards add unwanted design complexity, the **Equal + Small Generic**
baseline is nearly as good (1.79 vs 1.82 late S/A). The remaining convergence
gap (1.82 vs 2.0 target) should be addressed through algorithm parameter tuning
(lowering K or raising C) rather than pool restructuring.
