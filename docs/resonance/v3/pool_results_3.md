# Pool Distribution Analysis for Lane Locking

## Comparison Table (archetype_committed strategy, 1000 drafts each)

| Metric | Target | A: Equal Flat | B: Equal Dual | C: Heavy Generic | D: Bridge-Heavy | E: Asymmetric |
|--------|--------|:-:|:-:|:-:|:-:|:-:|
| Early unique archs w/ S/A | >= 3 | 6.43 PASS | 6.45 PASS | 5.73 PASS | **6.74 PASS** | 6.48 PASS |
| Early S/A for arch/pack | <= 2 | 1.82 PASS | 1.80 PASS | **1.52 PASS** | 2.00 FAIL | 1.74 PASS |
| Late S/A for arch/pack | >= 2 | 2.68 PASS | 2.62 PASS | 2.60 PASS | **2.79 PASS** | 2.56 PASS |
| Late C/F cards/pack | >= 0.5 | 0.69 PASS | 0.80 PASS | 0.75 PASS | 0.58 PASS | **0.82 PASS** |
| Convergence pick | 5-8 | **6.1 PASS** | 6.2 PASS | 6.3 PASS | **6.1 PASS** | 6.2 PASS |
| Deck concentration | 60-80% | 98.8% FAIL | 98.4% FAIL | 97.2% FAIL | 99.0% FAIL | 97.8% FAIL |
| Card overlap | < 40% | 4.8% PASS | 5.3% PASS | 6.3% PASS | 5.3% PASS | 4.5% PASS |
| Archetype freq | 5-20% | 8-18% PASS | 9-19% PASS | 8-22% FAIL | 9-21% FAIL | 5-23% FAIL |
| **Targets passed** | | **7/8** | **7/8** | **6/8** | **5/8** | **6/8** |

## Key Finding: Pool Breakdown Matters Less Than Expected

The headline result is that Lane Locking is remarkably robust to pool composition changes. Late S/A per pack ranges only from 2.56 (Model E) to 2.79 (Model D) -- a 9% spread. Convergence pick is nearly identical (6.1-6.3) across all five models. The lock mechanism dominates: once slots lock to a resonance, the pool's internal structure has limited effect on what lands in locked slots.

This is actually good news for game design. It means the card pool can be designed primarily for gameplay variety and archetype identity rather than worrying that small distribution shifts will break the draft algorithm.

## Model-by-Model Analysis

**Model A (Equal Flat, 7/8 pass):** All cards carry only their primary resonance symbol [P]. This produces the cleanest lock behavior -- symbols accumulate at exactly 2 per pick, reaching threshold 3 on pick 2 and threshold 8 on pick 4. The downside: no secondary resonance information on cards means locked slots pull from ALL archetypes sharing that resonance equally. Deck composition skews toward S-tier cards (17.6 avg) with fewer A-tier (5.7) because mono-symbol cards are more concentrated.

**Model B (Equal Dual, 7/8 pass):** The mixed symbol pattern [P], [P,S], [P,P,S] is the recommended baseline from the final report. Ties Model A on targets but produces a better S/A ratio in drafted decks (15.8 S + 8.3 A vs 17.6 S + 5.7 A). The secondary symbols create more cross-archetype visibility in packs and slightly more splash (0.80 C/F vs 0.69). Bridge strategy is also stronger (3.12 vs 2.95 S/A per pack).

**Model C (Heavy Generics 33%, 6/8 pass):** Fails archetype frequency (Flash at 22%) because fewer archetype cards means less resonance diversity in the pool. The 120 generics (B-tier everywhere) inflate early flexibility but at the cost of archetype identity. Committed players find fewer S-tier cards (14.6 avg) and more B-tier filler. The draft feels less directional -- picking generics provides no lock progress.

**Model D (Bridge-Heavy, 5/8 pass):** The worst performer despite having the highest raw S/A count (132 per archetype, thanks to bridge cards being S-tier in two archetypes). The problem: bridges inflate early S/A to 2.00 (barely failing the <= 2 target), and the high S/A density across the pool means archetype frequency skews (Flash at 20.7%). Bridge cards are excellent for bridge strategists (3.23 S/A per pack, highest) but they paradoxically make single-archetype commitment harder to distinguish from bridging.

**Model E (Asymmetric, 6/8 pass):** Alternating deep (50) and shallow (30) archetypes creates measurable frequency skew -- Flash (deep, 50 cards) is drafted 23.2% of the time while Ramp (shallow, 30 cards) appears only 4.9%. The deep archetypes dominate because more cards in the pool means more chances to appear in random open slots early, biasing initial picks. This creates a "natural signal" effect (deep archetypes are easier to find) but at the cost of balanced play.

## Generics: 10% Is the Sweet Spot

The Model C result (33% generics, 6/8 pass) demonstrates that heavy generics dilute the draft experience without proportional benefit. Every generic card picked is a pick that adds no lock progress. At 33%, roughly 1.3 of 4 pack slots show generics, creating "dead" slots that feel undirected. The committed player's S-tier count drops from ~16-18 to 14.6.

At 10% (Models A/B), generics appear in roughly 0.4 of 4 slots -- enough to provide the occasional "good stuff" pick without disrupting resonance accumulation. The 36-card generic pool should be reserved for genuinely interesting cards that any deck wants (utility, removal, card draw) rather than filler.

## Bridge Cards: Unnecessary as a Separate Category

Model D's explicit bridge category (84 cards, 23% of pool) actively hurts. The bridge cards' dual S-tier fitness inflates early S/A above the 2.0 ceiling and creates archetype frequency imbalance. Meanwhile, Model B achieves strong bridge viability (3.12 S/A per pack) without any dedicated bridge cards, simply through natural dual-resonance patterns.

The key insight: **any card with [P, S] symbols already functions as a bridge card** between the two archetypes sharing those resonances. A Warriors card with [Tide, Zephyr] is automatically useful for Ramp (Zephyr/Tide). There is no need to design explicit bridge cards. The circular archetype arrangement guarantees that dual-symbol cards naturally serve bridge strategies.

Bridge strategy viability is uniformly high (2.95-3.23 S/A per pack) across all models. This is because committing to two adjacent archetypes doubles the S/A-eligible pool, which overwhelms any pool composition differences. Bridge strategies are inherently viable under Lane Locking regardless of distribution.

## Mono vs Dual Resonance Cards

Model A (all mono [P]) vs Model B (mixed) reveals the effect of symbol complexity. Both pass 7/8 targets. The functional differences:

- **Mono-heavy (Model A):** Slower secondary lock acquisition (no secondary symbols to accumulate). Drafted decks are more S-tier concentrated (17.6 S). Locks commit purely to primary resonances.
- **Mixed (Model B):** Secondary symbols enable locking a second resonance faster. Decks have more A-tier variety (8.3 A). Cards feel more "typed" -- a [Tide, Zephyr] card reads as specifically Warriors, while [Tide] reads as generically Tide.

The recommended distribution is Model B's mix. Dual-symbol cards create the most interesting draft decisions: "This [Tide, Zephyr] card advances both my Tide lock AND my Zephyr lock, but this [Tide, Tide] card is a bigger commitment to Tide." Mono-symbol cards work mechanically but miss this decision richness.

## Recommended Pool Breakdown

**40 cards per archetype (320 total), 40 generic (11%), equal distribution.** This is effectively Model B with a few extra generics rounding to 360.

Per archetype, the symbol pattern mix:
- 25% mono primary [P] -- flexible, useful to any archetype sharing that resonance
- 15% mono secondary [S] -- bridges naturally to adjacent archetypes
- 25% dual [P, S] -- the archetype's signature pattern
- 15% dual [P, P] -- deep commitment, fast threshold accumulation
- 10% dual [S, P] -- secondary-led, naturally bridges the other direction
- 10% triple [P, P, S] -- maximum commitment signal, locks fast

This produces ~10 mono-P, 6 mono-S, 10 dual-PS, 6 dual-PP, 4 dual-SP, and 4 triple-PPS per archetype (40 total). No explicit bridge category needed. No asymmetric sizes needed.

## How Distribution Affects the Feel

The most important experiential finding: **the pool breakdown affects deck texture more than draft mechanics**. A player drafting Warriors under Model A gets 17-18 S-tier Warriors cards and 5-6 adjacent A-tier cards. Under Model B, they get 15-16 S-tier and 8-9 A-tier. The draft itself feels nearly identical (same convergence speed, same lock timing), but the resulting deck has a different character -- Model B decks are more "splashy" with adjacent archetype cards, while Model A decks are more focused.

For a roguelike deckbuilder, the Model B texture is preferable. Decks that mix home archetype cards with adjacent archetype cards feel more varied and create more interesting build-around decisions during gameplay. Pure-archetype decks risk feeling samey across runs.
