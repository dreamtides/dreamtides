# Model A: Big Archetypes with Internal Variety

## Core Idea

**4 archetypes, each containing 90 S-tier cards, with clustered sub-themes
that create internal variety.** The system trades archetype count for depth:
fewer archetypes means convergence is near-trivial from the math alone, so the
pack construction algorithm can stay simple and transparent. Variety comes from
internal archetype diversity (multiple viable builds within each archetype) and
per-run pool restriction.

**Player-facing explanation:** "Each run highlights different card combinations
-- read the early packs to find which strategy has the deepest support."

## Why 4 Archetypes

The Q1 analysis identified N=3 as problematic (only 44% of packs show 3+
archetypes) and N=5 as marginal. I choose N=4 as a compromise that preserves
the "big archetype" philosophy while keeping early-draft diversity viable:

- P(3+ archetypes in a 4-card pack) = ~66% at N=4, better than N=3's 44%
- P(all 4 different archetypes) = ~9.4%, creating occasional "one of each"
  packs
- Each archetype gets 90 S-tier unique cards (360/4), providing massive
  internal variety
- With 6 archetype pairs (C(4,2)=6), hybrid decks are a meaningful design
  axis

The key insight from Q1 is that at low N, early-draft diversity is the binding
constraint. At N=4 we accept that some packs will show only 2 archetypes early
on, but compensate with rich intra-archetype choice.

## Fitness Distribution

Each of the 360 cards has fitness scores in all 4 archetypes:

| Card Type | Count | % | Profile |
|-----------|-------|---|---------|
| Narrow Specialist | 144 | 40% | S in 1, B in 1, C in 1, F in 1 |
| Specialist with Splash | 108 | 30% | S in 1, A in 1, B in 1, C in 1 |
| Dual-Archetype Star | 36 | 10% | S in 2, B in 1, C in 1 |
| Broad Generalist | 54 | 15% | A in 2, B in 2 |
| Universal Star | 18 | 5% | S in 1, A in 2, B in 1 |

**Multi-archetype cards (S or A in 2+ archetypes): 60% of the pool.** This is
higher than Q2's recommended 25-35% because with only 4 archetypes, each card
naturally touches more of the design space. The 4-archetype structure makes
multi-archetype design easier -- there are only 6 possible archetype pairs to
design intersection cards for.

**Per-archetype S/A density:** Each archetype has ~90 S-tier cards plus shared
A-tier cards, totaling roughly 135-150 S/A unique cards per archetype. In a
pool of ~1000 entries (with rarity copies), about 40-45% of entries are S/A
for any given archetype. This means a random 4-card pack yields an expected
1.6-1.8 fitting cards -- close to the 2+ target without any algorithmic help.

## Pack Construction

**Adaptive weighted sampling with a gentle ramp.** The system uses a single
mechanism throughout the draft:

- **Picks 1-5 (exploration phase):** Cards are drawn with uniform weights.
  No archetype detection. The player sees the raw pool.
- **Picks 6+ (convergence phase):** Once the player has picked 3+ S/A-tier
  cards in one archetype, the system identifies their "lead archetype." Cards
  with S/A fitness in that archetype receive a weight multiplier that ramps
  from 1.5x at pick 6 to 3.0x by pick 15, then holds steady.

The ramp is gentle because the underlying math already favors convergence at
N=4. The multiplier ensures the player sees 2+ fitting cards reliably without
making off-archetype cards vanish.

**Commitment detection:** The archetype with the most S/A-tier picks becomes
the lead archetype once the player has 3+ such picks in it. If two archetypes
are tied, no bias is applied (the player is still exploring or building a
hybrid).

## Variety Mechanisms

Three layers create run-to-run variety:

1. **Copy-count variance (per-run).** Each card's copy count is randomly
   adjusted by +/- 1 (clamped to 1 minimum). This creates subtle per-run pool
   asymmetries without removing any card entirely.

2. **Archetype weighting (per-run).** At pool creation, each archetype
   receives a random multiplier between 0.7x and 1.3x applied to copy counts
   of its S-tier specialists. This creates 1 "deep" archetype and 1 "shallow"
   archetype per run, producing detectable signals for observant players.

3. **Internal archetype variety.** With 90 S-tier cards per archetype, no
   two runs draft the same 20-25 cards. The large pool within each archetype
   ensures different card combinations emerge even when drafting the same
   archetype.

Signal reading works via the archetype weighting: in picks 1-5, the deep
archetype appears more frequently, and an observant player drafting toward it
will be rewarded with better convergence later.

## Why This Works (Theoretical Argument)

The central advantage of N=4 is that convergence is nearly free. With ~40-45%
of the pool fitting any given archetype, the system barely needs to intervene.
This means:

- The algorithm stays simple and transparent (goal 1)
- Off-archetype cards remain abundant (goals 4, 6)
- The player's choices drive the draft, not the algorithm (goal 2)

The central risk is early-draft monotony (only 4 archetypes to show). The
mitigation is that each archetype is so internally diverse that "two cards from
the same archetype" still presents a meaningful choice between different
sub-strategies.

## Archetype Structure

The 4 archetypes are conceptual slots. For concreteness in simulation, imagine:
- **Archetype 0:** Aggressive (fast creatures, direct damage)
- **Archetype 1:** Control (removal, card advantage, late-game threats)
- **Archetype 2:** Synergy (tribal, tokens, sacrifice combos)
- **Archetype 3:** Resource (ramp, big spells, energy manipulation)

Each archetype has 2-3 internal sub-themes represented by different card
clusters within the S-tier pool, creating deck-building variety within a
single archetype commitment.
