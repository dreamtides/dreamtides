# Model A v2: Consensus Hybrid -- N=8 with Suppression and Soft Floor

## Core Idea

**8 archetypes with 2 suppressed per run, adaptive weighted sampling with a
soft floor guarantee, clustered neighbor topology, and a starting card signal.**
This revision abandons the N=4 approach entirely in response to the unanimous
debate finding that 4 archetypes fails early diversity structurally (2.65 vs
3.0 target) and makes archetypes feel decorative. The new design adopts the
debate consensus: N=8 with Model D's suppression mechanism as the variety
engine, Model A's simple adaptive ramp philosophy for pack construction, and
a soft floor to prevent brick packs without inflating concentration.

**Player-facing explanation:** "Each quest draws from a shifting mix of
strategies -- the cards you see early tell you which ones are plentiful."

## Why N=8 (Changed from N=4)

The debate produced clear consensus: N=7-8 is the sweet spot. N=4 trivially
solves convergence but kills early diversity. N=10 requires 40%+ multi-archetype
cards -- an impractical design burden. N=8 gives:

- 45 S-tier cards per archetype (360/8) -- enough for identity
- ~90% probability of 3+ archetypes in a random 4-card pack
- 28 suppression configurations (C(8,2)) for structural run variety
- With 2 suppressed, 6 active archetypes per run boosts effective density by ~33%

## Card Fitness Distribution

360 unique cards, each with fitness scores in all 8 archetypes. Neighbor
topology: each archetype has 2-3 neighbors on a ring, sharing more overlap.

| Card Type | Count | % | Profile |
|-----------|-------|---|---------|
| Narrow Specialist | 155 | 43% | S in 1, B in 0-2 neighbors, C/F elsewhere |
| Specialist with Splash | 100 | 28% | S in 1, A in 1-2 neighbors, B in 1-2 |
| Multi-Archetype Star | 36 | 10% | S in 2 (neighbors), B in 2-3 |
| Broad Generalist | 51 | 14% | A in 2-3, B in 3-4, no S |
| Universal Star | 18 | 5% | S in 3+, high power, rare/legendary |

**Multi-archetype cards (S or A in 2+ archetypes): ~28% baseline** (100 splash
+ contributing generalists, but not all generalists qualify since some have A
in only 1). This is at the low end of viability, which is the point -- Model A
v2 tests the minimum design burden hypothesis. The sensitivity sweep will
determine if this is sufficient.

**Per-archetype S/A density:** Each archetype has ~45 S-tier unique cards plus
shared A-tier from splash and generalist cards, totaling roughly 65-75 S/A
unique cards. In the pool (~1000 entries), roughly 20-25% of entries are S/A
for any given archetype. With suppression boosting active archetype density
to ~28%, a random 4-card draw yields E[fitting] ~1.1 baseline.

## Pack Construction

**Phase 1 (Picks 1-5): Uniform random.** No bias. The pool's natural
composition (with 2 suppressed archetypes) provides implicit signals. The
starting card gives an explicit signal about active archetypes.

**Phase 2 (Picks 6+): Adaptive weighted sampling with soft floor.**
Once the player has 3+ S/A-tier picks in one archetype AND a 1+ lead over
the runner-up AND pick >= 5 (adapted from debate's standardized detection,
relaxed slightly because 8 archetypes with multi-archetype cards make clear
leads harder to establish), fitting cards receive weight multipliers:

- Picks 5-10: 6x weight for S/A cards in committed archetype
- Picks 11-20: 7x weight
- Picks 21-30: 8x weight

**Soft floor guarantee:** If weighted random produces 0 fitting cards in the
4-card pack, replace 1 random card with a fitting card drawn from the pool.
This fires only when needed (~15-25% of packs), preventing brick packs while
preserving natural variance in the majority of packs.

**Dedicated splash slot:** 1 of the 4 pack slots is always drawn from
off-archetype cards, biased toward high power or S-tier in a different active
archetype. This creates genuine "take the fitting card or the powerful bomb?"
tension.

## Variety Mechanisms

**Layer 1 -- Archetype Suppression (from Model D).** 2 of 8 archetypes are
suppressed per run: 50% of S-tier copies for suppressed archetype cards are
removed from the pool. Creates 28 structurally distinct run configurations.

**Layer 2 -- Starting Card Signal (from Model D).** Player sees 3 cards from
active archetypes, keeps 1 as a free pick. Reveals which archetypes are active
and nudges toward different starting points each run.

**Layer 3 -- Clustered Neighbor Topology (from Model B/D).** Each archetype
has 2-3 neighbors sharing more multi-archetype overlap. Pivoting to a neighbor
is cheap; pivoting to a distant archetype is expensive. Different suppression
patterns change which pivots are available.

**Layer 4 -- Copy-count variance (from Model A v1).** Each card's copy count
is randomly adjusted by +/-1 per run, creating subtle pool asymmetries.

**No depletion.** The debate found depletion's signal-reading value was hard
to validate and adds complexity. Dropped in favor of simplicity.

## Commitment Detection (Standardized from Debate)

Three conditions must ALL be met:
1. Pick number >= 5 (no bias before exploration phase ends)
2. 3+ S/A-tier picks in one archetype
3. 1+ lead over the runner-up archetype

The pick floor prevents premature convergence (Model C's pick-3 problem).
The lead requirement is relaxed to 1+ (from the debate's proposed 2+) because
at N=8 with multi-archetype cards, achieving a 2+ clear lead is unreasonably
difficult -- cards that are A-tier in multiple archetypes spread pick counts
across several archetypes simultaneously.

## Key Changes from Model A v1

| Aspect | v1 | v2 | Reason |
|--------|----|----|--------|
| N | 4 | 8 | Early diversity structurally broken at N=4 |
| Multi-arch % | 60% | 28% | N=8 needs less overlap; tests minimum viable |
| Variety mechanism | Copy-count variance | 2-of-8 suppression | 28 configs vs subtle adjustments |
| Soft floor | None | Yes, reactive | Prevents brick packs at higher N |
| Splash slot | None | 1 of 4 dedicated | Creates off-archetype tension |
| Commitment detection | 3+ picks, no pick floor | 3+ picks, 1+ lead, pick >= 5 | Prevents premature convergence |
| Depletion | None | None | Kept simple (debate consensus) |

## Design Philosophy

Model A v2 retains the original Model A philosophy of **algorithmic simplicity**
-- the pack construction is just weighted random sampling with two safety
mechanisms (soft floor + splash slot). The complexity budget is spent on pool
structure (suppression, topology) rather than pack algorithms. The hypothesis
is that with the right pool structure, simple pack construction is sufficient.
