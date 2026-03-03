# Results: Agent 3 — Pack Widening v2

## One-Sentence Algorithm

"Each symbol you draft earns 1 matching token (primary earns 2); before seeing a pack, you may spend 2 tokens of one resonance to add 2 extra cards with that primary resonance to the pack."

## Target Scorecard (Committed Player, Archetype-Level)

| Metric | Target | Pack Widening v2 | Lane Locking | PW | LL |
|--------|--------|:-:|:-:|:-:|:-:|
| Picks 1-5: unique archs w/ S/A | >= 3 | 6.85 | 6.51 | PASS | PASS |
| Picks 1-5: S/A for emerging arch | <= 2 | 2.48 | 1.81 | FAIL | PASS |
| Picks 6+: S/A for committed arch | >= 2 | 3.35 | 2.78 | PASS | PASS |
| Picks 6+: off-archetype (C/F) | >= 0.5 | 1.35 | 0.23 | PASS | FAIL |
| Convergence pick | 5-8 | 6.0 | 6.0 | PASS | PASS |
| Deck concentration | 60-90% | 98.6% | 98.6% | FAIL | FAIL |
| Run-to-run overlap | < 40% | 39.7% | 34.1% | PASS* | PASS |
| Archetype frequency | 5-20% each | 12.5% | 12.5% | PASS | PASS |
| StdDev S/A per pack (6+) | >= 0.8 | 0.94 | 0.70 | PASS | FAIL |

**Pack Widening: 7/9. Lane Locking: 6/9.** Both fail deck concentration. Pack Widening fails early S/A (early spending narrows too fast). Lane Locking fails variance and splash.

## Variance Report

| S/A count | Pack Widening v2 | Lane Locking |
|:-:|:-:|:-:|
| 0 | 0.0% | 0.0% |
| 1 | 0.0% | 0.7% |
| 2 | 18.9% | 35.4% |
| 3 | 39.5% | 48.7% |
| 4 | 30.2% | 15.2% |

StdDev 0.94 vs 0.70. Pack Widening spreads more naturally across 2-4 S/A cards.

## Per-Archetype Convergence Table

| Archetype | Pack Widening v2 | Lane Locking |
|-----------|:-:|:-:|
| Flash/Tempo/Prison | 6.0 | 6.1 |
| Blink/Flicker | 6.0 | 6.0 |
| Storm/Spellslinger | 6.0 | 6.0 |
| Self-Discard | 6.0 | 6.0 |
| Self-Mill/Reanimator | 6.0 | 6.0 |
| Sacrifice/Abandon | 6.0 | 6.0 |
| Warriors/Midrange | 6.0 | 6.0 |
| Ramp/Spirit Animals | 6.0 | 6.0 |

Uniform convergence at pick 6 for both algorithms, driven by symmetric pool and high base S/A rate.

## Symbol Distribution and Sensitivity

Used: 20/55/25 (1/2/3 symbols). Sweep showed **no meaningful difference** across distributions (all avg S/A 3.32-3.33). Bonus cards drawn from primary-resonance pool regardless of symbol count.

## Parameter Sensitivity

**Spend cost** (bonus=2): Cost 1-2 saturate (always spend, S/A 3.35). Cost 3 creates real decisions (S/A 2.70, stddev 1.34). Cost 4 drops convergence (S/A 2.30).

**Bonus count** (cost=2): Each bonus adds ~1.0 S/A. Bonus=1 (2.34) minimum viable. Bonus=3 (4.36) over-converges.

**Primary weight**: Weight 1 creates spending variance (S/A 2.49). Weight 2-3 saturate.

**Key finding:** At cost 2/bonus 2, tokens always suffice, making the economic decision trivial. Cost 3 restores decisions at convergence cost.

## Draft Traces

**Trace 1 — Early Committer (Warriors).** Spends Tide tokens from pick 3 onward. Every pack is 6 cards. Sees 2-5 S/A consistently. Tokens accumulate faster than spent (9 Tide, 24 Zephyr by pick 30). No real save/spend tension.

**Trace 2 — Power Chaser.** Spends on strongest resonance (Tide) but picks for raw power. Cards scatter across Warriors, Sacrifice, Self-Discard, Blink. Deck concentration ~36%. Bonus cards wasted because player ignores archetype focus.

**Trace 3 — Signal Reader.** Explores picks 1-5 across Ember/Tide. Commits to Warriors by pick 6. Begins Tide spending, sees 2-5 S/A per pack. Converges efficiently despite exploration phase.

## Self-Assessment

| Goal | Score | Justification |
|------|:-:|---|
| 1. Simple | 9 | One sentence, fully implementable, no hidden state. |
| 2. Not on rails | 7 | Spending optional, but at cost 2 always-spend is dominant. |
| 3. No forced decks | 5 | High base S/A rate + frequent bonuses create repetitive pools. |
| 4. Flexible archetypes | 7 | Multi-resonance tokens support pivots and hybrids. |
| 5. Convergent | 8 | 3.35 S/A exceeds 2.0 target comfortably. |
| 6. Splashable | 8 | 1.35 off-archetype cards per pack vs Lane Locking's 0.23. |
| 7. Open-ended early | 5 | Early spending narrows too fast; 2.48 > 2.0 target. |
| 8. Signal reading | 3 | No pool depletion; signals only from random card appearances. |

**Honest assessment:** Beats Lane Locking on variance and splash, loses on early openness. Spend at cost 2 is trivial (always spend), undermining the player-agency premise. Cost 3 restores decisions but drops convergence. Biggest weakness: no signal-reading support.
