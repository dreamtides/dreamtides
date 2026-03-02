# Q2: The Fitness Distribution Problem

## Key Takeaways

- **The specialist-to-generalist ratio is the single most important tuning lever in the entire draft system.** It controls convergence speed, archetype distinctness, flexibility, and the "on rails" feeling more directly than N (number of archetypes) or the pack construction algorithm.
- **Multi-archetype cards are expensive to design but NOT optional.** A system with zero multi-archetype cards produces a binary experience: either the system feeds you your archetype or you are picking from garbage. The minimum viable fraction is roughly 15-20% of cards having A-tier or better in 2+ archetypes, but the sweet spot is likely 25-35%.
- **"S in 2 archetypes" and "S in 1, A in 2" serve completely different design functions.** True dual-archetype stars create pivot points and draft tension; specialists-with-splash create smooth filler and prevent brick packs. The system needs both, but in different quantities.
- **Archetype depth asymmetry is a feature, not a bug, but only if intentional.** Having some archetypes with 50 S-tier cards and others with 30 creates natural difficulty tiers and discovery arcs for the roguelike meta-game. Accidental asymmetry just feels broken.
- **There is a critical threshold effect: below ~35 S/A-tier cards per archetype in the pool, convergence becomes mathematically impossible without algorithmic intervention.** This constrains the relationship between N (archetype count), pool size (360), and the fitness distribution.
- **Card overlap is a double-edged sword with a sharp tipping point.** Below ~20% overlap, archetypes feel like separate games shuffled together. Above ~50% overlap, archetypes lose identity. The 25-40% range is where interesting draft decisions live.
- **The fitness distribution determines whether signal-reading is possible.** If most cards are generalists, there are no signals to read. If most are narrow specialists, the signals are loud but the experience is brittle. The information content of each pack depends on the ratio of specialists to generalists.

---

## The Fitness Matrix as Design Space

Consider 360 cards and N archetypes. Each cell in the 360xN matrix contains a fitness tier (S/A/B/C/F). The distribution of values across this matrix defines the entire draft experience. I will analyze the key dimensions of this distribution and their consequences.

### Dimension 1: The Specialist-Generalist Spectrum

Define card archetypes by their fitness profile shape:

- **Narrow Specialist:** S in 1, F in most others (maybe B in 1-2). A Reanimator card that is useless outside Reanimator.
- **Specialist with Splash:** S in 1, A in 1-2, B/C elsewhere. A Reanimator card that also works in a sacrifice-themed deck.
- **Multi-Archetype Star:** S in 2+. A card designed at the intersection of two archetypes.
- **Broad Generalist:** A or B in many archetypes, S in none. A generically efficient card.
- **Universal Star:** S in 3+ or high raw power. Rare cards everyone wants.

**Too many specialists (>60%):** Convergence is fast but brittle. Once committed, every pack is "1 great card + 3 unplayables." The player feels locked in by pick 3-4 with no room to pivot. Worst of all, early picks before commitment feel like coin flips -- you cannot evaluate cards without knowing your archetype, but you do not know your archetype yet. The "not on rails" goal is violated even though the "convergent" goal is easily met.

**Too many generalists (>50%):** Convergence is technically easy (everything is playable) but meaningless. Archetypes dissolve into "slightly different flavors of good stuff." The "flexible archetypes" goal is trivially met but "signal reading" becomes impossible because there are no signals -- every card fits everywhere. Deck identity disappears.

**The productive tension lives at roughly 35-45% specialists, 25-35% specialists-with-splash, 10-15% multi-archetype stars, 10-20% generalists, and 2-5% universal stars.** This is a prediction, not a known answer, and Round 2 simulations should sweep these ratios.

### Dimension 2: S-Tier Depth per Archetype

With 360 cards and N=8 archetypes, if each card is S-tier in exactly 1 archetype, that is 360/8 = 45 S-tier cards per archetype. With rarity-based copies (common x4, uncommon x3, rare x2, legendary x1), the effective pool entries per archetype at S-tier could be ~120. From a pool of ~1000 entries, that is 12% of the pool. Drawing 4 cards randomly, the expected count of S-tier cards for your archetype is 0.48 -- well below the 2+ target.

This is the fundamental math that makes the system hard. Pure S-tier is not enough. You need S + A combined to reach the "2 fitting cards per pack" target. If A-tier cards roughly double the fitting pool (to ~24% of entries), expected fitting cards per pack of 4 rises to ~0.96. Still below 2. You need either algorithmic boosting OR a fitness distribution where ~30%+ of the pool is S/A-tier for any given archetype -- which means significant multi-archetype overlap.

**Minimum viable archetype depth:** An archetype needs enough S/A cards that, even without algorithmic help, you see at least 1 fitting card per pack on average. That requires roughly 25% of the pool at S/A tier. With 1000 pool entries, that is 250 entries -- meaning ~80-100 unique cards at S or A tier per archetype. With 360 total cards, this means each card must be S or A in about 2 archetypes on average. This is the core tension: either you need heavy algorithmic intervention, or you need extensive multi-archetype card design.

### Dimension 3: Multi-Archetype Card Design Cost

This is the most practically important dimension. Three categories of multi-archetype cards serve different functions:

**True dual-archetype stars (S in 2+):** These are the hardest to design. They require finding genuine mechanical overlap -- a card that is a core piece of two different strategies. In practice, these emerge naturally at archetype intersections (a "Reanimator + Tokens" card that creates tokens from the void). I estimate a realistic game can produce maybe 10-15% of cards at this level. These cards create the most interesting draft decisions: when two players want the same card for different reasons.

**Specialists with splash (S in 1, A in 1-2):** Easier to design. A card can be core to one strategy and incidentally useful in another without needing to be mechanically perfect for both. "A good removal spell that happens to trigger sacrifice synergies" is easier than "a card that is the centerpiece of both Reanimator and Tokens." Realistically 25-35% of cards can hit this level. These cards are the backbone of the "splashable" goal and prevent brick packs.

**Generalists (A/B in many):** The easiest category. Efficient stats, flexible utility, generic removal. These need no inter-archetype design work. But they dilute archetype identity.

**Can the system work with zero multi-archetype cards?** No. With zero overlap, each archetype is a hermetically sealed pool. The math above shows that without overlap, you cannot hit the convergence targets without extreme algorithmic intervention -- which violates the "simple" goal. The system needs multi-archetype cards to function at a basic level.

**Minimum viable multi-archetype percentage:** I predict the floor is around 15-20% of cards at A+ in 2+ archetypes. Below this, packs after commitment will regularly contain 0 fitting cards even with moderate algorithmic boosting. The comfortable operating range is 25-35%. Above 40%, archetype identity begins to blur.

**Sensitivity analysis prediction:** I predict the convergence metric (2+ fitting cards per pack at pick 6+) is HIGHLY sensitive to this percentage in the 10-25% range (small changes produce large effects) and much less sensitive above 30% (diminishing returns). This is the most important parameter for Round 2 to sweep.

### Dimension 4: Overlap Topology

Not just HOW MUCH overlap, but WHERE. Two structural options:

**Uniform overlap:** Every pair of archetypes shares roughly the same number of cards. With N=8, there are 28 pairs. If 100 cards are multi-archetype, each pair shares ~3.5 cards. This is thin but democratic.

**Clustered overlap:** Some archetypes are "neighbors" sharing many cards, others are distant. Reanimator and Sacrifice share 15 cards; Reanimator and Tokens share 2. This creates a topology -- an implicit graph or wheel of archetype relationships. This is much richer for drafting (you can pivot to a neighbor but not to a distant archetype) and much richer for signal reading.

**I strongly predict clustered overlap produces better draft experiences.** It creates meaningful pivot decisions ("I started Reanimator but Sacrifice is more open, and I can reuse 60% of my picks"), natural archetype families, and a learnable metagame structure. Uniform overlap makes every pivot equally costly, which is paradoxically less flexible because no pivot is ever cheap.

---

## Surprising Insights

**1. Specialists make early drafting WORSE, not better.** Intuitively, you might think specialists help because they send clear signals. But in picks 1-5 before commitment, a pack of 4 narrow specialists from 4 different archetypes presents the player with a choice they cannot meaningfully evaluate. Every card is "S in something I might not play." Generalists and multi-archetype cards are what make early picks feel good, because they retain value across future commitments. The ideal early-game pack has 2-3 cards with broad applicability and 1 specialist that tempts you toward a specific archetype.

**2. The fitness distribution affects convergence speed MORE than the pack construction algorithm.** A perfect algorithm sampling from a pool with no multi-archetype overlap cannot outperform a random algorithm sampling from a pool with 30% multi-archetype overlap, because the algorithm can only select from what exists. The distribution is the binding constraint; the algorithm is optimization on top.

**3. Asymmetric archetype depth creates a natural difficulty gradient for the roguelike.** If "Reanimator" has 50 S-tier cards and "Combo Control" has 25, Reanimator is easier to draft successfully. For a roguelike, this is a FEATURE: new players gravitate toward deep archetypes, experienced players challenge themselves with shallow ones. This also interacts with signal-reading: shallow archetypes produce stronger signals (their S-tier cards are rarer and more diagnostic) while deep archetypes are safer but harder to read.

---

## Parameters for Round 2 Simulation

1. **Specialist fraction:** Sweep 20%, 35%, 50%, 65% (cards S in exactly 1, C/F elsewhere)
2. **Multi-archetype fraction (A+ in 2+):** Sweep 0%, 10%, 20%, 30%, 40%
3. **Dual-star vs. splash split:** Within the multi-archetype fraction, vary the ratio of "S in 2" to "S in 1, A in 2"
4. **Overlap topology:** Compare uniform overlap vs. clustered (neighbor-pair) overlap
5. **Archetype depth symmetry:** Compare all-equal depth vs. varied (some 50 S-tier, some 25)
6. **Generalist fraction:** Sweep 5%, 15%, 25% (cards with B+ in 4+ archetypes, S in 0)

## Concrete Predictions

- **I predict that 30% multi-archetype cards with clustered overlap will outperform 40% multi-archetype cards with uniform overlap** on both the convergence and archetype-distinctness metrics simultaneously.
- **I predict that the "specialist with splash" category (S in 1, A in 1-2) contributes more to convergence targets than "true dual stars" (S in 2+)** because there can be more of them, making them the workhorse of the system.
- **I predict that below 15% multi-archetype cards, no pack construction algorithm can hit the 2+ fitting cards target without making the system feel "on rails"** (guaranteed slots feel deterministic).
- **I predict that generalist fraction above 20% will cause the "archetype frequency across runs" metric to fail** because power-chasing becomes dominant over archetype commitment.
- **I predict an inverted-U relationship between specialist fraction and signal-reading quality:** too few specialists means no signals, too many means signals are obvious but useless (you are locked in before you can act on them). Peak signal value is at 35-45% specialists.
