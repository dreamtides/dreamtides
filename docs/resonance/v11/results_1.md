# SIM-1 Results: Baseline — Pure Balanced Refills

3 rounds x 10 picks, 120-card pool, balanced 60-card refills, 5 Level 0 AIs
(saturation threshold 10), no refill bias, Graduated Realistic fitness, 1000 sims.

## Full Scorecard

| Metric | Target | Committed | Signal-Reader | Power-Chaser | Pass? |
|--------|--------|-----------|---------------|-------------|-------|
| M1: Unique archs w/ S/A (picks 1-5) | >= 3 | 2.73 | 2.71 | 2.76 | FAIL |
| M2: Max S/A emerging (picks 1-5) | <= 2 | 1.42 | 1.41 | 1.42 | PASS |
| M3: S/A for committed (picks 6+) | >= 2.0 | 0.25 | 0.42 | 0.50 | FAIL |
| M4: Off-archetype cards (picks 6+) | >= 0.5 | 4.55 | 4.32 | 4.19 | PASS |
| M5: Convergence pick | 5-8 | 1.0 | 5.0 | 2.0 | FAIL |
| M6: Deck concentration (% S/A) | 60-90% | 23.7% | 32.0% | 12.8% | FAIL |
| M7: Run-to-run overlap | < 40% | 0.0% | 0.0% | 0.0% | PASS |
| M8: Arch freq max | < 20% | 14.3% | 21.6% | 14.9% | PASS |
| M8: Arch freq min | > 5% | 9.0% | 4.3% | 10.3% | PASS |
| M9: StdDev S/A per pack (6+) | >= 0.8 | 0.50 | 0.64 | 0.69 | FAIL |
| M10: Max consec bad packs (6+) | <= 2 | 21.4 | 18.0 | 15.6 | FAIL |
| M11': S/A committed (picks 20+) | >= 2.5 | 0.22 | 0.29 | 0.51 | FAIL |
| M12: Signal - Committed M3 | >= 0.3 | — | 0.17 | — | FAIL |

## Open-Lane vs AI-Lane Breakdown (Committed Strategy)

- Committed to open lane: 38.3% of drafts
- Committed to AI lane: 61.7% of drafts
- M3 (open-lane only): 0.33
- M3 (AI-lane only): 0.20

Signal-reader commits to open lane 86% of time (M3 open=0.43, AI=0.37).

## Per-Archetype M3

| Flash/Te: 0.21 | Blink/Fl: 0.25 | Storm/Sp: 0.24 | Self-Dis: 0.26 |
| Self-Mil: 0.27 | Sacrific: 0.28 | Warriors: 0.29 | Ramp/Spi: 0.22 |

All archetypes cluster around 0.21-0.28 with no meaningful differentiation.

## Round-by-Round Pool Composition

Average cards per archetype at round start (aggregated):

| Moment | Avg Open-Lane | Avg AI-Lane | Total |
|--------|:---:|:---:|:---:|
| R1 start | 15.0 | 15.0 | 120 |
| R2 (post-refill) | 15.6 | 14.7 | 120 |
| R3 (post-refill) | 15.8 | 14.5 | 120 |

## Pack Quality Distribution (S/A per Pack, Picks 6+)

| Strategy | p10 | p25 | p50 | p75 | p90 |
|----------|:---:|:---:|:---:|:---:|:---:|
| Committed | 0.00 | 0.00 | 0.00 | 0.00 | 1.00 |
| Signal-Reader | 0.00 | 0.00 | 0.00 | 1.00 | 1.00 |
| Power-Chaser | 0.00 | 0.00 | 0.00 | 1.00 | 1.00 |

## Consecutive Bad Pack Analysis

Committed strategy: 57% of drafts have ALL picks 6+ below
1.5 S/A (max streak = 25). Median max streak: 25.
Min streak: 6, p25: 18, p75: 25.
This is catastrophic — nearly every draft is a continuous dry spell.

## S/A Density Trajectory

S/A density = (S/A cards for committed archetype) / (pool size), at key picks:

| Pick | Label | S/A Density |
|:----:|-------|:-----------:|
| 1 | R1 start | 0.0902 |
| 5 | R1 mid | 0.0659 |
| 10 | R1 end | 0.0503 |
| 11 | R2 start (post-refill) | 0.0698 |
| 15 | R2 mid | 0.0620 |
| 20 | R2 end | 0.0523 |
| 21 | R3 start (post-refill) | 0.0694 |
| 25 | R3 mid | 0.0586 |
| 30 | R3 end | 0.0482 |

## Draft Traces

### Committed Player Trace

```
Trace: Committed | Committed: Sacrifice/Abandon (OPEN) | Convergence: pick 1

Pick  1 (R1): pool=120, S/A=0, picked Self-Discard (pwr=0.7)
Pick  5 (R1): pool= 96, S/A=0, picked Warriors/Mid (pwr=2.3)
Pick 10 (R1): pool= 66, S/A=1, picked Sacrifice/Ab (pwr=4.3)
--- REFILL: 60 balanced cards added, pool -> 120 ---
Pick 11 (R2): pool=120, S/A=0, picked Sacrifice/Ab (pwr=4.0)
Pick 15 (R2): pool= 96, S/A=0, picked Self-Discard (pwr=0.7)
Pick 20 (R2): pool= 66, S/A=0, picked Storm/Spells (pwr=4.8)
--- REFILL: 60 balanced cards added, pool -> 120 ---
Pick 21 (R3): pool=120, S/A=1, picked Warriors/Mid (pwr=6.3)
Pick 25 (R3): pool= 96, S/A=0, picked Self-Discard (pwr=1.7)
Pick 30 (R3): pool= 66, S/A=0, picked Self-Discard (pwr=0.1)

Deck: 30 cards, 5 S/A (17%), avg S/A/pack picks 6+: 0.20
```

### Signal-Reader Trace

```
Trace: Signal-Reader | Committed: Self-Discard (OPEN) | Convergence: pick 5

Pick  1 (R1): pool=120, S/A=0, picked Blink/Flicke (pwr=8.9)
Pick  5 (R1): pool= 96, S/A=3, picked Self-Mill/Re (pwr=1.1)
Pick 10 (R1): pool= 66, S/A=0, picked Blink/Flicke (pwr=6.1)
--- REFILL: 60 balanced cards added, pool -> 120 ---
Pick 11 (R2): pool=120, S/A=2, picked Self-Discard (pwr=5.6)
Pick 15 (R2): pool= 96, S/A=0, picked Self-Discard (pwr=7.2)
Pick 20 (R2): pool= 66, S/A=0, picked Sacrifice/Ab (pwr=0.8)
--- REFILL: 60 balanced cards added, pool -> 120 ---
Pick 21 (R3): pool=120, S/A=0, picked Sacrifice/Ab (pwr=2.2)
Pick 25 (R3): pool= 96, S/A=0, picked Self-Mill/Re (pwr=3.4)
Pick 30 (R3): pool= 66, S/A=0, picked Blink/Flicke (pwr=0.0)

Deck: 30 cards, 10 S/A (33%), avg S/A/pack picks 6+: 0.44
```

## Comparison to V9 and V10

| Metric | V9 Hybrid B | V10 Best (Hybrid X) | SIM-1 Committed | SIM-1 Signal-Reader |
|--------|:-----------:|:-------------------:|:---------------:|:-------------------:|
| M3 | 2.70 | 0.84 | 0.25 | 0.42 |
| M11/M11' | 3.25 | 0.69 | 0.22 | 0.29 |
| M10 | 3.8 | — | 21.4 | 18.0 |
| M5 | 9.6 | — | 1.0 | 5.0 |
| M6 | 86% | — | 23.7% | 32.0% |
| M12 | — | — | — | 0.17 |

## Self-Assessment

**FAIL.** SIM-1 fails on: M3 (0.25 < 2.0), M11' (0.22 < 2.5), M10 (21.4 > 2), M12 (0.17 < 0.3).

SIM-1 fails more severely than the design prediction of M3 1.3-1.5. Three factors:

**1. Low S/A density.** ~10-13 S/A cards per archetype in a 120-card pool (8-11%).
A pack of 5 yields expected 0.4-0.55 S/A for any archetype. M3 = 2.0 requires
~24-30% density, only achievable through contraction or massive refill bias.

**2. Random lane selection.** The committed player picks an AI lane 62.5% of the time.
Open-lane M3 (0.33) > AI-lane M3 (0.20), but both far below 2.0.

**3. Refill reset.** Balanced refills restore uniformity, washing out AI-depletion gradients.

SIM-1 committed M3 (0.25) is below V10's best (0.84). V10 used smarter commitment
(picks 5-6, favoring open lanes) and PACK_SIZE=4. The signal-reader (0.42) performs
better by selecting open lanes 87% of the time, but cannot overcome the density problem.

**Calibration value:** SIM-1 establishes that pure balanced refills produce M3 far below 2.0.
SIM-2+ algorithms must achieve roughly 8x improvement over this baseline.
