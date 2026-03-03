# Results 3: Top-Pair Pool Seeding — V5 Round 3

## One-Sentence Algorithm

"After each pick, if you have drafted 2+ cards with the same ordered resonance
pair (first two symbols), 4 cards matching your most-drafted pair are added to
the pool from a reserve."

## Target Scorecard (Archetype Level, Committed Strategy, 1000 runs)

| Metric | Target | Actual | Pass/Fail |
|--------|--------|--------|-----------|
| Picks 1-5: unique archetypes w/ S/A per pack | >= 3 | 5.13 | PASS |
| Picks 1-5: S/A for emerging archetype per pack | <= 2 | 1.07 | PASS |
| Picks 6+: S/A for committed archetype per pack | >= 2.0 | 1.18 | **FAIL** |
| Picks 6+: off-archetype (C/F) per pack | >= 0.5 | 1.24 | PASS |
| Convergence pick | 5-8 | 22.5 | **FAIL** |
| Deck archetype concentration | 60-90% | 71.0% | PASS |
| Run-to-run card overlap | < 40% | 5.7% | PASS |
| Archetype frequency | No > 20%, no < 5% | 8.4%-19.0% | PASS |

Pool seeding **fails the two hardest targets** (late S/A and convergence) but
passes everything else comfortably, including metrics most algorithms struggle
with: low overlap, balanced archetype frequency, strong early diversity, healthy
splash.

## Variance Report

StdDev of S/A per pack (picks 6+): **0.90** (target >= 0.8, **PASS**).
Distribution: 0 S/A: 24.8%, 1: 41.2%, 2: 26.0%, 3: 7.3%, 4: 0.7%. Genuinely
natural variance with no mechanical consistency. Contrast Lane Locking's 0.32
stddev.

## Per-Archetype Convergence Table

| Archetype | Pool Seeding | Lane Locking | Pack Widening |
|-----------|:---:|:---:|:---:|
| Flash/Tempo | 23.0 | 4.9 | 26.8 |
| Blink/Flicker | 23.6 | 4.8 | 27.4 |
| Storm/Spellslinger | 24.4 | 4.9 | 28.2 |
| Self-Discard | 24.2 | 4.8 | 27.6 |
| Self-Mill/Reanimator | 22.6 | 4.8 | 27.5 |
| Sacrifice/Abandon | 22.5 | 4.9 | 26.8 |
| Warriors/Midrange | 23.5 | 4.8 | 26.9 |
| Ramp/Spirit Animals | 24.5 | 5.3 | 27.4 |

Pool seeding converges uniformly (~23, spread 22.5-24.5) showing perfect
archetype balance, but far too late. Lane Locking converges ~pick 5. Pack
Widening auto-spend never meaningfully converges (0.80 S/A at archetype level).

## V3/V4 Comparison

| Metric | Pool Seeding | Lane Locking | Pack Widening |
|--------|:---:|:---:|:---:|
| Late S/A (picks 6+) | 1.18 | 1.74 | 0.80 |
| Convergence pick | 22.5 | 6.4 | 26.4 |
| S/A StdDev | 0.90 | 0.32 | 0.80 |
| Off-archetype (C/F) | 1.24 | 0.55 | 1.40 |
| Deck concentration | 71.0% | 85.9% | 79.7% |
| Run-to-run overlap | 5.7% | 10.8% | 10.1% |

Lane Locking's 1.74 (not V3's 2.72) confirms archetype-level measurement
halves resonance-level numbers. Pack Widening auto-spend at single-resonance
performs *worse* than pool seeding. Pool seeding has best variance and lowest
overlap of all three.

## Symbol Distribution and Sensitivity

**Baseline**: 15/65/20 (1/2/3-symbol). Pair precision: 88.2%. The 12% miss
comes from 3-symbol cards with (primary, primary) non-archetype pairs.

**30% 1-symbol**: Late S/A drops negligibly (1.18 to 1.17). Pool seeding is
robust because injected reserve cards are always 2-symbol.

## Parameter Sensitivity

| Configuration | Late S/A | Conv. | StdDev | Pool@30 |
|---------------|:---:|:---:|:---:|:---:|
| Rate=3 | 1.11 | 23.5 | 0.89 | 402 |
| **Rate=4 (baseline)** | **1.19** | **22.6** | **0.91** | **427** |
| Rate=5 | 1.27 | 21.2 | 0.93 | 453 |
| Rate=4, remove=1 | 1.21 | 23.0 | 0.91 | 402 |
| Rate=4, remove=2 | 1.23 | 21.9 | 0.92 | 376 |
| Escalating min(count,5) | 1.18 | 23.8 | 0.91 | 435 |

Higher injection rates improve S/A modestly (+0.08/increment) but bloat the
pool (+25/increment). Removal controls pool size without convergence gain.
Escalating is slightly worse because early injection counts are low. **The pool
bloat ceiling is structural**: adding cards increases both numerator and
denominator, capping density shift at ~25-30%.

## Draft Traces

**Trace 1 (Early Committer, Sacrifice):** Pair (Tide,Stone) activates at pick
8. Pool grows 356 to 418. S/A fluctuates wildly (0,3,0,1,1,0,2,2,...) --
genuine natural variance. Occasionally hits 3-4 S/A packs but never
consistently.

**Trace 2 (Power Chaser, Ramp):** Scattered pair counts; top pair (Ember,Stone)
builds slowly and injects wrong-archetype cards. Ends at 28.1% deck
concentration. Pool seeding cannot help unfocused drafters.

**Trace 3 (Signal Reader, Self-Mill):** Commits pick 3 after seeing
(Stone,Tide) twice. Pair count reaches 17 by pick 30. Pool grows to 442. S/A
ranges 0-3 per pack. Signal reader benefits most from focused injection.

## Self-Assessment

| Goal | Score | Justification |
|------|:---:|---------------|
| 1. Simple | 8 | One concrete operation, implementable from description |
| 2. No extra actions | 10 | Fully automatic |
| 3. Not on rails | 9 | No locks or forced choices; invisible pool shift |
| 4. No forced decks | 9 | Random sampling from growing pool ensures variety |
| 5. Flexible archetypes | 7 | Pair-based seeding weakly supports hybrids |
| 6. Convergent | 2 | 1.18 S/A, far below 2.0. Structural ceiling confirmed |
| 7. Splashable | 8 | 1.24 C/F per pack; majority of pack remains random |
| 8. Open early | 9 | 5.13 unique archetypes per early pack |
| 9. Signal reading | 8 | Pool composition reflects availability; signal reader benefits most |

**Overall:** Pool seeding is the most natural-feeling algorithm but cannot cross
2.0 S/A alone. Its value is as a complementary layer enriching the pool for
another mechanism (D2/D4) to draw from. The pool bloat ceiling (~1.2 S/A) is
a mathematical limit of "add cards, draw randomly."
