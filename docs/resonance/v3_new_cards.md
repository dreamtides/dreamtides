# V3 Card Set — Simplified Designs

## Revision Process Summary

This document revises the V2 card set with a focus on **simplicity**. The core principle: each card should have ONE clean ability, not two unrelated abilities stapled together. Cards should support 2+ draft archetypes through a single cohesive design.

- **Cards redesigned:** 13 (simplified from complex/stapled designs)
- **Cards kept as-is:** 10 (already clean single-ability designs)
- **Cards flagged for implementation discussion:** 2

**Final count: 25 cards**

---

## SECTION 1: DUAL-RESONANCE SIGNPOST CARDS (10 cards)

---

### Card 1: Tideweaver Adept

**Archetype:** Mirage (Tide+Zephyr)

| Property | Value |
|----------|-------|
| Cost | 4 energy |
| Type | Character |
| Subtype | Visitor |
| Spark | 1 |
| Rarity | Uncommon |
| Resonance | Tide + Zephyr |

**Ability text:**
Materialized: Draw a card for each other ally that was materialized this turn.

**V3 change:** Removed the modal choice (draw 2 OR bounce+energy). Replaced with a single scaling draw trigger that rewards flicker turns. In Mirage, flickering 3 allies then flickering this card draws 3. In Basalt, deploying 2 cheap Spirit Animals before this draws 2. No energy manipulation, no bounce clause.

**Archetype fit:**
- **Mirage (primary):** Dream flicker target — each flicker turn draws proportional to allies re-materialized.
- **Basalt (secondary):** Wide deployment of cheap Spirit Animals triggers the draw naturally.

---

### Card 2: Abyssal Reclaimer

**Archetype:** Undertow (Tide+Ruin)

| Property | Value |
|----------|-------|
| Cost | 3 energy |
| Type | Character |
| Subtype | Survivor |
| Spark | 1 |
| Rarity | Uncommon |
| Resonance | Tide + Ruin |

**Ability text:**
Materialized: Put the top 3 cards of your deck into your void. Return a Survivor from your void to your hand.

**V3 change:** Removed the complex conditional ("if a card entered your void from another source this turn"). Reclaim is now unconditional but restricted to Survivors, giving a clean tribal signal. No tracking of prior void entries needed.

**Archetype fit:**
- **Undertow (primary):** Mill 3 + retrieve best Survivor. Recursive engine with Soulkindler.
- **Eclipse (secondary):** Void-filling + Survivor retrieval gives fuel for discard engines.

---

### Card 3: Stoneveil Guardian

**Archetype:** Basalt (Zephyr+Stone)

| Property | Value |
|----------|-------|
| Cost | 3 energy |
| Type | Character |
| Subtype | Spirit Animal |
| Spark | 1 |
| Rarity | Uncommon |
| Resonance | Zephyr + Stone |

**Ability text:**
Judgment: Each allied Spirit Animal gains +1 spark until end of turn.

**V3 change:** Removed the conditional mode switch (3+ allies = energy, otherwise = spark). Replaced with a single unconditional tribal pump. The archetype already has abundant energy generation (Luminwings, Dawnprowler, etc.); it needed a clean spark payoff.

**Archetype fit:**
- **Basalt (primary):** Direct reward for going wide with Spirit Animals. Scales linearly with board width.
- **Mirage (secondary):** Flicker loops with Blazing Emberwing re-trigger Judgment abilities, turning each replay into a spark engine.

---

### Card 4: Forgeborn Martyr

**Archetype:** Crucible (Stone+Ember)

| Property | Value |
|----------|-------|
| Cost | 3 energy |
| Type | Character |
| Subtype | Warrior |
| Spark | 2 |
| Rarity | Rare |
| Resonance | Stone + Ember |

**Ability text:**
Judgment: You may abandon an allied Warrior. If you do, each other allied Warrior gains +2 spark until end of turn. Otherwise, kindle 1.

**V3 change:** Collapsed two unrelated abilities (passive Warrior lord + conditional death trigger) into a single Judgment choice. Creates genuine tension: sacrifice a Warrior for explosive burst, or play conservatively and accumulate kindle. No board-width thresholds, no tracking who dissolved what.

**Archetype fit:**
- **Crucible (primary):** The tension card Crucible needs. Sacrifice-vs-preservation decision every Judgment. Diluting Blade of Unity count for a massive temporary pump.
- **Cinder (secondary):** Abandon integrates with Cinder's sacrifice infrastructure. Kindle fallback feeds voltron strategy.

---

### Card 5: Ashfire Ritualist

**Archetype:** Cinder (Ember+Ruin)

| Property | Value |
|----------|-------|
| Cost | 3 energy |
| Type | Character |
| Subtype | Outsider |
| Spark | 1 |
| Rarity | Rare |
| Resonance | Ember + Ruin |

**Ability text:**
When you abandon an ally, kindle 1 and reclaim that character.

**V3 change:** Collapsed two separate "once per turn" abilities (rate-limited kindle trigger + rate-limited activated recursion) into a single triggered ability. One trigger, two tightly coupled effects. No rate limiters, no "different character" restriction.

**Archetype fit:**
- **Cinder (primary):** The sacrifice-recursion engine. Abandon to any outlet, get kindle + the character back. Requires other sacrifice outlets (not self-contained), making it a build-around signpost.
- **Undertow (secondary):** Incidental kindle + character recovery from void for Survivor-based strategies.

---

### Card 6: Stormtide Oracle

**Archetype:** Tempest (Tide+Ember)

| Property | Value |
|----------|-------|
| Cost | 4 energy |
| Type | Character |
| Subtype | Ancient |
| Spark | 2 |
| Rarity | Rare |
| Resonance | Tide + Ember |

**Ability text:**
Whenever you play an event, foresee 2.

**V3 change:** Removed variable spark (events in void) AND activated retrieval. Replaced with a single evergreen-keyword trigger. Eliminates redundancy with Spirit of Smoldering Echoes (no longer two cards counting void events). Fixed spark of 2 gives honest board presence. Occupies a distinct Tempest role: card selection and deck velocity.

**Archetype fit:**
- **Tempest (primary):** Each event digs deeper into the deck, finding more events to chain. Storm fuel on a fundamentally different axis than Starcatcher (energy) or Smoldering Echoes (void counting).
- **Depths (secondary):** Repeated foresee is excellent card selection for control, smoothing draws to find answers.

---

### Card 7: Watcher of the Fathoms

**Archetype:** Depths (Tide+Stone)

| Property | Value |
|----------|-------|
| Cost | 4 energy |
| Type | Character |
| Subtype | Ancient |
| Spark | 0 |
| Rarity | Rare |
| Resonance | Tide + Stone |

**Ability text:**
Judgment: Kindle 1 for each card in your hand above 3.

**V3 change:** Merged two completely unrelated abilities (hand-size kindle + prevent-trigger energy) into one continuous scaling ability. No binary threshold, no separate prevent trigger. Prevents are indirectly rewarded because they're cards you hold in hand. The card reads as one idea: "the deeper your hand, the stronger this gets."

**Archetype fit:**
- **Depths (primary):** Depths hoards cards through Tide draw and holds Prevents. With 5-7 cards in hand, kindles 2-4 per Judgment. Creates real tension: every card played shrinks the kindle.
- **Tempest (secondary):** Storm turns that draw many cards can temporarily inflate hand size before deploying.

---

### Card 8: Windstride Runner

**Archetype:** Gale (Zephyr+Ember)

| Property | Value |
|----------|-------|
| Cost | 2 energy |
| Type | Fast Character |
| Subtype | Visitor |
| Spark | 1 |
| Rarity | Uncommon |
| Resonance | Zephyr + Ember |

**Ability text:**
When you play a fast card, this character gains +1 spark until end of turn.

**V3 change:** Removed disconnected hellbent bonus and sacrifice-for-draw activated ability. Replaced with a single triggered ability that directly rewards chaining fast cards. No conditions, no activated costs, no spark restrictions on targets.

**Archetype fit:**
- **Gale (primary):** Quintessential Gale signpost. Snowballs spark during burst turns of fast cards. Pairs with Musician engine (Sage draws into more fast cards, pumping this further).
- **Cinder (secondary):** Fast events like Pyrokinetic Surge already in Cinder's toolkit. Provides spark pressure alongside kindle voltron.

---

### Card 9: Voidthread Weaver

**Archetype:** Eclipse (Zephyr+Ruin)

| Property | Value |
|----------|-------|
| Cost | 3 energy |
| Type | Character |
| Subtype | Explorer |
| Spark | 1 |
| Rarity | Rare |
| Resonance | Zephyr + Ruin |

**Ability text:**
When you discard a card, draw a card.

**V3 change:** Removed both abilities (discard-to-grant-Reclaim + void-event-play draw). Replaced with a single static triggered ability. Eliminates redundancy with Ashmaze Guide (no longer granting Reclaim). Every discard becomes card-neutral, turning cycling into pure selection.

**Archetype fit:**
- **Eclipse (primary):** The missing engine piece. Fragments of Vision (draw 3, discard 2) becomes draw 5. Stacks with other discard payoffs (Mother of Flames kindles, Apocalypse Vigilante scores points).
- **Bedrock (secondary):** Discarding characters to void for reanimation setup becomes hand-size-neutral.

---

### Card 10: Roothold Keeper *(kept as-is)*

**Archetype:** Bedrock (Stone+Ruin)

| Property | Value |
|----------|-------|
| Cost | 2 energy |
| Type | Character |
| Subtype | Survivor |
| Spark | 0 |
| Rarity | Uncommon |
| Resonance | Stone + Ruin |

**Ability text:**
You may only play this character from your void. When you materialize this character from your void, gain 1 energy and draw 1.

**V3 verdict:** KEEP AS-IS. Clean void-only restriction + ETB reward. Two clauses thematically unified — the restriction is a defining constraint, the ETB is the payoff. Simple and readable.

---

## SECTION 2: MONO-STONE CARDS (5 cards)

---

### Card 11: Ironveil Watcher *(kept as-is)*

| Property | Value |
|----------|-------|
| Cost | 2 energy |
| Type | Character |
| Subtype | Ancient |
| Spark | 0 |
| Rarity | Uncommon |
| Resonance | Stone |

**Ability text:**
Judgment: Gain 1 point for each other Judgment ability that triggered this phase, up to 3.

**V3 verdict:** KEEP AS-IS. Clean single Judgment trigger, well-capped. Novel "Judgment storm" concept that rewards wide boards with Judgment abilities.

---

### Card 12: Stoneheart Veteran *(kept as-is)*

| Property | Value |
|----------|-------|
| Cost | 3 energy |
| Type | Character |
| Subtype | Warrior |
| Spark | 1 |
| Rarity | Uncommon |
| Resonance | Stone |

**Ability text:**
Judgment: You may pay 3 energy to kindle 2.

**V3 verdict:** KEEP AS-IS. Clean optional Judgment ability. Non-tribal energy sink on a Warrior body. Balance audit rated Appropriate.

---

### Card 13: Vanguard of the Summit *(implementation note)*

| Property | Value |
|----------|-------|
| Cost | 4 energy |
| Type | Character |
| Subtype | Mage |
| Spark | 2 |
| Rarity | Rare |
| Resonance | Stone |

**Ability text (v2):**
When you play your third character this turn, draw 2 and gain 2 energy.

**V3 note:** The `MaterializeNthThisTurn` trigger exists in the codebase but is not yet implemented (has a `todo!()` in trigger_queries.rs). Design is clean — single threshold trigger with clear reward. If implementation is deferred, a simpler alternative is:

**Alternative ability:** When you materialize another character, this character gains +1 spark.

This uses the already-implemented `Materialize(Predicate)` trigger, maintains the deployment-synergy theme, and rewards the same play pattern with less explosive but more consistent scaling.

---

### Card 14: Deepvault Keeper *(kept as-is)*

| Property | Value |
|----------|-------|
| Cost | 3 energy |
| Type | Character |
| Subtype | Explorer |
| Spark | 1 |
| Rarity | Uncommon |
| Resonance | Stone |

**Ability text:**
Characters you play from your void cost 1 less.

**V3 verdict:** KEEP AS-IS. Cleanest card in the batch. One sentence, one static ability. Economic bridge for Bedrock.

---

### Card 15: Everhold Protector *(implementation note)*

| Property | Value |
|----------|-------|
| Cost | 2 energy |
| Type | Character |
| Subtype | Ancient |
| Spark | 1 |
| Rarity | Common |
| Resonance | Stone |

**Ability text (v2):**
At the start of your turn, if this character has been on the battlefield since your last turn, kindle 1.

**V3 note:** Design is clean and thematically strong — the "anchor effect" is a novel mechanic with intentional anti-synergy with Zephyr flicker. However, the rules engine does not currently support "start of your turn" triggers (only "end of your turn"). The "stayed in play" condition check also needs implementation. The card's strategic role is sound; implementation details need resolution.

---

## SECTION 3: CROSS-POLLINATION CARDS (4 cards)

---

### Card 16: Ashen Threshold

| Property | Value |
|----------|-------|
| Cost | 3 energy |
| Type | Character |
| Subtype | Outsider |
| Spark | 1 |
| Rarity | Uncommon |
| Resonance | Ember |

**Ability text:**
When an ally leaves play, gain 1 energy and kindle 1.

**V3 change:** Removed the second unrelated ability (once-per-turn void-materialize draw). Unified both payoffs under a single trigger. Every time an ally departs (flickered, sacrificed, bounced, dissolved), the card converts that departure into fuel (energy) and scoring (kindle).

**Archetype fit:**
- **Cinder (primary):** Each sacrifice generates energy + kindle. Direct competition with Infernal Ascendant (kindle 2 on abandon) — weaker per trigger but works on ANY leaves-play, not just abandon.
- **Mirage (secondary):** Every flicker triggers the payoff. Energy refund makes flicker chains self-sustaining; kindle provides something Mirage currently lacks entirely.

---

### Card 17: Voidthorn Protector

| Property | Value |
|----------|-------|
| Cost | 2 energy |
| Type | Character |
| Subtype | Survivor |
| Spark | 1 |
| Rarity | Uncommon |
| Resonance | Ruin |

**Ability text:**
Abandon this character: Prevent a played character. Put the top 2 cards of your deck into your void.

**V3 change:** Collapsed two separate abilities (activated sacrifice-Prevent + passive mill-on-any-Prevent) into a single activated ability. The mill now happens once as part of the resolution, not as an independent engine. Removes the degenerate case of passive mill on every Prevent in the deck.

**Archetype fit:**
- **Cinder (primary):** Self-sacrifice feeds abandon loops while providing a defensive tempo play.
- **Depths (secondary):** Clean character-Prevent that extends the Prevent suite without becoming a passive mill engine.

---

### Card 18: Kindlespark Harvester *(kept as-is)*

| Property | Value |
|----------|-------|
| Cost | 3 energy |
| Type | Character |
| Subtype | Ancient |
| Spark | 0 |
| Rarity | Rare |
| Resonance | Stone |

**Ability text:**
Judgment: You may remove 3 spark from your leftmost ally to dissolve an enemy with cost 3 or less. If you do, kindle 2.

**V3 verdict:** KEEP AS-IS. Both audits praised this as excellent design. Single Judgment ability with novel "spark as spendable resource" mechanic. The spend-3-get-2-back math creates genuine tension.

---

### Card 19: Risen Champion *(kept as-is)*

| Property | Value |
|----------|-------|
| Cost | 4 energy |
| Type | Character |
| Subtype | Warrior |
| Spark | 2 |
| Rarity | Uncommon |
| Resonance | Ruin |

**Ability text:**
You may only play this character from your void. Dissolved: Reclaim a different Warrior from your void.

**V3 verdict:** KEEP AS-IS. Cohesive void-warrior fantasy. Void-only restriction is a defining constraint, not a separate ability. "A different Warrior" prevents self-loops. Clean Crucible-Bedrock bridge.

---

## SECTION 4: MODULAR ENGINE CARDS (3 cards)

---

### Card 20: Dreamtide Cartographer

| Property | Value |
|----------|-------|
| Cost | 3 energy |
| Type | Character |
| Subtype | Explorer |
| Spark | 1 |
| Rarity | Uncommon |
| Resonance | Tide |

**Ability text:**
Judgment: Foresee 2, then draw a card.

**V3 change:** Removed the hand-size threshold and conditional mode switch (draw vs. spark pump). Replaced with unconditional "Foresee 2, then draw a card" at Judgment. Eliminates the disconnected dual-identity problem. The card no longer pretends to serve non-Tide archetypes (Gale and Eclipse were flagged as inaccessible anyway).

**Archetype fit:**
- **Undertow (primary):** Foresee feeds cards into void for void-size payoffs. Draw keeps the engine flowing.
- **Mirage (secondary):** Flickering resets for additional Judgment triggers. Consistent card flow finds bounce/flicker pieces.
- **Depths (tertiary):** Steady incremental card advantage for the long game.

---

### Card 21: Nexus of Passing

| Property | Value |
|----------|-------|
| Cost | 4 energy |
| Type | Character |
| Subtype | Ancient |
| Spark | * |
| Rarity | Rare |
| Resonance | Neutral |

**Ability text:**
This character's spark is equal to the number of other allied characters.

**V3 change:** Replaced "count zone transitions this turn" (tracking nightmare — required counting every battlefield entry/exit throughout the turn) with a static board-state reference (count other allies). Zero tracking burden. The natural cap is the board itself (typically 3-5 spark) rather than an artificial "up to 8" rider.

**Archetype fit:**
- **Basalt:** Go-wide with cheap Spirit Animals. Consistently hits 4-5 spark.
- **Mirage:** Flicker keeps board full while generating ETB value. Maintains board width.
- **Crucible:** Warrior deployment rewards. Non-Warrior body creates Blade of Unity tension.
- **Cinder:** Rewards the "rebuild" half of sacrifice-recursion loops.

---

### Card 22: Archivist of Vanished Names *(kept as-is)*

| Property | Value |
|----------|-------|
| Cost | 3 energy |
| Type | Character |
| Subtype | Mage |
| Spark | 1 |
| Rarity | Rare |
| Resonance | Tide |

**Ability text:**
Materialized: Name a card type (Character or Event). Reveal cards from the top of your deck until you reveal a card of that type. Put the revealed card into your hand and the rest into your void.

**V3 verdict:** KEEP AS-IS. Both audits praised this as one of the best-designed cards in the batch. Binary named-type choice creates the most archetype-dependent evaluations from the simplest possible modal. "One of the best names in the batch."

---

## SECTION 5: GAP FILLER CARDS (3 cards)

---

### Card 23: Ironbark Warden *(kept as-is)*

| Property | Value |
|----------|-------|
| Cost | 4 energy |
| Type | Character |
| Subtype | Ancient |
| Spark | 2 |
| Rarity | Rare |
| Resonance | Stone |

**Ability text:**
Judgment: Each ally that has been on the battlefield since your last turn gains +1 spark until end of turn.

**V3 verdict:** KEEP AS-IS. Clean single Judgment ability. Deliberately narrow (Crucible + Depths only) — this is intentional, creating "purity vs. power" tension for Crucible (Ancient dilutes Blade of Unity) and a finisher for Depths. Anti-synergy with Zephyr flicker is an intentional learning trap.

---

### Card 24: Tidechannel Observer *(kept as-is)*

| Property | Value |
|----------|-------|
| Cost | 3 energy |
| Type | Character |
| Subtype | Ancient |
| Spark | 1 |
| Rarity | Uncommon |
| Resonance | Ruin |

**Ability text:**
Judgment: If 3 or more cards entered your void this turn, gain 2 points and kindle 1.

**V3 verdict:** KEEP AS-IS. Both audits praised this as the most important mechanical innovation in the batch. "Void velocity" cleanly differentiates Undertow (trivially hits 3/turn) from Bedrock (rarely hits 3/turn). Non-Survivor typing creates meaningful draft tension.

---

### Card 25: Duskwatch Vigil *(kept as-is with note)*

| Property | Value |
|----------|-------|
| Cost | 2 energy |
| Type | Character |
| Subtype | Outsider |
| Spark | 2 |
| Rarity | Uncommon |
| Resonance | Tide |

**Ability text:**
When you prevent a card, your next event this turn costs 2 less.

**V3 verdict:** KEEP AS-IS. Clean single triggered ability. Converts Prevents into tempo. One agent suggested changing resonance to Tide+Stone dual for better Depths signposting, but this would change the resonance distribution. Worth considering in a future pass.

**Known issue:** V2 integration review flagged that Gale (Zephyr+Ember) cannot access mono-Tide cards despite being listed as a target archetype. Gale should be removed from this card's archetype tags.

---

## V2 → V3 CHANGE SUMMARY

| # | Card | V3 Status | Key Change |
|---|------|-----------|------------|
| 1 | Tideweaver Adept | **REDESIGNED** | Modal choice → single scaling draw on materialize |
| 2 | Abyssal Reclaimer | **REDESIGNED** | Complex conditional reclaim → unconditional Survivor retrieval |
| 3 | Stoneveil Guardian | **REDESIGNED** | Conditional mode switch → unconditional Spirit Animal pump |
| 4 | Forgeborn Martyr | **REDESIGNED** | Two stapled abilities → single Judgment sacrifice-or-kindle choice |
| 5 | Ashfire Ritualist | **REDESIGNED** | Two rate-limited abilities → single abandon trigger (kindle + reclaim) |
| 6 | Stormtide Oracle | **REDESIGNED** | Variable spark + retrieval → "play event: foresee 2" (fixed 2 spark) |
| 7 | Watcher of the Fathoms | **REDESIGNED** | Two unrelated abilities → continuous hand-size scaling kindle |
| 8 | Windstride Runner | **REDESIGNED** | Hellbent + sacrifice → "play fast card: +1 spark" |
| 9 | Voidthread Weaver | **REDESIGNED** | Multi-step Reclaim engine → "discard: draw" |
| 10 | Roothold Keeper | KEEP | — |
| 11 | Ironveil Watcher | KEEP | — |
| 12 | Stoneheart Veteran | KEEP | — |
| 13 | Vanguard of the Summit | KEEP (impl. note) | Trigger not yet implemented; simpler alternative offered |
| 14 | Deepvault Keeper | KEEP | — |
| 15 | Everhold Protector | KEEP (impl. note) | "Start of turn" trigger needs engine support |
| 16 | Ashen Threshold | **REDESIGNED** | Two unrelated engines → single "leaves play: energy + kindle" |
| 17 | Voidthorn Protector | **REDESIGNED** | Two abilities → single activated "sacrifice self: Prevent + mill" |
| 18 | Kindlespark Harvester | KEEP | — |
| 19 | Risen Champion | KEEP | — |
| 20 | Dreamtide Cartographer | **REDESIGNED** | Threshold mode switch → unconditional "foresee 2, draw 1" |
| 21 | Nexus of Passing | **REDESIGNED** | Zone-transition tracking → "spark = other allied characters" |
| 22 | Archivist of Vanished Names | KEEP | — |
| 23 | Ironbark Warden | KEEP | — |
| 24 | Tidechannel Observer | KEEP | — |
| 25 | Duskwatch Vigil | KEEP (note) | Remove Gale from archetype tags (inaccessible) |

---

## DESIGN QUALITY COMPARISON: V2 vs V3

| Metric | V2 | V3 |
|--------|----|----|
| Cards with 2+ unrelated abilities | 10 | 0 |
| Cards with "once per turn" rate limiters | 3 | 0 |
| Cards with conditional mode switches | 3 | 1 (Forgeborn Martyr, but player-chosen) |
| Cards with tracking burden beyond board state | 3 | 0 |
| Average ability text length (words) | ~25 | ~12 |
| Cards needing 2+ sentences to explain | 13 | 3 |
