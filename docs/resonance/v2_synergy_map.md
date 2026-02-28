# V2 Synergy Map -- Unified Synthesis for Round 4 Card Design

## Purpose

This document is the single authoritative input for all five Round 4 card design agents. It synthesizes the Round 1 critiques (draft experience, deckbuilding flexibility, mechanical patterns) with the Round 2 synergy discoveries (Tide, Ember, Zephyr, Stone, Ruin) into a unified catalog of exploitable game-state conditions, ranked by design priority.

---

## 1. Synergy Vector Catalog

Every exploitable game-state condition discovered across all five resonance reports, organized into a single reference. Each vector is tagged with its source resonance(s), the archetype pairs that benefit, and the kind of card that would exploit it.

---

### V01: Hand-Size-Matters
- **Description:** Tide and Zephyr (via bounce) routinely maintain 6-8 card hands. Zero cards in the 222-card pool reference hand size.
- **Produced by:** Tide (draw accumulation), Zephyr (bounce inflating hand via Key to the Moment, Nomad of Endless Paths)
- **Archetypes that benefit:** Depths (control hoards answers), Mirage (bounce fills hand before replaying), Tempest (mid-chain hand fluctuation), Eclipse (draw-then-discard fills hand temporarily)
- **Breadth:** 4 archetypes
- **Why non-obvious:** Dreamtides has literally zero hand-size-matters effects despite Tide's entire identity being "draw cards." The axis is invisible.
- **Card design shape:** A character with "spark equal to cards in hand" or a threshold effect ("if 6+ cards in hand, gain energy/draw/cost reduction"). Could also be a conditional on an otherwise generic card.
- **Subtlety:** 8/10
- **Problem addressed:** Gives Depths a proactive finisher axis (currently lacks one), gives Mirage a reason to care about bounce beyond re-triggers.

### V02: Prevent-Trigger Payoffs
- **Description:** Tide prevents 2-4 cards per game, but Prevent is purely defensive. No card triggers "when you prevent."
- **Produced by:** Tide (9 Prevent effects in Depths alone)
- **Archetypes that benefit:** Depths (primary -- transforms reactive into proactive), Gale (splashes fast Prevents like Abolish), any Tide deck
- **Breadth:** 2-3 archetypes
- **Subtlety:** 7/10
- **Card design shape:** "When you prevent a card, kindle 1" or "When you prevent a card, gain 1 energy." Makes Depths proactive without violating its identity.
- **Problem addressed:** Depths signpost gap (only 1 dual signpost). Prevent-trigger payoff would be a natural dual Tide+Stone signpost.

### V03: Board-Width-from-Low-Spark-Bodies
- **Description:** Tide accumulates 3-5 low-spark Materialized-draw bodies; Zephyr fields 4-6 Spirit Animals/figments at 0-1 spark. No card rewards board WIDTH independent of tribal identity.
- **Produced by:** Tide (Materialized-draw bodies linger), Zephyr (cheap Spirit Animals + figments), Stone (cheap Warriors persist)
- **Archetypes that benefit:** Mirage (wide flicker boards), Basalt (wide Spirit Animal boards), Crucible (wide Warrior boards), Cinder (figment fodder boards)
- **Breadth:** 4+ archetypes
- **Subtlety:** 4/10 (go-wide is a known concept, but Dreamtides has zero such cards)
- **Card design shape:** "Gain 1 point per ally" or "Dissolve enemies with spark less than your ally count" or "Prevent if you have 4+ allies."
- **Problem addressed:** Stone deficit (needs new non-tribal cards), Crucible linearity (width payoff gives Warriors a non-lord axis).

### V04: Foresee-to-Top-of-Deck Pipeline
- **Description:** After Foresee, the top card is known and curated. No Tide card exploits this known-top-card state.
- **Produced by:** Tide (Foresee effects: Oracle of Shifting Skies, Astral Navigators, Synaptic Sentinel, Guiding Light)
- **Archetypes that benefit:** Tempest (play top card for free as chain extender), Basalt (Dreamborne Leviathan already plays from top), Depths (card selection into guaranteed play), Mirage (Foresee + play from top)
- **Breadth:** 3-4 archetypes
- **Subtlety:** 9/10 (the information is consumed by draw immediately, making the intermediate state invisible)
- **Card design shape:** "Once per turn, you may play the top card of your deck if its cost is 3 or less" or "Foresee 2, then play the top card for free."
- **Problem addressed:** Creates a unique Tide axis that differentiates from generic draw.

### V05: Unspent-Energy-at-End-of-Turn
- **Description:** Tide/Stone players hold up 2-3 energy for Prevent or activated abilities, then it goes unused if the opponent plays around it.
- **Produced by:** Tide (Prevent holding pattern), Stone (energy surplus from ramp -- SE-1 in Stone report)
- **Archetypes that benefit:** Depths (primary -- converts the Prevent/deploy tension into a decision rather than a waste), Basalt (energy surplus), Crucible (ramp surplus)
- **Breadth:** 3 archetypes
- **Subtlety:** 7/10
- **Card design shape:** "At end of turn, if you have 2+ energy, draw 1" or "Judgment: Gain 1 point for every 3 unspent energy." Stone-coded.
- **Problem addressed:** Stone deficit, Depths signpost gap (this is a natural Tide+Stone card).

### V06: Event-Count-in-Void
- **Description:** Tide/Tempest voids are 70-80% events. Distinct from Ruin's generic void-size-matters.
- **Produced by:** Tide (events resolve to void), Ember (Tempest plays 8-15 events per game)
- **Archetypes that benefit:** Tempest (primary -- scales with storm history), Depths (plays many Prevent events), any event-heavy deck
- **Breadth:** 2-3 archetypes
- **Subtlety:** 9/10 (everyone conflates "void count" with Ruin; event-specific void count is Tide territory)
- **Card design shape:** "Spark equal to events in your void" (a Tide-coded Spirit of Smoldering Echoes), or "Draw 1 for every 3 events in void."
- **Problem addressed:** Differentiates Tide void from Ruin void. Gives Tempest a late-game scaling payoff independent of storm-turn explosiveness.

### V07: Opponent's Hand Depletion (Hand-Size Differential)
- **Description:** Tide's disruption + draw creates a hand-size gap. No card exploits the differential.
- **Produced by:** Tide (Break the Veil, Lurking Dread shrink opponent's hand; draw grows yours)
- **Archetypes that benefit:** Depths (primary), Tempest (secondary)
- **Breadth:** 2 archetypes
- **Subtlety:** 8/10
- **Card design shape:** "If you have more cards in hand than the opponent, your events cost 1 less."
- **Problem addressed:** Depths signpost gap, Depths deckbuilding variety (creates an aggressive-disruption Depths build).

### V08: Prevent-as-Void-Fuel
- **Description:** Prevent events go to YOUR void after resolving. A Prevent is simultaneously a counter and a self-mill trigger.
- **Produced by:** Tide (Prevent effects resolving to void)
- **Archetypes that benefit:** Undertow hybrid (Depths drafter who splashes mill payoffs)
- **Breadth:** 1-2 archetypes
- **Subtlety:** 10/10 (conceptually paradoxical)
- **Card design shape:** "When a Prevent event you control enters the void, Foresee 1" or "When you play a Prevent, put the top card of your deck into your void."
- **Problem addressed:** Novel but niche. Low priority unless a Depths-Undertow hybrid is desired.

### V09: Draw-Triggers for Non-Tide Payoffs
- **Description:** Tide draws 3-6 extra cards per big turn. Only Eternal Sentry has a draw-count trigger.
- **Produced by:** Tide (draw volume), Zephyr (cycling produces draws)
- **Archetypes that benefit:** Undertow (Eternal Sentry proof-of-concept), Stone (kindle-on-draw), Eclipse (draw-discard cycles)
- **Breadth:** 3 archetypes
- **Subtlety:** 7/10
- **Card design shape:** "When you draw your second card each turn, kindle 1" or "When you draw 3+ cards in a turn, gain 1 energy." Stone-coded.
- **Problem addressed:** Stone deficit, creates a Tide-Stone bridge that is NOT Prevent-based.

### V10: Materialized Bodies as Sacrifice Fodder
- **Description:** Tide's low-spark Materialized-draw bodies are dead weight after their ETB. They are ideal sacrifice fodder.
- **Produced by:** Tide (Materialized-draw bodies linger), Zephyr (figment tokens)
- **Archetypes that benefit:** Cinder (sacrifice loops), Gale (fast bodies as fodder for Pyrokinetic Surge), Tempest (Packcaller figments as sacrifice targets)
- **Breadth:** 3 archetypes
- **Subtlety:** 8/10 (violates Tide's stated identity of "characters worth more alive")
- **Card design shape:** "Abandon an ally that entered play this turn: Draw 1" (the body served its Materialized purpose, now it serves as sacrifice).
- **Problem addressed:** Cross-archetype bridge between Tide and Ember archetypes.

### V11: Information Asymmetry from Opponent Hand Knowledge
- **Description:** Wraith of Twisting Shadows reveals the opponent's hand. No card rewards "having seen" the hand.
- **Produced by:** Tide (Wraith, flicker-Wraith loops in Mirage)
- **Archetypes that benefit:** Depths (surgical Prevents), Mirage (repeated Wraith flicker)
- **Breadth:** 2 archetypes
- **Subtlety:** 9/10
- **Card design shape:** "If you have seen the opponent's hand this turn, your Prevents cost 1 less." Digital-native.
- **Problem addressed:** Creates a unique digital-first mechanic. Low priority due to implementation complexity.

### V12: Deck-Size Differential
- **Description:** Tide draws aggressively, thinning deck. Zephyr thins via figment generation and cycling. No card rewards a small deck (except Lumineth alt-win).
- **Produced by:** Tide (heavy draw), Zephyr (cycling + figment generation)
- **Archetypes that benefit:** Mirage (sees most of deck), Basalt (rapid deck thinning), Tempest (draws through deck), Undertow (mills through deck)
- **Breadth:** 3-4 archetypes
- **Subtlety:** 7/10
- **Card design shape:** "If your deck has 5 or fewer cards, characters you play cost 2 less" or "Gain 1 energy for each card fewer than 10 in your deck."
- **Problem addressed:** Late-game payoff for aggressive draw/mill strategies. Moderate priority.

### V13: Void Character Density (Ember-specific)
- **Description:** Cinder's abandon fills the void specifically with characters, not events. Distinct from Undertow's random mill.
- **Produced by:** Ember (abandon effects send characters to void)
- **Archetypes that benefit:** Cinder (primary -- void is character-dense), Bedrock (deposits expensive characters)
- **Breadth:** 2 archetypes
- **Subtlety:** 7/10
- **Card design shape:** "Dissolve an enemy with cost less than the number of characters in your void" (character-specific Weight of Memory) or "Materialize the highest-cost character from your void; abandon it at end of turn."
- **Problem addressed:** Differentiates Cinder's void from Undertow's void, reducing Ruin bottleneck competition.

### V14: Hellbent Payoffs (Empty Hand)
- **Description:** Ember's discard-as-cost and aggressive deployment leave the player empty-handed. No card rewards an empty hand.
- **Produced by:** Ember (discard-as-cost: Fell the Mighty, Pulse of Sacrifice), Gale (aggressive fast deployment empties hand)
- **Archetypes that benefit:** Gale (primary -- fast deployment empties hand as a signal of success), Eclipse (draw-discard cycles can bottom out), Cinder (aggressive sacrifice empties hand)
- **Breadth:** 3 archetypes
- **Subtlety:** 8/10 (empty hand feels like losing, but in fast-tempo games it signals execution)
- **Card design shape:** "While you have no cards in hand, characters you control have +1 spark" or "While you have no cards in hand, your fast characters cost 1 less."
- **Problem addressed:** Gale signpost gap. Creates anti-synergy with Tide (which always has cards), producing clean resonance separation.

### V15: Abandon-as-Pseudo-Flicker (Materialize-Matters via Reclaim)
- **Description:** Cinder's sacrifice-Reclaim loops trigger Materialized abilities on re-entry, just like Mirage's flicker.
- **Produced by:** Ruin (Reclaim loops), Ember (abandon as the exit mechanism)
- **Archetypes that benefit:** Cinder (primary -- massive materialization count through loops), Bedrock (Reclaim expensive bodies re-triggers Materialized)
- **Breadth:** 2-3 archetypes
- **Subtlety:** 8/10 (players think of materializations as "from flicker" not "from Reclaim")
- **Card design shape:** "When you materialize a character from your void, draw 1 and gain 1 energy" or "When a character enters play from your void, this character gains +1 spark."
- **Problem addressed:** Creates genuine draft tension between Mirage and Cinder over materialize-matters cards. Addresses Cinder deckbuilding (new axis beyond pure sacrifice).

### V16: Kindle Concentration as Voltron
- **Description:** Kindle always targets leftmost character. Cinder's repeated kindle creates a single 6-12 spark "tower" body.
- **Produced by:** Ember (Infernal Ascendant kindle 2 per abandon), Stone (Ebonwing, Spirit Field Reclaimer kindle on Judgment)
- **Archetypes that benefit:** Cinder (primary -- active kindle through sacrifice), Crucible (Warrior kindle), Undertow (Silent Avenger passive kindle)
- **Breadth:** 3 archetypes
- **Subtlety:** 9/10 (kindle is treated as incremental, not as geometric growth)
- **Card design shape:** "Abandon a character: Gain points equal to that character's spark" (spark-to-points conversion) or "Remove 3 spark from leftmost ally: Dissolve an enemy cost 3 or less" (kindle-as-removal).
- **Problem addressed:** Crucible linearity (gives Warriors a spark-spending decision), Cinder deckbuilding variety (kindle voltron as alternate win condition).

### V17: Sacrifice Bodies as Prevent Fuel
- **Description:** Herald of the Last Light converts a body into a counter. Expanding this pattern would give Ember decks their own Prevent suite via expendable bodies.
- **Produced by:** Ember (Herald of the Last Light is the proof-of-concept)
- **Archetypes that benefit:** Gale (fast sacrifice + Prevent), Cinder (expendable bodies become counterspells), Depths bridge (control via bodies)
- **Breadth:** 2-3 archetypes
- **Subtlety:** 9/10 (Prevent is Tide territory; body-as-counter inverts that)
- **Card design shape:** "When you abandon an ally to prevent a card, draw 1" or "Fast. Abandon this: Prevent a played character."
- **Problem addressed:** Gale signpost gap (creates a unique Gale-Depths bridge), reduces Gale's dependence on the Musician trio.

### V18: Figment Sacrifice Bridge
- **Description:** Zephyr generates 0-spark figment tokens (Radiant Trio, Endless Projection, Packcaller). Cinder needs sacrifice fodder. The pipeline is disconnected.
- **Produced by:** Zephyr (figment production), Ember (sacrifice consumption)
- **Archetypes that benefit:** Gale (Zephyr+Ember is the natural home), Cinder (wants fodder), Mirage (figments as disposable ETB triggers)
- **Breadth:** 3 archetypes
- **Subtlety:** 7/10 (token-sacrifice is known from other games but absent in Dreamtides)
- **Card design shape:** "When you abandon a character with 0 spark, gain 1 energy and kindle 1" (dual Zephyr+Ember) or "Abandon a figment: Draw 1."
- **Problem addressed:** Creates a Gale-Cinder bridge that does not currently exist. Reduces Gale's Musician dependence.

### V19: "Leaves Play" Payoffs
- **Description:** Zephyr characters leave play constantly (bounce, flicker, opponent removal). Only Starlit Cascade uses the "when ally leaves play" trigger. This is Zephyr's most frequent zone change with only 1 card noticing it.
- **Produced by:** Zephyr (flicker = leave then return), Ember (sacrifice = leave permanently)
- **Archetypes that benefit:** Mirage (primary -- every flicker is a leave), Cinder (every sacrifice is a leave), Gale (bounced bodies leave)
- **Breadth:** 3 archetypes
- **Subtlety:** 4/10 (concept is straightforward, but the 1-in-222 exploitation rate is the real gap)
- **Card design shape:** "When an ally leaves play, Foresee 1" or "When an ally leaves play, the next character you play costs 1 less."
- **Problem addressed:** Fills a glaring mechanical hole (1 card for the most frequent Zephyr event), strengthens Mirage deckbuilding variety.

### V20: Banish Zone Exploitation
- **Description:** Flicker sends characters through the banish zone. Only Tideborne Voyager and Pyrestone Avatar notice this transit.
- **Produced by:** Zephyr (flicker effects use banish as transit)
- **Archetypes that benefit:** Mirage (primary), Depths (Paradox Enforcer banishes enemies)
- **Breadth:** 2 archetypes
- **Subtlety:** 8/10 (banish zone is treated as an implementation detail, not a game state)
- **Card design shape:** "While you have a banished ally, your characters have +1 spark" or "When a character returns from banish, draw 1."
- **Problem addressed:** Creates a genuinely novel mechanic unique to Dreamtides. Moderate priority due to complexity.

### V21: Judgment Storm (Counting Judgment Triggers)
- **Description:** Stone boards fire 3-6 Judgment triggers per phase. No card counts the total Judgment triggers.
- **Produced by:** Stone (dense Judgment infrastructure: Wolfbond Chieftain, Dawnblade Wanderer, Ebonwing, etc.)
- **Archetypes that benefit:** Crucible (Warrior Judgment triggers), Basalt (Spirit Animal Judgment triggers), Depths (if they have Judgment bodies)
- **Breadth:** 3 archetypes
- **Subtlety:** 8/10 (Judgment triggers are independent; treating the phase as a holistic event is novel)
- **Card design shape:** "Judgment: Gain 1 point for each other Judgment ability that triggered this phase" or "At end of Judgment, kindle equal to the number of Judgment abilities triggered."
- **Problem addressed:** Stone deficit (primary). Creates a new Stone scaling axis that is not tribal.

### V22: Warrior Death Dividends
- **Description:** When the opponent removes Warriors, Stone currently gets nothing. A "death tax" on Warriors being dissolved by the opponent creates a dilemma.
- **Produced by:** Stone (Warriors persist, inviting opponent removal)
- **Archetypes that benefit:** Crucible (primary -- Warriors either survive for value OR die for value), Bedrock (Warrior bridge via Ashen Avenger)
- **Breadth:** 2 archetypes
- **Subtlety:** 8/10 (the distinction between "profit from death" [Ruin] and "punish opponent for killing" [Stone] is subtle)
- **Card design shape:** "When an allied Warrior is dissolved by the opponent, each allied Warrior gains +1 spark" or "When the opponent dissolves a Warrior you control, gain 2 energy and kindle 1."
- **Problem addressed:** Crucible linearity (creates a resilience layer), Stone deficit.

### V23: Deployment Storm (Ramp into Action Density)
- **Description:** Nexus Wayfinder + energy surplus enables 3-4 character plays per turn, matching Tempest's cards-per-turn count through fundamentally different means.
- **Produced by:** Stone (cost reduction + energy ramp)
- **Archetypes that benefit:** Crucible (deploy many Warriors per turn), Basalt (deploy many Spirit Animals), Depths (deploy multiple control bodies)
- **Breadth:** 3 archetypes
- **Subtlety:** 9/10 (cards-per-turn is perceived as Tempest's exclusive domain; Stone reaches it orthogonally)
- **Card design shape:** "Judgment: Gain 1 spark for each character you played this turn" or "When you play your 3rd card this turn, draw 2."
- **Problem addressed:** Stone deficit, Crucible linearity (new axis for Warrior decks beyond lord stacking).

### V24: Kindle Conversion (Spark as Flexible Resource)
- **Description:** Kindle creates a high-spark leftmost character. That spark is only used for scoring. Converting it to removal or energy creates a decision.
- **Produced by:** Stone (Kindle on Judgment), Ember (Kindle from abandon)
- **Archetypes that benefit:** Crucible (primary), Cinder (kindle tower becomes a removal tool), Basalt (kindle on Spirit Animals)
- **Breadth:** 3 archetypes
- **Subtlety:** 8/10 (kindle is one-dimensional; making it spendable adds a strategic axis)
- **Card design shape:** "Remove 3 spark from your leftmost ally: Dissolve an enemy with cost 3 or less" or "Leftmost ally's spark counts as energy for activated abilities."
- **Problem addressed:** Crucible linearity (creates a purity-vs-power decision), Stone deficit.

### V25: Energy Overflow Conversion
- **Description:** Stone frequently has 2-4 unspent energy at end of turn by midgame.
- **Produced by:** Stone (compounding Judgment energy generation)
- **Archetypes that benefit:** Depths (convert surplus to value), Basalt (energy sink), Crucible (energy sink)
- **Breadth:** 3 archetypes
- **Subtlety:** 7/10
- **Card design shape:** "End of turn: Gain 1 point for every 3 unspent energy" or "Judgment: If you have 6 or more energy, kindle 2."
- **Problem addressed:** Stone deficit (primary energy sink card).

### V26: Persistent Tax Effects (Static Stacking)
- **Description:** Stone's static effects (Skyflame Commander, Nexus Wayfinder, Cloaked Sentinel) stack multiplicatively because Stone keeps bodies alive. More tax effects would create a "soft lockout" win condition.
- **Produced by:** Stone (board permanence enables stacking)
- **Archetypes that benefit:** Depths (primary -- tax-control as a proactive win condition), Crucible (lord stacking)
- **Breadth:** 2 archetypes
- **Subtlety:** 7/10
- **Card design shape:** "Opponent's characters enter play with -1 spark" or "Characters the opponent plays cost 1 more."
- **Problem addressed:** Depths deckbuilding (creates a distinct "Tax Depths" build variant), Stone deficit.

### V27: The Anchor Effect (Continuous Board Presence Rewards)
- **Description:** No card rewards a character staying in play across turns. Stone's board persists but is never rewarded for persistence itself.
- **Produced by:** Stone (characters never leave play)
- **Archetypes that benefit:** Crucible (Warriors persist), Depths (control bodies persist), Basalt (Spirit Animals with Judgment triggers persist)
- **Breadth:** 3 archetypes
- **Subtlety:** 7/10 (many "enters play" triggers exist; zero "stayed in play" rewards exist)
- **Card design shape:** "At the start of your turn, if this character has been on the battlefield since your last turn, kindle 1."
- **Problem addressed:** Stone deficit. Anti-synergy with Zephyr flicker creates interesting enemy-pair tension.

### V28: Formation Positioning (Left/Right Board Position)
- **Description:** Kindle always targets leftmost. No other card interacts with position.
- **Produced by:** Stone (Kindle's implicit position awareness)
- **Archetypes that benefit:** Crucible (primary -- Warrior placement decisions), Stone generically
- **Breadth:** 1-2 archetypes
- **Subtlety:** 9/10 (completely unexplored axis)
- **Card design shape:** "Allied Warriors to the right of this character have +1 spark" or "Rightmost ally gains +1 spark at end of turn."
- **Problem addressed:** Stone deficit, Crucible linearity (creates a placement puzzle).

### V29: Void Velocity (Cards Entering Void Per Turn)
- **Description:** Different Ruin archetypes fill the void at different rates. No card rewards velocity, only total size.
- **Produced by:** Ruin (Undertow mills 4-8/turn, Eclipse discards 2-4/turn, Cinder sacrifices 2-3/turn, Bedrock 1-2/turn)
- **Archetypes that benefit:** Undertow (primary -- high velocity), Eclipse (secondary), Cinder (tertiary)
- **Breadth:** 3 archetypes
- **Subtlety:** 8/10 (void velocity is the "storm count" of Ruin)
- **Card design shape:** "At end of turn, if 3+ cards entered your void this turn, draw 1 and kindle 1."
- **Problem addressed:** Differentiates Undertow from Bedrock (both want void cards, but velocity separates them).

### V30: From-Void Materialized Distinction
- **Description:** Characters materialized from void via Reclaim are identical to those played from hand. No card distinguishes origin.
- **Produced by:** Ruin (Reclaim creates void-to-battlefield materializations)
- **Archetypes that benefit:** Cinder (high frequency of re-materialization from void), Bedrock (high-value individual void materializations), Undertow (Survivor recursion)
- **Breadth:** 3 archetypes
- **Subtlety:** 7/10
- **Card design shape:** "When you materialize a character from your void, kindle 1" or "When a character enters play from your void, draw 1."
- **Problem addressed:** Gives Ruin its own materialize-matters axis, distinct from Zephyr's flicker-based one. Reduces Ruin-Zephyr boundary confusion.

### V31: Death Echo Chains (Meta-Triggers on Dissolved)
- **Description:** Boards with multiple Dissolved-trigger characters create exponentially more value per death event.
- **Produced by:** Ruin (Dissolved triggers: Sunset Chronicler, Avatar of Cosmic Reckoning, Volcanic Channeler, Dustborn Veteran, Silent Avenger, Seer of the Fallen)
- **Archetypes that benefit:** Cinder (primary -- engineers deaths), Undertow (Survivors have Dissolved triggers)
- **Breadth:** 2 archetypes
- **Subtlety:** 8/10 (meta-triggers on triggers are unusual)
- **Card design shape:** "Whenever a Dissolved ability triggers, kindle 1" or "When a Dissolved trigger fires, gain 1 energy."
- **Problem addressed:** Makes Cinder's identity more distinctive (death-trigger density as the win condition, not just "sacrifice stuff").

### V32: Void-Only Characters (Expanded Template)
- **Description:** Revenant of the Lost (void-only, 3 cost, 6 spark) is the only void-gated character. More void-only characters at different cost/spark points would create a "void aggro" sub-strategy.
- **Produced by:** Ruin (void-only play condition)
- **Archetypes that benefit:** Bedrock (primary -- deploys from void), Undertow (mills them naturally), Eclipse (discards them + Ashmaze gives Reclaim), Cinder (cheap sacrifice bodies)
- **Breadth:** 4 archetypes
- **Subtlety:** 7/10
- **Card design shape:** "2 cost, 4 spark. Only playable from void. Dissolved: Return to void."
- **Problem addressed:** Bedrock fragility (more void-only targets reduce dependence on 3 contested cards).

### V33: Event Reclaim as Engine
- **Description:** Events can have Reclaim but event-Reclaim is treated as utility, not as an engine axis. Eclipse/Tempest could use event recursion as a spellslinger variant.
- **Produced by:** Ruin (Ashlight Caller, Whisper of the Past, Ashmaze Guide giving Reclaim to events)
- **Archetypes that benefit:** Eclipse (primary -- Ashmaze Guide + event discard creates cycle), Tempest (event recovery for storm)
- **Breadth:** 2 archetypes
- **Subtlety:** 8/10 (event Reclaim exists but no card rewards playing events from void specifically)
- **Card design shape:** "When you play an event from your void, copy it" or "Events you play from your void cost 1 less."
- **Problem addressed:** Eclipse depth (its payoff layer is thin at 6 cards). Event Reclaim gives Eclipse a unique engine no other Ruin archetype wants.

### V34: Burst Energy into Activated Abilities
- **Description:** Ember generates 4-10 energy in burst. Activated abilities (Assault Leader 4-energy, Spiritbound Alpha 4-energy, Mystic Runefish 3-energy) can consume this burst outside of Tempest.
- **Produced by:** Ember (burst energy events: Genesis Burst, Flash of Power)
- **Archetypes that benefit:** Crucible (Assault Leader activation), Basalt (Spiritbound Alpha + Mystic Runefish), Depths (expensive activated abilities)
- **Breadth:** 3 archetypes
- **Subtlety:** 7/10 (burst energy is perceived as Tempest fuel; using it for activated abilities is different)
- **Card design shape:** "Until end of turn, activated abilities cost 2 less" or "Gain energy equal to the number of allies with activated abilities."
- **Problem addressed:** Crucible-Basalt bridge via Stone axis. Gives burst energy a non-Tempest purpose.

### V35: Removal as Storm Count
- **Description:** Tempest's removal events (Immolate, Fell the Mighty, etc.) increment storm count while also interacting with the board.
- **Produced by:** Ember (removal events that are also storm fuel)
- **Archetypes that benefit:** Tempest (primary -- interactive storm), Depths (control + storm hybrid)
- **Breadth:** 2 archetypes
- **Subtlety:** 6/10
- **Card design shape:** "When you dissolve an enemy, draw 1 and gain 1 energy" (permanent that converts removal into storm fuel).
- **Problem addressed:** Tempest linearity (creates an interactive storm variant instead of "goldfish combo").

### V36: Figments as Storm Payoff / Sacrifice Input
- **Description:** Packcaller of Shadows generates figments equal to cards played. Figments are idle post-storm in Tempest but premium sacrifice fodder in Cinder.
- **Produced by:** Ember/Zephyr (figment generation from storm or deployment)
- **Archetypes that benefit:** Tempest (figments as storm board payoff), Cinder (figments as sacrifice inputs)
- **Breadth:** 2 archetypes
- **Subtlety:** 7/10
- **Card design shape:** "When you abandon a character with 0 spark, gain 1 energy and kindle 1."
- **Problem addressed:** Tempest-Cinder bridge (both share Ember but are currently disconnected).

### V37: Temporary Spark Matters (Zephyr-Specific Spark Identity)
- **Description:** Several proposed cards use "until end of turn" spark. This could become a Zephyr-defining mechanical axis.
- **Produced by:** Zephyr (Tempest Striker, Voidweave Dancer, Verdant Packmother all grant temporary spark)
- **Archetypes that benefit:** Gale, Eclipse, Basalt, Mirage (all Zephyr archetypes)
- **Breadth:** 4 archetypes
- **Subtlety:** 8/10 (temporary-spark-matters does not exist in any card game)
- **Card design shape:** "When a character you control loses spark at end of turn, draw 1" or "Characters with temporary spark have +1 additional spark."
- **Problem addressed:** Gives Zephyr a mechanical identity for spark that is distinct from Stone's permanent accumulation.

### V38: Survivor Spread as Cross-Archetype Incentive
- **Description:** 23 Survivors exist but tribal payoffs are concentrated in Undertow. Many non-Undertow archetypes have 2-4 incidental Survivors.
- **Produced by:** Ruin (Survivor bodies distributed across archetypes)
- **Archetypes that benefit:** Eclipse (Wasteland Arbitrator, Emberwatch Veteran), Cinder (Resilient Wanderer, Dustborn Veteran), Gale (Wasteland Arbitrator)
- **Breadth:** 3 archetypes
- **Subtlety:** 6/10
- **Card design shape:** "Once per turn, when a Survivor enters your void, [small effect]." Low-threshold Survivor payoff that non-Undertow archetypes can trigger.
- **Problem addressed:** Reduces Undertow's tribal insularity, creates draft tension over Survivor bodies.

### V39: Reclaim Cost Manipulation
- **Description:** Nexus Wayfinder's "characters cost 2 less" applies to Reclaim costs. This makes Stone's cost reduction disproportionately powerful in Ruin.
- **Produced by:** Stone (cost reduction) interacting with Ruin (Reclaim costs)
- **Archetypes that benefit:** Bedrock (primary -- expensive targets become cheap from void), Undertow (cheap Survivors become free)
- **Breadth:** 2 archetypes
- **Subtlety:** 6/10
- **Card design shape:** "Characters you play from your void cost 2 less." Targeted cost reducer for void plays.
- **Problem addressed:** Bedrock fragility (makes ramp backup plan more viable).

### V40: Spirit Animal Discard Bridge
- **Description:** A Spirit Animal whose entry mechanism is being discarded would create an Eclipse-Basalt axis.
- **Produced by:** Zephyr (Spirit Animal tribal), Ruin (discard)
- **Archetypes that benefit:** Eclipse (discard triggers deployment), Basalt (Spirit Animal density from unexpected source)
- **Breadth:** 2 archetypes
- **Subtlety:** 7/10 (tribal and discard are treated as completely separate)
- **Card design shape:** "Spirit Animal, 2 cost, 1 spark. When you discard this character, materialize it."
- **Problem addressed:** Eclipse depth (new type of discard payoff), creates Eclipse-Basalt bridge that does not currently exist.

---

## 2. Top 25 Priority Vectors

Ranked by composite score: Breadth (1-5) x Subtlety (1-10) x Problem-Solving Value (1-10, how directly it addresses known v1 problems).

| Rank | Vector | Breadth | Subtlety | Problem Value | Composite | Primary Problem Addressed |
|------|--------|---------|----------|---------------|-----------|--------------------------|
| 1 | **V01: Hand-Size-Matters** | 4 | 8 | 9 | 288 | Depths signpost gap, Mirage diversity |
| 2 | **V21: Judgment Storm** | 3 | 8 | 10 | 240 | Stone deficit, Crucible linearity |
| 3 | **V23: Deployment Storm** | 3 | 9 | 9 | 243 | Stone deficit, Crucible linearity |
| 4 | **V16: Kindle Concentration** | 3 | 9 | 8 | 216 | Crucible linearity, Cinder variety |
| 5 | **V14: Hellbent Payoffs** | 3 | 8 | 9 | 216 | Gale signpost gap, resonance separation |
| 6 | **V15: Abandon-as-Pseudo-Flicker** | 2.5 | 8 | 9 | 180 | Cinder-Mirage bridge, Ruin bottleneck |
| 7 | **V06: Event-Count-in-Void** | 2.5 | 9 | 8 | 180 | Tempest variety, Tide-Ruin differentiation |
| 8 | **V17: Sacrifice Bodies as Prevent** | 2.5 | 9 | 8 | 180 | Gale signpost gap, Gale-Depths bridge |
| 9 | **V22: Warrior Death Dividends** | 2 | 8 | 10 | 160 | Crucible linearity, Stone deficit |
| 10 | **V29: Void Velocity** | 3 | 8 | 8 | 192 | Ruin bottleneck (Undertow vs Bedrock) |
| 11 | **V02: Prevent-Trigger Payoffs** | 2.5 | 7 | 9 | 158 | Depths signpost gap |
| 12 | **V19: Leaves-Play Payoffs** | 3 | 4 | 10 | 120 | Mechanical hole (1 card in 222) |
| 13 | **V33: Event Reclaim Engine** | 2 | 8 | 9 | 144 | Eclipse depth (thin payoff layer) |
| 14 | **V18: Figment Sacrifice Bridge** | 3 | 7 | 7 | 147 | Gale Musician dependence, Gale-Cinder bridge |
| 15 | **V32: Void-Only Characters** | 4 | 7 | 7 | 196 | Bedrock fragility |
| 16 | **V30: From-Void Materialized** | 3 | 7 | 7 | 147 | Ruin materialize-matters axis |
| 17 | **V05: Unspent Energy at EOT** | 3 | 7 | 7 | 147 | Stone deficit, Depths signpost |
| 18 | **V24: Kindle Conversion** | 3 | 8 | 6 | 144 | Crucible linearity, Stone deficit |
| 19 | **V04: Foresee-to-Top-of-Deck** | 3 | 9 | 5 | 135 | Unique Tide axis |
| 20 | **V09: Draw-Triggers for Non-Tide** | 3 | 7 | 7 | 147 | Stone deficit, Tide-Stone bridge |
| 21 | **V26: Persistent Tax Effects** | 2 | 7 | 8 | 112 | Depths variety, Stone deficit |
| 22 | **V37: Temporary Spark Matters** | 4 | 8 | 4 | 128 | Zephyr identity cohesion |
| 23 | **V34: Burst Energy into Activated** | 3 | 7 | 6 | 126 | Crucible-Basalt bridge |
| 24 | **V27: Anchor Effect** | 3 | 7 | 6 | 126 | Stone deficit |
| 25 | **V31: Death Echo Chains** | 2 | 8 | 7 | 112 | Cinder identity sharpening |

---

## 3. Archetype Gap Analysis

Using the drafter critique's rails scores and the deckbuilder critique's flex-slot counts, here are the archetypes most in need of help, cross-referenced with which vectors serve them.

### Tier 1: Desperate Need

**Crucible (Stone+Ember)** -- Rails 9/10, Flex 6-8, Auto-includes 16
- THE most on-rails archetype. Builds itself. Warrior lords create zero decisions.
- **Priority vectors:** V21 (Judgment Storm), V22 (Warrior Death Dividends), V23 (Deployment Storm), V24 (Kindle Conversion), V28 (Formation Positioning), V16 (Kindle Concentration), V03 (Board Width), V34 (Burst Energy into Activated)
- **What Crucible needs:** Cards that create tension between Warrior density and alternative axes. A non-Warrior card that is individually powerful but dilutes lords. A Warrior with an ability relevant outside Crucible.

**Bedrock (Stone+Ruin)** -- Rails 2/10 (discovery-rich but fragile), Flex 8-10, Auto-includes 8
- The most contested archetype -- every card is shared. Fragile to being out-drafted.
- **Priority vectors:** V32 (Void-Only Characters), V39 (Reclaim Cost Manipulation), V30 (From-Void Materialized), V15 (Abandon-as-Pseudo-Flicker), V13 (Void Character Density)
- **What Bedrock needs:** More cards that ONLY Bedrock wants (reducing 3-way Ruin competition), backup void-only targets, cheaper void-to-play enablers.

### Tier 2: Significant Need

**Gale (Zephyr+Ember)** -- Rails 7/10, Flex 8-10, Auto-includes 10
- Musician trio is too dominant. Only 1 dual signpost. Aggressive build is suboptimal.
- **Priority vectors:** V14 (Hellbent), V17 (Sacrifice as Prevent), V18 (Figment Sacrifice Bridge), V37 (Temporary Spark), V19 (Leaves-Play)
- **What Gale needs:** A second mechanical axis beyond "play fast cards, trigger Musicians." Signpost cards that create a distinct Gale identity separate from "fast-matters."

**Depths (Tide+Stone)** -- Rails 6/10, Flex 10-12, Auto-includes 8
- Only 1 dual signpost. Lacks a proactive finisher axis. Tax Depths build is underserved.
- **Priority vectors:** V01 (Hand-Size-Matters), V02 (Prevent-Trigger), V05 (Unspent Energy), V07 (Hand Differential), V26 (Persistent Tax)
- **What Depths needs:** Dual Tide+Stone signpost cards (needs 2-3 more per the signpost policy). A proactive win condition that emerges from the control game plan.

**Stone (resonance deficit)** -- 31 mono cards vs. ~40 target = 9-card deficit
- The resonance with fewest cards. Needs 5-9 new mono-Stone or Stone-dual cards.
- **Priority vectors:** V21, V22, V23, V24, V25, V27, V28, V05, V09 (all Stone vectors)
- **What Stone needs:** Cards that exploit side effects of Stone's existing mechanics without touching void, sacrifice, or fast.

### Tier 3: Moderate Need

**Eclipse (Zephyr+Ruin)** -- Rails 3/10, Flex 12-14, Auto-includes 6
- Best emergent discovery but thin payoff layer (only 6 discard payoffs).
- **Priority vectors:** V33 (Event Reclaim Engine), V40 (Spirit Animal Discard Bridge), V29 (Void Velocity)
- **What Eclipse needs:** 1-2 more discard payoffs that are NOT linear "when you discard, gain +1 X."

**Tempest (Tide+Ember)** -- Rails 5/10, Flex 8-10, Auto-includes 12
- Mechanic critique found 4/5 new Tempest cards use identical "2+ events" trigger. Needs variety.
- **Priority vectors:** V06 (Event-Count-in-Void), V35 (Removal as Storm Count), V04 (Foresee Pipeline), V36 (Figments as Sacrifice)
- **What Tempest needs:** Storm payoffs that use DIFFERENT triggers than "events played this turn." Late-game scaling that rewards cumulative event play.

### Tier 4: Healthy (Minimal Intervention)

**Mirage (Tide+Zephyr)** -- Rails 4/10, Flex 10-12, Auto-includes 8. Well-designed.
**Cinder (Ember+Ruin)** -- Rails 5/10, Flex 10-12, Auto-includes 10. Well-designed.
**Undertow (Tide+Ruin)** -- Rails 5/10, Flex 8-10, Auto-includes 12. Survivor package is too synergistic but functional.

---

## 4. Cross-Archetype Bridge Map

This matrix shows which synergy vectors bridge NON-ADJACENT archetype pairs (pairs that do NOT share a resonance). These are the most valuable for reducing the "on-rails" feel because they create draft tension across unrelated lanes.

### Non-Adjacent Pairs and Their Vectors

| Pair | Shared Resonances | Bridge Vectors | Value |
|------|-------------------|----------------|-------|
| **Tempest -- Basalt** | None (Tide+Ember vs Zephyr+Stone) | V04 (Foresee pipeline -- Dreamborne Leviathan), V34 (Burst energy into activated abilities) | HIGH -- these archetypes currently have zero overlap |
| **Tempest -- Cinder** | Ember | V36 (Figments as sacrifice), V06 (Event-void-count benefits both), V35 (Removal as storm for Cinder's removal events) | MEDIUM -- Ember connection exists but is underexploited |
| **Tempest -- Bedrock** | None (Tide+Ember vs Stone+Ruin) | V06 (Event-void density in Tempest's void), V04 (Foresee into Bedrock's expensive targets) | LOW -- distant resonance gap |
| **Depths -- Cinder** | None (Tide+Stone vs Ember+Ruin) | V16 (Kindle conversion -- Depths can spend kindle for removal), V17 (Sacrifice as Prevent -- Cinder bodies become counterspells) | HIGH -- control-sacrifice hybrid is novel |
| **Depths -- Eclipse** | None (Tide+Stone vs Zephyr+Ruin) | V01 (Hand-size-matters -- Eclipse temporarily inflates hand before discarding), V07 (Hand differential -- Eclipse cycling creates asymmetry) | MEDIUM -- hand-size bridge |
| **Depths -- Gale** | None (Tide+Stone vs Zephyr+Ember) | V17 (Sacrifice as Prevent), V14 (Hellbent -- Gale empties hand, Depths fills it -- creates opposite poles of same axis) | HIGH -- tempo-control hybrid |
| **Mirage -- Crucible** | None (Tide+Zephyr vs Stone+Ember) | V03 (Board width -- both field wide boards), Frost Visionary already bridges (Warrior body + flicker target) | MEDIUM -- existing bridge works |
| **Mirage -- Cinder** | None (Tide+Zephyr vs Ember+Ruin) | V15 (Abandon-as-Pseudo-Flicker -- materialize-matters cards contested), V19 (Leaves-play payoffs -- both cause allies to leave) | HIGH -- most valuable non-adjacent bridge |
| **Mirage -- Bedrock** | None (Tide+Zephyr vs Stone+Ruin) | V30 (From-void materialized -- Bedrock's Reclaim targets trigger Materialized), Echoing Monolith already bridges | MEDIUM |
| **Basalt -- Cinder** | None (Zephyr+Stone vs Ember+Ruin) | V18 (Figment sacrifice bridge), V34 (Burst energy into activated -- Cinder's energy burst powering Basalt's activated abilities is bizarre but functional) | MEDIUM |
| **Basalt -- Eclipse** | None (Zephyr+Stone vs Zephyr+Ruin -- WAIT: both share Zephyr) | V40 (Spirit Animal Discard Bridge), V37 (Temporary Spark) | MEDIUM -- Zephyr connection makes this easier |
| **Crucible -- Undertow** | None (Stone+Ember vs Tide+Ruin) | V22 (Warrior Death Dividends -- Warriors with Dissolved triggers bridge to Ruin), Speaker for the Forgotten already bridges | LOW -- distant pair |
| **Crucible -- Eclipse** | None (Stone+Ember vs Zephyr+Ruin) | V16 (Kindle concentration -- Eclipse's Torchbearer kindles on discard), V28 (Formation -- novel Stone mechanic irrelevant to Eclipse) | LOW -- very distant |
| **Gale -- Undertow** | None (Zephyr+Ember vs Tide+Ruin) | V38 (Survivor spread -- Wasteland Arbitrator is a Gale Survivor), V14 (Hellbent -- both can empty hand) | LOW |
| **Gale -- Bedrock** | None (Zephyr+Ember vs Stone+Ruin) | V14 (Hellbent -- Gale empties hand; Bedrock discards expensive cards to void), V18 (Figment sacrifice -- marginal) | LOW |

### Highest-Value Non-Adjacent Bridges (Top 5)

1. **Mirage -- Cinder** via V15 (Abandon-as-Pseudo-Flicker) and V19 (Leaves-Play). These archetypes share zero resonances but both generate massive "ally leaves play" and "ally enters play" events through completely different mechanisms. A single "when ally leaves play" or "when you materialize from void" card creates explosive draft tension.

2. **Depths -- Gale** via V17 (Sacrifice as Prevent). The control-tempo hybrid that uses expendable fast bodies as counterspells is mechanically novel and would make Herald of the Last Light a format-defining bridge card.

3. **Depths -- Cinder** via V16 (Kindle Conversion) and V17 (Sacrifice as Prevent). Spending kindle for removal and using sacrifice as denial creates a control-aristocrats hybrid that no current archetype supports.

4. **Tempest -- Basalt** via V34 (Burst Energy into Activated). Using Tempest-style energy burst to power Basalt's activated abilities (Spiritbound Alpha, Mystic Runefish) creates a combo that crosses the entire pentagon.

5. **Crucible -- Bedrock** via V22 (Warrior Death Dividends) and Speaker for the Forgotten. The "Warrior Reanimator" 3-color splinter identified in the deckbuilder critique -- Warriors that die and return through Ruin's recursion.

---

## 5. Design Constraints Summary

All anti-synergy warnings from all five resonance agents, consolidated into a single "do not cross" list.

### Tide Constraints
1. **Tide must NOT generate energy.** Tide's resource is cards, not energy. No "when you draw, gain energy."
2. **Tide must NOT have aggressive spark scaling.** Finishers should reach 5-7 spark through game-state conditions, not explosive growth.
3. **Tide must NOT do mass recursion.** Surgical event recovery is Tide; mass void-to-hand is Ruin.
4. **Tide must NOT provide tribal payoffs.** No "for each Warrior, draw 1" on a Tide card.
5. **Tide must NOT own sacrifice outlets.** Tide supplies fodder; Ember supplies the knife.
6. **Tide must NOT bypass draw-or-deploy tension.** Converting all characters to fast would collapse Tide into Zephyr.

### Ember Constraints
1. **Ember does NOT recur.** No "when you dissolve, Reclaim." Recursion is Ruin's territory.
2. **Ember does NOT accumulate card advantage.** Draw is always incidental (1 card max per effect).
3. **Ember does NOT build persistent engines.** Effects are one-shot detonations; persistence comes from Ruin.
4. **Ember does NOT protect.** No shields, damage prevention, or indestructibility.
5. **Ember does NOT manipulate the deck.** No Foresee or top-of-deck effects on Ember cards.
6. **Ember does NOT benefit from patience.** All conditional energy must trigger on aggressive actions.

### Zephyr Constraints
1. **Zephyr does NOT accumulate static board presence.** No "if in play for 3+ turns" rewards.
2. **Zephyr does NOT profit from a full void.** Incidental void-filling is fine; void-size-matters is Ruin.
3. **Zephyr does NOT destroy permanently.** Dissolve is Ember; Zephyr displaces.
4. **Zephyr does NOT sacrifice for value.** Abandon is Ember/Ruin. Figment sacrifice bridge must be dual-coded.
5. **Zephyr does NOT generate energy via ramp.** Judgment-phase energy is Stone.
6. **Zephyr does NOT hoard cards.** Net-positive draw without discard rider is Tide.

### Stone Constraints
1. **Stone must NOT gain void interaction.** No Reclaim, no void-size-matters, no Dissolved triggers as primary effects.
2. **Stone must NOT gain primary card draw.** Draw only as secondary rider (e.g., Invoker of Myths draws when materializing a Warrior, but the primary identity is "Warrior-matters").
3. **Stone must NOT gain fast-speed play.** Stone is deliberately slow.
4. **Stone must NOT gain one-shot energy burst.** "Gain 6 energy" as an event is Ember. Stone's energy comes in drips.
5. **Stone must NOT gain self-sacrifice payoffs.** "When Warrior dissolved" triggers must be OPPONENT-caused only. "Abandon a Warrior: gain energy" is Ember.
6. **Stone must NOT gain bounce/flicker.** Board permanence means characters stay. Return-to-hand is Zephyr.

### Ruin Constraints
1. **Ruin does NOT generate energy.** Cheats costs through void, never produces energy directly. Exception: death-event energy (Volcanic Channeler) is acceptable; void-growth-to-energy is not.
2. **Ruin does NOT interact with opponent's hand or deck.** No hand disruption, no Prevent. Disruption is Tide.
3. **Ruin does NOT do fast-speed recursion.** Reclaim operates at sorcery speed.
4. **Ruin does NOT protect its board.** No shields, no "cannot be dissolved." Ruin's defense is recursion.
5. **Ruin does NOT care about board width.** Scaling is void-based and death-count-based, not ally-count-based.

### Cross-Resonance Boundary Rules
- Sacrifice outlets: Must be Ember-coded or dual (never mono-Tide, mono-Zephyr, or mono-Stone)
- Board wipes: Must be Ember-coded (Apocalypse is Ember)
- Recursion: Must involve Ruin coding (never mono-Tide or mono-Stone)
- Fast keyword: Must be Zephyr-coded or dual; never mono-Stone
- Energy ramp on Judgment: Must be Stone-coded; never mono-Ruin
- Draw-then-discard: Must be Zephyr-coded; pure draw is Tide

---

## 6. Card Design Assignments

### Agent A: 10 Signpost Duals

Each dual card signals a specific archetype pair. Based on the signpost gap analysis, Depths (1 signpost) and Gale (1 signpost) need the most help. The revised signpost policy calls for 3-4 duals per enemy pair.

| # | Archetype | Resonances | Recommended Vector(s) | Design Guidance |
|---|-----------|-----------|----------------------|-----------------|
| 1 | **Depths** | Tide+Stone | **V01 (Hand-Size-Matters)** | A character whose spark scales with hand size, or a Judgment trigger that rewards 6+ cards in hand. This is Depths' missing proactive finisher. The card says "I win by knowing more than you" without being reactive. Consider: "3 cost, Ancient, Tide+Stone. Judgment: If you have 5+ cards in hand, kindle 2." |
| 2 | **Depths** | Tide+Stone | **V02 (Prevent-Trigger)** | A character that rewards preventing. "3 cost, Mage, Tide+Stone. When you prevent a card, gain 1 energy and kindle 1." This transforms Prevent from pure denial into investment. Do NOT make the trigger too generous -- 1 energy per Prevent is already strong given 9 available Prevents. |
| 3 | **Depths** | Tide+Stone | **V05 (Unspent Energy) + V26 (Tax)** | "4 cost, Ancient, Tide+Stone. Opponent's characters cost 1 more. At end of turn, if you have 3+ energy, draw 1." Combines tax with energy-sink reward. Signals Tax Depths specifically. |
| 4 | **Gale** | Zephyr+Ember | **V14 (Hellbent)** | "2 cost, Fast Visitor, Zephyr+Ember. While you have no cards in hand, this character has +3 spark and 'When this character deals Judgment spark, draw 1.'" Rewards emptying your hand through fast deployment. The draw-on-Judgment prevents the player from being permanently helpless. Anti-synergy with Tide's hand accumulation creates clean separation. |
| 5 | **Gale** | Zephyr+Ember | **V17 (Sacrifice as Prevent) + V18 (Figment Sacrifice)** | "3 cost, Fast Survivor, Zephyr+Ember. Abandon an ally with 0 spark: Prevent a played card. When you prevent a card this way, draw 1." Converts figments into counterspells. Bridges Gale to Depths (control) while using Zephyr's token production. References Herald of the Last Light's pattern but expands it. |
| 6 | **Tempest** | Tide+Ember | **V06 (Event-Count-in-Void)** | Replace one of the "2+ events this turn" cards with: "3 cost, Ancient, Tide+Ember. This character's spark equals the number of events in your void." A Tide-coded version of Spirit of Smoldering Echoes. Scales with cumulative event play (rewarding Tempest's game-long event history) rather than single-turn storm count. Also usable in Depths as a control finisher. |
| 7 | **Eclipse** | Zephyr+Ruin | **V33 (Event Reclaim Engine)** | Replace Voidweave Dancer with: "3 cost, Explorer, Zephyr+Ruin. When you play an event from your void, copy it. Once per turn, discard a card: An event in your void gains Reclaim equal to its cost this turn." This gives Eclipse a unique engine -- event recycling -- that no other Ruin archetype wants. It uses Ashmaze Guide's Reclaim and feeds back into itself. |
| 8 | **Bedrock** | Stone+Ruin | **V32 (Void-Only Characters) + V30 (From-Void Materialized)** | "2 cost, Survivor, Stone+Ruin. 4 spark. You may only play this character from your void. When you materialize this character from your void, gain 2 energy." A smaller Revenant of the Lost that self-funds through Bedrock's void deployment. The Survivor typing creates an unexpected Undertow bridge. |
| 9 | **Crucible** | Stone+Ember | **V22 (Warrior Death Dividends) + V21 (Judgment Storm)** | Replace Warbond Sentinel concept with: "3 cost, Warrior, Stone+Ember. When an allied Warrior is dissolved by the opponent, each allied Warrior gains +1 spark. Judgment: If 2+ Judgment abilities triggered this phase, kindle 1." Creates the dilemma: leave Warriors alive (they accumulate Judgment value) or kill them (remaining Warriors get stronger). The Judgment-count rider rewards Stone's Judgment infrastructure. |
| 10 | **Basalt** | Zephyr+Stone | **V21 (Judgment Storm) + V34 (Burst Energy)** | Replace Verdant Packmother with: "3 cost, Spirit Animal, Zephyr+Stone. Judgment: Gain 1 point for each Judgment ability that triggered this phase (including this one). When you pay 4+ energy for an activated ability, draw 1." The Judgment-count payoff rewards Basalt's dense Judgment board. The activated-ability-draw rider rewards spending energy on Spiritbound Alpha and Mystic Runefish. NOT tribal-locked -- any deck with Judgment density benefits. |

### Agent B: Stone/Bedrock Specialist (4-6 cards)

Stone has a 9-card deficit. Agent B should design mono-Stone and Stone-dual cards that fill this gap.

**Priority vectors for Stone cards:**
1. **V21 (Judgment Storm)** -- 1 mono-Stone card: "Judgment: Gain 1 point for each other Judgment ability that triggered this phase." This is the most impactful new Stone axis. Put it on a cheap body (2 cost, 0 spark) that is itself a Judgment piece.
2. **V25 (Energy Overflow)** -- 1 mono-Stone card: "3 cost, Warrior, 1 spark. Judgment: You may pay 3 energy to kindle 2." A non-tribal energy sink that any Stone deck can use. The Warrior typing gives Crucible density without the card being tribal-locked (any deck can use the activated ability).
3. **V27 (Anchor Effect)** -- 1 mono-Stone card: "2 cost, Ancient, 0 spark. At the start of your turn, if this character has been on the battlefield since your last turn, kindle 1." The purest expression of Stone's permanence theme. Anti-synergy with Zephyr flicker (which resets the presence check). This card gets better the longer it survives -- quintessential Stone.
4. **V23 (Deployment Storm)** -- 1 mono-Stone card: "4 cost, Mage, 2 spark. When you play your 3rd character this turn, draw 2 and gain 2 energy." Rewards Stone's cost-reduction + ramp into multiple deployments. The threshold of 3 characters is reachable with Nexus Wayfinder but impossible without Stone's ramp.

**Priority vectors for Bedrock cards:**
5. **V32 (Void-Only)** -- additional void-only body (the signpost dual in slot 8 above is one; Bedrock may need a second). Consider: "5 cost, Ancient, Ruin. 7 spark. Only playable from void. Dissolved: Put this card into your void." Self-recurring void-only threat that reduces Bedrock's dependence on external recursion.
6. **V39 (Reclaim Cost Manipulation)** -- 1 Stone+Ruin card: "3 cost, Explorer, Stone+Ruin. Characters you play from your void cost 2 less." Bedrock-specific cost reducer. Irrelevant to Undertow (cheap Survivors) and Eclipse (cheap cycles). Gives Bedrock the economic bridge between Stone's ramp and Ruin's void deployment.

### Agent C: Cross-Pollination Specialist (4-6 cards)

Agent C designs cards that bridge non-adjacent archetype pairs. The top-5 non-adjacent bridges from Section 4 guide these assignments.

1. **Mirage -- Cinder bridge** via V15 + V19: "3 cost, Visitor, Ember+Tide. When an ally leaves play, gain 1 energy. When you materialize a character from your void, draw 1." This single card bridges Mirage (flicker = ally leaves + re-enters), Cinder (sacrifice = ally leaves; Reclaim = enters from void), and Bedrock (Reclaim from void). It is the single highest-value cross-archetype card possible. Dual Ember+Tide resonance ensures it lives in the Tempest pair slot but is poached by Mirage and Cinder.

2. **Depths -- Gale bridge** via V17: A fast character that extends Herald of the Last Light's pattern. "2 cost, Fast Outsider, Tide+Ember. Abandon this: Prevent a played character. When you prevent a card, your events cost 1 less this turn." The character-Prevent fills a gap in the Prevent suite (most Prevents target events, not characters). The cost-reduction rider rewards the Depths control plan. Dual Tide+Ember means Tempest can also use it as chain fuel.

3. **Tempest -- Basalt bridge** via V34: "2 cost, Event, Stone+Ember. Until end of turn, activated abilities cost 2 less. Draw 1." This lets Crucible slam Assault Leader's 4-energy ability for 2, lets Basalt chain Spiritbound Alpha + Mystic Runefish in one turn, and is a cheap cantrip for Tempest. The Ember component justifies burst-enabling; the Stone component justifies targeting activated abilities.

4. **Crucible -- Bedrock bridge** via V22: A Warrior with a Dissolved trigger that explicitly Reclaims. "3 cost, Warrior, Stone+Ruin. 2 spark. Dissolved: Reclaim a Warrior from your void." This bridges Crucible (Warrior density) and Bedrock (recursion) while giving Crucible a resilience narrative it currently lacks. When the opponent kills this Warrior, another Warrior returns. The Ruin half is the Dissolved trigger; the Stone half is the Warrior typing and permanence aspiration.

5. **Eclipse -- Basalt bridge** via V40: "2 cost, Spirit Animal, Zephyr+Ruin. 1 spark. When you discard this character, materialize it. Materialized: Foresee 1." A Spirit Animal that enters play when discarded. Eclipse wants it as a free body + discard trigger. Basalt wants it for Spirit Animal density + Materialized trigger. Mirage can flicker it for repeated Foresee. This is the format's most surprising cross-archetype card.

### Agent D: Modular Engines (4-6 cards)

Agent D designs cards that create context-dependent behavior -- cards whose function changes based on the archetype using them. These follow the mechanic critique's counter-patterns: modality, thresholds, conditional branches.

**Counter-patterns to reference from the mechanic critique:**
- Counter-Pattern 1 (Named-Type Choice)
- Counter-Pattern 2 (Threshold-Gated Mode Switch)
- Counter-Pattern 3 (Cross-Zone Scaling)
- Counter-Pattern 10 (Board-State-Reading Dual Mode)

1. **Threshold Mode Switch** (V01 + V14 combined): "3 cost, Character, Zephyr+Tide. Judgment: If you have 3 or fewer cards in hand, draw 2. If you have 4 or more, each ally gains +1 spark until end of turn." Gale/Eclipse use the draw mode (empty hand from fast deployment or discarding). Mirage/Depths use the spark mode (full hand from draw/bounce). The card is never dead but is always asking "which mode am I in?" This is the format's best draft-tension card because ALL 4 Zephyr archetypes AND Depths want it.

2. **Cross-Zone Scaling** (V19 + V15 + V10): "4 cost, Character, Neutral. This character's spark equals the number of cards that have changed zones this turn." Mirage counts flicker as 2 zone changes per character. Cinder counts sacrifice + recursion. Tempest counts events played (hand -> stack -> void = zone changes). Eclipse counts discard (hand -> void). Every archetype's core action contributes but at different rates. This card creates the most multi-archetype draft tension possible.

3. **Board-State Dual Mode** (V03 + V16): "2 cost, Event, Stone+Ember. Choose one: Dissolve an ally -- draw 3. OR Dissolve an enemy -- gain 3 energy." Cinder uses mode 1 (sacrifice for cards). Crucible/Depths uses mode 2 (removal + energy). Tempest agonizes (draw 3 feeds the chain vs. removal + energy). The correct mode depends on board state, making it a skill-testing card. Follows Counter-Pattern 10 exactly.

4. **Named-Type Choice** (V04 + V06): "3 cost, Character, Tide. Materialized: Name a card type (Character or Event). Draw cards until you hit that type; put all others into your void." Tempest names Event. Mirage names Character. Undertow uses it as self-mill regardless of choice. Bedrock might name Character to dump expensive bodies into void. Eclipse profits from void-filling. Follows Counter-Pattern 1.

5. **Conditional Recursion** (V30 + V31): "3 cost, Character, Ruin. 1 spark. When a card enters your void from any zone, you may pay 1 energy: Return a different card from your void to your hand." Eclipse triggers on every discard. Cinder triggers on every sacrifice. Undertow triggers on every mill. Tempest triggers on event resolution. Universal recursion engine with rate determined by archetype's void-filling speed. The 1-energy cost prevents infinite loops. This is Memory Unraveler's more powerful cousin.

6. **Scaling Cost with Variable Payoff** (V29 + V32): "X cost, Event, Ruin. Put the top X cards of your deck into your void. For each character put into void, materialize a figment. For each event, draw 1." Undertow pays high X. Basalt pays moderate X for figment triggers. Tempest wants event draws. Bedrock wants characters in void. Eclipse profits from void-filling. Follows Counter-Pattern 7.

### Agent E: Gap Fillers (4-6 cards)

Agent E addresses remaining holes not covered by Agents A-D.

**Remaining gaps to fill:**

1. **Crucible non-Warrior power card:** Crucible needs a non-Warrior card that is individually powerful enough to create a "purity vs. power" tension. "4 cost, Ancient, Stone. 3 spark. Judgment: Each ally gains +1 spark for each consecutive turn this character has been in play." This is V27 (Anchor Effect) on a high-spark body. Crucible drafters must decide: dilute Warrior density (Blade of Unity counts fewer) for a card that scales exponentially with game length. This is the single best card for reducing Crucible's 9/10 rails score.

2. **Undertow differentiation card:** Undertow's 12 auto-includes make the Survivor package feel mandatory. A non-Survivor void payoff competing for deck slots would create tension. "3 cost, Ancient, Ruin. Judgment: If 3+ cards entered your void this turn, gain 2 points." This is V29 (Void Velocity) as an Undertow payoff that competes with Survivor tribal slots. It rewards the milling behavior Undertow already does but on a non-Survivor body.

3. **Additional "leaves play" card:** V19 identified that only 1 card in 222 uses this trigger. "2 cost, Event, Zephyr. Until end of turn, when an ally leaves play, Foresee 1 and the next character you play costs 1 less." This is a Mirage tempo card (flicker chains get filtering + cost reduction), a Gale card (fast deployment + bounce triggers it), and a Cinder splash (sacrifice generates value). It lives in Zephyr, keeping the cycling pattern.

4. **Tempest variety card:** Replace one of the "2+ events this turn" cards with a fundamentally different storm payoff. "2 cost, Character, Tide+Ember. When you play an event that was copied this turn, gain 1 energy. Materialized: The next event you play this turn costs 1 less." This rewards event-copying specifically (Tempest's unique mechanic) rather than generic event-count. It creates a different deckbuilding decision: how many copy effects vs. how many events to copy.

5. **Hand-size card for Mirage:** V01 on a Zephyr body. "3 cost, Visitor, Zephyr. This character's spark equals the number of cards in your hand minus 3." In Mirage (average hand 6-7), this is a 3-4 spark body. In Gale (average hand 2-3), this is a 0-1 spark body. In Depths (average hand 5-6), this is a 2-3 spark body. Creates draft tension between Mirage, Basalt (which sometimes bounces to large hands), and Depths.

6. **Existing card exploitation:** Starcatcher ("when you play an event, gain 1 energy") is classified as Tempest but is secretly amazing in Depths (each Prevent is energy-neutral). Design a card that makes this bridge explicit without violating constraints. "2 cost, Event, Tide. Your next Prevent event this turn costs 0." This is a one-shot "free Prevent" that Depths uses for tempo and Tempest uses as a chain link that happens to protect against interaction.

---

## Appendix: Existing Cards That Partially Exploit Priority Vectors

For card designers' reference -- these existing cards already touch the listed vector and should inform (not duplicate) new designs.

| Vector | Existing Partial Exploiters |
|--------|-----------------------------|
| V01 (Hand-Size) | None (completely untapped) |
| V02 (Prevent-Trigger) | None (completely untapped) |
| V06 (Event-Void-Count) | Spirit of Smoldering Echoes (+1 spark per event entering void), Leyline Detonation (proposed) |
| V14 (Hellbent) | None (completely untapped) |
| V15 (Abandon-as-Flicker) | Angel of the Eclipse, The Bondweaver, Lumin-Gate Seer (all trigger on materialize regardless of source) |
| V16 (Kindle Voltron) | Infernal Ascendant (kindle 2 per abandon), Shardwoven Tyrant (uses spark for removal) |
| V17 (Sacrifice as Prevent) | Herald of the Last Light (abandon self to Prevent event) |
| V19 (Leaves Play) | Starlit Cascade (gain 2 energy when ally leaves) |
| V21 (Judgment Storm) | Surge of Fury (extra Judgment phase), Conduit of Resonance (materialize triggers Judgment) |
| V22 (Warrior Death) | Warbond Sentinel proposed ("dissolved Warrior: gain 1 energy") |
| V23 (Deployment Storm) | Illumination of Glory (1 point per card played -- not character-specific), Echoes of the Journey (draw per card played) |
| V29 (Void Velocity) | None (completely untapped -- void-size-matters exists but not velocity) |
| V30 (From-Void Materialized) | None (completely untapped) |
| V32 (Void-Only) | Revenant of the Lost (3 cost, 6 spark, void-only -- the only example) |
| V33 (Event Reclaim) | Ashlight Caller (give event Reclaim), Whisper of the Past (event Reclaim 0), Ashmaze Guide (discard gives Reclaim) |
| V34 (Burst into Activated) | Assault Leader (4 energy: +1 spark per Warrior), Spiritbound Alpha (4 energy: SA +2 spark) |

---

*End of Synergy Map. This document should be treated as the canonical reference for all Round 4 card design decisions.*
