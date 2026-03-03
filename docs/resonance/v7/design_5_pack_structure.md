# V7 Agent 5: Novel Pack Structures

## Key Takeaways

- **The fixed 4-card pack is an untested axiom.** V3--V6 all assume packs are 4 random cards with optional modifications. Changing the pack's internal structure -- how cards are selected, grouped, and presented -- is an orthogonal axis to token counters and surge mechanics.
- **Guaranteed-floor mechanisms avoid state tracking entirely.** If every pack always contains at least one card matching the player's top resonance, no tokens, thresholds, or surge cycles are needed. The pack itself adapts.
- **Structured slots create archetype disambiguation for free.** A pack with one slot drawn from the player's primary resonance pool and one from their secondary resonance pool narrows from 4 candidate archetypes to 1--2, directly attacking V7's central fitness problem.
- **Variable pack sizes trade complexity for expressiveness.** Expanding packs in response to draft state is functionally equivalent to Surge Packs but expressed as pack structure rather than token mechanics. The question is whether the structural framing enables simpler rules.
- **Pack structure changes degrade more gracefully under realistic fitness.** Because structured packs can target two resonances simultaneously (primary + secondary), they achieve partial archetype disambiguation that pure single-resonance mechanisms cannot, reducing dependence on cross-archetype card quality.
- **The simplicity bar is high.** Any pack structure must be describable in one sentence. "Split packs" and "tiered slots" risk cognitive load even if mechanically simple.

---

## Five Algorithm Proposals

### 1. Guaranteed Floor Packs

**One sentence:** Each pack always contains at least one card whose primary resonance matches the player's current top resonance; remaining slots are random.

**Technical description:** After each pick, compute the player's weighted resonance totals (same +2/+1 counting as Surge Packs). Before generating a pack, identify the top resonance. Fill slot 1 with a random card from that resonance's pool. Fill slots 2--4 randomly from the full pool. No tokens, no thresholds, no state beyond the running symbol counts. The guaranteed slot begins from pick 2 onward (pick 1 is fully random since no resonance data exists yet).

**Predicted behavior:** Under optimistic fitness, the guaranteed slot contributes ~1.0 S/A (100% S/A precision) plus 3 random slots at ~0.75 S/A = ~1.75 total. This is below the 2.0 target. Under realistic fitness, the guaranteed slot drops to ~0.75 S/A (moderate) or ~0.625 (pessimistic), yielding ~1.4--1.5 total. The mechanism is too weak as a standalone -- one guaranteed slot is insufficient. However, it requires zero state tracking beyond symbol counts, making it extremely simple.

### 2. Dual-Resonance Structured Packs

**One sentence:** Each pack has one slot filled from the player's top resonance pool, one from their second-highest resonance pool, and two slots random.

**Technical description:** Track weighted resonance totals. Before each pack (from pick 3 onward, when two resonances typically have nonzero counts), fill slot 1 with a random card whose primary resonance matches the player's highest-weighted resonance, fill slot 2 with a card matching the second-highest resonance, and fill slots 3--4 from the full pool. Picks 1--2 are fully random. If only one resonance has accumulated weight, both structured slots draw from that resonance.

**Predicted behavior:** Under optimistic fitness, the two structured slots each deliver ~1.0 S/A, and the archetype implied by the primary+secondary resonance combination is narrowed to 1--2 candidates (e.g., Tide primary + Zephyr secondary points to Warriors or Ramp). Total late-game S/A: ~2.0 + ~0.5 from random = ~2.5 optimistic. Under realistic fitness, the dual targeting provides archetype disambiguation: a Tide+Zephyr structured pack surfaces cards from both the primary and secondary resonance of Warriors, meaning ~62.5% of those cards are directly S-tier (home archetype), not just resonance-matched. Estimated moderate S/A: ~1.8--1.9. This is the key advantage -- by targeting the intersection of two resonances, the pack structure implicitly identifies the archetype.

### 3. Expanding Echo Packs

**One sentence:** Packs start at 3 cards and grow to 5 as the player's top resonance strengthens, with extra slots filled from that resonance's pool.

**Technical description:** Base pack size is 3 (all random). Track weighted resonance totals. When the top resonance total reaches 3, pack size becomes 4 (3 random + 1 resonance-matched). At 6, pack size becomes 5 (3 random + 2 resonance-matched). The resonance-matched slots always target the current top resonance, not a permanently locked one. If the player pivots, the matched slots follow the new top resonance.

**Predicted behavior:** Under optimistic fitness, this mirrors Surge Packs' ADD mechanism but with permanent expansion rather than rhythmic alternation. Late-game S/A with 2 matched slots: ~2.0 + ~0.75 random = ~2.75 optimistic. Under realistic fitness, the permanent expansion means every late pack delivers matched cards (no "normal pack" gaps), yielding more consistent but less variant delivery. Estimated moderate S/A: ~1.7. The smaller base pack (3 instead of 4) reduces early exploration slightly but also reduces early noise. Concern: permanent expansion feels like Lane Locking's determinism problem -- once at 5-card packs, there is no going back to the interesting 3-card exploration phase.

### 4. Resonance Pair Packs

**One sentence:** From pick 5 onward, each pack contains two cards from the player's top resonance and two from their secondary resonance, with both drawn from the full pool before pick 5.

**Technical description:** Picks 1--4: standard 4-card random packs. From pick 5 onward, identify the player's top two resonances by weighted count. Fill 2 slots from the top resonance's card pool and 2 slots from the secondary resonance's card pool. All four slots use random selection within their respective resonance pools. This is a hard structural switch at a fixed pick number, not a gradual transition. The resonances targeted are always the current top two, allowing pivots.

**Predicted behavior:** Under optimistic fitness, all 4 slots are resonance-matched post-pick-5, each at ~100% S/A = ~4.0 S/A. This is clearly too high and would fail M2 (early S/A ceiling), M4 (off-archetype cards), and M9 (variance). Under realistic fitness with moderate model, each slot is at ~75% S/A = ~3.0, still too deterministic. The mechanism provides too much convergence and no splash. However, a weakened variant (1 top + 1 secondary + 2 random) is essentially Proposal 2 and is promising.

### 5. Archetype Compass Packs (CHAMPION)

**One sentence:** Each pack contains one card from the player's top resonance pool, one from an adjacent resonance on the archetype circle, and two random cards, with the adjacent resonance rotating each pick.

**Technical description:** Track weighted resonance totals. Before each pack (from pick 2+), identify the player's top resonance R. Each resonance has two "neighbor" resonances on the archetype circle (e.g., Tide's neighbors are Stone and Zephyr, since Tide-primary archetypes have Stone or Zephyr as secondary). Alternate between the two neighbors each pick: odd picks use neighbor A, even picks use neighbor B. Fill slot 1 from R's pool, slot 2 from the current neighbor's pool, slots 3--4 random. Pick 1 is fully random. The neighbor rotation ensures the player sees cards from both adjacent archetypes over consecutive packs, enabling the draft to naturally reveal which adjacent archetype has the stronger card quality in this particular pool.

**Predicted behavior:** Under optimistic fitness, slot 1 (top resonance) contributes ~1.0 S/A, slot 2 (neighbor resonance, which is the secondary resonance of one of R's archetypes) contributes variable S/A depending on which archetype the player is building. For a Warriors player (Tide/Zephyr), the Zephyr-neighbor pack hits Warriors' secondary perfectly (~B-tier under optimistic, but these are actual archetype-relevant cards). Combined with the top-resonance slot, the pack provides both primary and secondary resonance coverage over a 2-pack window. Estimated optimistic S/A: ~2.0--2.2. Under moderate fitness, the neighbor slot's value depends on whether the player's archetype aligns with that neighbor. Over the alternating cycle, one neighbor aligns perfectly (secondary resonance) and the other does not, creating natural variance. Estimated moderate S/A: ~1.7--1.9. The rotation provides built-in stddev (>= 0.8) because alternate packs have different compositions. Crucially, the mechanism requires no tokens, no thresholds, and no state beyond the running symbol counts.

---

## Champion Selection: Archetype Compass Packs

**Justification:** Archetype Compass Packs is selected as champion for four reasons:

1. **Zero state tracking beyond symbol counts.** No tokens, no thresholds, no surge cycles. The pack structure is entirely determined by the player's current top resonance and a simple odd/even pick counter. This is simpler than Surge Packs.

2. **Natural archetype disambiguation.** By showing the player's top resonance plus a rotating neighbor, the mechanism implicitly tests both adjacent archetypes on the circle. Over two consecutive packs, the player sees cards relevant to all four resonances touching their position on the circle. This addresses V7's central fitness problem: under realistic models where cross-archetype cards are often B/C tier, the player can naturally gravitate toward whichever neighbor provides better actual card quality.

3. **Built-in variance.** The alternating neighbor creates a natural 2-pick rhythm: one pack emphasizes the clockwise neighbor, the next the counterclockwise. This produces variance without requiring surge/normal alternation.

4. **Graceful fitness degradation.** Under realistic fitness, the neighbor slot sometimes provides A-tier cards and sometimes B/C. But the player sees both neighbors over two packs and drafts from whichever offers better cards. This self-correcting behavior means the mechanism degrades more slowly than pure single-resonance targeting.

---

## Champion Deep-Dive: Archetype Compass Packs

### Example Draft Sequences

**Sequence 1: Early-committing Warriors player**
- Pick 1: Random pack [Storm card, Ramp card, generic, Sacrifice card]. Player takes the Ramp card (Zephyr primary). Top resonance: Zephyr.
- Pick 2: Slot 1 = Zephyr card (Flash-home). Slot 2 = Ember card (neighbor A: Zephyr's clockwise neighbor). Slots 3--4 random. Player takes a Warriors card from random slot. Top resonance shifts to Tide.
- Pick 3: Slot 1 = Tide card (Warriors-home, S-tier). Slot 2 = Stone card (neighbor A of Tide). Slots 3--4 random. Player takes the Tide card. Tide solidifies.
- Pick 4: Slot 1 = Tide card. Slot 2 = Zephyr card (neighbor B of Tide -- this is Warriors' secondary!). Slots 3--4 random. The Zephyr card is B-tier for Warriors. Player takes the Tide S-tier card.
- Pick 5: Slot 1 = Tide. Slot 2 = Stone (Sacrifice territory, not Warriors). Player may take from random slots or the Tide slot. The Stone-neighbor pack is "off" for Warriors, creating natural variance.
- Picks 6+: Alternating Tide+Stone / Tide+Zephyr packs. The Tide+Zephyr packs are strong for Warriors (both resonances align). The Tide+Stone packs are weaker (Stone is Sacrifice territory). This creates the variance the system needs.

**Sequence 2: Signal-reading player**
- Picks 1--3: Random-heavy packs reveal strong Ember cards. Player tentatively accumulates Ember.
- Pick 4: Slot 1 = Ember. Slot 2 = Zephyr (neighbor, pointing toward Blink). Good Blink card appears. Player takes it.
- Pick 5: Slot 1 = Ember. Slot 2 = Stone (neighbor, pointing toward Storm). Weak Storm cards available. Player takes the Ember card instead.
- Pick 6+: The alternating Ember+Zephyr / Ember+Stone pattern reveals that Blink cards are consistently stronger. Player converges on Blink naturally, guided by the compass rotation.

### Failure Modes

1. **Top resonance oscillation.** If the player's top resonance flips between two resonances for several picks, the structured slots target different pools each time, reducing convergence. Mitigation: weighted counting (+2 primary) creates momentum that resists oscillation after 3--4 picks of consistent drafting.

2. **Neighbor misalignment.** For some archetypes, one neighbor is strongly aligned (secondary resonance) and the other is weakly aligned (tertiary or irrelevant). If the "wrong" neighbor appears in a critical pack, the player gets a weak structured slot. Mitigation: the alternation ensures the "right" neighbor appears every other pack, and the 2 random slots provide fallback options.

3. **Late-game staleness.** After pick 15+, the alternating pattern becomes predictable. The player knows that odd packs will have one composition and even packs another. Mitigation: the 2 random slots provide unpredictability, and the mechanism naturally produces >= 0.8 stddev.

### Parameter Variants

**Variant A (Conservative): 1+1+2 structure.**
Slot 1 = top resonance, slot 2 = rotating neighbor, slots 3--4 = random. This is the base proposal. Predicted S/A: ~2.0 optimistic, ~1.7--1.9 moderate.

**Variant B (Aggressive): 2+1+1 structure.**
Slots 1--2 = top resonance, slot 3 = rotating neighbor, slot 4 = random. More convergence, less exploration. Predicted S/A: ~2.4 optimistic, ~1.9--2.1 moderate. Risk of failing M4 (off-archetype) and M9 (variance) if too many slots are locked.

**Variant C (Delayed): Random until pick 4, then 1+1+2.**
Fully random packs for picks 1--3, structured packs from pick 4 onward. Ensures early exploration (M1) is preserved. Predicted behavior similar to Variant A but with better M1/M2 scores and slightly later convergence.

### Proposed Fitness Models for Testing

**Model 1 (Optimistic):** Adjacent-primary cards are 100% A-tier. Compass Packs should score ~2.0--2.2 S/A. This tests whether the mechanism's ADD component (1 guaranteed resonance-matched slot) is sufficient to cross 2.0.

**Model 2 (Moderate, primary test):** Adjacent-primary cards are 50% A / 30% B / 20% C. The compass rotation should reveal which neighbor provides genuine A-tier cards, allowing the player to preferentially draft from good-neighbor packs. Predicted S/A: ~1.7--1.9. The key metric is whether the alternating structure creates enough information for the archetype-committed player to self-correct.

**Model 3 (Secondary-aware):** Same as moderate, but additionally model the secondary resonance: cards whose secondary resonance matches the player's secondary are B+ tier (upgraded from B). This tests whether the compass's neighbor slot, which often aligns with the player's secondary resonance, captures this upgrade. If so, Compass Packs gain ~0.1--0.2 S/A from the secondary resonance alignment that pure primary-resonance mechanisms miss.

The core hypothesis to test: **Does the rotating neighbor slot provide enough archetype disambiguation to outperform single-resonance targeting under realistic fitness?** If the answer is yes, Compass Packs should degrade from ~2.0 (optimistic) to ~1.8 (moderate) rather than Surge Packs' ~2.0 to ~1.6 -- a significantly shallower degradation slope. That robustness advantage, even at slightly lower peak performance, would make Compass Packs the superior choice for real-world deployment.
