# Pool Results 5: Pair-Escalation Slots Parameter Tuning

## Simulation Summary

Tested 120 configurations (5 K values x 4 C values x 2 pack sizes x 3 symbol distributions), 500 drafts each, with 1000-draft reruns for the top 5. All metrics at archetype level.

## Key Findings

### 1. Pack Size 4 Dominates Pack Size 5

Pack size 5 over-converges across the board. With 5 slots, late S/A averages 3.2-3.4 (vs 2.6-2.7 for pack size 4), convergence arrives too early (pick 4.2-4.5 vs 5.9-6.6), and deck concentration balloons to 97%+. Pack size 5 requires C=0.35 to stay in the sweet spot, which eliminates the escalation arc. **Recommendation: keep pack size at 4.**

### 2. C (Cap) Is the Primary Convergence Lever

C controls the S/A ceiling directly. Across all K values and distributions at pack size 4:

| C | Late S/A | Off-arch C/F | Convergence |
|---|----------|-------------|-------------|
| 0.35 | 2.17 | 0.91 | 6.2 |
| 0.50 | 2.50 | 0.74 | 6.1 |
| 0.65 | 2.80 | 0.58 | 6.3 |
| 0.80 | 3.08 | 0.44 | 6.4 |

C=0.35 provides the best balance of convergence with splash (0.91 off-archetype cards per pack) but sits right at the 2.0 S/A floor. C=0.50 safely exceeds 2.0 while keeping 0.74 off-archetype cards. C=0.65 is the sweet spot for S/A strength (2.80) while maintaining 0.58 off-archetype. C=0.80 over-converges -- off-archetype drops below 0.5 and the algorithm starts to feel mechanical.

### 3. K Has Minimal Impact on Late S/A but Shapes the Arc

K controls how quickly P rises, not its ceiling (C controls that). At pack size 4, default distribution:

| K | Late S/A | P@5 | P@10 | P@15 | Three-Act Ratio |
|---|----------|-----|------|------|-----------------|
| 4 | 2.66 | 0.44 | 0.56 | 0.57 | 1.8x |
| 6 | 2.66 | 0.35 | 0.53 | 0.57 | 2.7x |
| 8 | 2.66 | 0.28 | 0.51 | 0.57 | 3.5x |
| 10 | 2.66 | 0.22 | 0.46 | 0.57 | 4.6x |
| 12 | 2.63 | 0.17 | 0.40 | 0.56 | 6.0x |

K=4 reaches cap too quickly (P@5 already 0.44), collapsing the exploration phase. K=12 creates the best three-act structure but slows early commitment. K=8-10 produces the ideal ramp: P reaches ~0.25 by pick 6 (1 targeted slot) and ~0.50 by pick 12 (2 targeted slots).

### 4. Symbol Distribution Has Surprisingly Little Impact

Heavy 2-symbol (5/75/20) accelerates pair accumulation by ~15% vs heavy 1-symbol (40/40/20), but late S/A difference is only 0.06 (2.68 vs 2.59 at pack size 4). The algorithm is robust across distributions because even the heavy 1-symbol pool has 54% paired cards -- enough to drive pair counts. **The algorithm does not require pool redesign.**

### 5. The Escalation Curve Sweet Spot

The best three-act arc comes from K=10-12, C=0.65, pack size 4. Example (K=10, C=0.65, default):

- **Exploration (picks 1-5):** P=0.12, ~0.5 targeted slots. Packs are mostly random.
- **Commitment (picks 6-15):** P=0.50, ~2.0 targeted slots. Clear ramp.
- **Refinement (picks 16-30):** P=0.65, ~2.6 targeted slots. Steady convergence with 1.4 random slots.

This produces an 5.6x exploration-to-refinement ratio -- the draft feels fundamentally different at each stage.

### 6. Over/Under Convergence Boundaries

- **Over-converged** (late S/A > 3.0 or StdDev < 0.8): C=0.80 at pack size 4, or any C at pack size 5.
- **Under-converged** (late S/A < 2.0): No configurations under-converge. Even C=0.35/K=12 hits 2.15.
- **Sweet spot** (2.0-3.0 S/A, StdDev >= 0.8, overlap < 40%): 63 of 120 configs, all at pack size 4.

### 7. The Universal Failure: Deck Concentration

All configurations exceed the 60-90% deck concentration target (93-97%). This is structural: pair-matched cards are near-100% S/A, and even random slots produce ~22% S/A baseline. A committed player drafting S/A whenever available will always exceed 90%. This target may need revisiting for pair-based algorithms, or the pool needs more generics to dilute S/A density.

## Scorecard: Top Configurations (1000 drafts)

| Metric | Target | K=8 C=0.65 | K=10 C=0.65 | K=6 C=0.65 | K=12 C=0.80 |
|--------|--------|-----------|------------|-----------|------------|
| Early unique archs | >= 3 | 6.4 PASS | 6.4 PASS | 6.2 PASS | 6.4 PASS |
| Early S/A | <= 2 | 1.71 PASS | 1.60 PASS | 1.84 PASS | 1.55 PASS |
| Late S/A | >= 2.0 | 2.85 PASS | 2.84 PASS | 2.84 PASS | 3.01 PASS |
| Off-arch C/F | >= 0.5 | 0.56 PASS | 0.58 PASS | 0.55 PASS | 0.49 FAIL |
| Convergence pick | 5-8 | 6.0 PASS | 6.0 PASS | 5.9 PASS | 6.4 PASS |
| Deck concentration | 60-90% | 95% FAIL | 96% FAIL | 94% FAIL | 95% FAIL |
| Run-to-run overlap | < 40% | 13% PASS | 14% PASS | 12% PASS | 14% PASS |
| S/A StdDev (6+) | >= 0.8 | 1.03 PASS | 1.01 PASS | 1.07 PASS | 1.06 PASS |
| **Total** | | **7/8** | **7/8** | **7/8** | **6/8** |

## Recommended Configuration

**K=10, C=0.65, Pack Size=4** with the V5 default symbol distribution (15/60/25).

- Passes 7/8 targets (only deck concentration exceeds 90%)
- Late S/A of 2.84 with 1.01 StdDev -- strong convergence with variance
- Off-archetype C/F of 0.58 -- meaningful splash every other pack
- Convergence at pick 6.0 -- right in the target window
- Excellent three-act arc (0.12 / 0.50 / 0.65) with 5.2x ratio
- Robust across all three symbol distributions (heavy 1-sym still hits 2.68)
- K=10 produces the cleanest escalation: ~1 targeted slot at pick 8, ~2 at pick 14, ~2.6 at pick 20

K=8 C=0.65 is the close runner-up with nearly identical metrics but a slightly less dramatic exploration phase (P@5=0.31 vs 0.23). Either works well.
