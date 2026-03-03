# Results 5: Deck Echo Filter

## One-Sentence Algorithm

To make each pack, draw 12 random cards, then keep each independently with
probability (2 + its weighted symbol overlap with your drafted deck) / 6, and
fill any remaining pack slots randomly from the rejects.

Primary resonance matches in drafted deck weighted 1.5x, secondary/tertiary
1.0x. Uses only visible card properties.

## Target Scorecard (Archetype-Level, 1000 Drafts)

| Metric | Target | Deck Echo | Pass/Fail |
|--------|--------|-----------|-----------|
| Picks 1-5: unique archetypes w/ S/A per pack | >= 3 | 6.50 | PASS |
| Picks 1-5: S/A for emerging archetype | <= 2 | 1.54 | PASS |
| Picks 6+: S/A for committed archetype | >= 2 | 1.55 | **FAIL** |
| Picks 6+: off-archetype (C/F) per pack | >= 0.5 | 1.27 | PASS |
| Convergence pick | Pick 5-8 | 3.5-6.1 | PARTIAL |
| Deck concentration | 60-90% | 87.2% | PASS |
| Run-to-run card overlap | < 40% | 11.0% | PASS |
| Archetype frequency | No >20%/<5% | 11.7%-13.3% | PASS |
| StdDev S/A (picks 6+) | >= 0.8 | 0.973 | PASS |

**7/9 pass, 1 fail, 1 partial.** Critical failure: 1.55 S/A vs 2.0 target.

## Variance Report

| S/A Cards | Pct |  | S/A Cards | Pct |
|-----------|-----|--|-----------|-----|
| 0 | 14.0% |  | 3 | 14.4% |
| 1 | 35.6% |  | 4 | 2.2% |
| 2 | 33.8% |  |   |       |

StdDev = 0.973 (PASS). Genuinely spread: 50% of packs have 2+ S/A, 14% have
0. No mechanical floor or ceiling.

## Per-Archetype Convergence Table

| Archetype | Convergence Pick |
|-----------|-----------------|
| Flash/Tempo/Prison | 4.7 |
| Blink/Flicker | 3.8 |
| Storm/Spellslinger | 4.3 |
| Self-Discard | 4.2 |
| Self-Mill/Reanimator | 4.8 |
| Sacrifice/Abandon | 4.7 |
| Warriors/Midrange | 4.2 |
| Ramp/Spirit Animals | 3.5 |

Caveat: convergence uses a rolling 3-pick window. The filter produces
intermittent spikes satisfying this window without sustaining 2.0+ long-run
(which remains 1.55).

## V3 Lane Locking Comparison

| Metric | Deck Echo | Lane Lock | Target |
|--------|-----------|-----------|--------|
| Picks 6+: S/A for target | **1.55** | **2.37** | >= 2 |
| Off-archetype (C/F) | 1.27 | 0.61 | >= 0.5 |
| Deck concentration | **87.2%** | **97.9%** | 60-90% |
| S/A stddev | 0.973 | 0.995 | >= 0.8 |
| Card overlap | 11.0% | 19.1% | < 40% |

Lane Locking wins convergence (2.37 vs 1.55). Deck Echo wins concentration
(87.2% vs 97.9% -- Lane Locking FAILS 60-90%), splash, and variety. Neither
passes all targets.

## Symbol Distribution and Sensitivity

**Distribution:** 30/50/20 (1/2/3-sym) plus 36 generic.
All tested distributions (60/30/10 through 10/30/60) produce late S/A between
1.52-1.56. Base acceptance floor (2/6) dominates over symbol signal.

**Candidate pool (8-16):** Minimal impact; late S/A stays 1.53-1.55.

**Acceptance formula:** (1+echo)/4 best at 1.58; (3+echo)/7 weakest at 1.49.
Even most aggressive cannot reach 2.0. Bottleneck: 12 candidates contain ~1.3
expected S/A cards for any archetype.

**Primary weight (1.0-2.0):** Marginal improvement from 1.51 to 1.56.

**Progressive denominator (6/5/4):** Counterintuitively reduces S/A to 1.49 --
tighter filter pushes more slots to random reject-fills.

## Draft Traces

**Early Committer (Blink/Flicker):** Near-random packs picks 1-3. Pick 6:
4-S/A jackpot. Picks 7-15 fluctuate 1-4 S/A. Momentum builds but never
guarantees -- pick 8 delivers only 1 S/A.

**Signal Reader (Self-Mill):** Chases power picks 1-5. At pick 6 infers
Self-Mill, immediately finds S-tier card. Picks 7-15 average 1.9 S/A --
benefits from flexible early history feeding echo score.

**Power Chaser (Blink/Flicker):** Picks highest power. Incidentally accumulates
Ember symbols. By pick 7, pack contains 2 Blink S-tier cards despite no
deliberate commitment.

## Self-Assessment

| Goal | Score | Justification |
|------|-------|---------------|
| 1. Simple | 7 | Clear three-phase process; probability formula adds friction. |
| 2. Not on rails | 9 | Never locks packs; 14% of late packs have 0 S/A. |
| 3. No forced decks | 9 | 11% overlap across same-archetype runs. |
| 4. Flexible archetypes | 8 | Overlap scoring supports bridge strategies. |
| 5. Convergent | 3 | **Critical failure:** 1.55 vs 2.0 target. |
| 6. Splashable | 9 | 1.27 off-archetype per pack, well above 0.5. |
| 7. Open early | 9 | 6.5 unique archetypes early; near-random before commitment. |
| 8. Signal reading | 3 | No pool-state info; filter uses player history only. |

**Honest assessment:** Deck Echo produces natural variance and avoids V3's
mechanical problems. It fails the key convergence metric. Per-card filtering
cannot reliably surface archetype-specific cards when each archetype is ~11% of
the pool. Lane Locking's deterministic placement brute-forces past this
bottleneck; Deck Echo's stochastic approach cannot.
