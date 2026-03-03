# Comparison 2: Agent 2 (Probabilistic Slot Targeting)

## Scorecard (Strategy × Goal, 1-10)

| Goal | D1 PairThresh | D2 PairEsc | D3 PoolSeed | D4 DualThresh | D5 HybridTrig |
|------|:---:|:---:|:---:|:---:|:---:|
| 1. Simple | 8 | 6 | 7 | 9 | 7 |
| 2. No actions | 10 | 10 | 10 | 10 | 10 |
| 3. Not on rails | 8 | 5 | 9 | 4 | 9 |
| 4. No forced decks | 7 | 5 | 9 | 6 | 8 |
| 5. Flexible archetypes | 7 | 5 | 7 | 5 | 8 |
| 6. Convergent | 2 | 10 | 2 | 7 | 3 |
| 7. Splashable | 8 | 5 | 8 | 7 | 8 |
| 8. Open early | 9 | 7 | 9 | 8 | 9 |
| 9. Signal reading | 2 | 3 | 8 | 3 | 3 |

**Justifications (1 sentence each):**

- D1 Simple=8: Threshold/reset/bonus cycle is concrete and unambiguous.
- D2 Simple=6: Probability formula min(count/K, cap) is precise but requires math reasoning.
- D3 Simple=7: One operation, but invisible pool changes are hard for players to observe.
- D4 Simple=9: Binary thresholds with deterministic outcomes; most transparent algorithm.
- D5 Simple=7: Two interacting concepts (resonance trigger + pair bonus) add mild complexity.
- D1 Convergent=2: 1.10 S/A; bonus injection structurally capped at ~1.3 regardless of tuning.
- D2 Convergent=10: 3.00 S/A at cap=0.65; 2.61 at cap=0.50; best convergence by large margin.
- D3 Convergent=2: 1.18 S/A; pool bloat dilution creates structural ceiling at ~1.2.
- D4 Convergent=7: 2.22 S/A passes 2.0 but convergence pick 11.9 is far outside 5-8 target.
- D5 Convergent=3: 1.52 standalone; D4 hybrid reaches 2.10 but pure D4 (2/5) at 2.48 is better.
- D3 Signal=8: Only algorithm where pool composition affects outcomes; signal readers benefit most.

## Biggest Strength / Weakness per Strategy

| Strategy | Biggest Strength | Biggest Weakness |
|----------|-----------------|-----------------|
| D1 PairThresh | Clearest one-sentence description | Structural S/A ceiling at ~1.3 |
| D2 PairEsc | Highest convergence (3.00 S/A) | Over-convergence: 97% deck conc. at cap=0.65 |
| D3 PoolSeed | Only algorithm with signal reading | Pool bloat ceiling caps S/A at ~1.2 |
| D4 DualThresh | Most transparent to players | Convergence pick 11.9, far too slow |
| D5 HybridTrig | Most organic variance (pack-dependent) | 1.52 S/A standalone is inadequate |

## Proposed Improvements

**D1:** Cannot cross 2.0 standalone. Best as a bonus layer: D2 (cap=0.40) + D1 (T=2, B=1) could yield ~2.2 base + ~0.3 bonus = ~2.5 with variable pack size creating organic variance.

**D2:** Reduce cap from 0.65 to 0.50. This yields 2.61 S/A (still well above 2.0), improves off-archetype from 0.51 to 0.71, and reduces deck concentration from 97% toward ~85%. The sweet spot balancing convergence, splash, and diversity.

**D3:** Increase injection rate to 6-8 per pick with off-resonance removal (2-3 per pick). Net pool growth is smaller, density shift is larger. Still won't cross 2.0 but would improve from 1.18 to ~1.4. Primary value remains as invisible complement to D2/D4.

**D4:** Lower thresholds to (2/5) for convergence pick 9.2 and 2.48 S/A. Consider counting 1-symbol cards at 0.5 pair contribution to accelerate accumulation. The fundamental bottleneck is that only 2+ symbol picks contribute pairs, making early convergence impossible.

**D5:** Abandon standalone; D4 hybrid (2.10) is already outperformed by pure D4 at (2/5). The conditional trigger's variance contribution (~1.21 stddev) is valuable but costs net S/A.

## V3/V4 Comparison

D2 at cap=0.50 strictly dominates both Lane Locking and Pack Widening:

| Metric | D2 cap=0.50 | Lane Lock | Pack Widen (auto) |
|--------|:---:|:---:|:---:|
| Late S/A | **2.61** | 2.34 | 1.96 |
| Conv pick | **~6-7** | 6.6 | 9.5 |
| StdDev | 0.97 | 0.96 | **1.08** |
| Off-arch | **0.71** | 0.70 | 1.50 |
| Player decisions | None | None | Spending |
| Permanent locks | No | Yes | No |

The zero-decision interface is worth the tradeoff. Pack Widening's original advantage (player-controlled spending) is irrelevant in V5's no-decisions framework. Lane Locking's permanent locks feel mechanical; D2's probabilities create natural variance around the same convergence level. D2 achieves higher S/A than Lane Locking because pair matching (100% precision) outperforms single-resonance matching (~50%).

## The Hybrid Question: Best Possible Algorithm

**Recommended: Pair-Escalation Slots at cap=0.50, K=6.**

One-sentence: "Track the resonance pair (first, second symbol) of each 2+ symbol card you draft; each pack slot independently shows a card matching your most common pair with probability min(that pair's count / 6, 0.50), otherwise a random card."

This achieves 2.61 S/A with 0.97 stddev, 0.71 off-archetype, convergence ~pick 6-7, and ~85% deck concentration. It requires zero player decisions, has no permanent state transitions, and naturally varies pack quality through per-slot coin flips.

Adding D3 pool seeding as a second layer would improve signal reading (Goal 9, currently scored 3) and boost random-slot quality. The combined description: "Each slot has up to 50% chance of being pair-matched; additionally, 4 pair-matched cards are added to the pool after each pick." Two sentences, two mechanisms — acceptable if signal reading matters.

## The Pair-Matching Question

Pair matching broke through the archetype dilution ceiling — but only for slot-replacement mechanisms. D2 (pair slots) achieved 3.00 vs Lane Locking's 2.34 (single-resonance slots). That's a 28% improvement attributable to pair precision.

For non-replacement mechanisms (D1, D3, D5), pair matching improved precision but didn't overcome the structural limitation that adding cards to packs or shifting pool density can't override the 4-card baseline. The ceiling moved from ~1.7 (V4, single resonance) to ~1.5 (V5, pairs) — a puzzling regression explained by the fact that pair-matched pools are smaller (40 cards per archetype pair vs 90 per resonance), making bonus card draws more repetitive.

**Bottom line:** Pair matching is transformative for slot targeting, modest for everything else. The V5 recommendation should be pair-based slot targeting.
