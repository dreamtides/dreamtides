# Results 4: Multiple Phantoms, Ecosystem Competition

## One-Sentence Algorithm

"Two phantom drafters, each assigned a random resonance (sometimes the same one), each remove the best-matching card from the pool each round; you draft from what remains."

Uses only visible card properties (symbols). Packs are 4 random cards from pool.

## Target Scorecard (Committed Player, Archetype Level)

| Metric | Target | Phantom | Lane Locking | Pass? |
|--------|--------|---------|-------------|-------|
| Picks 1-5: unique archs w/ S/A | >= 3 | 6.49 | 6.41 | PASS |
| Picks 1-5: S/A for emerging arch | <= 2 | 1.34 | 1.41 | PASS |
| Picks 6+: S/A for committed arch | >= 2.0 | 1.26 | 1.81 | **FAIL** |
| Picks 6+: off-archetype (C/F) | >= 0.5 | 1.37 | 1.02 | PASS |
| Convergence pick | 5-8 | 11.1 | 9.1 | **FAIL** |
| Deck concentration | 60-90% | 72.0% | 82.3% | PASS |
| Run-to-run variety | < 40% | 4.9% | 6.1% | PASS |
| Archetype frequency | No >20%/<5% | Balanced | Balanced | PASS |
| StdDev S/A (picks 6+) | >= 0.8 | 0.92 | 0.93 | PASS |

**Passes 7/9. Fails convergence (1.26 vs 2.0) and convergence pick (11.1 vs 5-8).**

## Variance Report

| S/A count | Phantom | Lane Locking |
|-----------|---------|-------------|
| 0 cards | 22.4% | 10.2% |
| 1 card | 40.3% | 27.5% |
| 2 cards | 27.7% | 37.5% |
| 3 cards | 8.7% | 20.9% |
| 4 cards | 0.9% | 3.9% |

StdDev: Phantom=0.92, Lane Locking=0.93. Both pass variance target. Phantom's modal pack has 1 S/A card; Lane Locking's has 2. Phantom produces 0-S/A packs 22% of the time.

## Per-Archetype Convergence Table

| Archetype | Phantom | Lane Locking |
|-----------|---------|-------------|
| Flash/Tempo/Prison | 11.2 | 9.6 |
| Blink/Flicker | 10.2 | 8.9 |
| Storm/Spellslinger | 10.3 | 8.1 |
| Self-Discard | 10.7 | 8.5 |
| Self-Mill/Reanimator | 11.9 | 8.8 |
| Sacrifice/Abandon | 11.5 | 9.7 |
| Warriors/Midrange | 12.7 | 10.8 |
| Ramp/Spirit Animals | 10.3 | 9.9 |

Phantom converges pick 10-13 (all outside 5-8 target). Lane Locking converges pick 8-11, consistently 1-2 picks faster. Warriors slowest for both.

## V3 Lane Locking Comparison

Lane Locking outperforms on convergence (1.81 vs 1.26 late S/A; pick 9.1 vs 11.1). Signal-reader gap is sharp: Lane Locking 2.10 S/A / 91% concentration vs Phantom 1.36 / 73%. Phantom wins on variety (4.9% vs 6.1% overlap), splash (1.37 vs 1.02), and signal reading as a first-class external signal.

## Symbol Distribution and Sensitivity

**Used:** 35/45/20% (1/2/3-symbol), 36 generic. Symbol distribution has essentially no effect -- all distributions produce ~1.26 late S/A. This confirms the fundamental limitation: phantoms remove too few cards (60 total = 17% of pool) to shift archetype-level density. The open resonance rises from ~25% to ~30%, but only ~50% of those cards are S/A for a specific archetype.

## Parameter Sensitivity

**Phantom count (1-4):** No meaningful effect. Late S/A: 1.22-1.27. Pool suppression is too weak at any count.

**Cards per phantom (1-3):** Aggressive removal (6/round) *hurts* convergence (1.18 S/A, pick 12.8) by depleting the pool faster overall.

**Resonance overlap:** Negligible (1.26 vs 1.24 distinct vs overlap).

## Draft Traces

**Early Committer (Warriors, pick 5):** Variable packs post-commitment: pick 6 has 0 S/A, pick 8 has 3 S/A. 80% concentration. Variance is real but average too low.

**Power Chaser:** 26.7% concentration. Algorithm does not reward commitment -- power-chasing works fine.

**Signal Reader (Self-Mill, pick 6):** Identifies Stone as open, achieves 1.64 late S/A and 70% concentration. Better than blind commitment (1.26) but still below 2.0 target.

## Self-Assessment

| Goal | Score | Justification |
|------|-------|--------------|
| 1. Simple | 9 | One sentence fully specifies the algorithm. |
| 2. Not on rails | 9 | No locks or forced slots; pivot freely anytime. |
| 3. No forced decks | 9 | Phantom resonances randomize each run. |
| 4. Flexible | 7 | All 8 viable but open-resonance favors 2-4 per run. |
| 5. Convergent | 3 | 1.26 vs 2.0 target; suppression fundamentally too weak. |
| 6. Splashable | 8 | 1.37 off-archetype cards per pack. |
| 7. Open early | 9 | 6.49 unique archetypes in early packs. |
| 8. Signal reading | 10 | First-class: phantom consumption is directly observable. |

**Verdict:** Excels at simplicity, openness, signal reading, and variance, but fails convergence. Pool suppression raises open-resonance density from ~25% to ~30%, and half of those cards belong to the wrong archetype. A hybrid (phantoms + convergence layer) would be needed to hit all targets.
