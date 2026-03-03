# Round 2 Discussion Output — Agent 1 (Passive Resonance Bonus)

## Simplicity Ranking (Most to Least Simple)

Consensus across all 5 agents (rankings were nearly identical):

1. **D4 Pair Slot Guarantee** — Binary threshold, one guaranteed slot, no
   formulas. Implementable from one sentence with zero ambiguity.
2. **D1 Pair-Based Threshold Auto-Spend** (mine) — Threshold + reset + bonus
   card. Concrete and unambiguous. Two agents independently invented this
   algorithm (D1 champion = D4 Algorithm 5).
3. **D5 Pair Cluster Bonus** — Two-phase pack generation (draw, inspect,
   conditionally add). Clear but trigger probability is hard for players to
   intuit.
4. **D2 Pair-Escalation Slots** — Probability formula min(count/K, cap) is
   precise but requires mathematical reasoning from the player.
5. **D3 Proportional Pair Seeding** — "Distributed in proportion" hides
   allocation rounding. "From a reserve" hides reserve management. Agent 3
   self-criticized and simplified to Top-Pair Pool Seeding.

## Scorecard Table

Scores updated after discussion-phase convergence math and baseline S/A
calibration (baseline = ~0.89 S/A per random 4-card pack, or 22.2% per slot).

| Goal | D1: Pair Threshold | D2: Pair-Esc Slots | D3: Top-Pair Seed | D4: Dual-Thresh Guar | D5: Hybrid Trigger |
|------|:---:|:---:|:---:|:---:|:---:|
| 1. Simple | 8 | 6 | 6 | 8 | 7 |
| 2. No extra actions | 10 | 10 | 10 | 10 | 10 |
| 3. Not on rails | 7 | 6 | 8 | 5 | 9 |
| 4. No forced decks | 7 | 6 | 8 | 6 | 8 |
| 5. Flexible archetypes | 7 | 6 | 7 | 5 | 7 |
| 6. Convergent (2.0 S/A) | 4 | 9 | 4 | 7 | 3 |
| 7. Splashable | 8 | 5 | 7 | 7 | 8 |
| 8. Open early | 8 | 6 | 8 | 8 | 9 |
| 9. Signal reading | 3 | 3 | 7 | 3 | 3 |
| **Total** | **62** | **57** | **65** | **59** | **64** |

### Key Scoring Rationale

- **D2 Convergence = 9:** Per-slot replacement is structurally the strongest
  mechanism. At P=0.65 (refined cap), each slot yields 0.65×0.95 + 0.35×0.22
  = 0.70 S/A. Total: 4 × 0.70 = **2.79 S/A**. Only mechanism that
  comfortably crosses 2.0.
- **D1 Convergence = 4:** Honest self-assessment after discussion math.
  Bonus fires ~29% of packs (threshold 3) or ~43% (threshold 2). Per-pack
  contribution: 0.89 + 0.43 × 0.95 = **1.30 S/A** at best. Below 2.0.
  Bonus injection adds marginal value atop a ~0.89 baseline.
- **D4 Convergence = 7:** Agent 4's dual-threshold refinement (3/7) gives
  2 guaranteed slots: 2 × 0.95 + 2 × 0.22 = **2.34 S/A**. Crosses 2.0
  after second threshold activates (~pick 10-12).
- **D5 Convergence = 3:** Agent 5 acknowledged the 2-of-4 pair trigger has
  ~5% fire rate (not 45-60%). Even the hybrid variant (resonance trigger,
  pair bonus) yields ~1.6 S/A.
- **D3 Convergence = 4:** Pool bloat dilution caps density at ~25-27%.
  4 × 0.27 = ~1.1 S/A from archetype density alone. Agent 3 acknowledged
  pool seeding likely cannot solo-cross 2.0.

## Critical Discussion Finding: The Structural Convergence Gap

The discussion revealed a fundamental hierarchy of mechanism types:

1. **Slot replacement** (D2): Replaces 22% S/A baseline slots with 95% S/A
   targeted slots. Per-slot gain: +0.73. Can target all 4 slots.
   **Ceiling: ~3.6 S/A.**
2. **Guaranteed slots** (D4 refined): N deterministic slots at ~95% S/A.
   **Ceiling: ~2.4 at N=2.**
3. **Bonus injection** (D1, D5): Adds one ~95% S/A card to a pack with ~0.89
   baseline. Fires 29-43% of packs. **Ceiling: ~1.3 S/A.**
4. **Pool manipulation** (D3): Shifts archetype density from 11% to ~25%.
   **Ceiling: ~1.5 S/A.**

Only Tier 1-2 mechanisms reliably cross 2.0. Bonus injection and pool seeding
are structurally capped because they ADD marginal cards rather than REPLACING
low-value baseline slots with high-value targeted draws.

Agent 4 confirmed pair precision is ~100% S-tier (not 90%), since ordered pairs
uniquely identify archetypes. This makes slot replacement even more powerful.

## Final Championed Algorithm

I maintain **Pair-Based Threshold Auto-Spend** for simulation, with aggressive
parameters to test whether bonus injection can cross 2.0:

**One-sentence:** "Each card you draft with 2+ symbols adds 1 to its ordered
pair count; when any pair reaches 2, your next pack gets 2 bonus cards sharing
that pair and the count resets to 0."

Threshold lowered to 2, bonus increased to 2 cards per trigger. This is the
most aggressive configuration that remains simple. Projected: fires ~43% of
packs, adds 2 × 0.95 = 1.90 S/A per trigger, averaged = 0.82. Total: 0.89 +
0.82 = **1.71 S/A**. Still below 2.0 — confirming the structural ceiling.

I will honestly report this result and test a D1+D4 hybrid: threshold bonus
plus one guaranteed slot.

## Specific Modifications for Round 3

1. **Threshold 2, Bonus 2** — most aggressive standalone configuration
2. **Threshold 2, Bonus 1** — moderate configuration for comparison
3. **D1+D4 hybrid** — threshold bonus + 1 guaranteed pair slot at count 3
4. **Parameter sweep:** Threshold {1, 2, 3} × Bonus {1, 2, 3}
5. **Baseline comparison:** Lane Locking + auto-spend Pack Widening
6. **Measure actual pair precision** in simulated card pool
7. **Test 30% 1-symbol robustness** case per Agent 3's suggestion

## Proposed Symbol Distribution

| Symbols | % of Non-Generic | Cards |
|---------|:---:|:---:|
| 0 (generic) | — | 36 |
| 1 symbol | 15% | 49 |
| 2 symbols | 65% | 211 |
| 3 symbols | 20% | 64 |

All 5 agents converged on ~15/65/20. The critical number is 85% of cards with
2+ symbols, ensuring pair tracking activates on most picks. Will also test 30%
1-symbol variant for robustness.
