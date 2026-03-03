# V5 Final Pool Specification: Pair-Escalation Slots

## Complete Pool Specification

**360 cards total.** 8 archetypes on a resonance circle (Ember, Stone, Tide, Zephyr).

### Card Counts

| Category | Count | Details |
|----------|-------|---------|
| Archetype cards | 276 | ~34-35 per archetype (34x4, 35x4) |
| Bridge cards | 48 | 6 per adjacent pair, alternating ownership |
| Generic cards | 36 | No symbols, B-tier for all archetypes |

### Per-Archetype Breakdown (~34 cards each)

| Symbol Count | % | Cards/Arch | Pattern |
|---|---|---|---|
| 1-symbol | 15% | 5 | 70% [P], 30% [S] |
| 2-symbol | 60% | 21 | 85% [P,S], 10% [P,P], 5% [S,P] |
| 3-symbol | 25% | 8-9 | 100% [P,S,X] where X is any resonance |

### Bridge Cards (48 total)

6 bridge cards per adjacent archetype pair. Each bridge is S-tier for both adjacent archetypes. Ownership alternates: 3 cards use archetype A's ordered pair, 3 use archetype B's. This adds ~3 cards to each archetype's pair-matched pool.

| Adjacent Pair | Shared Resonance | Bridge Cards |
|---|---|---|
| Flash/Blink | Ember+Zephyr | 6 |
| Blink/Storm | Ember | 6 |
| Storm/Self-Discard | Ember+Stone | 6 |
| Self-Discard/Self-Mill | Stone | 6 |
| Self-Mill/Sacrifice | Stone+Tide | 6 |
| Sacrifice/Warriors | Tide | 6 |
| Warriors/Ramp | Tide+Zephyr | 6 |
| Ramp/Flash | Zephyr | 6 |

### Rarity Distribution (Standard TCG)

| Rarity | Count | Power Range |
|---|---|---|
| Common | 180 | 3-6 |
| Uncommon | 100 | 4-7 |
| Rare | 60 | 5-8 |
| Legendary | 20 | 6-9 |

Rarity is orthogonal to symbol count. No correlation between rarity and number of symbols.

### Algorithm Parameters

| Parameter | Value |
|---|---|
| K (divisor) | 10 |
| C (cap) | 0.65 |
| Pack size | 4 |
| Picks per draft | 30 |

---

## Agent Finding Incorporation

**Agent 1 (Symbol Distribution): Adopted.** 15/60/25 validated as sitting in a robust plateau (10-20% 1-sym, 50-65% 2-sym, 20-30% 3-sym all equivalent). Keeping 1-symbol cards below 40% is the critical constraint; 15% provides design flexibility with negligible stall risk.

**Agent 2 (Rarity): Adopted.** Standard TCG rarity with no symbol correlation. Rarity is a feel lever, not a balance lever -- convergence varies only 0.04 S/A across all rarity models. Inverse correlation (rares with fewer symbols) was interesting for tension but not needed given the algorithm already creates strong convergence.

**Agent 3 (Archetypes): Adopted with modification.** Equal archetype sizes with 10% generic and 48 bridge cards. Bridge cards increase late S/A from 1.79 to 1.82 at K=6/C=0.50 and support dual-archetype strategies. The archetype card count drops from 40 to ~34 per archetype to make room for bridges, but the per-archetype pair pool actually grows because of the high-[P,S] pattern composition (from Agent 4).

**Agent 4 (Pattern Composition): Adopted -- the highest-impact finding.** 85% [P,S] for 2-symbol cards ensures the vast majority of drafted cards produce the home archetype's ordered pair. All 3-symbol cards start with [P,S,...] to guarantee home pair contribution regardless of the third symbol. This single change increased the average pair-matched pool from 26.4 cards (mixed patterns) to 33.5 cards (high-PS), driving a +0.45 S/A improvement when combined with Agent 5's parameters. The 10% [P,P] and 5% [S,P] minority provides pattern variety without degrading convergence.

**Agent 5 (Parameters): Adopted.** K=10, C=0.65, Pack Size=4. K=10 creates a pronounced three-act arc (2.8x exploration-to-refinement ratio vs 1.4x at K=6). C=0.65 raises the S/A ceiling to 2.88 while maintaining 0.59 off-archetype cards per pack (above the 0.5 target). Pack size 4 avoids the over-convergence seen at pack size 5.

---

## Tension Resolution

### Pattern Composition + Parameter Tuning

Agent 4 achieved 2.61 S/A at K=6/C=0.50 with all-[P,S]. Agent 5 achieved 2.84 S/A at K=10/C=0.65 with mixed patterns. **Combined:** high-[P,S] patterns with K=10/C=0.65 achieves 2.88 S/A -- the two optimizations are additive. High-PS accelerates pair accumulation (pairs reach ~3 by pick 5), while K=10 prevents this from hitting the cap too early (P=0.23 at pick 5 instead of P=0.37), preserving the exploration phase. C=0.65 then allows the refinement phase to deliver 2.6 expected targeted slots per pack.

### Agent 3's Low S/A vs Agent 4's High S/A

Agent 3 reported 1.82 S/A with bridges at K=6/C=0.50, while Agent 4 got 2.61 with all-[P,S] at the same parameters. The gap was entirely due to pattern composition: Agent 3 used mixed patterns where ~50% of pair-producing picks fed the wrong pair. The reconciled design uses Agent 3's bridge structure with Agent 4's high-[P,S] patterns, resolving this tension completely. Config B in the simulation confirms this: same pool, K=6/C=0.50 yields 2.59 S/A -- far above Agent 3's 1.82.

### Deck Concentration

All agents reported 93-97% deck concentration vs the 60-90% target. This persists in the reconciled design (96.4%). This is structural to Pair-Escalation: pair-matched cards are 100% S/A-tier, and committed players always pick S/A when available. The target should be revised to 90-98% for pair-based algorithms, or addressed through fitness model changes (widening the gap between S and A tiers, adding more B-tier cards to pair pools).

---

## Before/After Comparison

| Metric | Baseline (K=6, C=0.50, mixed) | Reconciled (K=10, C=0.65, high-PS) | Delta |
|--------|:---:|:---:|---|
| Late S/A | 2.43 | **2.88** | +0.45 |
| Convergence Pick | 6.1 | **5.6** | -0.5 faster |
| Off-Arch C/F | 0.73 | 0.59 | -0.14 (still passes) |
| S/A StdDev | 1.07 | 0.98 | -0.09 (still passes) |
| Early Diversity | 6.3 | **6.5** | +0.2 |
| Early S/A | 1.75 | **1.66** | -0.09 (lower = better exploration) |
| Overlap | 13.4% | 17.3% | +3.9pp (still well below 40%) |
| Three-Act Ratio | 1.4x | **2.8x** | +1.4x more dramatic arc |
| Targets Passed | 7/8 | 7/8 | Same (deck conc only failure) |
| Arch Conv Range | -- | 1.4 picks | Excellent balance |

**Power chaser penalty:** 2.88 S/A for committed vs 1.45 for power chasers -- a 1.43 S/A gap that strongly rewards archetype commitment.

**Signal reader viability:** 2.73 S/A with convergence at pick 7.1, confirming the algorithm supports flexible early play.

---

## Pair Economy Analysis

### Average Top Pair Count at Key Picks

| Pick | Pairs Accumulated | P (Probability) | Expected Targeted Slots |
|---|---|---|---|
| 1 | 0.0 | 0.000 | 0.00 |
| 5 | 2.9 | 0.230 | 0.92 |
| 8 | 5.1 | 0.428 | 1.71 |
| 10 | 6.7 | 0.542 | 2.17 |
| 15 | 11.0 | 0.636 | 2.54 |
| 20 | 15.5 | 0.648 | 2.59 |
| 30 | 24.4 | 0.650 | 2.60 |

### Probability Escalation Curve

The draft naturally divides into three acts:

- **Exploration (picks 1-5):** P averages 0.23. Packs are mostly random (~0.9 targeted slots). Players see 6.5 unique archetypes per pack. Low early S/A (1.66) preserves genuine exploration.
- **Commitment (picks 6-15):** P ramps from 0.29 to 0.64. Targeted slots rise from 1.2 to 2.5. The player's archetype crystallizes as pair-matched cards dominate. Convergence fires at pick 5.6 on average.
- **Refinement (picks 16-30):** P saturates at 0.65 (the cap). Steady 2.6 targeted slots + 1.4 random slots per pack. Late S/A averages 3.0 with stddev 0.98, providing consistent convergence with meaningful pack-to-pack variance.

### Per-Archetype Convergence

| Archetype | Conv Pick | Late S/A |
|---|---|---|
| Flash | 4.9 | 2.96 |
| Self-Discard | 5.0 | 2.96 |
| Storm | 5.5 | 2.87 |
| Self-Mill | 5.5 | 2.93 |
| Warriors | 5.6 | 2.84 |
| Ramp | 5.6 | 2.92 |
| Sacrifice | 6.0 | 2.86 |
| Blink | 6.3 | 2.84 |
| **Average** | **5.6** | **2.90** |

Range: 1.4 picks -- all archetypes are viable and balanced.

---

## Open Questions for Playtesting

1. **Deck concentration at 96%.** Is this a problem in practice? Committed players rarely draft off-archetype cards. The fitness model may need a wider S-A-B-C gap, or the pool needs more B-tier cards in pair pools, to bring this into the 60-90% range.

2. **Off-archetype visibility (0.59).** This passes the 0.5 target but is lower than the baseline's 0.73. Do players feel they can splash effectively? If not, C=0.55 would increase off-archetype cards at the cost of ~0.2 S/A.

3. **Overlap at 17%.** Very low run-to-run overlap but slightly higher than baseline (13%). With 33-39 cards per pair pool, the same cards reappear more often. Does this feel repetitive across runs of the same archetype?

4. **Bridge card design.** 48 bridge cards is a significant design commitment. Each must be compelling for two adjacent archetypes. Does this constrain card design too much?

5. **K=10 early game.** P=0 at pick 1 and P=0.23 at pick 5 means the first 5 packs are nearly random. Does this exploration phase feel engaging or aimless?

6. **3-symbol card third slot.** All 3-symbol cards must start [P,S,...] for pair accumulation. The third symbol is free design space. Does this constraint leave enough creative room?
