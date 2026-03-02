# Cross-Comparison: Agent 1 (Accumulation Domain)

## Scorecard (1-10, based on simulation data)

| Goal | S1: Weighted Lottery | S2: Balanced Pack | S3: Lane Locking | S4: Echo Window | S5: Resonance Swap |
|------|-----|-----|-----|-----|-----|
| 1. Simple | **9** | **9** | **8** | **9** | **6** |
| 2. Not on rails | **7** | **7** | **5** | **8** | **8** |
| 3. No forced decks | **8** | **7** | **7** | **7** | **9** |
| 4. Flexible archetypes | **7** | **6** | **6** | **8** | **7** |
| 5. Convergent | **8** | **7** | **5** | **7** | **3** |
| 6. Splashable | **8** | **9** | **7** | **5** | **8** |
| 7. Open early | **7** | **9** | **8** | **5** | **8** |
| 8. Signal reading | **4** | **3** | **4** | **2** | **9** |
| **Total** | **58** | **57** | **50** | **51** | **58** |

### Justifications

**S1 (Weighted Lottery):** Simplicity is genuine — one sentence is the complete algorithm. Convergence at 2.31 arch-fit passes comfortably. Splash at 1.69 is strong. Early-fit at 2.02 is a marginal fail (sw=3 tuning drops it to 1.73). No signal reading mechanism. 7.5% overlap is excellent variety.

**S2 (Balanced Pack):** 1/1/1/1 → 2/1/1/0 is the most transparent pack structure. 7/8 targets passed (best raw rate). Splash at 1.92 and early openness at 3.72 are best-in-class. Weakness: majority bonus supports only ONE resonance, so dual-resonance archetypes (e.g., Warriors needing both Tide and Zephyr) get incomplete support.

**S3 (Lane Locking):** Permanent binary locks give excellent transparency but create the worst pivot flexibility. Late arch-fit at 1.83 fails the 2.0 target — locking a resonance slot doesn't guarantee archetype-appropriate cards (Tide lock shows Sacrifice OR Warriors). Even (5,12) thresholds only reach 1.92. Irreversible locks mean no recovery from early mistakes.

**S4 (Echo Window):** 3-pick memory enables the best pivots, but it's a double-edged sword. Early arch-fit at 2.58 massively fails the <=2.0 target — one pick immediately determines 2/4 slots. Splash at 0.43 also fails. Strong convergence (2.83) is actually over-convergence. The algorithm is "on rails from pick 1" with just a short rail.

**S5 (Resonance Swap):** 44.8% signal detection rate is the standout result across all strategies (+19.8pp over random). But convergence at 1.61 is structurally unfixable — swapping 3 cards in a 360-card pool can't shift resonance distributions fast enough. The strategy itself correctly identifies its optimal role as a complementary layer, not a standalone mechanism.

## Biggest Strength / Weakness per Strategy

| Strategy | Biggest Strength | Biggest Weakness |
|----------|-----------------|------------------|
| S1: Weighted Lottery | Balanced convergence (2.31 arch-fit) with strong splash (1.69) | No signal reading mechanism (4/10) |
| S2: Balanced Pack | Structural guarantee gives best splash (1.92) and early openness (3.72) | Only supports one resonance via majority; dual-archetype support weak |
| S3: Lane Locking | Perfect transparency — players know exact pack structure | Permanent locks kill pivots AND fail convergence (resonance ≠ archetype) |
| S4: Echo Window | Best pivot flexibility via 3-pick memory | Biases from pick 1 (2.58 early-fit); over-converges while failing splash |
| S5: Resonance Swap | Best signal reading (44.8% detection) and run variety (6.5% overlap) | Convergence structurally capped at ~1.67; cannot standalone |

## Improvement Proposals

**S1:** Use starting weight 3 instead of 1 (fixes early-fit from 2.02→1.73 while keeping convergence at 2.14). Layer S5's starting pool asymmetry for signal reading.

**S2:** Explore "top 2 resonances each get a bonus slot" variant for dual-resonance support. Risk: this may collapse into "always 2/2/0/0" too quickly. Alternative: keep 2/1/1/0 but use the top-2 resonances for the 2 slots instead of just majority.

**S3:** Add lock decay — locks older than 10 picks revert to open. This preserves transparency in the active window while allowing pivots. Or: replace permanent locks with "priority slots" that bias toward locked resonance (70%) but don't guarantee it.

**S4:** Increase window to 5+ picks to dampen early bias. Or: use "2/1/0+1" allocation (2 top, 1 second, 1 guaranteed random) as the default — this fixes splash to 0.56 and slightly improves early openness.

**S5:** Accept the convergence limitation and focus on being a complementary layer. Simplify: drop the per-pick swap mechanism and keep only the starting asymmetry (+20/-20) as a signal-reading add-on for other algorithms.

## Proposed Best Algorithm

**Weighted Lottery with Starting Asymmetry** (hybrid of S1 + S5's signal layer)

Core: S1's weighted lottery with starting weight 3, which passes 7/8 targets (only deck concentration fails, which ALL strategies fail). Layer S5's starting pool asymmetry (+20/-20 cards of one resonance) for signal reading.

**One-sentence description:**

"Each resonance starts at weight 3; each drafted symbol adds to weights (primary +2, others +1); 3 of 4 pack slots draw a card of a resonance chosen proportionally to weights, and the 4th slot is a random card from the full pool — which starts each run with one resonance boosted and one suppressed."

**Expected performance (projected from S1 sw=3 data + S5 signal data):**
- Early diversity: 3.3+ unique resonances/pack (PASS)
- Early arch-fit: ~1.73 (PASS)
- Late arch-fit: ~2.14 (PASS)
- Splash: ~1.86 (PASS)
- Convergence pick: ~6.6 (PASS)
- Card overlap: <10% (PASS)
- Signal detection: ~40%+ (significant improvement over base S1)

The pool asymmetry is a separate, independent layer from the pack assembly algorithm. Players can reason about each independently: "my weights determine my pack structure" AND "the pool has more of some resonances this run."
