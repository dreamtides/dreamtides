# SIM-2 Results: Hybrid 1 — Robustly Biased Pressure + N=12 + Floor

## Algorithm Summary

- Pool: 120 cards (15/arch), refills 50/30/0 with 2.0x open-lane bias
- Open-lane refills use 40% S/A rate (vs 36% baseline)
- 5 AIs (3 open lanes), avoidance ramp pick 5 (0%) to pick 12 (80%)
- N=4 (picks 1-5), N=12 (picks 6-30), visible resonance ranking
- Floor slot: guarantee 1 S/A in shown 4 if any S/A drawn
- 1000 simulations per strategy

## Full Scorecard

| Metric | Target | Committed | Signal | Power | Pass? |
|--------|--------|:---------:|:------:|:-----:|:-----:|
| M1: Unique archs S/A (1-5) | >= 3 | 2.26 | 2.25 | 2.25 | FAIL |
| M2: Max S/A emerging (1-5) | <= 2 | 1.21 | 1.21 | 1.22 | PASS |
| M3: S/A committed (6+) | >= 2.0 | 0.51 | 0.62 | 0.82 | FAIL |
| M4: Off-arch (6+) | >= 0.5 | 2.84 | 2.86 | 2.80 | PASS |
| M5: Convergence pick | 5-8 | 1.0 | 5.0 | 2.0 | FAIL |
| M6: Deck concentration | 60-90% | 42.4% | 47.2% | 19.7% | FAIL |
| M7: Run overlap | < 40% | 0.0% | 0.0% | 0.0% | PASS |
| M8: Arch freq max | < 20% | 14.4% | 17.0% | 14.1% | PASS |
| M8: Arch freq min | > 5% | 10.2% | 5.7% | 10.8% | PASS |
| M9: StdDev S/A (6+) | >= 0.8 | 0.62 | 0.66 | 0.72 | FAIL |
| M10: Consec bad (6+) | <= 2 | 18.9 | 17.4 | 13.9 | FAIL |
| M11': S/A (20+) | >= 2.5 | 0.30 | 0.36 | 0.91 | FAIL |
| M12: Signal - Committed | >= 0.3 | -- | 0.11 | -- | FAIL |
| M13: Avoidance detect | 6-10 | 7.8 | 8.4 | -- | PASS |
| M14: Inference correct | 4-7 | 7.7 | 8.3 | -- | FAIL |

## Per-Archetype M3 (Committed)

| Archetype | M3 | Sibling Rate |
|-----------|:---:|:-----------:|
| Flash/Tempo | 0.44 | 25% |
| Blink/Flicker | 0.46 | 30% |
| Storm/Spellslinger | 0.48 | 30% |
| Self-Discard | 0.53 | 40% |
| Self-Mill/Reanim | 0.52 | 40% |
| Sacrifice/Abandon | 0.59 | 50% |
| Warriors/Midrange | 0.60 | 50% |
| Ramp/SpiritAnim | 0.42 | 25% |

## AI Avoidance Timeline

| Pick | Inference Accuracy | Effective Avoidance |
|:----:|:------------------:|:-------------------:|
| 1 | 0% | 0% |
| 3 | 0% | 0% |
| 5 | 0% | 0% |
| 6 | 0% | 0% |
| 7 | 64% | 10% |
| 8 | 74% | 19% |
| 9 | 77% | 28% |
| 10 | 79% | 38% |
| 12 | 79% | 53% |
| 15 | 27% | 8% |
| 20 | 2% | 0% |
| 25 | 3% | 1% |
| 30 | 1% | 0% |

## Pool Contraction Trajectory

| Pick | Pool Size | S/A in Pool | Arch Density | N |
|:----:|:---------:|:-----------:|:------------:|:-:|
| 1 | 120 | 0.0 | 12.5% | 4 |
| 5 | 96 | 8.5 | 14.3% | 4 |
| 10 | 66 | 4.5 | 16.8% | 12 |
| 11 | 110 | 9.4 | 17.5% | 12 |
| 15 | 86 | 5.7 | 17.5% | 12 |
| 20 | 56 | 2.4 | 14.5% | 12 |
| 21 | 80 | 4.8 | 14.9% | 12 |
| 25 | 56 | 2.0 | 12.3% | 12 |
| 28 | 38 | 1.3 | 10.2% | 12 |
| 30 | 26 | 1.1 | 8.7% | 12 |

## S/A Counts at Key Picks

- Pick 20: 2.4 S/A remaining
- Pick 25: 2.0 S/A remaining
- Pick 30: 1.1 S/A remaining

## Oversampling Analysis

- Picks 6-15 (N=12, pool ~80-110): avg S/A = 0.73
- Picks 16-20 (N=12, pool ~40-80): avg S/A = 0.49
- Picks 21-30 (N=12, pool ~10-50): avg S/A = 0.29

## Pack Quality Distribution (S/A per Pack, Picks 6+)

| Strategy | p10 | p25 | p50 | p75 | p90 |
|----------|:---:|:---:|:---:|:---:|:---:|
| Committed | 0.0 | 0.0 | 0.0 | 1.0 | 1.0 |
| Signal | 0.0 | 0.0 | 1.0 | 1.0 | 1.0 |
| Power | 0.0 | 0.0 | 1.0 | 1.0 | 2.0 |

## Consecutive Bad Pack Analysis

- Median max consecutive bad: 19
- p25=14, p75=25, p90=25
- All packs bad (streak=25): 34%
- 5+ consecutive bad: 100%

## Pool Composition Trajectory

Average cards per archetype at key picks:

| Pick | Top Arch | 2nd | 3rd | Avg Others | Total |
|:----:|:--------:|:---:|:---:|:----------:|:-----:|
| 1 | 15.0 | 15.0 | 15.0 | 15.0 | 120 |
| 5 | 12.0 | 12.0 | 12.0 | 12.0 | 96 |
| 10 | 8.3 | 8.3 | 8.3 | 8.2 | 66 |
| 11 | 14.2 | 14.1 | 14.0 | 13.5 | 110 |
| 15 | 11.1 | 11.0 | 11.0 | 10.6 | 86 |
| 20 | 7.2 | 7.2 | 7.2 | 6.9 | 56 |
| 21 | 10.2 | 10.2 | 10.1 | 9.9 | 80 |
| 25 | 7.2 | 7.2 | 7.1 | 6.9 | 56 |
| 30 | 3.4 | 3.4 | 3.3 | 3.2 | 26 |

## Draft Traces

### Committed Player
```
Trace: Committed | Blink/Flicker (OPEN) | conv pick 1
AIs: ['Self-M', 'Storm/', 'Flash/', 'Ramp/S', 'Self-D']
Open: ['Blink/', 'Sacrif', 'Warrio']

   1 R1 N= 4: pool=120 SA_pool=0 SA_pack=0 pick=Self-Dis
   2 R1 N= 4: pool=114 SA_pool=10 SA_pack=0 pick=Storm/Sp
   3 R1 N= 4: pool=108 SA_pool=10 SA_pack=0 pick=Flash/Te
   4 R1 N= 4: pool=102 SA_pool=9 SA_pack=1 pick=Storm/Sp
   5 R1 N= 4: pool= 96 SA_pool=8 SA_pack=0 pick=Self-Dis
   6 R1 N=12: pool= 90 SA_pool=8 SA_pack=1 pick=Blink/Fl
   7 R1 N=12: pool= 84 SA_pool=6 SA_pack=1 pick=Blink/Fl [AI->Blink/ c=0.3 a=7%]
   8 R1 N=12: pool= 78 SA_pool=5 SA_pack=0 pick=Blink/Fl [AI->Blink/ c=0.7 a=24%]
   9 R1 N=12: pool= 72 SA_pool=5 SA_pack=2 pick=Storm/Sp [AI->Blink/ c=0.9 a=43%]
  10 R1 N=12: pool= 66 SA_pool=4 SA_pack=1 pick=Blink/Fl [AI->Blink/ c=0.9 a=54%]
  --- REFILL +50 ---
  11 R2 N=12: pool=110 SA_pool=7 SA_pack=2 pick=Blink/Fl [AI->Blink/ c=0.9 a=65%]
  12 R2 N=12: pool=104 SA_pool=5 SA_pack=1 pick=Blink/Fl [AI->Blink/ c=0.9 a=76%]
  13 R2 N=12: pool= 98 SA_pool=4 SA_pack=1 pick=Blink/Fl [AI->Flash/ c=0.7 a=56%]
  14 R2 N=12: pool= 92 SA_pool=3 SA_pack=0 pick=Ramp/Spi [AI->Flash/ c=0.5 a=36%]
  15 R2 N=12: pool= 86 SA_pool=3 SA_pack=2 pick=Blink/Fl [AI->Flash/ c=0.5 a=36%]
  16 R2 N=12: pool= 80 SA_pool=1 SA_pack=0 pick=Self-Dis [AI->Flash/ c=0.4 a=31%]
  17 R2 N=12: pool= 74 SA_pool=1 SA_pack=0 pick=Blink/Fl [AI->Storm/ c=0.5 a=36%]
  18 R2 N=12: pool= 68 SA_pool=1 SA_pack=0 pick=Flash/Te [AI->Self-D c=0.5 a=36%]
  19 R2 N=12: pool= 62 SA_pool=1 SA_pack=0 pick=Blink/Fl [AI->Self-D c=0.4 a=31%]
  20 R2 N=12: pool= 56 SA_pool=1 SA_pack=1 pick=Blink/Fl [AI->Self-D c=0.3 a=26%]
  --- REFILL +30 ---
  21 R3 N=12: pool= 80 SA_pool=3 SA_pack=1 pick=Blink/Fl [AI->Self-D c=0.3 a=26%]
  22 R3 N=12: pool= 74 SA_pool=2 SA_pack=0 pick=Blink/Fl [AI->Self-D c=0.3 a=26%]
  23 R3 N=12: pool= 68 SA_pool=1 SA_pack=0 pick=Storm/Sp [AI->Self-D c=0.3 a=22%]
  24 R3 N=12: pool= 62 SA_pool=1 SA_pack=0 pick=Storm/Sp [AI->Flash/ c=0.5 a=36%]
  25 R3 N=12: pool= 56 SA_pool=1 SA_pack=1 pick=Storm/Sp [AI->Flash/ c=0.4 a=31%]
  26 R3 N=12: pool= 50 SA_pool=0 SA_pack=0 pick=Blink/Fl [AI->Flash/ c=0.3 a=26%]
  27 R3 N=12: pool= 44 SA_pool=0 SA_pack=0 pick=Storm/Sp [AI->Flash/ c=0.3 a=22%]
  28 R3 N=12: pool= 38 SA_pool=0 SA_pack=0 pick=Blink/Fl [AI->Self-M c=0.5 a=36%]
  29 R3 N=12: pool= 32 SA_pool=0 SA_pack=0 pick=Ramp/Spi [AI->Self-M c=0.4 a=31%]
  30 R3 N=12: pool= 26 SA_pool=0 SA_pack=0 pick=Self-Mil [AI->Self-M c=0.3 a=26%]

Deck: 30 cards, 12 S/A (40%), avg S/A 6+: 0.56
```

### Signal-Reader
```
Trace: Signal-Reader | Warriors/Midrange (OPEN) | conv pick 5
AIs: ['Self-M', 'Storm/', 'Flash/', 'Ramp/S', 'Self-D']
Open: ['Blink/', 'Sacrif', 'Warrio']

   1 R1 N= 4: pool=120 SA_pool=0 SA_pack=0 pick=Self-Dis
   2 R1 N= 4: pool=114 SA_pool=0 SA_pack=0 pick=Ramp/Spi
   3 R1 N= 4: pool=108 SA_pool=0 SA_pack=0 pick=Warriors
   4 R1 N= 4: pool=102 SA_pool=0 SA_pack=0 pick=Warriors
   5 R1 N= 4: pool= 96 SA_pool=0 SA_pack=0 pick=Sacrific
   6 R1 N=12: pool= 90 SA_pool=13 SA_pack=0 pick=Warriors
   7 R1 N=12: pool= 84 SA_pool=13 SA_pack=0 pick=Warriors [AI->Warrio c=0.7 a=16%]
   8 R1 N=12: pool= 78 SA_pool=13 SA_pack=0 pick=Self-Mil [AI->Warrio c=0.9 a=33%]
   9 R1 N=12: pool= 72 SA_pool=13 SA_pack=1 pick=Sacrific [AI->Warrio c=0.9 a=43%]
  10 R1 N=12: pool= 66 SA_pool=12 SA_pack=1 pick=Sacrific [AI->Warrio c=0.7 a=40%]
  --- REFILL +50 ---
  11 R2 N=12: pool=110 SA_pool=16 SA_pack=1 pick=Sacrific [AI->Warrio c=0.7 a=48%]
  12 R2 N=12: pool=104 SA_pool=15 SA_pack=0 pick=Sacrific [AI->Warrio c=0.7 a=56%]
  13 R2 N=12: pool= 98 SA_pool=15 SA_pack=1 pick=Sacrific [AI->Flash/ c=0.7 a=56%]
  14 R2 N=12: pool= 92 SA_pool=14 SA_pack=1 pick=Warriors [AI->Flash/ c=0.6 a=48%]
  15 R2 N=12: pool= 86 SA_pool=13 SA_pack=1 pick=Sacrific [AI->Flash/ c=0.5 a=40%]
  16 R2 N=12: pool= 80 SA_pool=10 SA_pack=1 pick=Warriors [AI->Flash/ c=0.4 a=34%]
  17 R2 N=12: pool= 74 SA_pool=8 SA_pack=1 pick=Warriors [AI->Flash/ c=0.4 a=29%]
  18 R2 N=12: pool= 68 SA_pool=5 SA_pack=1 pick=Warriors [AI->Flash/ c=0.3 a=25%]
  19 R2 N=12: pool= 62 SA_pool=2 SA_pack=0 pick=Warriors [AI->Flash/ c=0.3 a=21%]
  20 R2 N=12: pool= 56 SA_pool=1 SA_pack=0 pick=Warriors [AI->Self-D c=0.5 a=36%]
  --- REFILL +30 ---
  21 R3 N=12: pool= 80 SA_pool=6 SA_pack=1 pick=Sacrific [AI->Self-D c=0.5 a=36%]
  22 R3 N=12: pool= 74 SA_pool=4 SA_pack=0 pick=Sacrific [AI->Self-D c=0.5 a=36%]
  23 R3 N=12: pool= 68 SA_pool=4 SA_pack=0 pick=Warriors [AI->Self-D c=0.4 a=31%]
  24 R3 N=12: pool= 62 SA_pool=3 SA_pack=0 pick=Sacrific [AI->Flash/ c=0.5 a=36%]
  25 R3 N=12: pool= 56 SA_pool=2 SA_pack=1 pick=Warriors [AI->Flash/ c=0.4 a=31%]
  26 R3 N=12: pool= 50 SA_pool=1 SA_pack=0 pick=Sacrific [AI->Flash/ c=0.3 a=26%]
  27 R3 N=12: pool= 44 SA_pool=1 SA_pack=0 pick=Sacrific [AI->Flash/ c=0.3 a=22%]
  28 R3 N=12: pool= 38 SA_pool=1 SA_pack=1 pick=Sacrific [AI->Self-M c=0.5 a=36%]
  29 R3 N=12: pool= 32 SA_pool=0 SA_pack=0 pick=Warriors [AI->Flash/ c=0.5 a=36%]
  30 R3 N=12: pool= 26 SA_pool=0 SA_pack=0 pick=Self-Mil [AI->Ramp/S c=0.5 a=36%]

Deck: 30 cards, 12 S/A (40%), avg S/A 6+: 0.48
```

## Comparison to V9 Baseline and V11

| Metric | V9 Hybrid B | V11 SIM-4 | SIM-2 Committed | SIM-2 Signal |
|--------|:-----------:|:---------:|:---------------:|:------------:|
| M3 | 2.70 | 0.83 | 0.51 | 0.62 |
| M11' | 3.25 | -- | 0.30 | 0.36 |
| M10 | 3.8 | -- | 18.9 | 17.4 |
| M5 | 9.6 | -- | 1.0 | 5.0 |
| M6 | 86% | -- | 42.4% | 47.2% |
| M12 | -- | -- | -- | 0.11 |

## Self-Assessment

**FAIL.** Critical failures: M3 (0.51 < 2.0), M11' (0.30 < 2.5), M10 (18.9 > 2), M12 (0.11 < 0.3).

### Root Cause Analysis

M3 = 0.51 falls short of the 2.0 target.

**Pool contraction works as designed.** The pool contracts from 120 to ~26 cards by pick 30, matching the design's prediction of ~20 cards. The biased refills (50 after R1, 30 after R2) with 2.0x open-lane multiplier correctly favor the player's archetype in refill composition.

**The binding constraint is the sampling bottleneck, not avoidance or contraction.** Drawing 12 random cards from a pool of 60-110 cards where only 10-15% are on-archetype yields E[on-arch drawn] = 0.7-1.6. Of those, only 36% are S/A, giving E[S/A drawn] = 0.25-0.58. The 'best 4' ranking and floor slot can only show S/A cards that were randomly drawn in the first place.

**The oversampling N=12 is insufficient when the pool is large.** In the critical picks 6-20 window (pool ~60-110), N=12 draws only 11-20% of the pool. The probability of drawing the player's 3-6 S/A cards from 60-110 total is too low for M3 >= 2.0. Only when the pool contracts below ~25 cards (picks 27-30) does N=12 sample enough of the pool to reliably find S/A cards.

**S/A exhaustion compounds the problem.** The player consumes their own S/A through picks. Starting with ~5 S/A, refills add ~4-5 more, but the player picks ~8-10 S/A over 30 picks, leaving only ~1-2 S/A by pick 25 (2.0 observed). Even with perfect pool contraction, if only 1-2 S/A remain, M3 cannot reach 2.0.

**AI avoidance works but too slowly.** Inference accuracy reaches useful levels (~50%+) around pick 7, but the avoidance benefit is modest: AIs preferentially taking from non-player archetypes raises the player's archetype density from 12.5% to ~18% at best -- insufficient for M3 >= 2.0 via oversampling alone.

### Is AI avoidance + pool contraction + N=12 oversampling viable?

**Not viable.** The three mechanisms fail to produce meaningful concentration. The sampling bottleneck identified in V11 persists: drawing N=12 from a pool of 60-120 cards with 12% on-archetype density cannot achieve M3 >= 2.0 regardless of AI avoidance behavior. The pool must contract to <25 cards before oversampling becomes effective, but by that point S/A cards are nearly exhausted. V9 fallback recommended.
