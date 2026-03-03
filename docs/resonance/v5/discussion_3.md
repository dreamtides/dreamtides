# Discussion 3: Pool Evolution / Seeding — Round 2 Output

## Simplicity Ranking (Most to Least Simple)

All 5 agents agree on the ranking within 1-2 positions:

1. **Pair Slot Guarantee (D4)** — 9/10. "Draft 3 of a pair, 1 slot guaranteed."
   Crystal-clear threshold, deterministic outcome, easiest mental model.
2. **Pair-Based Threshold Auto-Spend (D1)** — 8/10. Clear trigger-reset cycle.
   Minor ambiguity on simultaneous triggers, but fully implementable.
3. **Pair Cluster Bonus (D5)** — 7/10. Simple concept, but the trigger
   probability (~5% for strict pair, ~26-68% for hybrid) is hard to intuit.
4. **Top-Pair Pool Seeding (D3, mine)** — 7/10 (up from 5/10 after
   simplification). Invisible mechanism is elegant but opaque to players.
5. **Pair-Escalation Slots (D2)** — 6/10. The probability formula
   `min(count/K, cap)` is concrete but requires mathematical reasoning.

## Scorecard Table

Scores incorporate the corrected baseline S/A of ~0.89 per pack (80 S/A cards
out of 360 = 22.2% density, confirmed by Agent 5's analysis), the convergence
hierarchy (Agent 1), and pair precision of ~95-100% (Agent 4's stress-test).

| Goal | D1 Auto-Spend | D2 Escalation | D3 Pool Seed | D4 Dual-Thresh | D5 Hybrid |
|------|:---:|:---:|:---:|:---:|:---:|
| 1. Simple | 8 | 6 | 7 | 8 | 7 |
| 2. No extra actions | 10 | 10 | 10 | 10 | 10 |
| 3. Not on rails | 6 | 4 | 8 | 4 | 8 |
| 4. No forced decks | 6 | 5 | 8 | 6 | 7 |
| 5. Flexible archetypes | 5 | 4 | 6 | 4 | 6 |
| 6. Convergent (2.0+) | 4 | 9 | 3 | 7 | 4 |
| 7. Splashable | 7 | 4 | 7 | 6 | 8 |
| 8. Open early | 6 | 6 | 8 | 7 | 9 |
| 9. Signal reading | 2 | 2 | 7 | 2 | 2 |
| **Total** | **54** | **50** | **64** | **54** | **61** |

**Convergence scoring rationale (the key discussion finding):**

Agent 1 identified a clear convergence hierarchy of mechanism types:
- **Tier 1 — Slot replacement (D2):** Replaces 22% S/A baseline slots with 95%
  S/A targeted slots. At P=0.65 (refined): 4 * [0.65*0.95 + 0.35*0.22] = 2.78.
  Comfortably crosses 2.0.
- **Tier 2 — Guaranteed slots (D4 refined):** 2 guaranteed slots at count 3/7:
  2*0.95 + 2*0.22 = 2.34. Crosses 2.0 after second threshold.
- **Tier 3 — Bonus injection (D1, D5):** Adds ~0.3-0.6 bonus S/A per pack on
  top of 0.89 baseline. Total ~1.2-1.5. Structurally below 2.0.
- **Tier 4 — Pool manipulation (D3):** Shifts density from 22% to ~28-30% at
  best. Total 4*0.30 = 1.20. Structurally below 2.0.

Only D2 and D4 (refined to dual-threshold) clearly cross 2.0 standalone.

## Final Championed Algorithm

**Top-Pair Pool Seeding (refined from Proportional Pair Seeding)**

One-sentence: "After each pick, if you have drafted 2+ cards with the same
ordered resonance pair, 4 cards matching your most-drafted pair are added to
the pool from a reserve."

I keep pool seeding despite acknowledging it cannot cross 2.0 standalone. The
discussion crystallized three reasons:

1. **Unique strengths.** Pool seeding is the only mechanism supporting signal
   reading (Goal 9) and the most resistant to forced decks (Goal 4). No other
   champion interacts with pool composition.

2. **Best complement.** Pool seeding + any other champion outperforms that
   champion alone. Seeding enriches random slots, directly boosting D2/D4's
   untargeted-slot contribution. A D3+D4 hybrid (seeding + 1 guaranteed slot)
   is a natural combination: 1 slot guaranteed at ~0.95, 3 random from enriched
   pool at ~0.28 each = 0.95 + 0.84 = 1.79. Close to 2.0 with modest seeding.

3. **Invisible mechanism.** Zero cognitive load. The player drafts cards and
   the pool quietly shifts. This is the strongest candidate for "feels like
   natural variation" — the V5 experiential goal.

**Honest failure:** Pool seeding alone hits ~1.2-1.5 S/A. The pool bloat
problem (denominator grows with numerator) is structural, as confirmed by
Agents 2, 4, and 5 in their stress-tests of my math.

## Modifications for Round 3

1. **Simplified to top-pair only** — dropped proportional allocation. Only the
   leading pair gets injection. Universally agreed as clearer.
2. **Activation gate at pair count 2** — prevents injection before commitment.
3. **Test injection rates:** 3, 4, and 5/pick, plus 5/pick with 1 removal.
4. **Test Escalating variant:** inject min(top_pair_count, 5) per pick. Best
   temporal profile (Agent 1 and Agent 5 both flagged this as underrated).
5. **Test D3+D4 hybrid:** pool seeding + 1 guaranteed pair slot (threshold 3).
   This hybrid tests whether invisible pool improvement + reliable slot
   delivery crosses 2.0 together.
6. **Measure actual pair precision** in the simulated card pool (Agent 2's
   recommendation). Validate the 95% assumption all champions depend on.
7. **Test 30% 1-symbol robustness.** All champions share vulnerability to
   1-symbol-heavy distributions. Quantify degradation.

## Proposed Symbol Distribution

| Symbols | % Non-Generic | Cards |
|---------|:---:|:---:|
| 0 (generic) | — | 36 |
| 1 symbol | 15% | 49 |
| 2 symbols | 65% | 211 |
| 3 symbols | 20% | 64 |

Unanimous consensus across all 5 agents: 85% of non-generic cards need 2+
symbols. This distribution maximizes pair generation for all pair-based
champions. Round 3 robustness test at 30% 1-symbol will quantify the shared
vulnerability.
