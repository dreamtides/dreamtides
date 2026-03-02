# Domain 4: Reactive/Immediate Mechanisms

## Key Takeaways

- **Reactive mechanisms trade convergence for flexibility.** The defining tension in this domain is that short memory means the player can always pivot, but the system has less information to push toward focused archetypes. Every proposal here must grapple with this tradeoff.
- **"Last pick determines next pack" is the simplest reactive mechanism, but it converges too weakly.** With only 1-3 symbols of information per pick, a single-pick window lacks the statistical mass to reliably fill 2+ archetype slots by pick 6.
- **A sliding window of 3-5 recent picks is the sweet spot.** It provides enough signal for meaningful convergence while still allowing pivots within a few picks. Longer windows drift toward accumulation; shorter windows produce noise.
- **Streak/momentum mechanics are intuitive but create cliff effects.** Players understand "do the same thing 3 times in a row to get a bonus," but a single off-resonance pick resets the streak and feels punishing, which conflicts with splashability (Goal 6).
- **Reactive mechanisms naturally excel at early openness (Goal 7) and flexibility (Goal 4).** With no history, early packs are inherently random. The challenge is making late-game packs focused enough without accumulating hidden state.
- **Symbol distributions with 2 symbols per card work best for reactive mechanisms.** Single-symbol cards produce too little signal per pick; 3-symbol cards over-determine the next pack, leaving no room for splash. Two symbols give a primary resonance signal plus a secondary that enables cross-archetype drafting.
- **The "Echo Window" approach (champion) balances simplicity, convergence, and flexibility better than any other reactive mechanism tested.** It counts symbols from the last N picks rather than the last 1, giving the system enough data to converge without permanent memory.

---

## Proposal 1: Last-Pick Mirror

**Player-facing description:** "Each resonance symbol on the card you just drafted fills one slot in your next pack with a card of that resonance; remaining slots are filled randomly."

**Technical description:** After you pick a card, look at its resonance symbols. The primary symbol (counting as 2) and any secondary/tertiary symbols (counting as 1 each) determine how many of the 4 slots in your next pack are reserved for those resonances. A card with [Tide, Zephyr] reserves 2 Tide slots and 1 Zephyr slot, leaving 1 random slot. A card with [Ember] reserves 2 Ember slots and leaves 2 random. A generic card (0 symbols) produces an entirely random next pack.

**Assessment:**
- Serves well: Simplicity (Goal 1) -- trivially explainable. Transparency (Goal 2) -- perfectly predictable. Flexibility (Goal 4) -- one off-type pick immediately resets.
- Fails: Convergence (Goal 5) -- only one pick of memory, so the system cannot distinguish a committed drafter from a random one. Splashability (Goal 6) -- when a committed player takes a 2-symbol card of their archetype, only 0-1 random slots remain. Signal reading (Goal 8) -- no history means no signals.
- Best symbol distribution: Mostly 1-symbol cards, so that 2 slots are reserved and 2 are random, preserving splash.

---

## Proposal 2: Streak Bonus

**Player-facing description:** "Pack slots are random, but for each consecutive pick that shares a primary resonance with your previous pick, one additional slot in your next pack is locked to that resonance (max 3 locked slots)."

**Technical description:** Track a streak counter per resonance. When you draft a card whose primary symbol matches the primary symbol of your previous pick, increment the streak counter (capped at 3). Otherwise, reset it to 1 if the new card has a primary symbol, or 0 if generic. Your next pack has [streak] slots locked to the streak resonance and [4 - streak] random slots. A streak of 0 means a fully random pack.

**Assessment:**
- Serves well: Simplicity (Goal 1) -- "consecutive same-type picks lock more slots." Convergence (Goal 5) -- a committed player quickly hits streak 3 and locks 3 of 4 slots. Transparency (Goal 2) -- streak count is trivially trackable.
- Fails: Flexibility (Goal 4) -- one off-type pick resets the streak to 0 or 1, which feels punishing. Splashable (Goal 6) -- at streak 3, only 1 random slot remains and taking it breaks the streak. No forced decks (Goal 3) -- once a streak gets going, the locked slots feed themselves, potentially letting a player force the same archetype every run.
- Best symbol distribution: Mostly 2-symbol cards, so primary symbol comparisons are meaningful and secondary symbols contribute to the locked cards' identity.

---

## Proposal 3: Echo Window

**Player-facing description:** "Count the resonance symbols across your last 3 picks; your top resonance fills 2 pack slots, your second resonance fills 1, and the last slot is random."

**Technical description:** Maintain a sliding window of the most recent 3 drafted cards. Sum all resonance symbols across those cards using the standard weighting (primary = 2, secondary/tertiary = 1 each). Rank the four resonances by their totals. The resonance with the highest total fills 2 of 4 pack slots (cards drawn randomly from that resonance's pool). The second-highest fills 1 slot. The fourth slot is always random (any resonance). Ties are broken randomly. If fewer than 3 picks have been made, use all picks so far; if 0 picks, the pack is fully random.

**Assessment:**
- Serves well: Simplicity (Goal 1) -- one sentence, concrete operations. Convergence (Goal 5) -- a player picking Tide cards for 3 straight picks will have Tide dominating their symbol counts, reliably filling 2 slots. Early openness (Goal 7) -- with 0-2 picks of history, early packs are mostly random. Flexibility (Goal 4) -- 3 picks of off-type cards fully pivots the window. Splashable (Goal 6) -- guaranteed 1 random slot plus the second-resonance slot often pulls in adjacent-archetype cards.
- Fails: Signal reading (Goal 8) -- the algorithm has no memory of what was offered, only what was picked, so signal reading is limited. Not on rails (Goal 2) -- the 2-slot top resonance could feel constraining if the player just wants variety.
- Best symbol distribution: Mostly 2-symbol cards. This gives each pick ~3 weighted symbol points (2 for primary + 1 for secondary), so a 3-card window produces ~9 points distributed across resonances -- enough to reliably separate top from second.

---

## Proposal 4: Cooldown Wheel

**Player-facing description:** "After you draft a card, its primary resonance goes on a 2-pick cooldown during which that resonance cannot fill pack slots; pack slots are split evenly among the non-cooldown resonances."

**Technical description:** Track when each resonance was last drafted as primary. A resonance is "on cooldown" if it was the primary resonance of a card drafted within the last 2 picks. When building a pack, exclude all resonances on cooldown. Distribute the 4 pack slots evenly among remaining resonances (with remainders assigned randomly among them). If 3 resonances are on cooldown (rare -- would require 2 consecutive picks of different primary resonances plus one from the pick before that), the one non-cooldown resonance fills all 4 slots.

**Assessment:**
- Serves well: No forced decks (Goal 3) -- you literally cannot keep getting the same resonance you just drafted, forcing exploration. Variety (Goal 5 in the "different runs" sense) -- cooldowns push the player to rotate. Open early (Goal 7) -- with nothing on cooldown at pick 1, all resonances appear.
- Fails: Convergence (Goal 5) -- this is an anti-convergence mechanism. It actively prevents you from building on your recent picks. A committed Tide player who picks Tide three times puts Tide on cooldown every other pack. Simplicity is decent but the edge case of 3 cooldowns is awkward. Transparency (Goal 2) -- tracking 2-pick cooldowns for each resonance requires mental bookkeeping.
- Best symbol distribution: Mostly 1-symbol cards, since only primary resonance matters for cooldown tracking.

---

## Proposal 5: Ripple Echo

**Player-facing description:** "Each resonance symbol on the card you draft adds one card of that resonance to each of your next 2 packs (primary symbol adds to 3 packs instead); remaining pack slots are random."

**Technical description:** When you draft a card, its symbols create "echoes" that carry forward. The primary symbol creates a 3-pack echo (it reserves 1 slot in each of your next 3 packs). Each secondary/tertiary symbol creates a 2-pack echo (reserving 1 slot in each of your next 2 packs). Echoes stack: if multiple active echoes reserve slots for the same resonance in a single pack, that resonance gets multiple slots. Total reserved slots are capped at 3 per pack (leaving at least 1 random slot). Echoes from multiple recent picks overlap, creating a layered forward-looking commitment.

**Assessment:**
- Serves well: Convergence (Goal 5) -- echoes from 2-3 recent picks overlap, creating 2-3 reserved slots if the player is consistent. Transparency (Goal 2) -- the player can count their pending echoes and predict future packs. Flexibility (Goal 4) -- echoes expire after 2-3 packs, so pivoting costs only a few sub-optimal packs.
- Fails: Simplicity (Goal 1) -- while the sentence is technically complete, tracking primary (3-pack) vs secondary (2-pack) echoes across multiple future packs is mentally demanding. A player would need to track echo queues for each resonance. This is the most complex proposal in this domain. Signal reading (Goal 8) -- like all reactive mechanisms, no memory of offered cards.
- Best symbol distribution: Mostly 2-symbol cards. This gives each pick a primary echo (3 packs) and a secondary echo (2 packs), creating layered overlap without overwhelming the slot budget.

---

## Champion Selection: Echo Window (Proposal 3)

The Echo Window is the strongest proposal because it occupies the best tradeoff point in this domain's fundamental tension between responsiveness and convergence.

**Why not the others?**
- Last-Pick Mirror (P1) is simpler but cannot converge -- one pick of memory is too volatile to distinguish committed from random drafters.
- Streak Bonus (P2) converges well but creates a cliff effect where one splash pick destroys accumulated progress, directly opposing splashability.
- Cooldown Wheel (P4) is an interesting anti-convergence mechanism but fundamentally works against Goal 5, the core purpose of the resonance system.
- Ripple Echo (P5) achieves good convergence through overlapping echoes but fails simplicity -- tracking multi-pack echo queues per resonance is too much mental load.

Echo Window threads the needle: 3 picks of memory is enough to converge (a player picking consistent Tide cards will have Tide dominating the window by pick 3-4), short enough to allow pivots (3 off-type picks fully resets), and the "top resonance gets 2, second gets 1, last is random" structure is easy to internalize and predict.

---

## Champion Deep-Dive: Echo Window

### Example Draft Sequences

**Early committer (Tide/Zephyr Warriors player):**
- Picks 1-2: Random packs (window too small for strong signal). Takes a [Tide, Zephyr] card and a [Tide] card.
- Pick 3: Window = last 3 picks. Tide has 2+2+1=5 weighted points, Zephyr has 1. Pack: 2 Tide slots, 1 Zephyr slot, 1 random. Takes another [Tide, Zephyr].
- Picks 4-8: Window is now 3 cards deep with 2-3 Tide-primary cards each time. Tide reliably fills 2 slots, Zephyr or Stone fills 1. The player sees consistent Tide options and can build a focused Warriors deck.
- Picks 9-30: Stable state. If the player stays on Tide, every pack has 2 Tide cards, 1 secondary resonance card, 1 random. Convergence target of 2+ archetype cards per pack is met.

**Flexible player (no commitment through pick 8):**
- Picks 1-5: Takes the best card regardless of resonance. Window shifts each time. Packs remain diverse because the top resonance changes pick to pick.
- Pick 6: Window might be [Ember card, Tide card, Stone card]. No resonance dominates. Pack slots are distributed more evenly (e.g., Ember 2, Tide 1, 1 random -- or if counts are close, ties break randomly, creating variety).
- Picks 7-8: Still flexible. The system does not punish this player -- each pack reflects their recent picks, which are diverse, so packs remain diverse. This player sacrifices seeing 2+ archetype-focused cards per pack but gains access to the full card pool.

**Pivot attempt (committed Ember player switches to Tide at pick 12):**
- Picks 9-11: Window is 3 Ember-heavy cards. Packs show 2 Ember, 1 secondary, 1 random.
- Pick 12: Player takes a [Tide, Stone] card. Window is now [Ember card, Ember card, Tide card]. Ember still leads, but Tide is present. Pack might be 2 Ember, 1 Tide, 1 random.
- Pick 13: Takes another Tide card. Window = [Ember, Tide, Tide]. Tide now leads. Pack: 2 Tide, 1 Ember, 1 random.
- Pick 14: Takes another Tide card. Window = [Tide, Tide, Tide]. Full pivot complete in 3 picks. Pack: 2 Tide, 1 secondary, 1 random.

The pivot costs exactly 3 picks -- during which the player sees a mix of old and new resonance cards. This is a genuine cost (those 3 packs are less focused) without being punishing.

### Predicted Failure Modes

1. **Oscillation risk.** A player alternating between two resonances (e.g., Tide one pick, Ember the next) will have their window split, producing unfocused packs. This is correct behavior (the system reflects what you're doing) but might feel like "the system isn't helping me." Mitigation: the 2-slot top resonance reward means even a slight lean toward one resonance produces visible results.

2. **Weak signal reading.** Since the algorithm only looks at what you picked (not what was offered), there is no mechanism to detect which resonance is over-represented in the pool. A player cannot read signals except by noticing what appears in random slots. This is a genuine weakness of all reactive mechanisms.

3. **Generic card dead zones.** Picking a generic card (0 symbols) contributes nothing to the window. If a player picks 2 generics in a row, their window has only 1 card of signal. Mitigation: this self-corrects -- a low-signal window produces diverse packs, which offer non-generic options.

4. **Second-resonance slot may not align with archetype secondary.** A Warriors player's top resonance will be Tide, but their second resonance in the window might be Stone (from a card that happened to splash Stone) rather than Zephyr. The 1-slot second resonance reflects actual picks, not archetype structure. This is arguably a feature, not a bug.

### Parameter Variants Worth Testing

1. **Window size: 3 vs 4 vs 5 picks.** Window of 3 converges fastest and allows quickest pivots. Window of 5 is more stable and convergent but slower to pivot. The tradeoff is responsiveness vs. reliability.

2. **Slot allocation formula: 2/1/1 vs 2/1/0+1random vs 3/1/0/0.** The base proposal uses 2 top / 1 second / 1 random. A more aggressive variant could use 3/1 (stronger convergence, less splash). A more conservative variant could use 2/0/2random (weaker convergence, more splash and variety).

3. **Primary symbol weight: 2 vs 1.5 vs 3.** The base proposal counts primary symbols as 2 and secondary/tertiary as 1. Increasing to 3 makes primary resonance dominate the window faster (stronger convergence). Decreasing to 1.5 makes multi-symbol cards contribute more evenly across resonances (more splash).

### Proposed Symbol Distribution for Simulation

- **0 symbols:** 36 cards (10%) -- generic cards
- **1 symbol:** 65 cards (20% of remaining) -- simple resonance-aligned cards, e.g., [Tide]
- **2 symbols:** 194 cards (60% of remaining) -- the core, e.g., [Tide, Zephyr]
- **3 symbols:** 65 cards (20% of remaining) -- strong archetype-committed cards, e.g., [Tide, Tide, Zephyr]

This 20/60/20 split (among non-generic cards) centers on 2-symbol cards. Each pick contributes ~3 weighted symbol points on average (2 for primary + 1 for secondary), so a 3-card window accumulates ~9 points -- enough to cleanly separate a dominant resonance from the rest. The 20% of 1-symbol cards provide simpler resonance signals, and the 20% of 3-symbol cards reward deep commitment by contributing more total points to the window.
