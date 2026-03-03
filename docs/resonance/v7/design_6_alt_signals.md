# V7 Agent 6: Alternative Signal Systems

## Key Takeaways

- **Resonance alone identifies a resonance pair (4 archetypes), not a single archetype.** Under realistic fitness, this ambiguity wastes 25-45% of resonance-matched slots on cards that are B/C-tier for the player's actual archetype. A second signal dimension could cut this waste significantly.
- **Energy cost is the strongest alternative signal.** Archetypes have natural cost profiles: aggro/tempo clusters at 0-2 energy, midrange at 3-4, control/ramp at 5+. Cost distribution of drafted cards is a visible, trackable property that correlates with archetype identity independently of resonance.
- **Card type (Character vs. Event) provides a weaker but still useful signal.** Spell-heavy archetypes (Storm, Self-Discard) draft more Events than creature-based archetypes (Warriors, Ramp). The binary nature limits its disambiguation power but makes it trivially implementable.
- **Subtype is the highest-precision signal but the hardest to use systematically.** Warrior subtype strongly predicts Warriors archetype; Ancient predicts Ramp. However, subtypes are unevenly distributed and only apply to Characters, making them unreliable as a universal signal.
- **Hybrid resonance+cost systems offer the best precision-to-complexity ratio.** Tracking one resonance counter plus one cost-band counter doubles the signal dimensions while remaining one-sentence describable.
- **Any alternative signal system must be designable -- card designers must be able to control the signal property per archetype when building the card pool.** Energy cost and card type are easily controlled; keyword presence is harder to guarantee.
- **The realistic fitness problem is fundamentally an archetype disambiguation problem.** Alternative signals attack it directly rather than trying to brute-force more resonance-matched cards into packs.

---

## Five Algorithm Proposals

### 1. Cost-Band Surge

**One sentence:** Each drafted card earns cost-band tokens (low/mid/high) alongside resonance tokens; surges fill slots with cards matching BOTH the top resonance AND the dominant cost band.

**Technical description:** Maintain 4 resonance counters (as in Surge Packs) plus 3 cost-band counters: Low (cost 0-2), Mid (cost 3-4), High (cost 5+). Each drafted card earns +2 primary resonance, +1 secondary resonance, AND +1 to its cost band. When a resonance counter hits threshold T, spend T and generate a surge pack where 3 of 4 slots are filled with cards matching that resonance AND the player's dominant cost band. If insufficient cards match both criteria, fall back to resonance-only matching.

**Predicted behavior:** Under optimistic fitness, performs similarly to Surge Packs (~2.0 S/A) since the cost filter is redundant when all resonance-matched cards are already S/A. Under realistic fitness, the cost-band filter preferentially selects cards from the archetype whose cost profile matches the player's draft pattern, boosting S/A precision from ~75% to ~85-90%. Warriors (midrange costs) and Sacrifice (lower costs with sacrifice fodder) naturally separate along cost lines even though both share Tide primary.

### 2. Type-Weighted Resonance

**One sentence:** Characters and Events earn resonance tokens at different rates; when drafted card type matches the archetype's natural type distribution, bonus tokens are awarded.

**Technical description:** Maintain standard 4 resonance counters. Also track the Character/Event ratio of drafted cards. Characters earn +2 primary resonance as normal. Events earn +3 primary resonance (because Events are rarer and archetype-defining -- Storm drafts more Events than Warriors). When generating surge packs, if the player's Event ratio exceeds 40%, surge slots prefer Event cards of the matching resonance; if below 25%, prefer Characters. This implicitly narrows from 4 candidate archetypes toward the 1-2 that match the player's type distribution.

**Predicted behavior:** Under optimistic fitness, slightly faster surges for Event-heavy drafters (~2.1 S/A). Under realistic fitness, the type preference improves S/A precision by ~5-10% for archetypes with strong type skews (Storm is heavily Event-based; Warriors is heavily Character-based). Less effective for archetypes with balanced type distributions (Blink, Self-Mill).

### 3. Rarity Anchor

**One sentence:** Rare/Legendary cards ("anchors") carry a hidden archetype affinity signal; when a surge fires, one surge slot is filled with a card of matching resonance AND matching rarity tier to the player's highest-rarity drafted card.

**Technical description:** Track the highest rarity among the player's drafted resonance-matched cards. During surge pack generation, 2 of 3 surge slots are standard resonance-matched, but 1 slot specifically draws from the same rarity tier as the player's anchor card. The reasoning: Rare/Legendary cards tend to be the most archetype-specific (they carry the strongest synergy text), so a player who drafts a Rare Tide card is sending a strong signal about WHICH Tide archetype they are in. By matching rarity tier, the algorithm implicitly clusters around the mechanical neighborhood of the anchor card.

**Predicted behavior:** Under optimistic fitness, marginal improvement (~2.05-2.10 S/A) since rarity matching is a weak filter. Under realistic fitness, the anchor effect depends heavily on whether rarity correlates with archetype specialization in the card pool. If Rares are more archetype-specific (likely in practice), this improves precision by ~5-8%. Fails if Rares are "good stuff" cards that work in multiple archetypes.

### 4. Dual-Counter Surge (CHAMPION)

**One sentence:** Track resonance tokens and cost-profile tokens separately; surges fire when resonance threshold is met AND fill surge slots by filtering for cards whose cost matches the player's emerging cost profile.

**Technical description:** Maintain 4 resonance counters (standard Surge Packs rules: +2 primary, +1 others, threshold T, spend T on surge). Additionally, maintain a running average cost of all drafted cards (a single floating-point number, not shown to player). During surge pack generation, the 3 resonance-matched surge slots are filtered: draw from resonance-matched cards whose cost is within +/-1 of the player's average cost. If fewer than 3 candidates exist in this band, widen to +/-2, then fall back to unfiltered. The cost filter narrows the resonance pool from ~80 cards (all cards of one resonance) to ~30-40 cards clustered around the player's cost preference, which naturally correlates with archetype identity.

**Predicted behavior:** Under optimistic fitness, approximately equivalent to Surge Packs (2.0-2.05 S/A) since the cost filter doesn't help when all resonance-matched cards are already playable. Under realistic fitness, the cost filter acts as an archetype disambiguator. If Warriors cards average cost 3.5 and Sacrifice cards average cost 2.0, a player drafting at average cost 3.2 will see surge slots heavily weighted toward Warriors-home cards rather than a 50/50 split with Sacrifice. Estimated realistic S/A improvement: +0.15-0.25 over base Surge Packs, potentially keeping the algorithm at 1.7-1.85 under moderate fitness vs. Surge Packs' 1.6.

### 5. Keyword Echo

**One sentence:** When a drafted card contains a keyword that appears on 3+ of the player's existing picks, the next pack gets one bonus slot filtered to cards sharing that keyword AND the player's top resonance.

**Technical description:** Track keyword occurrences across drafted cards (keywords: Reclaim, Dissolve, Prevent, Foresee, Kindle, Abandon, Discover, Judgment, Materialize triggers, etc.). When the player drafts a card and any keyword on it has appeared on 3+ previously drafted cards, add 1 bonus card to the next pack filtered to: (a) shares the player's top resonance AND (b) contains that keyword. This creates a synergy-echo effect where mechanical themes self-reinforce. The keyword acts as an implicit archetype identifier since each archetype has characteristic keywords (Storm uses Prevent/Dissolve, Self-Discard uses Discard triggers, Sacrifice uses Abandon).

**Predicted behavior:** Under optimistic fitness, adds ~0.3-0.5 S/A on top of a base mechanism (best as a supplement to Surge Packs). Under realistic fitness, keyword filtering is the highest-precision archetype signal available -- a card with Abandon is almost certainly Sacrifice-home, not Warriors-home. However, keyword echo fires late (needs 3+ keyword matches) and fire rate is unpredictable across archetypes. Archetypes with concentrated keywords (Sacrifice/Abandon) benefit enormously; archetypes with diffuse keywords (Midrange) barely benefit.

---

## Champion Selection: Dual-Counter Surge

**Justification:** Dual-Counter Surge is selected because it attacks the core disambiguation problem (separating archetypes that share a primary resonance) using the most universally applicable alternative signal (energy cost), while requiring minimal additional complexity beyond standard Surge Packs. The one-sentence description adds only "filtered by cost profile" to the existing Surge Packs sentence. Cost is fully visible to the player, easily controlled by card designers, and correlates with archetype identity across ALL eight archetypes (unlike keywords or subtypes, which help some archetypes more than others).

Cost-Band Surge (proposal 1) was a close runner-up but requires maintaining 3 additional discrete counters. Dual-Counter Surge uses a single running average, which is simpler to implement and produces a smoother signal. Keyword Echo (proposal 5) has the highest theoretical precision but fires too inconsistently and benefits archetypes unequally.

---

## Champion Deep-Dive: Dual-Counter Surge

### Example Draft Sequences

**Sequence A -- Warriors player (Tide primary, Zephyr secondary, avg cost ~3.5):**
- Picks 1-4: Mixed cards, average cost drifts to 3.2. Tide counter reaches 5, fires surge.
- Pick 5 surge pack: 3 Tide cards filtered to cost 2-4. Pool contains ~40 Tide-primary cards; cost filter narrows to ~25, of which ~15 are Warriors-home (cost 3-4 characters) and ~10 are Sacrifice-home (cost 2-3 abandon enablers). S/A precision: ~60% S + ~15% A = ~75% under moderate fitness (vs. ~50% S + ~25% A = 75% without filter). The improvement is modest here because the cost bands overlap.
- Picks 6-10: Average cost stabilizes at 3.5. Subsequent surges filter to cost 2.5-4.5, which excludes more Sacrifice low-cost cards. S/A precision rises to ~65% S + ~15% A = ~80%.

**Sequence B -- Storm player (Ember primary, Stone secondary, avg cost ~2.5, Event-heavy):**
- Picks 1-5: Drafts Events at cost 2-3. Ember counter fires surge.
- Pick 6 surge pack: 3 Ember cards filtered to cost 1.5-3.5. Storm-home cards cluster at cost 2-3 (cheap spells); Blink-home cards cluster at cost 3-5 (ETB creatures). Cost filter preferentially selects Storm cards. S/A precision improvement: ~+10% over unfiltered.

**Sequence C -- Pivot scenario:**
- Picks 1-4: Player drafts Zephyr cards at high cost (avg 4.5), suggesting Ramp.
- Pick 5: Player picks a powerful low-cost Zephyr card (cost 1). Average drops to 3.8.
- Pick 6 surge: Filter still targets cost 2.8-4.8, which serves both Ramp and Flash. The running average adjusts slowly, providing pivot tolerance.

### Failure Modes

1. **Overlapping cost profiles:** If two archetypes sharing a primary resonance have similar average costs (e.g., both cluster at cost 3), the cost filter provides no disambiguation. This is the most likely failure mode and depends entirely on card pool design. Mitigation: card designers should aim for at least 1.0 average cost difference between archetypes sharing a primary resonance.

2. **Insufficient pool depth at intersection:** If very few cards match both the resonance AND cost band, the algorithm falls back to unfiltered resonance matching, providing zero benefit. This is most likely early in the draft when the cost profile is unstable. Mitigation: use wider initial cost bands (+/-2) that narrow as draft progresses.

3. **Power-chaser disruption:** Players who pick raw power regardless of archetype will have unstable cost averages, causing the cost filter to oscillate unhelpfully. However, power-chasers already defeat pure resonance tracking, so this is not a regression.

### Parameter Variants

**Variant A -- Tight filter (baseline):** Cost window +/-1 of average. Strongest disambiguation when it works, but frequent fallback to unfiltered. Best for card pools where archetypes have well-separated cost curves.

**Variant B -- Adaptive filter:** Cost window starts at +/-2 (picks 1-8) and narrows to +/-1 (picks 9+). Provides reliable early packs while increasing precision as the cost profile stabilizes. Adds slight complexity but may deliver better convergence timing.

**Variant C -- Median instead of mean:** Use median cost of drafted cards instead of mean. More robust to outlier picks (e.g., one expensive splash card doesn't skew the profile). Slightly harder to compute but more stable.

### Proposed Fitness Models for Testing

**Model 1 -- Optimistic (V6 baseline):** All cross-resonance cards are A-tier. Cost filter has no effect on S/A since all resonance-matched cards are already S/A. Expect results identical to Surge Packs (~2.05 S/A). This confirms the algorithm does no harm.

**Model 2 -- Moderate with cost correlation:** 50% of sibling-archetype cards are A-tier, 30% B, 20% C. Additionally, cards whose cost is within 1 of the player's archetype average are 20% more likely to be A-tier (simulating the natural correlation between cost profile and archetype fit). Expect ~1.75 S/A (vs. Surge Packs' ~1.60 under moderate).

**Model 3 -- Pessimistic with cost correlation:** 25% A, 40% B, 35% C for sibling cards. Cost-correlated A-tier bonus of 15%. Expect ~1.50 S/A (vs. Surge Packs' ~1.38 under pessimistic). The cost filter provides diminishing returns as the base A-tier rate drops, because there are fewer A-tier cards to preferentially select.

**Model 4 -- Moderate with designed cost separation:** Same base rates as Model 2, but card pool is explicitly designed so that archetypes sharing a primary resonance have average costs separated by >= 1.5 energy. Under this constraint, the cost filter becomes highly effective: estimated ~1.85-1.90 S/A. This model tests the upper bound of what card designers can enable through deliberate cost-curve management.

The key insight for the card designer: if each pair of archetypes sharing a primary resonance is designed with meaningfully different cost curves (e.g., Warriors averages 3.5 while Sacrifice averages 2.0), Dual-Counter Surge can recover most of the S/A lost to realistic fitness assumptions without requiring any cross-archetype card playability. The cost signal does the disambiguation work that resonance alone cannot.
