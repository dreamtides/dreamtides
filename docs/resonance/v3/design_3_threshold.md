# Domain 3: Threshold/Progression Mechanisms

## Key Takeaways

- **Thresholds create natural drama and decision points.** The moment a player crosses a threshold is felt as a discrete "level up," giving the draft clear milestones. This is the domain's strongest advantage over gradual-scaling approaches.
- **Threshold placement is the critical design lever.** Set thresholds too low and the system locks in too early; set them too high and the player never experiences the payoff. The sweet spot is a first threshold reachable by pick 4-6, with a second around pick 10-15.
- **Binary unlock mechanics are the simplest to explain but the hardest to balance.** "Reach X to unlock Y" is trivially simple, but if Y is too powerful the system feels on-rails, and if Y is too weak the threshold feels meaningless.
- **Multi-resonance thresholds risk tunnel vision.** If thresholds only reward a single resonance, players are incentivized to draft mono-resonance aggressively. Designs must actively ensure secondary resonances remain viable.
- **The best threshold algorithms combine a clear milestone with a structural pack change.** Rather than unlocking hidden quality tiers, the strongest designs change the visible composition of packs in a way the player can predict and plan around.
- **Symbol distributions with 2 symbols per card (majority) work best for thresholds.** With mostly 1-symbol cards, thresholds take too long to reach; with mostly 3-symbol cards, players cross multiple thresholds too quickly and the "level up" moments blur together.
- **Lane-locking (Proposal 5) is the champion.** It is the simplest to explain, creates visible pack structure changes at clear milestones, naturally supports splashing, and avoids the quality-tier opacity that plagues rarity-gating approaches.

---

## Proposal 1: Rarity Gating

**Player-facing description:** "Drafting 4 symbols of a resonance unlocks that resonance's uncommon cards in your packs; 8 symbols unlocks rares; 14 unlocks legendaries."

**Technical description:** Each resonance has a "level" based on the player's accumulated symbol count (primary counts as 2, secondary/tertiary as 1). Cards in the pool are tagged by rarity. When building a pack of 4, each card is drawn randomly from the full eligible pool, but a card's rarity must not exceed the player's level in that card's primary resonance. Cards with no symbols (generic) are always eligible regardless of rarity.

**Assessment:**
- Serves well: Convergent (goal 5) -- committed players unlock powerful rares/legendaries. Simple concept (goal 1).
- Fails: Transparency (goal 2) -- players cannot tell which rarity tier they have unlocked without tracking counts precisely. Splashable (goal 6) -- off-resonance cards are stuck at common tier, making them undesirable. Not on rails (goal 4) -- once you unlock rares in one resonance, switching is heavily punished because you lose access to rares.
- Best symbol distribution: Mostly 2-symbol cards. With 1-symbol cards, reaching 8 or 14 takes too long. With 3-symbol cards, cross-resonance thresholds are reached too easily, undermining the gating.

---

## Proposal 2: Bonus Card Thresholds

**Player-facing description:** "For each resonance where you have 6 or more symbols, your packs contain 5 cards instead of 4, with the bonus card always matching that resonance."

**Technical description:** The base pack is 4 random cards drawn from the full pool. For each resonance where the player's accumulated symbol count (primary=2, secondary/tertiary=1) is 6 or more, one additional card of that resonance is appended to the pack. The player still picks exactly 1 card from the expanded pack.

**Assessment:**
- Serves well: Simple (goal 1) -- one sentence, fully specified. Convergent (goal 5) -- committed players get larger packs skewed toward their resonance. Splashable (goal 6) -- the base 4 cards remain random. Flexible archetypes (goal 4) -- a player at 6 in two resonances gets 6-card packs.
- Fails: Not on rails (goal 2) -- reaching 6 in two resonances is much better than 6 in one, creating a strong incentive to dual-commit early. Open-ended early (goal 7) -- the threshold at 6 may be unreachable before pick 5-6, so no effect is felt early, but this is actually fine since early picks should be open.
- Best symbol distribution: Mixed, leaning toward 2-symbol cards. Need enough symbols per pick to reach 6 by pick 4-6 (2 symbols/pick average = 8-12 symbols by pick 4-6, distributed across resonances).

---

## Proposal 3: Escalating Slot Reservation

**Player-facing description:** "For every 5 symbols you have in a resonance, one slot in your 4-card pack is guaranteed to be a card of that resonance."

**Technical description:** Divide each resonance's accumulated symbol count by 5 (round down) to get the number of reserved slots for that resonance. When building a 4-card pack, fill reserved slots first (each with a random card whose primary resonance matches). If total reserved slots exceed 4, prioritize by highest count. Remaining slots are filled randomly from the full pool.

**Assessment:**
- Serves well: Simple (goal 1) -- the division-based rule is concrete and predictable. Convergent (goal 5) -- at 10 symbols, 2 of 4 slots are reserved. Transparent (goal 2) -- players can count their symbols and know exactly how many slots are reserved.
- Fails: Not on rails (goal 2) -- at 15+ symbols in one resonance, 3 of 4 slots are locked, leaving almost no choice. Splashable (goal 6) -- late in the draft, reserved slots crowd out off-resonance cards. Flexible archetypes (goal 4) -- the system strongly rewards concentration in one resonance since 10 in one resonance gives 2 slots, but 5 in two resonances gives only 1 slot each (same total but less focused benefit).
- Best symbol distribution: Mostly 1-2 symbol cards. With 3-symbol cards, players accumulate symbols in multiple resonances too quickly, and total reserved slots exceed 4 by mid-draft.

---

## Proposal 4: Tiered Pool Expansion

**Player-facing description:** "All cards start face-down in 4 resonance piles; drafting 3/7/12 symbols of a resonance flips that pile's common/uncommon/rare cards face-up, and your packs draw only from face-up cards plus generics."

**Technical description:** The 360-card pool is divided into 4 resonance sub-pools (by primary symbol) plus a generic pool. Each sub-pool is further divided by rarity. Initially, only commons in all sub-pools and all generics are "face-up" (available). When a player's symbol count in a resonance reaches 3, that resonance's uncommons become available; at 7, rares; at 12, legendaries. Packs of 4 are drawn randomly from the combined face-up pool.

**Assessment:**
- Serves well: Convergent (goal 5) -- committed players access deeper card pools with stronger cards. Signal reading (goal 8) -- which resonances have more commons visible affects early picks. Open-ended early (goal 7) -- all four resonances' commons are available from the start.
- Fails: Simple (goal 1) -- the mechanic is understandable but requires tracking 4 separate resonance levels and 3 thresholds each (12 thresholds total). Transparent (goal 2) -- hard to predict which specific cards are available. Not on rails (goal 2) -- once rares are unlocked in one resonance, switching loses access to them.
- Best symbol distribution: 2-symbol cards predominantly. Reaching thresholds 3/7/12 requires enough symbol accumulation to hit the first threshold by pick 3-4.

---

## Proposal 5: Lane Locking (CHAMPION)

**Player-facing description:** "Your pack has 4 slots; when your symbol count in a resonance first reaches 3, one random slot permanently becomes that resonance's slot and always shows a card of that resonance."

**Technical description:** Each of the 4 pack slots starts "open" (filled with a random card from the full pool). When the player's accumulated symbol count in any resonance first crosses 3 (primary=2, secondary/tertiary=1), one of the remaining open slots is permanently assigned to that resonance. From that point on, that slot always contains a random card whose primary resonance matches. If all 4 slots are locked, the system is fully determined. The threshold is crossed once per resonance, so at most 4 slots can be locked (one per resonance).

**Assessment:**
- Serves well: Simple (goal 1) -- one sentence, completely specified, a player can predict their pack structure. Transparent (goal 2) -- the player knows exactly which slots are locked and to what. Convergent (goal 5) -- committed players lock slots to their resonance early. Splashable (goal 6) -- unlocked slots remain random, and even locked slots in a secondary resonance provide splash. Flexible archetypes (goal 4) -- a player can lock 2 slots to different resonances to build a dual-resonance archetype. Open-ended early (goal 7) -- with 0 locked slots, all 4 cards are random. Not on rails (goal 2) -- with 4 possible locks, the player shapes their pack structure through choices.
- Fails: No forced decks (goal 3) -- if a player always rushes Tide to 3, their first lock is always Tide. But the random card pool and the remaining 3 open slots provide enough variance. Signal reading (goal 8) -- moderate; the open slots in early packs give some signal about pool composition.
- Best symbol distribution: Majority 2-symbol cards (60%), with 25% 1-symbol and 15% 3-symbol. This ensures the first threshold is reachable by pick 3-4 (drafting two 2-symbol Tide cards gives 4 Tide symbols, crossing the threshold).

---

## Champion Selection: Lane Locking

Lane Locking is the champion because it delivers the core promise of the threshold domain -- discrete, felt "level up" moments -- while being genuinely simple enough to pass the Simplicity Test. The one-sentence description fully specifies the algorithm. Unlike rarity gating or pool expansion (which hide their effects behind invisible quality tiers), lane locking produces a visible, structural change to the pack that the player can see and reason about.

The key insight is that each threshold event (locking a slot) is both **permanent** and **visible**. After locking a Tide slot, the player knows that one of their four cards will always be a Tide card. This creates a clear mental model: "I have 1 Tide slot, 0 other locked slots, 3 random slots." The player can plan around this structure.

Additionally, the one-lock-per-resonance cap (4 locks maximum, one per resonance type) naturally prevents the "on rails" failure mode. Even a fully committed Tide player can only lock 1 Tide slot -- the other 3 slots remain open (or lock to other resonances if the player drafts those symbols too). This is a major advantage over Proposal 3 (Escalating Slot Reservation) where a single resonance can dominate all 4 slots.

---

## Champion Deep-Dive: Lane Locking

### Example Draft Sequences

**Early Committer (Warriors/Tide-Zephyr):**
- Picks 1-2: Drafts [Tide, Zephyr] and [Tide] cards. Tide count: 5, Zephyr count: 1. Tide crosses 3 -- one slot locks to Tide.
- Picks 3-4: Drafts [Tide, Zephyr] and [Zephyr, Tide]. Tide count: 9, Zephyr count: 4. Zephyr crosses 3 -- second slot locks to Zephyr.
- Picks 5-30: Player has 1 Tide slot, 1 Zephyr slot, 2 random slots. Every pack shows at least one Tide card and one Zephyr card, with 2 random cards for variety. This is exactly the convergence target: 2+ archetype cards per pack.

**Flexible Player:**
- Picks 1-4: Drafts cards from different resonances. Ember count: 2, Tide count: 2, Stone count: 1, Zephyr count: 1. No threshold crossed yet.
- Pick 5: Drafts a [Tide, Stone] card. Tide count: 4, crosses 3. First slot locks to Tide.
- Picks 6-8: Explores further. Still only 1 locked slot, 3 random slots. Lots of variety.
- Pick 9: Stone count reaches 3. Second slot locks to Stone.
- Remainder: 1 Tide slot, 1 Stone slot, 2 random. Player building Sacrifice (Tide/Stone) or Self-Mill (Stone/Tide).

**Pivot Attempt:**
- Picks 1-3: Drafts Ember cards. Ember count: 6, locks Ember slot at pick 2.
- Pick 4-6: Realizes Ember cards are weak in the pool. Starts drafting Tide.
- Pick 7: Tide count reaches 3. Second slot locks to Tide.
- Result: 1 Ember slot, 1 Tide slot, 2 random. The Ember lock is a "sunk cost" but not catastrophic -- Ember cards still appear in that slot, and the player can build toward a deck that uses some Ember cards. The 2 random slots give flexibility. The pivot is possible but has a cost -- exactly the right incentive structure.

### Failure Modes

1. **Fourth lock irrelevance.** If a player somehow crosses threshold 3 in all four resonances, all 4 slots are locked and there are no random slots. This removes all surprise from packs. Mitigation: the threshold of 3 is high enough that crossing it in 3+ resonances requires deliberate effort, and most players will only lock 2-3 slots. A committed player will concentrate symbols and only cross 1-2 thresholds.

2. **Threshold too easy for primary resonance.** A single card with [Tide, Tide, Zephyr] gives 3 Tide symbols (primary=2, secondary=1), instantly crossing the threshold on pick 1. Mitigation: limit 3-symbol cards to ~15% of the pool. With 2-symbol cards dominant, the earliest a player can lock a slot is pick 2 (two [Tide] cards = 4 Tide symbols).

3. **Locked slot shows unwanted cards.** A player who locked Ember early but pivoted to Tide is stuck seeing an Ember card every pack. This is a mild form of "sunk cost" that is actually desirable -- it creates a cost to pivoting, which is the convergence incentive. The Ember cards may still be useful as splash.

4. **Insufficient convergence.** With only 1 locked slot per resonance, a committed Tide player sees exactly 1 guaranteed Tide card per pack. The target is 2+. Mitigation: the random slots have a ~25% chance each of showing a Tide card (since ~25% of the pool is Tide-primary), so on average the committed player sees 1 guaranteed + 0.75 random = 1.75 Tide cards. This is close to the target but may fall short. Parameter tuning (threshold value, whether primary lock gives 2 slots) can address this.

### Parameter Variants Worth Testing

1. **Threshold value: 3 vs 4 vs 5.** Lower thresholds (3) create faster lock-in and stronger convergence but risk locking too early. Higher thresholds (5) delay the "level up" moment but keep packs open longer. Recommended starting point: 3 for the first lock, possibly 5 for subsequent locks.

2. **Lock bonus: 1 slot vs 2 slots for primary resonance.** If the first resonance to lock gets 2 slots instead of 1, convergence jumps significantly. This creates a stronger commitment incentive but reduces flexibility. Worth testing: first lock = 2 slots, subsequent locks = 1 slot.

3. **Lock threshold scaling: fixed 3 vs escalating (3, 5, 8, 12).** Each subsequent resonance requires a higher threshold to lock its slot. This prevents locking all 4 slots easily and ensures later locks are meaningful commitments. Worth testing: 3 for the first lock, 5 for the second, 8 for the third, 12 for the fourth.

### Proposed Symbol Distribution for Simulation

| Symbol Count | Percentage | Cards (of 324 non-generic) |
|---|---|---|
| 0 symbols | 10% | 36 (generic) |
| 1 symbol | 25% | 81 |
| 2 symbols | 55% | 178 |
| 3 symbols | 20% | 65 |

**Rationale:** 2-symbol cards dominate so that each draft pick contributes meaningful symbol counts (3 per pick on average for 2-symbol cards: 2 for primary + 1 for secondary). This means threshold 3 is reachable by pick 2-3 for a committed player, creating the first "level up" moment early enough to feel impactful. The 25% 1-symbol cards provide archetype-focused options with concentrated primary resonance, and the 20% 3-symbol cards enable cross-resonance threshold advancement for flexible players.

(Note: 25% + 55% + 20% = 100% of the 324 non-generic cards.)
