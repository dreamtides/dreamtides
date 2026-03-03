# Agent 5 Comparison: Double Enhancement

## Scorecard (1-10, all 7 algorithms x 9 design goals)

| Goal | LL (1) | TAS (2) | SL (3) | PS (4) | DE (5) | RS (6) | SP (7) |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | 9 | 7 | 5 | 4 | 8 | 8 | 9 |
| 2. No actions | 10 | 10 | 10 | 10 | 10 | 10 | 10 |
| 3. Not on rails | 2 | 7 | 7 | 9 | 7 | 3 | 8 |
| 4. No forced decks | 7 | 8 | 7 | 8 | 7 | 6 | 7 |
| 5. Flexible | 2 | 6 | 6 | 8 | 5 | 3 | 7 |
| 6. Convergent | 8 | 6 | 8 | 4 | 5 | 9 | 6 |
| 7. Splashable | 4 | 8 | 5 | 3 | 7 | 7 | 8 |
| 8. Open early | 6 | 8 | 7 | 9 | 7 | 6 | 8 |
| 9. Signal reading | 3 | 6 | 5 | 5 | 5 | 3 | 5 |
| **Total** | **51** | **66** | **60** | **60** | **61** | **55** | **68** |

**Scoring philosophy -- I penalize railroading more than others:**

- LL rails 2, flexibility 2: Permanent locks at pick 3-4 eliminate meaningful choice for 85% of the draft.
- RS convergent 9: 2.20 S/A at pick 6.7 is exactly what Goal 6 asks for.
- DE convergent 5: Champion fails at 1.32; thresh=1 variant is just additive injection in disguise.
- SP convergent 6: 2.08 mean S/A is inflated by surge packs; median is ~1.5.

## Biggest Strength and Weakness Per Strategy

| Algo | Biggest Strength | Biggest Weakness |
|------|-----------------|------------------|
| LL (1) | The 100% S/A locked slot finding eliminates theoretical convergence risk | 0.49 stddev is the lowest variance of any algorithm -- antithetical to Goal 3 |
| TAS (2) | 9/9 metric passage; best C/F splash at 1.45 | 2.01 S/A is not statistically distinguishable from 1.99 |
| SL (3) | Uniquely achieves 2.0+ S/A with 0.8+ stddev -- the dual target | 97% deck concentration for committed players; splash barely misses |
| PS (4) | 77.9% deck concentration is the healthiest in V6 | 0.36 C/F and 8.6 convergence pick: two hard failures |
| DE (5) | 1.55 stddev is the highest natural variance; pack-inspection is elegant | Threshold dilemma has no solution: meaningful trigger rate and 2.0 S/A are incompatible |
| RS (6) | Strongest convergence in V6 at a comfortable margin (2.20 S/A, pick 6.7) | 75% of pack locked by mid-draft; variance 0.69 fails hard |
| SP (7) | Only non-permanent convergence mechanism in V6; allows genuine pivoting | 25.5% zero-S/A post-commitment packs create frustrating droughts |

## Proposed Improvements

- **LL (1):** Cannot be improved without abandoning its core identity. Higher thresholds delay lock timing but do not change the fundamental variance problem.
- **TAS (2):** The fragility is real. C3B3 would hit ~3.5 S/A (like V4's Pack Widening) but would over-concentrate. The right fix: C4B3 for ~2.5 S/A with headroom.
- **SL (3):** The splash problem is solvable by reserving slot 4 for off-resonance cards. This is a clean one-line addition to the algorithm.
- **PS (4):** As Agent 4 acknowledged, this should be a supplementary layer, not a standalone algorithm.
- **DE (5) -- self-critique:** My algorithm has a fundamental structural flaw. The base rate of resonance matching in a 4-card pack is ~22% per card. At thresh=2, the binomial probability (21%) is too low. At thresh=1 (63%), the trigger is not meaningfully conditional. There is no parameter setting where the trigger fires often enough to cross 2.0 while remaining rare enough to feel special. I concede this approach does not work as a standalone mechanism.
- **RS (6):** The 2/4/7 threshold variant gets stddev to 0.81 while maintaining 2.34 S/A. This is strictly better than the 3/6/10 configuration.
- **SP (7):** The bimodal problem is structural to surges. A gentler version: threshold 5, surge 2 slots instead of 3, with pool sculpting filling the gaps. This reduces peak S/A but smooths the distribution.

## Baseline Comparison

**Does any V6 algorithm clearly beat both baselines?**

Against Lane Locking: TAS, SL, and SP all improve on LL's worst dimensions while matching its convergence.

Against Pack Widening: Removing player decisions costs 1.2-1.3 S/A. No zero-decision algorithm matches PW's 3.35.

My ranking: Surge Packs (best experience) or Threshold Auto-Spend (most robust metrics).

## Proposed Best Algorithm

**Surge Packs at T=4/S=3 -- the rhythmic draft.**

"Each drafted symbol adds tokens (+2 primary, +1 others); when any counter reaches 4, spend 4 and fill 3 of the next pack's 4 slots with random cards of that resonance, fourth slot random."

I champion Surge Packs despite not being its designer because it solves the two problems that kill every other algorithm:

1. **Permanence:** Surge state resets after each trigger. If the player pivots, surges follow. No other slot-based algorithm allows this.
2. **Rhythm:** The alternation between 3-slot surge packs and fully random packs creates genuine drama. Lane Locking, Ratcheting, and Soft Locks all converge to a steady state. Surge maintains tension throughout.

The bimodal distribution (25% zero-S/A packs) is the price. I believe players prefer occasional droughts with dramatic payoffs over mechanical consistency. Roguelike players are accustomed to variance.

## 15% Dual-Resonance Constraint Impact

The constraint made the problem **harder for conditional approaches and neutral for everything else**. My original Cascading Enhancement relied on dual-type cards to provide archetype-specific targeting. With only 15% dual-type, conditional triggers based on card properties cannot distinguish archetypes within a resonance.

The constraint is irrelevant for slot-locking (LL, SL, RS), injection (TAS), and surge (SP) because these algorithms operate on resonance counters, not card types. The 100% S/A finding in locked slots means mono-resonance cards already provide sufficient archetype convergence.

At 10%: My original champion would have been even weaker. No impact on other algorithms.
At 20%: Conditional Enhancement gains ~0.15 S/A from better dual-type trigger cards, still insufficient.

The constraint's main lesson: resonance-level targeting IS archetype-level targeting due to the S/A tier structure.
