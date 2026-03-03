# V12 SIM-4 Results: Design 1 Champion (N=4 Isolation Baseline)

## Configuration

- Starting pool: 120 cards (15 per archetype, 8 archetypes)
- 5 AIs assigned to 5 of 8 archetypes (3 open lanes)
- AI avoidance: gradual ramp from pick 3 (30% -> 60% -> 85%)
- Refills: 60/0/0 (single refill after R1), 2.0x open-lane bias
- Pack construction: N=4 uniform random (NO oversampling)
- 1000 drafts x 30 picks x 3 player strategies

## Full Scorecard

| Metric | Committed | Signal | Target | Status |
|--------|:---------:|:------:|:------:|:------:|
| M1 | 4.97 | 4.97 | >= 3 | PASS |
| M2 | 1.11 | 1.01 | <= 2 | PASS |
| **M3** | **0.66** | **0.99** | >= 2.0 | **FAIL** |
| M4 | 3.22 | 2.89 | >= 0.5 | PASS |
| M5 | 22.59 | 17.42 | 5-8 | FAIL |
| M6 | 49.8% | 60.1% | 60-90% | FAIL/PASS |
| M7 | 13% | 16% | < 40% | PASS |
| M8 | 13.9%/10.6% | 29.7%/2.5% | <=20%/>=5% | PASS/FAIL |
| M9 | 0.71 | 0.85 | >= 0.8 | FAIL/PASS |
| M10 | 13.07 | 8.38 | <= 2 | FAIL |
| M11' | 0.48 | 0.77 | >= 2.5 | FAIL |
| **M12** | **0.337** | -- | >= 0.3 | **PASS** |
| M13 | 6.15 | 6.06 | 6-10 | PASS |
| M14 | 21.28 | 13.32 | 4-7 | FAIL |

**Committed player passes 6/13 metrics. Signal reader passes 7/13.**

## CALIBRATION BASELINE

**M3 = 0.658 (committed) / 0.995 (signal reader)**

This is the M3 achieved by AI avoidance + pool contraction WITHOUT any
oversampling. Other V12 simulations should measure their N=8/N=12 oversampling
contribution as delta above this baseline. To reach M3 = 2.0, oversampling must
add approximately 1.34 (committed) or 1.00 (signal reader).

## Per-Archetype M3

| Archetype | M3 (Committed) | Frequency | M3 (Signal) | Frequency |
|-----------|:---:|:---:|:---:|:---:|
| Flash | 0.56 | 12.5% | 1.06 | 6.2% |
| Blink | 0.58 | 12.7% | 0.94 | 6.5% |
| Storm | 0.71 | 13.7% | 0.97 | 12.1% |
| Self-Discard | 0.69 | 12.2% | 1.02 | 15.5% |
| Self-Mill | 0.73 | 13.9% | 1.03 | 20.0% |
| Sacrifice | 0.72 | 13.7% | 0.97 | 29.7% |
| Warriors | 0.65 | 10.6% | 1.01 | 7.5% |
| Ramp | 0.59 | 10.7% | 0.98 | 2.5% |

The committed player drafts archetypes uniformly (10-14% each). The signal
reader concentrates heavily on high-sibling-rate archetypes (Sacrifice 29.7%,
Self-Mill 20.0%) because these have the most S/A remaining in the pool. The
signal reader's M3 advantage (~0.34 over committed) comes from choosing open
lanes 80% of the time vs 43% for committed players.

## Pool Contraction Trajectory

| Pick | Pool | Arch Cards | Arch % | S/A | S/A % |
|:----:|:----:|:----------:|:------:|:---:|:-----:|
| 5 | 90 | 10.5 | 11.7% | 18.5 | 20.6% |
| 10 | 60 | 6.6 | 11.0% | 11.5 | 19.2% |
| 11 (post-refill) | 114 | 13.6 | 11.9% | 23.4 | 20.5% |
| 15 | 90 | 10.5 | 11.7% | 17.9 | 19.9% |
| 20 | 60 | 5.9 | 9.8% | 10.2 | 17.1% |
| 25 | 30 | 2.1 | 7.0% | 3.7 | 12.3% |
| 28 | 12 | 0.4 | 3.6% | 0.8 | 7.1% |
| 30 | 0 | 0.0 | 0.0% | 0.0 | 0.0% |

**Critical finding: archetype density DECREASES over the draft** (from 11.7%
at pick 5 down to 7.0% at pick 25 and 0% at pick 30). The avoidance mechanism
fails to create concentration because:

1. The committed player picks their own archetype out of the pool faster than
   AIs deplete competing archetypes
2. AI inference accuracy peaks at only ~33% and degrades in late draft (as the
   pool becomes small and depletion signals become noisy)
3. S/A exhaustion is severe: from 18.5 S/A at pick 5 to 0.8 at pick 28
4. The pool empties completely by pick 30 -- total exhaustion

## Pack Quality Distribution (Picks 6+)

| Percentile | S/A per Pack |
|:----------:|:---:|
| p10 | 0 |
| p25 | 0 |
| p50 | 0 |
| p75 | 1 |
| p90 | 2 |

The median pack contains zero S/A cards for the committed archetype. 75% of
packs have 0 or 1 S/A. This is structurally insufficient for a satisfying
draft experience and confirms that N=4 uniform random cannot deliver quality
packs from these pool compositions.

## AI Avoidance Analysis

- **M13 (avoidance onset): 6.15** -- AIs begin behavioral change around pick 6,
  within the target range of 6-10. The avoidance ramp itself is correctly timed.
- **M14 (inference accuracy): 21.28** -- AIs do not reliably identify the
  player's archetype until very late (well outside the 4-7 target). Peak accuracy
  is only 33% around picks 6-14, dropping to single digits by pick 25+.
- **Root cause of M14 failure**: With 6 drafters taking from 8 archetypes,
  depletion signals are heavily confounded. The AI cannot distinguish the
  player's 1 card/cycle depletion from the background noise of 5 other AIs
  taking cards. The 3-pick sliding window is too narrow to build reliable
  inference from such noisy data.

## Draft Traces

**Committed Trace #1**: Player commits to Blink (a closed lane -- Blink AI is
present). Despite avoidance reaching 85%, the Blink AI never correctly infers
the player's archetype (0/5 correct throughout). The player finds 18/30 S/A
cards (60% concentration) but mostly from sibling archetypes (Flash, Storm),
not from the pack mechanism delivering Blink cards. Pool empties to 0 by pick 30.

**Signal Reader Trace #1**: Player reads pool and commits to Sacrifice (an open
lane). The pool retains Sacrifice cards better -- 10 remain at R3 start vs 1
for Blink in trace #1. AI inference intermittently correct (5/5 at picks 7-9
for this run). Finds 15/30 S/A (50%). The open-lane advantage is visible in
the pool composition snapshots.

## Comparison to V9 and V11

| Metric | V9 Hybrid B | V11 SIM-4 | V12 Design 1 | vs V9 | vs V11 |
|--------|:---:|:---:|:---:|:---:|:---:|
| M3 | 2.70 | 0.83 | 0.66 | -2.04 | -0.17 |
| M5 | 9.6 | -- | 22.59 | +12.99 | -- |
| M6 | 86% | -- | 49.8% | -36% | -- |
| M10 | 3.8 | -- | 13.07 | +9.27 | -- |
| M11' | 3.25 | -- | 0.48 | -2.77 | -- |

V12 Design 1 is **below V11 SIM-4** on M3 (0.66 vs 0.83). This is despite
adding AI avoidance and biased refills. The explanation: V11 SIM-4 used 4
rounds with 3 refill events (48/36/21/0 = 105 total refill cards), keeping
the pool larger for longer. V12 Design 1 uses only 1 refill (60 cards),
causing the pool to exhaust completely by pick 30. The pool exhaustion
destroys late-draft quality.

## Self-Assessment

**Is AI avoidance + physical pool contraction a viable concentration mechanism
without oversampling?** No. With N=4 uniform packs, M3 = 0.66 -- far below
the 2.0 target. The fundamental problems are:

1. **Pool exhaustion**: The 60/0/0 refill schedule provides only 180 total
   cards (120 + 60) for 180 total removals (6 per cycle x 30 picks). The pool
   reaches zero by pick 30. Late-draft packs draw from pools of 6-12 cards,
   which sounds like it should produce concentration but S/A has been exhausted.

2. **AI inference failure**: AI inference accuracy of 30-33% means avoidance is
   barely effective. AIs avoid the wrong archetype ~70% of the time. The
   player's 1-card-per-cycle signal is lost in 5-AI background noise.

3. **S/A exhaustion**: From 18.5 S/A at pick 5 to 0.8 at pick 28. The player
   and AIs together deplete S/A faster than the biased refill restores it.

4. **Density paradox**: Despite avoidance, archetype density *decreases* from
   12% to 7% because the player's own consumption outpaces the concentration
   benefit from AI avoidance.

**The positive finding**: M12 = 0.337 (PASS). Signal readers who commit to
open lanes achieve meaningfully higher M3 (0.99 vs 0.66). The open-lane
mechanism works -- the signal reader's 80% open-lane commit rate vs the
committed player's 43% rate is the primary differentiator. This validates
the core V12 thesis that reading the pool produces better outcomes.

**Calibration conclusion**: Oversampling must contribute approximately +1.34
M3 (for committed players) or +1.00 (for signal readers) to reach M3 = 2.0.
At N=8 from a 30-card pool with 3.7 S/A, oversampling yields 8 * 3.7/30 = 0.99
S/A per pack -- insufficient. At N=12 from the same pool, 12 * 3.7/30 = 1.48
S/A per pack. Neither reaches 2.0. **The S/A exhaustion problem, not pool size,
is the binding constraint.** Oversampling designs must solve S/A preservation
(through stronger biased refills or higher S/A rates in refill cards) to have
any hope of reaching M3 = 2.0.
