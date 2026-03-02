# Question 1: The Cardinality Problem

How does the number of archetypes (N) interact with the fixed draft structure
(360 cards, 4 per pack, 30 picks)?

## Key Takeaways

- **The convergence target (2+ fitting cards per 4-card pack) is very hard to
  hit from pure random sampling unless N is small or A-tier breadth is high.**
  At N=8 with each card A-tier in 2 archetypes, uniform random gives only
  P(2+)=60%. Reaching 85% requires a 1.6x algorithmic boost. At N=10, it
  requires a 2.0x boost. This is the central tension.

- **N=6 to N=8 is a sweet spot** where early-draft diversity is high (87-90%
  chance of 3+ archetypes in a random pack), convergence is achievable with
  moderate algorithmic assistance, and archetypes have enough cards (45-60
  S-tier, 135-180 S+A) to feel internally varied.

- **A-tier breadth matters more than N for convergence math.** The difference
  between "each card is A in 1 other archetype" and "A in 2 others" is often
  larger than moving N by 2-3. This makes the fitness distribution (Q2) the
  most tightly coupled question to cardinality.

- **Low N (3-4) trivially solves convergence but kills early-draft exploration
  and signal reading.** With N=3, only 44% of packs show 3 different
  archetypes. Every pack is "two of the same thing," which undermines the
  feeling of meaningful choice.

- **High N (12+) creates a paradox: more variety in theory, but each archetype
  becomes too thin to support a full 30-card deck** without heavy algorithmic
  intervention. At N=15, there are only 24 S-tier cards per archetype; a
  committed player needs to draft nearly every one they see.

- **The hybrid-archetype design space scales quadratically with N** (C(N,2)
  pairs), which is a hidden benefit of moderate N. N=8 gives 28 possible
  archetype pairs, providing enormous combinatorial replayability without
  requiring that each pair be individually designed.

- **Depletion over 30 picks is nearly negligible** (~1-2 percentage points of
  P(2+) degradation), meaning late-draft convergence problems are not caused
  by pool exhaustion. The system does not need to worry about "running out" of
  archetype cards.

## Mathematical Foundation

### Cards Per Archetype

Assuming each card is S-tier in exactly 1 archetype and the pool has ~1000
entries (with rarity-based copy counts averaging ~3.0x), the baseline numbers
are:

| N | S-tier unique | S-tier pool entries | S+A pool (A-in-2) | SA% of pool |
|---|---------------|--------------------|--------------------|-------------|
| 3 | 120 | 396 | 1188 (capped at 1000) | 100% |
| 5 | 72 | 238 | 713 | 71% |
| 7 | 51 | 170 | 509 | 51% |
| 8 | 45 | 148 | 446 | 45% |
| 10 | 36 | 119 | 356 | 36% |
| 12 | 30 | 99 | 297 | 30% |
| 15 | 24 | 79 | 238 | 24% |

### The Convergence Threshold

For P(2+ S/A in a 4-card pack) to meet various targets from uniform random
sampling (no algorithmic boost):

| Target P(2+) | SA pool entries needed | % of pool | Achievable at N= |
|--------------|----------------------|-----------|-----------------|
| 50% | 386 | 38.6% | N<=9 (with A-in-2) |
| 60% | 445 | 44.5% | N<=8 (with A-in-2) |
| 70% | 509 | 50.9% | N<=7 (with A-in-2) |
| 80% | 583 | 58.3% | N<=6 (with A-in-2) |
| 90% | 680 | 68.0% | N<=5 (with A-in-2) |

This is the core constraint. Hitting 70% P(2+) from pure random requires that
half the pool be fitting cards, which only works if N is small or A-tier
coverage is broad. Any system with N>=8 *must* use algorithmic pack
construction to hit convergence targets.

### Early-Draft Diversity

The complementary concern: early packs (picks 1-5) should show variety. The
probability that a random 4-card pack contains cards from 3+ different
archetypes (assuming uniform archetype distribution):

| N | P(3+ archetypes) | P(all 4 different) |
|---|------------------|-------------------|
| 3 | 44% | 0% (impossible) |
| 5 | 77% | 19% |
| 7 | 88% | 35% |
| 8 | 90% | 41% |
| 10 | 94% | 50% |
| 15 | 97% | 65% |

N=3 is immediately disqualified for early-draft exploration: nearly half of
packs show only 1-2 archetypes. N=5 is marginal. N>=7 consistently delivers
diverse early packs.

## Design Space Exploration

### The Low-N Region (N=3-4): "Big Tent" Archetypes

Each archetype has 90-120 S-tier cards. Convergence is trivial, but the
problems are severe:

1. **Archetypes are too broad to feel strategic.** With 120 cards, an
   archetype is not a coherent strategy -- it is a third of the card pool.
   Players cannot meaningfully "figure out" what their archetype wants.

2. **Early packs lack variety.** With N=3, over half of packs show only 2
   archetypes, making early exploration feel monotonous.

3. **Signal reading is nearly impossible.** If each archetype covers 33% of the
   pool, removing 10% from one archetype changes its pack frequency by less
   than 4%. Over 5 early packs, the expected deficit is under 0.5 cards --
   invisible noise.

4. **Hybrid decks are trivial.** With only 3 pairs, every hybrid has been
   explored in a few runs.

### The Mid-N Region (N=6-8): The Operational Sweet Spot

This is where the math converges on a workable system:

- **Archetypes are large enough** (45-60 S-tier, 135-180 S+A unique) to
  support full decks with internal variety. A player needing 18-24 S/A cards
  for their deck must find only 13-18% of the archetype's available pool,
  leaving room for choice and divergent builds within the same archetype.

- **Early diversity is high** (87-90% of packs show 3+ archetypes), satisfying
  the "open-ended early" goal.

- **Convergence requires moderate assistance.** At N=8 with A-in-2, baseline
  P(2+) is 60%. A 1.3-1.6x algorithmic boost reaches 70-85%. This is a
  gentle thumb on the scale, not a heavy-handed override -- preserving the
  "not on rails" feeling.

- **Archetype overlap is moderate** (19-22% shared S/A cards between any two
  archetypes), enabling hybrid decks without making archetypes feel
  interchangeable.

- **Combinatorial richness is high** (21-28 archetype pairs), providing
  significant replayability.

### The High-N Region (N=10-15): "Micro-Archetypes"

The appeal is clear: many distinct strategies, high variety. The problems are
mathematical:

1. **Convergence requires heavy intervention.** At N=10, baseline P(2+) is
   only 45% even with A-in-2. Reaching 85% requires a 2.0x boost -- the
   algorithm must roughly double the probability of seeing fitting cards. This
   makes the system either invisible (opaque to the player) or feels like it
   is deciding for you.

2. **Deck construction becomes fragile.** With only 36 S-tier and 108 total
   S/A unique cards per archetype, a player needs to draft 17-22% of all
   available cards in their archetype. Missing a few key offerings is
   devastating. This creates the "on rails" feeling where you *must* take
   every fitting card you see.

3. **Card design burden is extreme.** Each of 360 cards needs meaningful
   fitness scores in 10-15 archetypes. The design cost of creating natural
   multi-archetype cards (already identified as expensive) scales roughly with
   N.

4. **But signal reading improves.** Smaller archetypes create larger relative
   fluctuations, making "which archetype is open" easier to detect. This is
   one genuine advantage of high N.

## Surprising Insights

### 1. A-tier breadth and N are substitutable to a surprising degree

Moving from "A-in-1" to "A-in-2" at N=8 has the same convergence effect as
moving from N=8 to N=5 at fixed A-in-1. This means the cardinality question
cannot be answered independently of the fitness distribution question. A
designer choosing N=10 with broad A-tier coverage can achieve the same pack
math as N=7 with narrow A-tier -- but the archetypes will *feel* different
because the broad A-tier means more overlap and less distinctiveness.

### 2. The "not on rails" and "convergent" goals create a hard mathematical tradeoff

To be "not on rails," most packs should offer cards from multiple archetypes
(need high N or low archetype density). To be "convergent," most packs should
offer 2+ cards from the player's archetype (need low N or high archetype
density). These pull in opposite directions. The resolution must come from
*pack construction algorithms* that tilt early packs toward diversity and late
packs toward focus, or from *fitness distributions* that create a rich middle
tier (B-level cards are everywhere, S/A are concentrated).

### 3. N affects "archetype identity" nonlinearly

At N=3, archetypes are so large (120 cards) that they cannot have a coherent
mechanical identity. At N=15, archetypes are so small (24 cards) that each is
essentially a single combo or strategy with no room for variation. But between
N=6 and N=10, there is a qualitative transition: around N=7-8, archetypes are
large enough to have internal sub-strategies (e.g., "aggressive tokens" vs.
"controlling tokens" within a Tokens archetype) but small enough that the
archetype label still means something specific. This internal richness is
critical for replayability -- it means two players who both draft "Tokens" can
end up with very different decks.

## Parameters for Round 2 Simulations

Simulations should systematically test:

1. **N values: 5, 7, 8, 10** -- spanning the viable range from "few big
   archetypes" to "many small ones."
2. **A-tier breadth: 1, 2, 3** -- the number of archetypes each card is A-tier
   in, which the math shows is as impactful as N itself.
3. **Algorithmic boost factor: 1.0x to 2.5x** -- how much the pack
   construction favors the player's archetype.
4. **Convergence timing: measure at pick 5, 8, 12, 20** -- to see the full
   trajectory, not just end-state.
5. **Deck coherence vs. deck size** -- at high N, does the system produce
   decks that are 60-80% S/A, or does it force compromises?

## Concrete Predictions

1. **I predict that N=7 or N=8 with A-in-2 will be the sweet spot** where
   convergence is achievable with a 1.3-1.5x boost, early diversity is high,
   and archetypes feel distinct. N=5 will converge too easily (on rails) and
   N=10 will require too much algorithmic force.

2. **I predict that N>=10 systems will fail the "not on rails" goal** even when
   they hit convergence targets, because the small archetype size forces
   players to take every fitting card offered, eliminating meaningful choice
   within the archetype.

3. **I predict that the % of multi-archetype cards (S/A in 2+ archetypes)
   matters more at high N than low N.** At N=5, convergence works even without
   many multi-archetype cards. At N=10, multi-archetype cards are mandatory
   bridges that make the math possible.

4. **I predict that hybrid decks (blending 2 archetypes) will emerge naturally
   at N=7-8** because the 20-25% overlap between any two archetypes provides
   just enough shared cards to build a viable bridge, whereas at N=3 hybrids
   are trivial and at N=12 the overlap is too thin (~10%) for unassisted
   hybrid construction.

5. **I predict that the early vs. late pack tension will be the dominant design
   challenge,** more so than choosing N. Whatever N is chosen, the system must
   solve the problem of showing variety in picks 1-5 and focus in picks 6+,
   and the mechanism for that transition will define the player experience more
   than the archetype count.
