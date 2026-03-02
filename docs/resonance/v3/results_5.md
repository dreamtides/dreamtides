# Results: Resonance Swap (Modified) — Agent 5 (Corrected)

## One-Sentence Algorithm

"When you draft a card, 3 random cards matching its primary resonance are added
to the draft pool from a reserve, and 3 random cards of other resonances are
moved from the pool to the reserve; each run starts with one resonance boosted
(+20) and another suppressed (-20)."

## Critical Correction

Previous metrics measured at the RESONANCE level. A resonance (Tide) is shared
by multiple archetypes (Warriors, Sacrifice). This rerun measures at the
ARCHETYPE level: a card "fits" only with S/A tier fitness for the specific
target archetype.

## Target Scorecard (archetype_committed, 1000 drafts)

| Metric | Target | Actual | Result |
|--------|--------|--------|--------|
| Picks 1-5: unique archetypes w/ S/A per pack | >= 3 | 6.44 | **PASS** |
| Picks 1-5: S/A cards for emerging arch per pack | <= 2 | 1.68 | **PASS** |
| Picks 6+: S/A cards for committed arch per pack | >= 2 | 1.58 | **FAIL** |
| Picks 6+: C/F-tier cards per pack | >= 0.5 | 1.24 | **PASS** |
| Convergence pick | 5-8 | 7.6 | **PASS** |
| Deck concentration (S/A) | 60-80% | 83.2% | **FAIL** |
| Run-to-run overlap | < 40% | 6.9% | **PASS** |
| Archetype frequency | 5-20% each | 9.7%-15.4% | **PASS** |

**6/8 passed.** Late S/A (1.58) misses 2.0 due to the pool-size bottleneck:
swapping 3 cards in a 360-card pool shifts resonance too slowly. Deck
concentration exceeds 80% because adjacent archetypes all get A-tier.

## Symbol Distribution

25% 1-symbol, 50% 2-symbol, 15% 3-symbol among non-generic; 36 generic cards.

## Parameter Sensitivity

**Swap count (2/3/4):**

| Swap | S/A late | Conv. pick | Conv. rate | Deck conc. |
|------|----------|------------|------------|------------|
| 2 | 1.50 | 8.2 | 83.4% | 81.8% |
| 3 | 1.57 | 7.6 | 91.0% | 83.3% |
| 4 | 1.65 | 7.8 | 94.0% | 84.4% |

Even swap=4 cannot reach 2.0 S/A per pack. The 360-card pool size is the
binding constraint.

**Symmetric vs asymmetric:**

| Mode | S/A late | Conv. pick | Overlap |
|------|----------|------------|---------|
| Symmetric | 1.55 | 8.6 | 6.7% |
| Asymmetric | 1.57 | 7.6 | 6.9% |

Asymmetric improves convergence timing without harming other metrics.

**Reserve size (150/200/300):**

| Reserve | S/A late | Conv. pick | Overlap |
|---------|----------|------------|---------|
| 150 | 1.55 | 7.4 | 6.6% |
| 200 | 1.57 | 7.6 | 6.9% |
| 300 | 1.61 | 7.9 | 5.9% |

Minimal impact. Reserve recycling prevents exhaustion.

**Signal detection:** 44.8% correct identification of boosted resonance (vs 25%
baseline, +19.8pp). Strongest signal-reading result of any domain.

## Draft Traces

**Early Committer (Warriors):** Boosted=Ember, Suppressed=Tide. Locks Warriors
by pick 4. Sees 2+ S/A in picks 4, 8, 11, 14, 17, 28, 30. Pool drifts Tide
from 19% to 28%. Final: 73.3% S/A (S=11, A=11, B=8).

**Flexible Player (power-chaser):** Archetype drifts (Ramp->Storm->Blink).
Pool barely shifts. Final: 36.7% S/A (S=3, A=8, B=13, C=4, F=2). Algorithm
correctly avoids forcing convergence.

**Signal Reader (Self-Discard):** Boosted=Zephyr, Suppressed=Stone. Commits
Self-Discard by pick 5. Pool shifts Stone 20% to 28%. Final: 76.7% S/A
(S=14, A=9, B=5). Late packs frequently show 2-3 S/A cards.

## Self-Assessment

| Goal | Score | Justification |
|------|-------|---------------|
| 1. Simple | 7/10 | Concrete description; reserve is hidden infrastructure. |
| 2. Not on rails | 8/10 | Gentle pool shifts keep all strategies viable. |
| 3. No forced decks | 9/10 | 6.9% overlap is extremely low. |
| 4. Flexible | 7/10 | Pool always contains all resonances. |
| 5. Convergent | 4/10 | 1.58 S/A misses 2.0; structurally unfixable. |
| 6. Splashable | 8/10 | 1.24 C/F per pack far exceeds 0.5. |
| 7. Open early | 9/10 | 6.44 unique archetypes (target 3). |
| 8. Signal | 9/10 | 44.8% detection rate, strongest of any domain. |

**Key finding:** Correcting to archetype-level metrics slightly reduced late S/A
from 1.61 to 1.58 but did not change the fundamental picture. Pool manipulation
remains best as a complementary layer providing signal reading and variety atop
a primary convergence mechanism.
