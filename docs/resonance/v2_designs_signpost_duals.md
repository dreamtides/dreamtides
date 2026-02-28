# V2 Signpost Dual Card Designs -- Agent A

## Design Philosophy

These 10 cards are the ONLY dual-resonance signpost cards for their respective archetype pairs. Each card must:
- Be desirable in 2+ archetypes for genuinely different mechanical reasons
- Create decision points rather than passive value
- Avoid stapled designs (Resonance A keyword + Resonance B keyword, no emergent third behavior)
- Exploit the counter-patterns from the mechanic critique (named-type choice, threshold-gated modes, cross-zone scaling, conditional commons boost, board-state-reading dual modes)
- Be slightly above-rate for their cost (worth building around, not format-warping)

---

## Card 1: Tideweaver Sentinel

**Archetype:** Mirage (Tide+Zephyr)

| Property | Value |
|----------|-------|
| Name | Tideweaver Sentinel |
| Cost | 3 energy |
| Type | Character |
| Subtype | Visitor |
| Spark | 1 |
| Rarity | Uncommon |
| Resonance | Tide + Zephyr |

**Ability text:**
Materialized: Choose one -- Draw 2, or return an ally to hand and gain 2 energy.

**Synergy explanation:**

- **Mirage (primary):** Both modes are excellent. The draw mode generates card advantage on every flicker. The bounce mode returns another Materialized-value body to hand while generating energy to replay it, creating a self-fueling flicker loop. The *choice* between modes matters: early game you draw for cards, mid-game you bounce for re-triggers. Every flicker of this card presents a real decision.
- **Basalt (secondary):** The bounce mode returns a Spirit Animal to hand for re-deployment, triggering materialize-matters payoffs. The energy gain offsets the re-play cost. The draw mode feeds Seeker of the Radiant Wilds.
- **Depths (tertiary):** A 3-cost body with Materialized draw-2 is a strong control value piece even without flicker. The bounce mode can save a threatened body.

**V1 problem addressed:** The QA report noted Mirage had 3 duals (below the allied-pair target of 4-6). This fills a gap. The modal design addresses the mechanic critique's Pattern 6 complaint (zero new cards had choose-one modality).

**Synergy vectors exploited:** V19 (Leaves-Play Payoffs -- the bounce mode causes an ally to leave play, triggering Starlit Cascade), V04 (the draw mode feeds hand-based strategies).

**Power level reference:** Frost Visionary is 2 cost, 2 spark, Materialized: Draw 1. Tideweaver Sentinel costs 1 more, has 1 less spark, but offers a choice of Draw 2 OR bounce+energy. The modal flexibility justifies the cost increase; the lower spark prevents it from being a superior finisher body.

---

## Card 2: Abyssal Reclaimer

**Archetype:** Undertow (Tide+Ruin)

| Property | Value |
|----------|-------|
| Name | Abyssal Reclaimer |
| Cost | 3 energy |
| Type | Character |
| Subtype | Survivor |
| Spark | 1 |
| Rarity | Uncommon |
| Resonance | Tide + Ruin |

**Ability text:**
Materialized: Put the top 3 cards of your deck into your void. If 2 or more cards entered your void this turn, you may return a card from your void to your hand.

**Synergy explanation:**

- **Undertow (primary):** The self-mill grows the void for Abomination of Memory and Weight of Memory. The threshold of "2 or more cards entered your void this turn" is trivially met by the card itself (it mills 3), so it functions as mill-3-then-Reclaim-1. However, this is only guaranteed when *this* card triggers the mill -- if the card enters play after other void-filling, the threshold is already met and the Reclaim is free. The Survivor typing adds tribal density for Hope's Vanguard and Soulkindler.
- **Eclipse (secondary):** Eclipse fills the void through discard effects. After a discard cycle, this card's Materialized trigger adds more void fuel AND retrieves a key card. The threshold is met by any combination of discards + mill.
- **Mirage (tertiary):** This is a strong flicker target -- each flicker mills 3 and retrieves 1, creating a net-2-mill engine with card selection. Flicker decks that splash Ruin for recursion tools want this body.

**V1 problem addressed:** QA report noted Undertow had 4 duals (meets allied minimum) but the archetype could benefit from a non-Survivor-lord Tide+Ruin signpost that bridges to other archetypes. This card's threshold mechanic makes it work differently in each archetype rather than being a linear tribal piece.

**Synergy vectors exploited:** V29 (Void Velocity -- the threshold rewards high void-entry rates), V30 (From-Void Materialized -- the Reclaim enables void-to-hand cycling).

**Power level reference:** Searcher in the Mists is 2 cost, 1 spark, Materialized/Dissolved: Mill 4. Abyssal Reclaimer costs 1 more for mill-3 + conditional Reclaim. The Reclaim is genuinely powerful, but the card costs more and mills 1 fewer. The conditional Reclaim at a higher cost is a fair trade-up.

---

## Card 3: Basalt Warden

**Archetype:** Basalt (Zephyr+Stone)

| Property | Value |
|----------|-------|
| Name | Basalt Warden |
| Cost | 3 energy |
| Type | Character |
| Subtype | Spirit Animal |
| Spark | 1 |
| Rarity | Uncommon |
| Resonance | Zephyr + Stone |

**Ability text:**
Judgment: If you have 3 or more allies, gain 2 energy. Otherwise, each ally gains +1 spark until end of turn.

**Synergy explanation:**

- **Basalt (primary):** Basalt builds wide Spirit Animal boards. With 3+ allies, this generates 2 energy per Judgment to fuel activated abilities (Spiritbound Alpha's 4-energy activation, Mystic Runefish's 3-energy activation). When the board is thin (early game, after removal), the spark boost makes each remaining body more relevant for scoring. The mode switch is automatic but creates genuine board-state reading -- the card rewards you differently based on your position.
- **Crucible (secondary):** Warrior boards naturally hit 3+ allies through cheap deployment (Wolfbond Chieftain, Aspiring Guardian, Ethereal Trailblazer). The 2-energy generation fuels Assault Leader's 4-energy activation. When Warriors are removed, the +1 spark mode keeps remaining lords threatening. Not a Warrior itself, so it creates the "purity vs. power" tension the synergy map demands for Crucible.
- **Mirage (tertiary):** Wide flicker boards often have 3+ allies. The energy mode funds more flicker plays.

**V1 problem addressed:** QA report flagged Basalt at 3 duals (below the allied minimum of 4). Verdant Packmother was a tribal lord with no off-tribe application (Pattern 4 from mechanic critique). This replacement uses a threshold-gated mode switch (Counter-Pattern 2) that reads the board state, creating different value depending on context. The Spirit Animal typing maintains tribal density without the card being tribal-locked.

**Synergy vectors exploited:** V03 (Board-Width-from-Low-Spark-Bodies -- the 3+ allies threshold rewards wide boards), V21 (Judgment Storm -- the energy generation is a Judgment trigger that contributes to Judgment density), V34 (Burst Energy into Activated -- the energy fuels expensive activated abilities).

**Power level reference:** Ghostlight Wolves is 3 cost, 1 spark, Judgment: Gain 1 energy per Spirit Animal. Basalt Warden costs the same and generates 2 energy with 3+ allies (any ally, not just Spirit Animals) OR gives spark. It is more flexible but less synergistic with pure Spirit Animal density -- Ghostlight Wolves is better when you have 4+ Spirit Animals specifically, while Basalt Warden is better in mixed boards.

---

## Card 4: Forgeborn Martyr

**Archetype:** Crucible (Stone+Ember)

| Property | Value |
|----------|-------|
| Name | Forgeborn Martyr |
| Cost | 3 energy |
| Type | Character |
| Subtype | Warrior |
| Spark | 2 |
| Rarity | Rare |
| Resonance | Stone + Ember |

**Ability text:**
Judgment: Each allied Warrior gains +1 spark until end of turn. When an allied Warrior is dissolved by the opponent, draw 1 and gain 1 energy.

**Synergy explanation:**

- **Crucible (primary):** The Judgment pump makes your entire Warrior board more threatening each scoring phase -- this is the lord effect Crucible wants. But the dissolved-trigger creates a genuine dilemma for the opponent: leaving Warriors alive means they accumulate Judgment value; killing them replaces themselves with cards and energy. The opponent faces a lose-lose, and the Crucible player must decide whether to invest in board width (more Warriors to pump) or accept that dead Warriors fund the next wave. This is NOT stapled design -- the two abilities create an emergent "damned if you do, damned if you don't" pattern that neither ability creates alone.
- **Cinder (secondary):** In a Warrior-sacrifice build with Grim Reclaimer and The Dread Sovereign, the dissolved trigger fires when you sacrifice Warriors (they are dissolved, and "by the opponent" is NOT required for the draw -- wait, it IS "by the opponent"). Actually, this means Cinder does NOT benefit from self-sacrifice, which is correct per the design constraint (Stone must NOT gain self-sacrifice payoffs). Cinder benefits only if the opponent kills their Warriors, which is a subtle defensive angle.
- **Bedrock (tertiary):** Warriors that die and return through Ruin's recursion (Ashen Avenger's self-Reclaim, Grim Reclaimer's Warrior Reclaim) create a "Warrior reanimator" sub-build. Forgeborn Martyr makes each Warrior death less painful, encouraging the risky Bedrock strategy of letting expensive Warriors die and returning them.

**V1 problem addressed:** QA flagged Crucible at 9/10 rails and 3 duals (below allied minimum of 4). Warbond Sentinel was a 5/10 subtlety card that created opponent tension but no player decisions. Forgeborn Martyr preserves the opponent-facing tension (V22: Warrior Death Dividends) while adding the Judgment pump (V21: Judgment Storm) that rewards board investment. The card creates a decision web: invest in board width for Judgment pumps, or accept attrition for card advantage.

**Synergy vectors exploited:** V22 (Warrior Death Dividends -- the dissolved trigger), V21 (Judgment Storm -- the Judgment pump adds to Judgment density), V23 (Deployment Storm -- the energy from dead Warriors funds more deployments).

**Power level reference:** Skyflame Commander is 2 cost, 1 spark, "Allied Warriors have +1 spark" (permanent). Forgeborn Martyr costs 1 more for temporary +1 spark on Judgment plus the death-draw-energy rider. The temporary pump is weaker than Skyflame's permanent buff, but the death trigger compensates. At 3 cost and 2 spark, this is a fair package -- stronger than Skyflame in attrition games, weaker in pure racing.

---

## Card 5: Cinder Ritualist

**Archetype:** Cinder (Ember+Ruin)

| Property | Value |
|----------|-------|
| Name | Cinder Ritualist |
| Cost | 3 energy |
| Type | Character |
| Subtype | Outsider |
| Spark | 1 |
| Rarity | Rare |
| Resonance | Ember + Ruin |

**Ability text:**
When a card enters your void from any zone, kindle 1. Once per turn, you may abandon an ally: return a different character from your void to your hand.

**Synergy explanation:**

- **Cinder (primary):** The two abilities create a sacrifice-recursion engine. Abandon an ally (card enters void -> kindle 1), then retrieve a different character from void to sacrifice again next turn (retrieved character does NOT enter void, so no kindle). The kindle accumulates on the leftmost character, building a voltron threat. Each sacrifice loop nets 1 kindle and churns through the sacrifice roster. The "different character" clause prevents infinite same-card loops.
- **Undertow (secondary):** Self-mill puts 3-6 cards into void per turn, each triggering kindle 1. A single Harvest the Forgotten (mill 3) generates kindle 3. The retrieval ability recovers key milled Survivors. This is a DIFFERENT use pattern -- Undertow does not sacrifice, it mills, and the kindle scales with mill velocity.
- **Eclipse (tertiary):** Each discard triggers kindle 1. Draw-then-discard cycles (Fragments of Vision discards 2 = kindle 2) build spark passively. The retrieval recovers discarded cards.

**V1 problem addressed:** QA flagged Cinder at 3 duals (below allied minimum of 4). The synergy map identified V15 (Abandon-as-Pseudo-Flicker) and V16 (Kindle Concentration) as priority vectors. This card creates a sacrifice loop that generates kindle (V16) while the "card enters void" trigger makes it usable in non-sacrifice archetypes (V29: Void Velocity). The mechanic critique demanded cards whose function changes by archetype context -- this card is a sacrifice-kindle engine in Cinder, a mill-kindle engine in Undertow, and a discard-kindle engine in Eclipse.

**Synergy vectors exploited:** V16 (Kindle Concentration -- the kindle from void entry builds a voltron character), V29 (Void Velocity -- faster void entry = faster kindle), V15 (Abandon-as-Pseudo-Flicker -- the retrieve-from-void enables repeated materialization triggers).

**Power level reference:** Infernal Ascendant is 3 cost, 1 spark, "When you abandon an ally, kindle 2." Cinder Ritualist kindles 1 per void entry (including the sacrifice) and adds retrieval. In a pure sacrifice turn, Infernal Ascendant kindles more per sacrifice (2 vs 1), but Cinder Ritualist kindles from ALL void entries and provides recursion. It is more versatile but less explosive in the specific sacrifice axis.

---

## Card 6: Stormtide Oracle

**Archetype:** Tempest (Tide+Ember)

| Property | Value |
|----------|-------|
| Name | Stormtide Oracle |
| Cost | 4 energy |
| Type | Character |
| Subtype | Ancient |
| Spark | * |
| Rarity | Rare |
| Resonance | Tide + Ember |

**Ability text:**
This character's spark is equal to the number of events in your void. Judgment: You may pay 2 energy to return an event from your void to your hand.

**Synergy explanation:**

- **Tempest (primary):** Tempest plays 8-15 events per game; they all resolve to the void. This character scales with cumulative event history, NOT with single-turn storm count. By mid-game, 5-7 events in void makes this a 5-7 spark body -- a powerful finisher. The Judgment retrieval recovers key storm pieces (Genesis Burst, Cascade of Reflections) for the next explosive turn. The tension: retrieving events shrinks the void (reducing spark), so you must decide whether the event in hand is worth more than the spark on the body.
- **Depths (secondary):** Control decks play many Prevent events (9 in the pool). Each Prevent that resolves enters the void, growing this character. By late game, 4-6 Prevents in void makes this a reliable finisher. The retrieval recovers spent Prevents for reuse. Depths wants this as a proactive win condition that emerges from its reactive game plan.
- **Undertow (tertiary):** Self-mill puts events into void incidentally. This body grows from milled events without needing to cast them. The event-retrieval recovers milled events.

**V1 problem addressed:** The mechanic critique identified Pattern 5 (Tempest overconcentration on "2+ events this turn" trigger -- 4/5 Tempest cards used the same template). This card uses V06 (Event-Count-in-Void), which rewards CUMULATIVE event play rather than single-turn storm count. It creates a fundamentally different Tempest axis -- late-game scaling through event history. The retrieval creates a real decision (event in hand vs. spark on body) that Pattern 1 cards lacked.

**Synergy vectors exploited:** V06 (Event-Count-in-Void -- spark scales with voided events), V35 (Removal as Storm Count -- removal events in void grow this body).

**Power level reference:** Spirit of Smoldering Echoes is 4 cost, * spark, "When an event is put into your void, +1 spark." Both scale with events in void. The difference: Smoldering Echoes grows in real-time (each event entering void grows it immediately), while Stormtide Oracle has a static count (counts events currently in void, so retrieval reduces spark). Stormtide Oracle adds the retrieval ability, making it a more complete package but with the interesting retrieval-vs-spark tension. The two cards occupy different strategic niches and are both desirable in Tempest.

---

## Card 7: Depthswatcher

**Archetype:** Depths (Tide+Stone)

| Property | Value |
|----------|-------|
| Name | Depthswatcher |
| Cost | 4 energy |
| Type | Character |
| Subtype | Ancient |
| Spark | 0 |
| Rarity | Rare |
| Resonance | Tide + Stone |

**Ability text:**
Judgment: If you have 5 or more cards in hand, kindle 2. When you prevent a card, gain 2 energy.

**Synergy explanation:**

- **Depths (primary):** This card embodies the Depths archetype's two pillars: hand advantage (Tide) and resource accumulation (Stone). The kindle-2-on-Judgment rewards maintaining a full hand -- which Tide's draw naturally provides. The Prevent-trigger converts reactive denial into proactive investment, transforming each Prevent from "I stopped your thing" into "I stopped your thing AND gained 2 energy." With 9 Prevent effects in the pool, the energy generation is meaningful. The kindle builds the leftmost character into a finisher -- this is Depths' missing proactive win condition.
- **Tempest (secondary):** Storm turns involve drawing many cards, often holding 5+ before deploying them as a chain. The kindle-2 fires on the Judgment following a draw-heavy setup turn. The Prevent-trigger is less relevant but the Abolish/Ripple of Defiance that Tempest splashes for protection now generates energy for the next storm turn.
- **Basalt (tertiary):** Stone-heavy Basalt boards hold up energy for activated abilities, sometimes accumulating cards. The Prevent-trigger is marginal here, but the kindle reward for hand size is reachable in slow Basalt builds that ramp before deploying.

**V1 problem addressed:** QA flagged Depths at 1 dual signpost (CRITICAL failure -- needs 3-4 for enemy pairs). Tidestone Arbiter was the only Depths dual and scored 5/10 on subtlety (clean but obvious stapled design). Depthswatcher exploits V01 (Hand-Size-Matters -- the most impactful untapped vector with breadth 4) and V02 (Prevent-Trigger Payoffs -- the second most impactful Depths vector). The two abilities create emergent behavior: Prevent events keep you reactive (maintaining hand size) while generating energy AND building a kindle finisher. This is a card that makes you want to play the slow, reactive Depths game and rewards you with a proactive win condition.

**Synergy vectors exploited:** V01 (Hand-Size-Matters -- kindle-2 threshold at 5+ cards), V02 (Prevent-Trigger Payoffs -- energy on Prevent), V05 (Unspent Energy at EOT -- the Prevent energy compounds with Stone's ramp).

**Power level reference:** Tidestone Arbiter is 4 cost, Ancient, "Judgment: Draw 1. Opponent's characters cost 1 more." Depthswatcher has the same cost and type but 0 starting spark (vs. Tidestone's unspecified-but-implied spark). Depthswatcher's kindle-2 per Judgment is MUCH stronger than Draw 1 when the hand condition is met, but it requires maintaining 5+ cards. The Prevent-energy is strong but requires running many Prevents. The card is powerful when conditions are met but requires genuine deckbuilding commitment, making it above-rate in its home deck and below-rate elsewhere.

---

## Card 8: Galerunner

**Archetype:** Gale (Zephyr+Ember)

| Property | Value |
|----------|-------|
| Name | Galerunner |
| Cost | 2 energy |
| Type | Fast Character |
| Subtype | Visitor |
| Spark | 1 |
| Rarity | Uncommon |
| Resonance | Zephyr + Ember |

**Ability text:**
While you have 2 or fewer cards in hand, this character has +2 spark. Abandon an ally with 0 spark: Draw 1.

**Synergy explanation:**

- **Gale (primary):** Gale deploys fast threats aggressively, emptying the hand. With 0-2 cards remaining, Galerunner becomes a 3-spark threat for 2 energy -- an exceptional rate. The abandon ability converts spent figments and 0-spark utility bodies (Wolfbond Chieftain, Ethereal Trailblazer) into cards, preventing the fast-tempo deck from running out of gas. The tension: playing cards to empty your hand makes Galerunner bigger, but the draw ability refills your hand, potentially turning off the bonus. This creates genuine moment-to-moment decisions about whether to draw or keep the spark.
- **Cinder (secondary):** Cinder produces 0-spark bodies through figments (Packcaller of Shadows) and can empty its hand through aggressive abandon costs. The sacrifice-for-draw provides card advantage in a resource-hungry archetype. Cinder does not care about the spark bonus specifically but uses the card as a sacrifice outlet that replaces itself.
- **Eclipse (tertiary):** Eclipse discards to empty the hand, triggering the spark bonus. The 0-spark abandon converts low-value bodies into cards after discard cycles.

**V1 problem addressed:** QA flagged Gale at 1 dual signpost (CRITICAL failure). Tempest Striker scored 4/10 subtlety ("fourth copy of Musician pattern"). Galerunner uses V14 (Hellbent Payoffs -- the most impactful untapped vector for Gale) combined with V18 (Figment Sacrifice Bridge -- the 0-spark abandon converts figments to value). The hellbent threshold creates clean anti-synergy with Tide (which always has cards), making resonance separation explicit. The card is a TOOL that rewards empty-hand aggression, not a LABEL that says "play fast cards."

**Synergy vectors exploited:** V14 (Hellbent Payoffs -- +2 spark at 2 or fewer cards), V18 (Figment Sacrifice Bridge -- abandon 0-spark figments for draw), V10 (Materialized Bodies as Sacrifice Fodder -- 0-spark Materialized-draw bodies become draw fuel).

**Power level reference:** Horizon Follower is a Fast Visitor at 2 cost, 1 spark, Judgment: Gain 1 point. Galerunner has the same base stats but gains +2 spark conditionally and has an activated ability. The conditional spark makes it stronger in Gale (where hand is often low) but weaker in slow decks (where hand is full). The abandon ability is a genuine cost (requires a 0-spark body). At 2 cost for a conditional 3-spark fast body with upside, this is above-rate in its home deck and below-rate elsewhere.

---

## Card 9: Eclipse Weaver

**Archetype:** Eclipse (Zephyr+Ruin)

| Property | Value |
|----------|-------|
| Name | Eclipse Weaver |
| Cost | 3 energy |
| Type | Character |
| Subtype | Explorer |
| Spark | 2 |
| Rarity | Rare |
| Resonance | Zephyr + Ruin |

**Ability text:**
Once per turn, discard a card: An event in your void gains Reclaim equal to its cost this turn. When you play an event from your void, draw 1.

**Synergy explanation:**

- **Eclipse (primary):** This is Eclipse's event-recycling engine. Discard a card (Zephyr cycling), give an event in your void Reclaim (Ruin recursion), then when you play that event from the void, draw 1. The loop: discard -> grant Reclaim -> play from void -> draw -> discard again next turn. Each cycle triggers discard payoffs (Mother of Flames kindles, Apocalypse Vigilante gains points) AND generates card flow. The "different card" implicit in discard-one-retrieve-one prevents infinite loops. The card creates a sustainable event-cycling engine that no other Ruin archetype wants because it requires discard infrastructure.
- **Tempest (secondary):** Event Reclaim from void provides storm fuel recovery. The draw-on-void-event rewards replaying cheap events. A Tempest deck that fills the void with events can use this to replay key pieces. The discard cost is manageable when you are drawing 3-6 cards per turn.
- **Bedrock (tertiary):** Ashmaze Guide already grants Reclaim on discard; Eclipse Weaver does the same for events specifically and adds a draw reward. In a Ruin deck with expensive events (Path to Redemption at 6 cost), this grants them Reclaim and draws when replayed.

**V1 problem addressed:** QA flagged Eclipse at "Moderate to Strong" depth with a thin payoff layer (only 6 discard payoffs). The synergy map identified V33 (Event Reclaim Engine) as a priority vector for Eclipse. Voidweave Dancer (v1 design) scored 3/10 subtlety as a linear "discard -> grow" payoff. Eclipse Weaver creates a genuine engine (discard -> Reclaim -> replay -> draw) that rewards deckbuilding around event density and discard outlets. The draw-on-void-event is a cross-archetype hook that Tempest and Bedrock both value.

**Synergy vectors exploited:** V33 (Event Reclaim Engine -- the discard-to-Reclaim cycle), V06 (Event-Count-in-Void -- events move through the void, triggering void-entry effects), V29 (Void Velocity -- each discard is a void entry).

**Power level reference:** Ashmaze Guide is 3 cost, 1 spark, "When you discard a card, it gains Reclaim equal to its cost this turn." Eclipse Weaver costs the same with 1 more spark, but its Reclaim is event-only and requires an activation (once per turn, discard a card). The narrower scope (events only vs. all cards) and activation cost balance the additional draw-on-void-event ability. Ashmaze Guide is better for Bedrock (grants Reclaim to expensive characters); Eclipse Weaver is better for Eclipse (draws when events cycle). They coexist as complementary pieces.

---

## Card 10: Bedrock Anchor

**Archetype:** Bedrock (Stone+Ruin)

| Property | Value |
|----------|-------|
| Name | Bedrock Anchor |
| Cost | 2 energy |
| Type | Character |
| Subtype | Survivor |
| Spark | 0 |
| Rarity | Uncommon |
| Resonance | Stone + Ruin |

**Ability text:**
You may only play this character from your void. When you materialize this character from your void, gain 2 energy and draw 1. Judgment: If this character has been on the battlefield since your last turn, kindle 1.

**Synergy explanation:**

- **Bedrock (primary):** A void-only character that self-funds on entry (gain 2 energy means net 0 cost from void) and replaces itself (draw 1). The Judgment kindle rewards Stone's board permanence -- if Bedrock protects this body, it accumulates spark over time. The card solves Bedrock's core tension: cheap void deployment (Ruin) that generates ramp (Stone) and scales with persistence (Stone). It is a smaller Revenant of the Lost (2 cost vs. 3 cost, 0 spark vs. 6 spark) that compensates with energy generation and card draw on entry.
- **Undertow (secondary):** The Survivor typing gives Undertow tribal density. Self-mill naturally puts this into the void. The 2-energy refund and draw make it a free inclusion that replaces itself. The kindle reward is less relevant in Undertow (which does not focus on board permanence) but is gravy.
- **Crucible (tertiary):** Stone's ramp and Warrior bridges to Ruin (Ashen Avenger, Grim Reclaimer) create a "Warrior Reanimator" sub-build. Bedrock Anchor is not a Warrior, but its energy generation and card draw on void-entry support the economics of this strategy.

**V1 problem addressed:** QA flagged Bedrock as the thinnest archetype (B- grade, 23 cards). The synergy map identified V32 (Void-Only Characters) as a priority vector -- only Revenant of the Lost exists as a void-only body. Bedrock Anchor adds a second void-only character at a different cost point, reducing Bedrock's dependence on a single contested card. The Survivor typing creates an unexpected Undertow bridge (V38: Survivor Spread). The Judgment kindle (V27: Anchor Effect) rewards Stone's permanence theme while being achievable only from the void (Ruin's territory).

**Synergy vectors exploited:** V32 (Void-Only Characters -- second void-gated body), V30 (From-Void Materialized -- the energy and draw trigger on void-to-battlefield), V27 (Anchor Effect -- kindle rewards continuous board presence), V38 (Survivor Spread -- Survivor typing creates Undertow bridge).

**Power level reference:** Revenant of the Lost is 3 cost, 6 spark, void-only. Bedrock Anchor is 2 cost, 0 spark, void-only, with +2 energy +draw on entry and kindle-1 per Judgment. Revenant is a finisher body (6 spark immediately); Bedrock Anchor is an engine piece (0 spark but generates resources and grows slowly). They serve completely different roles and both belong in Bedrock -- Revenant closes games, Bedrock Anchor builds toward them.

---

## Summary Table

| # | Name | Archetype | Resonance | Cost | Type | Subtype | Spark | Rarity | Primary Vector(s) |
|---|------|-----------|-----------|------|------|---------|-------|--------|-------------------|
| 1 | Tideweaver Sentinel | Mirage | Tide+Zephyr | 3 | Character | Visitor | 1 | Uncommon | V19, V04 |
| 2 | Abyssal Reclaimer | Undertow | Tide+Ruin | 3 | Character | Survivor | 1 | Uncommon | V29, V30 |
| 3 | Basalt Warden | Basalt | Zephyr+Stone | 3 | Character | Spirit Animal | 1 | Uncommon | V03, V21, V34 |
| 4 | Forgeborn Martyr | Crucible | Stone+Ember | 3 | Character | Warrior | 2 | Rare | V22, V21, V23 |
| 5 | Cinder Ritualist | Cinder | Ember+Ruin | 3 | Character | Outsider | 1 | Rare | V16, V29, V15 |
| 6 | Stormtide Oracle | Tempest | Tide+Ember | 4 | Character | Ancient | * | Rare | V06, V35 |
| 7 | Depthswatcher | Depths | Tide+Stone | 4 | Character | Ancient | 0 | Rare | V01, V02, V05 |
| 8 | Galerunner | Gale | Zephyr+Ember | 2 | Fast Character | Visitor | 1 | Uncommon | V14, V18, V10 |
| 9 | Eclipse Weaver | Eclipse | Zephyr+Ruin | 3 | Character | Explorer | 2 | Rare | V33, V06, V29 |
| 10 | Bedrock Anchor | Bedrock | Stone+Ruin | 2 | Character | Survivor | 0 | Uncommon | V32, V30, V27, V38 |

## Design Pattern Audit

Checking each card against the mechanic critique's bad patterns:

| Bad Pattern | Cards Using It | Assessment |
|------------|----------------|------------|
| Pattern 1: Explicit Event-Count Trigger | 0 of 10 | PASS -- no card uses "2+ events this turn" |
| Pattern 2: Single-Trigger-Matters Linear Payoff | 0 of 10 | PASS -- all cards have conditional, threshold, or modal effects |
| Pattern 3: Stapled Signpost (A effect + B effect) | 0 of 10 | PASS -- each card creates emergent behavior from its two abilities |
| Pattern 4: Tribal Lord with No Off-Tribe Application | 0 of 10 | PASS -- Forgeborn Martyr's Warrior pump is paired with a death trigger; Basalt Warden's Spirit Animal typing does not lock its abilities |
| Pattern 5: Overconcentration on One Archetype | N/A | PASS -- exactly 1 card per archetype pair |
| Pattern 6: Lack of Modal/Conditional Abilities | 0 of 10 have NO modality | PASS -- Tideweaver Sentinel has choose-one, Basalt Warden/Depthswatcher/Galerunner have thresholds, Stormtide Oracle has a retrieval decision, others have conditional triggers |
| Pattern 7: Temporary Spark as Default Safety Valve | 1 of 10 (Basalt Warden) | PASS -- only one card uses temporary spark, and it is one mode of a threshold switch, not the whole card |

## Multi-Archetype Demand Audit

| Card | Archetypes That Want It | Count | Assessment |
|------|------------------------|-------|------------|
| Tideweaver Sentinel | Mirage, Basalt, Depths | 3 | PASS |
| Abyssal Reclaimer | Undertow, Eclipse, Mirage | 3 | PASS |
| Basalt Warden | Basalt, Crucible, Mirage | 3 | PASS |
| Forgeborn Martyr | Crucible, Bedrock | 2 | PASS |
| Cinder Ritualist | Cinder, Undertow, Eclipse | 3 | PASS |
| Stormtide Oracle | Tempest, Depths, Undertow | 3 | PASS |
| Depthswatcher | Depths, Tempest, Basalt | 2.5 | PASS |
| Galerunner | Gale, Cinder, Eclipse | 3 | PASS |
| Eclipse Weaver | Eclipse, Tempest, Bedrock | 3 | PASS |
| Bedrock Anchor | Bedrock, Undertow, Crucible | 2.5 | PASS |
| **Average** | | **2.8** | **PASS** (target: 2+) |

## Signpost Coverage After These Additions

| Archetype | Type | Previous Duals | +This Batch | New Total | Target | Status |
|-----------|------|---------------|-------------|-----------|--------|--------|
| Mirage (Tide+Zephyr) | Allied | 3 | +1 | 4 | 4-6 | MEETS MINIMUM |
| Undertow (Tide+Ruin) | Allied | 4 | +1 | 5 | 4-6 | PASS |
| Basalt (Zephyr+Stone) | Allied | 3 | +1 | 4 | 4-6 | MEETS MINIMUM |
| Crucible (Stone+Ember) | Allied | 3 | +1 | 4 | 4-6 | MEETS MINIMUM |
| Cinder (Ember+Ruin) | Allied | 3 | +1 | 4 | 4-6 | MEETS MINIMUM |
| Tempest (Tide+Ember) | Enemy | 4 | +1 | 5 | 3-4 | EXCEEDS (acceptable) |
| Depths (Tide+Stone) | Enemy | 1 | +1 | 2 | 3-4 | STILL BELOW -- needs 1-2 more from other agents |
| Gale (Zephyr+Ember) | Enemy | 1 | +1 | 2 | 3-4 | STILL BELOW -- needs 1-2 more from other agents |
| Eclipse (Zephyr+Ruin) | Enemy | 3 | +1 | 4 | 3-4 | MEETS TARGET |
| Bedrock (Stone+Ruin) | Enemy | 3 | +1 | 4 | 3-4 | MEETS TARGET |

**Note:** Depths and Gale remain below enemy-pair minimums even after this batch. This is by design -- Agent A contributes 1 card per pair, and the QA report's recommended fix (promoting existing cards like Cloaked Sentinel and Pallid Arbiter to Tide+Stone duals, and creating additional Zephyr+Ember duals) should be executed by other agents to bring these archetypes to 3-4 duals each.
