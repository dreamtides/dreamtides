# Agent 5 Results: Double Enhancement

**One-sentence:** "Draw 4 random cards; if 2 or more share a primary resonance with your top resonance, add 2 cards of that resonance to the pack."

**One-sentence test:** Implementable from description alone. Implicit state: "top resonance" (weighted count, primary=2, others=1). Zero decisions verified.

**Pool:** 360 cards, 36 generic, 54 dual-type (15.0%). Each resonance ~22.5%.

## Scorecard (Champion: thresh=2, bonus=2)

| Metric | Target | Committed | Power Chaser | Signal Reader |
|--------|--------|-----------|--------------|---------------|
| Picks 1-5: unique archs w/ S/A | >= 3 | 5.11 | 5.11 | 5.14 |
| Picks 1-5: S/A emerging arch | <= 2 | 1.06 | 1.10 | 1.08 |
| **Picks 6+: S/A per pack** | **>= 2.0** | **1.32** | **1.23** | **1.31** |
| Picks 6+: C/F per pack | >= 0.5 | 1.35 | 1.37 | 1.37 |
| Convergence pick | 5-8 | 8.9 | 8.8 | 8.6 |
| Deck concentration | 60-90% | 61.7% | 33.7% | 62.3% |
| Card overlap | < 40% | 15.9% | 17.0% | 15.3% |
| S/A stddev (picks 6+) | >= 0.8 | 1.55 | 1.44 | 1.55 |

**The champion fails the convergence target.** At 1.32 S/A, it falls well below 2.0. Root cause: each resonance is ~22% of pool, so P(2+ of 4 match) = Bin(4, 0.22) = ~21%. The trigger fires only 1 in 5 packs.

## Parameter Sensitivity (committed, 500 drafts)

| Config | Late S/A | StdDev | Conv Pick | Trig Late | Pack |
|--------|----------|--------|-----------|-----------|------|
| thresh=1, bonus=1, primary | 1.52 | 1.24 | 7.7 | 65% | 4.59 |
| **thresh=1, bonus=2, primary** | **2.13** | **1.69** | **5.1** | **65%** | **5.19** |
| thresh=1, bonus=2, dual-only | 2.03 | 1.67 | 7.0 | 64% | 5.07 |
| thresh=2, bonus=2, primary | 1.29 | 1.54 | 8.6 | 21% | 4.39 |
| thresh=2, bonus=3, primary | 1.52 | 1.94 | 7.5 | 22% | 4.61 |
| thresh=3, bonus=2, primary | 0.96 | 1.03 | 12.6 | 3% | 4.06 |

**Best variant: thresh=1, bonus=2** crosses 2.0 (2.13 S/A, conv 5.1). But P(1+ match)=63%, so most packs trigger -- the conditional aspect weakens.

## Trigger Frequency Over Draft

| Phase | thresh=2 | thresh=1 |
|-------|----------|----------|
| Picks 1-5 | 11% | ~8% |
| Picks 6-15 | 22% | ~64% |
| Picks 16-30 | 22% | ~65% |

No escalation effect for either variant -- rate is flat after activation.

## Per-Archetype Convergence (committed, thresh=2)

| Archetype | Avg Conv Pick | Conv Rate | Count |
|-----------|---------------|-----------|-------|
| Flash | 9.4 | 99% | 159 |
| Blink | 9.7 | 97% | 143 |
| Storm | 9.0 | 97% | 94 |
| Self-Discard | 7.9 | 97% | 146 |
| Self-Mill | 9.3 | 100% | 107 |
| Sacrifice | 8.7 | 98% | 151 |
| Warriors | 7.8 | 96% | 82 |
| Ramp | 8.9 | 97% | 118 |

Convergence range 7.8-9.7 across all archetypes. Frequency 8.2%-15.9% (all within 5-20% bounds).

## Pack-Quality Variance (committed, picks 6+)

S/A distribution: 0 cards=35%, 1=43%, 2=1%, 3=0%, 4=17%, 5=3%. Bimodal: untriggered packs give 0-1 S/A (78%), triggered packs give 4+ (20%). High stddev (1.55) reflects feast-or-famine pattern.

## Draft Traces

**Early Committer (Flash):** 57% deck S/A, conv pick 10. Trigger fires picks 10, 20, 30 with 4+ S/A each. Between triggers, 0-1 S/A. Long droughts punctuated by rich packs.

**Signal Reader (Storm):** 63% deck S/A, conv pick 8. Similar feast-or-famine. Triggered 6-card packs provide clear archetype signal.

**Power Chaser (Ramp):** 43% deck S/A, conv pick 2. Scatters across resonances, trigger fires rarely.

## Baseline Comparison

V3 Lane Locking: 2.72 S/A, pick 6.1. V4 Pack Widening: 3.35 S/A, pick 6.0. Champion (thresh=2) at 1.32 falls short. Thresh=1 at 2.13 approaches Lane Locking with higher variance.

## Self-Assessment

| Goal | Score | Note |
|------|-------|------|
| 1. Simple | 8 | One sentence, one condition, one action |
| 2. No actions | 10 | Zero decisions verified |
| 3. Not on rails | 7 | Untriggered packs fully diverse |
| 4. No forced decks | 7 | Random base ensures variety |
| 5. Flexible | 5 | Single-resonance trigger is 4-archetype ambiguous |
| 6. Convergent | 4 | Champion 1.32; thresh=1 variant 2.13 |
| 7. Splashable | 6 | C/F=1.35 per pack |
| 8. Open early | 7 | 11% early trigger rate |
| 9. Signal reading | 5 | Pack size signals resonance match |

**Verdict:** Champion (thresh=2) does not cross 2.0 -- 22% resonance base rate makes 2-of-4 too rare (21%). Thresh=1 crosses 2.0 but fires 63%, losing conditional drama. Strength: natural variance (1.55 stddev), zero-decision simplicity. Weakness: fundamental tension between meaningful threshold and sufficient fire rate.
