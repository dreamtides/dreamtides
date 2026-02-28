# Ember Synergy Discovery Report -- Hidden Vectors and Cross-Resonance Bridges

## Purpose

This document does not describe what Ember IS. That work is done. This document identifies what Ember COULD synergize with -- game-state conditions that Ember's mechanics produce as unintentional side effects, and which no current card in the pool explicitly exploits. These are the seams where future connector cards could create surprising cross-resonance interactions.

---

## 1. Side Effect Inventory

Ember's five core themes (removal, abandon-for-value, burst energy, aggressive spark, discard-as-cost) each produce game-state conditions beyond their intended purpose.

### Side Effect 1: Relative Board Advantage from Enemy Dissolution
**Source:** Dissolve/Banish effects (Immolate, Fury of the Clan, Scorched Reckoning, Apocalypse, Burst of Obliteration, Obliterator of Worlds, etc.)
**Intended purpose:** Remove opponent's threats.
**Unintended side effect:** Ember decks often achieve asymmetric board states -- many allies, few enemies -- without explicitly building around "go wide." A Crucible deck that removes 2-3 blockers has a wider effective board than its raw character count suggests. This is a *relative* board density that no card currently keys off.

### Side Effect 2: Self-Void Growth from Abandon
**Source:** Every abandon effect (Forsaker, Desperation, Spirit Reaping, Blade of Oblivion, Forsworn Champion, Exiles of the Last Light, etc.)
**Intended purpose:** Power immediate payoffs (spark, points, kindle, energy).
**Unintended side effect:** Abandoned characters go to YOUR void. Over a game, an Ember/Cinder player's void fills rapidly with characters -- specifically characters, not events. This is distinct from Undertow's void-filling (mill, which is random) and Eclipse's (discard from hand, which is mixed). Ember's void is character-dense by construction.

### Side Effect 3: Temporary Resource Spikes Enable Off-Curve Plays
**Source:** Genesis Burst, Flash of Power, Catalyst Ignition, Arc Gate Opening, Spirit Reaping, Pulse of Sacrifice.
**Intended purpose:** Fuel storm chains (Tempest) or deploy multiple fast threats (Gale).
**Unintended side effect:** These energy spikes can fund ANY expensive play, not just chained events. A 6-cost or 8-cost body played two turns early is game-warping. Currently, Ember's energy burst is funneled into Tempest (event chains) or Gale (fast deployment). No card specifically rewards the pattern of "burst energy into a single expensive play."

### Side Effect 4: Frequent Dissolution Triggers (on YOUR side)
**Source:** Aggressive bodies with low durability, abandon effects, Nightmare Manifest's forced symmetrical sacrifice, Apocalypse.
**Intended purpose:** Exchange bodies for value.
**Unintended side effect:** Ember generates more allied Dissolved events per game than any resonance except dedicated Cinder. Even non-Cinder Ember decks (Tempest, Gale, Crucible) lose allies regularly through combat, abandon, and sweepers. This creates a latent trigger density that Dissolved-trigger cards could exploit in decks that are not Cinder.

### Side Effect 5: Hand Depletion
**Source:** Discard-as-cost (Fell the Mighty, Echoing Denial, Pyrokinetic Surge), Pulse of Sacrifice, Rebirth Ritualist, aggressive low-curve deployment.
**Intended purpose:** Power immediate effects.
**Unintended side effect:** Ember players end up hellbent (empty-handed) more often than other resonances. An empty hand is usually bad, but it is also a game-state condition that could be exploited. Currently no card in the pool rewards having an empty hand.

### Side Effect 6: Opponent's Void Growth from Your Removal
**Source:** Every Dissolve effect sends the opponent's character to THEIR void.
**Intended purpose:** Remove threats.
**Unintended side effect:** Ember's removal unintentionally fills the opponent's void. In most games this is irrelevant. But against Ruin-based opponents (Undertow, Cinder, Bedrock, Eclipse), your removal actively fuels their void-matters payoffs (Abomination of Memory, Architect of Memory, Weight of Memory). This is a hidden anti-synergy. Banish effects (Fell the Mighty, Veil Shatter, Judgment of the Blade) avoid this problem because they exile rather than void.

### Side Effect 7: Events Accumulating in Void
**Source:** Tempest plays 8-15 events per game; even non-Tempest Ember decks play 4-6 removal events. All resolve to void.
**Intended purpose:** N/A -- this is purely a consequence of playing events.
**Unintended side effect:** Ember players (especially Tempest) have event-dense voids. Spirit of Smoldering Echoes already keys off this (+1 spark per event entering void), but the condition "number of events in your void" is underexploited. Only Whisper of the Past, Archive of the Forgotten, and Ashlight Caller interact with events in void, and those are Tide/Ruin recovery tools, not Ember payoffs.

### Side Effect 8: Kindle Concentration
**Source:** Infernal Ascendant (kindle 2 on abandon), Exiles of the Last Light (kindle 1 on abandon), Spirit Field Reclaimer (kindle 1 on Judgment).
**Intended purpose:** Distribute spark across characters.
**Unintended side effect:** Kindle always goes to the leftmost character. In Ember decks with active abandon loops, kindle accumulates on a single survivor -- the character that is NOT being sacrificed. This creates a single high-spark character rather than a distributed board. Shardwoven Tyrant's ability (dissolve enemy with spark < abandoned ally's spark) already partly exploits this, but the "one tall character" pattern is broader than that single interaction.

---

## 2. Cross-Resonance Bridge Opportunities

These are existing cards in OTHER resonances that would become surprisingly effective in Ember archetypes if the right connector existed.

### 2a. Ruin Cards That Ember Enables Accidentally

**Abomination of Memory** (Ruin -- Undertow/Bedrock): Spark equal to void size. Ember's abandon chain fills the void with 5-8 characters over a game. A Cinder drafter already has access to Ruin, but Abomination is currently classified as Undertow/Bedrock. If Cinder's void reliably hits 7+ (which it does through abandon + Apocalypse), Abomination becomes a 7+ spark body in Cinder -- arguably better than in Undertow, where mill fills the void less predictably.

**Weight of Memory** (Ruin -- Undertow): Dissolve enemy with cost less than void size. In a Cinder deck that has abandoned 5-6 bodies, this becomes "Dissolve any enemy with cost 5 or less for 1 energy" -- essentially Immolate at one-third the cost. Currently classified as pure Undertow, but Cinder's void count is just as reliable.

**Architect of Memory** (Ruin -- Undertow/Bedrock): While void >= 7, all cards have Reclaim equal to cost. In Cinder, reaching 7 void cards is faster than in Undertow (abandon is more controllable than mill). Once active, every abandoned character becomes Reclaimable, creating infinite sacrifice loops. The card is classified as Undertow/Bedrock, but a Cinder deck that meets the threshold gets disproportionate value.

### 2b. Zephyr Cards That Ember Could Bridge To

**Starlit Cascade** (Zephyr -- Mirage primary): When ally leaves play, gain 2 energy. Already noted as a Cinder secondary card, but it is underappreciated in Crucible. A Crucible Warrior sacrifice turn (abandon 2 Warriors for Blade of Oblivion's dissolve trigger) generates 4 energy from Starlit Cascade -- enough to replay both Warriors. This is a hidden Crucible engine piece disguised as a Mirage card.

**Nomad of Endless Paths** (Zephyr -- Gale): Return an ally to hand on Materialized. In a Crucible deck with lords, bouncing a Warrior to replay it triggers Company Commander (+1 spark) and Invoker of Myths (draw 1). Currently viewed as a Gale tempo card, but it functions as a pseudo-flicker engine in a Warrior deck with materialize-matters payoffs. The bridge would require a card that explicitly connects "return to hand" with "Warrior enters play" -- which Crucible's lords already do implicitly.

**Moonlit Dancer** (Zephyr -- Gale): Characters in hand have fast. In Crucible, this turns every redeployed Warrior into a fast threat. Combined with Grim Reclaimer (Reclaim a Warrior by abandoning an ally), you create a Reclaim-then-deploy-at-fast-speed loop. Currently this interaction is two resonance-hops away (Stone -> Ember -> Zephyr), but Gale's Zephyr+Ember identity already sits at that junction.

### 2c. Tide Cards That Ember's Storm Enables

**Celestial Reverie** (Tide -- Mirage/Tempest secondary): Play a character, draw 1 until end of turn. Tempest is already a secondary user, but the card's power scales with character density, not event density. A hypothetical connector card that generates figment tokens from event plays (Packcaller of Shadows already does this!) would make Celestial Reverie a massive draw engine in Tempest: play events -> generate figments -> draw from figment materializations. This loop exists but is unrecognized as a synergy chain.

**The Power Within** (Tide -- Tempest): Double drawn cards from effects. In Cinder, Prophet of the Consumed draws 1 per ally abandoned this turn. If you abandon 3 allies with The Power Within active, Prophet draws 6 instead of 3. This cross-archetype combo (Cinder sacrifice + Tempest draw doubling) is accessible because both archetypes share Ember, but no card currently bridges them.

### 2d. Stone Cards in Ember Contexts

**Surge of Fury** (Stone -- Crucible/Basalt): Extra Judgment phase. In Cinder, an extra Judgment triggers Nightmare Manifest TWICE (each player abandons a character per Judgment), Eclipse Herald TWICE (banish 3 from void to dissolve), and The Dread Sovereign TWICE (abandon Warrior to upgrade). Currently classified as Crucible/Basalt, but Cinder's Judgment-trigger density is comparable. The disconnect is that Cinder's Ruin half does not access Stone cards easily.

**Conduit of Resonance** (Stone -- Basalt): When you materialize a character, trigger all Judgment abilities. In Cinder, materializing a sacrifice body would trigger Nightmare Manifest (force abandon), Eclipse Herald (void-to-dissolve), and The Dread Sovereign (sacrifice-upgrade). This is a devastating combo, but the card is locked in Zephyr resonance (Basalt core), making it inaccessible to Cinder.

---

## 3. Hidden Synergy Vectors

### Vector 1: Void Character Density as a Unique Metric
**Condition:** Ember's abandon effects fill the void specifically with characters (not events, not mixed). After 4-5 abandons, a Cinder void might have 5+ characters and 0-2 events.
**Archetypes bridged:** Cinder (Ember+Ruin) <-> Bedrock (Stone+Ruin)
**Connector card concept:** "Materialize the highest-cost character from your void. It gains 'Abandon this at end of turn.'" -- a one-turn reanimation that triggers Cinder's abandon payoffs when the body self-sacrifices. Bedrock provides the expensive targets; Cinder provides the sacrifice infrastructure.
**Non-obviousness: 7/10.** Players associate Bedrock with permanent reanimation. A temporary reanimation that triggers abandon payoffs inverts the expected interaction: you WANT the reanimated body to die because its death is the payload.

### Vector 2: Dissolution Chain Cascade (Non-Cinder)
**Condition:** Ember's removal causes opponent characters to be dissolved. Dissolved triggers like Sunset Chronicler and Avatar of Cosmic Reckoning fire when ANY ally is dissolved -- including opponents' characters that you dissolve. Wait -- actually no, these trigger on "an ally" meaning YOUR ally. But Apocalypse ("dissolve ALL characters") dissolves YOUR allies too, firing every Dissolved trigger you control simultaneously.
**True condition:** In a Crucible deck (not Cinder), Apocalypse with 4 Warriors on board fires 4 Dissolved events. Currently no Crucible card cares about this. But if a Warrior had "Dissolved: Kindle 2" (Silent Avenger exists for Survivors but not for Warriors), Apocalypse becomes "dissolve everything, kindle 8 to your leftmost surviving... oh wait, everything is dissolved." This reveals a deeper pattern: Dissolved triggers need at least one character to SURVIVE the Apocalypse. The real synergy is Dissolved triggers on characters that survive sweepers (via indestructibility or post-sweep recursion).
**Archetypes bridged:** Crucible (Stone+Ember) <-> Undertow (Tide+Ruin)
**Connector card concept:** "When a Warrior you control is dissolved, you may Reclaim a Warrior from your void." This bridges Crucible's Warrior density with Ruin's recursion -- the Warrior tribe version of Dustborn Veteran's self-Reclaim.
**Non-obviousness: 6/10.** Warrior recursion exists (Grim Reclaimer, Ashen Avenger) but none triggers automatically on dissolution. Moving from active abandon to passive dissolution-triggered Reclaim changes the deck's texture significantly.

### Vector 3: Hellbent Payoffs (Empty-Hand Exploitation)
**Condition:** Ember's discard-as-cost and aggressive deployment leave the player empty-handed more frequently than any other resonance. Pulse of Sacrifice literally discards your entire hand. Fell the Mighty and Echoing Denial each cost a card from hand.
**Archetypes bridged:** Gale (Zephyr+Ember) <-> Eclipse (Zephyr+Ruin) -- both share Zephyr
**Connector card concept:** "While you have no cards in hand, characters you control have +1 spark" or "While you have no cards in hand, your fast characters cost 1 less." This rewards the natural endgame state of both Gale (deployed everything at fast speed) and aggressive Ember decks. It is anti-synergistic with Tide (which always has cards in hand), creating clean resonance separation.
**Non-obviousness: 8/10.** Deckbuilding games rarely reward empty hands because it feels like losing. But in a fast-tempo game where you have already deployed your threats, the empty hand is a signal of successful execution, not failure. The card would need careful tuning to avoid punishing card draw.

### Vector 4: Event Void Density as Ember's Unique Void Signature
**Condition:** Tempest fills the void with events; Cinder fills it with characters. These are distinct void compositions that no card currently distinguishes.
**Archetypes bridged:** Tempest (Tide+Ember) <-> Undertow (Tide+Ruin) -- both share Tide
**Connector card concept:** "Gain energy equal to the number of events in your void" (proposed as Leyline Detonation in the Tempest allocation). Or the inverse: "Dissolve an enemy with cost less than the number of characters in your void" -- a character-void version of Weight of Memory that Cinder would use better than Undertow.
**Non-obviousness: 5/10.** The Tempest allocation already proposed Leyline Detonation. The character-void version for Cinder is the less obvious half. Weight of Memory already counts total void cards; splitting the metric by card type creates archetype differentiation within Ruin.

### Vector 5: Burst Energy into Activated Abilities
**Condition:** Ember generates 4-10 energy in a single burst. Activated abilities with energy costs (Assault Leader: 4 energy for +1 spark per Warrior; Spiritbound Alpha: 4 energy for all Spirit Animals +2 spark; Mystic Runefish: 3 energy for all Spirit Animals become 5 spark) can consume burst energy in non-Tempest contexts.
**Archetypes bridged:** Crucible (Stone+Ember) <-> Basalt (Zephyr+Stone) -- both share Stone
**Connector card concept:** "Until end of turn, activated abilities cost 2 less" or "Gain energy equal to the number of allies with activated abilities." This turns Ember's burst energy into a Crucible win condition (Assault Leader for free) or a Basalt alpha strike (Spiritbound Alpha + Mystic Runefish in one turn). The bridge is Stone, which both archetypes share.
**Non-obviousness: 7/10.** Players think of burst energy as Tempest fuel. Using it to power activated abilities in creature-based decks is a completely different gameplay pattern that happens to use the same resource.

### Vector 6: Abandon as Pseudo-Flicker (Materialize-Matters Exploitation)
**Condition:** In Cinder, characters leave the battlefield (abandon) and return (Reclaim + replay). Each re-entry triggers Materialized abilities. Cards like Bloomweaver (gain 1 energy on materialize), Angel of the Eclipse (gain 1 energy on ally materialize), Company Commander (gain +1 spark on Warrior materialize), and Lumin-Gate Seer (draw 1 on cheap materialize) all trigger on materialization, regardless of whether it came from flicker or sacrifice-and-Reclaim.
**Archetypes bridged:** Cinder (Ember+Ruin) <-> Mirage (Tide+Zephyr)
**Connector card concept:** The connector already exists conceptually -- materialize-matters cards. But no single card explicitly bridges sacrifice-Reclaim cycles to materialize payoffs. A card like "When you play a character from your void, draw 1 and gain 1 energy" would be the missing link: it triggers on Reclaimed characters (Cinder's loop) and on void-recursive Mirage targets (Shadowpaw's return-to-hand chains).
**Non-obviousness: 8/10.** Mirage players think of materializations as coming from flicker. Cinder players think of materializations as deployment costs for sacrifice fodder. The insight that both archetypes generate massive materialization counts through completely different mechanisms is non-obvious and would create genuine draft tension over materialize-matters cards.

### Vector 7: Kindle Accumulation as a Single-Target Voltron
**Condition:** Kindle always targets the leftmost character. Cinder's repeated kindle (Infernal Ascendant: kindle 2 per abandon, Exiles: kindle 1 per abandon) concentrates spark on one body. After 3 abandons with Infernal Ascendant, your leftmost character has +6 spark. This is effectively a voltron (single-threat) strategy emerging from what looks like a sacrifice engine.
**Archetypes bridged:** Cinder (Ember+Ruin) <-> Crucible (Stone+Ember)
**Connector card concept:** "When your leftmost character gains spark from kindle, it gains that much spark again" (double kindle) or "Your leftmost character has 'Cannot be dissolved while another ally is in play'" (protection for the kindle target). The former turns Cinder's incidental kindle into a deliberate win condition; the latter ensures the investment survives.
**Non-obviousness: 9/10.** Kindle is treated as a small, distributed bonus in the current design. The realization that Ember's abandon loops create concentrated kindle on a single target is a hidden geometric growth pattern: each abandon both triggers kindle AND removes a non-leftmost body, ensuring the kindle target stays leftmost. The self-reinforcing nature of this loop is genuinely non-obvious.

### Vector 8: Sacrifice Bodies as Prevent Fuel (Herald Multiplication)
**Condition:** Herald of the Last Light (1 cost, Fast, "Abandon this: Prevent a played event") is the only card that converts a body into a counter. Cinder generates expendable bodies (Reclaim loops, cheap Survivors, figments from Packcaller). Each body is a potential counterspell.
**Archetypes bridged:** Gale (Zephyr+Ember) <-> Depths (Tide+Stone)
**Connector card concept:** "When you abandon an ally to prevent a card, draw 1" or "Sacrifice bodies gain 'Abandon this: Prevent a played card.'" The former is modest; the latter is transformative. If every Cinder sacrifice body doubles as a counterspell, you create a control-aggro hybrid that does not currently exist. Depths (the control archetype) uses Tide's Prevent effects; this connector would give Cinder its own Prevent suite fueled by expendable bodies rather than held cards.
**Non-obviousness: 9/10.** Prevent is Tide's territory. The idea that Ember's sacrifice bodies could function as counterspells through a Herald-like pattern inverts the resonance's identity without violating it: you are still spending resources aggressively (Ember's philosophy), but the resource being spent is a body, and the effect is denial.

### Vector 9: Removal as Storm Count (Tempest's Hidden Creature Strategy)
**Condition:** Tempest's storm count rewards "cards played this turn." Removal events (Immolate, Fell the Mighty, Scorched Reckoning, Momentum of the Fallen, Unleash Ruin) each increment the storm count while also interacting with the board. The side effect is that a Tempest deck can play a heavily interactive game (removing 2-3 threats) AND still build a lethal storm count.
**Archetypes bridged:** Tempest (Tide+Ember) <-> Depths (Tide+Stone) -- both share Tide
**Connector card concept:** "When you dissolve an enemy, draw 1 and gain 1 energy" -- a card that converts Ember's removal into Tempest fuel. Currently Momentum of the Fallen (dissolve + draw 1, costs 1 after a dissolution) is the closest, but it is a single card. A permanent or repeatable version would turn interactive Ember plays into storm enablers. This creates a "control storm" hybrid: answer threats while building toward a combo finish.
**Non-obviousness: 6/10.** The idea that removal IS storm fuel is implicit in Tempest's design (removal events add to storm count), but no card explicitly rewards the "removal storm" pattern as a distinct strategy.

### Vector 10: Opponent Board Depletion as a Win Condition
**Condition:** Ember's heavy removal suite (5+ dissolve effects in a typical Ember deck) can leave the opponent with zero characters. An opponent with no characters scores zero spark in Judgment. This is a de facto lockout that no card currently names or exploits.
**Archetypes bridged:** Gale (Zephyr+Ember) <-> Depths (Tide+Stone)
**Connector card concept:** "If the opponent controls no characters, gain 3 points" or "When you dissolve the opponent's last character, take an extra Judgment phase." This turns Ember's natural removal density into a distinct win condition: not just "remove blockers for spark advantage" but "remove everything and score from the empty board." The distinction matters because it rewards over-removal (currently wasteful) as a strategy.
**Non-obviousness: 4/10.** "Remove everything and win" is a fairly obvious control strategy. The non-obvious part is that Gale (a tempo deck, not a control deck) can achieve this state through fast-speed removal + aggressive deployment, creating a tempo-control hybrid that looks like Gale but plays like Depths.

### Vector 11: Discard-from-Hand as Void Setup for Bedrock
**Condition:** Ember's discard-as-cost effects (Fell the Mighty banishes from hand, Echoing Denial banishes from hand, Pyrokinetic Surge discards 1) can discard expensive characters from hand into void. This is exactly what Bedrock wants: expensive bodies in the void for reanimation.
**Archetypes bridged:** Tempest (Tide+Ember) <-> Bedrock (Stone+Ruin)
**Connector card concept:** The connector arguably already exists in "Entomb" (proposed new Bedrock card: "Put a character from your hand into your void. Draw 2."), but no Ember card specifically rewards discarding high-cost characters. A card like "Discard a character: Gain energy equal to its cost minus 2" would bridge Ember's discard-as-cost philosophy with Bedrock's void-setup needs. You discard a 6-cost Ancient to gain 4 energy (fueling Ember's burst turns) while setting up Bedrock's reanimation target.
**Non-obviousness: 6/10.** Bedrock already wants discard enablers; the insight is that Ember's existing discard-as-cost cards incidentally perform this function without any Ruin involvement. An Ember drafter who picks up Fell the Mighty is already half-building Bedrock's setup without realizing it.

### Vector 12: Figment Generation from Storm as Sacrifice Fodder
**Condition:** Packcaller of Shadows (Ember, Tempest core) generates figments equal to cards played this turn. Figments are 0-spark characters. In Cinder, these figments are free sacrifice bodies.
**Archetypes bridged:** Tempest (Tide+Ember) <-> Cinder (Ember+Ruin) -- both share Ember
**Connector card concept:** Already partially exists. Packcaller generates figments; Cinder abandons them. The missing link is a card that rewards abandoning figments specifically, or a figment generator that triggers on abandon rather than on card plays. "When you abandon a character with 0 spark, gain 1 energy and kindle 1" would make figments premium sacrifice fodder and bridge Tempest's token generation with Cinder's sacrifice loops.
**Non-obviousness: 7/10.** Figments are currently a Tempest board payoff (go wide with bodies for Judgment spark). Reframing them as Cinder sacrifice inputs is a complete inversion of their intended role. The two archetypes share Ember but want figments for opposite reasons: Tempest wants them alive (spark at Judgment), Cinder wants them dead (trigger abandon payoffs).

---

## 4. Anti-Synergy Awareness: What Ember Does NOT Do

These are areas where a proposed bridge card would violate Ember's identity, even if the mechanical synergy appears sound.

### Anti-Synergy 1: Ember Does Not Recur
Bridge cards that give Ember recursion (e.g., "When you dissolve an enemy, Reclaim a character") violate the core principle. Ember sends things to the void and does not bring them back. Recursion is Ruin's territory. Any "Ember recursion" card should instead be dual Ember+Ruin (Cinder signpost) or pure Ruin.

### Anti-Synergy 2: Ember Does Not Accumulate Card Advantage
A connector card like "When you dissolve an enemy, draw 2" would violate Ember's identity. Momentum of the Fallen (dissolve + draw 1) is at the absolute boundary. Ember's card draw is always incidental (1 card) and never the primary purpose of the effect. A synergy that requires net-positive draw should route through Tide, not Ember.

### Anti-Synergy 3: Ember Does Not Build Persistent Engines
Cards like "At the start of each turn, dissolve the weakest enemy" would violate Ember's burst philosophy. Ember's effects are one-shot detonations, not repeating processes. Persistent engines that generate value over many turns belong to Stone (Judgment triggers) or Ruin (recurring loops). The distinction: Cinder's sacrifice loop LOOKS persistent, but the persistence comes from Ruin's Reclaim, not from Ember. Remove the Ruin half and Ember burns through its board in 2-3 turns.

### Anti-Synergy 4: Ember Does Not Protect
Cards that give Ember defensive tools (shields, damage prevention, indestructibility for allies) violate the expendable philosophy. Ember characters are meant to burn bright and die. A "protect your kindle target" card might be mechanically logical (Vector 7), but it should be Stone-coded (board durability is Stone's territory), not Ember-coded.

### Anti-Synergy 5: Ember Does Not Manipulate the Deck
Foresee, Discover (generic), library ordering, and top-of-deck manipulation are Tide and Zephyr tools. Any connector card involving deck manipulation should route through the non-Ember resonance. For example, a Tempest card that Foresees while generating energy should be Tide-coded (Foresee half) + Ember-coded (energy half) as a dual card, not pure Ember.

### Anti-Synergy 6: Ember Does Not Benefit from Patience
A card that says "If you have not played a card this turn, gain 5 energy" is anti-Ember. Ember's burst energy comes from SPENDING resources, not from waiting. Any conditional energy generation should trigger on aggressive actions (playing events, abandoning allies, dissolving enemies), never on restraint.

---

## 5. Priority Recommendations

The highest-impact synergy vectors for new card design, ranked by (a) non-obviousness, (b) number of archetypes bridged, and (c) alignment with Ember's identity:

1. **Vector 7 -- Kindle Concentration as Voltron** (9/10 non-obvious). Bridges Cinder and Crucible. Requires no identity violation. Exploits a mathematical inevitability of Ember's existing mechanics that no player is currently building around.

2. **Vector 6 -- Abandon-Reclaim as Pseudo-Flicker** (8/10 non-obvious). Bridges Cinder and Mirage. Unlocks an entirely new cross-resonance relationship between Ember and Zephyr through the shared condition of "frequent materializations."

3. **Vector 8 -- Sacrifice Bodies as Prevent Fuel** (9/10 non-obvious). Bridges Gale and Depths. Creates a genuinely novel play pattern (sacrifice-as-counterspell) that respects Ember's "spend resources aggressively" philosophy while producing a defensive outcome.

4. **Vector 3 -- Hellbent Payoffs** (8/10 non-obvious). Bridges Gale and Eclipse. Exploits an inevitable side effect that currently has zero payoff cards. Clean resonance separation (anti-Tide).

5. **Vector 1 -- Void Character Density** (7/10 non-obvious). Bridges Cinder and Bedrock. Differentiates Cinder's void (character-dense) from Undertow's void (mixed) and creates a new axis for void-matters design.

---

## 6. Summary Table

| # | Vector | Side Effect | Archetypes Bridged | Non-Obviousness |
|---|--------|-------------|-------------------|-----------------|
| 1 | Void Character Density | Abandon fills void with characters | Cinder <-> Bedrock | 7/10 |
| 2 | Dissolution Chain Cascade | Warrior dissolution events | Crucible <-> Undertow | 6/10 |
| 3 | Hellbent Payoffs | Hand depletion from discard-as-cost | Gale <-> Eclipse | 8/10 |
| 4 | Event Void Density | Events in void from storm/removal | Tempest <-> Undertow | 5/10 |
| 5 | Burst Energy into Activated Abilities | Off-curve energy for abilities | Crucible <-> Basalt | 7/10 |
| 6 | Abandon as Pseudo-Flicker | Materialize triggers from Reclaim loops | Cinder <-> Mirage | 8/10 |
| 7 | Kindle Concentration Voltron | Single-target spark growth | Cinder <-> Crucible | 9/10 |
| 8 | Sacrifice Bodies as Prevent | Expendable bodies as counterspells | Gale <-> Depths | 9/10 |
| 9 | Removal as Storm Count | Interactive play building combo count | Tempest <-> Depths | 6/10 |
| 10 | Opponent Board Depletion | Empty opponent board as condition | Gale <-> Depths | 4/10 |
| 11 | Discard-as-Void-Setup | Hand discard placing reanimation targets | Tempest <-> Bedrock | 6/10 |
| 12 | Figments as Sacrifice Fodder | Storm tokens as abandon inputs | Tempest <-> Cinder | 7/10 |
