# V7 Agent 3: Archetype Disambiguation

## Key Takeaways

- **The disambiguation problem is the root cause of realistic-fitness degradation.** All V6 algorithms target at resonance level (4 archetypes per resonance). Under realistic fitness, ~25-45% of resonance-matched cards are B/C for the player's actual archetype. Disambiguation directly attacks this loss.
- **Each archetype has a unique (primary, secondary) resonance pair.** Warriors = (Tide, Zephyr), Sacrifice = (Tide, Stone). If the system reads both signals, it can narrow from 4 candidate archetypes to 1-2, recovering most of the realistic-fitness loss.
- **Secondary resonance signal emerges naturally from drafting.** A player building Warriors drafts Tide-primary cards and also accumulates Zephyr as a secondary signal. By pick 6-8, the top-2 resonance profile reliably identifies the archetype.
- **Dual-resonance pair matching is limited to 15% of cards -- too small for a primary mechanism.** Disambiguation must work through card selection from the single-resonance pool, not through the dual-resonance card pool.
- **Cost curve and card type distribution correlate with archetype identity** but are too noisy for reliable targeting. Resonance pair signals are far stronger.
- **The timing paradox: disambiguation requires signal, but early packs need to provide cards before the signal exists.** The mechanism must tolerate ambiguity early and sharpen late.
- **The best disambiguation mechanism layers on top of Surge Packs**, not replacing it. Surge handles the pack-filling mechanics; disambiguation refines which cards fill those slots.

---

## Five Algorithm Proposals

### 1. Dual-Counter Surge ("Paired Surge")

**One sentence:** "Surge packs fill slots with cards whose primary resonance matches the player's top resonance AND whose secondary symbol (if any) matches the player's second-highest resonance, falling back to primary-only matches when insufficient dual-matches exist."

**Technical description:** Maintain 4 resonance counters as in standard Surge Packs. When a surge triggers for resonance X, examine the player's second-highest counter Y. Fill surge slots by preferring cards with primary=X and any secondary/tertiary symbol=Y. If fewer than 3 such cards exist in the pool, fill remaining surge slots with primary=X cards regardless of secondary. This biases surge packs toward the player's specific archetype pair without requiring dual-resonance cards -- a single-resonance Tide card with secondary Zephyr symbols is preferred for a (Tide, Zephyr) Warriors player.

**Predicted behavior:** Under optimistic fitness, performs identically to Surge Packs (~2.05 S/A) since all resonance-matched cards are already S/A. Under moderate fitness, the secondary filtering preferentially selects home-archetype cards, boosting S/A precision from 75% to ~85-90%. The improvement depends on how many single-resonance cards carry secondary symbols matching the archetype pair. With only 0-3 symbols per card and the leftmost being primary, secondary symbol availability may be too sparse for reliable filtering.

### 2. Archetype Gradient Filter ("Gradient Filter")

**One sentence:** "When filling resonance-matched slots, score each candidate card by how well its full symbol profile matches the player's accumulated resonance weights, preferring cards whose symbol ratios mirror the player's draft history."

**Technical description:** Each card has 0-3 resonance symbols. Compute a "profile match score" for each candidate: for each of the card's symbols, add the player's weight in that resonance. Cards scoring highest are selected for surge slots. A Warriors player with weights {Tide: 12, Zephyr: 8, Ember: 2, Stone: 1} would strongly prefer a card with symbols [Tide, Zephyr] over [Tide, Stone], and would prefer [Tide, Tide] over [Tide, Ember]. This creates a gradient that naturally pulls surge packs toward the player's archetype without binary thresholds.

**Predicted behavior:** Under optimistic fitness, minimal difference (~2.05 S/A). Under moderate fitness, cards whose symbol profiles match the player's archetype are more likely to be home-archetype cards (S-tier), improving S/A precision to ~80-85%. The gradient approach avoids hard cutoffs and gracefully handles early ambiguity. However, it introduces a sorting step that is harder to describe in one sentence and harder for players to understand intuitively.

### 3. Secondary Resonance Slot ("Split Surge")

**One sentence:** "When a surge triggers for resonance X, fill 2 slots with primary=X cards, 1 slot with a card whose primary resonance matches the player's second-highest counter, and the 4th slot random."

**Technical description:** Standard Surge Packs token system. When resonance X surges, the pack becomes: 2 slots of primary=X, 1 slot of primary=Y (where Y is the player's second-highest resonance counter), 1 random slot. This directly leverages the archetype circle structure: a Warriors player (Tide primary, Zephyr secondary) gets 2 Tide cards + 1 Zephyr card + 1 random. The Zephyr card has a 50% chance of being a Ramp card (Zephyr primary, Tide secondary) -- which shares the Warriors secondary resonance and is likely B-tier or better. More importantly, the split gives the player cards from both of their archetype's resonances, enabling the "secondary resonance" half of their deck.

**Predicted behavior:** Under optimistic fitness, slight decrease from Surge Packs (~1.95 S/A) since 1 slot is now secondary-resonance instead of primary-resonance, and secondary-resonance cards are B-tier in the optimistic model. Under moderate/pessimistic fitness, the diversification helps: primary slots deliver 75%/62.5% S/A, but the secondary slot now provides B-tier cards that are genuinely useful for the player's deck construction. The key question is whether the secondary slot's B-tier contribution compensates for losing one primary-matched slot. Under moderate fitness, expected S/A per surge pack: 2*(0.75) + 1*(~0.35) + 1*(0.22) = 2.07 vs. standard surge's 3*(0.75) + 1*(0.22) = 2.47 -- this is actually worse by 0.4 S/A. The split only helps if the secondary-resonance slot achieves better than B-tier fitness for the player's archetype, which is not guaranteed.

### 4. Draft History Clustering ("Pattern Match")

**One sentence:** "After pick 5, classify the player's likely archetype from their drafted card distribution across all 4 resonances, and bias surge pack composition toward cards whose home archetype matches the inferred archetype."

**Technical description:** Maintain a 4-dimensional resonance weight vector. After pick 5, compute the (primary, secondary) resonance pair by ranking the top 2 counters. Map this pair to the corresponding archetype on the circle. When filling surge slots, prefer cards whose home archetype matches the inferred archetype. Since home archetype is evaluation-only metadata (not visible on the card), the system uses a proxy: prefer cards whose full symbol profile best matches the archetype's expected profile. For Warriors (Tide, Zephyr), prefer cards with Tide primary and Zephyr secondary symbols. When no strong secondary signal exists (top 2 are close), fall back to pure resonance matching.

**Predicted behavior:** Under optimistic fitness, identical to Surge Packs. Under moderate fitness, significant improvement after pick 5 when the archetype inference kicks in. The improvement depends on inference accuracy. With 8 archetypes and a clean (primary, secondary) signal, accuracy should be >80% by pick 6-7. Failure mode: power-chaser players who draft across resonances produce ambiguous signals, causing misclassification. This is acceptable since power-chasers don't need convergence support.

### 5. Resonance Pair Surge ("Pair Surge") -- CHAMPION

**One sentence:** "Each drafted symbol earns resonance tokens (+2 primary, +1 others); when the top counter reaches 4, spend 4 and fill 2 surge slots with that resonance's cards plus 1 slot with the second-highest resonance's cards, fourth slot random."

**Technical description:** Identical token accumulation to standard Surge Packs. The change is in surge pack composition: instead of 3 slots of the top resonance, a surge delivers 2 slots of the top resonance + 1 slot of the second-highest resonance + 1 random slot. The (top, second) pair maps directly to an archetype on the circle. By filling the secondary slot with the player's secondary resonance, the system implicitly targets the specific archetype, not just the resonance. The secondary slot's cards come from a resonance that contains the player's archetype's secondary pool -- cards designed to complement the archetype's primary strategy.

The critical difference from Proposal 3 (Split Surge): Pair Surge uses the second-highest *counter* (reflecting accumulated draft behavior), while Split Surge statically uses counter rankings. In practice they converge, but Pair Surge's framing emphasizes that the secondary signal is earned through drafting, not imposed.

**Predicted behavior:** Under optimistic fitness: ~1.95-2.00 S/A (slight decrease from losing one primary slot). Under moderate fitness: the secondary-resonance slot improves archetype targeting. Key insight -- under moderate fitness, the secondary slot provides cards that are B-tier for the player's archetype (since secondary-resonance archetypes share secondary resonance). But the secondary slot also provides S-tier cards from the secondary resonance's home archetypes, some of which the player may splash. The real win is that 2 primary + 1 secondary gives the algorithm a fingerprint for the specific archetype, enabling future surge packs to better serve the player as the card pool is designed to support archetype pairs.

---

## Champion Selection: Pair Surge

**Justification:** Pair Surge is selected as champion for four reasons:

1. **Minimal complexity increase over Surge Packs.** The one-sentence description adds only the secondary slot concept. Token earning is identical. The player sees the same rhythmic surge/normal cycle.

2. **Directly addresses the disambiguation problem.** A 2+1+1 surge pack for a Warriors player contains 2 Tide cards and 1 Zephyr card, implicitly targeting the (Tide, Zephyr) archetype pair. Under realistic fitness, the Tide cards include both Warriors-home (S) and Sacrifice-home (mixed A/B/C) cards, but the Zephyr card is an additional signal that the system is tracking the player's specific archetype.

3. **Secondary slot provides genuine deck-building value.** Real decks in archetype games need cards from both their primary and secondary resonance. A Warriors deck wants mostly Tide cards but also needs Zephyr support cards. The secondary surge slot naturally provides this, improving deck quality even when the card isn't S/A-tier for the primary archetype evaluation.

4. **Graceful degradation.** Under optimistic fitness, Pair Surge performs within 0.05-0.10 of standard Surge Packs. Under moderate/pessimistic fitness, it does not lose ground as fast because the secondary slot provides useful (B-tier) cards instead of potentially C-tier cross-archetype primary cards. The floor is higher.

**Why not others:** Gradient Filter (Proposal 2) is the strongest alternative but fails the one-sentence simplicity test -- scoring cards by profile match is opaque. Pattern Match (Proposal 4) uses archetype metadata (even indirectly), creating coupling between the draft algorithm and card evaluation data. Dual-Counter Surge (Proposal 1) depends on secondary symbol availability, which is too sparse with 0-3 symbols per card and the 15% dual-resonance cap.

---

## Champion Deep-Dive: Pair Surge

### Example Draft Sequences

**Sequence 1: Committed Warriors player**
- Picks 1-3: Drafts Tide and Zephyr cards. Counters: Tide=7, Zephyr=4, Ember=1, Stone=0.
- Pick 4 pack: Tide surges (7>=4, spend 4). Pack = 2 Tide + 1 Zephyr (second-highest) + 1 random. Player sees 2 Warriors/Sacrifice cards, 1 Ramp/Flash card, 1 random. Under moderate fitness, expected S/A = 2*(0.75) + 1*(~0.35) + 1*(0.22) = 2.07.
- Picks 5-8: Alternating surges. Tide counter rebuilds. Zephyr counter also grows. By pick 8, the secondary slot consistently delivers Zephyr cards, building the Warriors + Zephyr splash deck.
- Late draft: Stable surge rhythm. 2 Tide + 1 Zephyr per surge pack. The player's deck contains ~60% Tide, ~20% Zephyr, ~20% other -- matching the expected Warriors deck composition.

**Sequence 2: Signal reader pivoting from Sacrifice to Warriors**
- Picks 1-4: Drafts Tide/Stone cards (Sacrifice direction). Counters: Tide=8, Stone=5, Zephyr=1, Ember=0.
- Pick 5-6: Notices Zephyr cards are stronger in packs. Starts drafting Tide/Zephyr instead.
- By pick 8: Counters: Tide=14, Zephyr=7, Stone=5, Ember=1. Surges now deliver 2 Tide + 1 Zephyr, naturally pivoting to Warriors without any permanent lock preventing it.

**Sequence 3: Power-chaser with mixed signals**
- Picks 1-10: Drafts highest-power cards across resonances. Counters: Tide=6, Ember=5, Zephyr=4, Stone=3.
- Surges are infrequent (no counter builds far ahead). When Tide surges at pick 5, the secondary slot shows Ember (second-highest). Pack = 2 Tide + 1 Ember + 1 random. This targets the general Tide/Ember area (Sacrifice or Blink) without strong disambiguation. Under moderate fitness, S/A is lower (~1.5) but the player doesn't need convergence since they're power-chasing.

### Failure Modes

1. **Weak secondary signal.** If the player's top 2 and 3rd resonance counters are close (e.g., Tide=8, Zephyr=5, Stone=4), the secondary slot may target the wrong archetype pair in some surges. Mitigation: this self-corrects as drafting continues and the secondary counter separates from the third.

2. **S/A regression vs. Surge Packs under optimistic fitness.** Losing one primary slot for a secondary-resonance slot costs ~0.10 S/A under optimistic assumptions. If the game's actual card design is closer to optimistic (high cross-archetype playability), Pair Surge is strictly worse than standard Surge Packs. Mitigation: this is the intended tradeoff -- sacrifice peak optimistic performance for robustness under realistic fitness.

3. **Secondary resonance cards are B-tier, not S/A.** Under the fitness model, secondary-resonance cards (e.g., Zephyr cards for a Warriors player) are B-tier for Warriors. The secondary slot delivers useful but not top-tier cards. This means the secondary slot contributes to deck quality but not to the S/A metric directly. Mitigation: reconsider the fitness model -- a Ramp card (Zephyr primary, Tide secondary) in a Warriors deck (Tide primary, Zephyr secondary) shares BOTH resonances with the player. Under a refined fitness model, such "complementary pair" cards might be A-tier rather than B-tier. This warrants simulation.

### Parameter Variants

**Variant A: 2+1+1 (Champion default)**
- 2 slots top resonance, 1 slot second resonance, 1 random.
- Moderate fitness estimated S/A: ~1.75 (depends on secondary slot fitness contribution).
- Most balanced between disambiguation and convergence.

**Variant B: 1+1+2 (Heavy Secondary)**
- 1 slot top resonance, 1 slot second resonance, 2 random.
- Moderate fitness estimated S/A: ~1.50.
- Maximum disambiguation but insufficient primary convergence. Likely fails M3.

**Variant C: 2+1+1 with Complementary Pair Preference**
- Same as Variant A, but the secondary slot prefers cards that also carry a symbol matching the player's primary resonance (e.g., a Zephyr card with a Tide secondary symbol for a Warriors player). Falls back to any secondary-resonance card if no complementary match exists.
- This directly targets "complementary pair" cards -- cards from the adjacent archetype that share both resonances with the player. Under any fitness model, these cards are highly likely to be playable because they were designed for an archetype with the same resonance pair (just reversed).
- Moderate fitness estimated S/A: ~1.85-1.95 if complementary pairs average A-tier fitness.

### Proposed Fitness Models for Testing

**Model 1: Standard Moderate (from Agent 1)**
- Home: S-tier. Adjacent primary: 50% A, 30% B, 20% C. Secondary: B-tier. Distant: C-tier.

**Model 2: Complementary Pair Aware**
- Home: S-tier. Adjacent primary sharing BOTH resonances (complementary pair): 80% A, 15% B, 5% C. Adjacent primary sharing only primary resonance (non-complementary): 30% A, 40% B, 30% C. Secondary: B-tier. Distant: C-tier.
- Rationale: A Ramp card (Zephyr primary, Tide secondary) is designed for a deck that wants both Zephyr and Tide cards. A Warriors player (Tide primary, Zephyr secondary) also wants both resonances, just with reversed priority. The mechanical overlap between "complementary pair" archetypes (sharing both resonances, reversed) should be much higher than between "same-primary-only" archetypes (e.g., Warriors and Sacrifice share Tide primary but have different secondaries).

**Model 3: Symbol Profile Fitness**
- Home: S-tier. For cross-archetype fitness, each card's fitness in archetype X depends on how many of the card's symbols appear in archetype X's resonance pair. Cards with 2+ symbols matching archetype X's pair: 70% A, 20% B, 10% C. Cards with 1 symbol matching: 35% A, 35% B, 30% C. Cards with 0 symbols matching: C-tier.
- Rationale: This treats the symbol profile as a proxy for mechanical relevance. A card with symbols [Tide, Zephyr] is likely designed for Warriors or Ramp, not Sacrifice.

The Complementary Pair Aware model (Model 2) is the most interesting for testing because it captures the structural hypothesis that drives Pair Surge: archetypes sharing both resonances (reversed) have higher cross-archetype playability than archetypes sharing only one resonance. If this hypothesis holds in card design, Pair Surge's secondary slot preferentially surfaces complementary-pair cards, and Variant C's explicit preference for them maximizes the effect.

### Simulation Priority

1. Run Variant A (2+1+1) under all Agent 1 fitness models for direct baseline comparison.
2. Run Variant C (complementary pair preference) under Model 2 (complementary pair aware) to test the structural hypothesis.
3. Compare both to standard Surge Packs at each fitness level to measure the disambiguation benefit.
