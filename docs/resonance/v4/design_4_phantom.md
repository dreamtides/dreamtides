# Design 4: Phantom Drafter / Competitive Scarcity

## Key Takeaways

- **Phantom drafters create natural variance without touching pack generation.** The pack algorithm can be trivially simple (draw 4 random cards from the pool) because all convergence and variance emerge from how the pool is depleted by competition. This is structurally different from every V3 approach, which manipulated pack composition directly.
- **Signal reading is native to competitive scarcity.** When certain resonances disappear from your packs, a phantom is consuming them. This makes Goal 8 (signal reading) a first-class feature rather than an afterthought bolted on via pool asymmetry.
- **The archetype-vs-resonance gap is a major challenge.** Phantoms that draft by resonance remove cards from 4 archetypes simultaneously, which means a phantom "competing" for Tide hurts Warriors, Sacrifice, Self-Mill, and Ramp equally. Archetype-level convergence requires phantoms that create scarcity in specific resonance *combinations*, not single resonances.
- **Pool depletion rate is the critical tuning parameter.** Too few phantom picks and the pool barely changes (no convergence). Too many and the pool empties fast, creating forced scarcity that feels like railroading. The sweet spot is 1-3 phantom picks per player pick, depleting roughly 10-15% of the pool by pick 15.
- **Phantom strategies must be heterogeneous.** A single phantom with one resonance preference creates a binary world (competing vs. not competing). Multiple phantoms with diverse preferences create a richer landscape where the player must read which archetypes are contested and which are open.
- **Convergence through competition requires the player to *avoid* contested resources, not accumulate preferred ones.** This inverts the V3 model. The player converges by reading what is scarce and pivoting to what is abundant, rather than by building up resonance counts that mechanically improve their packs.
- **Simplicity is achievable but fragile.** "Phantom drafters remove cards from the pool each round" is genuinely simple. But the moment you add complex phantom strategies or reactive phantom behavior, the algorithm becomes opaque to players. The best designs keep phantom behavior dead simple.

---

## Proposal 1: Single Phantom, Fixed Resonance

**One-sentence description:** A phantom drafter with a randomly-assigned primary resonance removes 2 cards per round from the pool, always choosing the cards with the most symbols matching its resonance.

**Technical description:** At quest start, a phantom is assigned a random resonance (e.g., Ember). Each round, before the player's pack is generated, the phantom scans the pool for the 2 cards with the highest symbol count in its assigned resonance (primary=2, secondary/tertiary=1) and removes them permanently. The player's pack is then 4 random cards drawn from the remaining pool. Over 30 picks, the phantom removes 60 cards, depleting roughly 17% of the pool with a heavy bias toward one resonance.

**Assessment:** Serves signal reading (Goal 8) well --- the player can notice Ember cards becoming scarce and pivot away. Fails convergence (Goal 5) because the mechanism only *discourages* one resonance; it does not actively *encourage* the player's chosen archetype. Also fails flexibility (Goal 4) since the phantom's fixed resonance creates a permanent no-go zone rather than dynamic scarcity.

**Best symbol distribution:** Mostly 2-symbol cards (55%), so the phantom's resonance preference creates broad but not total scarcity across related archetypes.

---

## Proposal 2: Rival Pack Draft

**One-sentence description:** Each round, 8 cards are drawn from the pool; a phantom picks 2 of them (choosing the best match for its resonance), and you see the remaining 6 as a pack of 6, pick 1 of 6.

**Technical description:** Instead of drawing 4 cards for the player, the system draws 8. A phantom drafter with a randomly-assigned resonance preference selects the 2 cards with the highest symbol match to its resonance and removes them. The player sees the remaining 6 cards and picks 1. The 5 unpicked cards return to the pool. The phantom's resonance is fixed at quest start but hidden from the player. This is essentially a shared draft table where the phantom gets first pick.

**Assessment:** Strong on signal reading (Goal 8) --- the player sees 6 cards but can infer the phantom's preference from what is *missing*. Good on openness (Goal 7) because 6-card packs offer more variety. Convergence (Goal 5) is weak because the phantom only removes 2 cards per round from a single pack, not enough to reshape the pool. Splashability (Goal 6) is excellent with 6-card packs. The "6 cards, pick 1" change is a significant departure from the 4-card format.

**Best symbol distribution:** Heavy on 2-symbol cards. The phantom needs enough symbol density to have clear preferences, but 3-symbol cards would make the phantom's picks too predictable.

---

## Proposal 3: Multiple Phantoms, Ecosystem Competition

**One-sentence description:** Three phantom drafters each remove 1 card per round from the pool, each preferring a different randomly-assigned resonance; you draft from whatever is left.

**Technical description:** At quest start, 3 phantoms are each assigned a distinct resonance (the 4th resonance is "open"). Each round, each phantom removes the single card from the pool with the highest symbol count in its assigned resonance. The player's pack is 4 random cards drawn from the depleted pool. Over 30 picks, the 3 phantoms remove 90 cards total (25% of the pool), heavily concentrated in 3 resonances. The 4th "open" resonance becomes naturally abundant, creating a strong signal for the player to read.

**Assessment:** Excellent signal reading (Goal 8) --- three contested resonances means one is always open, and a perceptive player can identify it. Strong on "no forced decks" (Goal 3) because the open resonance changes each run. Convergence (Goal 5) is indirect --- the player converges by drafting the abundant resonance, not by accumulating symbols. Risk: with 3 of 4 resonances contested, the player effectively has only 2 viable archetypes (those sharing the open resonance as primary), which badly fails flexibility (Goal 4).

**Best symbol distribution:** 40% one-symbol, 45% two-symbol, 15% three-symbol. More one-symbol cards make phantom preferences sharper and the open/contested signal clearer.

---

## Proposal 4: Mirrored Phantom

**One-sentence description:** A phantom drafter mirrors your picks --- each card you draft causes the phantom to remove 2 cards from the pool that share the same primary resonance as your pick.

**Technical description:** There is no pre-assigned phantom preference. Instead, after each player pick, the phantom removes the 2 cards from the pool with the highest symbol count matching the *primary resonance of the card the player just drafted*. This creates a feedback loop: drafting Tide cards makes Tide scarcer, which makes it harder to continue drafting Tide, which creates natural diminishing returns on deep commitment. The player's pack is 4 random cards from the depleted pool.

**Assessment:** Strong on "not on rails" (Goal 2) because deep commitment to one resonance actively works against you by depleting your own resources. Creates genuine pivot incentives. Convergence (Goal 5) is problematic --- the mechanism *fights* convergence by removing the cards you want. Signal reading (Goal 8) is weak because scarcity directly mirrors the player's own actions, not an external signal. The self-limiting dynamic is strategically interesting but may frustrate players who want to commit.

**Best symbol distribution:** Mostly 2-symbol cards (60%) to ensure the feedback loop is meaningful but not devastating. Too many 1-symbol cards would make depletion of a single resonance too aggressive.

---

## Proposal 5: Drifting Phantom Swarm

**One-sentence description:** Four phantom drafters each start with a random resonance preference but shift their preference toward whichever resonance the player drafts least, so the open lane the player ignores gradually becomes contested.

**Technical description:** Four phantoms are initialized with random resonance assignments (one per resonance). Each round, each phantom removes 1 card matching its current resonance. After every 5 player picks, each phantom has a 40% chance of switching its resonance to the player's *least-drafted* resonance (by symbol count). This creates a dynamic where initially all resonances are equally contested, but over time the phantoms migrate toward resonances the player is ignoring, making the player's chosen resonance relatively more abundant. Packs are 4 random cards from the depleted pool.

**Assessment:** Strong convergence (Goal 5) --- the player's archetype becomes more available as phantoms drift away from it. Good flexibility (Goal 4) because pivoting causes phantoms to eventually drift away from the new target too. Signal reading (Goal 8) is moderate --- the player can observe which resonances are becoming more/less available. Fails simplicity (Goal 1) --- the drifting behavior is hard to explain in one sentence, and the 40%-every-5-picks rule is arbitrary. Also somewhat "on rails" because the phantoms are effectively serving the player's interests, making the competition feel artificial.

**Best symbol distribution:** 30% one-symbol, 50% two-symbol, 20% three-symbol. Balanced distribution so the phantom drift creates gradual rather than sudden shifts.

---

## Champion Selection: Proposal 3 --- Multiple Phantoms, Ecosystem Competition

**Why this champion?** Proposal 3 hits the intersection of simplicity, natural variance, and genuine signal reading better than any other phantom design. The one-sentence description is fully implementable: three phantoms, each removes one card per round from its assigned resonance, player drafts from what is left. There is no hidden complexity, no reactive behavior, no feedback loops.

The critical insight is that 3-of-4-resonance scarcity naturally creates 1 "open" resonance per run, which solves signal reading (Goal 8) as a first-class feature. This is something V3's Lane Locking could not do at all. The open resonance changes every run, preventing forced decks (Goal 3). Variance is natural because the pool depletion is noisy --- sometimes the phantoms take cards from archetypes you care about, sometimes they do not.

The main risk --- that 3 contested resonances leave only 2 viable primary archetypes --- can be mitigated by tuning phantom aggressiveness. If each phantom removes only 1 card per round (not 2-3), the contested resonances are diminished but not eliminated, keeping all 8 archetypes viable while still creating a detectable signal.

Proposal 1 is too simple (single phantom, weak convergence). Proposal 2 changes the pack size. Proposal 4 fights convergence. Proposal 5 fails simplicity. Proposal 3 is the sweet spot.

---

## Champion Deep-Dive: Multiple Phantoms, Ecosystem Competition

### Example Draft Sequences

**Early Committer (commits to Warriors by pick 5):**
Phantoms are assigned Ember, Stone, Zephyr (Tide is open). Picks 1-3: the player sees a mix of cards but notices Tide cards appearing frequently while Ember and Stone cards are thinning out. By pick 4-5, the player recognizes Tide is the open resonance and commits to Warriors (Tide/Zephyr primary). Picks 6-15: Warriors cards appear regularly because Tide is abundant, though Zephyr (Warriors' secondary) is contested, meaning pure [Zephyr, Tide] cards from Ramp are somewhat scarce. The player builds a Warriors-heavy deck with some splash from Sacrifice (also Tide-primary). By pick 20, the pool has lost 60 cards (17%), with Tide relatively overrepresented. The player's deck is 65-75% Warriors S/A, with meaningful variance pack to pack.

**Flexible Player (stays open through pick 10):**
Phantoms are assigned Ember, Tide, Zephyr (Stone is open). Picks 1-8: the player picks the strongest card in each pack regardless of archetype, noticing that Stone cards are consistently available. By pick 8, they have accumulated a mix of symbols but with a Stone lean. Picks 9-15: they commit to Storm (Ember/Stone) --- Stone is abundant but Ember is contested, so they get plenty of Stone-primary cards but fewer Ember-primary ones. Their Storm deck is viable but slightly weaker than if they had committed to a pure Stone-primary archetype like Self-Discard. The flexibility penalty is real but not crippling.

**Pivot Attempt (starts Warriors, pivots to Sacrifice at pick 10):**
Phantoms are assigned Ember, Stone, Zephyr (Tide is open). Picks 1-7: the player drafts Warriors (Tide/Zephyr). At pick 8 they realize the Warriors cards appearing are not high-power and consider pivoting. Picks 8-10: they experiment with Sacrifice (Tide/Stone). Since Tide is still open (no phantom is consuming it), the pivot is viable --- Sacrifice cards are available. However, Stone (Sacrifice's secondary) is contested, so the player gets strong Tide-primary Sacrifice cards but fewer Stone-secondary ones. The pivot works partially, resulting in a Sacrifice-leaning deck that splashes some earlier Warriors picks. The shared Tide resonance between Warriors and Sacrifice makes intra-resonance pivots relatively smooth.

### Predicted Failure Modes

1. **Binary archetype viability.** With 3 contested resonances, only 2 archetypes share the open resonance as primary. If phantoms are too aggressive, the player is effectively choosing between 2 archetypes, failing Goal 4 (flexible archetypes). Mitigation: reduce phantom picks to 1 per round, making contested resonances "diminished" (losing ~25% of cards by draft end) rather than "depleted."

2. **Adjacent archetype bleed.** The 2 archetypes sharing the open resonance as primary compete with each other for the same abundant cards. If both are equally viable, the player has no signal to choose between them. Mitigation: this is actually acceptable --- the player differentiates by secondary resonance, which has varying levels of contestation.

3. **Weak convergence for secondary-resonance archetypes.** Archetypes that use the open resonance only as secondary (not primary) get a weaker benefit. A player targeting Ramp (Zephyr/Tide) when Tide is open benefits less than a Warriors (Tide/Zephyr) player. Mitigation: moderate phantom aggressiveness so the benefit is a tilt, not a binary.

4. **Predictability across runs.** With only 4 possible "open resonance" states, experienced players might recognize the open resonance by pick 2-3 and always follow it, leading to formulaic drafts. Mitigation: randomize phantom aggressiveness (1-2 picks per round instead of fixed 1), and allow occasional phantom resonance overlap (two phantoms share a resonance, leaving 2 resonances open).

### Parameter Variants Worth Testing

1. **Phantom count and pick rate:** 3 phantoms each removing 1 card/round (base case) vs. 2 phantoms removing 2 cards/round vs. 4 phantoms removing 1 card/round. The total removal rate is similar (3 vs. 4 cards/round) but the distribution of scarcity differs.

2. **Phantom overlap:** Always 3 distinct resonances (base case) vs. allowing 2 phantoms to share a resonance (creating 1 heavily contested + 1 mildly contested + 2 open resonances). Overlap creates more varied strategic landscapes.

3. **Phantom escalation:** Phantoms remove 1 card/round for picks 1-10, then 2 cards/round for picks 11-20, then 3 for picks 21-30. This keeps early packs diverse (Goal 7) while increasing convergence pressure in the late draft.

### Proposed Symbol Distribution

| Symbol Count | % of Non-Generic | Cards |
|---|---|---|
| 0 (generic) | -- | 36 |
| 1 symbol | 35% | 113 |
| 2 symbols | 45% | 146 |
| 3 symbols | 20% | 65 |

A higher proportion of 1-symbol cards (35% vs. the V3 recommendation of 25%) sharpens phantom preferences. When a phantom assigned to Ember picks from the pool, 1-symbol [Ember] cards are unambiguous targets, making the scarcity signal cleaner. Two-symbol cards create cross-archetype competition (the phantom taking a [Tide, Zephyr] card hurts both Warriors and Ramp), which is desirable for creating nuanced scarcity. Three-symbol cards at 20% provide strong resonance signals per drafted card for the player's own symbol accumulation, though this is less critical for Proposal 3 since the algorithm does not use the player's symbol counts at all.
