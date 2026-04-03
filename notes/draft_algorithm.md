# Dreamtides Draft Algorithm: Tide Current

## Player-Facing Explanation

"As you draft, the current shifts to match your recent picks -- cards from your
chosen tides appear more often, and distant tides fade away."

## Overview

The draft shows 4 cards at a time. The player picks 1 card to add to their deck;
all 4 cards leave the pool permanently. Two mechanisms shape which cards appear:

1. **Initial Tide Exclusion** -- at quest start, entire tides are removed from
   the pool
2. **Tide Current** -- each pack is drawn with weights that increasingly favor
   the player's recent draft choices

## Mechanism 1: Initial Tide Exclusion

At the start of each quest, **N** random core tides are permanently removed from
the draft pool. Neutral is never removed.

**Default N = 2.** This is configurable via a flag and can range from 0 to 4.

With N=2, there are C(7,2) = 21 possible quest configurations, providing strong
variance between runs. Five core tides plus Neutral remain, which is enough for
2-3 viable tide alliances per quest.

Any combination of tides may be removed (including adjacent tides).

### Why N=2

- **N=0**: Every quest has the same card pool. Low variance. Convergence is too
  slow -- dominant tide only reaches 0.72 at pick 5 and 1.62 at pick 10 because
  7 competing tides dilute the weights.
- **N=1**: Convergence still undershoots targets by ~1-2 picks (0.82 at pick 5,
  1.85 at pick 10). Quests feel similar with only 7 configurations.
- **N=2**: ~142 cards removed. Hits convergence targets precisely (0.97/2.10/3.12
  at picks 5/10/15). 21 quest configurations with 3-5 viable alliances each.
- **N=3**: Over-converges (1.23 at pick 5, 2.46 at pick 10). Pool depletion
  causes regression at pick 25. Only 2-3 alliances per quest.
- **N=4**: Extreme. Only 3 core tides remain; pool physically runs out of
  dominant-tide cards by pick 25 (dominant drops from 3.68 to 2.75). Only 1-2
  alliances available.

These results were validated via Monte Carlo sweep across N=0..4 (5,000 trials
each, simulation at `scripts/draft_simulation/sweep_exclusion.py`). The algorithm
parameters (focus_rate, decay_factor) are tuned for N=2; other N values would
require recalibration.

### Adapting to N=0

If playtesting shows N=0 is more fun (all tides always available), the algorithm
can be recalibrated. Six parameter variants were tested for N=0 (simulation at
`scripts/draft_simulation/sweep_n0.py`). The best approach is increasing
focus_rate to ~0.45-0.50 and optionally starting focus 1 pick earlier. This gets
mono-tide convergence close to targets (0.94/2.29/3.21 at picks 5/10/15 with
focus_rate=0.45 and focus_start=2).

However, N=0 has a fundamental tension: **convergence and pivot viability trade
off sharply.** With 7 competing tides, increasing focus_rate to hit mono-tide
targets crushes pivoting -- post-pivot dominant reaches only 0.51-0.71 at pick
20 across all N=0 variants tested, compared to 1.78 at N=2. The extra tides
create too much affinity inertia for the decay to overcome. Tightening similarity
values (distance-2 from 0.15 to 0.05, distance-3 from 0.05 to 0.0) does not
help; it improves mono-tide convergence slightly but makes pivoting even harder
by widening the affinity gap the new tide must close.

If N=0 is chosen, accept that pivoting will be a significantly weaker strategic
option, or consider a more aggressive decay factor (e.g. 0.75 instead of 0.85)
to allow faster affinity turnover at the cost of less stable convergence for
committed players.

## Mechanism 2: Tide Current (Weighted Sampling)

When generating each pack of 4, every card in the pool is assigned a sampling
weight. Cards are drawn without replacement (within a single pack) proportional
to weight.

### Step 1: Compute Tide Affinity

For each tide `t`, compute an affinity score based on the player's drafted cards,
with a **recency decay** so that recent picks influence the pool more than older
ones:

```
affinity[t] = BASE_AFFINITY + sum over all drafted cards c (newest first) of:
    circle_similarity(t, tide(c)) * DECAY_FACTOR ^ (position of c from newest)
```

The newest drafted card has position 0 (full weight), the second-newest has
position 1 (multiplied by 0.85), the third-newest position 2 (multiplied by
0.85^2 = 0.72), and so on.

**BASE_AFFINITY = 1.0** -- ensures every tide has nonzero weight, even those the
player hasn't touched.

**DECAY_FACTOR = 0.85** -- a card drafted 5 picks ago retains ~44% of its
influence. This allows pivoting: if a player switches tides, the old tide's
influence fades within several picks rather than persisting forever.

**circle_similarity** by distance on the tide circle
(Bloom-Arc-Ignite-Pact-Umbra-Rime-Surge-Bloom):

| Distance   | Similarity |
|------------|------------|
| 0 (same)   | 1.0        |
| 1 (allied) | 0.5        |
| 2           | 0.15       |
| 3 (far)    | 0.05       |

**Neutral rules:**

- When a player drafts a Neutral card, it contributes
  `NEUTRAL_DRAFT_CONTRIBUTION (0.4) * decay` affinity to ALL core tides (it
  gently reinforces whatever the player is already doing, subject to the same
  recency decay as other cards).
- Neutral's own affinity = `max(BASE_AFFINITY + count_of_neutral_drafted,
  NEUTRAL_AFFINITY_FACTOR * max_core_affinity)`. This ensures Neutral cards
  remain available both for players who draft Neutral frequently (first term
  dominates) and for mono-tide players who never draft Neutral (second term
  tracks their strongest tide at half strength).

### Step 2: Compute Focus

Focus controls how aggressively the weights separate high-affinity tides from
low-affinity ones. It starts at 0 (uniform random) and increases linearly with
pick number.

```
focus = max(0, (pick_number - 2) * FOCUS_RATE)
```

**FOCUS_RATE = 0.35**

| Pick | Focus | Effect                        |
|------|-------|-------------------------------|
| 1-2  | 0.00  | Fully random                  |
| 3    | 0.35  | Barely perceptible bias       |
| 5    | 1.05  | Noticeable tilt               |
| 8    | 2.10  | Strong preference             |
| 10   | 2.80  | Dominant tide ~50% of packs   |
| 15   | 4.55  | Dominant tide ~75% of packs   |
| 20   | 6.30  | Stabilized, very concentrated |

### Step 3: Compute Card Weights

For each card in the pool:

```
weight = affinity[card.tide] ^ focus
```

When focus = 0, every weight is 1 (uniform distribution). As focus increases,
high-affinity tides are exponentially amplified relative to low-affinity tides.

### Step 4: Draw the Pack

Sample 4 cards from the pool without replacement, probability proportional to
weight. Present them to the player. The player picks 1; all 4 cards leave the
pool permanently (the 3 unpicked cards are discarded).

## Expected Behavior

The following tables are derived from Monte Carlo simulation against the real
582-card pool (5,000+ trials per scenario). The simulation code is at
`scripts/draft_simulation/draft_simulation.py`.

### Mono-Tide Player (consistently picks one tide)

| Pick | Dominant | Allied | Neutral | Distant | Pool | P(>=1) | P(>=2) | P(>=3) |
|------|----------|--------|---------|---------|------|--------|--------|--------|
| 1    | 0.67     | 0.86   | 0.71    | 1.76    | 437  | 0.53   | 0.13   | 0.02   |
| 5    | 1.00     | 0.99   | 0.55    | 1.46    | 421  | 0.67   | 0.26   | 0.06   |
| 10   | 2.15     | 0.96   | 0.37    | 0.53    | 382  | 0.93   | 0.73   | 0.38   |
| 15   | 3.14     | 0.56   | 0.20    | 0.10    | 310  | 0.98   | 0.95   | 0.80   |
| 20   | 3.53     | 0.31   | 0.11    | 0.05    | 236  | 0.99   | 0.98   | 0.93   |
| 25   | 3.54     | 0.29   | 0.13    | 0.04    | 206  | 0.98   | 0.97   | 0.93   |

Hits the convergence targets: ~1 at pick 5, ~2 at pick 10, ~3 at pick 15. The
system stabilizes around 3.5/4 in the late draft, leaving consistent room for
allied/neutral splash cards.

### Pivot Scenario (switch tide at pick 8)

Player drafts tide A for picks 1-8, then switches to a distant tide B. Dominant
column tracks the post-pivot tide B:

| Pick | Dominant | Allied | Neutral | Distant | Pool | P(>=1) | P(>=2) | P(>=3) |
|------|----------|--------|---------|---------|------|--------|--------|--------|
| 5    | 0.61     | 0.83   | 0.56    | 2.00    | 421  | 0.49   | 0.11   | 0.01   |
| 10   | 0.50     | 0.72   | 0.35    | 2.43    | 384  | 0.34   | 0.12   | 0.03   |
| 12   | 0.74     | 0.71   | 0.29    | 2.26    | 364  | 0.41   | 0.22   | 0.10   |
| 15   | 1.19     | 0.59   | 0.22    | 2.00    | 326  | 0.48   | 0.38   | 0.25   |
| 18   | 1.58     | 0.43   | 0.15    | 1.84    | 286  | 0.51   | 0.46   | 0.39   |
| 20   | 1.74     | 0.34   | 0.12    | 1.80    | 263  | 0.52   | 0.50   | 0.45   |
| 25   | 1.93     | 0.22   | 0.15    | 1.70    | 227  | 0.53   | 0.52   | 0.50   |

Pivoting is costly but viable. The new tide reaches ~1.2 cards per pack by pick
15 (7 picks after the switch) and approaches ~1.9 by pick 25. Without the
recency decay, these numbers are significantly worse (0.84 at pick 15, 1.19 at
pick 20) -- the old tide's affinity dominates indefinitely.

## Tide Circle Distance Reference

```
        Bloom
       /     \
    Surge     Arc
      |         |
    Rime     Ignite
       \     /
        Umbra --- Pact
```

Distances (shortest path on the circle):

|        | Bloom | Arc | Ignite | Pact | Umbra | Rime | Surge |
|--------|-------|-----|--------|------|-------|------|-------|
| Bloom  | 0     | 1   | 2      | 3    | 3     | 2    | 1     |
| Arc    | 1     | 0   | 1      | 2    | 3     | 3    | 2     |
| Ignite | 2     | 1   | 0      | 1    | 2     | 3    | 3     |
| Pact   | 3     | 2   | 1      | 0    | 1     | 2    | 3     |
| Umbra  | 3     | 3   | 2      | 1    | 0     | 1    | 2     |
| Rime   | 2     | 3   | 3      | 2    | 1     | 0    | 1     |
| Surge  | 1     | 2   | 3      | 3    | 2     | 1    | 0     |

## Required Card Metadata

Each card needs only its **tide** field (already present in rendered-cards.toml)
for this system to work. No new card metadata is required. The algorithm derives
everything from the tide assignment and the tide circle distance table.

## Card Pool Statistics

~582 total cards across 8 tides (including Neutral), roughly evenly distributed:

| Tide    | Cards |
|---------|-------|
| Surge   | 76    |
| Pact    | 74    |
| Rime    | 73    |
| Neutral | 73    |
| Umbra   | 72    |
| Bloom   | 72    |
| Ignite  | 71    |
| Arc     | 71    |

After removing 2 tides (default), the pool contains ~430 cards across 6 tides.

## Parameters

All parameters are configured in TOML.

| Parameter                    | Default | Description                                          |
|------------------------------|---------|------------------------------------------------------|
| `initial_tide_exclusion`     | 2       | Number of core tides removed at quest start          |
| `base_affinity`              | 1.0     | Minimum affinity for any tide                        |
| `focus_start_pick`           | 3       | First pick where focus > 0                           |
| `focus_rate`                 | 0.35    | Focus increase per pick after focus_start_pick       |
| `decay_factor`               | 0.85    | Recency decay per pick position (1.0 = no decay)    |
| `ally_similarity`            | 0.5     | Affinity contribution from allied-tide drafted cards |
| `distance_2_similarity`      | 0.15    | Affinity contribution from distance-2 drafted cards  |
| `distance_3_similarity`      | 0.05    | Affinity contribution from distance-3 drafted cards  |
| `neutral_draft_contribution` | 0.4     | Affinity added to all tides per Neutral card drafted |
| `neutral_affinity_factor`    | 0.5     | Neutral's affinity as fraction of highest core tide  |
| `pack_size`                  | 4       | Number of cards shown per pick                       |

## Design Rationale

### Why weighted sampling (not pure physical removal)?

The player sees the same effect -- dominant tides appear more, distant tides
disappear -- but the implementation is smoother and more forgiving:

- **Tunable**: adjusting focus_rate changes the concentration curve without
  risking pool exhaustion or edge cases.
- **Pivot-friendly**: weights shift dynamically with the player's choices. A
  pivot at pick 8 is viable because the weights reflect the new direction within
  several picks. Pure removal would have permanently destroyed those cards.
- **No edge cases**: with pure removal, aggressive trimming can accidentally
  eliminate a tide the player wants to splash. Weighted sampling always keeps a
  nonzero (if tiny) chance.

### Why exponential (power) weighting?

Using `affinity ^ focus` has a natural property: when focus is low, differences
between tides are compressed (everything looks similar). When focus is high,
small affinity differences become large weight differences. This creates the
desired "open early, focused late" curve without needing separate logic for
different draft phases.

### Why recency decay?

Without decay, every drafted card contributes equally to affinity forever. This
means a player who drafts 8 cards of tide A and then wants to pivot to tide B
is permanently weighed down by the accumulated tide-A affinity. With decay
factor 0.85, a card from 5 picks ago has ~44% influence and a card from 10 picks
ago has ~20%. The old tide fades naturally as the player commits to the new one.

Monte Carlo simulation confirmed this is the single most impactful algorithmic
change for pivot viability: at pick 15 (7 picks after a pivot), the new tide
averages 1.19 cards per pack with decay vs 0.84 without. This difference
compounds -- by pick 20 it's 1.74 vs 1.19.

Decay does not hurt mono-tide convergence because a committed player keeps
drafting the same tide, continuously refreshing its affinity. The system hits
1.0/2.15/3.14 dominant cards at picks 5/10/15 with or without decay.

### Why no pool trimming?

An earlier version of this algorithm included a "pool trimming" mechanism that
physically removed low-affinity cards from the pool after each pick. Monte Carlo
simulation showed this has zero measurable impact on any metric -- the weighted
sampling alone produces identical convergence curves, pivot behavior, and pool
diversity. The natural pool shrinkage from 4 cards leaving per pick (~60 cards
gone by pick 15, ~100 by pick 25) provides all the physical narrowing needed.
Pool trimming was removed to keep the algorithm simple.

### Why track per-card similarity (not just count the dominant tide)?

Counting only the most-drafted tide would create hard category boundaries: "you
drafted 5 Bloom and 4 Arc, so you're a Bloom player." The similarity-sum
approach produces soft, continuous affinity that naturally handles:

- Two-tide decks (both tides contribute to each other's affinity via ally bonus)
- Gradual pivots (new tide builds affinity while old tide's stops growing)
- Neutral-heavy decks (Neutral contributes to all tides, keeping options open)

## Simulation Evidence

Parameters were validated via Monte Carlo simulation against the real 582-card
pool. The simulation code is at `scripts/draft_simulation/draft_simulation.py`.

### Variants tested

Eight algorithmic variants were compared on mono-tide convergence and pivot
recovery (5,000 mono-tide trials, 2,000 pivot trials each):

| Variant                     | P5 Dom | P10 Dom | P15 Dom | P15 Pivot | P20 Pivot |
|-----------------------------|--------|---------|---------|-----------|-----------|
| Baseline (focus=0.35)       | 1.03   | 2.16    | 3.10    | 0.84      | 1.19      |
| Focus cap at 3.5            | 0.98   | 2.18    | 2.68    | 0.72      | 0.93      |
| Sqrt affinity               | 0.93   | 1.61    | 2.37    | 0.92      | 1.30      |
| Sqrt + focus 0.50           | 1.07   | 2.12    | 2.98    | 0.90      | 1.28      |
| **Decay 0.85 (chosen)**     | 1.00   | 2.15    | 3.14    | 1.19      | 1.74      |
| Decay 0.90 + focus 0.40     | 1.07   | 2.39    | 3.34    | 1.11      | 1.52      |
| Guaranteed off-tide slot    | 0.82   | 1.79    | 2.44    | 0.51      | 0.69      |
| No trimming                 | 1.00   | 2.13    | 3.09    | 0.83      | 1.21      |

### Key findings

- **Pool trimming has no effect.** Baseline and no-trimming variants are
  statistically identical. Removed from the algorithm.
- **Decay 0.85 is the best pivot fix** without hurting mono-tide convergence.
  It's the only variant that reaches 1.19 dominant at pick 15 post-pivot (vs
  0.84 baseline). The improvement comes entirely from the old tide's influence
  fading.
- **Focus cap causes regression.** After the cap point, the pool shrinks
  physically but the weights stop concentrating, causing dominant tide frequency
  to drop in the late draft.
- **Sqrt affinity converges too slowly** at the default focus rate. Compensating
  with a higher rate (0.50) works but adds complexity for marginal benefit over
  decay.
- **Guaranteed off-tide slot kills convergence.** Forcing 1 of 4 cards to be
  non-dominant makes it impossible to reach the 3/4 target.

### Neutral affinity fix

The original Neutral formula (`0.5 * max_core_affinity`) caused Neutral cards
to be suppressed for all-Neutral drafters. The revised formula
(`max(base + neutral_count, 0.5 * max_core)`) was validated to work correctly
for both mono-tide players (Neutral tracks dominant tide at half strength) and
Neutral-focused drafters (Neutral affinity grows with count of Neutral drafted).

## Discussion: Richer Card Metadata

The current algorithm uses only a card's `tide` field and the fixed tide circle
distance table. An alternative approach would give each card a numerical
"archetype fitness" score for each of the 7 core archetypes -- e.g., a Bloom
spirit animal with a Materialized trigger might score Bloom=0.9, Arc=0.7 (good
blink target), Ignite=0.1, and so on. What would this buy?

### What richer metadata improves

**Cross-tide synergy cards.** The biggest win. The current system treats all
cards within a tide identically -- a Bloom ramp card and a Bloom voltron card get
the same weight for a Bloom-Arc player. In reality, the ramp card might be
irrelevant to Arc's blink plan while the voltron card with a Materialized trigger
is exactly what Arc wants. Archetype fitness scores would let the algorithm show
the Bloom cards that specifically synergize with Arc when a Bloom-Arc player is
drafting.

**Smarter splash cards.** When a committed Bloom player sees 3 Bloom cards and 1
splash card, the quality of that splash matters. Today, the splash is just "a
card from an allied tide." With fitness scores, the splash could be specifically
a card that works well in Bloom decks despite being from another tide -- an Arc
card with energy-related triggers, for example, rather than a random Arc tempo
card that does nothing for the Bloom plan.

**Differentiated Neutral cards.** Currently all Neutral cards are weighted the
same. But "draw a card" is truly generic while "Discover a character" is better
in creature-heavy tides. Fitness scores would let the algorithm surface relevant
Neutral cards for the player's specific strategy.

**Within-tide differentiation.** A tide like Bloom has multiple sub-strategies
(ramp, voltron, stompy). A player drafting voltron Bloom cards would see more
voltron Bloom cards and fewer ramp Bloom cards. This would make late-draft packs
feel more curated and coherent.

### What richer metadata costs

**Annotation burden.** ~570 cards x 7 archetype scores = ~4,000 values to assign
and maintain. This is a large upfront effort and an ongoing maintenance cost
whenever cards are added, changed, or rebalanced. Even with tooling, this is
probably 2-3 days of careful design work, plus ongoing upkeep.

**Subjectivity.** "How well does this card fit the Arc blink archetype?" is a
judgment call. Scores inevitably reflect the designer's current understanding of
the meta, which may not match how players actually use cards. A card scored 0.2
for Pact might turn out to be a Pact staple in practice.

**Staleness.** Fitness scores are a snapshot of design intent. As the card pool
evolves, scores silently drift out of date. The tide field rarely changes; a
7-dimensional fitness vector is much more fragile.

**Overfitting.** Detailed fitness scores can make the draft feel "on rails" at a
sub-tide level. If the algorithm knows a card is good in voltron-Bloom but not
ramp-Bloom, it starts making deck composition decisions for the player. Part of
the fun of drafting is discovering unexpected synergies -- the algorithm shouldn't
pre-chew that discovery.

**Complexity.** The current system is explainable in one sentence. Adding
per-card fitness scores turns it into a recommendation engine that's harder to
reason about, harder to debug, and harder to explain to players.

### How much better would it actually be?

Honestly: marginal improvement for significant cost. Here's why:

The current system's "error" is that it uses a single blanket similarity value
for all cards within a tide distance (all allied-tide cards get 0.5). The cases
where this is meaningfully wrong are real but limited:

- A card from a distant tide that happens to synergize with the player's
  archetype gets underweighted. But these are rare edge cases -- the tide circle
  was designed so that nearby tides synergize and distant tides don't.
- Within a tide, all cards get the same weight regardless of sub-archetype. But
  the player handles this themselves -- they see 2-3 cards from their tide per
  pack and pick the one that fits their deck. The algorithm doesn't need to
  pre-filter within a tide.

The place it matters most is the splash slot (the ~1 non-dominant card per pack).
Making that single card more relevant is a real improvement, but it's a marginal
one -- the player is already getting 3 good cards and can ignore the splash if it
doesn't fit.

### Middle ground: secondary tide

A lighter alternative to full archetype scores: add an optional `secondary_tide`
field to cards that meaningfully serve two archetypes. A Bloom spirit animal with
a strong Materialized trigger could be tagged `tide = "Bloom"`,
`secondary_tide = "Arc"`. The algorithm would then count this card as partially
belonging to both tides when computing affinity.

This captures the most important cross-tide cases (~50-100 cards would merit a
secondary tide) with a fraction of the annotation burden, no subjectivity about
numerical scores, and easy maintenance. It also doesn't require changes to the
core algorithm -- secondary_tide cards simply contribute similarity to an
additional tide when drafted.

### Recommendation

Start with tide-only metadata. It's simple, already available, and the tide
circle distance provides a reasonable approximation of cross-tide synergy. If
playtesting reveals that splash cards feel irrelevant or that the draft lacks
"smart" cross-archetype suggestions, the secondary_tide approach is a low-cost
improvement that can be added incrementally without changing the algorithm.

Full per-archetype fitness scores should be reserved for a scenario where the
game has shipped, the card pool is stable, and there is clear evidence that the
simpler system produces unsatisfying drafts.
