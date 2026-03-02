# Model D v2: Refined Variety-First Draft System

## Core Idea

N=8 archetypes with **2 suppressed per run** (28 distinct configurations), a
**starting card signal** (see 3, keep 1), **clustered neighbor topology**
(each archetype has 2-3 neighbors sharing multi-archetype cards), and a
**soft floor guarantee** (replace 1 card when weighted draw yields 0 fitting).
Depletion has been removed in response to debate consensus that its signal
value was unvalidated and its complexity unjustified.

**Player-facing explanation:** "Each quest draws from a shifting pool of
strategies -- pay attention to what appears early, because it tells you
what's plentiful."

## Changes from Round 2

1. **Depletion removed.** The Round 3 debate unanimously agreed depletion was
   over-engineered, hard to explain, and produced minimal measurable signal
   value. The signal-reader barely outperformed the committed player, and no
   simulation could distinguish depletion signals from natural variance.

2. **Soft floor guarantee added.** From Models B/C: if the weighted random
   draw for picks 6+ produces 0 fitting cards in the committed archetype,
   replace one card with a randomly selected fitting card from the pool. This
   fires only when needed (~15-20% of packs), preventing brick packs without
   inflating concentration. This directly addresses the Round 2 late fitting
   shortfall (1.94 vs 2.0 target).

3. **Commitment detection refined.** Require: (a) pick number >= 6, AND
   (b) 2+ S/A picks in the leading archetype, AND (c) the leading archetype
   has strictly more S/A picks than the runner-up. The pick>=6 gate prevents
   Model C's premature convergence (pick 3), while the clear-lead requirement
   ensures commitment is genuine rather than noise-driven. Convergence
   typically occurs at pick 7-8.

4. **Clustered neighbor topology formalized.** Archetypes arranged in a ring.
   Each archetype has 2 neighbors sharing ~60% of multi-archetype cards. This
   creates meaningful pivot corridors: shifting to a neighbor reuses ~40% of
   picks, while pivoting to a distant archetype is expensive. When a neighbor
   is suppressed, the pivot landscape changes structurally.

5. **Weight ramp tuned.** Picks 1-5: uniform random (1.0x). Picks 6-10:
   8.0x for fitting cards. Picks 11+: 10.0x. Higher than Round 2's 5-7x
   because the reduced multi-archetype card percentage (28% vs 42%) means
   fewer fitting cards in the pool, requiring stronger weights to hit 2.0
   fitting per pack. The soft floor catches remaining worst cases.

6. **Concentration target relaxed.** Per unanimous debate consensus, the
   60-80% concentration target is mathematically incompatible with 2+
   convergence for committed players. Redefined as 85-95% for committed play;
   the 60-80% range applies to power-chasers who balance fit against power.

## Card Fitness Distribution

360 unique cards, each with fitness scores in all 8 archetypes:

- **72% Narrow Specialists (259 cards):** S in 1, B in 1-2 neighbors, C/F
  elsewhere. Archetype-defining cards. ~32 per archetype at S-tier.
- **14% Specialists with Splash (50 cards):** S in 1, A in 1-2 neighbors,
  B in 1-2 others. The convergence workhorse. ~6 per archetype at S.
- **4.5% Multi-Archetype Stars (16 cards):** S in 2 neighbor archetypes, B in
  2-3 others. Concentrated at archetype-pair intersections.
- **7% Broad Generalists (25 cards):** A in 2-3, B in 3-4, no S-tier.
  Flexible filler that prevents brick packs and supports hybrid strategies.
- **2.5% Universal Stars (10 cards):** S in 3+, high raw power. Rare/legendary.

**Multi-archetype percentage:** ~28% of cards are S or A in 2+ archetypes
(reduced from 42% in Round 2). The debate found that N=8 with 2 suppressed
needs ~25-30% multi-archetype cards. The soft floor compensates for the
reduction. Sensitivity sweeps test 5-40%.

**Per-archetype totals:** ~45 S-tier cards per archetype. With A-tier from
splash cards, ~70-80 S+A unique cards per archetype. With 6 active archetypes
after suppression, each active archetype has ~28% pool density at S/A.

## Pack Construction

**Picks 1-5 (exploration):** Draw 4 cards uniformly from the pool. No
archetype bias. Pool composition (with 2 suppressed archetypes) creates
implicit signals -- suppressed archetype cards appear ~50% less often.

**Picks 6+ (convergence):** Once commitment is detected, apply adaptive
weighted sampling. 3 slots drawn with weight multiplier on committed
archetype's S/A cards. 1 dedicated splash slot drawn from off-archetype cards
weighted by raw power and S-tier status in other active archetypes.

**Soft floor:** If the 3 weighted slots produce 0 fitting cards, replace the
lowest-fitness card with a random S/A card from the committed archetype's
pool. This fires ~15-20% of packs, preventing bricks while preserving natural
variance (most packs still have organic 1-3 fitting cards).

## Variety Mechanisms

**Layer 1 -- Archetype Suppression (structural):** 2 of 8 suppressed per
run. 28 distinct configurations. Cards whose primary archetype is suppressed
have 50% of copies removed. The same card can be a bomb in one run and filler
in the next.

**Layer 2 -- Starting Signal (explicit):** Player sees 3 cards from active
archetypes, keeps 1 as a free pick. The kept card reveals which archetypes
are active and nudges toward different strategies each run.

**Layer 3 -- Clustered Topology (pivot variety):** Neighbor archetypes share
more multi-archetype cards. Suppression changes which pivots are cheap.

## Archetype Frequency Control

Each archetype is active in 75% of runs (6 of 8 active). Combined with
starting card variance and natural pool randomization, no archetype dominates.
Target: each archetype drafted 10-15% of the time, no archetype exceeds 20%
or falls below 5%.
