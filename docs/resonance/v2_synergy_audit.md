# V2 Synergy Audit -- Agent 1 (The Synergy Auditor)

## Methodology

For every proposed card across all five design documents (31 total), I verify the multi-archetype claims by asking:

1. Would a drafter in the claimed archetype actually pick this card over available alternatives?
2. Are the "different reasons" claims genuinely distinct, or the same reason with different names?
3. Does the card create real draft tension -- multiple drafters from different archetypes competing for it?
4. Is the secondary/tertiary archetype claim honest, or is it padding?

**Rating Scale:**
- **Strong** -- genuinely multi-archetype; wanted by 2+ archetypes for mechanically distinct reasons; no archetype's claim is marginal
- **Acceptable** -- primary archetype is clear and strong; secondary archetype use is real but weaker
- **Weak** -- really one archetype; the "second archetype" claim is forced or marginal
- **Fail** -- on-rails design; screams exactly one archetype despite claims otherwise

---

## PART 1: Signpost Duals (Agent A -- 10 cards)

### Card 1: Tideweaver Sentinel (Mirage -- Tide+Zephyr, 3 cost)

**Claimed archetypes:** Mirage (primary), Basalt (secondary), Depths (tertiary)

**Audit:**
- **Mirage:** Genuinely excellent. A modal Materialized trigger on a 3-cost body is a dream flicker target. Both modes generate distinct value depending on game state. The bounce mode creates a self-fueling loop (return ally, gain energy, replay). This is an honest, strong Mirage card. **Confirmed.**
- **Basalt:** The bounce mode returning a Spirit Animal for re-materialize triggers is real. Basalt already wants Materialized-value bodies, and 2 energy back offsets the replay cost. The draw mode is fine but not special in Basalt. The claim is real but secondary. **Confirmed as secondary.**
- **Depths:** A 3-cost body with Materialized draw-2 is generically strong but Depths doesn't flicker, so this is a one-shot draw-2 on entry. That is fine but not exciting -- Knowledge Restored draws 3 for 5 energy, and many other draw options exist. Depths would play this but wouldn't fight for it. **Marginal tertiary, honest but weak.**

**Verdict: Strong.** Mirage and Basalt both have genuine, mechanically distinct reasons to want this. The modal design is real -- Mirage wants both modes in different game states, Basalt primarily wants the bounce mode. Draft tension between Mirage and Basalt drafters is real.

---

### Card 2: Abyssal Reclaimer (Undertow -- Tide+Ruin, 3 cost)

**Claimed archetypes:** Undertow (primary), Eclipse (secondary), Mirage (tertiary)

**Audit:**
- **Undertow:** Self-mill 3 + Reclaim 1 is exactly what Undertow wants. Survivor typing adds tribal density. The threshold is trivially met by the card itself (mill 3 = 3 void entries), so it is guaranteed-mill-3-Reclaim-1 every time it triggers. This is strong Undertow value. **Confirmed.**
- **Eclipse:** After a discard cycle, this card adds more void fuel and retrieves a key card. The threshold is met by discards + mill combined. This is real but Undertow wants it more. Eclipse would play it if available but wouldn't fight an Undertow drafter for it. **Confirmed as secondary but the "different reason" is somewhat thin -- both archetypes want it as a void-filler + retriever.**
- **Mirage:** As a flicker target, each flicker mills 3 and retrieves 1. This is a real flicker value engine, but it requires Mirage to splash Ruin, which is a significant cost. Mirage already has many excellent flicker targets. **Tertiary is honest but marginal.**

**Verdict: Acceptable.** Strong Undertow card. Eclipse use is real but similar in function (void-fill + retrieve). Mirage use is technically real but practically unlikely to be preferred over native Tide+Zephyr flicker targets. The "different reasons" between Undertow and Eclipse are more "same reason at different rates" than "mechanically distinct."

---

### Card 3: Basalt Warden (Basalt -- Zephyr+Stone, 3 cost)

**Claimed archetypes:** Basalt (primary), Crucible (secondary), Mirage (tertiary)

**Audit:**
- **Basalt:** The automatic mode switch based on board state is genuinely interesting. With 3+ allies (common in Basalt), gain 2 energy for activated abilities. With fewer, +1 spark. Basalt generates wide boards naturally, so the energy mode fires most of the time. **Confirmed.**
- **Crucible:** Warrior boards naturally hit 3+ allies. 2 energy fuels Assault Leader's activation. The "purity vs. power" tension (non-Warrior body diluting Blade of Unity) is real. A Crucible drafter would genuinely consider this card for its energy generation despite the non-Warrior typing. **Confirmed as strong secondary.**
- **Mirage:** Wide flicker boards often have 3+ allies, and energy funds more flicker plays. This is real but Mirage doesn't struggle for energy the way Basalt and Crucible do, and this card isn't a flicker target (no Materialized trigger). **Tertiary is marginal.**

**Verdict: Strong.** Both Basalt and Crucible have genuine, distinct reasons to want this. Basalt wants it as activated-ability fuel; Crucible wants it as non-Warrior energy generation that creates a real purity tension. Draft tension between adjacent archetypes is healthy.

---

### Card 4: Forgeborn Martyr (Crucible -- Stone+Ember, 3 cost)

**Claimed archetypes:** Crucible (primary), Cinder (secondary -- but self-corrected), Bedrock (tertiary)

**Audit:**
- **Crucible:** Judgment pump for all Warriors + death-draw-energy rider creates a genuine "damned if you do, damned if you don't" for opponents. This is an honest, powerful Crucible card. The Warrior lord function is primary but the death trigger adds real depth. **Confirmed.**
- **Cinder:** The designer self-corrected mid-explanation: the death trigger requires "dissolved by the opponent," NOT self-sacrifice. Cinder's self-sacrifice does NOT trigger it. This means Cinder benefits only defensively (if the opponent kills Warriors). Cinder would almost never run this card -- it requires Warrior density that Cinder doesn't naturally have, and the death trigger doesn't fire on sacrifice. **Claim is effectively retracted by the designer's own analysis.**
- **Bedrock:** "Warrior reanimator" sub-build is real but extremely narrow. Bedrock would need to be specifically running Warriors AND reanimation, which is a marginal build. The card makes Warrior death less painful, but Bedrock doesn't naturally field many Warriors. **Marginal.**

**Verdict: Weak.** This is effectively a Crucible-only card. The designer caught the Cinder problem mid-explanation but didn't redesign. Bedrock's claim requires a narrow sub-build. The Judgment pump + opponent-death-trigger is genuinely good design for Crucible but the multi-archetype claim is hollow. Still a well-designed Crucible card, just not genuinely multi-archetype.

---

### Card 5: Cinder Ritualist (Cinder -- Ember+Ruin, 3 cost)

**Claimed archetypes:** Cinder (primary), Undertow (secondary), Eclipse (tertiary)

**Audit:**
- **Cinder:** The sacrifice-kindle engine is strong. Abandon ally (enters void = kindle 1), retrieve a different character from void. Each loop nets 1 kindle. This is genuinely excellent Cinder design that creates a sustainable sacrifice engine. **Confirmed.**
- **Undertow:** Self-mill puts 3-6 cards into void per turn, each triggering kindle 1. This is a REAL and DIFFERENT use pattern -- Undertow doesn't sacrifice, it mills. A single Harvest the Forgotten (mill 3) generates kindle 3. The retrieval recovers milled Survivors. This is genuinely distinct from Cinder's use. **Confirmed as strong secondary for a genuinely different reason.**
- **Eclipse:** Each discard triggers kindle 1. Draw-discard cycles generate kindle passively. This is a third genuinely distinct trigger mechanism. Eclipse wouldn't prioritize this over its core cycling payoffs (Mother of Flames, Apocalypse Vigilante), but it would happily play it as supplementary value. **Confirmed as real tertiary.**

**Verdict: Strong.** This is one of the best-designed cards in the batch. Three archetypes want it for three genuinely different mechanical reasons: sacrifice loops (Cinder), mill volume (Undertow), discard cycling (Eclipse). The "when a card enters your void from any zone" trigger is the key design insight -- it fires on completely different actions in each archetype. Real draft tension across three archetypes.

---

### Card 6: Stormtide Oracle (Tempest -- Tide+Ember, 4 cost)

**Claimed archetypes:** Tempest (primary), Depths (secondary), Undertow (tertiary)

**Audit:**
- **Tempest:** Events-in-void scaling is a genuinely different Tempest axis than "2+ events this turn." By mid-game, 5-7 events in void makes this a 5-7 spark body. The retrieval decision (event in hand vs. spark on body) creates real tension. **Confirmed.**
- **Depths:** Control decks play many Prevent events that resolve to the void. By late game, 4-6 Prevents in void makes this a reliable finisher. This is GENUINELY different from Tempest's use -- Depths grows it through reactive play, not storm turns. A Depths drafter would seriously consider this card. **Confirmed as strong secondary for a genuinely different reason.**
- **Undertow:** Self-mill puts events into void incidentally. The body grows from milled events without casting them. This is a real but less exciting third use -- Undertow wants void-size-matters effects, and this only counts events, not all cards. Abomination of Memory is usually better for Undertow. **Marginal tertiary.**

**Verdict: Strong.** Tempest and Depths both have genuinely distinct, mechanically compelling reasons to want this card. The retrieval creates a real decision. Stormtrace Augur (from Agent E) is nearly identical in concept -- see interaction notes below.

---

### Card 7: Depthswatcher (Depths -- Tide+Stone, 4 cost)

**Claimed archetypes:** Depths (primary), Tempest (secondary), Basalt (tertiary)

**Audit:**
- **Depths:** Hand-size-gated kindle plus Prevent-trigger energy is a powerful combination for the control archetype. Both abilities feed Depths' natural game plan. This is the proactive finisher Depths has been missing. **Confirmed.**
- **Tempest:** Storm turns involve drawing many cards, and 5+ cards in hand is achievable mid-chain. But Tempest's hand fluctuates wildly -- full during setup, empty during storm. The kindle-2 fires on the Judgment AFTER a draw-heavy turn, not during the storm itself. This timing disconnect makes it less useful than claimed. The Prevent-trigger is mostly irrelevant in Tempest. **Overstated -- Tempest would rarely prioritize this over Tempest-native cards.**
- **Basalt:** "Stone-heavy Basalt boards hold up energy for activated abilities, sometimes accumulating cards." This is a stretch. Basalt doesn't naturally hold 5+ cards -- it deploys Spirit Animals aggressively. The Prevent trigger is irrelevant in Basalt. **Marginal at best.**

**Verdict: Acceptable.** Excellent Depths card, but the multi-archetype claims are weaker than stated. Tempest's use is real but situational. Basalt's claim is forced. This card is primarily a Depths signpost with marginal off-archetype appeal. That's fine for an enemy-pair signpost -- it doesn't need to be wanted everywhere.

---

### Card 8: Galerunner (Gale -- Zephyr+Ember, 2 cost)

**Claimed archetypes:** Gale (primary), Cinder (secondary), Eclipse (tertiary)

**Audit:**
- **Gale:** +2 spark at 2 or fewer cards in hand is the hellbent payoff Gale has been missing. Fast deployment empties the hand, making this a conditional 3-spark body for 2 energy. The abandon-0-spark-draw-1 converts spent figments into cards. Both abilities align with Gale's game plan. **Confirmed.**
- **Cinder:** Cinder can empty its hand through sacrifice costs and produces 0-spark figments. The abandon-for-draw is real sacrifice-outlet-that-draws. But the +2 spark from hellbent is less relevant in Cinder (which has bigger bodies and doesn't optimize for hand-empty). The draw is the real value, but Cinder has many draw sources from sacrifice already. **Real but not strongly compelling.**
- **Eclipse:** Eclipse discards to empty the hand. But Eclipse's hand fluctuates rapidly (draw-discard cycles), so the hellbent bonus would flicker on and off unpredictably. The 0-spark abandon is marginal in Eclipse. **Marginal.**

**Verdict: Acceptable.** Strong Gale card. Cinder's use is real but not first-pick material. Eclipse is a stretch. The hellbent mechanic creates clean anti-synergy with Tide (always has cards), which is good for resonance separation. The card succeeds as a Gale signpost even if its multi-archetype reach is modest.

---

### Card 9: Eclipse Weaver (Eclipse -- Zephyr+Ruin, 3 cost)

**Claimed archetypes:** Eclipse (primary), Tempest (secondary), Bedrock (tertiary)

**Audit:**
- **Eclipse:** The discard-to-Reclaim-event cycle is a genuine engine. Each cycle triggers discard payoffs AND generates card flow. This is exactly the event-recycling engine Eclipse needs. **Confirmed.**
- **Tempest:** Event Reclaim from void provides storm fuel recovery. After a storm turn that dumps events into the void, this card can retrieve key pieces for next turn. The discard cost is manageable when drawing 3-6 cards per turn. Tempest would genuinely consider this as event recovery. **Confirmed as secondary for a distinct reason (storm recovery vs. cycling engine).**
- **Bedrock:** Ashmaze Guide already does a similar thing; Eclipse Weaver is event-only and adds a draw reward. In a Ruin deck with expensive events, this grants them Reclaim. But Bedrock primarily cares about characters, not events. **Marginal.**

**Verdict: Acceptable.** Strong Eclipse card. Tempest's secondary use is real and distinct (recovery vs. cycling). Bedrock is a stretch. The "copy event from void" claim from the synergy map (V33) is not actually present in the card -- the card grants Reclaim, not copies. Good design but the tertiary claim is weak.

---

### Card 10: Bedrock Anchor (Bedrock -- Stone+Ruin, 2 cost)

**Claimed archetypes:** Bedrock (primary), Undertow (secondary), Crucible (tertiary)

**Audit:**
- **Bedrock:** Void-only character that self-funds (gain 2 energy = net 0 from void) and replaces itself (draw 1). The Judgment kindle rewards board permanence. This is exactly what Bedrock needs -- a cheap, self-funding void-only body. **Confirmed.**
- **Undertow:** Survivor typing + self-mill naturally puts this into the void. The 2-energy refund and draw make it a free inclusion. This is real -- any Undertow deck that mills this naturally would play it for free from the void. The Survivor typing adds genuine tribal density. **Confirmed as secondary -- it's a free inclusion, not a build-around.**
- **Crucible:** "Stone's ramp and Warrior bridges to Ruin" -- this is a stretch. Crucible doesn't run void-deployment effects. The energy generation and card draw are nice but Crucible can't play this card because it requires void deployment infrastructure that Crucible doesn't have. **Forced claim.**

**Verdict: Acceptable.** Strong Bedrock card with a genuine Undertow secondary (free value from natural mill). Crucible claim is dishonest. The Survivor typing as an Undertow bridge is a clever design touch.

---

## PART 2: Mono-Stone Cards (Agent B -- 5 cards)

### Card 11: Ironveil Watcher (Stone, 2 cost, 0 spark)

**Claimed archetypes:** Crucible, Basalt, Depths

**Audit:**
- **Crucible:** 3+ Judgment triggers per phase is normal for Crucible. This converts that into direct point generation. A 2-cost body generating 3+ points per Judgment is excellent rate. But it's not a Warrior, diluting tribal density -- creating genuine purity tension. **Confirmed.**
- **Basalt:** Spirit Animal boards fire 4+ Judgment triggers routinely. Ironveil Watcher generating 4+ points per turn while Spirit Animals handle energy is real value. **Confirmed for a different reason (different board composition reaching the same payoff).**
- **Depths:** "Fewer but higher-impact Judgment triggers" -- honestly, Depths doesn't field many Judgment-trigger bodies. This card is mediocre in Depths unless they specifically build for Judgment density, which isn't Depths' natural plan. **Overstated.**

**Verdict: Strong.** Crucible and Basalt both want this for genuinely different reasons (Warrior Judgment density vs. Spirit Animal Judgment density). Depths is marginal. The non-tribal payoff is well-designed -- it rewards Judgment density without requiring a specific subtype.

---

### Card 12: Stoneheart Veteran (Stone, 3 cost, Warrior, 1 spark)

**Claimed archetypes:** Crucible, Basalt, Bedrock

**Audit:**
- **Crucible:** Warrior body + repeatable energy sink. Crucible generates surplus energy and this converts it to kindle. The Warrior typing provides tribal density. The "you may pay 3" is a genuine decision each Judgment. **Confirmed.**
- **Basalt:** Spirit Animal ramp generates energy surplus. Stoneheart Veteran is a non-Spirit-Animal body (dilutes tribal density) but the kindle effect is powerful. Basalt would consider it but not prioritize it over Spirit Animal tribal pieces. **Real but secondary.**
- **Bedrock:** "Backup energy sink when the reanimation plan is disrupted" -- this is extremely marginal. Bedrock doesn't generate enough energy surplus to make a 3-energy-for-kindle-2 activation worthwhile on a regular basis. **Forced.**

**Verdict: Acceptable.** Good Crucible card with real (but weaker) Basalt secondary. Bedrock claim is weak. The Warrior typing is important for Crucible density but the ability is subtype-agnostic, which is good design.

---

### Card 13: Oathbound Sentinel (Stone, 2 cost, 0 spark, Common)

**Claimed archetypes:** Depths, Crucible, Basalt

**Audit:**
- **Depths:** A ticking clock that kindles every turn it survives. Behind Prevent protection, this accumulates significant spark over time. The opponent must choose: waste removal on a 0-spark body or let kindle accumulate. Gives Depths a proactive finisher axis. **Confirmed.**
- **Crucible:** Warrior boards protect this body through sheer width. The non-Warrior typing creates a genuine draft decision. **Real but secondary -- Crucible has many kindle sources already and this is slower than most.**
- **Basalt:** Spirit Animal boards are wide enough to protect it. But Basalt's Zephyr half bounces and flickers creatures, which RESETS the "since your last turn" check. The designer explicitly notes this anti-synergy as a feature. **Marginal in Basalt -- the anti-synergy is real.**

**Verdict: Acceptable.** Strong Depths card. The "stayed in play" mechanic is the first of its kind and creates excellent anti-synergy with Zephyr (Stone's enemy). Crucible secondary is real. Basalt is rightly identified as an anti-synergy trap.

---

### Card 14: Vanguard of the Summit (Stone, 4 cost, Mage, 2 spark)

**Claimed archetypes:** Crucible, Basalt, Depths

**Audit:**
- **Crucible:** With Nexus Wayfinder and cheap Warriors, hitting 3 characters in a turn is achievable. The draw-2 + gain-2 reward is powerful. The Mage subtype creates purity-vs-power tension. **Confirmed.**
- **Basalt:** Cheap Spirit Animals (1-cost Ebonwing, Driftcaller, Dawnprowler) plus cost reduction make the threshold reachable. Draw 2 finds more Spirit Animals; energy funds activations. **Confirmed for a different reason (different cheap bodies, different use of the reward).**
- **Depths:** "Harder to trigger" -- the designer admits this. Depths plays few cheap characters. Reaching 3 deployments in one turn is unlikely without extreme ramp. The reward is powerful but the threshold is rarely met. **Honest but marginal.**

**Verdict: Strong.** Crucible and Basalt both have genuine, mechanically distinct paths to triggering this card. The threshold is high enough that it requires real deckbuilding commitment, not just including it passively. The Mage subtype creating Crucible purity tension is excellent design.

---

### Card 15: Deepvault Warden (Stone, 3 cost, Explorer, 1 spark)

**Claimed archetypes:** Bedrock, Crucible, Undertow

**Audit:**
- **Bedrock:** This is transformative. Revenant of the Lost (3 cost, 6 spark) becomes a 1-cost play from void. Echoing Monolith's Reclaim cost drops by 2. This is the economic bridge Bedrock has been missing. **Confirmed -- this is a Bedrock core card.**
- **Crucible:** Ashen Avenger (3 cost, Warrior) becomes a 1-cost re-deployment from void. Grim Reclaimer's Warrior Reclaim becomes cheaper. This is real -- Crucible has Warrior recursion through Ruin bridges, and this makes that sub-strategy viable. **Confirmed for a genuinely different reason (Warrior recursion vs. reanimation targets).**
- **Undertow:** Kindred Sparks' void cost drops. But Undertow plays most of its Survivors from hand, not void. The cost reduction is marginal for Undertow's typical play patterns. **Real but not compelling -- Undertow wouldn't fight for this.**

**Verdict: Strong.** Bedrock gets its missing economic engine. Crucible's Warrior recursion sub-build gains a meaningful enabler. Two genuinely distinct use cases. The "tools, not labels" philosophy is well-executed here.

---

## PART 3: Cross-Pollination Cards (Agent C -- 6 cards)

### Card 16: Ashen Threshold (Ember, 3 cost, Outsider, 1 spark)

**Claimed archetypes:** Mirage, Cinder, Bedrock

**Audit:**
- **Mirage:** Every flicker triggers "ally leaves play" for energy. If flicker routes through void (some flicker effects banish then materialize, but the standard pattern is banish -> return, not void -> return), then "materialize from void" would NOT fire. The claim that flicker triggers the second ability depends on whether Dreamtides' flicker goes through void or through banish zone. Standard flicker (Passage Through Oblivion, Portal of Twin Paths) banishes then materializes -- the character goes to BANISH, not VOID. So the "materialize from void" trigger does NOT fire on standard flicker. The "ally leaves play" trigger DOES fire. **Partially confirmed -- energy from flicker is real, but the draw-from-void-materialize may not fire on flicker. Mirage still wants the energy.**
- **Cinder:** Every sacrifice triggers "ally leaves play" for energy. Every Reclaim triggers "materialize from void" for draw. Both halves fire. This is a genuine sacrifice-recursion engine. **Confirmed.**
- **Bedrock:** "Materialize from void" draws a card on every reanimation. This is real and distinct from Cinder's use. **Confirmed for a different reason.**

**Verdict: Strong.** Despite the flicker-through-void question for Mirage, the card is genuinely wanted by all three archetypes for different reasons: Mirage wants the energy from flicker, Cinder wants the sacrifice-recursion engine, Bedrock wants the reanimation draw. The Ember coding forcing Mirage to splash is excellent draft tension.

---

### Card 17: Voidthorn Sentinel (Ruin, 2 cost, Survivor, 1 spark)

**Claimed archetypes:** Depths, Gale, Cinder

**Audit:**
- **Depths:** A body-based character Prevent that doesn't require holding up mana for a fast event. Currently Depths has only Cragfall as a character-targeted Prevent. This fills a real gap in the Prevent suite. **Confirmed.**
- **Gale:** Fast bodies that double as counterspells align with Gale's tempo plan. Deploy cheaply, hold up the Prevent threat. But this card is NOT fast -- it's a regular Character. Gale can't deploy it at instant speed. The designer mentions "via Moonlit Dancer" (which gives hand characters fast), but that's a conditional combo, not a baseline interaction. **Overstated -- without Moonlit Dancer, Gale can't deploy this reactively, significantly reducing its tempo value.**
- **Cinder:** Self-sacrifice as a counterspell + void-filling. The Survivor typing triggers Survivor-matters effects. This is real but narrow -- Cinder typically wants sacrifice that generates value (kindle, draw, spark), not Prevent. **Real but not compelling.**

**Verdict: Acceptable.** Strong Depths card. Gale's claim is weaker than stated (requires Moonlit Dancer for the tempo play). Cinder is marginal. The Depths-Gale bridge is the intended high-value bridge, but the Gale side is conditional.

---

### Card 18: Resonance Siphon (Neutral, 2 cost, Fast Event)

**Claimed archetypes:** Basalt, Tempest, Crucible

**Audit:**
- **Basalt:** Spiritbound Alpha's 4-energy becomes 2-energy, Mystic Runefish's 3-energy becomes 1-energy. This is a genuine tempo accelerator that lets Basalt fire expensive activations earlier. **Confirmed.**
- **Tempest:** The cantrip makes it storm-playable. Minstrel of Falling Light's 3-energy draw becomes 1-energy draw, which is a strong storm chain link. But "until end of turn" means you need to have both this card AND the activated ability in the same turn. In Tempest, you're usually spending energy on events, not activations. **Real but situational.**
- **Crucible:** Assault Leader's 4-energy becomes 2-energy. This is a significant tempo gain. But Crucible only has 1-2 expensive activations, so the card is less versatile here. **Real but narrow.**

**Verdict: Acceptable.** Basalt is the clear primary. Crucible and Tempest have real but narrower use cases. The Neutral coding is correct -- this is a tool that scales with activated-ability density. The Tempest-Basalt non-adjacent bridge is real but the Tempest side is weaker than claimed.

---

### Card 19: Kindlespark Harvester (Stone, 3 cost, Ancient, 0 spark)

**Claimed archetypes:** Crucible, Cinder, Depths

**Audit:**
- **Crucible:** 5-8 spark on leftmost is normal by midgame. Removing 3 for targeted removal while getting 2 back is a strong conversion. Creates genuine scoring-vs-removal tension. **Confirmed.**
- **Cinder:** Infernal Ascendant builds 6-12 spark on leftmost. Converting that into removal is a genuinely new axis for Cinder -- kindle has been one-dimensional (scoring only) until now. But this is mono-Stone, forcing Cinder to splash outside Ember+Ruin, which is a significant drafting cost. **Real and genuinely distinct reason, but splash cost is high.**
- **Depths:** Any kindle source accumulates over many turns. Depths typically doesn't build massive kindle towers, but if running other kindle effects (from Stone), this becomes viable. **Marginal -- Depths doesn't naturally generate much kindle.**

**Verdict: Acceptable.** "Spark as spendable resource" is a genuinely novel mechanic. Crucible is the clear primary. Cinder's use is real and mechanically distinct but requires a cross-resonance splash. Depths is marginal. The Stone coding maximizing Cinder draft tension is excellent design.

---

### Card 20: Echoing Departure (Zephyr, 2 cost, Event, Common)

**Claimed archetypes:** Mirage, Cinder, Gale

**Audit:**
- **Mirage:** Every flicker is a "leave play" event. With 3 allies flickered, 3 cost reductions + 3 Foresees is powerful. This is a genuine flicker payoff. **Confirmed.**
- **Cinder:** Every sacrifice is a "leave play" event. Sacrifice-heavy turns generate cost reduction + Foresee. Cinder has zero Foresee access normally, so this is a genuinely novel cross-resonance capability. But it's Zephyr-coded, requiring Cinder to splash outside Ember+Ruin. **Real and genuinely distinct, but splash cost is high.**
- **Gale:** Bounce effects trigger it. Bounce -> cost reduction -> replay cheaper -> bounce again. This creates a tempo chain. **Confirmed.**

**Verdict: Strong.** All three claims are mechanically honest and genuinely distinct: flicker chains (Mirage), sacrifice payoff with novel Foresee access (Cinder), bounce tempo chains (Gale). The "until end of turn" scoping prevents it from being permanent. The Zephyr coding creates appropriate splash costs for Cinder.

---

### Card 21: Risen Warden (Ruin, 4 cost, Warrior, 2 spark)

**Claimed archetypes:** Crucible, Bedrock, Cinder

**Audit:**
- **Crucible:** Warrior body for tribal density. Dissolved trigger Reclaims another Warrior. This creates the "Warrior Death Dividend" -- leaving it alive gives lord scaling, killing it returns another Warrior. **Confirmed.**
- **Bedrock:** Void-only character (like Revenant) that self-perpetuates through Dissolved trigger. The Dissolved trigger can target itself if it's the only Warrior in void. Self-recurring void threat. **Confirmed for a genuinely different reason (persistent void threat vs. tribal density).**
- **Cinder:** Sacrifice loops send it to void, Dissolved trigger chains to Reclaim another Warrior. But Cinder doesn't typically run Warrior density -- without other Warriors in void, the Dissolved trigger has nothing to reclaim except itself. **Real but requires Warrior-Cinder hybrid build, which is narrow.**

**Verdict: Strong.** Crucible and Bedrock both have genuinely distinct, strong reasons. A Warrior in Ruin is a deliberate cross-resonance signal. The void-only restriction creates a genuine deckbuilding puzzle. Cinder is narrower but real in the right build.

---

## PART 4: Modular Engine Cards (Agent D -- 5 cards)

### Card 22: Dreamtide Cartographer (Neutral, 3 cost, Explorer, 1 spark)

**Claimed archetypes:** Gale, Eclipse, Crucible, Basalt, Cinder, Depths, Tempest (7 claimed)

**Audit:**
- **Gale:** Fast deployment empties hand by turn 3-4. Draw-2 per Judgment refuels. This is a premium Gale card. **Confirmed.**
- **Eclipse:** Hand size oscillates wildly in Eclipse. Both modes are useful depending on phase. **Confirmed.**
- **Crucible:** Maintains hand through draw effects, so spark mode pumps all allies. With 5 Warriors, +5 total spark per Judgment is massive. **Confirmed.**
- **Basalt:** Spirit Animals are low-spark. +1 to each is strong. Stacks with Spiritbound Alpha. **Confirmed.**
- **Cinder:** Empties hand through sacrifice costs. Draw mode refuels. **Confirmed.**
- **Depths:** Accumulates cards. Spark mode pumps control bodies. **Confirmed.**
- **Tempest:** Post-storm Judgment draws 2 to rebuild. **Real but marginal -- Tempest rarely has this body survive.**

But wait -- the threshold of 3 cards is low. In practice, most aggressive decks (Gale, Cinder, Tempest post-storm) will be at 3 or fewer, meaning the draw mode fires. Most board-building decks (Crucible, Basalt, Depths) will be at 4+, meaning the spark mode fires. The mode switch is genuinely determined by deck archetype, not by player choice. This is excellent design.

However, there's a concern: is this card TOO good? A 3-cost body that either draws 2 per Judgment or gives +1 spark to all allies is above-rate for any archetype. At Uncommon, this could be format-warping. The designer claims 7 archetypes want it -- that makes it a first-pick in every draft.

**Verdict: Strong (but potential balance concern).** Genuinely multi-archetype with authentic mechanically distinct reasons. The threshold-gated mode switch works exactly as intended. The risk is that it's TOO universally desirable, creating a "best card in the format" rather than "interesting draft tension."

---

### Card 23: Nexus of Passing (Neutral, 4 cost, Ancient, * spark)

**Claimed archetypes:** Mirage, Cinder, Tempest, Eclipse, Undertow (5 claimed)

**Audit:**
- **Mirage:** 6-10 spark on a flicker turn. Each flicker = 2 zone changes. This is a massive finisher. **Confirmed.**
- **Cinder:** 5-8 spark on a sacrifice turn. Each abandon = 1 zone change, each Reclaim = 1. **Confirmed for a genuinely different mechanism.**
- **Tempest:** 8-12 spark on a storm turn. Each event = 2 zone changes (hand -> play -> void). This is Tempest's best single-turn finisher. **Confirmed for yet another different mechanism.**
- **Eclipse:** 4-7 spark from draw-discard cycling. **Confirmed.**
- **Undertow:** 4-8 spark from mill. **Confirmed.**

Every claim is honest. Five archetypes generate zone changes through five completely different mechanisms. However:

**Balance concern:** This card's spark is only measured at Judgment, meaning it needs to survive from deployment to the next Judgment phase. On a big combo turn, the player generates massive zone changes but the spark is only counted at Judgment -- after the turn, during the scoring phase. Wait, if the zone changes happen during the SAME turn as Judgment, then the card is measuring the current turn's activity. On turns where you do nothing, it's 0 spark. This is a significant constraint that balances it.

But in Tempest, playing 5 events = 10+ zone changes = 10+ spark body. For 4 energy. That seems very strong.

**Verdict: Strong (but balance concern).** The most genuinely multi-archetype card in the entire batch. Five archetypes, five different mechanisms, all honest. The risk is that Tempest or Mirage can generate absurd zone-change counts, making this a 10-15 spark body for 4 energy on combo turns. The "must survive to Judgment" constraint helps but may not be enough.

---

### Card 24: Crucible of the Commons (Stone, 3 cost, Visitor, 2 spark)

**Claimed archetypes:** Crucible, Basalt, Cinder, Mirage, Undertow (5 claimed)

**Audit:**
- **Crucible:** Pumps 0-spark utility Warriors (Wolfbond, Ethereal Trailblazer, Company Commander, Spirit Field Reclaimer). Typically +3-4 spark per Judgment. Creates anti-synergy with Skyflame Commander (which pushes Warriors above the 1-spark threshold). Genuine sequencing puzzle. **Confirmed.**
- **Basalt:** Twelve of eighteen Spirit Animals have exactly 1 spark. This pumps every one of them. Stacks with Spiritbound Alpha. **Confirmed for a genuinely different board composition.**
- **Cinder:** Figment tokens (0 spark), sacrifice fodder bodies at 0-1 spark. +2-3 spark per Judgment. Creates a timing tension: sacrifice the figment now or keep it for the spark boost. **Confirmed for yet another different reason.**
- **Mirage:** Low-spark utility bodies + figments gain +1. **Confirmed.**
- **Undertow:** Many Survivors are 1-spark utility bodies. +2-4 spark. **Confirmed.**

**Verdict: Strong.** All five claims are honest. The "spark 1 or less" threshold creates genuine board-composition evaluation that differs by archetype. The anti-synergy with lord effects (which push bodies above the threshold) creates real deckbuilding tension. Well-designed "tools not labels" card.

---

### Card 25: Archivist of Vanished Names (Tide, 3 cost, Mage, 1 spark)

**Claimed archetypes:** Tempest, Mirage, Undertow, Bedrock, Eclipse (5 claimed)

**Audit:**
- **Tempest:** Names Event to guarantee chain fuel. In a 60% event deck, mills 0-2 characters to find the event. The guaranteed event-finding is powerful for storm. **Confirmed.**
- **Mirage:** Names Character to find flicker targets. Milled events go to void. As a Materialized trigger, this is a repeatable flicker target (each flicker searches again). **Confirmed for a genuinely different choice (naming Character vs. Event).**
- **Undertow:** Doesn't care which type -- names the rarer type to maximize mill. This is genuinely a different use pattern (primary value is the milling, not the found card). **Confirmed for yet another reason.**
- **Bedrock:** Names Character to find reanimation targets or mill characters to void. **Confirmed -- similar to Undertow's use but different destination (void-filling for reanimation vs. void-filling for void-size).**
- **Eclipse:** Names Event to find cycling pieces. Milled characters go to void. **Confirmed -- similar to Tempest but in a cycling context rather than storm.**

The "reveal until" mechanic has variance -- in extreme cases, you could mill many cards. This is self-balancing (you're decking yourself) but creates exciting high-variance moments.

**Verdict: Strong.** The Named-Type Choice creates genuinely different evaluations by archetype. Five archetypes, each naming differently and valuing the mill vs. the found card differently. This is one of the best-designed cards in the batch -- the simplest possible modal (binary choice) creates the most archetype-dependent evaluations.

---

### Card 26: Ember of Recurrence (Ruin, 3 cost, Synth, 1 spark)

**Claimed archetypes:** Eclipse, Cinder, Undertow, Tempest, Bedrock (5 claimed)

**Audit:**
- **Eclipse:** Every discard triggers it. 3-5 triggers per turn in active cycling. Perpetual motion engine where every discard both fills void and retrieves from it. **Confirmed.**
- **Cinder:** Every abandon triggers it. Retrieve sacrifice fodder to sacrifice again. **Confirmed for a genuinely different trigger mechanism.**
- **Undertow:** Every mill triggers it. 4-8 triggers per turn during mill turns. Cherry-picks the best milled card. **Confirmed for yet another mechanism.**
- **Tempest:** Every event resolution triggers it. Retrieve key events for replay. But 1 energy per retrieval competes with event-playing energy. **Confirmed but weaker -- Tempest needs energy for events, not for retrieval.**
- **Bedrock:** 2-3 triggers per turn from self-mill and discard. Retrieves reanimation targets. **Real but marginal.**

**Potential overlap concern with Cinder Ritualist (Card 5):** Both cards trigger on "card enters void from any zone" and both retrieve from void. Cinder Ritualist adds kindle; Ember of Recurrence costs 1 energy per retrieval. They could both be in the same deck, creating a redundant recursion engine. This might be too much void-retrieval density.

**Verdict: Strong (with redundancy concern).** Genuinely multi-archetype with authentic distinct trigger mechanisms. The 1-energy cost is the correct throttle. But combined with Cinder Ritualist, the void-retrieval axis may be oversaturated.

---

## PART 5: Gap Filler Cards (Agent E -- 5 cards)

### Card 27: Ironbark Sentinel (Stone, 4 cost, Ancient, 3 spark)

**Claimed archetypes:** Crucible, Depths, Basalt (anti-synergy)

**Audit:**
- **Crucible:** At 4 cost, 3 spark, Ancient, this is individually more powerful than most Warriors. The +1 spark to each persisting ally per Judgment is massive in Crucible (5 Warriors = +5). The non-Warrior typing creates the "purity vs. power" tension. **Confirmed -- this is THE card for reducing Crucible's 9/10 rails score.**
- **Depths:** Control bodies persist across many turns. +1 spark per Judgment to each survivor turns defensive bodies into growing threats. This gives Depths a proactive win condition. **Confirmed for a genuinely different reason (scaling control finisher vs. Warrior army pump).**
- **Basalt (anti-synergy trap):** Zephyr flicker resets the "since your last turn" check. The designer explicitly flags this as anti-synergy. **Correctly identified as NOT a Basalt card.**

**Verdict: Strong.** Crucible and Depths both have genuinely distinct, compelling reasons. The anti-synergy with Zephyr is well-designed. The unprecedented "stayed in play" mechanic creates new strategic territory. However, this is very similar to Oathbound Sentinel (Card 13) -- both reward continuous board presence in Stone. Two "anchor effect" cards might be redundant. See interaction notes.

---

### Card 28: Tidechannel Observer (Ruin, 3 cost, Ancient, 1 spark)

**Claimed archetypes:** Undertow, Cinder, Eclipse

**Audit:**
- **Undertow:** Deliberately competes with Survivor tribal slots -- non-Survivor body that rewards what Undertow already does (void velocity). This creates the exact tension the deckbuilder critique demanded: a non-Survivor void payoff that dilutes tribal density. 3+ cards entering void is trivially met by Undertow (mills 4-8 per turn). **Confirmed -- excellent Undertow design.**
- **Cinder:** 3+ void entries from sacrifice turns is achievable on big sacrifice turns (Desperation + 3 abandons). Points + kindle on an Ancient body. **Confirmed for a genuinely different trigger mechanism.**
- **Eclipse:** 3+ discards per turn is achievable with Fragments of Vision (discard 2 from a draw-3-discard-2) plus other discard effects. **Confirmed for yet another mechanism.**

The "3+ cards entering void this turn" threshold is the key differentiator from Abomination of Memory (which rewards total void size). This velocity metric uniquely separates Undertow (trivial) from Bedrock (almost never reaches 3/turn), directly addressing the Ruin bottleneck.

**Verdict: Strong.** All three claims are honest. The velocity threshold is an excellent design innovation that differentiates void-filling archetypes by RATE rather than VOLUME. The non-Survivor typing creating Undertow draft tension is exactly the kind of design the format needs.

---

### Card 29: Fading Resonant (Zephyr, 2 cost, Visitor, 1 spark)

**Claimed archetypes:** Mirage, Gale, Cinder

**Audit:**
- **Mirage:** Every flicker triggers cost reduction. 2-3 flickers = 2-3 cost reductions for the next character play. Creates a tempo chain. **Confirmed.**
- **Gale:** Bounce effects trigger it. Each bounce reduces the next play. **Confirmed for a different mechanism (bounce vs. flicker).**
- **Cinder:** Every sacrifice triggers it. 2-3 sacrifices = 2-3 cost reductions. But it's Zephyr-coded, requiring Cinder to splash. **Real but requires cross-resonance splash.**

**Overlap concern with Echoing Departure (Card 20):** Both are Zephyr events/characters that trigger on "ally leaves play" and provide cost reduction. Echoing Departure is an event (one-shot per turn); Fading Resonant is a permanent (persistent). Having both creates a "leaves play" archetype sub-theme in Zephyr. Is this too much? Given that only 1 card in 222 used this trigger previously (Starlit Cascade), going from 1 to 3 (Starlit Cascade, Echoing Departure, Fading Resonant) is reasonable density for a mechanic that covers the most frequent Zephyr zone change.

**Verdict: Strong.** All three claims are honest and mechanically distinct. The "ally leaves play" mechanic is underexploited (1 in 222) and deserves multiple payoffs. The 2-cost body is appropriately costed for a utility engine.

---

### Card 30: Stormtrace Augur (Tide, 3 cost, Mage, 0 spark)

**Claimed archetypes:** Tempest, Depths, Eclipse

**Audit:**
- **Tempest:** Events in void = spark. By mid-game, 6-10 events in void = 6-10 spark body for 3 energy. This is the cheapest high-spark body in the format in its best case. **Confirmed.**
- **Depths:** Prevent events accumulate in void. 5 Prevents = 5 spark for 3 energy. This is a genuine Depths finisher that emerges from the reactive game plan. **Confirmed for a genuinely different reason (Prevents vs. storm events).**
- **Eclipse:** Draw-discard cycling puts events into void. 4-6 spark is respectable. **Confirmed.**

**CRITICAL OVERLAP: Stormtrace Augur is nearly identical to Stormtide Oracle (Card 6).** Both are characters whose spark equals the number of events in your void. Stormtide Oracle costs 4, is Rare, Tide+Ember, and adds a Judgment event-retrieval ability. Stormtrace Augur costs 3, is Rare, mono-Tide, and has no additional ability. These two cards occupy the same design space -- "event void count = spark." Having both in the format is redundant and would oversaturate this specific axis.

**Verdict: Acceptable (but REDUNDANT with Stormtide Oracle).** The multi-archetype claims are all honest. But one of these two cards should be cut or redesigned. They exploit the exact same vector (V06: Event-Count-in-Void) with the exact same mechanic. Stormtide Oracle is the more interesting design (has the retrieval decision), so Stormtrace Augur should be the cut candidate.

---

### Card 31: Duskwatch Warden (Tide, 2 cost, Outsider, 2 spark)

**Claimed archetypes:** Depths, Gale, Tempest

**Audit:**
- **Depths:** Prevents become tempo plays -- after Preventing, the next event costs 2 less, effectively refunding the Prevent cost. This converts reactive denial into proactive development. **Confirmed.**
- **Gale:** This is where the "hidden gem" claim gets tested. Gale runs 2-3 fast Prevents (Abolish, Ripple of Defiance) to trigger Musicians. Duskwatch Warden adds a second reward layer: Prevent -> discounted next event. But Gale runs very few Prevents -- is 2-3 triggers per game enough to justify a 2-cost body? Gale would need to actively splash Prevents to make this work, which dilutes the fast-tempo plan. **Real but the rate of triggers is low enough that Gale wouldn't prioritize this.**
- **Tempest:** 1-2 defensive Prevents per game. The cost reduction on the next event extends the chain. But at 1-2 triggers per game, this is marginal Tempest value. **Marginal.**

**Verdict: Acceptable.** Strong Depths card. Gale's hidden use is real but frequency-dependent -- 2-3 Prevents per game is not enough to make this a high-priority Gale pick. Tempest is marginal. The card succeeds as a Depths tempo tool but its multi-archetype reach is modest.

---

## SUMMARY TABLE

| # | Card Name | Rating | Primary Archetype | Genuine Secondary | Genuine Tertiary | Notes |
|---|-----------|--------|-------------------|-------------------|------------------|-------|
| 1 | Tideweaver Sentinel | **Strong** | Mirage | Basalt | Depths (marginal) | Modal design creates real decisions |
| 2 | Abyssal Reclaimer | **Acceptable** | Undertow | Eclipse | Mirage (marginal) | Secondary uses similar reason |
| 3 | Basalt Warden | **Strong** | Basalt | Crucible | -- | Threshold mode switch is excellent |
| 4 | Forgeborn Martyr | **Weak** | Crucible | -- | -- | Cinder claim self-retracted; Bedrock marginal |
| 5 | Cinder Ritualist | **Strong** | Cinder | Undertow, Eclipse | -- | Three archetypes, three mechanisms |
| 6 | Stormtide Oracle | **Strong** | Tempest | Depths | -- | Retrieval decision is genuine |
| 7 | Depthswatcher | **Acceptable** | Depths | Tempest (marginal) | -- | Tempest/Basalt claims overstated |
| 8 | Galerunner | **Acceptable** | Gale | Cinder (weak) | -- | Hellbent creates clean resonance separation |
| 9 | Eclipse Weaver | **Acceptable** | Eclipse | Tempest | -- | Event-recycling engine is well-designed |
| 10 | Bedrock Anchor | **Acceptable** | Bedrock | Undertow | -- | Crucible claim is forced |
| 11 | Ironveil Watcher | **Strong** | Crucible/Basalt | Depths (marginal) | -- | Non-tribal Judgment scaling |
| 12 | Stoneheart Veteran | **Acceptable** | Crucible | Basalt (weak) | -- | Bedrock claim is forced |
| 13 | Oathbound Sentinel | **Acceptable** | Depths | Crucible | -- | First "stayed in play" reward; anti-Zephyr |
| 14 | Vanguard of the Summit | **Strong** | Crucible | Basalt | -- | Threshold creates planning puzzle |
| 15 | Deepvault Warden | **Strong** | Bedrock | Crucible | -- | Transformative for Bedrock economy |
| 16 | Ashen Threshold | **Strong** | Cinder | Mirage, Bedrock | -- | Zone-change triggers across 3 mechanisms |
| 17 | Voidthorn Sentinel | **Acceptable** | Depths | Gale (conditional) | Cinder (marginal) | Gale claim requires Moonlit Dancer |
| 18 | Resonance Siphon | **Acceptable** | Basalt | Tempest, Crucible | -- | Activated-ability density dependent |
| 19 | Kindlespark Harvester | **Acceptable** | Crucible | Cinder (splash cost) | -- | Novel "spark as resource" mechanic |
| 20 | Echoing Departure | **Strong** | Mirage | Cinder, Gale | -- | Fills glaring "leaves play" hole |
| 21 | Risen Warden | **Strong** | Crucible | Bedrock | Cinder (narrow) | Warrior-in-Ruin cross-resonance signal |
| 22 | Dreamtide Cartographer | **Strong** | Many (7 claimed) | All confirmed | -- | Balance concern: too universally good? |
| 23 | Nexus of Passing | **Strong** | Many (5 claimed) | All confirmed | -- | Balance concern: Tempest ceiling too high? |
| 24 | Crucible of the Commons | **Strong** | Many (5 claimed) | All confirmed | -- | Excellent non-tribal width payoff |
| 25 | Archivist of Vanished Names | **Strong** | Many (5 claimed) | All confirmed | -- | Binary choice = archetype-dependent evaluation |
| 26 | Ember of Recurrence | **Strong** | Eclipse, Cinder | Undertow, Tempest | Bedrock | Redundancy concern with Cinder Ritualist |
| 27 | Ironbark Sentinel | **Strong** | Crucible | Depths | -- | THE purity-vs-power card; similar to Oathbound |
| 28 | Tidechannel Observer | **Strong** | Undertow | Cinder, Eclipse | -- | Velocity vs. volume distinction is excellent |
| 29 | Fading Resonant | **Strong** | Mirage | Gale, Cinder | -- | Fills "leaves play" gap alongside Echoing Departure |
| 30 | Stormtrace Augur | **Acceptable** | Tempest | Depths, Eclipse | -- | REDUNDANT with Stormtide Oracle -- cut one |
| 31 | Duskwatch Warden | **Acceptable** | Depths | Gale (low rate) | -- | Prevent-tempo conversion is real for Depths only |

---

## RATING DISTRIBUTION

| Rating | Count | Percentage |
|--------|-------|------------|
| Strong | 19 | 61% |
| Acceptable | 11 | 35% |
| Weak | 1 | 3% |
| Fail | 0 | 0% |

---

## TOP 5 BEST-DESIGNED CARDS (Most Genuinely Multi-Archetype)

### 1. Cinder Ritualist (Card 5)
Three archetypes trigger it through three completely different mechanisms: sacrifice (Cinder), mill (Undertow), discard (Eclipse). The "when a card enters your void from any zone" trigger is the single best piece of design innovation across all 31 cards. Each archetype uses the kindle and retrieval for different purposes. No forced claims. No padding.

### 2. Archivist of Vanished Names (Card 25)
The simplest possible modal (name Character or Event) creates the most archetype-dependent evaluations. Five archetypes want it, each naming differently: Tempest names Event, Mirage names Character, Undertow names whichever maximizes mill, Bedrock names Character for reanimation targets, Eclipse names Event for cycling pieces. The "reveal until" mechanic is stochastic in a way that rewards deckbuilding (character/event ratio awareness). Brilliant use of Counter-Pattern 1.

### 3. Crucible of the Commons (Card 24)
Five archetypes, each with genuinely different board compositions of low-spark bodies. The spark-1-or-less threshold creates anti-synergy with lord effects (which push bodies above the threshold), generating real deckbuilding tension. The card never mentions a tribe, so evaluation requires counting low-spark allies -- a calculation that differs completely by archetype.

### 4. Echoing Departure (Card 20)
Fills the most glaring mechanical hole in the 222-card pool (only 1 "leaves play" trigger existed). Three archetypes trigger it through three different mechanisms: flicker (Mirage), sacrifice (Cinder), bounce (Gale). The Foresee rider gives Cinder access to a Zephyr-native mechanic for the first time, creating genuinely surprising cross-resonance capability.

### 5. Tidechannel Observer (Card 28)
The void velocity concept (cards entering void PER TURN rather than total void size) is the most important mechanical innovation in the batch. It cleanly differentiates Undertow (trivially hits 3/turn) from Bedrock (almost never hits 3/turn), directly solving the Ruin bottleneck problem. The non-Survivor typing creating Undertow draft tension is exactly the structural design the format needs.

---

## CARDS NEEDING REDESIGN OR CUTS

### Must Redesign: Forgeborn Martyr (Card 4) -- Rating: Weak
The designer caught mid-explanation that the "dissolved by the opponent" requirement means Cinder's self-sacrifice does NOT trigger it, effectively retracting the secondary archetype claim. Bedrock's "Warrior reanimator" is too narrow. This is a well-designed Crucible card but a dishonest multi-archetype card.

**Fix:** Either remove the "by the opponent" clause (allowing self-sacrifice to trigger it, which makes it genuinely Cinder-viable) OR redesign the death trigger to reward any Warrior death regardless of source. Note: the Stone constraint says "When Warrior dissolved triggers must be OPPONENT-caused only," so the fix must navigate this constraint carefully. Perhaps: "When an allied Warrior is dissolved, if you have 3+ allies, draw 1 and gain 1 energy" -- conditional on board width rather than opponent action, playable in Crucible AND Cinder.

### Must Cut One: Stormtrace Augur (Card 30) vs. Stormtide Oracle (Card 6)
Both are characters whose spark equals events in your void. Stormtide Oracle (4 cost, Tide+Ember, Rare) adds a Judgment event-retrieval ability with a real decision (retrieving shrinks spark). Stormtrace Augur (3 cost, Tide, Rare) is a strictly simpler version. Having both oversaturates the V06 axis and reduces the novelty of each.

**Recommendation:** Cut Stormtrace Augur. Stormtide Oracle is the more interesting design (the retrieval-vs-spark tension is a genuine decision). Replace Stormtrace Augur with a card that addresses a DIFFERENT Tempest variety vector -- perhaps the event-copying payoff described in the synergy map's Agent E slot 4.

### Review: Oathbound Sentinel (Card 13) vs. Ironbark Sentinel (Card 27)
Both reward continuous board presence (the "Anchor Effect" -- V27). Oathbound Sentinel kindles if it stayed since last turn. Ironbark Sentinel gives +1 spark to each ally that has stayed. They are distinct enough to coexist (one is self-kindle, the other is team-pump), but two "stayed in play" rewards in a format that previously had zero may be aggressive. Monitor in testing.

### Review: Ember of Recurrence (Card 26) vs. Cinder Ritualist (Card 5)
Both trigger on "card enters void from any zone" and both retrieve from void. Cinder Ritualist adds kindle and is once-per-turn with no energy cost. Ember of Recurrence costs 1 energy per retrieval with no limit. Together they create a redundant void-retrieval axis. They are mechanically distinct enough to coexist, but the void-retrieval density should be monitored.

---

## MECHANICAL INTERACTIONS BETWEEN NEW CARDS

### Positive Interactions (Opportunities)

1. **Ironveil Watcher + Basalt Warden + Oathbound Sentinel:** All three have Judgment triggers. In a Crucible/Basalt deck, Ironveil Watcher counts the other two's triggers for its point generation. This creates a "Judgment Storm" micro-archetype across Stone cards that is genuinely new and rewards draft-time calculation of Judgment-trigger density.

2. **Cinder Ritualist + Ember of Recurrence:** In Cinder, both trigger on sacrifice (void entry). Cinder Ritualist kindles and retrieves once per turn for free. Ember of Recurrence retrieves additionally for 1 energy each. Together they create a powerful recursion engine: sacrifice one body, kindle from Ritualist, retrieve a body from Recurrence for 1 energy, have the Ritualist retrieve another. This might be TOO powerful in combination.

3. **Fading Resonant + Echoing Departure:** Both trigger on "ally leaves play." In Mirage, a flicker turn triggers both, generating cost reduction + Foresee (from Echoing Departure) AND additional cost reduction (from Fading Resonant). Combined cost reduction of 2 per ally-leave event enables massive tempo chains.

4. **Deepvault Warden + Bedrock Anchor:** Deepvault Warden reduces void-play costs by 2. Bedrock Anchor costs 2 from void and gains 2 energy on entry, meaning with Deepvault Warden it costs 0 and gains 2 energy. This is a free 0-spark body that draws a card and generates 2 energy. Strong Bedrock engine.

5. **Kindlespark Harvester + Stoneheart Veteran:** Both are mono-Stone cards that interact with kindle. Stoneheart Veteran BUILDS kindle (pay 3 energy for kindle 2). Kindlespark Harvester SPENDS kindle (remove 3 spark for removal). Together they create a kindle-management mini-game: build kindle -> spend kindle for removal -> build more kindle. This is excellent strategic depth within Stone.

### Negative Interactions (Problems)

1. **Dreamtide Cartographer + Crucible of the Commons:** Both are 3-cost Stone-coded bodies that pump low-spark allies on Judgment. In the same deck, they create redundant pump effects. Crucible of the Commons gives +1 to allies with spark 1 or less; Dreamtide Cartographer gives +1 to ALL allies (when hand is 4+). In a Basalt or Crucible deck, both would be played, creating a combined +2 per ally per Judgment on top of any lord effects. This could create an overpowered go-wide Stone strategy.

2. **Nexus of Passing + any Tempest storm turn:** Tempest playing 5 events generates ~10+ zone changes. Nexus of Passing becomes a 10+ spark body. Combined with Spirit of Smoldering Echoes (which ALSO grows from events entering void), Tempest's finisher density may be too high. Two scaling finisher bodies that both benefit from playing events is redundant power.

3. **Stormtide Oracle + Stormtrace Augur (if both kept):** Both scale with events in void. Having both in a Tempest draft deck means two 5-10 spark bodies for 7 total energy. Combined with Spirit of Smoldering Echoes, that's THREE scaling event-void-count bodies. This is clearly too much density on one axis. One must be cut.

4. **Ironbark Sentinel + Oathbound Sentinel in Depths:** Both reward continuous presence. If Depths runs both, every surviving turn generates kindle 1 (from Oathbound) AND +1 spark per surviving ally (from Ironbark Sentinel). Combined with Prevent protection keeping both alive, this is a fast-accelerating win condition that may make Depths too strong.

---

## FINAL OBSERVATIONS

### Strengths of the V2 Design Batch

1. **Agent D's modular engines are the standout.** Dreamtide Cartographer, Nexus of Passing, Crucible of the Commons, Archivist of Vanished Names, and Ember of Recurrence are the five most genuinely multi-archetype designs. Their shared philosophy -- "same text, different meaning in different decks" -- is the gold standard for draft-format card design.

2. **The "leaves play" gap is well-addressed.** Going from 1 card (Starlit Cascade) to 4 cards (Starlit Cascade, Ashen Threshold, Echoing Departure, Fading Resonant) that notice the most frequent Zephyr zone change is appropriate density without oversaturation.

3. **The Stone deficit is partially resolved.** 7 new mono-Stone cards (5 from Agent B, 2 from Agent D/E) bring the count from 31 toward 38. Not quite the 40 target, but substantial progress.

4. **The Crucible purity-vs-power tension is well-created.** Ironbark Sentinel, Ironveil Watcher, Vanguard of the Summit, Crucible of the Commons, and Dreamtide Cartographer all offer Crucible powerful non-Warrior cards, forcing real draft decisions about tribal density.

5. **Void velocity as a distinct mechanic** (Tidechannel Observer) is the single most important mechanical innovation for solving the Ruin bottleneck problem.

### Weaknesses and Concerns

1. **Balance on modular engines.** Dreamtide Cartographer and Nexus of Passing may be too universally desirable. A card that 7 archetypes want is a first-pick in every draft, which reduces draft variety rather than increasing it. Consider increasing costs or adding conditions.

2. **The V06 axis is oversaturated.** Stormtide Oracle and Stormtrace Augur both exploit event-count-in-void with nearly identical mechanics. Spirit of Smoldering Echoes already partially occupies this space. Three cards on one axis is too many. Cut one.

3. **Agent A's self-assessment is the most inflated.** The signpost dual designs consistently overstate secondary/tertiary archetype claims (Basalt claim for Depthswatcher, Crucible claim for Bedrock Anchor, Cinder claim for Forgeborn Martyr). Agent A averages 2.8 archetypes per card in self-assessment; my audit puts the genuine average closer to 2.1.

4. **Agent D's self-assessment is the most honest.** The modular engine cards genuinely work in the claimed archetypes. Agent D's average of 5.4 archetypes per card is ambitious but largely supported by the analysis.

5. **Forgeborn Martyr needs redesign.** The Crucible signpost's multi-archetype claim collapsed during the designer's own analysis. It's the only Weak-rated card in the batch and should be addressed before finalization.

6. **Two "anchor effect" cards (Oathbound Sentinel, Ironbark Sentinel) introducing an unprecedented mechanic simultaneously** should be monitored. Going from 0 to 2 "stayed in play" rewards is fine but could warp Stone's identity if both prove strong.
