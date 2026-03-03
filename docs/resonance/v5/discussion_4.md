# Round 2 Discussion Output — Agent 4 (Pair-Based Pack Construction)

## Simplicity Ranking (most to least simple)

Consensus across all 5 agents placed Pair Slot Guarantee (mine) and Pair-Based
Threshold Auto-Spend (Agent 1) as the top 2, with Proportional Pair Seeding
(Agent 3) consistently last. Agent 3 acknowledged this and simplified to
Top-Pair Pool Seeding for Round 3.

1. **Pair Slot Guarantee / Dual-Threshold (Agent 4)** — Binary threshold, no
   formulas. "Draft 3 of a pair → 1 slot guaranteed; at 7 → second slot." Every
   agent ranked this #1 or #2 for simplicity.
2. **Pair-Based Threshold Auto-Spend (Agent 1)** — Clear trigger-reset loop.
   Slightly more complex due to reset mechanic and variable pack size (4 or 5).
3. **Pair Cluster Bonus (Agent 5)** — Concrete condition, but Agent 5 revised
   to Hybrid Trigger (resonance trigger, pair bonus) which is slightly harder.
4. **Pair-Escalation Slots (Agent 2)** — Formula `min(count/K, cap)` is concrete
   but opaque to players. Revised to K=6, cap=0.65 for Round 3.
5. **Top-Pair Pool Seeding (Agent 3, revised)** — Simplified from proportional
   allocation, but pool/reserve concept remains invisible and hardest to explain.

## Scorecard Table

Revised after discussion. Key changes: lowered Agent 1 and Agent 5 convergence
scores after rigorous math showed bonus-injection mechanisms struggle to cross
2.0 S/A. Baseline S/A confirmed at ~0.89 per random 4-card pack.

| Goal | D1 Auto-Spend | D2 Escalation | D3 Seeding | D4 Guarantee | D5 Conditional |
|------|:---:|:---:|:---:|:---:|:---:|
| 1. Simple | 9 | 7 | 5 | 9 | 7 |
| 2. No extra actions | 10 | 10 | 10 | 10 | 10 |
| 3. Not on rails | 7 | 5 | 8 | 4 | 8 |
| 4. No forced decks | 7 | 5 | 8 | 6 | 7 |
| 5. Flexible archetypes | 7 | 5 | 8 | 5 | 7 |
| 6. Convergent (2.0+) | 3 | 9 | 3 | 7 | 3 |
| 7. Splashable | 8 | 4 | 7 | 7 | 8 |
| 8. Open early | 7 | 5 | 7 | 8 | 9 |
| 9. Signal reading | 3 | 3 | 7 | 3 | 3 |
| **Total** | **61** | **53** | **63** | **59** | **62** |

**Critical convergence finding from discussion:** Agent 1 identified a clear
convergence hierarchy: per-slot targeting (D2) > guaranteed slots (D4) >
bonus injection (D1, D5) > pool manipulation (D3). Only D2 and D4 (with dual
threshold) structurally cross 2.0 S/A. D1's bonus fires ~29% of packs,
contributing only ~0.26 S/A — total ~1.15. D5's conditional trigger fires even
less often. D3's pool bloat dilutes density gains. Slot REPLACEMENT (D2/D4)
outperforms slot ADDITION (D1/D5) because it replaces 22% baseline slots with
90%+ pair-matched slots — a 4x multiplier per targeted slot.

## Final Championed Algorithm

**Dual-Threshold Pair Guarantee** (refined from Round 1's single-threshold).

**One-sentence:** "Track the ordered symbol pair (first, second) of each 2+
symbol card you draft; at 3 matching picks one pack slot is pair-matched, at 7
a second slot is pair-matched, and remaining slots are random."

**Why this survives discussion:** The dual threshold was independently
recommended by Agents 1, 3, and 5 as the fix for my convergence problem.
Single-threshold (~1.5 S/A) fails; dual-threshold projects ~2.4 S/A (2 slots
× 0.95 pair precision + 2 random × 0.22 baseline). It occupies a unique niche
as the most **transparent** mechanism — players know exactly when thresholds
activate and what happens. This determinism is the strength (strategizable) and
weakness (mechanical). Two random slots (50% of pack) preserve splash better
than D2's 1.4 random slots at cap.

**Acknowledged weakness:** This is "Lane Locking Lite" — deterministic slot
assignment with pair precision. The "on rails" concern (Goal 3 score: 4) is
real. I accept this tradeoff: transparency and convergence at the cost of
organic feel. The probabilistic alternative (D2) is the natural comparison —
Round 3 simulation of both will reveal which tradeoff players prefer.

## Modifications for Round 3

1. **Dual threshold (3/7)** as primary. Test (2/5), (3/7), (4/9) variants.
2. **Leading pair tracks dynamically** — guaranteed slots follow the highest
   pair count, enabling pivots without permanent commitment.
3. **Fixed pack size (always 4)** — guaranteed slots replace random draws, not
   add to them. Simpler UI than bonus-card approaches.
4. **Measure actual pair precision** — all agents agreed this is the single
   most important validation. If pair precision is 75% not 95%, convergence
   projections drop significantly for ALL champions.
5. **Test 30% 1-symbol robustness** — Agent 3 flagged that all pair-based
   champions degrade simultaneously if 1-symbol cards are more common.
6. **Compare to D2 (Pair-Escalation K=6, cap=0.65)** as the probabilistic
   counterpart to my deterministic approach.

## Proposed Symbol Distribution

| Symbols | % of Non-Generic | Cards |
|---------|:---:|:---:|
| 0 (generic) | — | 36 |
| 1 | 15% | 49 |
| 2 | 65% | 211 |
| 3 | 20% | 64 |

All 5 agents converged on ~15/65/20. The critical number is 85% of non-generic
cards having 2+ symbols, ensuring most picks contribute pairs. Test 30%
1-symbol as robustness check per Agent 3's recommendation.
