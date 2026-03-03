# Agent 1 Comparison: Lane Locking (Reference Baseline)

## Scorecard (1-10, all 7 algorithms x 9 design goals)

| Goal | LL (1) | TAS (2) | SL (3) | PS (4) | DE (5) | RS (6) | SP (7) |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | **9** | 7 | 6 | 4 | 7 | 8 | 8 |
| 2. No actions | 10 | 10 | 10 | 10 | 10 | 10 | 10 |
| 3. Not on rails | 3 | 7 | 5 | 8 | 6 | 4 | 8 |
| 4. No forced decks | 7 | 8 | 7 | 8 | 7 | 6 | 7 |
| 5. Flexible | 3 | 6 | 5 | 7 | 5 | 4 | 7 |
| 6. Convergent | 8 | 7 | 8 | 4 | 5 | 8 | 6 |
| 7. Splashable | 4 | 8 | 5 | 3 | 7 | 7 | 8 |
| 8. Open early | 6 | 8 | 7 | 8 | 7 | 7 | 8 |
| 9. Signal reading | 3 | 6 | 5 | 5 | 5 | 3 | 5 |
| **Total** | **53** | **67** | **58** | **57** | **59** | **57** | **67** |

**Justifications for non-obvious scores:**

- LL convergent 8: 2.11 S/A is solid, but convergence at pick 3.3 is too fast (target 5-8), costing 1 point.
- TAS convergent 7: Passes 9/9 at C4B2 but 2.01 S/A is razor-thin; a bad seed could drop below 2.0.
- RS convergent 8: 2.20 S/A is strong but variance at 0.69 fails the 0.8 target.
- PS convergent 4: 2.01 barely crosses, pick 8.6 is outside the target window, and splash fails.
- DE convergent 5: Champion (thresh=2) fails at 1.32; thresh=1 variant hits 2.13 but fires 63%.
- SP convergent 6: T=4/S=3 hits 2.08 but the bimodal distribution means 48% of packs have 0-1 S/A.

## Biggest Strength and Weakness Per Strategy

| Algo | Biggest Strength | Biggest Weakness |
|------|-----------------|------------------|
| LL (1) | 100% S/A in locked slots; structurally guaranteed convergence | Variance 0.49, permanently on rails after pick 3-4 |
| TAS (2) | Only algorithm passing all 9 metrics simultaneously | 2.01 S/A is fragile -- one parameter nudge drops below 2.0 |
| SL (3) | 75% probability preserves genuine pack uncertainty | Splash at 0.49 borderline-fails; deck concentration 97% |
| PS (4) | Most natural feel -- packs are random draws, no mechanics visible | Structural ceiling at ~2.0; splash at 0.36 hard-fails |
| DE (5) | Highest variance (1.55 stddev) of any algorithm | Feast-or-famine bimodal: 78% of packs have 0-1 S/A at thresh=2 |
| RS (6) | Strong S/A (2.20) with smoother 3-threshold escalation | Variance 0.69 fails target; 75% of pack is locked mid-draft |
| SP (7) | Non-permanent surges allow pivoting; good rhythm | Bimodal distribution: 48% of post-commitment packs have 0-1 S/A |

## Proposed Improvements

- **TAS (2):** Raise bonus to 3 with threshold 5 to create headroom above 2.0.
- **SL (3):** Drop probability experiment; go binary with split-resonance third lock for 2.5+ S/A.
- **PS (4):** Cannot be salvaged as standalone. Layer it under slot-locking as a supplementary system.
- **DE (5):** Lock in thresh=1 as champion. The 63% fire rate is acceptable -- "conditional" drama is overrated versus reliable convergence.
- **RS (6):** Use 2/4/7 thresholds to get stddev 0.81 (meeting variance target) at cost of faster lock-in.
- **SP (7):** Reduce surge slots to 2, raise threshold to 5 for better non-surge pack quality.

## Baseline Comparison

**Does any V6 algorithm clearly beat both Lane Locking and Pack Widening?**

No. Threshold Auto-Spend (C4B2) passes 9/9 metrics where both baselines fail some, but its 2.01 S/A is well below LL's 2.11 and Pack Widening's 3.35. It wins on breadth of metric passage, not on depth. Ratcheting (2.20 S/A) matches LL's convergence strength but shares its variance and flexibility weaknesses. No algorithm delivers Pack Widening's raw S/A power without player decisions.

The honest conclusion: the zero-decision constraint costs roughly 1.0-1.3 S/A compared to Pack Widening. The best zero-decision algorithms cluster around 2.0-2.2 S/A.

## Proposed Best Algorithm

**Ratcheting Slots with split-resonance at 2/5/9 thresholds.**

"When your top resonance count reaches 2, 5, and 9, lock one more pack slot: the first two lock to your top resonance, the third locks to your second-highest; the fourth slot stays random."

This preserves Ratcheting's strong convergence (2.20+) while the lower first threshold gives faster initial feedback and the wider 5-to-9 gap creates a genuine exploration window. The split third lock provides both splash and archetype disambiguation.

## 15% Dual-Resonance Constraint Impact

The 15% constraint made the problem **harder in theory but neutral in practice**. My simulation confirmed that locked resonance slots deliver 100% S/A because primary-resonance pools contain exactly 2 archetypes that are mutual S/A. This structural property does not depend on dual-type cards at all. Dual-type cards help human players identify archetype signals, but the algorithm does not need them.

At 10%: Algorithms would perform identically. The constraint affects card design more than draft mechanics.
At 20%: A pair-matching layer becomes viable as a supplementary (not primary) system, potentially adding 0.1-0.2 S/A through better archetype targeting.
