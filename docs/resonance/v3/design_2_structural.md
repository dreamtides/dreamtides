# Domain 2: Structural/Guaranteed Mechanisms

## Key Takeaways

- **Structural guarantees are the most transparent domain.** When players know the exact rules for what goes in each slot, they can predict pack composition before seeing it. No other domain offers this level of certainty.
- **The tension is between predictability and variety.** Pure structural guarantees (e.g., "one card per resonance") are maximally transparent but create rigid, samey packs. The best algorithms introduce controlled variation within a guaranteed frame.
- **Slot assignment is the natural lever.** With 4 cards per pack and 4 resonance types, the mapping from slots to resonances is the core design space. Algorithms differ in how that mapping changes over the draft.
- **Convergence through slot ownership, not randomness.** The structural approach achieves convergence by giving the player's committed resonance more guaranteed slots, not by increasing probability. This is binary (you get the slot or you don't), which makes it legible but coarse.
- **The "Rotating Wheel" algorithm is the strongest candidate.** It provides a fixed, predictable rotation that the player can learn to read, while still creating natural convergence as drafted symbols shift which resonance "owns" which slot.
- **Symbol distribution matters less here than in accumulation-based systems.** Because structural algorithms assign slots deterministically, the key question is how symbols determine slot ownership, not how fast they accumulate. Two-symbol cards (primary + secondary) are the sweet spot: enough information to differentiate archetypes without overwhelming the counting.
- **Signal reading emerges naturally from structural guarantees.** When the player knows the pack rule, observing which specific cards appear in each guaranteed slot reveals information about pool composition (which archetypes are deep vs. shallow).

---

## Proposal 1: One-of-Each

**Player-facing description:** "Every pack contains exactly one card from each of the four resonance types: Ember, Stone, Tide, and Zephyr."

**Technical description:** For each pack, the algorithm selects one random card whose primary symbol is Ember, one whose primary is Stone, one whose primary is Tide, and one whose primary is Zephyr. Generic cards (0 symbols) replace one random resonance slot. Card selection within each resonance slot is uniformly random from all cards of that primary resonance.

**Design goal assessment:**
- Excellent: Simple (1), Transparent (2), Open-ended early (7), Splashable (6) -- every pack always shows all 4 resonances
- Poor: Convergent (5), Not on rails paradoxically becomes "always the same structure" -- packs never change based on player choices, so there is no convergence mechanism at all
- Fails: No forced decks (3) is weak because every run looks the same structurally

**Best symbol distribution:** Does not matter much -- only primary symbol is used for slot assignment. 1-symbol cards work fine.

---

## Proposal 2: Majority Rules

**Player-facing description:** "Count your drafted resonance symbols (primary counts as 2, others count as 1); your highest-count resonance fills 2 of 4 pack slots, your second-highest fills 1, and the last slot is a random resonance."

**Technical description:** After each pick, tally all resonance symbols the player has drafted using the standard weighting (primary=2, secondary/tertiary=1). The resonance with the highest count fills 2 pack slots (two random cards with that primary resonance). The second-highest fills 1 slot. The 4th slot draws from a uniformly random resonance. Ties are broken randomly. Each slot is filled by selecting a random card from the pool whose primary symbol matches the assigned resonance.

**Design goal assessment:**
- Excellent: Simple (1), Convergent (5), Transparent (2) -- player can compute their counts and predict pack structure
- Good: Splashable (6) -- the random slot and second-resonance slot provide off-archetype cards
- Poor: Open-ended early (7) -- after just 1-2 picks, the majority resonance already dominates 2 slots. Very early lock-in.
- Poor: Flexible archetypes (4) -- hard to pivot because accumulated counts create inertia
- Moderate: No forced decks (3) -- first few picks determine direction, but randomness in the last slot helps

**Best symbol distribution:** 2-symbol cards (primary + secondary). This ensures the secondary resonance count grows alongside the primary, giving the "second-highest" slot meaningful content and supporting archetype-pair drafting rather than single-resonance drafting.

---

## Proposal 3: Rotating Wheel

**Player-facing description:** "The 4 pack slots cycle through a fixed resonance order (Ember, Stone, Tide, Zephyr); each pick, the wheel advances by one, and any slot matching your most-drafted resonance is duplicated, replacing the slot opposite it on the wheel."

**Technical description:** There is a fixed wheel of resonance assignments for the 4 slots: [Ember, Stone, Tide, Zephyr]. Each pick, the wheel rotates by 1 position (so pick 1 shows E/S/T/Z, pick 2 shows S/T/Z/E, pick 3 shows T/Z/E/S, etc.). After rotation, if the player has a clear majority resonance (strictly more weighted symbols than any other), any slot that matches that resonance is duplicated into the slot directly opposite it on the wheel (Ember<->Tide, Stone<->Zephyr). If no majority exists, no duplication occurs. Each slot is filled with a random card whose primary resonance matches the assigned resonance.

**Design goal assessment:**
- Excellent: Simple (1), Transparent (2) -- the rotation is perfectly predictable, the duplication rule is one clear conditional
- Good: Convergent (5) -- once you have a majority, you get 2 slots of your resonance every time your resonance appears on the wheel, and 0 extra when it does not. Over 4-pick cycles, this averages to ~1.5 committed slots.
- Good: Open-ended early (7) -- no majority means no duplication, so early packs always show all 4 resonances
- Good: Signal reading (8) -- knowing which resonance fills which slot lets the player observe pool depth
- Moderate: Splashable (6) -- the non-duplicated slots always show other resonances
- Moderate: Varied (5) -- the rotation is deterministic, so pack structure repeats every 4 picks (mitigated by random card selection within slots)

**Best symbol distribution:** 2-symbol cards. The majority calculation needs enough signal to establish a lead, but not so much that it locks in after 2 picks. With 2 symbols per card (primary=2, secondary=1), a player needs ~3-4 picks of the same primary resonance to establish a clear majority.

---

## Proposal 4: Claim Slots

**Player-facing description:** "Each pack has 4 slots; whenever you draft a card, its primary resonance permanently claims one unclaimed slot, and future packs always fill claimed slots with cards of that resonance and unclaimed slots randomly."

**Technical description:** The player starts with 4 unclaimed slots. When they draft a card with a primary resonance, if there is any unclaimed slot, one unclaimed slot becomes permanently claimed by that resonance. If all slots are already claimed, the pick has no structural effect. Multiple slots can be claimed by the same resonance. For each pack, claimed slots are filled with a random card of the claimed resonance, and unclaimed slots are filled with a card of a uniformly random resonance.

**Design goal assessment:**
- Excellent: Simple (1) -- extremely easy to explain and predict
- Excellent: Transparent (2) -- player knows exactly which slots are claimed
- Good: Convergent (5) -- committing to one resonance claims all 4 slots by pick 4
- Poor: Open-ended early (7) -- slots get claimed immediately, so the window of flexibility is only picks 1-4
- Very Poor: Flexible archetypes (4) -- once claimed, slots are permanent. No pivoting.
- Very Poor: Not on rails (2) -- by pick 5, the pack structure is locked forever. The remaining 25 picks have zero structural agency.
- Poor: Varied (5) -- the endgame packs are always the same structure

**Best symbol distribution:** Only primary symbol matters, so 1-symbol cards are sufficient. But 2-symbol cards could enrich the "which resonance claims which slot" decision if expanded to include secondary resonances.

---

## Proposal 5: Balanced Quartet with Weighted Draw

**Player-facing description:** "Every pack has one slot per resonance, but cards in each slot are drawn from a quality tier based on your symbol count in that resonance: 0-2 symbols draws from commons, 3-5 from uncommons, 6-9 from rares, and 10+ from legendaries."

**Technical description:** Every pack always contains exactly one card per resonance (like One-of-Each). However, within each resonance slot, the rarity pool from which the card is drawn depends on the player's accumulated symbol count for that resonance (primary=2, secondary/tertiary=1). At 0-2 symbols, the slot draws from commons only. At 3-5, uncommons. At 6-9, rares. At 10+, legendaries. If no card of the required rarity exists for a resonance, fall back to the next lower tier.

**Design goal assessment:**
- Good: Simple (1) -- the base rule is simple, but the 4-tier threshold table adds complexity. Borderline on the Simplicity Test.
- Excellent: Transparent (2) -- player knows pack structure (one per resonance) and can compute their tier in each resonance
- Good: Convergent (5) -- investing in a resonance yields higher-rarity cards, which are presumably stronger
- Excellent: Splashable (6) -- always see all 4 resonances
- Good: Open-ended early (7) -- early packs are all commons, so no resonance feels "better" yet
- Good: Signal reading (8) -- observing which rarity tiers are deep reveals pool information
- Moderate: Varied (5) -- pack structure is always 1/1/1/1, only rarity changes. Could feel monotonous.
- Weakness: Convergence is through card quality, not card quantity. A committed player still sees only 1 card of their resonance per pack -- it is just a better card. This may not feel like enough convergence to hit the "2+ archetype cards per pack" target.

**Best symbol distribution:** 2-symbol cards (primary + secondary). With primary=2 and secondary=1, each pick adds 3 total symbols. Over 5 picks of the same archetype, a player accumulates ~10 symbols in their primary resonance and ~5 in secondary, unlocking legendaries in primary and uncommons in secondary. This creates a satisfying progression curve.

---

## Champion Selection: Rotating Wheel

The **Rotating Wheel** is the most promising structural algorithm for several reasons:

1. **Genuine simplicity.** The one-sentence description fully specifies the algorithm. A player can mentally track the rotation and predict whether their next pack will have a duplicated slot.

2. **Natural convergence curve.** Early picks (no majority) produce balanced packs. Mid-draft (majority established around pick 4-6) starts producing occasional double-slots. Late draft (strong majority) consistently doubles. This matches the desired convergence timeline of picks 5-8.

3. **Pivot-friendly.** Because the majority is recalculated each pick, a player who shifts resonances will eventually shift which resonance gets duplicated. The accumulated counts create some inertia (desirable -- commitment should matter), but not permanent lock-in.

4. **Signal reading.** The deterministic rotation means the player knows which resonance fills each slot. If the Tide slot keeps showing weak cards, the Tide pool might be shallow -- a signal to draft elsewhere.

5. **Balanced between the extremes.** Claim Slots locks in too fast. Majority Rules converges too aggressively. One-of-Each never converges. Balanced Quartet puts convergence on the wrong axis (quality not quantity). Rotating Wheel hits the middle ground.

---

## Champion Deep-Dive: Rotating Wheel

### Example Draft Sequences

**Early Committer (Warriors/Tide-Zephyr):**
- Picks 1-3: Drafts [Tide, Zephyr], [Tide], [Tide, Zephyr]. Symbol counts: Tide=9, Zephyr=3, others=0. Clear Tide majority.
- Pick 4: Wheel position is [Zephyr, Ember, Stone, Tide]. Tide slot exists -> duplicated into the Stone slot (opposite). Pack becomes [Zephyr, Ember, Tide, Tide]. Player sees 2 Tide cards, 1 Zephyr, 1 Ember. Hits the 2+ archetype cards target.
- Pick 8: Wheel position is [Zephyr, Ember, Stone, Tide] again (cycle repeats). Same duplication. Player consistently sees 2 Tide cards every time Tide appears in the rotation.
- Over a 4-pick cycle: 2 packs have a Tide slot (get duplicated to 2 Tide cards each), 2 packs have no Tide slot (no duplication, see 1 of each other resonance). Average Tide cards per pack: (2+2+0+0)/4 = 1.0. Hmm -- this undershoots the 2+ target. This is a predicted failure mode (see below).

**Flexible Player:**
- Picks 1-5: Drafts cards from 3 different resonances. No clear majority. No duplication occurs. All packs show one of each resonance type. Player sees maximum variety, hits the "3+ unique resonances per pack" early target easily.
- Pick 6: Player starts focusing on Ember. After 2-3 focused picks, Ember takes the lead. Duplication begins.
- Pick 10 onward: Ember majority is solid. Duplication fires whenever Ember slot appears on the wheel.

**Pivot Attempt:**
- Picks 1-7: Drafts heavily into Stone. Stone count = ~18, others low.
- Picks 8-12: Starts drafting Tide. By pick 12, Tide count might be ~12, Stone still at ~18.
- The pivot is slow because Stone's accumulated lead takes many picks to overcome. This is appropriate -- pivoting should be possible but costly.
- Around pick 14-16: If the player drafts Tide exclusively, Tide could overtake Stone. Duplication switches to Tide. The pivot succeeds but cost ~8 picks of suboptimal packs.

### Predicted Failure Modes

1. **Average convergence may be too low.** With 4-pick rotation cycles, the committed resonance only appears in 1 of 4 wheel positions. Even with duplication, the average is 1.0 committed-resonance cards per pack across the cycle, not 2.0. This misses the convergence target. **Mitigation:** Increase duplication power -- instead of replacing 1 opposite slot, replace 2 slots, or allow the majority resonance to appear in 2 wheel positions.

2. **Rotation is deterministic and could feel mechanical.** Players will learn the pattern and know exactly when their "good" packs come. This is transparent (good) but might feel like alternating between exciting and boring packs (bad). **Mitigation:** Add a small random offset to the starting wheel position each draft run.

3. **Tie-breaking matters more than expected.** With 4 resonances, ties in symbol counts are common in the first 3-5 picks. The "no majority, no duplication" rule handles this cleanly but means convergence cannot start until one resonance pulls ahead. With 2-symbol cards, this could take 4-5 picks (correct timing) or 2 picks (too fast) depending on draft pattern.

### Parameter Variants Worth Testing

**Variant A: Double Duplication.** When the majority resonance appears on the wheel, it replaces TWO other slots instead of one. Average committed cards per pack jumps from 1.0 to 1.5 (3 of 4 slots on majority turns, 0 of 4 on off turns). One-sentence: "The 4 pack slots cycle through E/S/T/Z each pick; any slot matching your top resonance takes over two other slots."

**Variant B: Adjacent Duplication.** Instead of replacing the opposite slot, the majority resonance replaces the adjacent slot on the wheel. This means on majority turns, you see 2 committed + 2 others (same as base), but the "replaced" resonance is always adjacent on the resonance circle, meaning the replaced slot's cards are naturally closer to the player's archetype. The structural effect is the same but the flavor differs.

**Variant C: Majority + Runner-Up.** The majority resonance duplicates into one slot. The second-highest resonance (the "runner-up") also gets a guaranteed slot on every pack, regardless of wheel position. This ensures archetype-pair support. One-sentence: "Slots cycle E/S/T/Z each pick; your top resonance duplicates one slot, and your second resonance always gets one guaranteed slot regardless of the wheel."

### Proposed Symbol Distribution for Simulation

- **1-symbol cards: 30%** (~97 cards). These are single-resonance focused cards, e.g., [Tide]. Simple, strong signal for one resonance.
- **2-symbol cards: 55%** (~178 cards). These are the archetype-pair cards, e.g., [Tide, Zephyr] for Warriors. These are the backbone -- they build both primary and secondary resonance counts.
- **3-symbol cards: 5%** (~16 cards). Rare multi-resonance cards, e.g., [Tide, Zephyr, Stone]. These bridge distant archetypes.
- **0-symbol cards: 10%** (~36 cards). Generic cards, as specified.

This distribution ensures that the majority resonance is established primarily through primary symbols (which count as 2), making the wheel duplication trigger around pick 4-6 for committed players. The 55% two-symbol share ensures secondary resonances grow meaningfully, supporting the runner-up variant and archetype-pair drafting.
