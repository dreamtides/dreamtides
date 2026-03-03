# Agent 7 Results: Surge Packs

## One-Sentence Algorithm

"Each drafted symbol adds tokens to that resonance (primary adds 2, others add
1); when any resonance reaches the threshold, auto-deduct those tokens and fill
3 of the next pack's 4 slots with random cards of that resonance, the fourth
slot random."

## Scorecard: Surge T=5/S=3 vs Lane Locking

| Metric | Target | Surge T=5/S=3 | Lane Locking |
|--------|--------|:---:|:---:|
| Picks 1-5 unique archs w/ S/A | >= 3 | **4.69** | 5.02 |
| Picks 1-5 S/A for emerging | <= 2 | **1.41** | 1.42 |
| Picks 6+ S/A committed | >= 2 | 1.79 | 1.83 |
| Picks 6+ off-archetype | >= 0.5 | **2.21** | 2.17 |
| Convergence pick | 5-8 | **5.9** | 8.2 |
| Deck concentration | 0.60-0.90 | **0.74** | 0.97 (FAIL) |
| S/A stddev (late) | >= 0.8 | **1.38** | 0.77 (FAIL) |
| Run-to-run overlap | < 0.40 | **0.08** | 0.08 |

At T=5/S=3, neither crosses 2.0 S/A. Surge Packs wins on convergence speed,
variance, and deck concentration.

## Parameter Sensitivity

| Thresh | Slots | Late S/A | StdDev | Conv Pick | Deck Conc | Surge% |
|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| **4** | **3** | **2.08** | 1.39 | **5.0** | 0.78 | 64% |
| 4 | 2 | 1.72 | 1.07 | 6.2 | 0.80 | 63% |
| 5 | 3 | 1.79 | 1.38 | 5.8 | 0.74 | 49% |
| 5 | 2 | 1.51 | 1.08 | 8.4 | 0.76 | 49% |
| 6 | 3 | 1.61 | 1.34 | 6.8 | 0.72 | 39% |
| 6 | 2 | 1.41 | 1.06 | 9.5 | 0.74 | 39% |

**Threshold=4, Slots=3 crosses 2.0 S/A (2.08).** Slot count matters more than
threshold: 3 slots outperforms 2 by 0.2-0.4 S/A. Lower threshold increases
surge frequency from 39% (T=6) to 64% (T=4).

## Surge Frequency Over Draft

| Stage | Surge % |
|-------|---------|
| Early (1-5) | 25% |
| Mid (6-10) | 50% |
| Late (16-30) | 54% |

Rare early, frequent post-commitment -- exactly as designed.

## Per-Archetype Convergence (T=5, S=3)

| Archetype | Conv Pick | | Archetype | Conv Pick |
|-----------|:-:|---|-----------|:-:|
| Flash | 6.0 | | Self-Mill | 5.6 |
| Blink | 5.7 | | Sacrifice | 6.2 |
| Storm | 5.8 | | Warriors | 5.9 |
| Self-Discard | 5.8 | | Ramp | 5.9 |

All within 0.6 picks (5.6-6.2). No archetype advantaged. Frequency: 11.8-13.3%.

## Pack Quality Variance

S/A distribution per pack (picks 6+):

| S/A | 0 | 1 | 2 | 3 | 4 |
|-----|:-:|:-:|:-:|:-:|:-:|
| Freq | 25.5% | 22.4% | 8.6% | 34.4% | 9.1% |

Bimodal: non-surge packs cluster at 0-1, surge packs at 3-4. This alternating
pulse is the algorithm's signature -- genuine valleys between peaks. Lane Locking
is unimodal at 2 (stddev 0.77, fails >= 0.8 target).

## Comparison to Agent 1 Baselines

**vs Lane Locking:** Surge T=4/S=3 crosses 2.0 (2.08 vs 1.83), converges 3
picks faster, nearly doubles variance, and achieves proper deck concentration.

**vs Pack Widening:** Hit 3.35 S/A in V4 but required spending decisions.
Surge achieves 2.08 with zero decisions -- lower but fully passive.

## Draft Traces (Summary)

**Trace 1 (Committed Warriors):** First surge at pick 4 from 3-symbol Tide card.
Surge packs show 3-4 S/A (mix of Warriors S and Sacrifice A). 13 total surges
across 30 picks. Non-surge packs show 0-1 S/A.

**Trace 2 (Flexible, power-first):** Tokens spread across resonances through
pick 10. Surges scatter across Stone/Zephyr/Tide. After committing at pick 11,
Stone surges dominate. Late convergence but still functional.

**Trace 3 (Signal reader, Self-Discard):** Stone tokens accumulate fast from
2-3 symbol picks. Surges fire every other pick. 15 surges total. Surge packs
consistently show 3-4 S/A for Self-Discard.

## One-Sentence Claim Test

Implementation reconstructed from the one-sentence description. All operations
explicit. No hidden mechanics. Zero player decisions. PASS.

## Self-Assessment (1-10)

| Goal | Score | Notes |
|------|:---:|-------|
| Simple | 9 | One counter, one trigger |
| No actions | 10 | Fully passive |
| Not on rails | 8 | Non-permanent; tracks current tokens |
| No forced decks | 7 | Resonance dilution prevents forcing |
| Flexible | 7 | Pivoting shifts surge resonance |
| Convergent | 7 | 2.08 at T=4/S=3; misses at T=5+ |
| Splashable | 8 | 1 random slot always present |
| Open early | 9 | 75% of early packs fully random |
| Signal reading | 5 | Some benefit from timing awareness |

## Recommended Configuration

**Threshold=4, Surge Slots=3.** Only config crossing 2.0 S/A. 64% surge rate
post-commitment creates rhythm without being mechanical. Symbol distribution:
36 generic, 80 mono-1, 134 mono-2, 54 dual-2 (15% cap), 56 mono-3.
