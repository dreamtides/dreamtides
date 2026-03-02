# Domain 1: Accumulation-Based Mechanisms

## Key Takeaways

- **Bag-building is the strongest accumulation mechanism** because it is genuinely simple (one sentence, no hidden parameters), creates natural convergence from player choices alone, and preserves early-game openness via the small initial bag.
- **Accumulation mechanisms inherently satisfy convergence (goal 5)** because every pick permanently increases the weight of chosen resonances. The challenge is preventing over-convergence that kills splashability and flexibility.
- **Dilution is the core tension.** Pure accumulation (no decay, no cap) inevitably creates late-game rails where 3 of 4 pack slots match the committed resonance. Every proposal must include a structural answer to how off-resonance cards continue to appear.
- **Symbol distributions with 2 symbols per card are the sweet spot for accumulation.** One-symbol cards accumulate too slowly (convergence doesn't arrive until pick 10+). Three-symbol cards accumulate too fast (convergence by pick 3 locks players in). Two symbols per card hits convergence around pick 5-7.
- **Signal reading is naturally weak in accumulation systems** because the pool is not visibly changing -- the player's private bag/counter is what shifts. Algorithms that draw from a shared pool (rather than a private bag) can partially address this.
- **Pivoting is expensive in all accumulation designs** because prior picks permanently pollute the accumulated state. This is a feature (commitment matters) and a risk (bad early picks feel punishing). The bag-building approach handles this best because early picks contribute few tokens relative to later accumulated mass.
- **Generic (0-symbol) cards are a natural pressure valve.** Since they add nothing to the bag/counter, drafting them avoids deepening commitment. This creates an interesting draft tension absent from non-accumulation designs.

---

## Proposal 1: Token Bag

**Player-facing description:** "Each resonance symbol you draft adds a token of that type to your bag (primary symbol adds 2 tokens); to fill each slot in your next pack, draw a random token from the bag and show a random card with that primary resonance."

**Technical description:** The player starts with an empty bag. When they draft a card, the card's primary (leftmost) symbol adds 2 tokens and each secondary/tertiary symbol adds 1 token. To construct a 4-card pack, draw 4 tokens from the bag (with replacement) and for each token, select a random card from the pool whose primary resonance matches. If the bag is empty (before any pick, or after drafting only generic cards), fill all 4 slots randomly.

**Assessment:**
- Serves well: Simple (goal 1), Convergent (goal 5), Open early (goal 7) -- the bag is tiny early so results are volatile, then stabilizes.
- Serves poorly: Splashable (goal 6) -- late-game bags are heavily skewed, off-resonance cards become rare. Signal reading (goal 8) -- the bag is private, no shared pool to read.
- Best symbol distribution: 2 symbols per card (e.g., [Primary, Secondary]). Adds 3 tokens per pick, reaching meaningful convergence around pick 5-6.

---

## Proposal 2: Running Tally Slots

**Player-facing description:** "Count your drafted resonance symbols (primary counts as 2); your highest-count resonance fills 2 pack slots, your second-highest fills 1 slot, and the last slot is a random resonance you have not yet drafted the most of."

**Technical description:** Maintain four running counters, one per resonance. After each pick, update counters using the 2/1/1 weighting rule. Sort resonances by counter value. The top resonance gets 2 of 4 pack slots (random cards with that primary resonance), the second resonance gets 1 slot, and the 4th slot is filled by a random card whose primary resonance is NOT the top resonance. Ties are broken randomly.

**Assessment:**
- Serves well: Convergent (goal 5) -- guaranteed 2 slots for top resonance after any commitment. Splashable (goal 6) -- the 4th slot is structurally guaranteed to be off-resonance. Simple (goal 1) -- one sentence covers it.
- Serves poorly: Open early (goal 7) -- even after 1 pick, the top resonance gets 2 slots, creating premature convergence. Not on rails (goal 2) -- the rigid 2/1/1 structure is very deterministic. Flexible archetypes (goal 4) -- always privileges exactly one resonance.
- Best symbol distribution: 1-2 symbols. Since the algorithm reads only which resonance is highest, more symbols just make ties rarer; the actual count magnitudes do not matter much.

---

## Proposal 3: Resonance Meters

**Player-facing description:** "Each resonance has a meter from 0 to 10; drafting a symbol adds to that meter (primary adds 2, others add 1); when you open a pack, each of the 4 slots rolls a random number 1-10 and shows a card from the resonance whose meter is >= that roll, or a random card if none qualify."

**Technical description:** Four meters, one per resonance, start at 0 and cap at 10. Each drafted symbol increments the corresponding meter (primary +2, secondary/tertiary +1). For each pack slot, generate a uniform random integer from 1-10. A resonance "qualifies" for that slot if its meter >= the rolled value. Among qualifying resonances, pick one uniformly at random and select a random card with that primary resonance. If no resonance qualifies, select a fully random card.

**Assessment:**
- Serves well: Open early (goal 7) -- meters are low so most slots default to random. Convergent (goal 5) -- high meters qualify more often, so committed resonances appear frequently. Splashable (goal 6) -- randomness in the roll means off-resonance results occur naturally.
- Serves poorly: Simple (goal 1) -- the "roll against meter" mechanic is a full paragraph to explain, borderline fails the Simplicity Test. Not on rails (goal 2) -- once a meter hits 10, that resonance appears in almost every slot. Flexible archetypes (goal 4) -- hard to build a balanced dual-resonance deck because one meter always races ahead.
- Best symbol distribution: 2 symbols per card. Needs moderate accumulation speed so meters reach interesting values (4-7) during the mid-draft.

---

## Proposal 4: Snowball Pool

**Player-facing description:** "You have a personal pool that starts with 1 copy of every card; each time you draft a card, add 3 more random cards that share its primary resonance to your personal pool; your packs are drawn from your personal pool."

**Technical description:** Initialize a personal pool containing one copy of each of the 360 cards. When the player drafts a card with primary resonance R, add 3 random cards from the master set whose primary resonance is R to the personal pool (duplicates allowed). Packs are 4 random cards drawn from the personal pool. The personal pool grows by 2 net cards per pick (add 3, remove 1 drafted). Over 30 picks the pool grows from 360 to ~420 cards, with chosen resonances increasingly over-represented.

**Assessment:**
- Serves well: Signal reading (goal 8) -- the player can observe which resonances are appearing more frequently in their packs, creating a feedback signal. Splashable (goal 6) -- the base pool of 360 cards means off-resonance cards always have a chance. Not on rails (goal 2) -- the pool shift is gradual enough that players see diverse options.
- Serves poorly: Convergent (goal 5) -- adding 3 cards to a pool of 360+ is a tiny shift. After 15 picks of Tide, only ~45 extra Tide cards in a pool of ~390, raising Tide concentration from ~25% to ~30%. Too weak. Simple (goal 1) -- "personal pool" is a somewhat abstract concept.
- Best symbol distribution: Does not strongly depend on symbol count since the mechanism adds cards, not tokens. 2 symbols per card is fine.

---

## Proposal 5: Weighted Lottery

**Player-facing description:** "Each resonance starts at weight 1; each drafted symbol adds 1 to that resonance's weight (primary adds 2); to fill a pack slot, pick a resonance with probability proportional to its weight, then show a random card of that resonance."

**Technical description:** Maintain four weights, all starting at 1. After each pick, increment weights: +2 for the primary symbol's resonance, +1 for each secondary/tertiary symbol's resonance. To construct a pack, for each of the 4 slots independently: select a resonance with probability weight_r / sum(all_weights), then select a random card from the pool whose primary resonance matches. Starting weights of 1 ensure all resonances have baseline representation.

**Assessment:**
- Serves well: Simple (goal 1) -- genuinely one sentence. Open early (goal 7) -- weights start at [1,1,1,1] so first packs are nearly uniform. Convergent (goal 5) -- committed resonance weight grows steadily. Splashable (goal 6) -- baseline weight of 1 ensures every resonance always has nonzero probability.
- Serves poorly: Signal reading (goal 8) -- weights are private. Flexible archetypes (goal 4) -- dual-resonance builds are possible but the dominant weight always gets ~50% of slots. Not on rails (goal 2) -- mid-to-late packs are fairly predictable once weights diverge.
- Best symbol distribution: 2 symbols per card. Adds 3 weight per pick. After 10 picks, dominant resonance might have weight ~20 out of total ~34, giving it ~59% of slots. With 1-symbol cards, adds only 2 per pick and convergence is slower. With 3-symbol cards, adds 4 per pick and convergence is too fast.

---

## Champion Selection: Weighted Lottery (Proposal 5)

The Weighted Lottery is the strongest proposal because it achieves the best balance across all eight design goals while being genuinely expressible in a single sentence. Here is why it wins over the alternatives:

**Over Token Bag (Proposal 1):** The Token Bag has no baseline representation for undrafted resonances. Once a player commits to Tide/Zephyr, the bag contains zero Ember/Stone tokens, so those resonances literally cannot appear. The Weighted Lottery's starting weight of 1 per resonance ensures every resonance always has some probability, directly solving the splashability problem without bolt-on fixes.

**Over Running Tally (Proposal 2):** The Running Tally's rigid 2/1/1 structure creates premature convergence (2 slots for top resonance even after pick 1) and feels deterministic. The Weighted Lottery achieves convergence gradually through probability rather than guaranteed slot counts.

**Over Resonance Meters (Proposal 3):** The Meters mechanism fails the Simplicity Test. "Roll a number, check against meter, pick qualifying resonance" is a multi-step conditional process that players cannot easily simulate in their heads.

**Over Snowball Pool (Proposal 4):** The Snowball Pool's convergence is far too weak. Adding 3 cards to a 360+ card pool barely shifts probabilities. To make convergence meaningful, you would need to add 10+ cards per pick, but then the pool description becomes "add a lot of cards" which raises questions about how many.

---

## Champion Deep-Dive: Weighted Lottery

### Algorithm Specification

Each resonance (Ember, Stone, Tide, Zephyr) starts at weight 1. When you draft a card, add 2 to the weight of its primary (leftmost) symbol's resonance and add 1 for each secondary/tertiary symbol. To fill each slot in a 4-card pack, pick a resonance with probability equal to its weight divided by the total of all weights, then show a random card from the pool with that primary resonance. Cards with no symbols add nothing to any weight.

### Example: Early Committer (Warriors/Tide-Zephyr)

**Pick 1:** Weights [E:1, S:1, T:1, Z:1]. Total=4. Each resonance ~25%. Pack is diverse. Player drafts a [Tide, Zephyr] card. Weights become [E:1, S:1, T:3, Z:2]. Total=7.

**Pick 2:** Tide=43%, Zephyr=29%, Ember=14%, Stone=14%. Pack likely has 1-2 Tide cards. Player drafts [Tide]. Weights: [E:1, S:1, T:5, Z:2]. Total=9.

**Pick 5:** After consistently drafting Tide/Zephyr, weights might be [E:1, S:1, T:11, Z:6]. Total=19. Tide=58%, Zephyr=32%, others=5% each. Packs typically show 2-3 Tide cards and 1 Zephyr card. Convergence achieved.

**Pick 15:** Weights ~[E:1, S:1, T:23, Z:14]. Total=39. Tide=59%, Zephyr=36%. Most packs are 2 Tide + 1-2 Zephyr. Off-resonance cards appear roughly 1 in 8 slots (~0.5 per pack), meeting splashability target.

### Example: Flexible Player

**Pick 1-3:** Player drafts one Tide card, one Ember card, one Stone card. Weights: [E:3, S:2, T:3, Z:1]. Total=9. No resonance dominates. Packs remain diverse.

**Pick 4-7:** Player begins favoring Ember/Stone (Storm archetype). By pick 7, weights might be [E:9, S:6, T:3, Z:1]. Total=19. Ember=47%, Stone=32%. Still seeing some Tide cards (~16%), plenty of flexibility to stay open.

**Pick 8+:** Commitment solidifies. Weights continue accumulating toward Ember/Stone. Convergence arrives later than the early committer (around pick 8-9 rather than 5-6), which is the natural cost of staying flexible.

### Example: Pivot Attempt

**Picks 1-6:** Player drafts Tide cards. Weights: [E:1, S:1, T:13, Z:4]. Total=19. Tide=68%.

**Pick 7:** Player wants to pivot to Ember. Drafts [Ember, Stone]. Weights: [E:3, S:2, T:13, Z:4]. Total=22. Tide still dominates at 59%.

**Picks 7-15:** Player drafts exclusively Ember/Stone. By pick 15, weights: [E:19, S:12, T:13, Z:4]. Total=48. Ember=40%, Stone=25%, Tide=27%. The pivot partially succeeds -- Ember is now the dominant resonance -- but Tide remains elevated as a "ghost" from early picks. The player ends up with an Ember-primary deck with significant Tide splash. This is a reasonable outcome: pivoting is possible but costly, and early picks leave a permanent (but diminishing) mark.

### Predicted Failure Modes

1. **Late-game homogeneity.** After 20+ picks, the dominant resonance's weight dwarfs the starting weights of 1. Off-resonance probability drops below 5%. Mitigation: the starting weights could be higher (e.g., 3 instead of 1) to maintain a stronger baseline, at the cost of slower convergence.

2. **Generic cards are dead weight.** Drafting a 0-symbol card adds nothing to any weight, meaning the player "wastes" a pick in terms of shaping future packs. This could make generic cards feel bad to draft even when they are the strongest card available. Counterpoint: this is actually an interesting strategic tension.

3. **Dual-resonance archetypes are asymmetric.** A Warriors player drafting [Tide, Zephyr] cards will always have Tide dominate because primary symbols count double. Their packs will skew Tide-heavy rather than balanced Tide/Zephyr. This could make the secondary resonance feel under-served. Mitigation: symbol distributions where archetype cards often have [Primary, Secondary] naturally split weight, and the secondary resonance still beats unrelated resonances.

### Parameter Variants Worth Testing

1. **Starting weight:** Test starting weights of 1, 3, and 5 per resonance. Higher starting weights slow convergence (more dilution) and maintain higher baseline splash probability. Weight 1 gives fastest convergence but potentially too fast. Weight 5 means the total starts at 20, so a single [P,S] card adding 3 is only a 15% shift -- convergence may be too slow.

2. **Primary symbol multiplier:** Test primary counting as 2, 3, and 4 (with secondary/tertiary always 1). Higher multiplier means primary resonance dominates faster and secondary resonance matters less. At multiplier 4, a [Tide, Zephyr] card adds +4 Tide and +1 Zephyr, heavily favoring mono-resonance convergence.

3. **Weight cap:** Test uncapped vs. cap at 15 or 20 per resonance. A cap prevents runaway dominance in late drafts and ensures off-resonance cards always have meaningful probability. With cap 15 and starting weight 1, the maximum share for one resonance is 15/18 = 83%.

### Proposed Symbol Distribution for Simulation

- **0 symbols:** 36 cards (10%) -- generic cards
- **1 symbol:** 65 cards (20%) -- [Primary] only, simple mono-resonance
- **2 symbols:** 194 cards (60%) -- [Primary, Secondary], the backbone
- **3 symbols:** 65 cards (20%) -- [Primary, Primary, Secondary] or [Primary, Secondary, Tertiary]

This distribution means the average drafted card adds ~2.7 weight to the bag per pick (2 for primary + ~0.9 for secondary/tertiary on average, accounting for 10% generic cards adding 0). After 6 picks, a committed player's dominant resonance has weight ~13 out of total ~20, giving ~65% pack share -- right in the convergence target zone.
