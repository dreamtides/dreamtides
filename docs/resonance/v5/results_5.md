# Results 5: Hybrid Resonance-Triggered Pair Bonus

## One-Sentence Algorithm

"Draw 4 random cards; if any card's primary resonance matches your most-drafted resonance, add 1 bonus card whose ordered pair matches your most-drafted pair."

Fully automatic. No player decisions beyond picking 1 card.

## Target Scorecard (Archetype Level)

| Metric | Target | Actual | Pass/Fail |
|--------|--------|--------|-----------|
| Picks 1-5: unique archs w/ S/A per pack | >= 3 | 5.2 | PASS |
| Picks 1-5: S/A for emerging arch per pack | <= 2 | ~0.9 | PASS |
| Picks 6+: S/A for committed arch per pack | >= 2.0 | **1.52** | **FAIL** |
| Picks 6+: off-archetype (C/F) per pack | >= 0.5 | 1.36 | PASS |
| Convergence pick | 5-8 | **11.8** | **FAIL** |
| Deck archetype concentration | 60-90% | 62% | PASS |
| Run-to-run overlap | < 40% | 5.8% | PASS |
| Archetype freq max | <= 20% | 19.8% | PASS |
| Archetype freq min | >= 5% | 6.6% | PASS |
| StdDev of S/A per pack (6+) | >= 0.8 | 1.23 | PASS |

**Standalone fails convergence (1.52 vs 2.0).** Fire rate: 64%. Pair precision: 100% S-tier. Contribution: 0.90 baseline + 0.64 bonus = 1.54 projected. Confirms the structural ceiling for bonus injection.

## Variance Report

StdDev = 1.23 (PASS). Packs are either 5 cards (bonus) or 4 cards (no trigger), creating organic variance. Lane Locking's 0.78 fails this target.

## Per-Archetype Convergence Table

| Archetype | Hybrid Trigger | V3 Lane Lock | V4 Pack Widen | D4 Hybrid |
|-----------|:-:|:-:|:-:|:-:|
| Flash/Tempo/Prison | 8.9 | 8.1 | 8.6 | 8.7 |
| Blink/Flicker | 9.0 | 8.2 | 7.7 | 7.7 |
| Storm/Spellslinger | 8.3 | 7.7 | 7.4 | 8.6 |
| Self-Discard | 7.3 | 7.8 | 8.3 | 8.6 |
| Self-Mill/Reanimator | 10.0 | 6.6 | 8.7 | 8.0 |
| Sacrifice/Abandon | 8.7 | 8.4 | 9.0 | 7.2 |
| Warriors/Midrange | 9.1 | 7.7 | 9.4 | 9.0 |
| Ramp/Spirit Animals | 7.5 | 8.0 | 8.4 | 8.5 |
| **Mean** | **8.6** | **7.8** | **8.4** | **8.3** |

Range 7.3-10.0. Self-Mill slowest (Stone-primary dilution).

## V3/V4 Comparison

| Metric | Hybrid +1 | Hybrid +2 | D4 Hybrid | Lane Lock | Pack Widen |
|--------|:-:|:-:|:-:|:-:|:-:|
| Late S/A | 1.52 | **2.13** | **2.10** | **2.29** | 1.53 |
| StdDev | **1.23** | **1.69** | **1.21** | 0.78 | **0.96** |
| Convergence | 11.8 | 9.2 | 10.2 | 9.7 | 12.8 |
| Deck conc. | 62% | 62% | 88% | 88% | 80% |

D4 Hybrid is most balanced: 2.10 S/A, 1.21 stddev, 88% deck concentration. Pack Widening underperforms (1.53) due to single-resonance 50% dilution.

## Symbol Distribution

15/60/25 split. At 30% 1-sym: S/A drops 1.52 to 1.50. Minimal impact since trigger is resonance-based; only bonus selection needs pairs.

## Parameter Sensitivity

| Variant | Late S/A | StdDev | Fire Rate |
|---------|:-:|:-:|:-:|
| Resonance trigger +1 | 1.52 | 1.23 | 64% |
| Resonance trigger +2 | 2.13 | 1.69 | 63% |
| Pair trigger +1 | 1.19 | 1.14 | 30% |
| Pair trigger +2 | 1.49 | 1.52 | 31% |
| D4 Hybrid (thresh 3) | 2.10 | 1.21 | 54% |
| D4 Hybrid (thresh 2) | 2.14 | 1.20 | 54% |

Resonance triggers fire 2x vs pair triggers (64% vs 30%). +2 bonus crosses 2.0 but stddev 1.69 is excessive. D4 Hybrid achieves 2.10 with controlled variance via guaranteed slot.

## Draft Traces

**Trace 1 -- Early Committer (Warriors, pick 3).** Top pair initially Zephyr/Ember (wrong archetype). Self-corrects by pick 7 as Tide overtakes. From pick 11+, bonuses consistently deliver Warriors cards. Final: 19/30 S/A (63%).

**Trace 2 -- Flexible Player (uncommitted until pick 10).** Stone/Tide pair dominant from early picks. Commits to Self-Mill at pick 11; bonuses immediately precise. Final: 15/30 S/A (50%) -- lower because 10 uncommitted picks diluted the deck.

**Trace 3 -- Signal Reader (commit pick 8).** Ember/Stone pair aligned from pick 1. From pick 9+, 73% of packs trigger with Storm/Spellslinger bonus. Final: 25/30 S/A (83%).

## Self-Assessment

| Goal | Score | Justification |
|------|:-----:|---------------|
| 1. Simple | 8 | One sentence, two concrete concepts |
| 2. No extra actions | 10 | Fully automatic |
| 3. Not on rails | 9 | 36% of packs unenhanced; no permanent lock |
| 4. No forced decks | 8 | Pool + trigger randomness prevent forcing |
| 5. Flexible archetypes | 8 | Pair tracking follows player's choices, pivots naturally |
| 6. Convergent | 3 | 1.52 S/A fails 2.0 standalone; 2.1+ only with D4 hybrid |
| 7. Splashable | 8 | 1.36 off-archetype cards per pack |
| 8. Open early | 9 | No mechanism pre-commitment |
| 9. Signal reading | 3 | Pool-independent; no signal to read |

**Honest summary:** The standalone algorithm cannot cross 2.0 S/A. Its value is organic variance and natural feel. The D4 Hybrid (1 guaranteed pair slot + conditional bonus) at 2.10 S/A / 1.21 stddev is the most promising path from this domain.
