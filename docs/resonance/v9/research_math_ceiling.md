# Mathematical Ceiling Analysis: Hidden Metadata vs. Visible Information

## Research Question

What is the maximum M3 achievable with only visible information (~10% dual-res,
no hidden metadata), and how much does each type of hidden metadata add?

______________________________________________________________________

## 1. Establishing the Precision Equation

V8's pool composition research derived the foundational equation governing all
targeting algorithms:

```
P(S/A | R1 filter) = 0.5 + 0.5 * F
```

where F = sibling A-tier rate (Graduated Realistic: Warriors/Sacrifice 50%,
Self-Discard/Self-Mill 40%, Blink/Storm 30%, Flash/Ramp 25%; weighted avg 36%).

For pair-matched filtering (visible dual-res symbols), the home-archetype
concentration of ~80% gives a fitness-resistant equation:

```
P(S/A | pair filter) ≈ 0.80 + 0.20 * F
```

At Graduated Realistic weighted average (36%): P(pair) ≈ 0.87. For the worst
pair (Flash/Ramp, F=25%): P(pair) ≈ 0.85.

The M3 target requires (with random slot baseline P(random) ≈ 0.125):

```
N_targeted * P + (4 - N_targeted) * 0.125 = 2.0
N_targeted = 1.875 / (P - 0.125)
```

This gives the minimum targeted slots needed for M3 = 2.0 at each precision
level:

| Precision P | Source                               | Targeted slots needed |             Feasible?              |
| :---------: | ------------------------------------ | :-------------------: | :--------------------------------: |
| 0.875 (avg) | Pair filter, GR fitness              |         2.12          |          Yes (2-3 slots)           |
|    0.850    | Pair filter, worst pair (Flash/Ramp) |         2.21          |                Yes                 |
|    0.625    | R1 filter, worst pair (F=25%)        |   2.00 = 4.00 slots   |       At limit (zero splash)       |
|    0.580    | R1 filter, GR weighted avg (F=36%)   |         3.49          | No (>3 targeted, leaves no splash) |
|    0.125    | Random (no targeting)                |       Infinite        |                 No                 |

**Core finding:** R1 filtering (the best visible-only strategy at 10% dual-res)
achieves 58-62.5% precision at Graduated Realistic fitness — not enough to reach
M3 = 2.0 with any pack structure that also passes M4 (splash) and M6
(concentration).

______________________________________________________________________

## 2. Level 0: Visible-Only Baseline (~10% Visible Dual-Res)

### Pool structure at 10% visible dual-res

V9 baseline: 36 dual-res cards across 360 (10%), 284 single-symbol (79%), 40
generic (11%). Per archetype: approximately 4-5 visible dual-res cards, 30-31
single-symbol, 5 generic.

**Pair-matchable subpool per archetype: ~4-5 cards.**

From V8's pool composition analysis, the minimum subpool for 1 sustained
pair-matched slot per pack over 25 post-commitment picks is 7 cards (6.25 * 1).
At 4-5 visible dual-res per archetype, even 1 pair-matched slot cannot be
sustained without excessive card repetition.

### Algorithmic ceiling with visible symbols only

The best visible-only algorithm tested was SF+Bias (R1) on V7's 15% dual-res
pool: **M3 = 2.24 (Graduated Realistic), all 8 archetypes >= 2.0 (worst: Ramp at
2.06)**. This is the proven visible-only ceiling at 15% dual-res.

At 10% dual-res, the ceiling drops further. The subpool effects compound:

- R1 pool precision is unchanged (still 58-87.5% depending on pair fitness)
- Pair-matched slot count drops from ~1 to essentially 0 (subpool too small)
- R1 filtering (single symbol) is the only viable targeting mode

**Estimated M3 ceiling at 10% visible dual-res, visible-only algorithms:**

SF+Bias (R1) on 15% pool = 2.24. Moving to 10% removes the modest pair-boost
component. R1 filtering alone at Graduated Realistic: N = 1.875/(0.58-0.125) =
4.12 targeted slots needed, exceeding pack size while leaving zero splash. The
best feasible configuration is 3 R1-filtered slots + 1 random:

```
M3 = 3 * 0.58 + 1 * 0.125 = 1.865
```

With R1 bias (not full filtering): approximately 2.0-2.1 aggregate, but with
per-archetype failures for Flash and Ramp where R1 precision at F=25% is only:

```
P(R1, Flash/Ramp) = 0.5 + 0.5 * 0.25 = 0.625
M3_max(Flash) = 3 * 0.625 + 1 * 0.125 = 2.00 (exactly at limit, no headroom)
```

V8 confirmed this empirically: SF+Bias (R1) on V7 pool achieves Flash = 2.07,
Ramp = 2.06 — at the very edge of passing, with no margin for M10 failures.

**Conclusion for Level 0:** M3 ceiling ≈ 2.0-2.1 at 10% visible dual-res with
visible-only algorithms. Flash and Ramp cannot reliably exceed 2.0 without
pair-matching. M10 failures (streak 6-8) are structural and cannot be addressed
without reducing M3. **M11 (picks 15+ target >= 3.0) is not achievable**: with
only R1 filtering and small dual-res pools, late-draft precision does not
increase — at pick 15+, the algorithm has no additional information over pick 6.

**V1 metric (visible symbol influence %):** 100% — but M3 < 2.0 for the worst
archetypes, and M11 not achievable. The visible system is at its ceiling.

______________________________________________________________________

## 3. Level 1: 3-Bit Hidden Archetype Tag (8 archetypes, one tag per card)

### What 3 bits provides

A 3-bit tag (log₂(8) = 3 bits) assigns each card to exactly one of 8 archetypes.
The algorithm can now query "cards tagged Warriors" rather than "cards with
(Tide, Zephyr) symbols."

### Precision impact

With 40 cards per archetype and each tagged to exactly one archetype, a "tagged
Warriors" query returns exactly 40 home-archetype cards. Precision:

```
P(S/A | archetype tag) = 1.0 (home cards are always S-tier)
```

This is the Optimistic ceiling for home-archetype draws. However, unlike visible
pair-matching which uses the actual dual-res pool, archetype tags apply to ALL
cards — including the 284 single-symbol cards.

**Effective subpool size per archetype with hidden tags: 40 cards.**

Compare to visible-only: ~4-5 pair-matchable cards. The 3-bit tag multiplies the
accessible pair-precision subpool by roughly 8x.

### M3 achievable with archetype tag targeting

With 40 home-archetype cards available for targeting and S/A precision = 100%
for tagged draws, the algorithm can fill 2-3 slots per pack with home-archetype
cards for the full 25-pick post-commitment draft without repetition exhaustion:

```
Minimum subpool for 2 slots/pack * 25 picks = 12.5 cards → 40 >> 12.5 ✓
Minimum subpool for 3 slots/pack * 25 picks = 18.75 cards → 40 >> 18.75 ✓
```

At 2 tagged home-archetype slots + 2 random slots:

```
M3 = 2 * 1.0 + 2 * 0.125 = 2.25
```

At 3 tagged + 1 random (risks M6 > 90%):

```
M3 = 3 * 1.0 + 1 * 0.125 = 3.125 (but M6 likely ~95-99%)
```

The M6 constraint limits aggressive tagging. With 2 home-tagged + 1 R1-filtered

- 1 random, the best M6-safe configuration:

  M3 = 2 * 1.0 + 1 * 0.58 + 1 * 0.125 = 2.705

**With a hidden archetype tag, M3 = 2.4-2.7 is achievable under Graduated
Realistic fitness, depending on M6 tolerance.** This matches or exceeds V8's
Narrative Gravity on the 40% enriched pool (M3 = 2.75) from a precision
standpoint. The key constraint shifts from precision to M6/M9 concentration.

### Per-archetype equity

Unlike visible pair-matching (where Flash/Ramp had lower precision due to low
fitness), archetype tags yield identical precision for every archetype: P = 1.0
for all 8. The equity gap collapses from 0.73 (Narrative Gravity) to
approximately 0.1-0.2 spread driven only by pool contraction dynamics.

### M11 (picks 15+) with archetype tag

At picks 15+, if the algorithm has been targeting home-archetype cards since
pick 6, the available pool is richer in home-archetype cards. With pool
contraction + archetype tag targeting, M11 = 3.0 is achievable:

```
3 home-tagged slots + 1 random: M3_late = 3.0 + 0.125 = 3.125
```

Without pool contraction, M11 follows the same precision as M3 (no late-draft
improvement). With contraction, the surviving pool increasingly favors home
cards even in "random" slots. **M11 >= 3.0 appears achievable with archetype
tags + pool contraction.**

**V1 metric estimate: ~25-35%.** The visible symbols (R1 primary) still
contribute: they provide the initial archetype inference for picks 1-5. But from
pick 6 onward, the hidden tags do the targeting work. The visible symbols are
meaningful early-draft signals that feel important to the player.

______________________________________________________________________

## 4. Level 2: Hidden Secondary Resonance Symbol

### Comparison to V8's visible 40% dual-res pool

V8's 40% enriched pool gave ~18 visible dual-res cards per archetype, enabling
pair-matched subpools of 18 cards. The precision:

```
P(S/A | visible pair filter) ≈ 0.87 (GR fitness, per pair)
```

A hidden secondary resonance is functionally equivalent to V8's visible dual-res
pool but invisible. The algorithm applies the same pair-matching logic (filter
for cards with primary + hidden secondary resonance = archetype pair), but the
player sees only single-symbol cards.

**If all 284 single-symbol cards receive a hidden secondary resonance, the
effective hidden-dual-res pool reaches 284 + 36 = 320 cards, giving ~40 pair-
matchable cards per archetype — equivalent to the 100% dual-res ceiling.**

This is strictly superior to V8's visible 40% pool in algorithmic terms:

| System                                               | Pair-matchable cards/archetype | Visible dual-res | Precision |
| ---------------------------------------------------- | :----------------------------: | :--------------: | :-------: |
| V8 visible 40% pool                                  |              ~18               | 132 cards (37%)  |   ~87%    |
| V9 hidden secondary (all single-symbol cards tagged) |              ~40               |  36 cards (10%)  |  ~87-90%  |
| V9 hidden secondary (home-archetype cards only)      |             ~30-35             |  36 cards (10%)  |   ~90%    |

**Hidden secondary resonance at full coverage matches or exceeds V8's visible
40% pool in pair-matching depth, with better subpool sustainability and equal or
slightly higher precision due to tighter home-archetype concentration.**

The precision advantage is modest (~3%) because V8's pair-filter already
achieved ~87% S/A. The sustainability advantage is significant: 40 vs 18
pair-matchable cards per archetype means 3 pair-matched slots/pack can be
sustained over 30 picks (requires 22.5 cards; 40 >> 22.5).

**M3 achievable with full hidden secondary resonance: ~2.7-3.0** (comparable to
V8 Narrative Gravity on 40% pool, with better depth). M11 >= 3.0 achievable with
3 pair-matched slots in the late draft.

**V1 metric estimate: ~40-50%.** The visible primary resonance still gates the
R1 pool (the player's visible (Tide) symbol narrows the field), and the hidden
secondary narrows further. The visible contribution is meaningful but the hidden
tag is doing substantial work.

______________________________________________________________________

## 5. Level 3: Archetype Affinity Scores (8 floats per card)

### Information content

8 floats per card = 8 * 32 bits = 256 bits per card (or ~8 * 8 = 64 bits at
reasonable precision). This is substantially more information than a 3-bit tag.

### Precision model

Where a tag assigns each card to one archetype with P = 1.0 for that archetype
and 0.0 for all others, affinity scores represent the real distribution of
archetype-playability. A card with archetype affinities:

```
Warriors: 0.9, Sacrifice: 0.6, Ramp: 0.2, Flash: 0.1, ...
```

...can be legitimately used for multiple archetypes with degrading confidence.

For targeting purposes, the algorithm draws from the top-K cards by affinity
score for the player's committed archetype. If scores are well-calibrated (they
reflect genuine mechanical fit), the top 30-40 cards by Warriors affinity will
have mean affinity of approximately 0.7-0.8, yielding S/A precision:

```
P(S/A | affinity score, Grad. Real.) ≈ 0.85-0.90
```

This is comparable to archetype tags for the home archetype, but with a key
difference: **affinity scores allow the algorithm to identify "good enough"
multi-archetype cards and include them in targeting.** A card that is A-tier in
both Warriors AND Sacrifice gets high affinity scores for both, whereas a tag
forces a single assignment.

### M3 ceiling with affinity scores

The M3 ceiling is bounded by the same pack-structure constraints as Level 1: 2-3
targeted slots with P ≈ 0.87-0.90 precision.

```
M3 = 2 * 0.88 + 1 * 0.65 (R1-boosted) + 1 * 0.125 = 2.63
M3 = 3 * 0.88 + 1 * 0.125 = 2.765 (risks M6)
```

The M3 ceiling with affinity scores is effectively the same as with archetype
tags (~2.6-2.8 at M6-safe configurations). The key difference is in **M11 and
late-draft quality**: affinity scores enable more precise late-draft targeting
as the algorithm has accumulated evidence about the player's sub-archetype
preferences.

**Pool contraction + affinity scores** is particularly powerful: the contraction
function weights cards by affinity score (not just binary home/not-home), giving
a smoother contraction that retains high-affinity multi-archetype cards longer.
This is equivalent to V8's Narrative Gravity but operating on a richer relevance
signal. **M3 = 2.7-2.9, M11 = 3.2-3.4 projected** for pool contraction with
affinity scores, conditional on M6 \<= 90%.

**V1 metric estimate: ~25-35%.** Affinity scores can be derived from visible
properties (mana cost, keywords, card type, visible resonance), making the
system "honest" in V9's sense. But the 8-float vector does substantially more
targeting work than the 1-bit visible symbol alone.

______________________________________________________________________

## 6. Minimum Hidden Information to Reach M3 = 2.0 and M11 = 3.0

### M3 = 2.0 under Graduated Realistic

From the Level 0 analysis, visible-only at 10% dual-res achieves M3 ≈ 2.0-2.1
with flash/ramp at the exact threshold. The failure mode is M10 (streaks 6-8)
and flash/ramp below 2.0. These failures cannot be fixed with visible symbols
alone.

**Minimum hidden information for reliable M3 >= 2.0 across all archetypes:** A
single-bit flag per card identifying home archetype membership (can be encoded
as 3-bit archetype tag applied only to home-archetype cards) plus R1 visible
targeting for the remaining slots.

More precisely: the algorithm needs to distinguish "this card is home-archetype
for Warriors" vs "this card is home-archetype for Ramp" for single-symbol Tide
and Zephyr cards respectively. A 3-bit hidden tag on ~80 cards (the 40 home
cards for each Tide archetype, differentiated by tag) provides this.

**Practical minimum: 3 bits per card (archetype tag).** Subpopulation tags
applied to single-symbol cards enable 1 reliable home-targeted slot per pack
beyond visible R1 filtering, lifting Flash/Ramp from ~2.0 (marginal) to ~2.2-2.3
(comfortable) with M10 improvement.

### M11 = 3.0 under Graduated Realistic

M11 (picks 15+, target >= 3.0) requires the late-draft to deliver 3 S/A cards
per 4-card pack. This means 3 targeted slots with near-perfect precision, or 2
targeted slots with P ≈ 1.0 and 1 bonus random slot from an already-concentrated
pool.

**From visible symbols alone:** Impossible. At pick 15 on a 10% visible dual-res
pool, the player has drafted ~15 cards, and the visible signal is clear. But the
random slots still draw from the full uncontracted pool at ~12.5% base rate.
Visible symbols cannot improve random slot quality.

**With 3-bit archetype tag + pool contraction:** The contraction mechanism,
using hidden tags rather than visible symbols for the relevance score,
concentrates the pool dramatically by pick 15. The surviving pool contains
~60-80 cards, heavily weighted toward the committed archetype. Random slots in
this pool yield P ≈ 0.5-0.6 S/A (vs 0.125 from the full pool). With 2 tagged + 2
contraction-boosted random:

```
M11 ≈ 2 * 1.0 + 2 * 0.55 = 3.10
```

**Conclusion: M11 = 3.0 requires pool contraction with hidden archetype tags OR
hidden secondary resonance. Pool contraction on visible symbols alone (V8
Narrative Gravity with 10% visible dual-res) produced M3 = 2.38 but Flash at
1.47 — the visible signal is too weak to drive sufficient contraction
precision.**

**Minimum hidden information for M11 >= 3.0: 3-bit archetype tag + pool
contraction algorithm using tags for the relevance score.** This is precisely
V8's Narrative Gravity with hidden tags substituting for visible dual-res
symbols in the relevance computation.

______________________________________________________________________

## 7. V1 Metric Across Information Levels

The V1 metric measures what fraction of the algorithm's targeting power comes
from visible symbols alone (run without hidden metadata, compare M3 delta).

| Level | Hidden info                | Visible-only M3 | Full M3  | Delta | V1 estimate |
| ----- | -------------------------- | :-------------: | :------: | :---: | :---------: |
| 0     | None                       |      ~2.05      |   2.05   |   0   |    100%     |
| 1     | 3-bit archetype tag        |      ~2.05      | ~2.5-2.7 | +0.5  |   ~40-50%   |
| 2     | Hidden secondary resonance |      ~2.05      | ~2.7-3.0 | +0.7  |   ~35-45%   |
| 3     | 8-float affinity scores    |      ~2.05      | ~2.7-2.9 | +0.7  |   ~30-40%   |

Interpretation: the visible symbols' contribution is essentially constant at
each level (~2.05 M3 without hidden data). The hidden metadata adds on top of
this baseline. V1 is the visible-only M3 divided by full M3.

**V1 ≈ 50% is achievable at Level 1 (3-bit tag) for M3 = 2.4-2.5.** For M3

> = 2.7, V1 drops to ~35-45%, meaning hidden metadata is doing more than half
> the targeting work. The V9 goal of V1 >= 50% alongside M3 >= 2.0 is achievable
> at Level 1.

______________________________________________________________________

## 8. Per-Archetype M3 Floor by Information Level

The V8 per-archetype data (Flash/Ramp as worst cases under GR) allows
projection:

| Archetype          | Level 0  | Level 1 (3-bit tag) | Level 2 (hidden 2nd res) | Level 3 (affinity) |
| ------------------ | :------: | :-----------------: | :----------------------: | :----------------: |
| Warriors (50%)     |   2.05   |        2.45         |           2.70           |        2.75        |
| Sacrifice (50%)    |   2.05   |        2.45         |           2.70           |        2.75        |
| Self-Mill (40%)    |   2.02   |        2.42         |           2.65           |        2.70        |
| Self-Discard (40%) |   2.02   |        2.42         |           2.65           |        2.70        |
| Storm (30%)        |   2.00   |        2.38         |           2.55           |        2.60        |
| Blink (30%)        |   2.00   |        2.38         |           2.55           |        2.60        |
| Flash (25%)        |   1.97   |        2.35         |           2.50           |        2.55        |
| Ramp (25%)         |   1.97   |        2.35         |           2.50           |        2.55        |
| **Worst**          | **1.97** |      **2.35**       |         **2.50**         |      **2.55**      |
| **Spread**         | **0.08** |      **0.10**       |         **0.20**         |      **0.20**      |

Note: The spread is small at Level 1 because the hidden tag delivers identical
precision (P=1.0) for all archetypes regardless of sibling fitness. The equity
advantage of hidden archetype tags over visible pair-matching is substantial.

______________________________________________________________________

## 9. Key Conclusions

**1. The visible-only ceiling is M3 ≈ 2.05, M11 ≈ 2.1** at 10% visible dual-res
under Graduated Realistic fitness. This is just barely above the M3 target but
with no headroom, poor M10, and M11 well below 3.0.

**2. A 3-bit archetype tag is the minimum unit of hidden metadata that
meaningfully changes the system.** It lifts M3 to ~2.4-2.5, eliminates M10
failures, and — combined with pool contraction — reaches M11 ≈ 3.0-3.1. The
information cost is 3 bits per card = 360 * 3 = 1080 bits total for the pool.

**3. Hidden secondary resonance matches V8's visible 40% pool in precision** but
is richer in subpool depth. The M3 ceiling is ~2.7-3.0 depending on contraction
parameters. The information cost is higher than a 3-bit tag (requires assigning
meaningful pair symbols to 284 single-symbol cards) but lower than affinity
scores.

**4. Affinity scores provide marginal M3 gains over a 3-bit tag** (~0.2-0.3 M3)
while substantially improving the system's design integrity (scores reflect real
mechanical fit). The M3 ceiling at Level 3 is ~2.7-2.9, similar to Level 2.

**5. The minimum hidden metadata path to M3 >= 2.0 and M11 >= 3.0 is a 3-bit
archetype tag with pool contraction.** This matches V8's Narrative Gravity
mechanism (hidden tag replaces visible dual-res in the relevance score),
requires 3 bits per card, and keeps V1 >= 40-50%.

**6. The M3-M11 gap requires pool contraction regardless of information level.**
No slot-filling algorithm achieves M11 >= 3.0 without late-draft pool
concentration. Pool contraction is the mechanism that converts early information
(archetype commitment) into late-draft quality. Hidden metadata expands what the
contraction algorithm can use as a relevance signal.
