# Round 4 Comparison — Agent 5 (Conditional Pack Enhancement)

## Scorecard Table (Strategy x Goal, 1-10)

| Goal | D1: Auto-Spend | D2: Pair-Esc | D3: Pool Seed | D4: Dual-Thresh | D5: Hybrid Trigger |
|------|:-:|:-:|:-:|:-:|:-:|
| 1. Simple | 8 | 5 | 7 | 9 | 7 |
| 2. No actions | 10 | 10 | 10 | 10 | 10 |
| 3. Not on rails | 8 | 4 | 9 | 5 | 9 |
| 4. No forced decks | 7 | 5 | 9 | 6 | 8 |
| 5. Flexible | 7 | 4 | 7 | 5 | 8 |
| 6. Convergent | 2 | 10 | 2 | 8 | 3 |
| 7. Splashable | 8 | 5 | 8 | 7 | 8 |
| 8. Open early | 9 | 7 | 9 | 8 | 9 |
| 9. Signal reading | 2 | 3 | 8 | 3 | 3 |
| **Total** | **61** | **53** | **69** | **61** | **65** |

Totals misleading: D3 scores highest but fails convergence. Convergence is pass/fail
— only D2 and D4 pass standalone. D4 convergence raised to 8 for (2,5) variant at
2.48 S/A.

## Biggest Strength / Weakness per Strategy

**D1 (1.10 S/A):** Strength: simplest threshold-reset mechanic. Weakness:
structurally capped at ~1.3 S/A. Unfixable standalone; retire as primary candidate.

**D2 (3.00 S/A):** Strength: demolishes convergence target with fastest convergence
(pick 5.8). Weakness: 97% deck concentration at default cap; at cap=0.50 drops to
healthier 2.61/0.71 off-arch but formula remains opaque to players.

**D3 (1.18 S/A):** Strength: most natural feel, only mechanism supporting signal
reading (Goal 9), lowest overlap (5.7%). Weakness: pool bloat ceiling at ~1.2 S/A.
Value as complementary layer only.

**D4 (2.22 S/A at 3/7; 2.48 at 2/5):** Strength: most transparent — binary
thresholds players can track and strategize around. Weakness: convergence pick 9.2 at
(2,5), just outside 5-8 target. Pair accumulation speed is the bottleneck.

**D5 (1.52 standalone; 2.10 D4 hybrid):** Strength: best organic variance (1.23
stddev). Weakness: cannot solo-cross 2.0. The D4 hybrid adds variance but REDUCES S/A
vs pure D4 (2,5) at 2.48 — the conditional layer adds complexity without net benefit.

## V3/V4 Comparison

Baselines vary across sims (Lane Locking: 1.74-2.61; Pack Widening: 0.80-1.96).
Round 5 must unify these on identical pools.

**V5 vs Lane Locking:** D2 cap=0.50 (2.61) and D4 (2,5) (2.48) both approach or
exceed Lane Locking's ~2.3-2.6. D4 trades 2-3 convergence picks for better variance
(0.82 vs 0.74), better splash, and no mechanical feel. D2 beats Lane Locking on
convergence speed while eliminating permanent locks.

**V5 vs Pack Widening:** All V5 algorithms crossing 2.0 decisively beat auto-spend
Pack Widening (0.80-1.96). Pack Widening with manual spending (~2.3-2.5) violates
V5's no-decision constraint. D4 (2,5) achieves 2.48 with zero decisions — matching
Pack Widening's best without any player burden.

**Is zero-decision worth convergence loss?** Yes. D4 at 2.48 with no decisions vs
Pack Widening at ~2.3-2.5 with decisions is equivalent or better. The cognitive load
elimination is the entire point of V5.

## Pair-Matching Analysis

Pair matching definitively breaks the archetype dilution ceiling. D2's pair-targeted
slots deliver ~93-100% S-tier vs ~50% for single-resonance targeting — a 15%
improvement for slot-based mechanisms. The benefit is PRECISION: pair matching doubles
hit rate per targeted slot, most valuable for slot-targeting (D2, D4). For bonus
injection (D1, D5), the bottleneck is fire rate, not per-bonus precision, so pair
matching adds less value. The breakthrough is real but mechanism-dependent.

All pair-based algorithms require explaining "ordered pair (first two symbols)" — one
extra concept justified by the data.

## Proposed Best Algorithm

**Dual-Threshold Pair Guarantee (2,5) — D4 Standalone**

**One-sentence:** "Track the ordered symbol pair (first, second) of each 2+ symbol
card you draft; at 2 matching picks one pack slot is pair-matched, at 5 a second slot
is pair-matched, and remaining slots are random."

**Why D4 (2,5) wins after discussion:**
1. Crosses 2.0 comfortably (2.48 S/A)
2. Simplest one-sentence description of any algorithm crossing 2.0
3. Transparent — players know exactly when thresholds activate
4. 50% random slots preserve splash and meaningful choice
5. 0.82 stddev passes variance target
6. Perfect archetype balance (all 8 within 1.3-pick window)
7. Zero player decisions

**Why I abandoned the D4+D5 hybrid:** Agent 4 showed D4 (2,5) alone gives 2.48 S/A
vs the hybrid's 2.10. The conditional trigger trades 0.38 S/A for 0.39 stddev — a bad
trade when D4's 0.82 already passes variance. The hybrid adds complexity without
improving on D4 with lower thresholds.

**Strong alternative: D2 cap=0.50** (2.61 S/A, conv ~6-7, 0.97 stddev). Better raw
numbers but formula-based, harder to explain, less transparent. The 2-3 pick
convergence advantage over D4 is marginal in a 30-pick draft.

**Remaining concern:** Conv pick 9.2 misses 5-8 target. Could count 1-symbol cards at
0.5 weight toward both possible pairs to accelerate accumulation. Worth testing in
Round 5.
