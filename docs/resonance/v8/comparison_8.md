# Comparison Agent 8: Compensated Pair Allocation Perspective

## Honest Assessment: My Algorithm's Failure and What It Teaches

CPA failed at M3 = 1.45 under Graduated Realistic. The root cause is
pair-counter misalignment (~40% of drafts). This is the same failure mode as
Agent 3's Escalating Pair Lock (55% alignment) and Agent 6's GF+PE
(per-archetype M3 ranging from 1.13 to 2.31). Three of nine agents independently
discovered that **discrete pair-counter mechanisms are fragile.**

The one salvageable contribution: pool compensation (distributing more dual-res
cards to low-overlap pairs) narrows the per-archetype M3 gap from 0.37 to 0.27.
This applies to any algorithm.

## Scorecard: Fitness Degradation (40% Enriched)

| Algorithm            | Opt. | Grad. | Pess. | Host. | Total Drop | Degradation Class |
| -------------------- | :--: | :---: | :---: | :---: | :--------: | ----------------- |
| 7. CSCT              | 3.07 | 2.92  | 2.88  | 2.85  |   -0.22    | Immune            |
| 9. Narr. Gravity     | 3.39 | 2.75  | 2.59  | 2.49  |   -0.90    | Robust            |
| 5. Sym-Weighted (SR) | 2.88 | 2.50  | 2.49  | 2.34  |   -0.54    | Robust            |
| 1. Pair-Esc.         | 2.34 | 2.16  | 2.12  | 2.08  |   -0.26    | Robust            |
| 2. Cont. Surge       | 3.10 | 2.48  | 2.43  | 2.25  |   -0.85    | Moderate          |
| 4. GPE-45            | 2.73 | 2.25  | 2.21  | 2.05  |   -0.68    | Moderate          |
| 6. GF+PE             | 2.57 | 1.72  | 1.58  | 1.34  |   -1.23    | Fragile           |
| 3. Esc. Pair Lock    | 1.98 | 1.50  | 1.46  | 1.31  |   -0.67    | Fragile           |
| 8. Comp. Pair        | 2.16 | 1.45  | 1.40  | 1.29  |   -0.87    | Fragile           |

Algorithms cluster into three degradation classes:

- **Immune** (< 0.3 drop): CSCT, Pair-Esc. baseline
- **Robust** (0.3-0.9 drop): Narrative Gravity, Symbol-Weighted, Continuous
  Surge, GPE-45
- **Fragile** (> 0.9 drop or misalignment-dependent): GF+PE, Esc. Pair Lock, CPA

Fragile algorithms share a common trait: they depend on precise pair
identification from noisy early signals.

## Player Experience Rating (1-10)

| Algorithm         | Rating | Justification                                                       |
| ----------------- | :----: | ------------------------------------------------------------------- |
| 9. Narr. Gravity  |   9    | I give the highest rating because it solves the problem I could not |
| 5. Sym-Weighted   |   7    | Strong graduated ramp; symbol-rich pool is ideal but demanding      |
| 7. CSCT           |   6    | Smooth delivery is genuine value, but autopilot after pick 5        |
| 2. Cont. Surge    |   5    | Better than bimodal surge/floor, worse than monotonic ramp          |
| 1. Pair-Esc.      |   5    | Solid baseline                                                      |
| 4. GPE-45         |   4    | Dead zone problem                                                   |
| 6. GF+PE          |   3    | Floor concept fails under realistic fitness                         |
| 8. Comp. Pair     |   2    | My algorithm: misalignment is fatal                                 |
| 3. Esc. Pair Lock |   1    | Broken alignment                                                    |

I rate Narrative Gravity 9/10 because it does exactly what CPA was designed to
do (compensate for per-archetype variance) but through a fundamentally superior
mechanism (pool contraction vs. pair counters). Narrative Gravity's pool
contraction naturally compensates for low-overlap archetypes by removing the
sibling B/C cards that cause the problem.

## KEY QUESTION 1: M3 >= 2.0 under harshest fitness?

**Per-archetype analysis is essential.** Aggregate M3 hides critical failures:

| Algorithm        | Agg. M3 (GR) | Worst Arch (GR) |   All >= 2.0?    |
| ---------------- | :----------: | :-------------: | :--------------: |
| 7. CSCT          |     2.92     |  2.88 (Storm)   | Yes (but M6=99%) |
| 9. Narr. Gravity |     2.75     |  2.40 (Flash)   |       Yes        |
| 5. Sym-Weighted  |     2.50     |  1.88 (Blink)   |        No        |
| 2. Cont. Surge   |     2.48     |   1.55 (Ramp)   |        No        |
| 4. GPE-45        |     2.25     |  1.92 (Flash)   |        No        |
| 1. Pair-Esc.     |     2.16     |  2.12 (Flash)   |       Yes        |

Only three algorithms have all archetypes above 2.0 under Graduated Realistic:
CSCT (disqualified by M6), Narrative Gravity, and Pair-Escalation baseline.
Narrative Gravity is the only one that passes both the per-archetype threshold
AND maintains acceptable M6/M9.

## KEY QUESTION 2: Best feel?

Narrative Gravity. Its mechanism is intuitive ("the draft adapts to you"), its
quality ramp is monotonic, and its run-to-run variety (M7 = 5.3%) means every
draft feels different. The M10 = 3.3 weakness is the one concern, and other
agents have proposed floor mechanisms to address it.

## The Compensation Insight Applied to Narrative Gravity

My algorithm's one surviving contribution -- pool compensation -- should be
applied to Narrative Gravity. Instead of 16 dual-res cards per archetype pair
uniformly:

| Pair                   | Overlap        | Standard | Compensated |
| ---------------------- | -------------- | :------: | :---------: |
| Warriors/Sacrifice     | High (50%)     |    16    |     14      |
| Self-Discard/Self-Mill | Medium (40%)   |    16    |     16      |
| Blink/Storm            | Low (30%)      |    16    |     18      |
| Flash/Ramp             | Very Low (25%) |    16    |     18      |

Total remains 128 dual-res cards (4 x 14 + 0 x 16 + 4 x 18 = 128). The extra 2
cards per low-overlap pair provide a larger pair-matched subpool, improving
early-draft quality for Flash/Ramp players.

## Proposed Best Algorithm + Pool Composition

**Narrative Gravity (12% contraction) on 40% Enriched Compensated Pool**

**Set Design Specification (Compensated Variant):**

| Archetype    |  Total  | Home-Only | Dual-Res | Generic |
| ------------ | :-----: | :-------: | :------: | :-----: |
| Flash        |   40    |    22     |    18    |   --    |
| Blink        |   40    |    22     |    18    |   --    |
| Storm        |   40    |    22     |    18    |   --    |
| Self-Discard |   40    |    24     |    16    |   --    |
| Self-Mill    |   40    |    24     |    16    |   --    |
| Sacrifice    |   40    |    26     |    14    |   --    |
| Warriors     |   40    |    26     |    14    |   --    |
| Ramp         |   40    |    22     |    18    |   --    |
| Generic      |   40    |    --     |    --    |   40    |
| **Total**    | **360** |  **188**  | **132**  | **40**  |

**Symbol Distribution:** 40 generic (11%), 188 single-res (52%), 132 dual-res
(37%).

**Per-Resonance:** R1 pool = 80 cards per resonance. Pair subpool: 14-18 cards
per archetype (compensated).

**Cross-Archetype Requirements:** Warriors/Sacrifice 50% A-tier (7/14 dual-res),
Self-Disc./Self-Mill 40% (6/16), Blink/Storm 30% (5/18), Flash/Ramp 25% (4/18).

**Minimum fitness: Graduated Realistic (36% avg).** Even under Pessimistic,
Narrative Gravity delivers M3 = 2.59.

**Minimum pool change:** 132 dual-res cards (up from 54). Compensation adds only
4 cards of rebalancing vs. uniform 128.

## Recommendations to the Card Designer

1. Adopt the compensated pool: give 2 extra dual-res cards to Flash, Blink,
   Storm, and Ramp (low-overlap pairs) while removing 2 each from Sacrifice and
   Warriors (high-overlap pairs). The total remains 360 cards.
2. The dual-res symbol assignment is a tagging exercise, not a fitness
   requirement. A Warriors card with (Tide, Zephyr) symbols need not be playable
   in Ramp.
3. For the 4-5 bridge cards per low-overlap pair: design characters with
   universal utility (efficient bodies, unconditional draw, removal) that carry
   the correct pair symbols.
4. Narrative Gravity's contraction handles the fitness problem algorithmically.
   The card designer's job is symbol assignment and reasonable (not heroic)
   bridge-card creation.
5. Report per-archetype M3 during playtesting. If Flash consistently
   underperforms, add 1-2 more bridge cards to the Flash/Ramp pair.
