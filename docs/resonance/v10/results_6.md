# Results 6: D3 Competitive Pressure with Safety Valve (revised)

## Full Scorecard (Config B Calibrated, Graduated Realistic, Committed Player)

| Metric | Target | Value | Status |
|--------|--------|-------|--------|
| M1 (early variety) | >= 3.0 | 3.04 | PASS |
| M2 (early S/A) | <= 2.0 | 0.60 | PASS |
| M3 (post-commit S/A) | >= 2.0 | 0.51 | **FAIL** |
| M4 (off-archetype) | >= 0.5 | 3.49 | PASS |
| M5 (convergence pick) | 5-8 | 24.9 | **FAIL** |
| M6 (concentration) | 60-90% | 43% | **FAIL** |
| M7 (variety) | < 40% | 7.1% | PASS |
| M9 (stddev) | >= 0.8 | 0.65 | **FAIL** |
| M10 (consec bad) | <= 2.0 | 14.8 | **FAIL** |
| M11 (late S/A) | >= 3.0 | 0.48 | **FAIL** |

**Pessimistic (committed, -10pp sibling rates):** M3=0.47, M10=15.9, M11=0.43

**6 of 10 metrics fail.** This algorithm does not pass.

## Per-Archetype M3

| Archetype | M3 | M11 |
|-----------|-----|-----|
| Flash | 0.46 | 0.41 |
| Blink | 0.46 | 0.41 |
| Storm | 0.46 | 0.40 |
| Self-Discard | 0.52 | 0.49 |
| Self-Mill | 0.51 | 0.47 |
| Sacrifice | 0.56 | 0.50 |
| Warriors | 0.57 | 0.56 |
| Ramp | 0.46 | 0.43 |
| **Spread** | **0.118** | **0.16** |

Higher-sibling-rate pairs (Warriors/Sacrifice at 50%) slightly outperform lower pairs (Flash/Ramp at 25%), but all are catastrophically below target. The spread is modest because the algorithm fails uniformly.

## Pack Quality Distribution (Picks 6+, Committed)

| Percentile | S/A Count |
|------------|-----------|
| P10 | 0 |
| P25 | 0 |
| P50 | 0 |
| P75 | 1 |
| P90 | 1 |

The median pack contains zero S/A cards. 75% of packs have at most 1 S/A card. This is near-random selection from an unconcentrated pool.

## Consecutive Bad Packs

Average max consecutive packs below 1.5 S/A: **14.8 picks** (target: <=2). Worst observed: 24 consecutive bad packs. Over 42% of drafts had 15+ consecutive bad packs. The player effectively never sees a good pack stretch.

## Open Lane vs Contested Lane

| Condition | M3 | M10 | M11 | M6 |
|-----------|-----|-----|-----|----|
| Open lane | 0.78 | 10.5 | 0.84 | 57% |
| Contested | 0.47 | 15.5 | 0.42 | 41% |

The open lane advantage exists (0.78 vs 0.47 M3) but even the open lane fails all core metrics by wide margins. Being in the uncontested lane provides a 66% improvement but from an extremely low base.

## AI Behavior Summary

- **All 7000 AIs saturated** (100%). Mean saturation pick: 12.8 (median: 13).
- 95.2% saturated during picks 11-15; 4.8% during picks 16-20.
- Average archetype cards per AI at draft end: 18.8 of 30 total drafted.
- After saturation, AIs shift to 50/30/20 mix but this only moderately reduces archetype pressure. By pick 13, each AI has already consumed ~10 of its 40 archetype cards, and the remaining 30 pool cards for that archetype are still competing with 7 other archetypes' surviving cards.

**Saturation timing is correct per design.** At 1 card/round with 85% archetype rate, AIs accumulate ~0.85 archetype cards/round and hit threshold 10 at pick ~12. This matches the observed mean of 12.8. The saturation mechanic triggers exactly when intended but produces negligible player-facing benefit.

## Draft Traces

**Trace 1 (Committed, Warriors, Open Lane):** Player in uncontested lane. Despite no AI competing for Warriors, picks show 0-1 S/A per pack through pick 22. Only 13/29 cards (45%) were S/A. The pool starts with 40 Warriors cards in 360 total (11.1%). Without targeted contraction, this ratio barely improves even as the pool shrinks to 12 cards.

**Trace 2 (Signal Reader, Storm, Contested Lane):** Storm AI competes directly. Player sees 0-1 S/A per pack throughout. 13/29 S/A (45%). The contested lane trace is nearly indistinguishable from open lane performance, confirming that the concentration mechanism is absent.

## Config A: As-Designed (2/AI + 10 cull)

The design-specified parameters (14 AI picks + 10 cull + 1 player = 25/round) exhaust the 360-card pool by pick 14. With only 14 picks, M11 is undefined (no picks 15+). M3 = 0.58. This configuration is structurally non-viable for a 30-pick draft.

## V9 Hybrid B Comparison

| Metric | V9 Hybrid B | D3 CompPress | Delta |
|--------|-------------|-------------|-------|
| M3 | 2.70 | 0.51 | -2.19 |
| M10 | 3.8 | 14.8 | +11.0 |
| M11 | 3.25 | 0.48 | -2.77 |
| M5 | 9.6 | 24.9 | +15.3 |

D3 underperforms V9 Hybrid B by catastrophic margins on every contested metric. The gap is not tunable -- it reflects a structural deficiency.

## Root Cause Analysis

The algorithm fails because **AI drafting does not create archetype concentration for the player.** V9's contraction explicitly removes cards irrelevant to the player's inferred archetype, increasing the S/A ratio in the remaining pool from ~11% toward ~50-75% by late draft. D3's AIs remove cards from *their own* archetypes -- 7 different archetypes, each depleting its own lane. The player's archetype concentration in the remaining pool changes minimally:

- Start: 40 Warriors in 360 = 11.1%
- After 7 AIs each remove ~0.85 of their own archetype per round for 12 rounds: Warriors pool is still ~40 (no AI targets Warriors in open lane), total pool ~216. Warriors ratio = 40/216 = 18.5%.
- The supplemental cull removes lowest-power cards indiscriminately, providing zero archetype targeting.

The ratio improvement from 11.1% to ~18.5% yields M3 ~ 0.74 (18.5% * 4 pack slots). This matches the observed open-lane M3 of 0.78. **The math is correct but the mechanism is fundamentally wrong** -- AI drafting creates a gentle natural concentration that peaks around 2x the starting ratio, far below V9's 5-7x concentration.

## Does Saturation Help?

Comparing early (pre-saturation) vs late (post-saturation) S/A rates: the open lane shows M3 improvement from ~0.6 early to ~0.84 late, a modest 40% increase. The saturation mechanic contributes a real but small effect -- when AIs stop taking their archetype cards as aggressively, adjacent archetypes benefit slightly. But this effect is dwarfed by the structural concentration gap.

## Key Question Answer

**Does the saturation mechanic alone, without Open Table's 3-lane structure, produce competitive metrics?** No. The saturation mechanic produces the most human-like AI behavior in the field (the critic correctly identified this) but is insufficient to drive archetype concentration. The mechanism is architecturally sound as a *pick-logic refinement* but cannot substitute for targeted pool contraction. Its value lies in being transplanted into a design (like Hybrid X) that has a separate contraction engine providing the mathematical foundation.

## Self-Assessment

This algorithm fails comprehensively. The design's post-critique revision honestly acknowledged this risk: "The saturation mechanic does not solve the contraction problem." Simulation confirms this finding with precision.

**What would fix it:** Replace indiscriminate power-based culling with archetype-aware contraction (remove cards least relevant to the player's inferred archetype). This would effectively recreate V9's contraction engine underneath the AI drafter layer -- which is exactly what the critic's "supplemental culling is V9 contraction in disguise" finding predicts. The saturation mechanic would then serve its intended role: adding realistic late-draft easing on top of a contraction engine that provides the mathematical foundation.
