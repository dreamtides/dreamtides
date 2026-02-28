# Agent D -- Modular Engine Designs (v2)

## Design Philosophy

Every card in this file has one job: be the same text that means completely different things in different decks. The ideal engine card does not mention an archetype, a subtype, or a specific resonance action. It describes a game-state condition or an effect that every archetype PRODUCES but through different mechanisms -- so the same trigger fires for different reasons, and the same payoff is cashed in for different purposes.

The v1 mechanic critique identified that 44% of new cards were wanted by only 1 archetype. These five designs target a minimum of 3 archetypes each, with the card's value coming from DIFFERENT abilities of the same text depending on the reader's deck context.

---

## Card 1: Dreamtide Cartographer

**Cost:** 3 energy | **Type:** Character | **Subtype:** Explorer | **Spark:** 1 | **Rarity:** Uncommon | **Resonance:** Neutral

**Ability text:**

> Judgment: If you have 3 or fewer cards in hand, draw 2. Otherwise, each ally gains +1 spark until end of turn.

### Context Map

| Archetype | Mode Used | Why It Wants This Card | How the Same Text Reads Differently |
|-----------|-----------|------------------------|-------------------------------------|
| **Gale** (Zephyr+Ember) | Draw mode (primary) | Fast deployment empties hand by turn 3-4. This refuels after you have vomited your hand onto the board. A 3-cost body that draws 2 per Judgment is elite in a deck that runs out of gas. | "I dumped my hand of fast threats; now I reload." |
| **Eclipse** (Zephyr+Ruin) | Both modes (oscillating) | Eclipse's hand size fluctuates wildly -- full after draw-discard cycles, empty after discarding to payoffs. This card naturally oscillates between modes within a single game, making it a value engine regardless of hand state. The draw mode triggers discard payoffs (Ashmaze Guide gives drawn cards Reclaim if you discard them next turn). | "I cycle through both modes as my hand inflates and deflates." |
| **Crucible** (Stone+Ember) | Spark mode (primary) | Crucible maintains hand through Judgment draw (Invoker of Myths, Seeker for the Way) and holds Warriors for deployment. With 4+ cards in hand, the +1 spark to each ally is a mass Warrior pump -- Aspiring Guardian (0 spark) becomes 1, Wolfbond Chieftain (0 spark) becomes 1, every lord gets +1. On a board of 5 Warriors, that is +5 total spark per Judgment. | "I keep a full hand of Warriors and pump my whole board." |
| **Basalt** (Zephyr+Stone) | Spark mode (primary) | Spirit Animals are notoriously low-spark (twelve of eighteen are 1-spark). The temporary +1 spark turns every utility Spirit Animal into a 2-spark threat at Judgment. With Spiritbound Alpha's activated ability on the same Judgment, the combination is devastating. | "My 1-spark Spirit Animals all become 2-spark scorers." |
| **Cinder** (Ember+Ruin) | Draw mode (primary) | Cinder empties its hand through discard-as-cost (Rebirth Ritualist, Pulse of Sacrifice) and sacrifice outlays. This card refuels the engine after a sacrifice turn. | "I burned my hand on sacrifice costs; now I draw into more fodder." |
| **Depths** (Tide+Stone) | Spark mode (primary) | Depths accumulates cards through draw; hand size stays at 5+. The spark mode pumps Depths' persistent control bodies (Cloaked Sentinel, Twilight Suppressor, Virtuoso of Harmony) for Judgment scoring without requiring any tribal density. | "My control bodies contribute meaningful spark every Judgment." |
| **Tempest** (Tide+Ember) | Draw mode (mid-storm) | During a storm turn, Tempest plays out its hand. If this character survived from a previous turn, the post-storm Judgment draws 2 to begin rebuilding for the next explosion. | "Post-storm refuel on the next Judgment." |

### Synergy Vectors Exploited

- **V01 (Hand-Size-Matters):** The threshold creates a hand-size axis where the card's function depends on whether you hoard or spend cards.
- **V14 (Hellbent):** The draw mode rewards the empty-hand state that Gale and Cinder naturally reach.
- **V03 (Board-Width):** The spark mode rewards wide boards regardless of tribal identity.

### Counter-Pattern Used

**Counter-Pattern 2 (Threshold-Gated Mode Switch)** from the mechanic critique. The threshold of 3 cards creates a dynamic binary: every turn, the player's hand size determines which mode fires. Unlike a choose-one modal, the mode is determined by game state, meaning the player must PLAN their hand management across turns to get the mode they want. This creates decisions that "when you discard, +1 spark" never does.

### Why This Is Better Than a v1-Style Explicit Trigger

A v1 design would be: "Judgment: If you played 2+ fast cards this turn, draw 2." That is Gale-only. This card's threshold is satisfied by ANY deck that empties its hand (Gale, Cinder, Eclipse, Tempest post-storm) and the alternative mode is desirable for ANY deck that holds cards (Crucible, Basalt, Depths). The same card text creates seven distinct use cases. The drafter's decision is not "am I in the right archetype?" but "which mode does my deck naturally land in, and is that the mode I want?"

---

## Card 2: Nexus of Passing

**Cost:** 4 energy | **Type:** Character | **Subtype:** Ancient | **Spark:** * (variable) | **Rarity:** Rare | **Resonance:** Neutral

**Ability text:**

> This character's spark is equal to the number of cards that have changed zones this turn.

### Context Map

| Archetype | Zone-Change Sources | Expected Spark per Turn | Why It Wants This Card |
|-----------|---------------------|------------------------|------------------------|
| **Mirage** (Tide+Zephyr) | Each flicker = 2 zone changes (battlefield -> banish, banish -> battlefield). A single Portal of Twin Paths flickering 2 allies = 4 zone changes. Aurora Rider flickering 3 allies = 6. Nomad of Endless Paths bouncing an ally = 1 (battlefield -> hand), replaying it = 1 (hand -> battlefield) = 2. Drawing cards via Materialized = 1 each (deck -> hand). | **6-10 spark** on a flicker turn | Mirage's primary action (flicker) generates the highest density of zone changes in the game. This card is Mirage's best finisher -- it scales with flicker volume without mentioning flicker, banish, or Materialized. |
| **Cinder** (Ember+Ruin) | Each abandon = 1 zone change (battlefield -> void). Each Reclaim play = 1 (void -> battlefield). Drawing from Specter of Silent Snow = 1 (deck -> hand). Spirit Reaping converting a body to energy doesn't move the card to a new zone -- wait, abandon DOES move to void. So 3 abandons = 3, replaying 2 via Reclaim = 2, incidental draws = 2-3. | **5-8 spark** on a sacrifice turn | Cinder's sacrifice-Reclaim loops generate zone changes through a completely different mechanism than Mirage's flicker. The same card becomes a sacrifice payoff without ever mentioning "abandon." |
| **Tempest** (Tide+Ember) | Each event played = 2 zone changes (hand -> stack/battlefield, then -> void). Playing 5 events = 10 zone changes. Drawing cards = 1 each (deck -> hand). Energy events that draw (Data Pulse) add draws. | **8-12 spark** on a storm turn | Tempest generates the most raw zone changes per turn. This card is Tempest's best single-turn finisher: play events, then score with a character whose spark equals your storm count times two (approximately). But it requires surviving to Judgment with the character in play, creating tension that "gain 1 point per event" does not. |
| **Eclipse** (Zephyr+Ruin) | Each discard = 1 zone change (hand -> void). Each draw = 1 (deck -> hand). A Moonlit Voyage (draw 2, discard 2) = 4 zone changes. Nocturne's Materialized (draw 1, discard 1) = 2. Reclaim from void to hand = 1. | **4-7 spark** on a cycling turn | Eclipse's draw-discard cycling generates moderate zone changes. The card is a respectable body that rewards the cycling engine without demanding "when you discard" triggers. |
| **Undertow** (Tide+Ruin) | Each mill = 1 zone change (deck -> void) per card. Searcher in the Mists (4 cards milled) = 4 zone changes. Harvest the Forgotten (3 milled + 1 drawn) = 4. Flagbearer of Decay (2 milled per Survivor played) = 2 per Survivor. | **4-8 spark** on a mill turn | Undertow's self-mill generates zone changes as a side effect. This card rewards aggressive milling without being a void-size-matters card (which would compete with Abomination of Memory). |

### Synergy Vectors Exploited

- **V19 (Leaves-Play):** Every "leaves play" event is a zone change.
- **V15 (Abandon-as-Pseudo-Flicker):** Both abandon and flicker generate zone changes, creating the Mirage-Cinder draft tension identified as the #1 most valuable non-adjacent bridge.
- **V29 (Void Velocity):** Cards entering the void are zone changes, making this a void-velocity payoff without using the word "void."

### Counter-Pattern Used

**Counter-Pattern 3 (Cross-Zone Scaling)** from the mechanic critique. The card counts a universal metric (zone changes) that every archetype contributes to through its natural game plan. The key design insight is that "zone change" is not an archetype action -- it is a consequence of EVERY archetype's action. Flicker, sacrifice, storm, cycling, and mill all generate zone changes at different rates, making this a universal finisher whose ceiling depends on your deck's engine speed.

### Why This Is Better Than a v1-Style Explicit Trigger

A v1 design would be: "When an ally is banished, +1 spark" (Tideborne Voyager). That is Mirage-only. This card's metric is satisfied by every major mechanical action in the game. The drafter asks: "How many zone changes does my deck generate per turn?" -- a question that requires understanding your own engine's throughput, not just checking which archetype you are in.

### Balance Note

The card's spark is only measured at Judgment, meaning it scales with the CURRENT turn's activity, not cumulative. A player who does nothing on a turn gets 0 spark from this body. This prevents it from being a "play and forget" threat -- you must keep your engine running every turn. The 4-cost is appropriate for a card that demands active support but has no ceiling.

---

## Card 3: Crucible of the Commons

**Cost:** 3 energy | **Type:** Character | **Subtype:** Visitor | **Spark:** 2 | **Rarity:** Uncommon | **Resonance:** Stone

**Ability text:**

> Judgment: Each ally with spark 1 or less gains +1 spark until end of turn.

### Context Map

| Archetype | Low-Spark Allies Present | Effective Bonus | Why It Wants This Card |
|-----------|--------------------------|-----------------|------------------------|
| **Crucible** (Stone+Ember) | Wolfbond Chieftain (0), Ethereal Trailblazer (0), Company Commander (0), Spirit Field Reclaimer (0), Dawnblade Wanderer (0), Bloomweaver (1). Typically 3-4 of these on board by midgame. | +3 to +4 total spark per Judgment | Crucible's Warrior infrastructure is built on 0-1 spark utility bodies. This card makes every ramp Warrior and lord a Judgment contributor. Skyflame Commander's +1 spark lord buff pushes some of these above the threshold (Bloomweaver goes to 2 from lord, then doesn't qualify), creating a genuine sequencing puzzle: do you play the lord first (fewer targets for this) or this first (more targets)? |
| **Basalt** (Zephyr+Stone) | Ebonwing (1), Dawnprowler Panther (1), Luminwings (1), Driftcaller Sovereign (1), Ghostlight Wolves (1), Sunshadow Eagle (1), Eternal Stag (1), Looming Oracle (1). Typically 3-5 of these on board. | +3 to +5 total spark per Judgment | Spirit Animals are the lowest-spark tribe in the game -- twelve of eighteen have exactly 1 spark. This card gives every Spirit Animal an extra point of Judgment contribution without requiring the 4-energy activation of Spiritbound Alpha. It stacks WITH Spiritbound Alpha's boost (a 1-spark Spirit Animal becomes 2 from this, then +2 from Alpha = 4). |
| **Cinder** (Ember+Ruin) | Figment tokens (0 spark), Exiles of the Last Light (1), Soulbinder (0), Resilient Wanderer (1). Typically 2-3 sacrifice fodder bodies at 0-1 spark. | +2 to +3 total spark per Judgment | In Cinder, this card turns sacrifice fodder into Judgment threats the turn BEFORE you sacrifice them. The decision: do you sacrifice a figment this turn (triggering abandon payoffs) or keep it alive for the +1 spark at Judgment? This creates a timing tension that pure sacrifice decks currently lack. |
| **Mirage** (Tide+Zephyr) | Looming Oracle (1), Keeper of the Tides (1), Herald of the Last Light (1), Nomad of Endless Paths (1), various flicker utility bodies at 1 spark. Typically 3-4 low-spark targets. | +3 to +4 total spark per Judgment | Mirage fields many low-spark utility bodies that exist only for their Materialized triggers. This card makes those bodies matter at Judgment. It also works with figments from Radiant Trio or Endless Projection -- figments gain +1 spark, turning them from pure board width into actual scoring threats. |
| **Undertow** (Tide+Ruin) | Hope's Vanguard (1), Seer of the Fallen (1), Dustborn Veteran (1), Flagbearer of Decay (1), Searcher in the Mists (1), Emberwatch Veteran (0). Typically 2-4 Survivors at 0-1 spark. | +2 to +4 total spark per Judgment | Many Survivors are 1-spark utility bodies. This pumps them without requiring Survivor tribal density -- any deck with incidental Survivors benefits. |

### Synergy Vectors Exploited

- **V03 (Board-Width-from-Low-Spark-Bodies):** This is the definitive board-width payoff. It rewards going wide with cheap utility creatures regardless of subtype.
- **V21 (Judgment Storm):** The effect fires during Judgment, contributing to the "Judgment matters" density that Stone builds.
- **V37 (Temporary Spark Matters):** The "until end of turn" clause means the spark is temporary -- if future Zephyr cards care about temporary spark, this card feeds that axis.

### Counter-Pattern Used

**Counter-Pattern 4 (Conditional Commons Boost)** from the mechanic critique. The spark-1-or-less threshold means the card rewards boards full of cheap utility creatures -- exactly the board state that Crucible, Basalt, Cinder, and Mirage all naturally build. The threshold also creates anti-synergy with decks that already buff their creatures above 1 spark (Skyflame Commander's lord effect pushes Warriors to 2+), generating a genuine deckbuilding tension: more lords = fewer targets for this card.

### Why This Is Better Than a v1-Style Explicit Trigger

A v1 design would be: "Judgment: Each allied Warrior with spark 1 or less gains +1 spark." That is Crucible-only. By removing the subtype lock, the SAME text pumps Warriors in Crucible, Spirit Animals in Basalt, figments in Cinder, flicker targets in Mirage, and Survivors in Undertow. The card never mentions a tribe, so the drafter evaluates it by counting low-spark allies in their deck -- a calculation that differs completely by archetype.

### Resonance Justification

Mono-Stone is correct because the effect is a Judgment-phase incremental board buff -- the purest expression of Stone's "board durability" and "incremental value" themes. It does not sacrifice (not Ember), does not draw (not Tide), does not bounce (not Zephyr), and does not recurse (not Ruin). It just makes your existing board better, slowly and repeatedly.

---

## Card 4: Archivist of Vanished Names

**Cost:** 3 energy | **Type:** Character | **Subtype:** Mage | **Spark:** 1 | **Rarity:** Rare | **Resonance:** Tide

**Ability text:**

> Materialized: Name a card type (Character or Event). Reveal cards from the top of your deck until you reveal a card of that type. Put the revealed card into your hand and the rest into your void.

### Context Map

| Archetype | Type Named | Expected Cards Milled | Why It Wants This Card |
|-----------|------------|----------------------|------------------------|
| **Tempest** (Tide+Ember) | **Event** | In a Tempest deck (~55-60% events), you mill 0-2 characters to find the next event. You get a guaranteed event for your storm chain and the milled characters fuel Spirit of Smoldering Echoes (which tracks events entering void -- wait, the milled characters are NOT events). Actually: the non-events milled are characters going to void, which does NOT trigger Spirit of Smoldering Echoes. But the found event itself is free card advantage -- you drew a guaranteed chain piece. | Tempest names Event to guarantee chain fuel. The 0-2 milled characters are acceptable collateral. This is "Demonic Tutor for events" in a deck that is 60% events, with void-filling as upside. |
| **Mirage** (Tide+Zephyr) | **Character** | In a Mirage deck (~55% characters), you mill 0-2 events to find the next flicker target or Materialized body. The milled events go to void, where Ashlight Caller can give them Reclaim. | Mirage names Character to guarantee a flicker target. The milled events are retrievable via Mirage's event-recovery splash. The Materialized trigger means this can be flickered to repeat the search. |
| **Undertow** (Tide+Ruin) | **Either** | Undertow does not care which type it names because the MILLING is the primary value. Naming whichever type is rarer in your deck maximizes cards milled. In a character-heavy Undertow deck, name Event to mill through many characters. In an event-heavy build, name Character. Either way, you mill 3-6 cards and get one useful card for your trouble. | Undertow uses this as a self-mill engine that also draws. The choice of type is a deckbuilding calculation: which type is rarer in my deck? Naming the rarer type mills more cards. |
| **Bedrock** (Stone+Ruin) | **Character** | Bedrock names Character to find a reanimation target OR to mill expensive characters into the void. If the top card happens to be a character, you draw it; if it is several events deep, you mill those events while searching. The real dream: you have Titan of Forgotten Echoes or The Devourer in your deck, and this card mills several events while putting the expensive body directly into your hand for later discard to void (via Entomb) or into your void as collateral. | Bedrock names Character. If the top card is a cheap character, you drew it. If expensive characters are deeper, you milled events toward them. Either way, you gained card advantage and void fuel. |
| **Eclipse** (Zephyr+Ruin) | **Event** | Eclipse names Event to find a discard-cycling piece (Moonlit Voyage, Fragments of Vision). The milled characters go to void, growing void count for Architect of Memory / Abomination of Memory. If Ridge Vortex Explorer is milled, it materializes itself from the void. | Eclipse uses this as a dual-purpose tool: find a cycling event AND mill characters to void. |

### Synergy Vectors Exploited

- **V04 (Foresee-to-Top-of-Deck Pipeline):** This card interacts with Foresee -- if you Foresee first and know the top card is the type you want, naming that type finds it immediately with zero mill. If the top card is NOT the type you want, Foresee away, then use this card.
- **V06 (Event-Count-in-Void):** When naming Character (Mirage, Bedrock), the milled events go to void, growing the event-void-count for Spirit of Smoldering Echoes.
- **V29 (Void Velocity):** The milling is incidental but can send 3-6 cards to void in one trigger, creating high void velocity.

### Counter-Pattern Used

**Counter-Pattern 1 (Named-Type Choice)** from the mechanic critique. The binary choice (Character or Event) is the simplest possible modal, but it creates completely different card evaluations depending on deck composition. Tempest decks want Event; Mirage decks want Character; Undertow decks want whichever mills more. The choice is not "which mode is better?" but "what does my deck look like right now?" -- a question with a different answer every game.

### Why This Is Better Than a v1-Style Explicit Trigger

A v1 design would be: "Materialized: Put the top 3 cards of your deck into your void. Draw 1 for each event." That is Tempest-weighted. This card's binary choice means different decks use it differently, and the AMOUNT of milling is stochastic, creating variance that rewards deckbuilding (controlling your character/event ratio). The Materialized trigger makes it a prime flicker target for Mirage, adding another layer of repeatable value.

### Balance Note

The "reveal until" mechanic has no upper bound -- in theory, you could mill your entire deck. In practice, decks are roughly 50/50 characters and events, meaning you typically reveal 1-3 cards. In a 40-card draft deck with 22 characters and 18 events, naming Event finds one within 2.2 cards on average. Naming Character finds one within 1.8 cards on average. The ceiling case (milling 8+ cards) requires an extreme deck composition and is self-punishing (you are decking yourself).

---

## Card 5: Ember of Recurrence

**Cost:** 3 energy | **Type:** Character | **Subtype:** Synth | **Spark:** 1 | **Rarity:** Rare | **Resonance:** Ruin

**Ability text:**

> When a card enters your void from any zone, you may pay 1 energy: Return a different card from your void to your hand.

### Context Map

| Archetype | Primary Trigger Source | Rate of Triggers | How It Uses the Retrieved Card |
|-----------|----------------------|------------------|-------------------------------|
| **Eclipse** (Zephyr+Ruin) | Every discard (hand -> void) triggers this. A single Moonlit Voyage (draw 2, discard 2) fires the trigger TWICE. Chronicle Reclaimer's Judgment (draw 1, discard 1) fires it once. Ashmaze Guide's discard-Reclaim interaction creates a loop: discard card A -> it enters void -> pay 1: retrieve card B -> card A gains Reclaim from Ashmaze. | **3-5 triggers per turn** in active cycling | Eclipse retrieves cycling pieces from void to cycle them again. The retrieved card is fuel for the next discard cycle. This is a perpetual motion engine where every discard both fills the void and retrieves from it. The 1-energy cost per activation prevents infinite loops but allows 2-3 retrievals per turn. |
| **Cinder** (Ember+Ruin) | Every abandon (battlefield -> void) triggers this. Abandoning 2 bodies per turn = 2 triggers. Spirit Reaping (abandon for energy) fires it. Desperation (abandon all for draw) fires it per body. | **2-4 triggers per turn** during sacrifice turns | Cinder retrieves sacrifice fodder from void to sacrifice again next turn. The retrieved card is the SAME body that was just abandoned, creating a recurring sacrifice loop. Pay 1 energy per loop cycle. This competes with Reclaim (which is free but sorcery-speed) as an alternative recursion path. |
| **Undertow** (Tide+Ruin) | Every mill (deck -> void) fires this. Searcher in the Mists (mill 4) fires it 4 times. Harvest the Forgotten (mill 3) fires it 3 times. Flagbearer of Decay (mill 2 per Survivor) fires it 2+ times. | **4-8 triggers per turn** during mill turns | Undertow cherry-picks the best milled card from each batch. Instead of random mill being uncontrollable, this card lets you retrieve the one Survivor or key card that you needed. The high trigger rate means Undertow can retrieve 2-3 cards per mill turn (spending 2-3 energy), transforming random mill into selective draw. |
| **Tempest** (Tide+Ember) | Every event that resolves goes to void. Playing 5 events = 5 triggers. Each trigger lets you retrieve a previously played event for replay next turn. | **3-6 triggers per turn** during storm turns | Tempest retrieves key events for replay. After a storm turn, retrieve the best event (Echoes of Eternity, Genesis Burst, Flash of Power) from the void to hand for next turn's storm. This is event recovery that Tempest desperately needs -- currently only Whisper of the Past and Archive of the Forgotten provide this, and both are expensive. |
| **Bedrock** (Stone+Ruin) | Self-mill and discard effects that put reanimation targets into void. Each card entering void from Entomb, Harvest the Forgotten, or Ashborn Necromancer triggers this. | **2-3 triggers per turn** | Bedrock retrieves reanimation targets that accidentally got milled too deep. If you mill 3 cards and one is a key target, you can retrieve a DIFFERENT card from void (perhaps a cheap recursion enabler) while the target stays in void for Reclaim. Or retrieve the target directly to hand for hard-casting with Stone's ramp. |

### Synergy Vectors Exploited

- **V29 (Void Velocity):** The card rewards high void-filling speed by offering more retrieval opportunities per turn.
- **V30 (From-Void Materialized):** Retrieved characters can be replayed, triggering Materialized abilities -- creating a pseudo-flicker loop through void recursion.
- **V15 (Abandon-as-Pseudo-Flicker):** In Cinder, abandoning and retrieving creates a hand-battlefield-void-hand cycle that generates Materialized triggers on replay.
- **V33 (Event Reclaim Engine):** In Tempest/Eclipse, retrieving events from void IS event Reclaim, just through a different mechanism.

### Counter-Pattern Used

**Counter-Pattern 9 (Conditional Recursion with Archetype-Dependent Trigger)** from the mechanic critique. The trigger "when a card enters your void from any zone" is the most universal trigger possible in Dreamtides -- every archetype's primary action sends cards to the void. The 1-energy cost is the throttle that prevents degeneracy while allowing 2-3 activations per turn in energy-rich decks.

### Why This Is Better Than a v1-Style Explicit Trigger

A v1 design would be: "When you discard a card, you may return a card from your void to your hand." That is Eclipse-only. By using "enters your void from ANY zone," the trigger fires on discards (Eclipse), abandons (Cinder), mill (Undertow), event resolution (Tempest), and combat deaths (everyone). The RATE of triggers varies by archetype, meaning the card's power level scales with how aggressively your deck fills the void -- a deckbuilding signal, not an archetype label.

### Balance Note

The 1-energy cost prevents infinite loops: you need 1 energy per retrieval, and energy is finite. The "different card" requirement prevents self-loops (you cannot pay 1 to return the card that just entered). In a Cinder deck abandoning 3 bodies, you could retrieve 3 cards for 3 energy -- significant value but a real energy cost that competes with playing the retrieved cards. In Undertow milling 8 cards, you could retrieve up to 8 (if you had 8 energy), but realistically you retrieve 2-3. The card is a scaling engine, not a combo piece.

---

## Summary Table

| # | Card Name | Cost | Type | Resonance | Primary Counter-Pattern | Archetypes (3+ minimum) | Key Synergy Vectors |
|---|-----------|------|------|-----------|------------------------|------------------------|---------------------|
| 1 | Dreamtide Cartographer | 3 | Character (Explorer) | Neutral | Threshold-Gated Mode Switch (CP-2) | Gale, Eclipse, Crucible, Basalt, Cinder, Depths, Tempest | V01, V14, V03 |
| 2 | Nexus of Passing | 4 | Character (Ancient) | Neutral | Cross-Zone Scaling (CP-3) | Mirage, Cinder, Tempest, Eclipse, Undertow | V19, V15, V29 |
| 3 | Crucible of the Commons | 3 | Character (Visitor) | Stone | Conditional Commons Boost (CP-4) | Crucible, Basalt, Cinder, Mirage, Undertow | V03, V21, V37 |
| 4 | Archivist of Vanished Names | 3 | Character (Mage) | Tide | Named-Type Choice (CP-1) | Tempest, Mirage, Undertow, Bedrock, Eclipse | V04, V06, V29 |
| 5 | Ember of Recurrence | 3 | Character (Synth) | Ruin | Conditional Recursion (CP-9) | Eclipse, Cinder, Undertow, Tempest, Bedrock | V29, V30, V15, V33 |

### Design Quality Metrics

- **Average archetypes per card:** 5.4 (vs. v1's 1.8 average)
- **Cards with explicit subtype references:** 0 of 5
- **Cards with archetype-naming triggers:** 0 of 5
- **Cards where different archetypes want it for DIFFERENT reasons:** 5 of 5
- **Counter-patterns from mechanic critique used:** 5 of 10 (CP-1, CP-2, CP-3, CP-4, CP-9)

### Resonance Distribution

- Neutral: 2 (Dreamtide Cartographer, Nexus of Passing)
- Stone: 1 (Crucible of the Commons) -- contributes to Stone's 9-card deficit
- Tide: 1 (Archivist of Vanished Names)
- Ruin: 1 (Ember of Recurrence)

### Gaps Addressed

1. **Stone deficit:** Crucible of the Commons is mono-Stone, contributing 1 card toward the 9-card shortfall.
2. **Crucible linearity (9/10 rails):** Dreamtide Cartographer and Crucible of the Commons are non-Warrior cards that Crucible wants, diluting Warrior auto-includes and creating purity-vs-power decisions.
3. **Gale signpost gap:** Dreamtide Cartographer is a premium Gale card (draw mode) that also rewards other archetypes, creating draft tension.
4. **Mirage-Cinder bridge (#1 priority non-adjacent bridge):** Nexus of Passing is the highest-value cross-archetype finisher for both Mirage (flicker zone changes) and Cinder (sacrifice zone changes), creating direct draft competition between non-adjacent archetypes.
5. **Eclipse depth:** Ember of Recurrence gives Eclipse a new engine piece (discard -> retrieve cycle) beyond its thin 6-card payoff layer.
6. **Tempest variety:** Archivist of Vanished Names is a Tempest card that does NOT use the "2+ events this turn" trigger -- it uses a named-type choice, giving Tempest a mechanically distinct tool.
