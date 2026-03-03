# V12 Simulation 1 Results: Design 3 Champion (Moderate Pressure + Floor, N=12)

## Verdict: Structural Failure (M3 = 0.42)

Design 3's champion algorithm fails M3 by a factor of ~5x. The predicted M3 of
2.0-2.2 does not materialize. Both variants (B1 pair-affinity, B2 symbol-only)
produce M3 well below target. The three-mechanism combination (AI avoidance +
pool contraction + oversampling) does not achieve meaningful concentration.
V12's face-up physical contraction approach, as instantiated here, falls far
short of replacing V9's invisible contraction.

---

## Full Scorecard

| Metric | Target | B1 Committed | B1 Signal | B2 Committed | B2 Signal |
|--------|--------|:---:|:---:|:---:|:---:|
| M1 (unique archs w/ S/A, 1-5) | >= 3 | 1.10 | 1.10 | 1.12 | 1.10 |
| M2 (S/A emerging arch, 1-5) | <= 2 | 0.28 | 0.25 | 0.25 | 0.25 |
| **M3 (S/A committed, 6+)** | **>= 2.0** | **0.42** | **0.38** | **0.32** | **0.33** |
| M4 (off-arch C/F, 6+) | >= 0.5 | 2.65 | 2.74 | 2.97 | 2.99 |
| M5 (convergence pick) | 5-8 | 5.0 | 3.0 | 5.0 | 3.0 |
| M6 (deck concentration) | 60-90% | 22.7% | 22.2% | 22.6% | 23.4% |
| M7 (card overlap) | < 40% | 0.0% | 0.0% | 0.0% | 0.0% |
| M9 (StdDev S/A) | >= 0.8 | 0.77 | 0.73 | 0.58 | 0.59 |
| M10 (max consec < 1.5) | <= 2 | 17.6 | 18.2 | 20.8 | 20.4 |
| **M11' (S/A 20+)** | **>= 2.5** | **0.27** | **0.24** | **0.18** | **0.19** |
| M12 (signal - committed) | >= 0.3 | -0.04 | -- | 0.01 | -- |
| M13 (avoidance detect pick) | 6-10 | 9 | 9 | 9 | 9 |
| M14 (inference correct pick) | 4-7 | 9 | 8 | 9 | 8 |

Every metric except M4 and M5 misses its target. M10 = 17.6 means the average
draft has an 18-pack consecutive run with fewer than 1.5 S/A cards -- nearly the
entire post-commitment draft.

---

## Per-Archetype M3 Table (B1 Committed)

| Archetype | M3 | SA Rate |
|-----------|:---:|:---:|
| Flash/Tempo | 0.35 | 0.305 |
| Blink/Flicker | 0.40 | 0.275 |
| Storm/Spellslinger | 0.41 | 0.330 |
| Self-Discard | 0.43 | 0.350 |
| Self-Mill/Reanimator | 0.40 | 0.400 |
| Sacrifice/Abandon | 0.46 | 0.450 |
| Warriors/Midrange | 0.44 | 0.375 |
| Ramp/Spirit Animals | 0.40 | 0.305 |

Higher-SA-rate archetypes (Sacrifice, Warriors) show marginally better M3
(0.44-0.46) vs lower-rate archetypes (Flash, Ramp at 0.35-0.40), but none
approach the 2.0 target.

---

## AI Avoidance Timeline

| Pick | Avoidance Strength | Inference Accuracy |
|:----:|:------------------:|:-----------------:|
| 5 | 0.00 | 0.5% |
| 7 | 0.00 | 21.2% |
| 8 | 0.10 | 49.6% |
| 9 | 0.19 | 57.1% |
| 10 | 0.32 | 48.3% |
| 11 | 0.48 | 27.3% |
| 14 | 0.75 | 34.2% |
| 20 | 0.80 | 26.7% |
| 30 | 0.80 | 25.3% |

AI inference peaks at 57% accuracy at pick 9, then collapses to ~25-35% for the
remainder. The pick-10 refill resets pool composition, destroying the depletion
signal. The pick-20 refill causes the same collapse (pick 11-12 drop to 2.8%).
**AI avoidance is mechanically applying 80% weight reduction, but to the wrong
archetype most of the time.** Effective avoidance of the player's actual
archetype is approximately 0.80 * 0.30 = 0.24, far below the assumed ~0.80.

---

## Pool Contraction Trajectory

| Pick | Pool Size | Player S/A | Arch Density |
|:----:|:---------:|:----------:|:------------:|
| 1 | 120 | (pre-commit) | -- |
| 5 | 96 | (pre-commit) | -- |
| 8 | 78 | 3.1 | 13.1% |
| 10 | 66 | 1.1 | 11.0% |
| 11* | 110 | 3.8 | 13.2% |
| 15 | 86 | 1.1 | 11.2% |
| 20 | 56 | 0.1 | 8.3% |
| 21* | 80 | 2.0 | 11.0% |
| 25 | 56 | 0.1 | 6.9% |
| 30 | 26 | 0.0 | 1.9% |

*Refill events at picks 10 and 20.

Pool contracts to 26 cards by pick 30 (within the predicted 20-30 range), but
archetype density **decreases** over the draft from 13% to 1.9%. The
concentration mechanism completely fails. S/A count crashes to near-zero by
pick 20 and is not recovered by the pick-20 refill (only 2.0 S/A restored,
immediately consumed).

---

## Oversampling Analysis (B1 vs B2)

| Metric | B1 (pair-affinity) | B2 (symbol-only) | Delta |
|--------|:---:|:---:|:---:|
| M3 (committed) | 0.42 | 0.32 | +0.10 |
| M11' (picks 20+) | 0.27 | 0.18 | +0.09 |
| M10 | 17.6 | 20.8 | -3.2 |

Pair-affinity provides a +0.10 M3 advantage over symbol-only ranking. This is
consistent with the predicted 0.2-0.3 delta from Design 3's specification, but
neither variant approaches M3 = 2.0. The delta is irrelevant given the scale of
the miss.

---

## Pack Quality Distribution (picks 6+, B1 committed)

| Percentile | S/A in Pack |
|:----------:|:----------:|
| p10 | 0 |
| p25 | 0 |
| p50 | 0 |
| p75 | 1 |
| p90 | 2 |

The median pack contains **zero** S/A cards. Only the top 25% of packs contain
any S/A at all.

---

## Floor Slot Firing Rate

| Pick Band | Rate |
|-----------|:----:|
| 6-10 | 47.3% |
| 11-20 | 28.1% |
| 21-30 | 19.6% |

The floor slot fires less than half the time even in its best band (picks 6-10),
and drops to 19.6% in the endgame. This confirms the critic's warning: when S/A
is exhausted from the pool, the floor slot cannot fire because there are no S/A
cards in the N=12 draw.

## S/A Counts at Key Picks

| Pick | S/A in Pool |
|:----:|:----------:|
| 20 | 0.1 |
| 25 | 0.1 |
| 30 | 0.0 |

S/A exhaustion is total by pick 20. The predicted values (~5.3, ~3.5, ~2.5)
from Design 3's walkthrough do not materialize. The player and AIs together
consume all S/A cards within the first 20 picks.

---

## Consecutive Bad Pack Analysis

| Statistic | Value |
|-----------|:-----:|
| Mean | 17.6 |
| Median | 18 |
| p90 | 25 |
| Max | 25 |

Nearly every draft has an 18+ pack stretch with below-threshold S/A.

---

## Draft Traces

**Trace 1 (committed, B1, Self-Mill/Reanimator):** Convergence pick 5. Pool:
120 -> 90 -> 110 (refill) -> 80 -> 80 (refill) -> 50. S/A: 5 at pick 6, 3
post-first-refill, 0 by pick 15. M3 per pack: mostly zeros with three
scattered 1s across 25 packs. Deck: 30 cards, 6 S/A total.

**Trace 2 (committed, B1, Sacrifice/Abandon):** Convergence pick 5. Higher
SA-rate archetype (0.45). S/A: 7 at pick 6, drops faster. M3 per pack: several
1s and one 3 early, then mostly zeros. Deck: 30 cards, 10 S/A total --
substantially better than Trace 1 due to higher base SA rate, but still far
from target.

---

## Comparison to V9 Baseline

| Metric | V9 Hybrid B | V12 B1 | V12 B2 | V11 SIM-4 |
|--------|:---:|:---:|:---:|:---:|
| M3 | 2.70 | 0.42 | 0.32 | 0.83 |
| M10 | 3.8 | 17.6 | 20.8 | -- |
| M6 | 86% | 22.7% | 22.6% | -- |

V12 B1 achieves 15% of V9's M3. It also performs **worse than V11 SIM-4**
(0.42 vs 0.83). The primary cause is that V11 SIM-4 used balanced refills
which at least maintained pool size, while V12's declining refills shrink the
pool but AI inference inaccuracy (~30%) means avoidance fails to preserve the
player's cards, causing both pool shrinkage AND S/A depletion.

---

## Root Cause Analysis

Three structural failures compound:

1. **AI inference accuracy is catastrophically low (~30%).** The depletion-rate
   method cannot reliably identify the player's archetype from pool snapshots
   with 6 concurrent drafters. Refill events at picks 10 and 20 destroy the
   depletion signal entirely. At 30% accuracy with 80% avoidance strength,
   effective avoidance of the player's actual archetype is ~24% -- barely
   different from no avoidance at all.

2. **S/A exhaustion is total by pick 20.** Starting with ~5 S/A per archetype,
   the player consumes their own S/A through picks 6-20. AIs, unable to
   correctly identify the player's archetype, also consume player S/A cards.
   The 50-card biased refill at pick 10 adds ~3-4 S/A, but these are consumed
   within 5 picks. By pick 20, S/A = 0.1 on average.

3. **Archetype density never concentrates.** The predicted trajectory (12.5% ->
   50% by pick 30) assumed ~80% effective avoidance. Actual density declines
   from 13% to 2% because AIs are not meaningfully avoiding the player's
   archetype, so all archetypes deplete roughly equally while the player
   preferentially exhausts their own.

## Self-Assessment

AI avoidance + physical pool contraction + modest oversampling (N=12) is **not
a viable replacement for V9's invisible contraction** as implemented here. The
binding constraint is AI inference accuracy: without reliable identification of
the player's archetype, avoidance cannot preserve the player's S/A supply, and
the entire mechanism chain collapses. The design's predictions assumed effective
avoidance at ~80%, but the simulation demonstrates ~24% effective avoidance.

The face-up pool concept remains narratively appealing, but the three-mechanism
approach requires either (a) dramatically better AI inference (perhaps direct
observation of the player's picks, which violates the public-information
constraint), (b) much larger starting S/A counts to survive the depletion, or
(c) a fundamentally different concentration mechanism that does not depend on
AI behavior.

V9's fallback (Design 4) remains the only proven path to M3 >= 2.0.
