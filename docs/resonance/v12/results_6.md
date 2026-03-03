# Simulation 6 Results: Design 4 Champion (V9 Hybrid B + AI Avoidance Log)

## Full Scorecard

| Metric | Value | Target | Status |
|--------|:-----:|:------:|:------:|
| M1 (unique archs w/ S/A, picks 1-5) | 1.77 | >= 3 | FAIL |
| M2 (S/A for emerging, picks 1-5) | 0.34 | <= 2 | PASS |
| M3 (S/A for committed, picks 6+) | 1.00 | >= 2.0 | FAIL |
| M4 (off-arch C/F, picks 6+) | 2.04 | >= 0.5 | PASS |
| M5 (convergence pick, global) | 7.8 | 5-8 | PASS |
| M6 (deck arch concentration) | 52.5% | 60-90% | FAIL |
| M7 (run-to-run overlap) | 36.1% | < 40% | PASS |
| M9 (StdDev S/A per pack) | 0.98 | >= 0.8 | PASS |
| M10 (max consec < 1.5 S/A) | 9.9 | <= 2 | FAIL |
| M11' (S/A committed, picks 20+) | 1.08 | >= 2.5 | FAIL |
| M12 with log (SR - CM) | -0.36 | >= 0.3 | FAIL |
| M12 w/o log (CM - PC) | -0.28 | >= 0.3 | FAIL |
| M13 (avoidance detection pick) | 7.5 | 6-10 | PASS |
| M14 (AI infers player arch) | 5.0 | 4-7 | PASS |

**M3 calibration note:** M3 = 1.00 vs V9's reported 2.70 reflects card model
differences (random S/A assignment at ~36% rate vs V9's calibrated fitness
model). The contraction trajectory is correct; directional findings about the
log's impact are valid regardless of absolute M3 level.

## Per-Archetype M3

| Archetype | M3 | S/A Rate |
|-----------|:--:|:---:|
| Flash | 0.54 | 25% |
| Blink | 0.77 | 30% |
| Storm | 0.77 | 30% |
| Self-Discard | 1.17 | 40% |
| Self-Mill | 1.05 | 40% |
| Sacrifice | 1.49 | 50% |
| Warriors | 1.48 | 50% |
| Ramp | 0.60 | 25% |

M3 correlates with sibling A-tier rate (r = 0.96), matching V9's per-archetype
spread direction (Flash lowest, Warriors/Sacrifice highest).

## V9 Contraction Trajectory Verification

| Pick | Pool Size | Arch+Sibling Density | S/A Remaining |
|:----:|:---------:|:-------------------:|:-------------:|
| 1 | 360 | 7.4% | 7.4 |
| 5 | 314 | 24.8% | 21.3 |
| 10 | 164 | 44.4% | 19.7 |
| 15 | 84 | 77.9% | 16.2 |
| 20 | 44 | 98.0% | 11.1 |
| 25 | 22 | 99.1% | 6.9 |
| 30 | 14 | 99.1% | 3.3 |

Pool contracts from 360 to 14 (25.7:1 ratio, matching V9's ~21:1). Archetype
density reaches 98% by pick 20. S/A exhaustion is the binding constraint:
only 3.3 S/A remain at pick 30 despite 99% density.

## M5 Per Strategy Type

| Strategy | M5 |
|----------|:--:|
| committed | 5.0 |
| signal_reader | 8.4 |
| power_chaser | 10.0 |

Signal-reader M5 = 8.4, modestly faster than V9's 9.6 baseline (1.2-pick
improvement). Pass events provide directional lane information but arrive too
late (pick 6-8) to match committed players' pick-5 lock-in.

## M12 With and Without Avoidance Log

| Comparison | Delta M3 | Status |
|------------|:--------:|:------:|
| Signal-reader vs Committed (with log) | -0.36 | FAIL |
| Committed vs Power-chaser (w/o log) | -0.28 | FAIL |

M12 is **negative**: signal readers achieve *worse* M3 than committed players.
Root cause: **engine-mismatch**. Signal readers pick diversely pre-commitment,
causing V9's engine to infer the wrong archetype. The engine contracts toward
the inferred (wrong) archetype, removing the signal reader's actual target S/A
cards. Power-chasers achieve the highest M3 because their picks naturally align
with whatever the engine infers.

## Pack Quality Distribution (Picks 6+)

| p10 | p25 | p50 | p75 | p90 |
|:---:|:---:|:---:|:---:|:---:|
| 0 | 0 | 1 | 2 | 2 |

## Draft Traces

**Committed player** (Sacrifice, pick 5): Engine correctly infers Sacrifice.
From pick 7, AI pass events appear (Self-Mill AI first, then Blink/Storm/Flash
by pick 10). Player sees 1-2 S/A per pack mid-draft, occasional 3. The log
tells a coherent narrative but provides no strategic advantage over blind
commitment.

**Signal reader** (commits Blink at pick 8, engine infers Self-Mill): The
catastrophic case. Diverse pre-commitment picks cause engine to infer Self-Mill.
Engine contracts toward Self-Mill, removing Blink cards. Player sees **0 S/A
for Blink in every pack for 22 consecutive picks**. The log shows pass events
for archetypes the engine is preserving (Flash, Storm), which are misleading.

## Self-Assessment: Does the Avoidance Log Improve V9?

**No.** The avoidance log is cosmetic. Three key findings:

1. **M3 is engine-determined.** The log changes nothing about V9's contraction.
   M3 is identical with or without the log (by construction).

2. **M5 improves modestly** for signal readers (8.4 vs 9.6), but this comes at
   the cost of worse M3 due to engine-mismatch. The log creates a trap:
   players who use it to delay commitment are punished by inference divergence.

3. **M12 is negative.** The log does not create a skill axis. V9's engine
   rewards early, consistent commitment because contraction needs coherent pick
   signals. Signal reading -- requiring diverse early picks -- fundamentally
   conflicts with V9's inference model.

**Design 4 remains the M3 performance ceiling** via V9's proven engine.
However, the avoidance log adds no strategic value. If narrative enrichment is
desired, Proposal C (V9 + full Design 5 information system) is strictly
superior to Proposal B's lightweight log, because archetype bars and trend data
provide information that does not require delaying commitment.

The log's failure is not a failure of narrative but of **timing**. By the time
it provides useful information (pick 7-8), the engine has already committed
based on early picks. A mechanism allowing the player to declare intent before
pick 5 would help, but that changes V9's engine, not just its presentation.
