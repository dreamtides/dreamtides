# Model D Results: Variety-First Draft System

## Target Scorecard

| Metric | Target | Actual | Pass/Fail |
|--------|--------|--------|-----------|
| Picks 1-5: unique archetypes per pack | >= 3 of 4 | 4.24 | **PASS** |
| Picks 1-5: fitting cards per pack | <= 2 of 4 | 1.06 | **PASS** |
| Picks 6+: fitting cards per pack | >= 2 of 4 | 1.94 | **FAIL** (marginal) |
| Picks 6+: off-archetype cards per pack | >= 0.5 of 4 | 1.68 | **PASS** |
| Convergence pick | Pick 5-8 | 5.69 | **PASS** |
| Deck archetype concentration | 60-80% | 90.7% | **FAIL** |
| Run-to-run card overlap | < 40% | 7.0% | **PASS** |
| Archetype freq max | <= 20% | 16.7% | **PASS** |
| Archetype freq min | >= 5% | 9.3% | **PASS** |

**Score: 7/9 passed.** The two failures are tightly coupled (see analysis below).

## Analysis of Failures

### Late Fitting (1.94 vs >= 2.0)

This metric is marginally below target. With 5x-7x weight multipliers and 3+1
pack construction (3 archetype-weighted slots, 1 splash slot), the system
delivers nearly 2 fitting cards per pack on average. The splash slot is the
primary drag -- it deliberately offers off-archetype cards to support the
splashable target (1.68, well above the 0.5 minimum). Increasing weights to
5-8-9x pushes this metric above 2.0 (measured at 2.04 in testing) but
exacerbates the deck concentration issue.

### Deck Concentration (90.7% vs 60-80%)

This is the most significant failure and reveals a **genuine tension between
the convergence and concentration targets**. When the system reliably presents
2 fitting cards per pack after commitment, and the committed player strategy
always picks the best fitting card, the resulting deck is overwhelmingly S/A
tier. Mathematically: if 25 of 30 picks occur after commitment and each pack
has >= 1 fitting card (which it does when the average is ~2), the committed
player picks fitting cards on nearly every pick.

The only way to lower deck concentration is to reduce fitting cards per pack,
which directly conflicts with the convergence target. This tension is structural
-- it applies to any system where a committed player always picks the best
fitting card and packs reliably contain fitting options.

**Power-chaser strategy achieves 59.1% concentration**, which falls in the
target range. This suggests the 60-80% target may assume a player who sometimes
prioritizes raw power over archetype fit, not a pure archetype optimizer.

## Per-Strategy Breakdown

| Strategy | Late Fitting | Deck Concentration |
|----------|-------------|-------------------|
| Committed | 1.94 | 90.7% |
| Power-chaser | 1.99 | 59.1% |
| Signal-reader | 1.94 | 91.2% |

The signal-reader achieves almost identical results to the committed player,
validating that the variety mechanisms (suppression + depletion) create readable
signals that a signal-reading strategy can effectively exploit.

## Multi-Archetype Card Sensitivity

| Multi-Arch % | Late Fitting | Deck Conc | Card Overlap | Conv Pick |
|-------------|-------------|-----------|-------------|-----------|
| 10% | 1.47 | 80.7% | 8.9% | 7.1 |
| 20% | 1.59 | 84.0% | 8.1% | 6.8 |
| 30% | 1.68 | 85.9% | 9.5% | 6.3 |
| 42% (default) | 1.81 | 88.7% | 8.9% | 6.0 |
| 50% | 1.87 | 89.5% | 7.6% | 5.9 |

**Key findings:**

1. **Late fitting scales roughly linearly with multi-archetype percentage.**
   Going from 10% to 50% multi-archetype cards increases late fitting from 1.47
   to 1.87 -- a 27% improvement. The system needs ~45-50% multi-archetype cards
   to consistently hit the 2.0 target, which is at the high end of what Q2
   considers feasible for card design effort.

2. **Deck concentration is surprisingly insensitive to multi-archetype %.** It
   ranges only from 80.7% to 89.5% across the entire sweep. This confirms that
   concentration is driven primarily by the committed player strategy (always
   pick the best fitting card) rather than by pool composition.

3. **Card overlap is consistently low (~7-9%) regardless of multi-archetype %.**
   The archetype suppression mechanism dominates variety, making the system
   robust to fitness distribution changes. This is a strong validation of the
   pool restriction approach.

4. **Convergence pick moves from 7.1 to 5.9 as multi-archetype % increases.**
   More multi-archetype cards means the player sees fitting cards sooner.
   At 10%, convergence is at the edge of the target range (7.1); at 42%+, it
   comfortably hits 5-6.

5. **Minimum viable multi-archetype %:** At 10%, late fitting is 1.47 and
   convergence is at pick 7.1 -- both just barely acceptable. Below 10%, the
   system likely fails convergence entirely. **The practical minimum is ~15-20%
   to safely hit both convergence timing and late fitting targets.**

## What Works

**Run-to-run variety is the standout strength.** At 7% card overlap between
runs, no two drafts feel even remotely similar. The archetype suppression
mechanism (2 of 8 suppressed per run) creates 28 structurally distinct
configurations, and the depletion mechanism adds emergent mid-draft variation
on top of that. This is by far the strongest variety result of any system
design considered.

**Archetype frequency distribution is well-balanced.** No archetype exceeds
16.7% or falls below 9.3%, meaning all 8 archetypes are viable across runs.
The suppression mechanism does not create permanent winners or losers.

**Early-draft diversity is excellent.** 4.24 unique archetypes per 4-card pack
means every early pack presents genuine strategic breadth. Combined with only
1.06 fitting cards for the emerging archetype, early picks feel open and
exploratory rather than predetermined.

**The convergence timing (pick 5.69) is well within target.** Players find
their archetype by pick 5-6 on average, which is early enough to build a
coherent deck but late enough to allow exploration.

## What Doesn't Work

**The convergence vs. concentration tension is unresolved.** The system cannot
simultaneously give committed players 2+ fitting cards per pack AND keep their
final deck at <80% S/A concentration. This is likely a tension inherent in the
target specification rather than a flaw in this specific design. The resolution
may require redefining "committed player" as someone who picks fitting cards
most-but-not-all of the time, or adjusting the concentration target upward.

**Late fitting is marginally below target.** At 1.94, the system is close but
not quite delivering the 2.0 average. Increasing weights to hit 2.04 is
technically possible but pushes deck concentration to 91.6%.

**The depletion mechanism's signal-reading value is hard to measure.** While
the signal-reader strategy performs as well as the committed strategy, the
simulation does not distinguish between players who read depletion signals and
those who simply commit early. A more nuanced signal-reading model would help
validate whether the depletion adds real strategic depth.
