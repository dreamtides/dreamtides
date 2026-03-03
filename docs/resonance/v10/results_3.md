# Results: D4 Escalating Aggression (Simulation Slot 3)

## Full Scorecard

| Metric | Value | Target | Pass | Pessimistic |
|--------|------:|--------|:----:|------------:|
| M1 (archs w/ S/A, picks 1-5) | 3.83 | >= 3 | PASS | — |
| M2 (S/A emerging, picks 1-5) | 1.55 | <= 2 | PASS | — |
| M3 (S/A committed, picks 6+) | 0.20 | >= 2.0 | FAIL | 0.19 |
| M4 (off-archetype, picks 6+) | 3.04 | >= 0.5 | PASS | — |
| M5 (convergence pick) | 28.1 | 5-8 | FAIL | — |
| M6 (deck concentration) | 22.2% | 60-90% | FAIL | — |
| M7 (run-to-run overlap) | 23.4% | < 40% | PASS | — |
| M8 (arch freq max/min) | 14.5/10.4% | <20/>5% | PASS | — |
| M9 (S/A stddev, picks 6+) | 0.42 | >= 0.8 | FAIL | — |
| M10 (max consec bad packs) | 22.43 | <= 2 | FAIL | 22.44 |
| M11 (S/A committed, picks 15+) | 0.07 | >= 3.0 | FAIL | 0.07 |

**Result: 5/11 pass.** Catastrophic failure on all convergence metrics (M3, M5, M6, M10, M11). Only early-draft openness (M1, M2), variety (M7, M8), and splash (M4) pass.

## Per-Archetype M3

| Archetype | All | Open Lane | Contested | N |
|-----------|----:|----------:|----------:|--:|
| Flash | 0.18 | 0.52 | 0.12 | 133 |
| Blink | 0.22 | 0.55 | 0.12 | 145 |
| Storm | 0.17 | 0.49 | 0.12 | 119 |
| Self-Discard | 0.20 | 0.59 | 0.13 | 135 |
| Self-Mill | 0.22 | 0.56 | 0.14 | 113 |
| Sacrifice | 0.19 | 0.58 | 0.14 | 142 |
| Warriors | 0.21 | 0.57 | 0.14 | 104 |
| Ramp | 0.19 | 0.49 | 0.12 | 109 |

Even open lanes reach only M3 ~ 0.54 — far below 2.0. Contested lanes: 0.12-0.14.

## Pack Quality Distribution (Picks 6+)

| p10 | p25 | p50 | p75 | p90 |
|----:|----:|----:|----:|----:|
| 0 | 0 | 0 | 0 | 1 |

Median pack contains zero S/A cards. At p90, only 1.

## Consecutive Bad Packs

55% of drafts hit 25 consecutive bad packs (maximum). 81% have 20+. The draft is functionally non-convergent.

## Draft Traces

**Committed Player (Self-Discard, contested):** 27 S/A cards remain at commitment (pick 5). By pick 10, only 2 S/A remain. By pick 12 (pool at floor), zero. Picks 13-30 draw from a stale 28-card pool of generics and off-archetype junk.

**Signal Reader (Self-Discard, contested):** Similar trajectory. 59 S/A at pick 1, 4 by pick 11, zero by pick 12. Even correct signal reading cannot overcome pool exhaustion.

## AI Behavior Summary

| Phase | Total AI Removal | Per Pick | Pool After (incl. cull) |
|-------|:----------------:|:--------:|:-----------------------:|
| Phase 1 (picks 1-5) | 72 | 14.5 | ~247 |
| Phase 2 (picks 6-10) | 105 | 21.0 | ~102 |
| Phase 3 (picks 11-15) | 53 | 10.6 | ~30 (floor) |
| Phase 4 (picks 16+) | 0 | 0 | Pool exhausted |

**Phase 4 never executes.** The pool reaches floor around pick 12. The design's signature feature — 5 cards/AI/round at 95% focus — never fires. Per-archetype AI removal: ~28 cards from each of the 7 AI archetypes (70% of each archetype's 40 cards).

## V9 Comparison

| Metric | D4 | V9 Hybrid B | Delta |
|--------|---:|:-----------:|------:|
| M3 | 0.20 | 2.70 | -2.50 |
| M5 | 28.1 | 9.6 | +18.5 |
| M10 | 22.43 | 3.8 | +18.6 |
| M11 | 0.07 | 3.25 | -3.18 |

D4 is catastrophically worse on every convergence metric. V9's 12% contraction preserves S/A density while concentrating the pool. D4's AIs deplete S/A cards preferentially (picking the best first), while market culling removes B-tier bridges. The combination strips both top and bottom, leaving only C/F generics.

## Key Question: Does Escalating Contraction Improve M5/M10?

**No.** The failure is structural: escalating AI pick rates plus market culling deplete the pool to floor before the escalation curve delivers convergence. The design's thesis — "early openness, late concentration" — is contradicted by math: Phase 4's intended concentration arrives after the pool is already empty of S/A cards.

The critical difference from V9: V9's contraction removes *low-relevance* cards, enriching the pool in S/A cards. D4's AIs remove *high-relevance* cards (the best cards for their archetypes), depleting S/A. AI drafting and pool contraction are not mathematically equivalent — they have opposite effects on S/A density.

## Self-Assessment

**Does not pass.** Fails 6/11 metrics. The failure is structural, not parametric:

1. **Pool exhaustion at pick 12.** 7 AIs removing 14-28 cards/round + 8 cull = 22-36/round against 360 cards. Floor reached before escalation completes.

2. **S/A preferential depletion.** AIs select highest-affinity cards first, correlating with S/A fitness. Remaining pool enriches in C/F, the opposite of V9's behavior.

3. **Market culling amplifies damage.** Removes B-tier bridges and accelerates depletion without convergence benefit.

**What would fix it:** (a) Far fewer AIs (3-4), (b) no market culling, (c) much higher pool floor, or (d) AIs that avoid player's archetype (Level 3 reactivity). Options (a)+(b) produce something resembling Hybrid Y. The escalation mechanic is not inherently broken — the total removal budget is. Escalation cannot converge an empty pool.
