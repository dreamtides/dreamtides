# V9 Algorithm Overview: Complete Catalog

This document catalogs every algorithm tested in V9, ordered by recommendation
strength. V9's defining contribution over V8 is the hidden/visible information
split: keeping V8's mathematical gains (M3 >= 2.0, M11 >= 3.0) while reducing
visible dual-resonance from 40% to 10% of the pool.

______________________________________________________________________

## Recommended: Hybrid B -- Affinity-Tagged Gravity

**The only algorithm that passes M11 >= 3.0.**

**One sentence (player-facing):** "As you draft, the game removes cards that
don't match your style, so your future packs get better."

**One sentence (technical):** Pool contraction at 12% per pick using a blended
relevance score (40% visible dot-product + 60% pair-affinity score for the
committed archetype), with one top-quartile floor slot from pick 3.

### Mechanism

Combines Design 2's Tag-Gravity contraction framework with Design 5's honest
affinity insight, using a minimal two-float pair-affinity encoding instead of
either a binary tag or a full affinity vector.

**Hidden metadata:** 8 bits per card. Two 4-bit floats encoding the card's
affinity for each of the two archetypes sharing its primary resonance symbol. A
Tide card carries (warriors_affinity, sacrifice_affinity). Derived from card
mechanics using published rules. V3 = 9/10.

**Draft algorithm:**

1. Maintain a 4-element resonance signature (+2 primary, +1 secondary per pick).
2. From pick 4, compute relevance per pool card: 0.4 * visible_dot_product + 0.6
   - affinity_score[committed_archetype].
3. Remove bottom 12% of pool per pick. Generics protected at 0.5 baseline.
4. Pack construction: 1 top-quartile floor slot from pick 3; 3 random slots from
   surviving pool.
5. Archetype inference from pick 5: committed archetype = the archetype with
   higher mean affinity among drafted cards sharing the dominant resonance.

### Full Metrics

| Metric | Graduated Realistic | Pessimistic | Target | Status |
| ------ | :-----------------: | :---------: | ------ | ------ |
| M3     |        2.70         |    2.60     | >= 2.0 | PASS   |
| M11    |        3.25         |    3.17     | >= 3.0 | PASS   |
| M10    |         3.8         |     4.5     | \<= 2  | FAIL   |
| M6     |         86%         |     83%     | 60-90% | PASS   |
| M5     |         9.6         |     --      | 5-8    | FAIL   |
| M9     |        1.08         |     --      | >= 0.8 | PASS   |

| V-Metric |    Value    | Assessment                                                          |
| -------- | :---------: | ------------------------------------------------------------------- |
| V1       |    84.8%    | Visible symbols are doing 85% of the targeting work                 |
| V2       | 8 bits/card | 2.7x more than a 3-bit tag; 4x less than full affinity              |
| V3       |    9/10     | Affinities derived from card mechanics; no forced misclassification |
| V4 gap   |    2.08     | Committed players dramatically outperform power-chasers             |

### Per-Archetype M3

| Archetype    |   M3 |  M11 |
| ------------ | ---: | ---: |
| Flash        | 2.58 | 3.12 |
| Blink        | 2.63 | 3.19 |
| Storm        | 2.66 | 3.21 |
| Self-Discard | 2.76 | 3.31 |
| Self-Mill    | 2.82 | 3.38 |
| Sacrifice    | 2.75 | 3.26 |
| Warriors     | 2.78 | 3.30 |
| Ramp         | 2.66 | 3.22 |

All 8 archetypes pass M3 >= 2.0 and M11 >= 3.0. Spread = 0.25 (best equity of
any V9 algorithm).

### Card Designer Guidance

Assign two affinity scores (0.0-1.0) per non-generic card: one for each
archetype sharing the card's primary resonance. Use card mechanics as the guide:
does this Tide card care about combat (high Warriors affinity) or sacrifice
triggers (high Sacrifice affinity)? Bridge cards with meaningful value for both
archetypes should receive moderate scores for both (0.4-0.6 each). Design 5-10
bridge cards per resonance pair intentionally. Design 4-5 visible dual-res
signpost cards per archetype as mechanical hinges.

______________________________________________________________________

## Viable Alternative: Design 4 -- Layered Salience (Two-Stage Filter)

**Best V1 architecture. Recommended if M11 >= 3.0 is not a hard requirement.**

**One sentence:** "As you draft, packs shift toward your resonance and then
toward your specific style within it."

### Mechanism

Two-stage slot construction: Stage 1 (visible) gates 3 of 4 slots to the
committed primary resonance via R1 filtering. Stage 2 (hidden) weights
home-archetype-tagged cards at 4x within the R1 pool. Late-draft contraction
from pick 12 at 8% per pick.

**Hidden metadata:** 3 bits per card. One archetype tag. 1,080 bits total.

### Key Metrics

| Metric | Value | Status                    |
| ------ | ----: | ------------------------- |
| M3     |  2.36 | PASS                      |
| M11    |  2.40 | FAIL                      |
| M10    |  2.13 | MARGINAL FAIL             |
| V1     | 76.3% | Best among all algorithms |
| V3     |  8/10 | Honest tag assignment     |

**Why viable:** The layered architecture provides the strongest structural
guarantee that visible symbols do primary work. Stage 1 (R1 filtering) cannot
activate without visible resonance commitment -- this is a categorical V1
advantage. M3 = 2.36 passes with 18% margin. All 8 archetypes above 2.0.

**Why not recommended:** M11 = 2.40 fails the 3.0 target by 25%. Late-draft
contraction starting at pick 12 is too late to concentrate the pool for M11. The
design predicted M11 = 3.0-3.1; simulation showed the prediction was optimistic.

**When to choose this:** If the team accepts M11 < 3.0 and wants the absolute
minimum hidden metadata with the highest visible-influence guarantee. Also
serves as the V9 integrity benchmark for evaluating other algorithms.

______________________________________________________________________

## Viable Alternative: Design 2 -- Tag-Gravity (60/40 Blend)

**Direct successor to V8 Narrative Gravity. Best archetype equity among tag-only
algorithms.**

**One sentence:** "As you draft, the game removes cards that don't match your
style, so your future packs get better."

### Mechanism

V8 Narrative Gravity with a 60/40 blended relevance score (40% visible
dot-product + 60% hidden archetype tag match). Contraction at 12% per pick from
pick 4. Floor slot from pick 3.

**Hidden metadata:** 3 bits per card. 1,080 bits total.

### Key Metrics

| Metric | Value | Status                         |
| ------ | ----: | ------------------------------ |
| M3     |  2.37 | PASS                           |
| M11    |  2.81 | FAIL                           |
| M10    |   4.4 | FAIL                           |
| V1     | 98.1% | Unexpectedly high              |
| V3     |  8/10 | Honest tag assignment          |
| Spread |  0.18 | Best among tag-only algorithms |

**Surprising V1 finding:** V1 = 98.1% means the hidden tag adds only +0.04 M3
over visible-only contraction. The visible resonance symbol dominates
contraction so heavily that the tag's within-sibling discrimination (Warriors
vs. Sacrifice) contributes negligibly -- because at 50% sibling fitness,
Sacrifice cards are genuinely S/A for Warriors players.

**Why viable:** Simplest mechanism. Best archetype equity among tag-only
algorithms (spread 0.18). Direct continuation of V8's proven Narrative Gravity
family.

**Why not recommended:** M11 = 2.81 fails. M10 = 4.4 fails. The V1 = 98.1%
finding raises the question of whether the 3-bit tag adds enough value to
justify its existence for this algorithm specifically.

______________________________________________________________________

## Viable Alternative: Hybrid A -- Visible-First Anchor Gravity

**Highest M3 among tag-only algorithms. Strongest V4 gap.**

**One sentence:** "Your picks shape which cards appear in future packs --
especially dual-resonance cards that focus your deck."

### Mechanism

Design 4's layered R1 filtering + Design 6's anchor-scaled contraction rates (6%
generic / 10% single / 18% dual-res). Contraction from pick 5 using 60/40
visible/hidden blend.

**Hidden metadata:** 3 bits per card. 1,080 bits total.

### Key Metrics

| Metric | Value | Status                        |
| ------ | ----: | ----------------------------- |
| M3     |  2.62 | PASS (highest among tag-only) |
| M11    |  2.83 | FAIL                          |
| M10    |  5.26 | FAIL                          |
| V1     | 77.0% | Strong                        |
| V4 gap |  1.97 | Strongest                     |

**Why viable:** Highest M3 (2.62) of any 3-bit algorithm. Strong V1 (77%) from
layered architecture. The anchor mechanic creates visible cause-and-effect:
dual-res picks trigger 18% contraction, producing a felt quality burst.

**Why not recommended:** M11 = 2.83 fails. M10 = 5.26 fails. The anchor-scaled
contraction rates contract too slowly in the first 15 picks to achieve M11 >=
3.0. Only Warriors reaches M11 >= 3.0 individually (3.09).

______________________________________________________________________

## Eliminated: Design 5 -- AWNG (Affinity-Weighted Narrative Gravity)

**Failure mode: Over-parameterized hidden metadata that adds no targeting
precision.**

### Key Findings

- M3 = 2.32 (missed predicted 2.65-2.80 by 0.33-0.48)
- M11 = 2.71 (fails 3.0)
- V1 = 99.3% (the 8-float affinity vector is functionally identical to a 1-float
  resonance score)
- V2 = ~32 bits/card (11x more than a 3-bit tag for no measurable gain)
- Blink fails M3 >= 2.0 (1.80)
- Archetype spread = 0.88 (worst equity)

**Root cause:** The +0.60 primary resonance contribution in the affinity vector
dominates all mechanical keyword contributions (+0.20 max). Stripping keywords
changes M3 by only 0.009. The 8-float design adds nothing over a simple
visible-symbol dot product for contraction purposes.

**Lesson:** Honest metadata derived from card mechanics (V3 = 9/10) does not
guarantee useful metadata. The mechanical keywords that distinguish Warriors
from Sacrifice are too sparse and weakly weighted to overcome the dominance of
the primary resonance signal. The correct abstraction is a direct pair-level
distinction (Hybrid B's two-float approach), not a bottom-up derivation from
mechanical features.

______________________________________________________________________

## Eliminated: Design 6 -- Anchor-Scaled Contraction

**Failure mode: Hidden archetype tags are net-negative. Archetype inference
unreliable for same-primary-resonance siblings.**

### Key Findings

- M3 = 2.00 (marginal pass)
- M11 = 2.21 (fails 3.0)
- M10 = 7.44 (catastrophic fail)
- V1 = paradox (visible-only M3 = 2.25 > full M3 = 2.00 -- hidden tags hurt
  performance)
- Per-archetype spread = 1.42 (Ramp at 1.28, Warriors at 2.70)
- 4 of 8 archetypes fail M3 >= 2.0

**Root cause:** At 10% visible dual-res, archetype inference based on hidden tag
majority-vote often resolves to the wrong same-primary-resonance sibling (e.g.,
Sacrifice instead of Warriors for a Tide-committed player). The contraction then
culls the correct archetype's secondary-resonance cards. The V1 paradox
(visible-only outperforms full) is diagnostic: undirected visible contraction
that preserves both Tide siblings performs better than misdirected tag-based
contraction that culls the wrong one.

**Lesson:** The anchor mechanic (differentiated contraction rates by pick type)
is a genuine player experience innovation, but it cannot compensate for
unreliable archetype inference. The anchor mechanic survives as a component of
Hybrid A, where it is paired with Design 4's layered R1 filtering that provides
a more robust inference foundation.

______________________________________________________________________

## Eliminated: Designs 1 and 3

**Design 1 (Tagged Narrative Gravity)** was displaced before simulation by the
critic review. It was functionally redundant with Design 2 (Tag-Gravity),
offering the same mechanism family (tag + contraction) with minor parameter
differences (additive +0.5 tag bonus vs. multiplicative 60/40 blend). Its
contribution -- confirming the 3-bit tag + contraction + floor slot mechanism
family -- was absorbed into the surviving designs.

**Design 3 (CSCT-2+C)** was displaced before simulation by the critic review.
Its commitment-ratio mechanism was the most complex of all proposals (three
independent sub-systems) for similar predicted performance to simpler
alternatives. The V1 accounting was identified as overstated. Its core insight
-- that the player's rate of visible commitment, not just direction, should
determine pack quality -- is a genuine design property that no other V9
algorithm tests, and may be worth revisiting in a future version.

______________________________________________________________________

## Structural Findings

### 1. Pool contraction is mandatory for M11 >= 3.0

Every algorithm that achieves M11 >= 3.0 uses pool contraction. No slot-filling
algorithm at 10% visible dual-res can deliver 3+ S/A cards per 4-card pack in
picks 15+ without concentrating the pool first. This was predicted by the math
ceiling research and confirmed by all six simulations.

### 2. The 3-bit archetype tag is necessary but not sufficient

All six simulations used hidden metadata. The minimum useful unit is a 3-bit
archetype tag (1,080 bits total for 360 cards). It lifts M3 from ~2.05
(visible-only ceiling) to 2.36-2.62. However, no 3-bit-tag algorithm achieves
M11 >= 3.0. The binary forced-assignment creates bridge card misclassification
that caps late-draft precision below the M11 target.

### 3. Two-float pair affinity (8 bits) is the right abstraction level

Hybrid B demonstrates that the critical hidden information is the relative value
of each card for its two same-primary-resonance archetypes. This requires
exactly two floats (8 bits at 4-bit precision). Full 8-float affinity vectors
(AWNG) add nothing because the visible resonance symbol already provides the
dominant contraction signal. The two-float encoding captures the one thing
visible symbols cannot: which of two same-resonance archetypes does this card
serve better?

### 4. Visible symbols do 77-99% of targeting work

Every simulation measured V1 >= 76%. The pre-simulation predictions of V1 =
40-50% were systematically pessimistic. At 10% visible dual-res, pool
contraction driven by visible resonance symbols already concentrates the pool
effectively on the committed primary resonance. Hidden metadata provides
within-sibling refinement worth 0.04-0.49 additional M3 points. The visible
resonance system is genuinely the primary drafting signal across all tested
algorithms.

### 5. The M3-M10-M6 triangle persists from V8

No V9 algorithm achieves M10 \<= 2 alongside M3 >= 2.0 and M6 in 60-90%. Design
4 comes closest (M10 = 2.13) but at the cost of M11 failure. The transition zone
(picks 6-10), where archetype inference is stabilizing and pool contraction has
not yet concentrated the pool, remains structurally resistant to M10
improvement. This is the same finding as V8, now confirmed across a different
pool composition and hidden metadata scheme.

### 6. Archetype inference is the critical implementation challenge

Design 6's catastrophic failure (hidden tags net-negative) demonstrates that
reliable archetype inference from early picks is harder than the design phase
assumed. At 10% visible dual-res, the player's first 5 picks rarely contain
enough information to distinguish between two archetypes sharing a primary
resonance. Pair-affinity encoding (Hybrid B) mitigates this by allowing the
algorithm to maintain partial commitment to both archetypes until evidence
accumulates, rather than forcing a binary choice at pick 5.

### 7. Design integrity and performance are not in tension

The V9 data shows no tradeoff between V3 (design integrity) and M3
(performance). Hybrid B achieves the highest V3 (9/10) alongside the highest M3
(2.70) and the only M11 pass (3.25). Honest metadata derived from genuine card
properties is also the most effective metadata -- because it captures real
mechanical distinctions that predict in-game performance. The hypothesized
"honest but weak vs. arbitrary but strong" tradeoff does not appear in the data.
