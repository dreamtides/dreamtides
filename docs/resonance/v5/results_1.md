# Results — Agent 1: Pair-Based Threshold Auto-Spend with Scaling Bonus

## One-Sentence Algorithm

"Each card you draft with 2+ symbols adds 1 to its ordered pair count; when any
pair reaches 3, your next pack gets a bonus card sharing that pair and the count
resets to 0; if the pair has already triggered before, add 2 bonus cards instead."

## Target Scorecard (Archetype Level)

| Metric | Target | Actual | Pass/Fail |
|--------|--------|--------|-----------|
| Picks 1-5: unique archetypes w/ S/A per pack | >= 3 | 5.15 | PASS |
| Picks 1-5: S/A for target per pack | <= 2 | 0.92 | PASS |
| Picks 6+: S/A for archetype per pack | >= 2.0 | 1.10 | **FAIL** |
| Picks 6+: off-archetype (C/F) per pack | >= 0.5 | 1.37 | PASS |
| Convergence pick | 5-8 | 16.4 | **FAIL** |
| Deck concentration | 60-90% | 51.0% | **FAIL** |
| Run-to-run variety (overlap) | < 40% | 5.1% | PASS |
| Variance (stddev S/A picks 6+) | >= 0.8 | 0.99 | PASS |

**Passes 5/8. Fails convergence, deck concentration, and S/A threshold.**

## Variance Report

StdDev: **0.99** (PASS). S/A distribution (committed, picks 6+): 0 S/A = 29%,
1 = 38%, 2 = 22%, 3 = 9%, 4+ = 3%. Mode is 1, not 2. Genuine variance but
average too low.

## Per-Archetype Convergence Table

| Archetype | Avg. Convergence Pick |
|-----------|-----------------------|
| Flash/Tempo/Prison | 14.7 |
| Blink/Flicker | 14.6 |
| Storm/Spellslinger | 13.5 |
| Self-Discard | 13.7 |
| Self-Mill/Reanimator | 13.8 |
| Sacrifice/Abandon | 14.8 |
| Warriors/Midrange | 14.6 |
| Ramp/Spirit Animals | 14.8 |

Uniformly balanced (13.5-14.8 range) but uniformly too slow (target: 5-8).

## V3/V4 Comparison

| Metric | Pair Thresh | Lane Lock | Pack Widen |
|--------|-------------|-----------|------------|
| Late S/A per pack | **1.10** | **1.78** | **1.38** |
| Late C/F per pack | 1.37 | 0.99 | 1.51 |
| Convergence pick | 16.4 | 12.2 | 9.7 |
| Deck concentration | 51.0% | 65.4% | 57.7% |
| Variance (stddev) | 0.99 | 1.10 | 0.98 |

Lane Locking dominates (1.78 S/A; committed converge at 5.0-6.1). Pack Widening
achieves 1.38 with ~50% archetype precision. My algorithm trails both. None
cross 2.0 at archetype level with mixed strategies.

## Symbol Distribution & Sensitivity

Default: 15/65/20 (1-sym/2-sym/3-sym). Pair precision: 100% S-tier for all 8
archetypes.

| Distribution | Late S/A | Conv Pick | StdDev |
|-------------|----------|-----------|--------|
| 15% 1-sym (default) | 1.10 | 16.4 | 0.99 |
| 30% 1-sym (stress) | 1.02 | 18.5 | 0.94 |
| 5% 1-sym (pair-heavy) | 1.11 | 16.4 | 1.01 |

Bottleneck is the injection mechanism, not pair generation rate.

## Parameter Sensitivity

| Config | Late S/A | Conv Pick | StdDev |
|--------|----------|-----------|--------|
| T=2, B=1, No Scaling | 1.13 | 14.7 | 0.94 |
| T=2, B=1, Scaling | 1.30 | 12.8 | 1.13 |
| T=2, B=2, No Scaling | 1.36 | 10.3 | 1.18 |
| T=3, B=1, No Scaling | 1.03 | 17.7 | 0.91 |
| T=3, B=1, Scaling (main) | 1.10 | 16.6 | 1.00 |
| T=3, B=2, No Scaling | 1.14 | 15.1 | 1.05 |

Best: T=2, B=2 (1.36 S/A, conv 10.3). Even the most aggressive configuration
stays well below 2.0. Scaling bonus adds +0.17 S/A over no scaling at T=2.

## Draft Traces

**Trace 1 (Early Committer, Warriors):** Built (Tide,Zephyr)=2 by pick 3.
Threshold fired pick 11 (Tide,Stone triggered first via Sacrifice drafts).
Bonuses too infrequent to sustain 2+ S/A per pack.

**Trace 2 (Signal Reader, Storm):** Scattered across pairs through pick 8. First
trigger at pick 13 (wrong archetype). Signal reading provides zero benefit.

**Trace 3 (Power Chaser, Blink):** Pairs scattered across 5+ pairs. First bonus
pick 12. Power chasing yields no coherent pair profile.

## Self-Assessment

| Goal | Score | Justification |
|------|-------|---------------|
| 1. Simple | 8 | One-sentence fully specifies algorithm |
| 2. No extra actions | 10 | Fully automatic |
| 3. Not on rails | 8 | No permanent commitment |
| 4. No forced decks | 7 | Pool randomness dominates |
| 5. Flexible archetypes | 7 | All archetypes treated equally |
| 6. Convergent | 2 | 1.10 S/A; structural ceiling confirmed |
| 7. Splashable | 8 | 1.37 C/F per pack |
| 8. Open early | 9 | 5.15 unique archetypes early |
| 9. Signal reading | 2 | Self-referential; pool irrelevant |

**Verdict:** Bonus injection is structurally capped at ~1.3-1.4 S/A. Cannot
reach 2.0 regardless of tuning. Excels at simplicity and variance but cannot
deliver convergence standalone. Best role: complement to slot-replacement (D2/D4).
