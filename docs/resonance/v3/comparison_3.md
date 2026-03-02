# Cross-Comparison — Agent 3 (Lane Locking / Threshold Domain)

## Scorecard (1-10, based on simulation data)

| Goal | S1: Weighted Lottery | S2: Balanced Pack | S3: Lane Locking | S4: Echo Window | S5: Resonance Swap |
|------|---------------------|-------------------|------------------|-----------------|-------------------|
| 1. Simple | 7 | **9** | 8 | 8 | 5 |
| 2. Not on rails | 7 | 7 | 5 | **8** | **8** |
| 3. No forced decks | 7 | 7 | 7 | 7 | **9** |
| 4. Flexible archetypes | 7 | 6 | 6 | **8** | 7 |
| 5. Convergent | **8** | **8** | 6 | 7 | 4 |
| 6. Splashable | 8 | **9** | 7 | 5 | 8 |
| 7. Open early | 7 | **9** | 8 | 5 | 8 |
| 8. Signal reading | 4 | 3 | 4 | 2 | **9** |
| **Total** | **55** | **58** | **51** | **50** | **58** |

### Scoring Justifications

**S1 Weighted Lottery:** Simplicity docked to 7 — the probabilistic nature means players cannot predict exact pack contents, only distributions. Convergence strong (2.31 late fit, pick 6.4). Starting weight is hidden complexity.

**S2 Balanced Pack:** Top simplicity — two pack states (1/1/1/1 or 2/1/1/0) are instantly comprehensible. Best early openness (3.72 unique res). Convergence hits 2.08 target. Flexible archetypes slightly weak because majority bonus supports only one resonance, not a pair. Signal reading essentially absent.

**S3 Lane Locking:** Simplicity is good (binary lock states) but the primary=2 weighting adds hidden math. Core failure: late fitness at 1.83 misses the 2.0 target because locked slots guarantee resonance, not archetype fit. Permanent locks cripple late pivots (not-on-rails = 5).

**S4 Echo Window:** Best flexibility and pivot capability from 3-pick memory. But early bias is worst of all (2.58 early fit vs 2.0 cap), and splash misses target (0.43 vs 0.5). The algorithm converges too fast early and too hard late. Signal reading impossible by design.

**S5 Resonance Swap:** Uniquely strong signal reading (44.8% detection). Best run-to-run variety (6.5% overlap). But convergence is structurally unfixable at 1.61 — the 360-card pool is too large for 3-card swaps to shift meaningfully. One-sentence description requires hidden infrastructure (reserve pool).

## Biggest Strength & Weakness per Strategy

| Strategy | Biggest Strength | Biggest Weakness |
|----------|-----------------|------------------|
| S1: Weighted Lottery | Smooth, gradual convergence (2.31 fit, pick 6.4) | Probabilistic = unpredictable pack contents |
| S2: Balanced Pack | Perfect early diversity (3.72 res, 1/1/1/1 baseline) | No signal reading mechanism (3/10) |
| S3: Lane Locking | Perfect transparency (binary lock state always known) | Permanent locks kill pivoting (not-on-rails = 5) |
| S4: Echo Window | Instant pivotability (3-pick memory) | Early over-convergence (2.58 fit in picks 1-5) |
| S5: Resonance Swap | Signal reading (44.8% detection, unique mechanic) | Convergence structurally unfixable (1.61 fit) |

## Proposed Improvements

**S1:** Increase starting weight to 3 (fixes early-fit to 1.73). Make weights visible as a UI element.

**S2:** Add a threshold before majority activates (e.g., 5+ symbols ahead) to slow feedback snowball.

**S3:** Replace permanent locks with decaying locks that reopen unless threshold is maintained. Lower second threshold from 8 to 6.

**S4:** Delay slot allocation until pick 3+ (pure random for first 2 packs). Change to 1/1/1+1 allocation to fix early bias and splash.

**S5:** Abandon standalone use — mechanism belongs as a complementary layer atop a primary convergence engine.

## Proposed Best Algorithm

**Balanced Pack with Static Pool Asymmetry**

> "Each pack shows one card per resonance type; if you have a clear majority resonance (primary symbols count double), it replaces one non-majority slot, giving you 2 of your majority. Each quest starts with one random resonance having extra cards in the pool."

This hybrid draws from:
- **S2's pack structure** as the primary mechanism — it has the best target pass rate (7/8), the clearest simplicity, and the strongest early diversity
- **S5's pool asymmetry** (simplified to a static per-run boost) — adds the only signal-reading mechanism any strategy achieved, without ongoing swap complexity

Why this dominates: S2 already passes convergence (2.08), splash (1.92), early openness (3.72), and variety (5.8% overlap). Its only weakness is signal reading (3/10). Adding static pool asymmetry fixes this without affecting pack construction — the 1/1/1/1 or 2/1/1/0 structure is unchanged. The boosted resonance creates a detectable signal in early packs.

The deck concentration "failure" (92.8%) is shared across all strategies and likely reflects optimal play rather than an algorithm flaw.
