# Ruin Synergy Discovery Analysis (v2)

## Purpose

This document does not describe what Ruin IS -- that work is done. Instead, it maps what Ruin COULD synergize with if the right connector cards existed. The goal is to find hidden synergy vectors: game-state conditions that Ruin's mechanics create as side effects but that no current card exploits.

---

## 1. Side Effect Inventory

Every mechanic has downstream consequences beyond its intended purpose. Here is a catalog of the game-state conditions that Ruin creates incidentally.

### 1a. Repeated Materialized Triggers via Reclaim

**Intended:** Reclaim lets you replay cards from the void at reduced cost.
**Side effect:** Every Reclaimed character triggers its Materialized ability again. A Reclaimed Nocturne draws and discards again. A Reclaimed Reclaimer of Lost Paths grants another Reclaim 0. This means Ruin is secretly a second source of Materialized trigger volume, alongside Zephyr's flicker. The difference is that Ruin's re-materializations pass through the void (filling it, triggering Dissolved on the way out) while Zephyr's pass through banish.

**Currently exploited?** No. No Ruin card explicitly rewards "number of times you have materialized a character this turn" or "when you materialize a character from your void." The materialize-matters cluster (Angel of the Eclipse, The Bondweaver, Lumin-Gate Seer, Bloomweaver) is Zephyr-coded and primarily serves Mirage/Basalt. Ruin archetypes generate Materialized triggers but have no payoffs that specifically reward them.

### 1b. Opponent Benefits from Your Dissolved Triggers

**Intended:** Dissolved triggers punish the opponent for removing your board.
**Side effect:** They also fire when YOU sacrifice your own characters (Cinder), when combat kills them, or when board wipes hit. More subtly, if the opponent is also running Ruin (or Cinder), your removal of THEIR creatures fires THEIR Dissolved triggers. This creates a feedback loop: in Ruin mirrors, every removal spell benefits the defender. The game-state condition is that Dissolved triggers are symmetric in who can cause them -- only the "allied" qualifier on the trigger limits exploitation.

**Currently exploited?** Partially. Cinder deliberately engineers its own deaths. But no card exists that triggers on an ENEMY character being dissolved (from your perspective), which would let you profit from your own removal. Eclipse Herald banishes from your void to dissolve enemies -- it cares about your void, not the enemy's death. The enemy-death vector is untapped.

### 1c. Void Size Is Monotonically Increasing (Absent Hate)

**Intended:** Void-size-matters cards (Abomination of Memory, Weight of Memory, Architect of Memory) scale with void count.
**Side effect:** Barring void hate (Soulflame Predator, Spirit Field Reclaimer, Wheel of the Heavens, Ruin Scavenger), the void only grows. Every card that is played, discarded, milled, or sacrificed adds to it. Even non-Ruin decks have 5-10 void cards by midgame. This means void-size-matters effects have a guaranteed late-game power floor even without active void-filling. The game-state condition is: every deck's void is large by turn 6-7 regardless of strategy.

**Currently exploited?** Only by dedicated Undertow builds. No card rewards the opponent's void size, and no card rewards the combined void size of both players. No card converts a large void into energy, fast deployment, or card selection beyond the existing three void-size-matters cards.

### 1d. Survivors Are the Most Numerous Subtype

**Intended:** Survivor tribal provides Ruin's creature identity.
**Side effect:** With 23 members, Survivors are the largest subtype (vs. 21 Warriors, 18 Ancients, 18 Spirit Animals). This means Survivor-incidental effects have the widest targeting range. Cards that say "Discover a Survivor" or "when you materialize a Survivor" have the most potential hits. Yet Survivor tribal payoffs are concentrated in a single archetype (Undertow), unlike Warriors (Crucible core + Cinder/Bedrock bridges) or Spirit Animals (Basalt core + Mirage bridge). Survivors are underexploited outside Undertow.

**Currently exploited?** Only by Undertow. Cinder runs some Survivors (Dustborn Veteran, Exiles of the Last Light, Resilient Wanderer) but for their individual abilities, not tribal synergy. Eclipse runs Wasteland Arbitrator and Emberwatch Veteran as Survivors, but again, not tribally. Bedrock has almost no Survivor density.

### 1e. Self-Mill Creates Information Asymmetry

**Intended:** Self-mill fills the void for Ruin payoffs.
**Side effect:** When you mill cards, you SEE what went into the void. You know whether your Soulkindler, your Revenant of the Lost, or your key Reclaim target is now accessible. Your opponent does not have this information (they see count, not specific cards). This creates a hidden information advantage: the Ruin player can make Reclaim decisions with perfect void knowledge while the opponent guesses. The game-state condition is: Ruin players consistently have better information about their own void contents than opponents.

**Currently exploited?** No. No card rewards void information directly. There is no "choose a card in your void and reveal it" effect, no "name a card in your void" effect, and no decision tree that rewards void-content knowledge beyond the implicit Reclaim choices. A card that said "Discover a card in your void" would monetize this information advantage.

### 1f. Reclaim Creates Known Top-of-Deck Information

**Intended:** Reclaim returns cards to hand from the void.
**Side effect:** Actually, Reclaim in Dreamtides gives cards in the void the Reclaim keyword, which lets you play them from the void (paying their Reclaim cost). This means Reclaim effectively gives you a second hand -- the void becomes a publicly visible hand extension. The side effect is that the opponent can see your Reclaim options (the void is public information in most card games), but you have MORE playable options than your hand alone provides. The game-state condition is: Ruin players reliably have more total playable cards (hand + Reclaimed void cards) than other resonances, but those extra options are visible to the opponent.

**Currently exploited?** The Reclaim infrastructure is core Ruin. But no card punishes or rewards the total count of playable cards across hand + void. No card says "if you have 8+ playable cards (hand + Reclaimed void), gain X."

### 1g. Sacrifice Loops Create Repeated Enter/Leave Events

**Intended:** Cinder sacrifices and recurs creatures for abandon payoffs and Dissolved triggers.
**Side effect:** Each sacrifice-and-return cycle means a character enters play, leaves play, enters the void, and returns from the void. This generates FOUR zone-transition events per cycle (materialize, dissolve, enter-void, leave-void). Only two of these transitions are currently tracked by mechanics (Materialized, Dissolved). "Enter void" and "leave void" are not distinct trigger conditions on any current card. The game-state condition is: Cinder generates 2x more zone transitions than it currently exploits.

### 1h. Ruin Decks Tend to Have Low Spark Boards That Grow Over Time

**Intended:** Survivors have low spark (many 1-spark bodies) and use kindle/tribal buffs to grow.
**Side effect:** Ruin's boards look weak early and strong late. In the midgame, a Ruin board of four 1-spark Survivors appears non-threatening. But with Soulkindler in void (+2 to each) and a kindle or two, that board is suddenly 12+ spark. The game-state condition is: Ruin boards have high spark VARIANCE over time, starting low and spiking. No current card explicitly rewards having many low-spark characters (which would synergize with Ruin's early board state) or rewards the delta between current spark and previous spark.

---

## 2. Cross-Resonance Bridge Opportunities

These are existing cards in OTHER resonances that would become surprisingly powerful in Ruin archetypes with the right connector.

### 2a. Conduit of Resonance (Stone/Zephyr -- Basalt core)

"When you materialize a character, trigger the Judgment ability of each ally."

**Surprise synergy:** If Ruin is materializing characters through Reclaim, each Reclaim-materialize fires all Judgment abilities. In an Undertow deck with Hope's Vanguard ("Judgment: With 2+ Survivors, draw 1"), materializing a single Survivor via Reclaim would draw a card from Hope's Vanguard. With Scrap Reclaimer ("Judgment: return from void to hand") on board, materializing anything triggers Scrap Reclaimer's return, which fills the hand. The connector card needed: a dual Stone+Ruin body that either has Reclaim itself or rewards Judgment triggers from void-based plays.

**Bridges:** Bedrock (Stone+Ruin) already has access to both resonances. This synergy could strengthen Bedrock by making Stone's Judgment infrastructure relevant to Ruin's recursion.

### 2b. Celestial Reverie (Tide -- Tempest/Mirage)

"Until end of turn, when you play a character, draw 1."

**Surprise synergy:** In a Cinder deck that sacrifices and Reclaims 3-4 characters per turn, each re-materialization draws a card. The connector: a cheap Ruin event that gives multiple void characters Reclaim on the same turn (Path to Redemption does this at 6 energy, but a cheaper version at 2-3 energy for "up to 2 characters" would make the combo consistent).

**Bridges:** Tempest (Tide+Ember) to Cinder (Ember+Ruin). The Ember overlap means a Cinder deck could splash Celestial Reverie through its Ember half accessing Tide-adjacent cards.

### 2c. Starlit Cascade (Zephyr -- Mirage)

"Until end of turn, when an ally leaves play, gain 2 energy."

**Already identified** as a Cinder bridge card (Strong card in Cinder allocation). But the deeper synergy is with Bedrock: when The Rising God ("Abandon 2: Reclaim") fires, abandoning two allies generates 4 energy from Starlit Cascade, which can partially fund the next Reclaim cost. The connector needed: a way for Bedrock to access Zephyr cards. Currently, Stone and Zephyr share Basalt, so a Bedrock drafter has no natural path to Starlit Cascade. A neutral or Ruin-coded version of this effect ("when an ally enters your void, gain 1 energy") would serve Cinder and Bedrock without requiring Zephyr access.

### 2d. Blazing Emberwing (Ember/Zephyr -- Basalt)

"The Judgment ability of allies triggers when you materialize them."

**Surprise synergy:** In Cinder, if you have Volcanic Channeler ("When ally dissolved: gain 1 energy") and Sunset Chronicler ("When ally dissolved: draw 1"), these are "when dissolved" triggers, not Judgment triggers, so Blazing Emberwing does not apply. BUT if you have Spirit of the Greenwood ("Judgment: Gain 1 energy per ally") or other Judgment-energy characters, each Reclaim-materialize in a Cinder deck fires them. The deeper point: Ruin's repeated materializations could exploit Judgment triggers if Ruin archetypes had access to Judgment-trigger characters.

### 2e. Looming Oracle / Frost Visionary (Tide -- Mirage)

"Materialized: Draw 1."

**Surprise synergy:** These are premium flicker targets in Mirage, but they would also be premium Reclaim targets in Undertow or Bedrock. Every time you Reclaim them, you draw a card. Frost Visionary is a Warrior (Crucible bridge) and Looming Oracle is a Spirit Animal (Basalt). Neither has a natural path to Ruin. A Survivor with "Materialized: Draw 1" would be a natural Undertow card, and such a card does not currently exist in Ruin's pool -- Hope's Vanguard requires 2+ Survivors for its conditional draw, Seer of the Fallen triggers on Dissolved not Materialized. Connector needed: a cheap Survivor with unconditional Materialized draw.

### 2f. The Bondweaver (Zephyr -- Mirage/Basalt)

"When you materialize a character, this character gains +1 spark."

**Surprise synergy:** In Cinder, materializing 3-4 characters per turn through Reclaim would grow The Bondweaver by 3-4 spark. But The Bondweaver is Zephyr-coded (no Ruin access). A Ruin-coded version -- "when a character enters play from your void, this character gains +1 spark" -- would be a Cinder/Undertow payoff for recursion volume. This is distinct from Harvester of Despair ("when you abandon, +1 spark") because it rewards the RETURN half of the loop, not the sacrifice half.

---

## 3. Hidden Synergy Vectors

### Vector 1: "From Void" Materialized Distinction

**Condition:** Characters materialized from the void via Reclaim are mechanically identical to characters played from hand, but their origin is different. No current card distinguishes "materialized from void" from "materialized from hand."

**Archetypes bridged:** Bedrock (expensive targets from void) + Cinder (cheap recursive bodies from void) + Undertow (Survivor recursion from void).

**New card concept:** "When you materialize a character from your void, [effect]." This could be kindle, draw, energy gain, or points. It would reward all Ruin archetypes that use Reclaim but in proportion to their recursion volume -- Cinder generates the most re-materializations per turn, Bedrock generates the highest-value individual ones, Undertow falls in between.

**Non-obviousness: 7/10.** The distinction between "from hand" and "from void" materialization seems obvious once stated, but no existing card tracks it, and the entire materialize-matters cluster (8 cards) ignores origin zone. This vector would give Ruin its own materialize-matters subtheme that does not compete with Zephyr's.

### Vector 2: Void Velocity (Cards Entering Void Per Turn)

**Condition:** Different Ruin archetypes fill the void at different rates. Undertow mills 4-8 cards per turn into the void. Eclipse discards 2-4 per turn. Cinder sacrifices 2-3 per turn. Bedrock places 1-2 deliberately. The RATE of void growth varies, but no card cares about velocity -- only the existing three cards care about total size.

**Archetypes bridged:** Primarily differentiates Undertow (high velocity) from Bedrock (low velocity). A velocity-matters card would be strong in Undertow and weak in Bedrock, creating natural archetype separation.

**New card concept:** "At end of turn, if 3+ cards entered your void this turn, [effect]." Could be draw, kindle, gain energy, or gain points. The threshold of 3 is reachable by Undertow (mill), Eclipse (draw-discard cycles), and aggressive Cinder (multi-sacrifice turns) but not by Bedrock (1-2 deliberate placements).

**Non-obviousness: 8/10.** Void velocity is an emergent game-state property that no existing mechanic tracks. It is the "storm count" of Ruin -- a hidden counter that rewards high-throughput void strategies.

### Vector 3: Void Composition (Character vs. Event Ratio)

**Condition:** The void accumulates both characters and events, but in different ratios depending on the deck. Cinder's void is character-heavy (sacrificed bodies). Eclipse's void is mixed (discarded hand cards). Undertow's void is random (milled). Tempest's splash-accessible void might be event-heavy (cheap events played and resolved). No card currently differentiates void composition.

**Archetypes bridged:** Could differentiate Eclipse (mixed) from Cinder (character-heavy) further. A "number of characters in your void" payoff would specifically reward Cinder and Bedrock (which deposit expensive characters), while "number of events in your void" would reward Tempest-Ruin hybrids.

**New card concept:** "This character's spark equals the number of character cards in your void" (Cinder/Bedrock payoff) or "Gain 1 energy for each event in your void" (Eclipse/Tempest payoff). These would be more targeted than Abomination of Memory's blanket void count.

**Non-obviousness: 6/10.** Card type distinction in the void is a natural extension of Abomination of Memory, but the design implication -- creating archetype-specific void payoffs rather than universal ones -- is less obvious.

### Vector 4: Death Echo Chains (Dissolved Triggers Triggering Each Other)

**Condition:** When a character with a Dissolved trigger is dissolved, its trigger fires. If that trigger creates a new character that also has a Dissolved trigger (e.g., Twilight Reclaimer grants Reclaim to a Survivor, that Survivor is Reclaimed and later dissolved, firing its own trigger), you get a chain of death echoes across turns. The game-state condition is: boards with multiple Dissolved-trigger characters create exponentially more value per death event.

**Archetypes bridged:** Cinder (engineers deaths) + Undertow (has the most Dissolved-trigger Survivors). A Cinder deck that splashes Undertow's Survivor Dissolved triggers would get compound value per sacrifice.

**New card concept:** "When a Dissolved trigger fires, [additional effect]." This is a meta-trigger -- a trigger on triggers. Example: "Whenever a Dissolved ability triggers, kindle 1." This rewards boards with high Dissolved trigger density. It is strongest in Cinder (many deaths) with Undertow's Survivor bodies (many triggers per body).

**Non-obviousness: 8/10.** Meta-triggers (triggers on triggers) are unusual in card game design. The concept of "death echo amplification" is invisible unless you model the board state mathematically.

### Vector 5: Void as a Shared Resource Between Players

**Condition:** Several Ruin cards interact with the opponent's void (Soulflame Predator banishes it, Spirit Field Reclaimer banishes from it, Ruin Scavenger banishes and gains energy, Eclipse Herald banishes from your own void). But no current card BENEFITS from the opponent having a large void. In a Ruin mirror, both players' voids are enormous -- a card that rewards combined void size would be asymmetrically powerful in these matchups.

**Archetypes bridged:** All Ruin archetypes in mirror matches. More interestingly, this creates an anti-synergy against non-Ruin opponents (who have smaller voids) and a synergy in Ruin-heavy metagames.

**New card concept:** "Dissolve an enemy with cost less than the total number of cards in both players' voids." A scaled-up Weight of Memory that counts all void cards everywhere. Or: "Gain 1 point for each card in the opponent's void" as a one-shot burst.

**Non-obviousness: 7/10.** Opponent's void as a resource is counterintuitive (Ruin focuses inward on its own void). But in a format where self-mill and sacrifice are common, opponent voids are large by default.

### Vector 6: Kindle Accumulation as a Late-Game Win Condition

**Condition:** Kindle adds spark to the leftmost character. Ruin, through Cinder (Infernal Ascendant, Exiles of the Last Light) and Undertow (Silent Avenger), generates significant kindle over a game. The side effect is that Ruin's leftmost character accumulates enormous spark -- potentially 8-12+ spark on a single body by late game. No card currently rewards having a single character with very high spark. The game-state condition is: Ruin often creates one "tower" character with disproportionate spark.

**Archetypes bridged:** Cinder (active kindle through sacrifice) + Undertow (passive kindle through Silent Avenger). A card that rewards high single-character spark would serve both archetypes.

**New card concept:** "Abandon a character: Gain points equal to that character's spark." This turns the kindle tower into a burst-point finisher. Already partially exists (Fathomless Maw gives 1 point per abandon, not scaling with spark; Shardwoven Tyrant uses spark for removal). A direct spark-to-points conversion would close the loop.

**Non-obviousness: 5/10.** Kindle accumulation is visible to players. But the strategic insight -- that Ruin is the resonance most likely to create a single high-spark character, and that this could be a dedicated win condition -- is under-recognized because kindle is treated as incremental rather than as a buildup mechanic.

### Vector 7: Reclaim Cost Manipulation

**Condition:** Reclaim costs vary: Reclaim 0 (free), Reclaim = cost (pay full), Reclaim 3 (fixed discount), etc. Stone's energy ramp (Nexus Wayfinder: "Characters cost 2 less") applies to characters played from the void if they have Reclaim. This means Stone's cost reduction double-dips: it reduces both the hand-play cost and the Reclaim cost. The game-state condition is: energy cost reduction is disproportionately powerful in Ruin because it applies to more playable cards (hand + void).

**Archetypes bridged:** Bedrock (Stone+Ruin) specifically. But also any Ruin deck that splashes Stone ramp.

**New card concept:** "Characters you play from your void cost 2 less." A targeted cost reducer for void plays only. This would be Bedrock-exclusive in practice (the only archetype with both expensive targets and void deployment) and would not help Undertow (whose Survivors are already cheap) or Eclipse (which cycles cheap cards). Distinct from Nexus Wayfinder because it only applies to void plays, making it less universally powerful.

**Non-obviousness: 6/10.** Cost reduction for void plays is a natural design space, but the realization that generic cost reduction already does this (and that Bedrock therefore wants Stone ramp cards more than it initially appears) is a hidden synergy within existing cards.

### Vector 8: Sacrifice Fodder as a Cross-Archetype Resource

**Condition:** Cinder needs sacrifice bodies. Figment tokens (from Packcaller of Shadows, Radiant Trio, Endless Projection) are ideal sacrifice fodder -- they are free bodies that enter play and can be immediately abandoned. But figment generators are Zephyr-coded (Mirage/Basalt territory). No Ruin or Ember card currently generates figments.

**Archetypes bridged:** Cinder (needs fodder) + Mirage/Basalt (generates figments). A Ruin-coded figment generator would give Cinder free sacrifice bodies without requiring Zephyr access.

**New card concept:** "When a character enters your void, materialize a figment." This turns every death, discard, or mill into a free body. In Cinder, sacrificing a character creates a figment, which can be immediately sacrificed, creating a chain. In Undertow, milling characters creates figments for board presence. This might be too powerful without restrictions (e.g., "once per turn" or "the first time each turn a character enters your void").

**Non-obviousness: 7/10.** Figments are currently a Zephyr/neutral mechanic. Connecting them to void entry would recontextualize figments as a Ruin mechanic in a surprising way.

### Vector 9: Event Reclaim as a Tempo Tool

**Condition:** Most Reclaim discussion focuses on characters (Reclaimer of Lost Paths, Path to Redemption, Architect of Memory). But events can also have Reclaim (Echoes of Eternity has Reclaim 2, Passage Through Oblivion has Reclaim 1, Guiding Light has Reclaim 3). Ashlight Caller gives event Reclaim specifically. The game-state condition is: events with Reclaim are replayable cheap spells, effectively giving Ruin a "spellslinger" sub-mode where cheap events cycle between void and play.

**Archetypes bridged:** Eclipse (cycles cheap cards) + Tempest (plays many events). A card that rewards playing events from the void would bridge Ruin's Reclaim infrastructure with Tempest's event-chain strategy.

**New card concept:** "When you play an event from your void, copy it." This turns Reclaimed events into double-value plays. It would be strongest in a Tempest deck that splashes Ruin for event Reclaim (via Ashlight Caller + Whisper of the Past), or in Eclipse (where Ashmaze Guide gives discarded events Reclaim). Ties into a potential "Ruin spellslinger" hybrid that does not currently exist.

**Non-obviousness: 8/10.** Event Reclaim exists in the card pool but is treated as utility, not as an engine. The idea that Ruin could enable a spellslinger variant through event recursion (rather than through Tide's draw and Ember's energy burst) is a genuinely novel archetype direction.

### Vector 10: Board Wipe Asymmetry Through Dissolved Triggers

**Condition:** Apocalypse ("Dissolve all characters") is classified as a Cinder flex card. When it fires in a Cinder deck with Avatar of Cosmic Reckoning, Volcanic Channeler, Sunset Chronicler, and Dustborn Veteran on board, each ally being dissolved triggers all "when ally dissolved" effects. If you have 4 allies and 4 Dissolved-trigger characters, you get up to 16 trigger events (each of 4 deaths fires each of 4 triggers). The opponent gets nothing because their triggers (if any) fire for their own allies, not yours.

**Archetypes bridged:** Cinder (sacrifice) + Depths (control, which wants board wipes). A control deck splashing Ruin's Dissolved triggers could use board wipes offensively while recovering through death triggers.

**New card concept:** Not a card per se, but a recognition that Apocalypse + Dissolved trigger density is an underexploited Cinder finisher package. A cheaper board wipe (3-4 cost, "Dissolve all characters with spark 2 or less") would more precisely target Cinder's low-spark recursive bodies while leaving the opponent's high-spark threats intact -- then the Dissolved triggers generate massive value.

**Non-obviousness: 5/10.** Board wipe + death triggers is a known MTG strategy (aristocrats + Wrath). But the specific math of 4x4 trigger multiplication in Dreamtides is more explosive than casual reading suggests.

### Vector 11: Void-Gated Alternate Play Patterns

**Condition:** Revenant of the Lost ("Only playable from void") establishes a template: characters that can ONLY be played from the void, at a discount. Currently, Revenant is the only example (3 cost for 6 spark, void-only). The game-state condition is: void-only characters become "free" to dump into the void (no opportunity cost since you cannot play them from hand anyway) and reward void-filling infrastructure disproportionately.

**Archetypes bridged:** Bedrock (primary, wants to deploy from void) + Undertow (mills them into void naturally) + Eclipse (discards them, and Ashmaze Guide gives them Reclaim). A suite of 2-3 void-only characters at different costs would create a "void aggro" sub-strategy across all Ruin archetypes.

**New card concept:** "2 cost, 4 spark. Only playable from void. Dissolved: Return to void." A smaller Revenant that self-recurs to the void on death, making it permanently available as a void resource without needing Reclaim infrastructure. This would serve Cinder (cheap sacrifice body that always returns to void), Undertow (strong Survivor body to mill), and Bedrock (additional void-only threat) differently.

**Non-obviousness: 7/10.** Revenant of the Lost is already a powerful card, but the design space of void-only characters as a category (with varying costs, sparks, and abilities) is underexplored. The concept that "hand-unplayable" is actually an advantage in Ruin (because it eliminates the tension between playing from hand vs. filling the void) is counterintuitive.

### Vector 12: Survivor Spread as a Cross-Archetype Draft Incentive

**Condition:** 23 Survivors exist, but Survivor tribal payoffs are concentrated in Undertow. Many Survivors have abilities that serve other archetypes: Emberwatch Veteran (Eclipse), Wasteland Arbitrator (Gale/Eclipse), Ashlight Caller (Tempest), Resilient Wanderer (Cinder), Dustborn Veteran (Cinder/Undertow). The game-state condition is: non-Undertow archetypes often end up with 2-4 Survivors incidentally, but have no reason to care about that tribal density.

**Archetypes bridged:** Eclipse, Cinder, and even Gale could benefit from light Survivor tribal rewards if a payoff existed at low density thresholds (e.g., "with 1+ Survivor" rather than Undertow's "with 2+ Survivors"). This would make Survivor subtype matter in non-tribal contexts.

**New card concept:** "Once per turn, when a Survivor enters your void, [small effect]." This would trigger in Eclipse (Wasteland Arbitrator discarded), Cinder (Resilient Wanderer sacrificed), Undertow (Survivors milled), and even Gale (Wasteland Arbitrator sacrificed for tempo). It is a "Survivor death" payoff at a lower commitment level than Undertow's tribal density requirements.

**Non-obviousness: 6/10.** The observation that Survivors are spread across archetypes is in the existing documentation. The design implication -- that a low-threshold Survivor payoff could unify incidental Survivors across archetypes -- is the non-obvious part.

---

## 4. Anti-Synergy Awareness: What Ruin Does NOT Do

### 4a. Ruin Does Not Generate Energy

Ruin cheats costs through the void, but it never directly produces energy. Energy generation belongs to Stone (Judgment triggers), Ember (burst events), and Zephyr (materialize-matters). A bridge card that said "when a card enters your void, gain 1 energy" would violate this boundary by giving Ruin native energy ramp, which would reduce its dependence on Stone (undermining Bedrock's identity) or Ember (undermining Cinder's energy from Spirit Reaping). Exception: Volcanic Channeler ("when ally dissolved, gain 1 energy") exists as a Ruin-coded energy source, but it is conditional and tied to death events, not void-filling. Keep energy tied to death events, not void growth.

**Boundary:** Bridge cards can convert DEATH into energy (Cinder territory) but should not convert VOID GROWTH into energy.

### 4b. Ruin Does Not Interact with the Opponent's Hand or Deck

Ruin has no hand disruption (Break the Veil, Lurking Dread are Tide), no Prevent effects (Tide), and no deck manipulation of the opponent. Ruin's interaction is entirely through board state (Dissolved triggers, Weight of Memory as removal) and void (Eclipse Herald banishing own void to remove enemies). A bridge card that said "discard a card from the opponent's hand for each card in your void" would violate Ruin's non-reactive identity by giving it proactive disruption.

**Boundary:** Ruin can RESPOND to the opponent's actions (Dissolved triggers fire when opponent removes your creatures) but should not PREEMPT them. Disruption and denial are Tide's territory.

### 4c. Ruin Does Not Do Fast-Speed Recursion

No Ruin card has Fast. Ashlight Caller and Eternal Sentry are Fast characters, but their fastness comes from their Survivor/Ruin typing, not from void interaction. Reclaim inherently operates at sorcery speed (you play cards from void during your main phase). A bridge card that said "Flash -- Reclaim a character during the opponent's turn" would violate this by giving Ruin reactive recursion, which would step on Zephyr's flash identity and Tide's reactive timing.

**Boundary:** Ruin operates on its own turn. Its inevitability comes from grinding advantage over many turns, not from responding at instant speed. Fast Ruin cards should be enablers (Ashlight Caller) not recursion pieces.

### 4d. Ruin Does Not Protect Its Board

Ruin has no damage prevention, no shields, no hexproof-equivalent, no "cannot be dissolved" effects. Ruin's board resilience comes from accepting death and profiting from it (Dissolved triggers) or returning from death (Reclaim), never from preventing death. A bridge card that said "Survivors cannot be dissolved" would violate Ruin's fundamental identity -- Ruin WANTS things to die and come back.

**Boundary:** Never give Ruin protection. Ruin's defense is recursion, not prevention. "Indestructible Survivor" is an oxymoron in Ruin's philosophy.

### 4e. Ruin Does Not Care About Board Width (Number of Allies)

Stone cares about board width (Warrior tribal scaling). Zephyr cares about materializations (which correlate with board width). Ruin does not have "for each ally" effects -- its scaling is void-based, not board-based. Spirit of the Greenwood ("Judgment: Gain 1 energy per allied character") is Stone-coded. A Ruin card that said "gain 1 point per ally" would be Stone territory in Ruin clothing.

**Boundary:** Ruin's scaling axis is VOID SIZE and DEATH COUNT, not board width. A Ruin card can reward deaths-this-turn (Dissolved trigger density) but should not reward allies-in-play (board width).

---

## 5. Archetype Differentiation via Synergy Vectors

The critical test for any new synergy card: does it help differentiate Ruin's four archetypes, or does it make them compete for the same slot?

| Vector | Undertow | Eclipse | Cinder | Bedrock | Differentiation |
|--------|----------|---------|--------|---------|----------------|
| V1: From-void materialized | Medium (Survivors Reclaimed) | Low (cycles cheap cards) | High (repeated re-materialization) | High (expensive single targets) | Good -- Cinder/Bedrock want it most but for different reasons |
| V2: Void velocity | High (mills 4-8/turn) | Medium (discards 2-4/turn) | Medium (sacrifices 2-3/turn) | Low (places 1-2/turn) | Excellent -- naturally separates Undertow from Bedrock |
| V3: Void composition | Medium (random) | Medium (mixed) | High (character-heavy) | High (character-heavy) | Moderate -- Cinder and Bedrock overlap here |
| V4: Death echo chains | Medium (passive deaths) | Low (no sacrifice) | High (engineered deaths) | Low (few deaths) | Excellent -- purely Cinder |
| V5: Opponent's void | Medium | Low | Medium | Low | Weak -- meta-dependent, not archetype-specific |
| V6: Kindle accumulation | Medium (Silent Avenger) | Low | High (Infernal Ascendant) | Low | Good -- Cinder primary |
| V7: Reclaim cost reduction | Low (cheap Survivors) | Low (cheap cycles) | Low (cheap bodies) | High (expensive targets) | Excellent -- purely Bedrock |
| V8: Figment from void entry | Medium (mill creates figments) | Medium (discard creates figments) | High (sacrifice creates figments) | Low (low volume) | Good -- Cinder primary, others secondary |
| V9: Event Reclaim engine | Medium (Ashlight Caller) | High (Ashmaze Guide + events) | Low (character-focused) | Low | Excellent -- Eclipse primary |
| V10: Board wipe asymmetry | Low | Low | High (Dissolved density) | Low | Excellent -- purely Cinder |
| V11: Void-only characters | Medium (mills them) | Medium (discards them) | Medium (sacrifices them) | High (deploys them) | Good -- Bedrock primary, all participate |
| V12: Survivor spread | Medium (tribal) | Medium (incidental) | Medium (incidental) | Low | Moderate -- bridges non-Undertow archetypes |

### Recommended Priority Vectors for New Card Design

**Highest priority (excellent differentiation):**
- V2 (Void velocity) -- Undertow card that Bedrock does not want
- V4 (Death echo chains) -- Cinder card that Undertow does not want
- V7 (Reclaim cost reduction) -- Bedrock card that Eclipse does not want
- V9 (Event Reclaim engine) -- Eclipse card that Cinder does not want

**Medium priority (good differentiation):**
- V1 (From-void materialized) -- Shared Cinder/Bedrock with different use patterns
- V8 (Figment from void entry) -- Cinder primary with splash potential
- V11 (Void-only characters) -- Bedrock primary, universal utility

**Lower priority (less differentiation or niche):**
- V5 (Opponent's void) -- Too meta-dependent
- V10 (Board wipe asymmetry) -- Already exists implicitly, just needs recognition
- V12 (Survivor spread) -- Moderate value, potentially dilutes Undertow's tribal identity

---

## 6. Summary of Key Findings

1. **Ruin's biggest unexploited side effect is repeated Materialized triggers from Reclaim.** The entire materialize-matters cluster (8 cards) is Zephyr-coded. A "from-void materialized" payoff (Vector 1) would give Ruin its own version of this mechanic.

2. **Void velocity (Vector 2) is the hidden differentiator between Ruin archetypes.** Undertow fills the void fast, Bedrock fills it slowly and deliberately. A velocity-matters card would reward Undertow specifically without helping Bedrock, solving the "both want void cards" competition problem.

3. **Event Reclaim (Vector 9) is Eclipse's most underexploited synergy.** Eclipse is identified as the thinnest Ruin archetype (Moderate depth). Event Reclaim would give Eclipse a unique engine that no other Ruin archetype wants, because Cinder and Bedrock are character-focused.

4. **Death echo chains (Vector 4) could make Cinder's identity more distinctive.** Currently, Cinder's identity is "sacrifice + recur." Death echo amplification (meta-triggers on Dissolved triggers) would make Cinder specifically about maximizing death-trigger density, rather than just "any sacrifice is good."

5. **Ruin should NEVER get energy ramp, board protection, hand disruption, or fast-speed recursion.** These would violate its identity and collapse the distinctions that make Bedrock (Stone partner = ramp), Undertow (Tide partner = information), and Eclipse (Zephyr partner = speed) feel different.

6. **The four Ruin archetypes are currently well-differentiated by sub-theme but could be further differentiated by synergy axis.** The existing framework separates them by what aspect of the void they care about (volume vs. cycling vs. specific targets vs. death triggers). The synergy vectors identified here would add a second axis of differentiation based on RATE (velocity), COMPOSITION (character vs. event), and ORIGIN (from-void vs. from-hand materialization).
