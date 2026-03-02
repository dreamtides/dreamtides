# Model B: Many Archetypes with Tight Card Pools (N=10)

## One-Sentence Player Explanation

Each quest highlights a shifting landscape of ten distinct strategies; read the early packs to find which ones are deep this run, then commit and the draft will reward you.

## Archetype Count: 10

Ten archetypes is chosen deliberately at the upper end of the viable range. The Round 1 cardinality analysis identified N=6-8 as the "sweet spot" for easy convergence, but also noted that high N improves signal reading (smaller pools create larger relative fluctuations) and that A-tier breadth can substitute for lower N. This model tests the hypothesis that N=10 can work if the fitness distribution is designed carefully, the pack construction provides moderate algorithmic assistance, and per-run pool restriction creates readable asymmetries.

With 360 cards, each archetype has 36 S-tier cards and roughly 100 pool entries at S-tier (with rarity-based copy counts). This is thin but not unworkable -- the key is that multi-archetype cards expand the effective "fitting" pool significantly.

## Card Fitness Distribution

The distribution is designed to compensate for the thin per-archetype S-tier pools:

| Card Type | % of 360 | Count | Profile |
|-----------|----------|-------|---------|
| Narrow specialist | 30% | 108 | S in 1, B in 1-2, C/F elsewhere |
| Specialist with splash | 35% | 126 | S in 1, A in 1-2, B in 1-2, C/F elsewhere |
| Dual-archetype star | 10% | 36 | S in 2, A in 1, B in 1-2, C/F elsewhere |
| Broad generalist | 15% | 54 | A in 2-3, B in 3-4, no S |
| Universal star | 2% | 7 | S in 3+, high raw power |
| Pure filler | 8% | 29 | B in 2-3, C/F elsewhere, no S or A |

**Multi-archetype card percentage:** 62% of cards are S or A tier in 2+ archetypes (specialists-with-splash + dual stars + generalists + universal stars). This is high, but Q2 analysis predicts that N=10 demands it -- with only 36 S-tier cards per archetype, the A-tier breadth must compensate.

**Per-archetype effective pool:** Each archetype has approximately 36 S-tier unique cards plus ~55 A-tier cards = ~91 fitting (S/A) unique cards. With copy counts, this yields roughly 260-280 pool entries at S/A tier per archetype out of ~1000 total, or ~27% of the pool. A random 4-card draw yields E[fitting] = 1.08. This baseline is still below the 2+ target, requiring algorithmic assistance.

**Overlap topology:** Clustered, not uniform. The 10 archetypes are arranged in a ring of 5 pairs with natural overlap. Each archetype shares more multi-archetype cards with its 2-3 neighbors than with distant archetypes. This creates meaningful pivot paths ("I can shift from archetype A to its neighbor B and reuse 40% of my picks") while keeping distant archetypes distinct.

## Pack Construction: Adaptive Weighted Sampling with Soft Floor

The system uses weighted random sampling with two key mechanisms:

**1. Adaptive archetype ramp.** The system tracks the player's emerging archetype based on their picks' S/A fitness. The weight multiplier for cards fitting the player's strongest archetype(s) increases gradually:
- Picks 1-4: 1.0x (pure random, no bias)
- Picks 5-7: 2.0x for fitting cards (mild bias)
- Picks 8-12: 3.5x for fitting cards (moderate bias)
- Picks 13+: 5.0x for fitting cards (strong bias)

**2. Soft floor guarantee.** After pick 6, if the weighted random draw would produce 0 fitting cards, one random card in the pack is replaced with a fitting card. This prevents "brick packs" without making every pack predictable. The guarantee fires only when needed (roughly 15-25% of packs), preserving natural variance in the majority of packs.

**Archetype detection:** The player's archetype is identified as the archetype where they have the most S/A-tier cards picked. If there is a tie, both archetypes are considered (supporting hybrid play). The system requires 3+ S/A picks in an archetype before applying any bias.

## Run-to-Run Variety: Archetype Weighting with Pool Restriction

Each run, the system generates an archetype availability vector. Three archetypes are "boosted" (1.5x copy count multiplier for their S-tier cards), three are "normal" (1.0x), and four are "suppressed" (0.6x copy count multiplier). This creates a strong asymmetry: boosted archetypes have ~50% more available pool entries than suppressed ones.

This is the primary variety mechanism. Combined with random card-level variance (a further random 0.8x-1.2x multiplier on individual card copy counts), no two runs have the same archetype landscape.

**Signal design:** The first pack of the draft is assembled from the boosted archetypes' S-tier cards, serving as a semi-explicit signal. Observant players notice "lots of Reanimator in pack 1" and can infer that Reanimator is boosted this run. Subsequent packs use the standard weighted algorithm, but the pool asymmetry creates implicit frequency signals throughout picks 1-5.

## Why This Works (Theoretically)

The combination addresses the core challenge of high-N systems:
- **Thin S-tier pools** are compensated by broad A-tier coverage (62% multi-archetype cards)
- **Convergence** is achieved through adaptive weighting + soft floor, not through overwhelming archetype density
- **Variety** comes from pool restriction (archetype weighting per run), not from pack construction tricks
- **Signal reading** is naturally strong because small archetype pools create detectable fluctuations, amplified by the 3-boosted/4-suppressed structure

The risk is archetype identity dilution: with 62% multi-archetype cards, archetypes might feel too similar. The clustered overlap topology mitigates this -- archetypes share cards with neighbors but not with distant archetypes, preserving distinctness for the most part.
