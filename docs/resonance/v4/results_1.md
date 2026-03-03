# Results: Agent 1 — Exile Pressure

## One-Sentence Algorithm

"When you pass a card, add 2 to its primary resonance's exile counter and 1 per secondary/tertiary symbol; all counters decay by 1 each pick; each pack card is independently skipped with probability (its primary resonance's counter / 20), rerolling on a skip."

## Target Scorecard (Archetype Level, 1000 Drafts)

| Metric | Target | Actual | Pass/Fail |
|--------|--------|--------|-----------|
| Picks 1-5: unique archetypes w/ S/A per pack | >= 3 | 6.42 | PASS |
| Picks 1-5: S/A for emerging archetype per pack | <= 2 | 1.41 | PASS |
| Picks 6+: S/A for committed archetype per pack | >= 2 | 1.57 | **FAIL** |
| Picks 6+: off-archetype cards per pack | >= 0.5 | 2.43 | PASS |
| Convergence pick | 5-8 | 7.4 | PASS |
| Deck concentration | 60-90% | 85% | PASS |
| S/A stddev (picks 6+) | >= 0.8 | 1.00 | PASS |
| Run-to-run card overlap | < 40% | 8% | PASS |
| Archetype frequency balance | None >20%/<5% | 10.6-14.0% | PASS |

**7/9 passed.** Critical failure: late S/A at 1.57 vs. 2.0 target.

## Variance Report

StdDev = 1.00 (PASS).

| S/A Count | Frequency |
|-----------|-----------|
| 0 | 14.5% |
| 1 | 34.6% |
| 2 | 33.3% |
| 3 | 14.8% |
| 4 | 2.8% |

Genuine variance; lower mean is the tradeoff.

## Per-Archetype Convergence Table

| Archetype | Avg. Convergence Pick |
|-----------|-----------------------|
| Flash/Tempo/Prison | 7.6 |
| Blink/Flicker | 7.4 |
| Storm/Spellslinger | 7.4 |
| Self-Discard | 7.7 |
| Self-Mill/Reanimator | 7.2 |
| Sacrifice/Abandon | 7.5 |
| Warriors/Midrange | 7.5 |
| Ramp/Spirit Animals | 6.8 |

Tight band (6.8-7.7): excellent balance.

## V3 Lane Locking Comparison

| Metric | Target | Exile Pressure | Lane Locking | Winner |
|--------|--------|---------------|-------------|--------|
| Early unique archs | >= 3 | 6.42 | 6.53 | Tie |
| Early S/A emerging | <= 2 | 1.41 | 1.65 | Exile |
| Late S/A committed | >= 2 | **1.57** | **2.08** | Lane |
| Late off-archetype | >= 0.5 | 2.43 | 1.92 | Tie |
| Convergence pick | 5-8 | 7.4 | 5.7 | Lane |
| Deck concentration | 60-90% | 85% | **98%** | Exile |
| S/A stddev | >= 0.8 | 1.00 | 0.84 | Exile |
| Run overlap | < 0.40 | 0.08 | 0.08 | Tie |

Lane Locking wins convergence/speed. Exile Pressure wins variance/deck diversity (Lane Locking over-converges past 90%).

## Symbol Distribution & Sensitivity

**Used:** 30/50/20 (1-sym/2-sym/3-sym) + 36 generic.

| Distribution | Late S/A | StdDev | Conv. Pick |
|-------------|----------|--------|------------|
| Mostly 1-sym (60/30/10) | 1.63 | 0.99 | 7.8 |
| Balanced (30/50/20) | 1.56 | 1.00 | 7.4 |
| Mostly 2-sym (15/65/20) | 1.53 | 1.01 | 7.3 |
| Mostly 3-sym (15/30/55) | 1.43 | 1.02 | 6.9 |

## Parameter Sensitivity

| Parameter | Value | Late S/A | Conv. Pick |
|-----------|-------|----------|------------|
| Divisor | 15 | 1.48 | 6.9 |
| Divisor | 20 (default) | 1.57 | 7.0 |
| Divisor | 30 | 1.54 | 7.8 |
| Decay | 0.5 | 1.29 | 7.1 |
| Decay | 1.0 (default) | 1.57 | 7.2 |
| Decay | 2.0 | 1.52 | 8.4 |
| Cap | 10 | 1.42 | 8.3 |
| Cap | 20 (default) | 1.57 | 7.4 |
| Cap | 30 | 1.58 | 7.2 |

No combination reaches 2.0 S/A — bottleneck is structural.

## Draft Traces

**Trace 1 — Early Committer (Warriors):** Ember/Stone counters reach 11/4 by pick 7, producing 2 S/A packs. Pick 12: 0 S/A. Variance is real.

**Trace 2 — Flexible Player (power-chases 10 picks, then Flash):** Diffuse exile across all resonances by pick 10. Late commitment cripples convergence.

**Trace 3 — Signal Reader (Warriors):** Asymmetric exile against Stone/Ember. By pick 11, sees 3 S/A. Rewards early reads.

## Self-Assessment

| Goal | Score | Justification |
|------|-------|---------------|
| 1. Simple | 5/10 | Complete but long one-sentence |
| 2. Not on rails | 8/10 | Decay enables pivoting |
| 3. No forced decks | 8/10 | 8% run overlap |
| 4. Flexible archetypes | 7/10 | Supports bridge decks |
| 5. Convergent | 4/10 | 1.57 S/A, short of 2.0 |
| 6. Splashable | 8/10 | 2.43 off-archetype cards |
| 7. Open early | 9/10 | Near-zero exile in first 5 picks |
| 8. Signal reading | 6/10 | Rewards early reads |

**Bottom line:** Exile pressure on resonance, not archetype, means ~50% dilution caps archetype convergence at 1.5-1.6 regardless of tuning.
