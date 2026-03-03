# Simulation Results: Hybrid X (Open Table + Saturation)

## Algorithm

D1 Open Table + D3 Saturation: 5 random AIs from 8 archetypes (3 open lanes), each picking 4 cards/round using pair-affinity. Saturation at 16 archetype cards shifts distribution from 85/5/10 to 50/30/20. Market culling removes cards least relevant to open archetypes at 12% contraction/round. Level 0 reactivity. 1000 drafts x 30 picks x 3 strategies.

## Scorecard (Committed Player)

| Metric | Grad | Pess | Target | Pass? |
|--------|------|------|--------|-------|
| M1 unique archs S/A 1-5 | 3.87 | -- | >= 3 | PASS |
| M2 S/A emerging 1-5 | 1.87 | -- | <= 2 | PASS |
| M3 S/A committed 6+ | **0.84** | 0.79 | >= 2.0 | **FAIL** |
| M4 off-arch 6+ | 3.16 | -- | >= 0.5 | PASS |
| M5 convergence pick | 6.0 | -- | 5-8 | PASS |
| M6 deck concentration | **51.3%** | -- | 60-90% | **FAIL** |
| M7 card overlap | 17.4% | -- | < 40% | PASS |
| M8 freq max/min | 15.8/11.1% | -- | <20/>5% | PASS |
| M9 stddev S/A | 0.92 | -- | >= 0.8 | PASS |
| M10 max consec <1.5 | **12.15** | 12.68 | <= 2 | **FAIL** |
| M11 S/A picks 15+ | **0.69** | 0.66 | >= 3.0 | **FAIL** |

**7/11 pass. Fails: M3, M6, M10, M11.**

## Per-Archetype M3

| Archetype | M3 | Archetype | M3 |
|-----------|------|-----------|------|
| Warriors | 0.88 | Self-Mill | 0.85 |
| Sacrifice | 0.90 | Flash | 0.80 |
| Self-Discard | 0.86 | Blink | 0.79 |
| Ramp | 0.88 | Storm | 0.73 |

Spread: 0.17 (0.73-0.90). Tide-primary archetypes slightly favored by higher sibling rates.

## Pack Quality Distribution (Picks 6+)

| p10 | p25 | p50 | p75 | p90 |
|-----|-----|-----|-----|-----|
| 0 | 0 | 1 | 1 | 2 |

Median pack: 1 S/A. 25% of packs contain zero. Only 10% reach 2+.

## Consecutive Bad Packs

| Max run <1.5 S/A | % drafts |
|------------------|----------|
| 0-2 | 1.3% |
| 3 | 3.4% |
| 4+ | **95.3%** |

95% of drafts experience 4+ consecutive dry packs. The algorithm's most severe failure.

## Draft Traces

**Trace 1 (Committed, Storm open):** AI: Flash/Self-Mill/Sacrifice/Warriors/Ramp. Picks 6-14 averaged 2.2 S/A (strong). Picks 15-30 dropped to 0.6 S/A as pool collapsed below 50. Drafted 17 Storm cards (57% concentration).

**Trace 2 (Signal, Warriors open):** AI: Flash/Storm/Self-Discard/Self-Mill/Sacrifice. Picks 6-17 averaged 1.25 S/A. Picks 18-30 averaged 0.23 S/A. Pool reached 12 by pick 30.

## AI Saturation Timing

Avg saturation: **pick 5.6** (median 5). Distribution: p10=5, p25=5, p50=5, p75=6, p90=7. Only 4.6% never saturated. Saturation is extremely fast: at 85% archetype rate with 4 picks/round, AIs reach 16 archetype cards in ~5 rounds -- before the player commits. The saturation phase (50/30/20) is the dominant AI behavior for picks 6-30. Post-saturation, AIs take ~2 archetype cards/round (down from ~3.4), with ~1.2 adjacent and ~0.8 generic. Saturation creates a visible shift but triggers too early to create a mid-draft signal the player could read.

## V9 Comparison

| Metric | V9 Hybrid B | Hybrid X | Delta |
|--------|------------|----------|-------|
| M3 | 2.70 | 0.84 | **-1.86** |
| M10 | 3.8 | 12.15 | +8.35 |
| M11 | 3.25 | 0.69 | **-2.56** |
| M5 | 9.6 | 6.0 | -3.6 (better) |

Hybrid X dramatically underperforms V9 on all concentration metrics. Only M5 improves (earlier convergence).

## Root Cause

The failure is structural. Level 0 reactivity cannot achieve V9-level concentration because:

1. **V9 targets the player's archetype; V10 cannot.** V9 removes cards with low relevance to the player's specific archetype. V10's Level 0 can only concentrate toward the 3 open archetypes generally. The player's chosen archetype is 1 of 3, diluting concentration by ~3x.

2. **Math ceiling:** Player's archetype = ~55/360 = 15% of initial pool. Even after aggressive AI depletion, the ratio reaches ~40-50% only when the pool is very small (<60 cards, ~6 picks remaining).

3. **Saturation too fast.** AIs reach 16 archetype cards by pick 5, making saturation the default state. The pre/post transition is invisible to the player. Raising threshold to 24+ would help observability but not concentration.

4. **Pool exhaustion.** 12% contraction depletes pool to ~12 by pick 30. Final 10 picks produce 0-1 S/A packs, destroying M11.

## Does Saturation Improve M10 vs D1 Baseline?

Negligibly. Saturation triggers at pick 5, before M10-relevant picks begin (pick 6+). The post-saturation AI behavior (50% archetype) is the only behavior the player experiences. D1 baseline without saturation would face the same pool dynamics. The saturation mechanic's theoretical benefit -- creating a second convergence pathway via AI easing -- is correct in concept but fails because the threshold is reached too early.

## Self-Assessment

**Hybrid X fails.** Passing 7/11 metrics masks the severity: the 4 failures (M3, M6, M10, M11) are the core concentration metrics that define draft quality. The failures are architectural, not parametric -- no tuning of contraction rate, cull strategy, or saturation threshold can bridge the gap while maintaining Level 0 reactivity. Fixing requires either Level 1+ reactivity (player-aware culling after commitment), a much larger pool (600+), or weighted pack construction.
