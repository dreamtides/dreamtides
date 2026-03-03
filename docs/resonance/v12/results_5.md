# V12 SIM-5 Results: Design 2 Champion -- Steep Contraction + N=8

## Full Scorecard (Committed Player, Graduated Realistic)

| Metric | Value | Target | Status |
|--------|:-----:|--------|:------:|
| M1 (exploration diversity) | 5.01 | >= 3 | PASS |
| M2 (pre-commit S/A) | 1.11 | <= 2 | PASS |
| M3 (post-commit S/A/pack) | 1.33 | >= 2.0 | **FAIL** |
| M4 (off-arch variety) | 2.67 | >= 0.5 | PASS |
| M5 (convergence pick) | 13.5 | 5-8 | **FAIL** |
| M6 (deck concentration) | 72.2% | 60-90% | PASS |
| M7 (run-to-run overlap) | 20% | < 40% | PASS |
| M8 (arch frequency) | 15.6%/8.9% | <=20%/>=5% | PASS |
| M9 (S/A stddev) | 0.96 | >= 0.8 | PASS |
| M10 (max bad streak) | 6.21 | <= 2 | **FAIL** |
| M11' (late S/A/pack) | 1.18 | >= 2.5 | **FAIL** |
| M12 (signal-reader edge) | 0.52 | >= 0.3 | PASS |
| M13 (AI avoidance pick) | 11.5 | 6-10 | **FAIL** |
| M14 (AI inference pick) | 11.0 | 4-7 | **FAIL** |

**Result: 8/14 passed. M3 = 1.33, far below 2.0.**

Signal-reader M3 = 1.84 (closer to target but still fails). The M12 edge of 0.52
is robust -- signal reading works. The mechanism produces differentiation, just not
enough absolute quality.

## Per-Archetype M3

| Archetype | M3 | Frequency |
|-----------|:--:|:---------:|
| Warriors/Sacrifice (Tide, 50% sibling) | 1.46 / 1.57 | 11.1% / 13.4% |
| Self-Discard/Self-Mill (Stone, 40%) | 1.41 / 1.44 | 14.2% / 13.7% |
| Storm/Blink (Ember, 30%) | 1.31 / 1.14 | 12.5% / 10.6% |
| Flash/Ramp (Zephyr, 25%) | 1.13 / 1.06 | 15.6% / 8.9% |

Higher sibling rates produce modestly higher M3, but all 8 archetypes fail the 2.0
target. The spread (1.06 to 1.57) is 0.51, reflecting the graduated fitness model.

## AI Avoidance Timeline

The inference mechanism is the simulation's critical failure point. AIs begin
forming inferences at pick 3-5 but almost always infer the **wrong** archetype.
The rolling-window depletion rate comparison is confounded by 5 other AIs drawing
simultaneously. With 6 drafters removing cards, the depletion signal from a single
player is drowned in noise.

- **M13 (avg first avoidance pick):** 11.5 (target 6-10) -- too late
- **M14 (earliest correct inference):** 11.0 (target 4-7) -- too late
- Trace evidence: AIs confidently infer "Ramp" or "Sacrifice" when the player is
  drafting "Self-Discard." The depletion rate for the player's archetype is not
  anomalous enough to distinguish from random variation.
- Avoidance resets at each refill boundary (picks 11, 21) because the pool
  composition changes dramatically, wiping the depletion history.

The avoidance weight itself reaches 0.7-0.9 quickly, but it is applied to the
wrong archetype. The player's actual archetype receives no protection from AI
competition. This is the **primary structural failure**: the public-information
inference mechanism cannot reliably identify the player's archetype from aggregate
depletion patterns in a 6-drafter environment.

## Pool Contraction Trajectory

| Pick | Pool Size | Player Arch | S/A | Density |
|:----:|:---------:|:-----------:|:---:|:-------:|
| 5 | 90 | 10.4 | 18.4 | 11.5% |
| 10 | 60 | 4.8 | 9.6 | 8.0% |
| 11 (post-refill) | 114 | 12.0 | 22.0 | 10.5% |
| 15 | 90 | 8.2 | 15.4 | 9.1% |
| 20 | 60 | 4.6 | 8.9 | 7.6% |
| 21 (post-refill) | 74 | 6.1 | 11.7 | 8.3% |
| 25 | 50 | 3.7 | 7.0 | 7.4% |
| 30 | 20 | 1.0 | 1.9 | 5.1% |

The pool contracts from 120 to 20 as designed, but **density decreases rather than
increases** -- from 12.5% at start to 5.1% at pick 30. The 2.0x biased refills
add open-lane cards but the AIs, unable to correctly identify the player's
archetype, take player-archetype cards at roughly the same rate as any other
archetype. The concentration mechanism breaks down entirely when avoidance targets
the wrong archetype.

S/A at pick 25 = 7.0 (across all archetype cards that are S/A for the committed
arch). This counts cross-archetype S/A (sibling cards), which inflates the number.
The player's primary archetype cards in pool drop from 15 to 1.0 by pick 30.

## Oversampling Analysis

| N | Expected M3 | vs N=4 lift |
|:-:|:-----------:|:-----------:|
| 4 (uniform) | 0.63 | -- |
| 8 (actual) | 1.26 | +0.63 |
| 10 (fallback) | 1.58 | +0.95 |
| 12 (Design 3) | 1.89 | +1.26 |

N=8 provides exactly 2x the M3 of N=4, confirming the oversampling doubles the
sampling rate as expected. Even N=12 projects only M3 = 1.89, still below 2.0.
The bottleneck is pool density, not oversampling factor. With density stuck at
7-11% (not the 40-50% the design assumed), no reasonable N achieves M3 = 2.0.

## Pack Quality Distribution (picks 6+)

| Percentile | S/A Count |
|:----------:|:---------:|
| p10 | 0 |
| p25 | 1 |
| p50 | 1 |
| p75 | 2 |
| p90 | 3 |

Median pack has 1 S/A card. 25% of packs have zero S/A. This produces the bad
streak problem: M10 = 6.21, with streaks of 10+ zero-S/A packs occurring
regularly.

## Draft Traces

**Committed Player (Trace #1):** Commits to Self-Discard at pick 5 (open lane).
AIs infer "Ramp" as player's archetype (incorrect). Avoidance activates at pick 5
against Ramp, not Self-Discard. Player finds S/A in 25/30 picks (83% deck
concentration). Post-refill packs occasionally deliver 3-4 S/A (picks 8, 12, 19,
23) but many packs have 0-1 S/A. The biased refill at pick 10 adds 23 Storm and
26 Warriors cards to the pool, heavily favoring open lanes.

**Signal-Reader (Trace #1):** Commits to Self-Mill at pick 5 (finding it has most
S/A in pool). Despite Self-Mill being assigned to an AI, the signal reader
achieves 19/30 S/A (63%). Pack quality is volatile (4 S/A at pick 7, then 0 S/A
at picks 22, 26-28).

## Comparison to V9 Baseline

| Metric | V9 Hybrid B | V11 SIM-4 | V12 Design 2 | Delta vs V9 |
|--------|:-----------:|:---------:|:------------:|:-----------:|
| M3 | 2.70 | 0.83 | 1.33 | -1.37 |
| M5 | 9.6 | -- | 13.5 | +3.9 |
| M6 | 86% | -- | 72% | -14% |
| M10 | 3.8 | -- | 6.21 | +2.41 |
| M11' | 3.25 | -- | 1.18 | -2.07 |

Design 2 improves on V11 SIM-4 (M3 1.33 vs 0.83) through biased refills and
oversampling, but remains far below V9. The improvement over V11 comes primarily
from the N=8 oversampling (which exactly doubles effective M3), not from AI
avoidance, which is inaccurate and applies protection to wrong archetypes.

## Self-Assessment: Is N=8 Sufficient or Is N=12 Needed?

**Neither N=8 nor N=12 is sufficient.** The simulation conclusively demonstrates
that the binding constraint is not the oversampling factor but the **failure of
the AI inference mechanism**. With density at 5-11% (not the assumed 40-50%),
even N=48 would only reach M3 = 48 * 0.08 = 3.84 -- but from a 20-card pool, you
cannot draw 48 cards. The real problem sequence:

1. **AI inference fails:** Rolling-window depletion tracking cannot distinguish the
   player's archetype from noise in a 6-drafter environment. AIs avoid the wrong
   archetype in most games.
2. **Without correct avoidance, density never builds:** AIs take player-archetype
   cards at the baseline rate (~12.5% of their picks). The pool contracts in size
   but the player's archetype contracts at the same rate.
3. **Biased refills add volume, not concentration:** The 2.0x open-lane bias adds
   more cards for the 3 open lanes, but the player's specific archetype is only
   1 of 3 open lanes. Net benefit to the player's archetype is ~1.5x, not enough
   to overcome the lack of avoidance.

**What would fix it:** Either (a) grant AIs direct knowledge of the player's
archetype (Level 2, dishonest), or (b) make the pool contraction mechanism
independent of AI inference accuracy (which is essentially V9's approach). The
public-information inference model is too noisy with 6 drafters. It might work
with 2-3 drafters but not with 6.

**The critic was right:** N=8 is on a knife's edge, but the knife is not where we
expected. The knife is not S/A count vs pool size -- it is inference accuracy. The
entire Design 2 thesis (that AIs can infer and avoid the player's archetype from
public depletion patterns) is empirically falsified by this simulation. The M3
improvement over V11 (1.33 vs 0.83) comes entirely from N=8 oversampling applied
to the same ~12% density pool that V11 had. AI avoidance contributes approximately
zero because it targets the wrong archetype.
