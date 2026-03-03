# Round 2 Discussion Output — Agent 2 (Probabilistic Slot Targeting)

## Simplicity Ranking (Most to Least Simple)

1. **Agent 4: Pair Slot Guarantee** — Binary state machine. Before threshold:
   random. After threshold: 1 guaranteed + 3 random. No probabilities, no
   proportions, no fire rates. A player knows exactly what to expect.
2. **Agent 1: Pair-Based Threshold Auto-Spend** — Threshold-reset with bonus
   card. Concrete counter + trigger. Slightly less simple than Agent 4 because
   the bonus card changes pack size (4 vs 5).
3. **Agent 2: Pair-Escalation Slots (mine)** — Probability formula per slot.
   Concrete and implementable but requires understanding "each slot rolls
   independently." Players can't predict exact pack composition.
4. **Agent 5: Pair Cluster Bonus** — Conditional trigger on pack composition.
   One sentence but the trigger condition ("2 of 4 random cards share your
   pair") requires inspecting the pack's own composition, a self-referential
   concept.
5. **Agent 3: Proportional Pair Seeding** — Proportional allocation with
   fractional rounding across an invisible pool. Most abstract; players cannot
   observe or reason about what's happening.

## Scorecard Table

| Goal | 1: Pair Threshold | 2: Pair-Escalation (mine) | 3: Prop. Seeding | 4: Pair Slot Guarantee | 5: Pair Cluster Bonus |
|------|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | 7 | 6 | 4 | 8 | 6 |
| 2. No Actions | 10 | 10 | 10 | 10 | 10 |
| 3. Not Rails | 7 | 8 | 8 | 5 | 9 |
| 4. No Forced | 7 | 5 | 8 | 6 | 7 |
| 5. Flexible | 6 | 6 | 7 | 5 | 8 |
| 6. Convergent | 5 | 9 | 5 | 4 | 4 |
| 7. Splashable | 7 | 5 | 8 | 7 | 7 |
| 8. Open Early | 7 | 7 | 5 | 8 | 8 |
| 9. Signal | 3 | 3 | 7 | 3 | 3 |
| **Total** | **59** | **59** | **62** | **56** | **62** |

**Key scoring rationale for Goal 6 (Convergent):** Baseline S/A from random
packs is ~0.89 per 4-card pack (80 S/A cards out of 360 for any archetype).
Crossing 2.0 requires adding ~1.11 S/A per pack on average. Single-bonus-card
approaches (Agents 1, 5) that fire 20-60% of the time add at most ~0.6 S/A,
reaching ~1.5. Agent 4's single guaranteed slot reaches ~1.67. Only slot
targeting (replacing ~2.6 random draws with pair-targeted draws at ~100%
S-tier precision) projects above 2.0. Agent 3's pool density shift (11% to
~25%) yields ~1.0 S/A from 4 random draws — well below 2.0.

**Pair precision insight:** In the simulation model, ordered pair (Tide, Zephyr)
maps exclusively to Warriors cards — 100% S-tier precision. But pair targeting
MISSES A-tier cards from adjacent archetypes (Sacrifice cards have pair (Tide,
Stone)). Pair targeting is extremely precise but narrow.

## Final Championed Algorithm: Pair-Escalation Slots v2

I retain my champion with refined parameters.

**One-sentence:** "Track the resonance pair (first, second symbol) of each 2+
symbol card you draft; each pack slot independently shows a card matching your
most common pair with probability min(that pair's count / 6, 0.65), otherwise a
random card."

**Why I'm keeping it:** The convergence analysis strongly suggests this is the
only champion that can reliably cross 2.0 S/A. The structural advantage is
that slot targeting REPLACES low-value random draws (~22% S/A) with high-value
pair-targeted draws (~100% S-tier), across multiple slots simultaneously.
Bonus-card approaches and pool seeding cannot achieve the same density shift
within 4-card packs.

Changes from v1: K=6 (was 5), cap=0.65 (was 0.80). Slower ramp preserves
early openness. Lower cap preserves 1.4 random slots for splash. Projected
S/A: 2.6 targeted at 100% + 1.4 random at 22% = 2.91. Even at conservative
80% pair precision: 2.6 * 0.80 + 1.4 * 0.22 = 2.39. StdDev from Binomial(4,
0.65) = 0.95, exceeding the 0.8 target.

**Honest concern:** If pair precision is lower than expected or the pair-matched
pool is too small for varied draws, convergence drops. Simulation is essential.

## Round 3 Modifications

1. **K=6, cap=0.65** (refined baseline)
2. **K=5, cap=0.65** (faster ramp variant)
3. **K=7, cap=0.70** (slower ramp variant)
4. **Measure actual pair precision** — report what % of pair-drawn cards are
   S/A for target archetype
5. **Drawing with replacement** for targeted slots (34-card pair pool would
   exhaust without replacement by mid-draft)
6. **Compare against Lane Locking and auto-spend Pack Widening baselines**
7. **Symbol distribution sensitivity sweep:** test 1-sym-heavy (40/40/20) vs
   2-sym-heavy (10/70/20)

## Proposed Symbol Distribution

| Count | % Non-Generic | Cards |
|-------|:---:|:---:|
| 0 (generic) | -- | 36 |
| 1 symbol | 15% | 49 |
| 2 symbols | 60% | 194 |
| 3 symbols | 25% | 81 |

85% of non-generic cards have 2+ symbols, ensuring most picks contribute pairs.
Each archetype gets ~34 pair-matchable cards, providing adequate draw variety.

## Cross-Cutting Observations

All 5 champions converged on pair-based matching — the strongest consensus
signal in V5. The structural finding from this discussion: **slot replacement is
more powerful than card addition.** Replacing a random slot (22% S/A) with a
targeted slot (100% S/A) gains 0.78 S/A. Adding a bonus card (100% S/A) gains
1.0 S/A but only when it fires (20-60% of packs), averaging 0.2-0.6 S/A per
pack. My approach replaces 2.6 slots = 2.03 S/A gain. The best bonus approach
adds 0.6 S/A gain. This is why slot targeting is the path to 2.0.
