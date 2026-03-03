# Agent 4 Comparison: Pool Sculpting

## Scorecard (1-10, all 7 algorithms x 9 design goals)

| Goal | LL (1) | TAS (2) | SL (3) | PS (4) | DE (5) | RS (6) | SP (7) |
|------|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | 9 | 7 | 5 | 4 | 8 | 8 | 9 |
| 2. No actions | 10 | 10 | 10 | 10 | 10 | 10 | 10 |
| 3. Not on rails | 3 | 7 | 7 | 9 | 7 | 4 | 8 |
| 4. No forced decks | 7 | 8 | 7 | 8 | 7 | 6 | 7 |
| 5. Flexible | 3 | 6 | 6 | 8 | 5 | 4 | 7 |
| 6. Convergent | 8 | 7 | 8 | 3 | 4 | 8 | 6 |
| 7. Splashable | 4 | 8 | 5 | 3 | 7 | 7 | 8 |
| 8. Open early | 7 | 8 | 7 | 9 | 7 | 7 | 8 |
| 9. Signal reading | 3 | 6 | 5 | 6 | 5 | 3 | 5 |
| **Total** | **54** | **67** | **60** | **60** | **60** | **57** | **68** |

**Where I disagree with other agents:**

- PS flexibility 8, not-on-rails 9: Packs are always 4 random draws from a shifting pool. No locked slots, no surge states, no triggered bonuses. The player feels choice, not mechanism.
- SP simple 9: "Earn tokens, spend tokens, fill slots" is as clean as Lane Locking.

## Biggest Strength and Weakness Per Strategy

| Algo | Biggest Strength | Biggest Weakness |
|------|-----------------|------------------|
| LL (1) | Absolute convergence certainty: 2 slots x 100% S/A = 2.0 minimum floor | Draft is functionally decided by pick 3-4; remaining 26 picks are autopilot |
| TAS (2) | 9/9 metric passage means no exploitable weakness | 16.4% of post-commitment packs have 0 S/A -- worse than Lane Locking's 0.1% |
| SL (3) | The 75% miss rate creates organic uncertainty that hard locks cannot | 97% deck concentration makes committed play feel samey |
| PS (4) | The only algorithm where the mechanism is completely invisible to the player | Structural 2.0 ceiling: my best variant hits 2.01 after exhaustive optimization |
| DE (5) | The pack-inspects-itself trigger is the most creative mechanic in V6 | No parameter sweet spot: either too rare (thresh=2: 21%) or too frequent (thresh=1: 63%) |
| RS (6) | Clean 3-act narrative: explore, commit, specialize | 75% of pack determined by locks makes late-draft picks feel hollow |
| SP (7) | Surge/normal rhythm creates anticipation and surprise | 25.5% zero-S/A packs post-commitment are a serious player experience problem |

## Proposed Improvements

- **LL (1):** Delay both thresholds by 2 (threshold 5/10). Convergence moves from pick 3.3 to ~pick 5-6, fixing the too-fast problem. S/A should increase slightly as later locks have more accumulated resonance.
- **TAS (2):** The 16% zero-S/A pack rate is the hidden problem. Consider a "floor" mechanic: if no bonus fires AND no random card matches top resonance, replace one random card with a resonance-matched one. This adds complexity but eliminates zero-S/A packs.
- **SL (3):** The 75% probability is smart but concentration is unsalvageable within this framework. Consider reducing to 2 soft-lock slots + 1 forced-splash slot + 1 random slot.
- **PS (4) -- self-improvement:** My algorithm is the honest failure of V6. The V4 structural finding is confirmed: probabilistic pool manipulation caps at ~2.0 S/A. Pool Sculpting should be retained only as a SUPPLEMENTARY LAYER under a slot-locking or injection algorithm. Pool evolution + 1 locked slot at threshold 5 would combine organic variance with a convergence floor.
- **RS (6):** Add a "lock decay" mechanic: each lock has a 10% chance per pick of unlocking and re-locking to the current top resonance. This addresses permanence without losing convergence.
- **SP (7):** The bimodal distribution is fixable: when NOT surging, apply a mild pool bias (draw from top-resonance cards with 25% probability on 1 slot). This fills the valleys.

## Baseline Comparison

**Does any V6 algorithm clearly beat both baselines?**

No algorithm clearly beats both, but Threshold Auto-Spend comes closest by passing 9/9 metrics. The problem is that "clearly beat" requires superior convergence (which needs 2.5+ S/A to be unambiguous) AND superior variance AND zero decisions. TAS achieves zero decisions and balanced variance but mediocre convergence.

I believe Surge Packs (T=4/S=3) is the most promising because its non-permanent state tracking, good simplicity, and rhythmic pacing create the best player experience even though its 2.08 S/A is modest. The bimodal distribution is a solvable problem.

## Proposed Best Algorithm

**Surge Packs with valley-fill: Pool Sculpting as background layer.**

"Each drafted symbol adds tokens (+2 primary, +1 others); when any counter reaches 4, spend 4 and fill 3 slots with that resonance's cards, fourth slot random. Between surges, the pool gradually shifts toward your top resonance (replace 6 off-resonance cards per pick)."

This hybrid combines Surge's rhythmic structure with Pool Sculpting's gradual background bias. Projected: surge packs deliver 2.25+ S/A; non-surge packs deliver ~1.3 S/A instead of 1.0 (from pool evolution). Blended: ~2.0 S/A with lower bimodality (stddev ~1.1).

The tradeoff is complexity: this is genuinely two mechanisms, though both share the same resonance-tracking state.

## 15% Dual-Resonance Constraint Impact

The constraint made the problem **harder for pool-based approaches and neutral for placement-based approaches**. Pool Sculpting relies on increasing density of target-resonance cards. With only 15% dual-type, the pool is dominated by mono-resonance cards serving 2 archetypes equally, capping archetype precision at ~50%. Slot-locking algorithms are unaffected because resonance-primary pools deliver 100% S/A due to the tier structure.

At 10%: Pool Sculpting drops to ~1.9 S/A. No impact on other algorithms.
At 20%: Pool Sculpting rises to ~2.1 S/A. Still insufficient to close the gap with slot-locking.

The deeper lesson: pool manipulation is structurally inferior to deterministic placement regardless of dual-type percentage.
