# Results 2: Square-Root Affinity Sampling

## One-Sentence Algorithm

"Each card in the pool is drawn with weight 1.5 + min(sqrt(symbol overlap with
your drafted deck), 4.5), so cards matching your deck appear more often but
with diminishing returns."

Uses ONLY visible properties (symbols). Archetype fitness used only for evaluation.

## Target Scorecard (Committed Player, Archetype-Level)

| Metric | Target | Actual | P/F |
|--------|--------|--------|-----|
| Picks 1-5: unique archetypes w/ S/A per pack | >= 3 | 6.48 | PASS |
| Picks 1-5: S/A for emerging archetype per pack | <= 2 | 1.87 | PASS |
| Picks 6+: S/A for committed archetype per pack | >= 2 | 1.74 | **FAIL** |
| Picks 6+: off-archetype (C/F) cards per pack | >= 0.5 | 1.11 | PASS |
| Convergence pick | 5-8 | 10.5 | **FAIL** |
| Deck concentration | 60-90% | 89.0% | PASS |
| Run-to-run overlap | < 40% | 12.1% | PASS |
| Archetype freq (max / min) | <20% / >5% | 19.4% / 7.1% | PASS |
| Late S/A stddev | >= 0.8 | 1.00 | PASS |

**7/9 pass.** Fails convergence strength and timing. Root cause: ~50% of
resonance-matched cards belong to wrong archetypes. Excellent variance and
splash, but indirect resonance-to-archetype mapping limits convergence.

## Variance Report

StdDev: **1.00** (target >= 0.8: PASS). Distribution of S/A cards per pack
(picks 6+): 0 cards=10.5%, 1=31.2%, 2=36.1%, 3=18.7%, 4=3.6%. Committed
players see 2+ S/A 58% of the time but 0 S/A in 10% of packs. Great packs
(3-4 S/A) occur 22% of the time.

## Per-Archetype Convergence Table

| Archetype | Avg Convergence Pick |
|-----------|:---:|
| Flash/Tempo/Prison | 16.1 |
| Blink/Flicker | 15.8 |
| Storm/Spellslinger | 17.5 |
| Self-Discard | 16.4 |
| Self-Mill/Reanimator | 17.5 |
| Sacrifice/Abandon | 16.1 |
| Warriors/Midrange | 15.9 |
| Ramp/Spirit Animals | 15.9 |

Uniform across archetypes (1.7-pick spread), confirming balance. Absolute
values high because "3 consecutive 2+ S/A" is demanding at 1.74 average.

## V3 Lane Locking Comparison

| Metric | Target | SqrtAff | LaneLk |
|--------|--------|:---:|:---:|
| Picks 1-5 unique archetypes | >= 3 | 6.48 | 6.51 |
| Picks 1-5 S/A for emerging | <= 2 | 1.87 | 1.83 |
| Picks 6+ S/A for committed | >= 2 | **1.74** | **2.39** |
| Picks 6+ off-archetype (C/F) | >= 0.5 | **1.11** | 0.60 |
| Convergence pick | 5-8 | 10.5 | 6.5 |
| Deck concentration | 60-90% | 89.0% | **96.2%** |
| Run-to-run overlap | < 40% | **12.1%** | 17.2% |
| Late S/A stddev | >= 0.8 | **1.00** | 0.94 |

Lane Locking converges stronger/faster but over-concentrates (96.2% > 90%
cap). Sqrt Affinity wins splash, variance, and variety. Tradeoff: reliable
delivery vs natural variation.

## Symbol Distribution and Sensitivity

Default: 25/55/20% (1/2/3-sym), 36 generic. Robust: 1-sym-heavy gave 1.78
S/A, extreme 1-sym gave 1.82. Not sensitive to symbol distribution.

## Parameter Sensitivity

| Param | Range | Late S/A | Off-Arch |
|-------|:---:|:---:|:---:|
| Base | 1.0-2.0 | 1.80-1.70 | 1.06-1.14 |
| Cap | 3.0-uncapped | 1.61-1.94 | 1.24-0.95 |
| Exponent | 0.33-1.0 | 1.72-1.63 | 1.13-1.22 |

Cap is most impactful. Aggressive config (base=1.0, no cap) achieves **2.01
S/A** with 0.89 off-archetype (both pass), but convergence pick remains ~8.9.
Base=0.5 pushes to 2.11 S/A but over-concentrates (94.2%). The 2.0 S/A target
is reachable with parameter tuning but at the cost of the algorithm's
signature splash advantage.

## Draft Traces

**Early Committer (Warriors/Sacrifice):** Committed pick 6. Packs ranged 0-4
S/A with natural variance — pack 12 had 4 S/A, pack 20 had 0. Final: 28/30
S/A (93%).

**Flexible Player (Self-Mill):** Emerged organically. Pack variance 0-4 S/A
throughout. Pick 21 had 0 S/A despite heavy commitment, demonstrating genuine
randomness. Final: 28/30 (93%).

**Signal Reader (Ember depleted):** 20% Ember cards removed from pool. Reader
gravitated to Tide/Stone (Sacrifice) by pick 5. Final: 26/30 (87%). Pool
asymmetry creates readable signals.

## Self-Assessment

| Goal | Score | Justification |
|------|:---:|---|
| 1. Simple | 6 | Concrete formula, but "square root" requires math literacy |
| 2. Not on rails | 9 | 10.5% of packs have 0 S/A; never locked in |
| 3. No forced decks | 8 | 12.1% overlap; pool too large to force |
| 4. Flexible archetypes | 8 | Concave scaling supports mixed investments |
| 5. Convergent | 4 | 1.74 misses 2.0 target; tunable to ~2.0 aggressively |
| 6. Splashable | 9 | 1.11 off-archetype per pack, well above 0.5 |
| 7. Open early | 9 | 6.48 unique archetypes in picks 1-5 |
| 8. Signal reading | 3 | Requires layered pool asymmetry; no native signal |
