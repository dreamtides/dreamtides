# Research Agent A: Pool Composition Space

## 1. Baseline Pool Structure and the Precision Problem

The V7 baseline pool contains 360 cards: ~40 per archetype (320 total) plus 36
generic cards with no resonance symbols. Each resonance is primary for exactly 2
archetypes. When an algorithm filters by "primary resonance = Tide," it draws
from a pool of ~80 cards belonging to Sacrifice and Warriors. For a Warriors
player, 50% of that R1 pool is home-archetype (always S-tier) and 50% is
sibling-archetype (fitness-dependent).

This gives us the **precision equation** that governs everything:

```
P(S/A | R1 filter) = 0.5 + 0.5 * F
```

where F = sibling A-tier rate. At F=1.0 (Optimistic), P=100%. At F=0.5
(Moderate), P=75%. At F=0.25 (Pessimistic), P=62.5%. At F=0 (Hostile), P=50%.

The M3 = 2.0 target requires:

```
N * P + (4-N) * 0.25 = 2.0
```

where N = targeted slots per pack (4-card packs, baseline S/A of random card =
0.25 per archetype). Solving for N:

| Fitness (F) | Precision (P) | Slots needed for M3=2.0 | Feasible at 4 cards?  |
| :---------: | :-----------: | :---------------------: | :-------------------: |
|    100%     |     100%      |          2.33           | Yes (3 slots surplus) |
|     50%     |      75%      |          3.50           | No (need fractional)  |
|     25%     |     62.5%     |          4.67           |   No (exceeds pack)   |
|     15%     |     57.5%     |          5.38           |          No           |
|     0%      |      50%      |          7.00           |          No           |

Under Pessimistic fitness (F=0.25), even filling all 4 slots with R1-filtered
cards yields only 4 * 0.625 = 2.50 S/A -- achievable but with zero splash (M4
fails). Under Moderate fitness, 3.5 targeted slots are needed, meaning surge
packs (3 slots) and floor packs (1 slot) must average to 3.5, which is
impossible with the current alternation pattern.

**This is the core mathematical obstacle.** R1-level filtering cannot reach 2.0
under Moderate fitness with 4-card packs. Solving it requires either increasing
precision above 75% (better filtering), increasing pack size, or some
combination.

## 2. Raising the Dual-Resonance Cap

The V7 baseline caps dual-resonance cards (cards bearing symbols of 2 different
resonance types) at 15% of the pool (54 cards). Dual-resonance cards are the key
to pair-matching algorithms: a card with symbols (Tide, Zephyr) can be
identified as a Warriors card with ~80% S-tier precision (V5 finding), versus
~50% for single-resonance "Tide" filtering.

### Pool composition at different dual-resonance caps

Assume 360 cards, 36 generic (0 symbols), 324 resonance-bearing cards.

|   Dual-res cap   | Dual-res cards | Single-res cards | % pool dual | Pair-matchable pool per archetype |
| :--------------: | :------------: | :--------------: | :---------: | :-------------------------------: |
|     15% (V7)     |       54       |       270        |     15%     |             ~7 cards              |
|       25%        |       90       |       234        |     25%     |             ~11 cards             |
|       40%        |      144       |       180        |     40%     |             ~18 cards             |
| 60% (V5 assumed) |      194       |       130        |     54%     |             ~24 cards             |
|       100%       |      324       |        0         |     90%     |             ~40 cards             |

The "pair-matchable pool per archetype" column assumes even distribution across
the 8 possible ordered pairs (each of 4 resonances with each of 3 others as
secondary, but only 8 archetype-aligned pairs matter). With 8 archetypes and
even distribution, each archetype gets ~(dual-res cards / 8) pair-matched cards
(some pairs span the same archetype pair, counted once).

**Impact on pair-matching precision:**

V5's Pair-Escalation achieved ~80% S-tier precision per targeted slot by
filtering on ordered pairs. But this requires a sufficiently large pair-matched
subpool. At 15% dual-res (V7), only ~7 cards per archetype pair exist -- too few
for reliable drawing without replacement exhaustion. At 60% (V5's implicit
assumption), ~24 cards exist per pair, ample for 2-3 pair-matched slots per pack
across a 30-pick draft.

The critical threshold is around **40% dual-resonance** (~18 cards per pair).
Below this, pair-matching algorithms suffer pool exhaustion: drawing 2-3
pair-matched cards per pack for 20+ packs requires a pool of at least 15-20
unique pair-matched cards to avoid excessive repetition (player seeing the same
cards repeatedly). At 25%, the pool is marginal. At 15%, pair-matching is
structurally non-viable at high slot counts.

### What dual-resonance costs the card designer

At 15%, the designer creates ~7 dual-resonance cards per archetype pair. These
are "bridge" cards designed to work in both archetypes of a pair (e.g., Warriors
and Ramp both using Tide+Zephyr). This is modest.

At 40%, the designer creates ~18 dual-resonance cards per pair. Since each
archetype has ~40 cards total, this means ~45% of each archetype's cards carry a
secondary resonance symbol. The designer must ensure these cards are
thematically coherent with both the primary and secondary resonance identities.

At 60%, roughly 60% of all resonance-bearing cards are dual-resonance. The
"default" card carries two resonance symbols. Single-resonance cards become the
minority -- reserved for archetype-specific effects.

At 100%, every card carries at least two resonance types. The resonance system
becomes fully a pair-identification system rather than a faction marker. This
fundamentally changes what resonance symbols mean: they are no longer "this card
belongs to the Tide faction" but "this card's archetype signature is (Tide,
Zephyr)."

## 3. Richer Symbol Information: 3 Symbols Per Card

What if every resonance-bearing card carried exactly 3 symbols instead of the V7
baseline of 0-3?

### Symbol repetition disallowed (3 distinct symbols)

With only 4 resonance types and 3 required distinct symbols, each card omits
exactly 1 resonance. This means a card carries {Ember, Stone, Tide} or {Ember,
Stone, Zephyr} or {Ember, Tide, Zephyr} or {Stone, Tide, Zephyr}. There are
exactly 4 possible symbol sets, each identified by the **missing** resonance.

This is equivalent to a single-symbol system where the symbol is the *absent*
resonance. Filtering by "has Tide" returns 75% of the pool (all cards except
those missing Tide). This is strictly worse than single-resonance filtering for
archetype identification -- the subpool is larger and less specific.

However, ordered triples (primary, secondary, tertiary) carry more information.
The first two symbols identify the archetype pair (as in V5), and the third
disambiguates further. With 4 resonances and ordered triples of 3 distinct
symbols, there are 4 * 3 * 2 = 24 possible ordered triples. Each archetype pair
maps to 2 triples (e.g., Warriors = (Tide, Zephyr, Ember) or (Tide, Zephyr,
Stone)). Filtering on the full ordered triple narrows to ~(324/24) = ~14 cards,
roughly half an archetype's pool.

**Archetype identification precision of ordered triples:**

- Filter by (R1, R2): ~80% S-tier (same as V5 pair matching)
- Filter by (R1, R2, R3): ~90-95% S-tier (the third symbol may distinguish
  sub-themes within an archetype)

The third symbol provides diminishing returns. The pair already identifies the
archetype with ~80% precision. The third symbol can help distinguish
sub-strategies but at the cost of a much smaller candidate pool.

### Symbol repetition allowed (e.g., Tide Tide Ember)

Repetition unlocks a new dimension: **symbol weight encoding**. A card with
(Tide, Tide, Ember) signals "strongly Tide, lightly Ember," while (Tide, Ember,
Ember) signals the reverse. This encoding communicates archetype affinity
without requiring explicit archetype labels.

With 4 resonances and 3 ordered positions allowing repetition, there are 4^3 =
64 possible symbol sequences. But many are equivalent for archetype
identification. The key patterns:

| Pattern | Example            | Archetype signal                 | Count |
| ------- | ------------------ | -------------------------------- | :---: |
| AAB     | Tide Tide Zephyr   | Strong Warriors                  |  12   |
| ABA     | Tide Zephyr Tide   | Warriors (Tide-leaning)          |  12   |
| ABB     | Tide Zephyr Zephyr | Warriors (Zephyr-leaning)        |  12   |
| ABC     | Tide Zephyr Ember  | Warriors + splash                |  24   |
| AAA     | Tide Tide Tide     | Pure Tide (both Tide archetypes) |   4   |

The AAB pattern is most interesting for algorithms. If a card's first two
symbols match the archetype's (primary, primary) -- e.g., (Tide, Tide, \*) -- it
strongly signals one of the two Tide-primary archetypes. Combined with the third
symbol, (Tide, Tide, Zephyr) uniquely identifies Warriors while (Tide, Tide,
Stone) uniquely identifies Sacrifice.

**Precision of repetition-based filtering:**

| Filter                               | Subpool size (approx) |        S-tier precision        |
| ------------------------------------ | :-------------------: | :----------------------------: |
| R1 only (Tide)                       |       ~80 cards       |              50%               |
| R1 pair (Tide, Zephyr)               |       ~24 cards       |              ~80%              |
| R1 doubled (Tide, Tide, \*)          |       ~16 cards       |  ~50% (both Tide archetypes)   |
| R1 doubled + R3 (Tide, Tide, Zephyr) |       ~8 cards        | ~95% (near-unique to Warriors) |
| Full triple (Tide, Zephyr, Ember)    |      ~5-8 cards       |              ~90%              |

The (AA\*) + R3 pattern achieves the highest precision (~95%) with a pool of ~8
cards. This is a very small subpool -- sufficient for occasional targeted draws
but not for filling 3 slots per pack reliably.

## 4. Archetype-Identification Properties by Pool Design

The fundamental question: given a filter query, how many archetypes are
represented in the result set, and how concentrated is the distribution?

### Filter query analysis

| Filter query                        |          V7 pool (15% dual)           |        40% dual pool        |  3-symbol pool (60% dual)   |
| ----------------------------------- | :-----------------------------------: | :-------------------------: | :-------------------------: |
| "Has Tide" (any position)           | 4 archetypes (2 primary, 2 secondary) |            Same             |            Same             |
| "Primary = Tide"                    |         2 archetypes (50/50)          |    2 archetypes (50/50)     |    2 archetypes (50/50)     |
| "Pair = (Tide, Zephyr)"             |  1 archetype dominant (~80% S-tier)   | Same precision, larger pool | Same precision, larger pool |
| "Triple = (Tide, Tide, Zephyr)"     |          N/A (no repetition)          |             N/A             |     1 archetype (~95%)      |
| "(Tide, Zephyr) AND secondary Tide" |                  N/A                  |  Possible if many 3-symbol  |     1 archetype (~95%)      |

**Key insight:** Every pool design produces the same archetype-identification
ceiling for single-resonance filtering (50% home-archetype). Pair filtering
achieves ~80% regardless of pool size -- the precision is determined by
archetype structure, not pool composition. What changes with pool composition is
the **size of the filterable subpool**, which determines how many targeted slots
per pack can be sustained.

### Subpool sustainability analysis

An algorithm that fills K targeted slots per pack needs a subpool large enough
to sustain K draws per pack over ~25 post-commitment packs without excessive
repetition. Assuming the player sees each subpool card at most 3-4 times over a
full draft (acceptable repetition threshold):

```
Minimum subpool size >= K * 25 / 4 = 6.25 * K
```

| K (slots/pack) | Min subpool | Viable with pair filter at 15% dual (~7)? | At 40% (~18)? | At 60% (~24)? |
| :------------: | :---------: | :---------------------------------------: | :-----------: | :-----------: |
|       1        |      7      |                 Marginal                  |      Yes      |      Yes      |
|       2        |     13      |                    No                     |      Yes      |      Yes      |
|       3        |     19      |                    No                     |   Marginal    |      Yes      |

At 15% dual-resonance (V7), pair-matching can sustain at most 1 targeted slot
per pack -- far too few for M3=2.0. At 40%, 2 targeted pair-matched slots are
sustainable. At 60%, 3 slots become feasible.

## 5. Mathematical Conditions for M3 >= 2.0 Under Pessimistic Fitness

Under Pessimistic fitness (F=0.25), the S/A precision of R1 filtering is 62.5%.
Pair filtering achieves ~80% S-tier precision (archetype-level, not
fitness-dependent since pair-matching targets the home archetype directly).

However, V5's ~80% figure was measured under Optimistic fitness. Under
Pessimistic fitness, the pair-matched pool is smaller (fewer dual-resonance
cards exist per pair), and the precision depends on whether pair-matched cards
are concentrated in the home archetype:

- If 80% of (Tide, Zephyr) cards belong to Warriors: precision = 80% * 1.0
  (S-tier) + 20% * 0.25 (sibling A-tier rate) = 85% S/A precision
- If 50%/50% split between Warriors and Ramp: precision = 50% + 50% * 0.25 =
  62.5%

**Critical finding:** Pair-matching precision is partially immune to fitness
degradation because it concentrates draws in the home archetype rather than
splitting 50/50 between home and sibling. The ~80% home-archetype rate of
pair-matched cards means:

```
P(S/A | pair filter, Pessimistic) = 0.80 + 0.20 * 0.25 = 0.85
```

Compare to R1 filtering:

```
P(S/A | R1 filter, Pessimistic) = 0.50 + 0.50 * 0.25 = 0.625
```

**Pair-matching at Pessimistic fitness (85% precision) exceeds R1-filtering at
Moderate fitness (75% precision).** This is the single most important finding
for V8: pair-matching fundamentally changes the fitness sensitivity equation.

### M3 = 2.0 feasibility under Pessimistic fitness with pair-matching:

```
N * 0.85 + (4-N) * 0.125 = 2.0
```

(Using 0.125 as random baseline: each archetype is 1/8 of pool, ~half of that
archetype's cards are S/A under Pessimistic.)

```
N * 0.725 = 1.5 → N = 2.07
```

**Only 2.07 pair-matched slots per pack are needed for M3=2.0 under Pessimistic
fitness.** This is achievable with 3 pair-matched slots (delivering M3 ~2.6) or
a mix of 2 pair-matched + 1 R1-filtered slots.

But this requires a pool where pair-matching is sustainable at 2+ slots per
pack, which (from Section 4) requires at least 40% dual-resonance cards.

## 6. Summary: Pool Design Space Map

| Pool design         | Dual-res % |   Symbols/card   | Pair subpool size | R1 precision (Pess.) | Pair precision (Pess.) | Min M3 ceiling (Pess.) | Card design cost |
| ------------------- | :--------: | :--------------: | :---------------: | :------------------: | :--------------------: | :--------------------: | :--------------: |
| V7 baseline         |    15%     |  0-3 (avg ~1.5)  |      ~7/pair      |        62.5%         |     N/A (too few)      |   ~1.4 (3 R1 slots)    |       Low        |
| Moderate enrichment |    40%     |  2-3 (avg ~2.5)  |     ~18/pair      |        62.5%         |          ~85%          | ~2.3 (2-3 pair slots)  |      Medium      |
| V5-like             |    60%     |  2-3 (avg ~2.5)  |     ~24/pair      |        62.5%         |          ~85%          |  ~2.6 (3 pair slots)   |   Medium-High    |
| Full dual-res       |    100%    |       2-3        |     ~40/pair      |        62.5%         |          ~85%          |          ~2.8          |       High       |
| Repetition-rich     |    40%     | 3 (with repeats) |     ~8/triple     |        62.5%         |     ~95% (triple)      | ~2.5 (2 triple slots)  |   Medium-High    |

### Recommendations for algorithm designers

1. **The 40% dual-resonance threshold is the critical inflection point.** Below
   it, pair-matching cannot sustain enough targeted slots. Above it, returns
   diminish. The jump from 15% to 40% is the highest-value pool change.

2. **Pair-matching is the key to fitness robustness.** Its 85% precision under
   Pessimistic fitness exceeds R1-filtering's 75% under Moderate. Algorithms
   should be designed around pair-matched subpools, not R1 pools.

3. **Symbol repetition enables ultra-high precision but small subpools.** (Tide,
   Tide, Zephyr) filtering achieves ~95% precision but only ~8 cards. Use for
   occasional high-confidence slots, not bulk filling.

4. **M3=2.0 under Pessimistic fitness is mathematically achievable** if (a) the
   pool has >= 40% dual-resonance cards and (b) the algorithm uses pair-matching
   for >= 2 slots per pack. This is not a theoretical ceiling -- it is a
   concrete, designable system.

5. **The card design cost of 40% dual-resonance is moderate.** It requires ~18
   cards per archetype pair (out of ~40 per archetype) to carry a secondary
   resonance symbol. This means roughly half of each archetype's cards must be
   designed with the sibling archetype's resonance in mind -- a meaningful but
   not heroic card design effort.
