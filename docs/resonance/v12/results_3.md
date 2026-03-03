# V12 Simulation 3 Results: Hybrid 2 — Progressive N + Steep Biased Contraction

## Algorithm

Hybrid 2: 120-card starting pool, 5 AIs with gradual avoidance (20% pick 3 to
90% pick 12), 60/20/0 refills with 2.0x open-lane bias and 40% S/A targeting,
progressive N (4/8/12). 1000 drafts x 30 picks x 3 strategies.

## Full Scorecard

| Metric | Target | Committed | PowerChaser | SignalReader |
|--------|--------|-----------|-------------|--------------|
| M1 | >= 3 | 1.10 | 1.11 | 1.13 |
| M2 | <= 2 | 0.33 | 0.34 | 0.34 |
| **M3** | **>= 2.0** | **0.37** | **0.32** | **0.40** |
| M4 | >= 0.5 | 1.92 | 2.40 | 1.73 |
| M5 | 5-8 | 13.77 | 17.75 | 11.38 |
| M6 | 60-90% | 25.7% | 18.4% | 28.5% |
| M7 | < 40% | 10.2% | 9.8% | 10.5% |
| M9 | >= 0.8 | 0.52 | 0.44 | 0.55 |
| M10 | <= 2 | 18.5 | 19.6 | 18.1 |
| M11' | >= 2.5 | 0.26 | 0.24 | 0.29 |
| M12 | >= 0.3 | 0.028 | — | — |
| M13 | 6-10 | 8.2 | — | — |
| M14 | 4-7 | 8.2 | — | — |

**M3 = 0.37: FAILS by 5.4x.** Only M4 and M7 pass targets.

## Per-Archetype M3

| Archetype | M3 | S/A Rate |
|-----------|:--:|:--------:|
| Warriors/Midrange | 0.561 | 50% |
| Sacrifice/Abandon | 0.526 | 50% |
| Self-Discard | 0.383 | 40% |
| Self-Mill/Reanim | 0.365 | 40% |
| Blink/Flicker | 0.291 | 30% |
| Flash/Tempo | 0.288 | 25% |
| Ramp/SpiritAnimal | 0.283 | 25% |
| Storm/Spellslinger | 0.245 | 30% |

M3 correlates directly with sibling S/A rate. Tide-pair (50%) gets 2x the M3 of Zephyr-pair (25%).

## AI Avoidance Timeline

AI inference correctly identifies the player's archetype by **pick 8.2** (98%
of drafts). Avoidance ramps: 0% (picks 1-2), 20% (3-5), 45% (6-8), 70%
(9-11), 90% (12+). The open-lane depletion comparison works well.

## Pool Contraction Trajectory

| Pick | Pool | Arch% | S/A |
|:----:|:----:|:-----:|:---:|
| 1 | 120 | 12.5% | 11.0 |
| 10 | 66 | 16.3% | 3.7 |
| 11 (refill) | 120 | 17.5% | 10.3 |
| 20 | 66 | 18.2% | 3.0 |
| 21 (refill) | 80 | 18.4% | 4.9 |
| 25 | 56 | 17.5% | 1.6 |
| 30 | 26 | 13.0% | 0.3 |

Pool contracts 120 to 26 (4.6:1). Density peaks at 18.4% — far below 45-55%
needed. **S/A at pick 25 = 1.6: binding constraint confirmed.**

## Oversampling Analysis

Pack quality at picks 6+: p10=0, p25=0, p50=0, p75=1, p90=1. 75% of packs
contain zero on-archetype S/A. Resonance ranking selects by symbol (shared
across 2-4 archetypes), not by specific archetype, diluting the benefit 2-4x.

## Consecutive Bad Packs

Avg max consecutive packs below 1.5 S/A: committed=18.5, signal_reader=18.1.
Target <= 2. Catastrophic player experience.

## Draft Traces

**Trace 1 (Committed, Self-Mill):** Arch density reaches 42% by pick 25 with
8 S/A remaining. Despite strong concentration, PackS/A=0 for most picks because
resonance ranking draws from multiple archetypes sharing Stone/Tide symbols.

**Trace 2 (Signal Reader, Blink):** S/A drops from 8 to 0 by pick 20. Low S/A
rate (30%) means exhaustion hits early. By pick 25: 5 Blink cards in pool, 0 S/A.

## Comparison to V9

| Metric | V9 | V12 H2 | Gap |
|--------|:--:|:------:|:---:|
| M3 | 2.70 | 0.37 | -2.33 |
| M5 | 9.6 | 13.8 | -4.2 |
| M6 | 86% | 25.7% | -60pp |
| M10 | 3.8 | 18.5 | -14.7 |
| M11' | 3.25 | 0.26 | -2.99 |

## Self-Assessment

**Is physical pool contraction + AI avoidance + oversampling viable?** No.

Three structural barriers:

1. **S/A exhaustion:** Player consumes ~10 S/A over 30 picks. Starting 11 +
   ~9 from refills = 20 total, but by pick 25 only 1.6 remain in pool. No
   oversampling produces M3=2.0 from 1.6 S/A in 56 cards.

2. **Density ceiling (18%):** 2.0x bias raises density from 12.5% to 18% — a
   44% improvement but 2.5x short of the 45% needed. Refills dilute faster
   than avoidance concentrates.

3. **Resonance breadth:** Symbol matching selects from 2-4 archetypes sharing
   the same symbols, diluting oversampling 2-4x vs archetype-specific ranking.

**Positive:** AI avoidance works (98% accuracy by pick 8). Pool contraction is
transparent and real. The face-up pool creates a readable drafting dynamic.

**Recommendation:** V9 fallback required. V12's face-up pool with AI avoidance
should enhance V9's narrative, not replace its contraction engine.
