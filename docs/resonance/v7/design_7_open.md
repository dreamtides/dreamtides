# Agent 7: Open Exploration -- Algorithm Design (V7)

## Key Takeaways

1. **The core problem is wrong-archetype pollution, not insufficient targeting.** V3-V6 algorithms already deliver resonance-matched cards reliably. The issue is that half those cards belong to the wrong archetype sharing that resonance. Under realistic fitness, this cuts S/A precision from 100% to 62-75%. The solution space should focus on *filtering out wrong-archetype cards* rather than *adding more resonance-matched cards*.

2. **Draft history contains archetype signal that no algorithm has exploited.** The sequence of picks encodes far more than cumulative resonance counts. A player who picks a Tide card with a Zephyr secondary symbol, then another Tide-Zephyr card, is almost certainly Warriors -- not Sacrifice. No V3-V6 algorithm uses pick-sequence patterns.

3. **Card rejection is an unexplored mechanism class.** Every prior algorithm focuses on what to ADD to packs. None systematically REMOVE cards. Removing a card known to be wrong-archetype from a random pack is worth as much as adding a right-archetype card, but operates in a completely different design space.

4. **The "meaningful choices" reframe has concrete value.** If the algorithm cannot guarantee 2.0 S/A under realistic fitness, it can instead ensure that every pack contains at least one genuine decision -- two cards that are both plausible for the player's emerging archetype. This shifts the goal from maximizing S/A quantity to maximizing S/A *choice depth*.

5. **Temporal structure (early vs. late behavior) has been underexplored.** V6 algorithms apply the same mechanism throughout. An algorithm that operates differently in three phases (exploration, narrowing, commitment) could match the natural draft arc more precisely.

6. **The opponent-signal dimension is untapped.** In a multiplayer draft, cards not taken by opponents carry information. Even in a single-player roguelike, the *absence* of certain cards from the pool can be simulated to create the feel of a contested draft.

---

## Five Algorithm Proposals

### 1. Negative Sculpting ("Card Culling")

**One sentence:** Before showing each pack, draw 6 cards randomly, then remove the 2 whose primary resonance is most distant from the player's top resonance, showing only the remaining 4.

**Technical description:** Track weighted resonance counters as usual. When generating a pack, draw 6 cards from the pool uniformly. Compute a "distance" for each card: cards whose primary resonance matches the player's top resonance score 0 (keep), cards matching the second-highest score 1, others score 2+. Remove the 2 highest-distance cards. Show the remaining 4. In early picks (before any resonance is dominant), all cards are equidistant and culling is effectively random -- preserving openness.

**Predicted behavior:** Under optimistic fitness, culling distant cards raises the concentration of resonance-matched cards from ~50% to ~67% per slot. Estimated S/A: ~1.7 (each slot has ~67% chance of matching the top-2 resonances, ~50% of which are S/A = ~33% S/A per slot, times 4 = 1.33). Under realistic fitness (moderate), the resonance-matched S/A precision drops to 75%, giving ~1.0 S/A. **Too weak as standalone.** The mechanism removes wrong-resonance cards but cannot distinguish wrong-archetype cards within the right resonance.

### 2. Phase-Shift Draft ("Tidal Draft")

**One sentence:** The draft alternates between 3-pick "exploration" phases (fully random packs) and 2-pick "focus" phases (3 of 4 slots match the player's top resonance), cycling throughout the draft.

**Technical description:** The draft follows a fixed rhythm: picks 1-3 are exploration (all random), picks 4-5 are focus (3 resonance-matched slots), picks 6-8 exploration, picks 9-10 focus, and so on in a 3/2 cycle. During focus phases, the system reads the player's current top resonance and fills 3 of 4 slots with that resonance's cards. No token tracking, no thresholds -- the timing is purely structural. The player's top resonance is recalculated before each focus phase, so pivoting between exploration phases is possible.

**Predicted behavior:** Under optimistic fitness, focus packs deliver ~3.25 S/A (3 resonance-matched at 100% + 1 random at 25%). Exploration packs deliver ~1.0. Weighted average across the 3/2 cycle: (3 * 1.0 + 2 * 3.25) / 5 = 1.9 S/A. Under moderate fitness, focus packs drop to ~2.47 (3 * 0.75 + 0.25), giving average 1.59. Under pessimistic, focus packs at ~2.08, average 1.43. **Reasonable robustness but falls short of 2.0 under any realistic model.** The fixed rhythm is simple but feels artificial -- the draft does not respond to player behavior, only to pick number.

### 3. Echo Drafting ("Memory Packs")

**One sentence:** After each pick, the next pack always contains one card that shares primary resonance with the card just drafted, plus 3 random cards.

**Technical description:** After the player selects a card, note its primary resonance. In the next pack, slot 1 is filled with a random card whose primary resonance matches the just-drafted card's primary resonance. Slots 2-4 are filled randomly. If the drafted card is generic (no resonance), all 4 slots are random. No counters, no thresholds, no state beyond "the last card's primary resonance." The mechanism is entirely memoryless beyond one pick.

**Predicted behavior:** Under optimistic fitness, committed players get 1 resonance-matched slot per pack (100% S/A) + 3 random (~0.75 S/A) = 1.75 S/A. Under moderate, the matched slot drops to 75% S/A, giving 1.5 S/A. Under pessimistic, 1.375. **Too weak for convergence.** One guaranteed slot is insufficient. However, the extreme simplicity is attractive -- this is the simplest possible resonance-responsive algorithm. Could serve as a base layer.

### 4. Pair-Echo Surge ("Harmonic Surge")

**One sentence:** Each drafted card's primary resonance earns 2 tokens and secondary symbols earn 1; when any counter reaches 4, spend it and fill 3 of the next pack's slots with cards whose primary resonance matches, but among those 3, prioritize cards that also carry a symbol of the player's second-strongest resonance.

**Technical description:** Token tracking identical to V6 Surge Packs (T=4, S=3). The innovation is in *how surge slots are filled*. When a surge fires for resonance R, the system identifies the player's second-strongest resonance R2. For each of the 3 surge slots, the system first attempts to draw a card with primary resonance R that also has R2 as any symbol (primary, secondary, or tertiary). If insufficient cards exist in this R+R2 intersection, fall back to any card with primary R. The 4th slot remains random. This applies the V7 archetype-disambiguation insight: using two resonance signals to narrow from 4 candidate archetypes to 1-2.

**Predicted behavior:** Under optimistic fitness, identical to Surge Packs (2.05 S/A) since all resonance-matched cards are S/A regardless. Under moderate fitness, the pair-filtering increases the home-archetype fraction of surge slots. If the R+R2 intersection contains ~60% home-archetype cards (vs. 50% for R-only), each surge slot S/A precision rises from 75% to ~80%, giving surge packs ~2.6 vs. 2.47, blended ~1.66 vs. 1.60. **Marginal improvement (~0.06 S/A) -- the intersection pool is too small to reliably filter.** The 15% dual-resonance cap means only ~13 cards per resonance carry a secondary symbol, and only a fraction match R2.

### 5. Aspiration Packs ("Draft Compass")

**One sentence:** After each pick, compute the player's top resonance pair (R1, R2); in the next pack, one slot shows a card with primary R1, one slot shows a card with primary R2, and two slots are random -- but if R1 and R2 are tied or R2 is below a minimum threshold, all four slots are random instead.

**Technical description:** Track weighted resonance counters. After each pick, identify R1 (highest) and R2 (second-highest). If R2 >= 2 weighted points AND R2 is at least 40% of R1's count, the next pack is an "aspiration pack": slot 1 draws from R1's primary pool, slot 2 draws from R2's primary pool, slots 3-4 are random. If R2 is below threshold (early draft, or very lopsided signal), all 4 slots are random. The R2 threshold gate ensures the system waits until the player shows a two-resonance pattern before activating. The two guaranteed slots target *different* resonances, which together identify 1-2 archetypes (the archetype sitting at the R1-R2 intersection on the circle).

**Predicted behavior:** Under optimistic fitness, aspiration packs contain 1 R1-matched slot (100% S/A) + 1 R2-matched slot (~50% S/A, since R2 is secondary for the target archetype but primary for two other archetypes) + 2 random (25% S/A each) = 1.0 + 0.5 + 0.5 = 2.0 S/A. Activation around pick 3-4 when R2 first crosses the threshold. Under moderate fitness, R1 slot drops to 75% S/A and R2 slot stays at ~50% (R2 matches are from archetypes where R2 is primary -- a different dilution structure than R1). Estimated 1.75 S/A. Under pessimistic, ~1.55 S/A. **The dual-slot approach provides structural archetype disambiguation.** The R2 slot ensures the player sees their secondary resonance's cards, which include their home archetype's splash cards. The R2 gate prevents early activation, preserving exploration. The mechanism is genuinely novel: no prior algorithm fills pack slots from two different resonances simultaneously based on the player's emerging pair.

---

## Champion Selection: Aspiration Packs ("Draft Compass")

**Justification:** Aspiration Packs is the only proposal that structurally addresses the core V7 problem -- archetype disambiguation under realistic fitness -- without requiring complex intersection pools, token thresholds, or multi-mechanism layering. By guaranteeing one slot from each of the player's top two resonances, it naturally surfaces the archetype sitting at their intersection on the circle. A Warriors player (Tide/Zephyr) sees one Tide card and one Zephyr card per pack, ensuring exposure to both halves of their archetype's identity.

The mechanism is simpler than Surge Packs (no token spending, no surge/non-surge alternation), more responsive than Phase-Shift (activates based on player signal, not fixed timing), and provides better archetype precision than any single-resonance targeting approach. The R2 activation gate is the critical innovation: it prevents the system from prematurely committing and preserves fully-random packs during exploration.

The honest weakness: 2 guaranteed slots (rather than Surge Packs' 3) means lower raw S/A under optimistic assumptions. But under realistic fitness, Aspiration Packs' archetype precision advantage compensates. The R2 slot serves a fundamentally different purpose than adding a second R1 slot -- it provides *orthogonal information* about the player's archetype.

---

## Champion Deep-Dive: Aspiration Packs

### Example Draft Sequences

**Committed Warriors player (Tide primary, Zephyr secondary):**

| Pick | Drafted | Counters (E/S/T/Z) | R1/R2 | R2 gate? | Next Pack |
|------|---------|---------------------|-------|----------|-----------|
| 1 | [Tide, Tide] | 0/0/3/0 | T/- | No (no R2) | 4 random |
| 2 | [Tide, Zephyr] | 0/0/5/1 | T/Z | No (Z=1 < 2) | 4 random |
| 3 | [Zephyr, Tide] | 0/0/6/3 | T/Z | Yes (Z=3, 50% of T=6) | 1 Tide + 1 Zephyr + 2 random |
| 4 | [Tide] from aspiration | 0/0/8/3 | T/Z | Yes | Aspiration |
| 5 | [Zephyr, Zephyr] from aspiration | 0/0/8/5 | T/Z | Yes | Aspiration |
| 6-30 | Continuing pattern | Rising T and Z | T/Z | Yes | Aspiration |

Convergence to aspiration packs at pick 3. The Tide slot shows Warriors or Sacrifice cards; the Zephyr slot shows Flash or Ramp cards. For a Warriors player, the Zephyr slot provides Ramp splash and Flash off-archetype options -- both adjacent on the circle. The player naturally gravitates toward the Tide-Zephyr intersection.

**Exploring player (picks spread across resonances):**

| Pick | Drafted | Counters (E/S/T/Z) | R1/R2 | R2 gate? |
|------|---------|---------------------|-------|----------|
| 1 | [Ember] | 2/0/0/0 | E/- | No |
| 2 | [Stone, Ember] | 3/2/0/0 | E/S | Yes (S=2, 67% of E=3) |
| 3 | [Tide, Tide] | 3/2/3/0 | E=T/S | Unstable (E and T tied) |
| 4 | [Tide] | 3/2/5/0 | T/E | Yes |
| 5 | [Tide, Zephyr] | 3/2/7/1 | T/E | Yes (E=3, 43% of T=7) |

The player's R1/R2 pair *shifts* as they explore. Aspiration packs track the current pair, so the system pivots naturally. At pick 5, the player sees one Tide card and one Ember card -- surfacing Self-Discard or Sacrifice (Tide-Ember archetypes). If they pivot to Zephyr, the pair shifts to T/Z and aspiration packs shift accordingly.

### Failure Modes

1. **Mono-resonance player never activates R2.** A player drafting only Tide cards (no secondary) never crosses the R2 gate. They get all-random packs forever. *Mitigation:* Most cards carry 2+ symbols, so secondary resonance accumulates naturally. A player drafting 2-symbol Tide-Tide cards still earns 0 secondary tokens, but this is rare -- most Tide cards carry a Zephyr or Stone secondary. In practice, R2 activates by pick 3-5 for typical drafters.

2. **R2 slot shows wrong-archetype cards.** The Zephyr slot for a Warriors player surfaces Flash and Ramp cards. Only Ramp (Zephyr/Tide) is adjacent; Flash (Zephyr/Ember) is distant. Under realistic fitness, the Zephyr slot has ~50% chance of showing the wrong pair of archetypes. *Mitigation:* The R2 slot's purpose is to provide variety and splash, not primary convergence. The R1 slot carries the convergence load. Even "wrong" R2 cards offer off-archetype options (M4 target).

3. **R2 gate oscillates.** If R2 hovers near the 40% threshold, packs alternate between aspiration and random unpredictably. *Mitigation:* Add hysteresis -- once activated, the gate stays open until R2 drops below 30% of R1. This prevents oscillation.

### Parameter Variants

**Variant A: Standard (R2 >= 2, R2 >= 40% of R1)**
- Activates pick 3-5 for most players
- 2 guaranteed slots (R1 + R2) + 2 random
- Estimated S/A: 2.0 optimistic, 1.75 moderate, 1.55 pessimistic

**Variant B: Aggressive (R2 >= 1, R2 >= 25% of R1)**
- Activates pick 2-3
- Same slot structure
- Earlier convergence, less exploration
- Estimated S/A: same per-pack, but earlier activation raises average

**Variant C: Triple (R2 >= 3, R2 >= 40% of R1, add third R1 slot)**
- 2 R1 slots + 1 R2 slot + 1 random when activated
- Higher convergence (estimated 2.25 optimistic, 1.9 moderate)
- Less variety in packs; may feel deterministic

### Proposed Fitness Models

1. **Optimistic:** All same-resonance cards are A-tier in sibling archetype. R1 slot = 100% S/A, R2 slot = 50% S/A (only half R2's archetypes share a resonance with the player).

2. **Moderate (50/30/20):** R1 slot = 75% S/A. R2 slot = ~37.5% S/A (50% of R2 cards are in the right pair of archetypes, of those 75% are S/A). Total aspiration pack: 0.75 + 0.375 + 0.5 = 1.625.

3. **Pessimistic (25/40/35):** R1 slot = 62.5% S/A. R2 slot = ~31.25% S/A. Total: 0.625 + 0.3125 + 0.5 = 1.4375.

The key advantage over Surge Packs: Aspiration Packs' dual-resonance structure provides *additive* archetype information. Even when individual slot S/A precision drops under realistic fitness, the *combination* of R1 and R2 slots in the same pack gives the player a higher probability of seeing at least one genuinely playable card than 3 slots all targeting the same resonance. This is the diversification argument: two independent 75%-precision signals are more reliable than three correlated 75%-precision signals.
