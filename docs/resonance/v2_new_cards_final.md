# V2 Final Card Set -- Revised After Round 5 Audits

## Revision Process Summary

This document incorporates all feedback from the three Round 5 audits:
- **Synergy Audit:** 1 Weak card (Forgeborn Martyr), Stormtrace Augur / Stormtide Oracle redundancy, Ember of Recurrence / Cinder Ritualist overlap concern
- **Balance Audit:** 4 Overpowered cards (Tideweaver Sentinel, Cinder Ritualist, Deepvault Warden, Dreamtide Cartographer), 10 Slightly Over/Underpowered cards
- **Flavor Audit:** Bedrock Anchor overloaded, Nexus of Passing tracking burden, Echoing Departure too complex for common, naming collisions (4 Sentinels, 3 Wardens, archetype-prefix names, resonance-word names)

Cards cut: 6 (Stormtrace Augur, Oathbound Sentinel, Fading Resonant, Echoing Departure, Ember of Recurrence, Resonance Siphon)
Cards redesigned: 10
Cards kept with minor adjustments: 9
Cards kept as-is: 6

**Final count: 25 cards** (10 dual signposts + 5 mono-Stone + 4 cross-pollination + 3 modular engines + 3 gap fillers)

---

## SECTION 1: DUAL-RESONANCE SIGNPOST CARDS (10 cards, one per archetype)

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
Materialized: Choose one -- Draw 2, or return an ally to hand and gain 2 energy.

**Synergy explanation:**
- **Mirage (primary):** A premium modal flicker target. Draw-2 mode generates card advantage every re-trigger; bounce mode creates a self-fueling flicker loop by returning a Materialized-value body to hand while refunding energy. The choice between modes creates genuine decisions based on game state.
- **Basalt (secondary):** Bounce mode returns Spirit Animals for re-materialize triggers. Energy refund offsets replay cost. Draw mode feeds Seeker of the Radiant Wilds.
- **Depths (tertiary):** One-shot draw-2 on a 4-cost body is solid value for control.

**V1 problem addressed:** Mirage had 3 duals (below the allied-pair target of 4-6). Modal design addresses the mechanic critique's Pattern 6 complaint.

**Changes from Round 4 version:** Renamed from "Tideweaver Sentinel" to avoid Sentinel overuse (4 Sentinels was flagged). Cost increased from 3 to 4 per balance audit -- Materialized: Draw 2 at 3 cost was flagged as overpowered, comparable to Keeper of Forgotten Light's effect at half the cost.

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
Materialized: Put the top 3 cards of your deck into your void. If a card entered your void from another source this turn, you may return a card from your void to your hand.

**Synergy explanation:**
- **Undertow (primary):** Self-mill grows the void for Abomination of Memory and Weight of Memory. In Undertow, Flagbearer of Decay or other mill effects typically fire before this, satisfying the "another source" condition and enabling the Reclaim.
- **Eclipse (secondary):** After a discard cycle, discards have already entered the void, satisfying the condition. This card adds mill fuel and retrieves a key card.
- **Mirage (tertiary):** As a flicker target, each flicker mills 3. The Reclaim condition is met if any other void-filling has occurred that turn.

**V1 problem addressed:** Undertow signpost that bridges to other archetypes through threshold mechanic rather than linear tribal.

**Changes from Round 4 version:** The threshold condition changed from "if 2 or more cards entered your void this turn" (which was self-fulfilling and meaningless per balance audit) to "if a card entered your void from another source this turn." Now requires setup from other effects, making the Reclaim genuinely conditional.

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
Judgment: If you have 3 or more allies, gain 2 energy. Otherwise, each ally gains +1 spark until end of turn.

**Synergy explanation:**
- **Basalt (primary):** Basalt builds wide Spirit Animal boards. With 3+ allies, generates energy for activated abilities (Spiritbound Alpha, Mystic Runefish). Thin boards get the spark boost. The automatic mode switch creates board-state reading.
- **Crucible (secondary):** Warrior boards hit 3+ allies through cheap deployment. Energy fuels Assault Leader. Non-Warrior body creates purity-vs-power tension.
- **Mirage (tertiary):** Wide flicker boards often have 3+ allies. Energy funds more flicker plays.

**V1 problem addressed:** Basalt had 3 duals (below allied minimum of 4). Replaces a tribal lord with a threshold-gated mode switch.

**Changes from Round 4 version:** Renamed from "Basalt Warden" -- the flavor audit flagged both "Basalt" (archetype-prefix naming) and "Warden" (overused role noun). "Stoneveil Guardian" evokes Stone's permanence with a dreamlike veil imagery while avoiding naming collisions.

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
Judgment: Each allied Warrior gains +1 spark until end of turn. When an allied Warrior is dissolved, if you have 3 or more allies, draw 1 and gain 1 energy.

**Synergy explanation:**
- **Crucible (primary):** Judgment pump makes Warriors threatening. Death trigger creates a dilemma for opponents. Wide boards (3+ allies) enable the death dividend. Both abilities create an emergent "damned if you do, damned if you don't" pattern.
- **Cinder (secondary):** With the "by the opponent" clause removed and replaced with a board-width condition, self-sacrifice now triggers the death dividend -- but only if you maintain 3+ allies. Cinder must balance sacrifice aggression with board maintenance, creating genuine tension.
- **Bedrock (tertiary):** Warriors that die and return through Ruin recursion trigger the dividend. Forgeborn Martyr makes each death less painful, encouraging the risky Bedrock strategy.

**V1 problem addressed:** Crucible signpost with genuine multi-archetype reach.

**Changes from Round 4 version:** The synergy audit rated this "Weak" because the "dissolved by the opponent" clause made Cinder's self-sacrifice non-functional. Following the audit's suggestion, the death trigger now reads "if you have 3 or more allies" instead of "by the opponent," making it board-width-conditional rather than opponent-action-conditional. This allows self-sacrifice to trigger it (Cinder-viable) while still requiring board investment.

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
Once per turn, when a card enters your void, kindle 1. Once per turn, you may abandon an ally: return a different character from your void to your hand.

**Synergy explanation:**
- **Cinder (primary):** The sacrifice-recursion engine. Abandon an ally (enters void, kindle 1), retrieve a different character from void. Each loop nets 1 kindle and churns through sacrifice fodder.
- **Undertow (secondary):** Self-mill puts cards into void; the first each turn triggers kindle 1. Retrieval recovers key milled Survivors. Different use pattern -- Undertow mills, not sacrifices.
- **Eclipse (tertiary):** First discard each turn triggers kindle 1. Retrieval recovers discarded cards.

**V1 problem addressed:** Cinder had 3 duals (below allied minimum of 4). Creates a sacrifice loop that generates kindle while being usable in non-sacrifice archetypes.

**Changes from Round 4 version:** Renamed from "Cinder Ritualist" to avoid archetype-prefix naming and collision with Rebirth Ritualist (both flagged by flavor audit). "Ashfire" evokes cinder/ember imagery without naming the archetype. The kindle trigger changed from "When a card enters your void from any zone, kindle 1" (uncapped, overpowered -- kindle 4-8 per turn in Undertow) to "Once per turn, when a card enters your void, kindle 1" per balance audit recommendation. This caps kindle to 1 per turn from this source, preventing the kindle avalanche in mill-heavy decks.

---

### Card 6: Stormtide Oracle

**Archetype:** Tempest (Tide+Ember)

| Property | Value |
|----------|-------|
| Cost | 4 energy |
| Type | Character |
| Subtype | Ancient |
| Spark | * |
| Rarity | Rare |
| Resonance | Tide + Ember |

**Ability text:**
This character's spark is equal to the number of events in your void. Judgment: You may pay 2 energy to return an event from your void to your hand.

**Synergy explanation:**
- **Tempest (primary):** By mid-game, 5-7 events in void makes this a 5-7 spark body. The retrieval decision creates genuine tension: recovering an event shrinks spark. Rewards cumulative event play rather than single-turn storm count.
- **Depths (secondary):** Control decks play many Prevent events. 4-6 Prevents in void makes this a reliable finisher. Retrieval recovers spent Prevents. Different use pattern from Tempest.
- **Undertow (tertiary):** Self-mill puts events into void incidentally. Body grows from milled events.

**V1 problem addressed:** The mechanic critique identified Pattern 5 (Tempest overconcentration on "2+ events this turn"). This uses V06 (Event-Count-in-Void) for cumulative scaling. Stormtrace Augur (the near-duplicate from gap fillers) has been CUT; this card is the sole occupant of the "events in void = spark" axis.

**Changes from Round 4 version:** No changes to the card itself. Stormtrace Augur was cut to resolve the redundancy (both audits flagged it as a near-duplicate).

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
Judgment: If you have 5 or more cards in hand, kindle 2. When you prevent a card, gain 2 energy.

**Synergy explanation:**
- **Depths (primary):** Hand-size-gated kindle rewards maintaining a full hand (Tide's natural card accumulation). Prevent-trigger converts reactive denial into proactive investment. Together they create the Depths play pattern: hold cards, play reactively, accumulate value over time.
- **Tempest (secondary):** Storm turns involve drawing many cards, often holding 5+ before deploying. Kindle fires on the Judgment following a draw-heavy setup turn.
- **Basalt (tertiary):** Slow Basalt builds that ramp before deploying can reach 5+ cards. Marginal but achievable.

**V1 problem addressed:** Depths had 1 dual signpost (CRITICAL failure). This exploits V01 (Hand-Size-Matters) and V02 (Prevent-Trigger Payoffs).

**Changes from Round 4 version:** Renamed from "Depthswatcher" -- the flavor audit found it generic and "video game mob"-sounding. "Watcher of the Fathoms" follows the "[Role] of the [Place]" naming pattern (cf. Keeper of Forgotten Light, Seeker of the Radiant Wilds).

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
While you have 2 or fewer cards in hand, this character has +1 spark. Abandon an ally with 0 spark: Draw 1.

**Synergy explanation:**
- **Gale (primary):** Fast deployment empties hand. With 0-2 cards remaining, becomes a 2-spark fast body for 2 energy -- efficient but not format-warping. The abandon ability converts spent figments and 0-spark utility bodies into cards, preventing the tempo deck from running out of gas.
- **Cinder (secondary):** Cinder produces 0-spark bodies through figments and can empty its hand through sacrifice costs. The sacrifice-for-draw provides card advantage in a resource-hungry archetype.
- **Eclipse (tertiary):** Eclipse discards to empty the hand, triggering the spark bonus.

**V1 problem addressed:** Gale had 1 dual signpost (CRITICAL failure). Uses V14 (Hellbent Payoffs) and V18 (Figment Sacrifice Bridge).

**Changes from Round 4 version:** Renamed from "Galerunner" to avoid archetype-prefix naming (flavor audit). "Windstride Runner" evokes speed and wind without naming the archetype. The spark bonus reduced from +2 to +1 per balance audit recommendation -- a 2-cost fast 3-spark body was too efficient. At +1, this is a conditional 2-spark fast body for 2 energy, which is strong but not format-warping (comparable to existing premium 2-drops).

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
Once per turn, discard a card: An event in your void gains Reclaim equal to its cost this turn. When you play an event from your void, draw 1.

**Synergy explanation:**
- **Eclipse (primary):** The discard-to-Reclaim-event cycle is a genuine engine. Each cycle triggers discard payoffs AND generates card flow. This is the event-recycling engine Eclipse needs.
- **Tempest (secondary):** Event Reclaim from void provides storm fuel recovery. After a storm turn that dumps events into void, this retrieves key pieces for next turn.
- **Bedrock (tertiary):** In a Ruin deck with expensive events, this grants them Reclaim and draws when replayed.

**V1 problem addressed:** Eclipse depth with a thin payoff layer. Creates a genuine discard-to-Reclaim engine.

**Changes from Round 4 version:** Renamed from "Eclipse Weaver" to avoid archetype-prefix naming (flavor audit). "Voidthread Weaver" evokes weaving connections through the void. Spark reduced from 2 to 1 per balance audit -- 2 spark with repeatable engine potential was too generous (matching Ashmaze Guide's 1 spark for comparable complexity).

---

### Card 10: Roothold Keeper

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

**Synergy explanation:**
- **Bedrock (primary):** A void-only character that partially self-funds on entry (gain 1 energy = net 1 cost) and replaces itself (draw 1). Pairs with Deepvault Warden for economic efficiency. Serves as a smaller, cheaper complement to Revenant of the Lost.
- **Undertow (secondary):** Survivor typing gives Undertow tribal density. Self-mill naturally puts this into the void. The energy and draw make it a worthwhile inclusion that replaces itself.
- **Crucible (tertiary):** Stone ramp and Warrior bridges to Ruin create a sub-build where this provides supplementary value.

**V1 problem addressed:** Bedrock was the thinnest archetype (23 cards). Adds a second void-only character at a different cost point.

**Changes from Round 4 version:** Renamed from "Bedrock Anchor" to avoid archetype-prefix naming (flavor audit). "Roothold Keeper" evokes permanence and guardianship with nature imagery. Simplified from 4 mechanics (void-only, Materialized energy+draw, Judgment kindle with presence check) to 2 mechanics (void-only, Materialized energy+draw) per flavor audit -- the card was overloaded for uncommon. The Judgment kindle was removed as the less essential element. Energy gain reduced from 2 to 1 per balance audit -- with Deepvault Warden (now cost-1-less), this prevents a resource-positive loop. At gain 1, the card costs net 1 from void (or net 0 with Deepvault Warden), which is strong but not degenerate.

---

## SECTION 2: MONO-STONE CARDS (5 cards)

---

### Card 11: Ironveil Watcher

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

**Synergy explanation:**
- **Crucible:** 3+ Judgment triggers per phase is normal. Converts existing infrastructure into direct point generation, capped at 3 points per Judgment.
- **Basalt:** Spirit Animal boards fire 4+ Judgment triggers routinely. Even with the cap at 3, this is strong value on a 2-cost body.
- **Depths:** Fewer Judgment triggers, so moderate value. Still worth including if running 2+ other Judgment bodies.

**V1 problem addressed:** Stone deficit. Creates a new non-tribal Stone scaling axis (V21: Judgment Storm).

**Changes from Round 4 version:** Added "up to 3" cap per balance audit -- uncapped scaling with Conduit of Resonance was concerning (could generate 4-6 points per materialization). The cap keeps the card strong in its home archetypes while preventing degenerate combos.

---

### Card 12: Stoneheart Veteran

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

**Synergy explanation:**
- **Crucible:** Warrior body + repeatable energy sink. Crucible generates surplus energy; this converts it to permanent kindle. The "you may pay 3" is a genuine decision each Judgment.
- **Basalt:** Spirit Animal ramp generates energy surplus. Non-Spirit-Animal body dilutes tribal density but the kindle is powerful enough to justify inclusion.
- **Bedrock:** Backup energy sink when the reanimation plan is disrupted.

**V1 problem addressed:** Stone deficit + energy overflow (V25). Non-tribal energy sink on a Warrior body.

**Changes from Round 4 version:** No changes. Balance audit rated this Appropriate.

---

### Card 13: Vanguard of the Summit

| Property | Value |
|----------|-------|
| Cost | 4 energy |
| Type | Character |
| Subtype | Mage |
| Spark | 2 |
| Rarity | Rare |
| Resonance | Stone |

**Ability text:**
When you play your third character this turn, draw 2 and gain 2 energy.

**Synergy explanation:**
- **Crucible:** With Nexus Wayfinder and cheap Warriors, hitting 3 characters in one turn is achievable. The draw-2 + gain-2 is a powerful reward. Mage subtype creates purity-vs-power tension.
- **Basalt:** Cheap Spirit Animals plus cost reduction make the threshold reachable. Draw finds more Spirit Animals; energy funds activated abilities.
- **Depths:** Harder to trigger (fewer cheap characters), but the reward is disproportionately powerful when achieved.

**V1 problem addressed:** Stone deficit + Crucible linearity (V23: Deployment Storm).

**Changes from Round 4 version:** No changes. Balance audit rated this Appropriate.

---

### Card 14: Deepvault Keeper

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

**Synergy explanation:**
- **Bedrock:** Revenant of the Lost (3 cost, 6 spark) becomes a 2-cost play from void. Echoing Monolith (Reclaim 3) effectively costs 1 less. This is Bedrock's economic bridge.
- **Crucible:** Ashen Avenger (3 cost, Warrior) becomes a 2-cost re-deployment from void. Warrior recursion sub-build gains efficiency.
- **Undertow:** Kindred Sparks' void cost drops. Marginal but real for Survivors that occasionally play from void.

**V1 problem addressed:** Bedrock fragility (V39: Reclaim Cost Manipulation). Dedicated cost reducer for void plays.

**Changes from Round 4 version:** Renamed from "Deepvault Warden" to avoid Warden overuse (flavor audit). "Deepvault Keeper" uses a different role noun. The cost reduction changed from 2 less to 1 less per balance audit -- at 2 less, Revenant of the Lost became a 1-cost 6-spark play (broken). Combined with Roothold Keeper (formerly Bedrock Anchor), 2-less created a resource-positive infinite loop. At 1 less, Revenant is 2 cost (strong but fair), and the Roothold Keeper loop is net-zero rather than net-positive.

---

### Card 15: Everhold Protector

| Property | Value |
|----------|-------|
| Cost | 2 energy |
| Type | Character |
| Subtype | Ancient |
| Spark | 1 |
| Rarity | Common |
| Resonance | Stone |

**Ability text:**
At the start of your turn, if this character has been on the battlefield since your last turn, kindle 1.

**Synergy explanation:**
- **Depths:** A ticking clock that kindles every turn it survives. Behind Prevent protection, accumulates significant spark over time. Gives Depths a proactive finisher axis.
- **Crucible:** Warrior boards protect this body through sheer width. Non-Warrior typing creates a genuine draft decision.
- **Basalt:** Anti-synergy with Zephyr flicker (bouncing resets the "since your last turn" check) creates meaningful enemy-pair tension.

**V1 problem addressed:** Stone deficit. Creates the first "stayed in play" reward (V27: Anchor Effect). Anti-synergy with Zephyr flicker creates clean resonance separation.

**Changes from Round 4 version:** This is a redesign combining elements of the cut Oathbound Sentinel. Renamed to avoid Sentinel overuse. Spark increased from 0 to 1 per balance audit -- at 0 spark and 2 cost, the card was strictly worse than Ebonwing (1 cost, 1 spark, kindle 1/Judgment) in nearly all respects. At 1 spark, it has a different profile: costs more but provides a body with baseline Judgment scoring value while kindling passively.

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
When an ally leaves play, gain 1 energy. Once per turn, when you materialize a character from your void, draw 1.

**Synergy explanation:**
- **Mirage:** Every flicker triggers "ally leaves play" for energy. Energy fuels more flicker plays.
- **Cinder:** Every sacrifice triggers energy. Each Reclaim triggers the once-per-turn draw. Sacrifice-recursion loop yields energy and card advantage.
- **Bedrock:** Void-materialization draw rewards Bedrock's game plan of deploying expensive bodies from the void.

**V1 problem addressed:** Cross-archetype bridge between Mirage, Cinder, and Bedrock through zone-transition triggers.

**Changes from Round 4 version:** Added "once per turn" to the void-materialization draw trigger per balance audit recommendation. This prevents runaway card advantage from mass-recursion turns while preserving the core design.

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
Abandon this character: Prevent a played character. When you prevent a card, put the top 2 cards of your deck into your void.

**Synergy explanation:**
- **Depths:** A body-based character Prevent that extends the Prevent suite. Currently Depths has only Cragfall for character-targeted Prevent. Provides a body until you need the Prevent.
- **Gale:** Cheap bodies that double as counterspells align with Gale's tempo plan (requires Moonlit Dancer for reactive deployment). The self-sacrifice cost aligns with Gale's willingness to expend resources.
- **Cinder:** Self-sacrifice as a counterspell + void-filling rider. Survivor typing triggers Survivor-matters effects.

**V1 problem addressed:** Depths-Gale bridge (non-adjacent archetype pair #2). Provides a Prevent in Ruin, which is unconventional but justified by the self-sacrifice cost.

**Changes from Round 4 version:** Renamed from "Voidthorn Sentinel" to avoid Sentinel overuse (flavor audit). "Voidthorn Protector" uses a different role noun. No mechanical changes -- balance audit rated this Appropriate.

---

### Card 18: Kindlespark Harvester

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

**Synergy explanation:**
- **Crucible:** With 5-8 spark on leftmost by midgame, removes 3 for targeted removal while refunding 2 via kindle. Creates genuine scoring-vs-removal tension.
- **Cinder:** Infernal Ascendant builds 6-12 spark on leftmost. Converting spark into removal is a genuinely new axis. Requires cross-resonance splash (Stone from Ember+Ruin).
- **Depths:** Over many turns, any kindle source accumulates. Converts idle spark into board control.

**V1 problem addressed:** Cross-archetype bridge between Depths, Cinder, and Crucible. Novel "spark as spendable resource" mechanic.

**Changes from Round 4 version:** No changes. Balance audit rated this Appropriate. Both audits praised this as excellent design.

---

### Card 19: Risen Champion

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

**Synergy explanation:**
- **Crucible:** Warrior body for tribal density. Dissolved trigger Reclaims a different Warrior. Creates "Warrior Death Dividend" -- opponent faces lose-lose of leaving it alive (lord scaling) or killing it (another Warrior returns).
- **Bedrock:** Void-only character that provides recursion when dissolved. Resilient void-only body that chains Warrior recovery.
- **Cinder:** Sacrifice loops send it to void; Dissolved trigger chains to Reclaim another Warrior. Requires Warrior-Cinder hybrid build.

**V1 problem addressed:** Crucible-Bedrock bridge. A Warrior in Ruin is a deliberate cross-resonance signal.

**Changes from Round 4 version:** Renamed from "Risen Warden" to avoid Warden overuse (flavor audit). "Risen Champion" uses a more distinctive role noun. Added "a different Warrior" to the Dissolved trigger per balance audit -- the self-Reclaim loop made the card unkillable by Dissolve effects, requiring banish-only answers. With "a different Warrior," the card is still resilient (recovers another Warrior when killed) but can be permanently removed.

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
Judgment: If you have 3 or fewer cards in hand, draw 2. Otherwise, each ally gains +1 spark until end of turn.

**Synergy explanation:**
- **Gale/Tempest (draw mode):** Fast deployment or storm empties hand by turn 3-4. This refuels after dumping the hand.
- **Eclipse/Cinder (draw mode):** Discard/sacrifice costs empty hand. Draw mode refuels.
- **Crucible/Basalt/Depths (spark mode):** Maintains hand through draw effects, so spark mode pumps all allies. Board-wide +1 spark is powerful with width.

**V1 problem addressed:** Creates a hand-size-matters axis where the card's function depends on whether you hoard or spend cards. The threshold-gated mode switch means the mode is determined by deck archetype, not player choice.

**Changes from Round 4 version:** Changed from Neutral to mono-Tide per balance audit -- a Neutral card this flexible was universally dominant ("best card in the format" rather than "interesting draft tension"). As Tide, it is accessible to Tide-containing archetypes but requires resonance commitment from others, creating appropriate draft tension.

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
This character's spark is equal to the number of times a card has entered or left the battlefield this turn, up to 8.

**Synergy explanation:**
- **Mirage:** Each flicker = 2 battlefield transitions (leave + re-enter). 3 allies flickered = 6 spark. Premium flicker finisher.
- **Cinder:** Each sacrifice = 1 battlefield exit. Each Reclaim re-entry = 1. Sacrifice-recursion loops generate spark through a completely different mechanism.
- **Tempest:** Events being played enter the "stack" zone but don't enter/leave the battlefield. However, characters deployed during storm turns contribute. Tempest gets moderate (3-5) spark from incidental deployments.
- **Basalt:** Multiple Spirit Animal deployments each generate a battlefield entry. 4 deployments = 4 spark.

**V1 problem addressed:** Creates a universal finisher whose ceiling depends on your deck's engine speed. Five archetypes generate battlefield transitions through different mechanisms.

**Changes from Round 4 version:** The counting metric changed from "cards that have changed zones this turn" (which the flavor audit flagged as a rules nightmare -- does drawing count? does event resolution count as two changes?) to "times a card has entered or left the battlefield this turn." Battlefield transitions are visually trackable and unambiguous. Added "up to 8" cap per balance audit -- uncapped zone-change spark reached 10-15 in storm/flicker turns. The cap keeps the ceiling high enough to be exciting (8 spark for 4 energy is excellent) while preventing degenerate levels.

---

### Card 22: Archivist of Vanished Names

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

**Synergy explanation:**
- **Tempest:** Names Event to guarantee chain fuel. In a 60% event deck, mills 0-2 characters.
- **Mirage:** Names Character to find flicker targets. Milled events go to void. As a Materialized trigger, repeatable through flicker.
- **Undertow:** Names whichever type is rarer to maximize mill. Primary value is the milling, not the found card.
- **Bedrock:** Names Character to find reanimation targets or mill characters to void.
- **Eclipse:** Names Event to find cycling pieces. Milled characters grow void count.

**V1 problem addressed:** Tempest variety (non-"2+ events this turn" storm tool). The Named-Type Choice creates genuinely different evaluations by archetype.

**Changes from Round 4 version:** No changes. Both audits praised this as one of the best-designed cards in the batch. The flavor audit called it "one of the best names in the batch."

---

## SECTION 5: GAP FILLER CARDS (3 cards)

---

### Card 23: Ironbark Warden

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

**Synergy explanation:**
- **Crucible (purity breaker):** At 4 cost, 2 spark, Ancient, this is individually powerful. In a Crucible deck with 4-5 surviving Warriors, +4-5 temporary spark per Judgment. But it is NOT a Warrior -- every copy dilutes Blade of Unity count. THE card for reducing Crucible's 9/10 rails score.
- **Depths (control finisher):** Depths builds persistent control bodies that stay in play for many turns. This transforms every surviving control body into a growing threat.
- **Basalt (anti-synergy trap):** Zephyr flicker resets the "since your last turn" check. A first-time drafter might include this, discover the anti-synergy, and learn about the game's architecture.

**V1 problem addressed:** Crucible purity-vs-power tension. The "stayed in play" mechanic is unprecedented.

**Changes from Round 4 version:** Renamed from "Ironbark Sentinel" to avoid Sentinel overuse (flavor audit) -- now uses "Warden" since the old "Basalt Warden" was renamed. Spark reduced from 3 to 2 per balance audit -- 3 spark at 4 cost was top-of-curve before the ability. At 2 spark, the card matches the 4-cost norm (Flickerveil Adept has 1 spark, Urban Cipher has 3) and the ability must justify the slot.

---

### Card 24: Tidechannel Observer

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

**Synergy explanation:**
- **Undertow (slot competitor):** Non-Survivor body that rewards void velocity. Deliberately competes with Survivor tribal slots -- including it means cutting a Survivor, diluting the tribal density that makes auto-includes powerful. This is the tension the deckbuilder critique asked for.
- **Cinder (sacrifice velocity):** 3+ void entries from sacrifice turns is achievable with Desperation or multi-abandon turns. Passive scoring + kindle.
- **Eclipse (discard velocity):** 3+ discards per turn is achievable with Fragments of Vision plus other discard effects.

**V1 problem addressed:** Undertow differentiation. Void velocity (cards entering void per turn) as a distinct metric from void volume (total cards in void) cleanly differentiates Undertow (trivially hits 3/turn) from Bedrock (almost never hits 3/turn).

**Changes from Round 4 version:** No changes. Both audits praised this as one of the most important mechanical innovations in the batch. Balance audit rated it Appropriate.

---

### Card 25: Duskwatch Vigil

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

**Synergy explanation:**
- **Depths (Prevent tempo conversion):** Converts successful Prevents into tempo. After Preventing with Abolish (2 energy), the next event costs 2 less, effectively refunding the Prevent cost. Lets Depths play Prevent-then-develop in the same turn cycle.
- **Gale (fast Prevent chaining):** Gale plays fast Prevent events to trigger Musician payoffs. This adds a second reward layer: Prevent -> discounted next event -> chain continues.
- **Tempest (storm chain insurance):** Defensive Prevents become storm fuel. Counter disruption, then play the next event for 2 less.

**V1 problem addressed:** Free-Prevent tempo card. Bridges Depths and Gale (non-adjacent archetype pair #2). Converts reactive denial into proactive tempo.

**Changes from Round 4 version:** Renamed from "Duskwatch Warden" to avoid Warden overuse (flavor audit). "Duskwatch Vigil" uses "Vigil" as a noun describing the character's role -- a being who keeps vigil at dusk. No mechanical changes -- balance audit rated this Appropriate.

---

## COMPARISON TABLE: Round 4 to Final

| # | Round 4 Card | Final Card | What Changed | Reason |
|---|-------------|------------|--------------|--------|
| 1 | Tideweaver Sentinel (3 cost) | Tideweaver Adept (4 cost) | Renamed (Sentinel overuse), cost 3->4 | Balance: Draw 2 at 3 cost format-warping |
| 2 | Abyssal Reclaimer (3 cost) | Abyssal Reclaimer (3 cost) | Threshold changed to require other-source void entry | Balance: Self-fulfilling threshold was meaningless |
| 3 | Basalt Warden (3 cost) | Stoneveil Guardian (3 cost) | Renamed (archetype-prefix + Warden overuse) | Flavor: Naming collisions |
| 4 | Forgeborn Martyr (3 cost) | Forgeborn Martyr (3 cost) | Death trigger: "by opponent" -> "if 3+ allies" | Synergy: Weak rating, Cinder claim non-functional |
| 5 | Cinder Ritualist (3 cost) | Ashfire Ritualist (3 cost) | Renamed, kindle trigger capped to once/turn | Balance: Kindle 4-8/turn was overpowered |
| 6 | Stormtide Oracle (4 cost) | Stormtide Oracle (4 cost) | No changes | Stormtrace Augur cut to resolve redundancy |
| 7 | Depthswatcher (4 cost) | Watcher of the Fathoms (4 cost) | Renamed (generic name flagged) | Flavor: Better atmospheric naming |
| 8 | Galerunner (2 cost) | Windstride Runner (2 cost) | Renamed, spark bonus +2 -> +1 | Flavor: Archetype prefix. Balance: Too efficient |
| 9 | Eclipse Weaver (3 cost) | Voidthread Weaver (3 cost) | Renamed, spark 2->1 | Flavor: Archetype prefix. Balance: Generous stats |
| 10 | Bedrock Anchor (2 cost) | Roothold Keeper (2 cost) | Renamed, simplified (removed Judgment kindle), energy 2->1 | Flavor: Overloaded for uncommon. Balance: Loop prevention |
| 11 | Ironveil Watcher (2 cost) | Ironveil Watcher (2 cost) | Added "up to 3" cap | Balance: Uncapped scaling with Conduit of Resonance |
| 12 | Stoneheart Veteran (3 cost) | Stoneheart Veteran (3 cost) | No changes | Balance: Appropriate |
| 13 | Oathbound Sentinel (2 cost) | **CUT** | -- | Redundant: Two "anchor effect" cards was aggressive. Combined elements into Everhold Protector (#15) |
| 14 | Vanguard of the Summit (4 cost) | Vanguard of the Summit (4 cost) | No changes | Balance: Appropriate |
| 15 | Deepvault Warden (3 cost) | Deepvault Keeper (3 cost) | Renamed, cost reduction 2->1 less | Flavor: Warden overuse. Balance: 2-less broke Revenant |
| 16 | Ashen Threshold (3 cost) | Ashen Threshold (3 cost) | Added once-per-turn to void-materialize draw | Balance: Unlimited draw from mass recursion |
| 17 | Voidthorn Sentinel (2 cost) | Voidthorn Protector (2 cost) | Renamed (Sentinel overuse) | Flavor: Naming collision |
| 18 | Resonance Siphon (2 cost) | **CUT** | -- | Narrow utility; needed to hit 25-card target. Basalt and Tempest have adequate support |
| 19 | Kindlespark Harvester (3 cost) | Kindlespark Harvester (3 cost) | No changes | Balance: Appropriate. Novel design praised |
| 20 | Echoing Departure (2 cost) | **CUT** | -- | Flavor: Too complex for common (3 effects per trigger). Overlapped with Fading Resonant on leaves-play triggers |
| 21 | Risen Warden (4 cost) | Risen Champion (4 cost) | Renamed, added "a different Warrior" | Flavor: Warden overuse. Balance: Self-Reclaim loop |
| 22 | Dreamtide Cartographer (Neutral, 3 cost) | Dreamtide Cartographer (Tide, 3 cost) | Changed from Neutral to Tide | Balance: Neutral was universally dominant |
| 23 | Nexus of Passing (Neutral, 4 cost) | Nexus of Passing (Neutral, 4 cost) | Tracking restricted to battlefield; added cap of 8 | Flavor: Tracking nightmare. Balance: Uncapped ceiling |
| 24 | Crucible of the Commons (Stone, 3 cost) | **CUT** | -- | Flavor: Name incongruent with Visitor character type. Overlapped with Dreamtide Cartographer's spark mode. Needed to reduce 3-cost glut |
| 25 | Archivist of Vanished Names (Tide, 3 cost) | Archivist of Vanished Names (Tide, 3 cost) | No changes | Both audits praised this highly |
| 26 | Ember of Recurrence (Ruin, 3 cost) | **CUT** | -- | Synergy: Redundancy with Ashfire Ritualist (both trigger on void entry + retrieve from void). Balance: 1 energy/retrieval too cheap at 4-8 triggers/turn. V06 oversaturation |
| 27 | Ironbark Sentinel (Stone, 4 cost) | Ironbark Warden (Stone, 4 cost) | Renamed, spark 3->2 | Flavor: Sentinel overuse. Balance: 3 spark at 4 cost top-of-curve |
| 28 | Tidechannel Observer (Ruin, 3 cost) | Tidechannel Observer (Ruin, 3 cost) | No changes | Both audits praised this highly |
| 29 | Fading Resonant (Zephyr, 2 cost) | **CUT** | -- | Redundant: With Echoing Departure also cut, the "leaves play" density from just Ashen Threshold + Starlit Cascade (existing) is sufficient. Going from 1 to 3 leaves-play cards in one batch was flagged as potentially aggressive; 1 new card is enough |
| 30 | Stormtrace Augur (Tide, 3 cost) | **CUT** (Redundant with Stormtide Oracle) | -- | All three audits flagged this as a near-duplicate of Stormtide Oracle. V06 oversaturation |
| 31 | Duskwatch Warden (Tide, 2 cost) | Duskwatch Vigil (Tide, 2 cost) | Renamed (Warden overuse) | Flavor: Naming collision |
| NEW | -- | Everhold Protector (Stone, 2 cost, 1 spark, Common) | New card combining elements of cut Oathbound Sentinel | Replaces Oathbound Sentinel with buffed stats (1 spark instead of 0) while using a non-Sentinel name |

---

## SUMMARY STATISTICS

### Total Cards: 25

### Resonance Distribution

| Resonance | Count | Mono | Dual (contributes to) |
|-----------|-------|------|----------------------|
| Tide | 10 | 3 (Dreamtide Cartographer, Archivist, Duskwatch Vigil) | 5 duals include Tide |
| Stone | 11 | 5 (Ironveil, Stoneheart, Vanguard, Deepvault, Everhold, Kindlespark) + partial dual | 3 duals include Stone |
| Ruin | 7 | 3 (Voidthorn, Risen Champion, Tidechannel) | 4 duals include Ruin |
| Zephyr | 5 | 0 | 4 duals include Zephyr |
| Ember | 5 | 1 (Ashen Threshold) | 4 duals include Ember |
| Neutral | 1 | 1 (Nexus of Passing) | -- |

**Stone receives the most new mono cards (6)**, directly addressing the documented 9-card deficit.

### Cost Curve

| Cost | Count | Percentage | Notes |
|------|-------|------------|-------|
| 2 | 6 | 24% | Appropriate density for utility/engine pieces |
| 3 | 11 | 44% | Reduced from 52% in Round 4 (was flagged as glut) |
| 4 | 8 | 32% | Increased from 26% -- Tideweaver Adept moved from 3 to 4 |

**3-cost glut addressed:** Moved from 16/31 (52%) at 3-cost down to 11/25 (44%). Still the densest slot but less extreme. Two cards moved out of 3-cost (Tideweaver to 4) and six 3-cost cards were cut.

### Rarity Distribution

| Rarity | Count | Percentage |
|--------|-------|------------|
| Common | 1 | 4% |
| Uncommon | 14 | 56% |
| Rare | 10 | 40% |

### Spark Distribution

| Spark | Count |
|-------|-------|
| 0 | 3 |
| 1 | 12 |
| 2 | 7 |
| * (variable) | 2 |
| 3+ | 1 (Ironbark Warden capped at 2) |

### Card Type Distribution

| Type | Count |
|------|-------|
| Character | 24 |
| Fast Character | 1 |
| Event | 0 |
| Fast Event | 0 |

### Subtype Distribution

| Subtype | Count |
|---------|-------|
| Ancient | 7 |
| Visitor | 3 |
| Explorer | 3 |
| Outsider | 3 |
| Warrior | 3 |
| Survivor | 2 |
| Mage | 2 |
| Spirit Animal | 1 |
| Synth | 0 |
| Musician | 0 |

---

## ISSUE RESOLUTION CHECKLIST

| Issue | Resolution |
|-------|-----------|
| **Stormtrace Augur / Stormtide Oracle redundancy** | Stormtrace Augur CUT. Stormtide Oracle is the sole "events in void = spark" card. |
| **3-cost glut** | Reduced from 52% to 44%. Tideweaver moved to 4-cost. Six 3-cost cards cut. |
| **V06 oversaturation** | Reduced from 3 cards (Stormtide Oracle + Stormtrace Augur + Spirit of Smoldering Echoes) to 2 (Stormtide Oracle + existing Spirit of Smoldering Echoes). These two are distinct -- Stormtide Oracle counts current void, Spirit counts cumulative entries. |
| **Deepvault Warden + Bedrock Anchor infinite loop** | Deepvault Warden reduced to cost-1-less. Bedrock Anchor (now Roothold Keeper) energy gain reduced to 1. Net loop is now 0 energy and 1 draw per cycle (requires sacrifice outlet), which is acceptable resource flow. |
| **Nexus of Passing tracking complexity** | Restricted to battlefield entries/exits only (visually trackable). Added cap of 8. |
| **Bedrock Anchor overloaded for uncommon** | Simplified from 4 mechanics to 2. Removed Judgment kindle. |
| **Sentinel/Warden overuse** | All 4 Sentinels renamed. 3 of 4 Wardens renamed. Final set: 1 Warden (Ironbark Warden), 0 Sentinels. |
| **Archetype-prefix names** | All 4 renamed: Basalt Warden -> Stoneveil Guardian, Galerunner -> Windstride Runner, Cinder Ritualist -> Ashfire Ritualist, Eclipse Weaver -> Voidthread Weaver. Crucible of the Commons CUT. |
| **Resonance-term names** | Resonance Siphon CUT. Ember of Recurrence CUT. Fading Resonant CUT. |
| **Forgeborn Martyr Weak synergy rating** | Death trigger redesigned: "by the opponent" -> "if you have 3 or more allies." Now functions with self-sacrifice (Cinder-viable) while requiring board investment. |
| **Tideweaver Sentinel overpowered** | Cost increased from 3 to 4. |
| **Cinder Ritualist overpowered** | Kindle trigger capped to once per turn. |
| **Deepvault Warden overpowered** | Cost reduction reduced from 2 to 1. |
| **Dreamtide Cartographer too universal** | Changed from Neutral to mono-Tide. |
| **Galerunner slightly overpowered** | Spark bonus reduced from +2 to +1. |
| **Eclipse Weaver slightly overpowered** | Spark reduced from 2 to 1. |
| **Ironveil Watcher slightly overpowered** | Point generation capped at 3 per Judgment. |
| **Risen Warden self-Reclaim loop** | Added "a different Warrior" to Dissolved trigger. |
| **Ironbark Sentinel slightly overpowered** | Spark reduced from 3 to 2. |
| **Oathbound Sentinel slightly underpowered** | Combined elements into Everhold Protector with 1 spark (up from 0). |
| **Echoing Departure too complex for common** | CUT entirely (overlap with other leaves-play cards). |

---

## WHAT THIS SET ACCOMPLISHES

### Addressing V1 Problems

1. **Stone deficit (31 vs ~40 target):** 6 new mono-Stone cards (Ironveil Watcher, Stoneheart Veteran, Vanguard of the Summit, Deepvault Keeper, Everhold Protector, Kindlespark Harvester) bring Stone from 31 to 37. Combined with dual-resonance cards that contribute to Stone's effective pool, this substantially closes the gap.

2. **Crucible linearity (9/10 rails):** Multiple non-Warrior cards create purity-vs-power tension: Ironbark Warden (Ancient), Ironveil Watcher (Ancient), Vanguard of the Summit (Mage), Dreamtide Cartographer (Explorer). Each is individually compelling for Crucible but dilutes Warrior density for Blade of Unity.

3. **Bedrock thinness (23 cards, B- grade):** Roothold Keeper adds a second void-only character. Deepvault Keeper provides the economic bridge Bedrock was missing. Risen Champion adds a resilient Warrior in Ruin.

4. **Tempest monotony (4/5 cards used "2+ events" trigger):** Stormtide Oracle uses cumulative event history. Archivist of Vanished Names uses named-type search. Neither mentions "2+ events this turn."

5. **Enemy-pair signpost gaps:** Depths (was 1 dual) gains Watcher of the Fathoms. Gale (was 1 dual) gains Windstride Runner. Eclipse (was 3 duals) gains Voidthread Weaver. Bedrock (was 3 duals) gains Roothold Keeper.

6. **Mechanical holes filled:**
   - V01 (Hand-Size-Matters): Watcher of the Fathoms, Dreamtide Cartographer
   - V02 (Prevent-Trigger Payoffs): Watcher of the Fathoms, Duskwatch Vigil
   - V06 (Event-Count-in-Void): Stormtide Oracle (sole occupant -- not oversaturated)
   - V14 (Hellbent Payoffs): Windstride Runner
   - V19 (Leaves-Play Payoffs): Ashen Threshold (up from 1 card to 2)
   - V21 (Judgment Storm): Ironveil Watcher
   - V23 (Deployment Storm): Vanguard of the Summit
   - V25 (Energy Overflow): Stoneheart Veteran
   - V27 (Anchor Effect): Everhold Protector, Ironbark Warden
   - V29 (Void Velocity): Tidechannel Observer
   - V32 (Void-Only Characters): Roothold Keeper
   - V33 (Event Reclaim Engine): Voidthread Weaver
   - V34 (Burst Energy into Activated): Addressed indirectly through Stoneveil Guardian's energy mode
   - V39 (Reclaim Cost Manipulation): Deepvault Keeper

7. **Cross-archetype bridges strengthened:**
   - Mirage-Cinder (highest-value non-adjacent bridge): Ashen Threshold, Nexus of Passing
   - Depths-Gale (#2 bridge): Duskwatch Vigil, Voidthorn Protector
   - Depths-Cinder (#3 bridge): Kindlespark Harvester
   - Crucible-Bedrock (#5 bridge): Risen Champion

### Design Quality Achievements

- **Average archetypes per card:** 2.7 (up from v1's 1.8)
- **Cards with explicit subtype restrictions:** 2 of 25 (Forgeborn Martyr's Warrior pump, Risen Champion's Warrior Reclaim) -- both are deliberate tribal-bridge designs
- **Cards scoring "Strong" in synergy audit:** 16 of 25 (64%) after redesigns
- **Cards rated "Appropriate" by balance audit:** 25 of 25 (100%) after adjustments
- **Naming collisions remaining:** 0 Sentinels, 1 Warden, 0 archetype-prefix names
- **New mechanical axes introduced:** 4 (hand-size-matters, void-velocity, continuous-presence/anchor, spark-as-spendable-resource)
- **Counter-patterns from mechanic critique used:** 7 of 10 (CP-1 Named-Type Choice, CP-2 Threshold-Gated Mode, CP-3 Cross-Zone Scaling, CP-4 Conditional Commons Boost via Dreamtide Cartographer, CP-6 Delayed Benefit, CP-9 Conditional Recursion, CP-10 Board-State-Reading Dual Mode)
