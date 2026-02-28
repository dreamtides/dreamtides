# V2 Balance Audit -- Agent 2 (The Balance Auditor)

## Methodology

Every proposed card (31 total) is compared against the existing 222-card pool in `cards_dense.txt`. For each card, I identify the closest existing comparables at the same cost, evaluate the cost-to-stats ratio, check for strictly-better/worse relationships, flag broken interactions with existing cards, assess format-warping potential, and evaluate the playability floor. Power assessments use a five-point scale: Underpowered, Slightly Underpowered, Appropriate, Slightly Overpowered, Overpowered/Broken.

---

## Part 1: Per-Card Analysis

---

### A. Signpost Duals (10 cards)

---

#### 1. Tideweaver Sentinel | 3 cost | 1 spark | Tide+Zephyr

**Comparable existing cards:**
- Frost Visionary: 2 cost, 2 spark, Materialized: Draw 1
- Keeper of Forgotten Light: 6 cost, 2 spark, Materialized: Draw 2
- Nocturne: 3 cost, 2 spark, Materialized: Draw 1, discard 1, Reclaim 3
- Ethereal Courser: 3 cost, 1 spark, Materialized: Return an ally to hand

**Analysis:** The draw-2 mode on Materialized is extremely strong at 3 cost. Keeper of Forgotten Light pays 6 energy for Materialized: Draw 2 with 2 spark. Tideweaver Sentinel pays 3 energy for the same draw-2 effect with 1 spark plus an alternative bounce+energy mode. Even accounting for the 1 less spark, getting Materialized: Draw 2 at half the cost is a massive rate improvement. In flicker decks, this is obscene -- every re-trigger draws 2 cards. Compare to Looming Oracle (2 cost, 1 spark, Materialized: Draw 1): Tideweaver costs 1 more for double the draw plus a second mode.

The bounce mode (return ally + gain 2 energy) is also above rate. Ethereal Courser is 3 cost, 1 spark, Materialized: Return an ally (no energy). Tideweaver's bounce mode adds 2 energy on top. This mode alone is better than Ethereal Courser.

**Strictly better check:** Not strictly better than any single card (modal cards are inherently more flexible). But the draw mode alone is a strict upgrade over Looming Oracle's draw for only 1 more energy, which is a very efficient trade-up.

**Broken interactions:** With Flickerveil Adept (Judgment: banish ally spark 2 or less, materialize it): Tideweaver has 1 spark, so it can be flickered every Judgment, drawing 2 cards per Judgment for free after the initial investment. With Passage Through Oblivion (1 energy: banish ally, materialize at end of turn): nets draw 2 for 1 energy. These are strong but require setup.

**Format-warping potential:** HIGH. Any Tide or Zephyr deck auto-includes this. The draw-2 mode alone would be a premium 3-drop; having a second mode makes this one of the best value creatures in the format. In Mirage, this is the best flicker target in the game by a wide margin.

**Power Assessment: OVERPOWERED**

**Recommendation:** Either increase cost to 4, reduce draw mode to "Draw 1, then Foresee 1", or make the draw mode "Draw 2, then discard 1" to bring it in line. At 3 cost for Materialized: Draw 2 with a second mode, this card is format-warping.

---

#### 2. Abyssal Reclaimer | 3 cost | 1 spark | Tide+Ruin

**Comparable existing cards:**
- Searcher in the Mists: 2 cost, 1 spark, Materialized/Dissolved: Mill 4
- Nocturne: 3 cost, 2 spark, Materialized: Draw 1, discard 1, Reclaim 3
- Shadowpaw: 3 cost, 1 spark, Materialized: Return a character from void to hand

**Analysis:** Mill 3 + conditional Reclaim 1 is reasonable. Searcher in the Mists mills 4 for 2 cost. Abyssal Reclaimer mills 3 for 3 cost but adds Reclaim (which Shadowpaw does at the same cost without the mill). The condition "if 2 or more cards entered your void this turn" is trivially met by the card itself (it mills 3 cards into void), so this is effectively "Materialized: Mill 3, return a card from void to hand." That is Shadowpaw (return from void) plus Searcher in the Mists (mill) on one body. The combined effect is strong but not broken at 3 cost with 1 spark.

**Broken interactions:** In Mirage, each flicker is mill 3 + Reclaim 1, which is powerful card selection. With Flickerveil Adept, this mills 3 and Reclaims 1 every Judgment. Strong but gated by Judgment timing.

**Format-warping potential:** Moderate. Very good in Undertow/Eclipse but not auto-include everywhere.

**Power Assessment: SLIGHTLY OVERPOWERED**

**Recommendation:** The self-fulfilling threshold makes the condition meaningless. Consider raising the threshold to "3 or more" so the card itself only meets it, rather than guaranteeing a surplus. Alternatively, make the Reclaim conditional on cards entering void from ANOTHER source this turn (before this triggers). As written, the condition is flavor text -- it always resolves as mill-3 + Reclaim-1.

---

#### 3. Basalt Warden | 3 cost | 1 spark | Zephyr+Stone

**Comparable existing cards:**
- Ghostlight Wolves: 3 cost, 1 spark, Judgment: Gain 1 energy per Spirit Animal
- Emerald Guardian: 3 cost, 1 spark, Judgment: Gain 2 energy
- Spirit of the Greenwood: 3 cost, 1 spark, Judgment: Gain 1 energy per ally

**Analysis:** With 3+ allies, gain 2 energy. This is Emerald Guardian's output (2 energy) but conditional on 3+ allies. Without 3+ allies, the +1 spark to each ally is a weaker but still useful mode. Compared to Ghostlight Wolves (1 energy per Spirit Animal), Basalt Warden caps at 2 energy but is not tribal-restricted. The modes create genuine board-state reading.

The 3-ally threshold is easily met in Basalt/Crucible (wide boards are the norm). In Basalt, this is roughly equivalent to Ghostlight Wolves with 2 Spirit Animals -- achievable by turn 2-3 of the game.

**Format-warping potential:** Low. Good in wide-board decks, mediocre elsewhere. The modes are context-appropriate.

**Power Assessment: APPROPRIATE**

---

#### 4. Forgeborn Martyr | 3 cost | 2 spark | Stone+Ember

**Comparable existing cards:**
- Skyflame Commander: 2 cost, 1 spark, Allied Warriors have +1 spark (permanent)
- Assault Leader: 3 cost, 2 spark, 4 energy: +1 spark per Warrior (this turn)
- Silent Avenger: 3 cost, 2 spark, Dissolved: Kindle 2; When allied Survivor dissolved: Kindle 2

**Analysis:** Judgment: +1 spark to Warriors (temporary) is weaker than Skyflame Commander's permanent buff. The death trigger (draw 1, gain 1 when opponent dissolves a Warrior) is powerful but requires the opponent to cooperate -- they choose whether to trigger it. At 3 cost with 2 spark and the Warrior subtype, this is a solid tribal piece that creates a dilemma for the opponent.

The "dissolved by the opponent" clause is important. This does NOT trigger on self-sacrifice, preventing Cinder from abusing it. The opponent-facing trigger creates genuine game tension.

The Judgment pump is temporary, which is weaker than Skyflame Commander at 2 cost. But the death trigger compensates. Overall a fair package.

**Broken interactions:** With Grim Reclaimer (abandon ally: Reclaim a Warrior), the abandoned Warrior goes to void but was abandoned by YOU, not the opponent. So the death trigger does NOT fire. Correct self-policing.

**Format-warping potential:** Low-Medium. Very good in Crucible but the non-Warrior typing means you must evaluate whether it is better than another Warrior body. Good design.

**Power Assessment: APPROPRIATE**

---

#### 5. Cinder Ritualist | 3 cost | 1 spark | Ember+Ruin

**Comparable existing cards:**
- Infernal Ascendant: 3 cost, 1 spark, When you abandon an ally, kindle 2
- Exiles of the Last Light: 3 cost, 1 spark, Abandon an ally: Kindle 1
- Ashmaze Guide: 3 cost, 1 spark, When you discard a card, it gains Reclaim equal to cost

**Analysis:** "When a card enters your void from any zone, kindle 1" is an extremely broad trigger. Every mill, discard, abandon, and event resolution triggers this. In Undertow, a single Harvest the Forgotten (mill 3, draw 1) kindles 3 -- for FREE. In Eclipse, Fragments of Vision (draw 3, discard 2) kindles 2. In Cinder, each sacrifice kindles 1.

Compare to Infernal Ascendant (kindle 2 per abandon): Cinder Ritualist kindles 1 per void entry but fires on EVERYTHING, not just abandons. In a typical Cinder turn with 2 abandons, Infernal Ascendant kindles 4 while Cinder Ritualist kindles 2 from the abandons alone -- but also kindles from any other void entries (discards, mill, events resolving).

The "once per turn, abandon an ally: return a different character from void to hand" is strong but gated by once-per-turn and the "different character" clause.

**Broken interactions:** FLAG: With Searcher in the Mists (Materialized/Dissolved: Mill 4), materializing Searcher triggers mill 4, which kindles 4 from Cinder Ritualist. If Searcher is then dissolved, it mills 4 more, kindling 4 more. That is kindle 8 from a 2-cost creature dying. With Flickerveil Adept flickering Searcher, each flicker is kindle 4 minimum.

FLAG: With Harvest the Forgotten (1 energy: mill 3, draw 1), each cast kindles 3. If you replay Harvest via Reclaim, that is kindle 3 per cast for 1 energy. With event copying (Cascade of Reflections), each copied Harvest kindles 3 more.

FLAG: With Flagbearer of Decay (when you play a Survivor, mill 2): each Survivor play kindles 2. In Undertow with 4-5 Survivor plays per game, that is kindle 8-10 passively.

The kindle rate from this card is potentially very high. The leftmost character could easily reach 8-12 spark in 2-3 turns with this card in play.

**Format-warping potential:** HIGH in any deck that fills the void quickly (Undertow, Eclipse, Cinder). The trigger is too broad -- it kindles off everything, including incidental void entries from events resolving.

**Power Assessment: OVERPOWERED**

**Recommendation:** Restrict the trigger to "When you abandon an ally or discard a card from your hand" (removes mill and event resolution as triggers). Alternatively, change to "Once per turn, when a card enters your void, kindle 1" to cap the rate. As written, the kindle velocity is too high in void-heavy decks.

---

#### 6. Stormtide Oracle | 4 cost | * spark | Tide+Ember

**Comparable existing cards:**
- Spirit of Smoldering Echoes: 4 cost, * spark, When an event enters void, +1 spark
- Abomination of Memory: 5 cost, * spark, Spark = cards in void

**Analysis:** Spark = events in void. By mid-game in Tempest, this is typically 5-7 spark for 4 energy. Abomination of Memory costs 5 for spark = ALL cards in void (usually more). Spirit of Smoldering Echoes costs 4 and grows with each event entering void (slightly different -- cumulative vs snapshot).

The key difference from Spirit of Smoldering Echoes: Stormtide Oracle has a static count (retrieving events shrinks it), while Smoldering Echoes is cumulative (never shrinks). This means Stormtide Oracle is actually WEAKER in some scenarios because using the Judgment retrieval reduces your spark. The retrieval creates a genuine tension.

The Judgment retrieval (pay 2 energy: return event from void to hand) is powerful but self-correcting -- each retrieval reduces spark by 1. At 2 energy per activation, you are paying real resources.

**Broken interactions:** Near-duplicate concern with Stormtrace Augur (gap filler card #4, also "spark = events in void"). See inter-card section below.

**Format-warping potential:** Medium. Excellent in Tempest/Depths but requires event density. The retrieval cost and spark-reduction tension keep it honest.

**Power Assessment: APPROPRIATE**

---

#### 7. Depthswatcher | 4 cost | 0 spark | Tide+Stone

**Comparable existing cards:**
- Tidestone Arbiter (referenced in design but not in cards_dense.txt -- appears to be a proposed card, not existing)
- Pyrestone Avatar: 4 cost, 2 spark, When ally banished, kindle 1
- Flickerveil Adept: 4 cost, 1 spark, Materialized/Judgment: flicker ally

**Analysis:** Kindle 2 per Judgment (if 5+ cards in hand) is a strong rate. Ebonwing kindles 1 per Judgment for 1 cost. This kindles 2 for 4 cost, but the hand-size condition is a real constraint -- you must maintain 5+ cards, which means NOT playing cards aggressively. The 0 starting spark means you get nothing from this body until the kindle accumulates on the leftmost (which is not this card unless it is leftmost).

The Prevent trigger (gain 2 energy) is contextual. In Depths with 9 Prevents in the pool, this could generate 4-6 energy per game. That is good but not broken -- it requires running many Prevents and the opponent playing into them.

At 4 cost with 0 spark, the opportunity cost is real. Compare to Flickerveil Adept (4 cost, 1 spark with a powerful ability) or Aurora Channeler (4 cost, 1 spark, Materialized: gain 3 energy). Depthswatcher requires ongoing conditions to generate value.

**Format-warping potential:** Low. Excellent in Depths, mediocre elsewhere. The hand-size condition naturally excludes aggro decks.

**Power Assessment: APPROPRIATE**

---

#### 8. Galerunner | 2 cost | Fast Character | 1 spark | Zephyr+Ember

**Comparable existing cards:**
- Horizon Follower: 2 cost, Fast Visitor, 1 spark, Judgment: Gain 1 point
- Sage of the Prelude: 2 cost, Fast Musician, 1 spark, Once per turn: fast card -> draw 1
- Wasteland Arbitrator: 1 cost, Fast Survivor, 1 spark, Materialized: each player discards 1
- Herald of the Last Light: 1 cost, Visitor, 1 spark, Fast, Abandon: Prevent event

**Analysis:** Base rate is 2 cost, 1 spark, Fast -- same as Horizon Follower. The +2 spark at 2 or fewer cards in hand makes this a conditional 3-spark fast body for 2 energy. That is extremely efficient -- the next best 3-spark fast body would be Intermezzo Balladeer at 3 cost, 2 spark (which requires playing fast cards to grow, and the growth is temporary).

The abandon ability (abandon 0-spark ally: draw 1) converts dead figments and spent utility bodies into cards. This is a reasonable rate -- you lose a body to draw 1.

In Gale, the hand empties fast, so the +2 spark is reliably active. A 2-cost fast 3-spark body is among the most efficient threats in the game. Compare to The Calling Night (2 cost, 2 spark, Judgment: Draw 1 but opponent gains 2 points) -- Galerunner gives 3 spark with no downside when the condition is met.

**Broken interactions:** With Moonlit Dancer (characters in hand have fast, play fast character: gain 1 energy): Galerunner already IS fast, but Moonlit Dancer still gives energy when you deploy it. Deploy Galerunner with Moonlit Dancer in play: 2 cost minus 1 energy from Dancer = effective 1 energy for a 3-spark fast body. That is efficient but not broken.

**Format-warping potential:** Medium-High in Gale specifically. A 2-cost conditional 3-spark fast body is the premium threat for the archetype. Other decks cannot reliably trigger the hellbent condition, so it is not universally dominant.

**Power Assessment: SLIGHTLY OVERPOWERED**

**Recommendation:** Consider changing the threshold to "1 or fewer cards in hand" to make the condition harder to maintain, or reduce the bonus to +1 spark. At +2 spark, the card offers a 3-spark fast body for 2 energy too reliably in Gale.

---

#### 9. Eclipse Weaver | 3 cost | 2 spark | Zephyr+Ruin

**Comparable existing cards:**
- Ashmaze Guide: 3 cost, 1 spark, When you discard a card, it gains Reclaim equal to cost
- Nocturne: 3 cost, 2 spark, Materialized: Draw 1, discard 1, Reclaim 3
- Ashlight Caller: 3 cost, Fast, 1 spark, Materialized: event in void gains Reclaim equal to cost

**Analysis:** "Once per turn, discard a card: an event in void gains Reclaim equal to its cost this turn" is Ashlight Caller's Materialized ability as a repeatable activated ability. Ashlight Caller needs to be flickered or replayed to repeat; Eclipse Weaver does it every turn for free (costing a discard).

The "when you play event from void, draw 1" rider makes each Reclaimed event cantrip. This is strong -- effectively "once per turn, discard a card to give an event Reclaim and draw a replacement when you play it." Net card flow: discard 1, gain Reclaim event, play Reclaimed event and draw 1 = net +1 card advantage per cycle (you traded a card for the Reclaimed event and drew 1).

At 3 cost with 2 spark, the base stats are good (Ashmaze Guide is 3/1, Nocturne is 3/2). The higher spark combined with a repeatable engine effect makes this quite strong.

**Broken interactions:** With Ashmaze Guide in play simultaneously: discard a card -> Ashmaze gives it Reclaim, Eclipse Weaver gives an event in void Reclaim, AND you can replay the discarded card later. This creates a double-Reclaim engine where every discard yields two Reclaim effects.

With Tempest storm pieces: recovering Genesis Burst or Flash of Power from the void for replay is powerful energy generation.

**Format-warping potential:** Medium. Strong in Eclipse, good in Tempest, but requires discard enablers to function optimally.

**Power Assessment: SLIGHTLY OVERPOWERED**

**Recommendation:** The 2 spark is generous for a card with this much engine potential. Consider reducing to 1 spark (matching Ashmaze Guide) to create a more meaningful deckbuilding cost. Alternatively, make the draw-on-void-event conditional ("If you discarded a card this turn, when you play an event from void, draw 1").

---

#### 10. Bedrock Anchor | 2 cost | 0 spark | Stone+Ruin

**Comparable existing cards:**
- Revenant of the Lost: 3 cost, 6 spark, void-only
- Kindred Sparks: 5 cost (1 with Survivor), 2 spark, play from hand or void with Survivor
- Aspiring Guardian: 0 cost, 1 spark, vanilla Warrior

**Analysis:** Void-only, but when played from void: gain 2 energy (net 0 cost) and draw 1. The Judgment kindle (if survived since last turn) adds slow but permanent spark growth.

Compared to Revenant of the Lost (3/6, void-only): Bedrock Anchor is a completely different card. Revenant is a finisher; Bedrock Anchor is an engine piece. At effective 0 cost (2 energy - 2 energy refund) with a draw, this is essentially a free cantrip body from the void. The kindle is gravy.

**Broken interactions:** FLAG: With Deepvault Warden (new Stone card, "Characters from void cost 2 less"): Bedrock Anchor from void costs 0 (2-2=0), gains 2 energy (net +2 energy), draws 1. This is a FREE body that generates 2 energy and draws a card. If you can bounce/sacrifice and replay it, each cycle nets +2 energy and +1 card. With Grim Reclaimer or sacrifice effects, this creates a resource-generation loop.

FLAG: With Cinder Ritualist (new card, "when card enters void, kindle 1"): sacrificing Bedrock Anchor kindles 1, then replaying it from void gives 2 energy and draws 1. This is a net-positive sacrifice loop.

The void-only restriction means it must be milled or discarded first, which is a real setup cost. But in Bedrock/Undertow, this happens naturally.

**Format-warping potential:** Medium when combined with Deepvault Warden. On its own, appropriate.

**Power Assessment: APPROPRIATE (but see inter-card interaction with Deepvault Warden below)**

---

### B. Mono-Stone Cards (5 cards)

---

#### 11. Ironveil Watcher | 2 cost | 0 spark | Stone

**Comparable existing cards:**
- Ethereal Trailblazer: 0 cost, 0 spark, Judgment: Gain 1 energy
- Ruin Scavenger: 1 cost, 0 spark, Judgment: Banish 1 from opponent void, gain 1 energy
- Spirit Field Reclaimer: 2 cost, 0 spark, Judgment: Pay 1 energy to kindle 1 + banish 1 from opponent void

**Analysis:** "Judgment: Gain 1 point per other Judgment trigger this phase" scales with board width of Judgment creatures. In a Crucible board with 4 other Judgment triggers, this is 4 points per Judgment. That is a very fast clock -- comparable to Melodist of the Finale (4 cost, 2 spark, gain 1 point per fast card, but requires active play each turn).

At 2 cost with 0 spark, the body is unimpressive. But points-per-Judgment is a strong win condition in a format where Judgment is the primary scoring mechanism.

**Broken interactions:** FLAG: With Conduit of Resonance (When you materialize a character, trigger Judgment ability of each ally): Each materialization triggers Ironveil Watcher's Judgment. If you have 4 other Judgment creatures and materialize a character, Ironveil Watcher counts those triggered Judgment abilities. In Basalt with Conduit of Resonance, this could generate 4-6 points per materialization -- on top of the actual Judgment phase points.

FLAG: With Surge of Fury (trigger additional Judgment phase): doubles the point output.

**Format-warping potential:** Medium-High in Stone decks that stack Judgment triggers. The scaling is multiplicative -- the more Judgment creatures you have, the better this gets. In a board of 5 Judgment creatures, this generates 5 points per Judgment, which wins the game in 3-4 Judgment phases.

**Power Assessment: SLIGHTLY OVERPOWERED**

**Recommendation:** Consider changing to "Gain 1 point for each other Judgment ability that triggered this phase, up to 3" to cap the scaling. Alternatively, increase cost to 3. The uncapped scaling with Conduit of Resonance is concerning.

---

#### 12. Stoneheart Veteran | 3 cost | 1 spark | Stone | Warrior

**Comparable existing cards:**
- Spirit Field Reclaimer: 2 cost, 0 spark, Judgment: Pay 1 to kindle 1 + banish void
- Ebonwing: 1 cost, 1 spark, Materialized/Judgment: Kindle 1
- Assault Leader: 3 cost, 2 spark, Pay 4: +1 spark per Warrior (temp)

**Analysis:** Pay 3 energy for kindle 2 on Judgment. Rate is 1.5 energy per kindle. Spirit Field Reclaimer is 1 energy per kindle 1 (1.0 rate) plus void hate. Ebonwing is free kindle 1 per Judgment (0.0 rate). Stoneheart Veteran's rate is worse per kindle but has no ceiling and is on a Warrior body.

Compared to Assault Leader (pay 4 for +1 per Warrior, temporary): with 4 Warriors, Assault Leader gives 4 temporary spark for 4 energy (1.0 energy per spark, temporary). Stoneheart Veteran gives 2 permanent kindle for 3 energy (1.5 per kindle, permanent). Kindle is permanent and compounds -- after 3 Judgments, you have invested 9 energy for 6 permanent spark, which far exceeds Assault Leader's per-Judgment temporary boost.

**Format-warping potential:** Low. The 3-energy cost per activation is a real constraint. In energy-rich boards (5+ energy per Judgment), this is strong but competes with deploying new threats.

**Power Assessment: APPROPRIATE**

---

#### 13. Oathbound Sentinel | 2 cost | 0 spark | Stone | Common

**Comparable existing cards:**
- Ebonwing: 1 cost, 1 spark, Materialized/Judgment: Kindle 1
- Spirit Field Reclaimer: 2 cost, 0 spark, Judgment: Pay 1 to kindle 1 + void hate

**Analysis:** "Start of turn, if survived since last turn, kindle 1" is unconditional kindle 1 per turn (after the first turn). Ebonwing also kindles 1 per Judgment for 1 cost with 1 spark. Oathbound Sentinel costs 1 more, has 0 spark (vs Ebonwing's 1), and kindles 1 on start of turn instead of Judgment. The timing difference is minor.

This is strictly worse than Ebonwing in most respects: costs more, less spark, same kindle rate, and requires survival (Ebonwing kindles on materialization too). The "since your last turn" condition means it does NOT kindle on the first turn you play it, while Ebonwing kindles immediately via Materialized.

The anti-synergy with Zephyr flicker (bouncing resets the survival check) is a nice design feature.

**Format-warping potential:** Low. A worse Ebonwing in most scenarios. The Common rarity means it is accessible filler, not a build-around.

**Power Assessment: SLIGHTLY UNDERPOWERED**

**Recommendation:** The card is intentionally modest at Common, which is fine. But at 2 cost with 0 spark and a delayed kindle that requires survival, it competes poorly with existing options. Consider making it 1 cost to compete with Ebonwing on rate, or giving it 1 spark.

---

#### 14. Vanguard of the Summit | 4 cost | 2 spark | Stone | Mage

**Comparable existing cards:**
- Invoker of Myths: 4 cost, 2 spark, Once per turn: materialize a Warrior -> draw 1
- Flickerveil Adept: 4 cost, 1 spark, Materialized/Judgment: flicker ally
- Lumin-Gate Seer: 4 cost, 2 spark, Once per turn: materialize cost 2 or less -> draw 1

**Analysis:** "When you play your third character this turn, draw 2 and gain 2 energy." The threshold is demanding -- you need to play 3 characters in one turn. With Nexus Wayfinder (characters cost 2 less), cheap characters become free, making this achievable in Crucible/Basalt. Without cost reduction, playing 3 characters in a turn requires significant energy (e.g., three 0-cost Ethereal Trailblazers, or multiple cheap bodies).

When triggered, draw 2 + gain 2 energy is a massive reward -- you have effectively cast a 4-cost body that refunds half its cost and draws 2. But the threshold ensures this does not happen every game or every turn.

At 4 cost, 2 spark with no ability if the threshold is not met, this is a mediocre vanilla body in the fail case. The floor is bad; the ceiling is excellent.

**Broken interactions:** With Nexus Wayfinder (characters cost 2 less): 0-cost creatures flow freely, making the 3rd-character threshold trivial. But Nexus Wayfinder is 5 cost and Rare, so this is a 9-energy combo that requires specific cards.

**Format-warping potential:** Low. Requires specific board state (cost reduction + cheap characters) to trigger. Auto-include in Crucible with Nexus Wayfinder but that is a two-card combo at high total cost.

**Power Assessment: APPROPRIATE**

---

#### 15. Deepvault Warden | 3 cost | 1 spark | Stone | Explorer

**Comparable existing cards:**
- Nexus Wayfinder: 5 cost, 2 spark, Characters cost 2 less
- Starsea Traveler: 4 cost, 2 spark, Play cost-2-or-less character from void
- Reclaimer of Lost Paths: 4 cost, 1 spark, Materialized: cost-3-or-less void card gains Reclaim 0

**Analysis:** "Characters from void cost 2 less" is Nexus Wayfinder's effect but only for void-sourced characters and at 3 cost (vs 5). Nexus Wayfinder reduces ALL character costs; Deepvault Warden only reduces void-sourced ones. The narrower scope justifies the lower cost.

**Broken interactions:** FLAG: Revenant of the Lost (3 cost, 6 spark, void-only) becomes a 1-cost 6-spark play. That is the single most efficient creature in the game by a massive margin. Currently, Revenant requires 3 energy to deploy from void -- a fair rate for 6 spark. At 1 energy for 6 spark, Revenant becomes a better rate than any card in the format.

FLAG: With Bedrock Anchor (new card, 2 cost, void-only, gains 2 energy on play): costs 0, gains 2 energy = net +2 energy and a free body with draw 1. Each sacrifice-and-replay cycle nets +2 energy and +1 card. This is a resource-generation engine with no cap.

FLAG: With Nexus Wayfinder (characters cost 2 less) AND Deepvault Warden: void characters cost 4 less total. Most characters in the game cost 4 or less, meaning they are FREE from the void. Combined with Path to Redemption (all void cards gain Reclaim), you can replay your entire void for free.

FLAG: The Devourer (8 cost, 8 spark, banish void 8+: Reclaim) becomes a 6-cost play from void. Still expensive but much more achievable.

**Format-warping potential:** HIGH in Bedrock specifically. Revenant of the Lost at 1 cost is game-breaking. The card makes Bedrock's win condition too cheap and too reliable.

**Power Assessment: OVERPOWERED**

**Recommendation:** Change to "Characters you play from your void cost 1 less" (not 2). At 1-less, Revenant becomes 2 cost (still strong but fair), and the Bedrock Anchor loop is less degenerate (nets +1 instead of +2). The 2-less reduction creates too many broken interactions with existing void-only and Reclaim cards.

---

### C. Cross-Pollination Cards (6 cards)

---

#### 16. Ashen Threshold | 3 cost | 1 spark | Ember

**Comparable existing cards:**
- Volcanic Channeler: 4 cost, 2 spark, When ally dissolved, gain 1 energy
- Angel of the Eclipse: 4 cost, 3 spark, When you materialize an ally, gain 1 energy
- Looming Oracle: 2 cost, 1 spark, Materialized: Draw 1

**Analysis:** "When ally leaves play, gain 1 energy" is Starlit Cascade as a permanent effect on a body (Starlit Cascade is "until end of turn" for 1 energy). Making this permanent on a 3-cost body is very strong. In Mirage, every flicker generates energy. In Cinder, every sacrifice generates energy.

"When you materialize from void, draw 1" rewards Bedrock/Cinder void recursion. Each Reclaim play draws a card.

Together: each sacrifice-and-Reclaim cycle nets 1 energy + 1 draw. That is significant repeatable value.

**Broken interactions:** With Starlit Cascade (when ally leaves play, gain 2 energy until end of turn) + Ashen Threshold: each ally leaving play gains 3 energy total. In Mirage mass-flicker turns (Aurora Rider flickering 3 allies), that is 9 energy from Starlit Cascade + 3 from Ashen Threshold = 12 energy. This is enormous but requires two specific cards and a mass-flicker setup.

FLAG: In Cinder with Exiles of the Last Light (abandon ally: kindle 1) + Ashen Threshold: each abandon kindles 1, gains 1 energy, and if the abandoned creature is Reclaimed, draws 1. This creates a very efficient sacrifice loop.

**Format-warping potential:** Medium-High. Any deck that generates frequent zone transitions wants this. It is broadly applicable but requires specific strategies to maximize.

**Power Assessment: SLIGHTLY OVERPOWERED**

**Recommendation:** Consider making one of the two triggers conditional. For example: "When an ally leaves play, gain 1 energy. Once per turn, when you materialize a character from your void, draw 1." The once-per-turn restriction on the draw prevents runaway card advantage from mass-recursion turns.

---

#### 17. Voidthorn Sentinel | 2 cost | 1 spark | Ruin | Survivor

**Comparable existing cards:**
- Herald of the Last Light: 1 cost, 1 spark, Fast, Abandon: Prevent a played event
- Cragfall: 2 cost, Fast Event, Prevent a played character
- Threadbreaker: 3 cost, Fast Synth, 1 spark, Materialized: Prevent cost 2 or less

**Analysis:** "Abandon this: Prevent a played character" is Cragfall as a body-based effect. Cragfall costs 2 energy and is gone after use. Voidthorn Sentinel also costs 2 energy but leaves a 1-spark body on board until you need to use it, then sacrifices to Prevent. This is strictly better than Cragfall in most scenarios (you get a body until you need the Prevent).

The "When you prevent, mill 2" rider is mild but adds void-filling for Ruin strategies.

**Strictly better check:** FLAG: This is strictly better than Cragfall (Fast Event, 2 cost: Prevent a played character). Both cost 2, both prevent a played character. But Voidthorn Sentinel also gives you a 1-spark body and a mill rider until you use it. The only advantage Cragfall has is being a fast event (can be held in hand as a surprise without telegraphing). Voidthorn Sentinel must be on the battlefield, which telegraphs the Prevent to the opponent. This is a meaningful distinction -- the opponent can play around a face-up Prevent body -- but it is still very close to strictly better.

**Format-warping potential:** Medium. A 2-cost body that doubles as character removal is broadly playable. The Prevent is telegraphed (the opponent sees the body), which provides counterplay.

**Power Assessment: APPROPRIATE** (the telegraphing balances the body advantage over Cragfall)

---

#### 18. Resonance Siphon | 2 cost | Fast Event | Neutral

**Comparable existing cards:**
- Guiding Light: 1 cost, Fast Event, Foresee 1, Draw 1, Reclaim 3
- Data Pulse: 2 cost, Event, Gain 2 energy, Draw 1
- Beacon of Tomorrow: 2 cost, Event, Discover cost 2

**Analysis:** "Until end of turn, activated abilities cost 2 less. Draw 1." This is a cantrip that enables explosive activated ability turns. The key targets:
- Spiritbound Alpha: 4 -> 2 energy (each Spirit Animal +2 spark)
- Mystic Runefish: 3 -> 1 energy (Spirit Animals become 5 spark)
- Assault Leader: 4 -> 2 energy (Warriors +1 spark each)
- Minstrel of Falling Light: 3 -> 1 energy (draw 1)

At 2 cost for a cantrip + global cost reduction, this is efficient but narrow. If you have no expensive activated abilities, it is a 2-cost cantrip (bad). If you have 2+ expensive abilities, it is a 2-cost cantrip that saves 4+ energy (excellent).

**Broken interactions:** With Minstrel of Falling Light (3 energy: draw 1 -> becomes 1 energy: draw 1 after Siphon): This creates a 1-energy draw-1 engine for the turn. With enough energy, you can draw your whole deck. With Starcatcher (when you play event, gain 1 energy): Siphon pays for itself via Starcatcher and then each Minstrel activation costs 1 for draw 1. This chain is strong but energy-gated.

FLAG: With Mystic Runefish (3 -> 1 energy: all Spirit Animals become 5 spark): if you also have Spiritbound Alpha (4 -> 2 energy: each Spirit Animal +2 spark), a single Resonance Siphon turn could give all Spirit Animals 7 spark for 3 total energy. With 4 Spirit Animals, that is 28 spark in one Judgment. This is a game-winning alpha strike but requires 3 specific cards in play and appropriate energy.

**Format-warping potential:** Low-Medium. Very deck-dependent. Dead in decks without expensive activated abilities.

**Power Assessment: APPROPRIATE**

---

#### 19. Kindlespark Harvester | 3 cost | 0 spark | Stone

**Comparable existing cards:**
- Eclipse Herald: 2 cost, 1 spark, Judgment: Banish 3 from void to dissolve enemy cost 2 or less
- Spirit Field Reclaimer: 2 cost, 0 spark, Judgment: Pay 1 to kindle 1 + banish opponent void

**Analysis:** "Judgment: Remove 3 spark from leftmost ally to dissolve enemy cost 3 or less. If you do, kindle 2." This converts accumulated spark into removal while refunding 2 kindle. Net cost: 3 spark spent, 2 kindle returned = net -1 spark for removing an enemy of cost 3 or less.

This is a fascinating mechanic -- spending spark (your scoring resource) for removal. The decision is genuinely interesting. However, dissolving "cost 3 or less" is narrow -- many important threats cost 4+. Compare to Eclipse Herald (dissolve cost 2 or less for banishing 3 void cards): Kindlespark Harvester has a higher cost threshold but requires 3 accumulated spark.

**Broken interactions:** With Cinder kindle engines (Infernal Ascendant: kindle 2 per abandon): if you build 8+ spark via kindle, you can dissolve 2 enemies per game while still maintaining 4+ spark for scoring. The removal is repeatable (every Judgment) which is strong over time.

**Format-warping potential:** Low. Requires spark accumulation to function. Dead if your leftmost ally has 0-2 spark. Interesting build-around that rewards kindle investment.

**Power Assessment: APPROPRIATE**

Good design. The spark-for-removal tension is novel and skill-testing.

---

#### 20. Echoing Departure | 2 cost | Event | Zephyr

**Comparable existing cards:**
- Starlit Cascade: 1 cost, Event, Until end of turn: when ally leaves play, gain 2 energy
- Celestial Reverie: 1 cost, Event, Until end of turn: when you play character, draw 1

**Analysis:** "Until end of turn, when ally leaves play, next character costs 1 less and Foresee 1." Similar to Starlit Cascade (which gives 2 energy per leave) but gives cost reduction + Foresee instead.

Starlit Cascade costs 1 and gives 2 energy per leave (net +2 per trigger). Echoing Departure costs 2 and gives 1 cost reduction + Foresee 1 per trigger (net value ~1.5 per trigger). Starlit Cascade is strictly better as an enabler for large turns. Echoing Departure provides card selection (Foresee) which is a different kind of value.

At 2 cost for an "until end of turn" effect, this is slightly expensive compared to Starlit Cascade (1 cost). The effect is weaker per trigger too.

**Format-warping potential:** Low. A setup card that requires allies to leave play to generate value. Common rarity is appropriate.

**Power Assessment: APPROPRIATE**

---

#### 21. Risen Warden | 4 cost | 2 spark | Ruin | Warrior

**Comparable existing cards:**
- Revenant of the Lost: 3 cost, 6 spark, void-only
- Ashen Avenger: 3 cost, 1 spark, Pay 2 + banish void card: Reclaim
- Forsworn Champion: 4 cost, 2 spark, Abandon ally: +1 spark

**Analysis:** Void-only, Dissolved: Reclaim a Warrior from void. At 4 cost, 2 spark, void-only, this is much less efficient than Revenant of the Lost (3 cost, 6 spark). The value comes from the Dissolved recursion -- when killed, it brings back another Warrior.

The self-referential loop: if it is the only Warrior in void when dissolved, can it target itself? The card says "Reclaim a Warrior from your void." After being dissolved, it goes to the void, and then its trigger fires. It could target itself, creating an infinite recursion loop if the opponent keeps dissolving it (you pay 4 each time, but it keeps coming back).

FLAG: The self-Reclaim loop means the opponent can NEVER permanently remove this card unless they banish it (banish skips void). Dissolve effects are useless against it. This makes it a must-answer threat that requires specific answers (banish removal only).

**Broken interactions:** With Deepvault Warden (void characters cost 2 less): costs 2 from void with 2 spark and is unkillable by dissolve. That is very efficient for a recurring threat.

With Forgeborn Martyr (new card, when opponent dissolves a Warrior: draw 1, gain 1 energy): opponent dissolves Risen Warden -> you draw 1, gain 1, and Reclaim a Warrior. The opponent can never profitably remove your Warriors.

**Format-warping potential:** Medium in Crucible/Bedrock. The unkillable-by-dissolve nature is a design concern.

**Power Assessment: SLIGHTLY OVERPOWERED**

**Recommendation:** Add "Reclaim a DIFFERENT Warrior from your void" to prevent the self-loop. This maintains the Warrior-recursion theme without creating an unkillable body.

---

### D. Modular Engine Cards (5 cards)

---

#### 22. Dreamtide Cartographer | 3 cost | 1 spark | Neutral | Explorer

**Comparable existing cards:**
- Seeker of the Radiant Wilds: 2 cost, 1 spark, Materialized: Draw 1 per Spirit Animal
- Spiritbound Alpha: 2 cost, 1 spark, Judgment: Pay 4 for Spirit Animals +2 spark
- Hope's Vanguard: 2 cost, 1 spark, Materialized/Judgment: with 2+ Survivors, draw 1

**Analysis:** "Judgment: If 3 or fewer cards in hand, draw 2. Otherwise, each ally +1 spark until end of turn." This is a flexible Judgment trigger -- either draw 2 or mass pump. Both modes are strong.

Draw-2 per Judgment is exceptional card advantage. Compare to The Calling Night (2 cost, 2 spark, Judgment: Draw 1, opponent gains 2 points): Dreamtide Cartographer draws 2 with no downside when the hand condition is met. Hope's Vanguard draws 1 per Judgment with a Survivor condition. Drawing 2 per Judgment is a better rate than almost any existing draw engine.

The +1 spark to each ally mode on a board of 4-5 allies is +4-5 spark per Judgment, which rivals lord effects.

At 3 cost for a NEUTRAL card (no resonance restriction), this is accessible to every deck.

**Broken interactions:** In Gale: dump hand -> draw 2 per Judgment. The refueling is constant and free. In Crucible: maintain a hand of Warriors -> +1 spark to each ally per Judgment. Both modes are premium value.

**Format-warping potential:** HIGH. A neutral 3-cost body that draws 2 per Judgment or pumps all allies is universally desirable. Every deck either empties its hand (draw mode) or holds cards (pump mode). There is no deck that does not want one of these modes.

**Power Assessment: OVERPOWERED**

**Recommendation:** Either make it mono-resonance (not neutral) to restrict access, reduce draw to 1, or increase cost to 4. A neutral card that is this flexible should not also be this efficient. At 3 cost neutral with draw-2-or-mass-pump, this is the single best value creature in the format.

---

#### 23. Nexus of Passing | 4 cost | * spark | Neutral | Ancient

**Comparable existing cards:**
- Spirit of Smoldering Echoes: 4 cost, * spark, +1 spark per event entering void
- Abomination of Memory: 5 cost, * spark, Spark = cards in void
- Tideborne Voyager: 4 cost, 0 spark, +1 spark per ally banished

**Analysis:** "Spark = cards that changed zones this turn." This measures the CURRENT turn's activity. Events going from hand to stack to void = 2 zone changes each. Flicker = 2 per creature. Mill = 1 per card.

In Tempest: playing 5 events = ~10 zone changes = 10 spark. That is potentially the highest single-turn spark in the game. Even Abomination of Memory (spark = total void, usually 8-12 by endgame) does not hit 10 consistently.

In Mirage: mass flicker of 3 allies = 6 zone changes + draws/materializations = ~8-10 spark.

The card has 0 spark on turns where nothing happens. It demands active engine maintenance every turn. But the ceiling is very high.

**Broken interactions:** FLAG: In Tempest with event copying (Cascade of Reflections, Echoes of Eternity), each copied event is additional zone changes. A Tempest turn of play 3 events + copy each = 12 zone changes = 12 spark from a 4-cost body.

FLAG: The "this turn" measurement resets each turn, so the card is only good during your active turn. On the opponent's turn, it provides 0 spark (unless you play fast cards). This means it only scores during YOUR Judgment, not the opponent's. Wait -- actually, in Dreamtides there is only one Judgment per round. If spark is measured at Judgment, what is "this turn"? If it means "this round," then all zone changes from both players' turns count. If it means "your most recent turn," then only your activities count. This needs clarification but I will assume it means the turn during which Judgment occurs.

**Format-warping potential:** Medium-High. In Tempest and Mirage, this is potentially the highest-spark body in the game. In slow decks, it is mediocre (few zone changes = low spark).

**Power Assessment: SLIGHTLY OVERPOWERED**

**Recommendation:** Consider adding a cap ("up to 8 spark") or changing to "Spark = half the number of cards that changed zones this turn (rounded down)" to moderate the ceiling in storm/flicker decks. Without a cap, this can reach 10-15 spark in a single Judgment, which is game-winning from a single 4-cost body.

---

#### 24. Crucible of the Commons | 3 cost | 2 spark | Stone | Visitor

**Comparable existing cards:**
- Skyflame Commander: 2 cost, 1 spark, Allied Warriors +1 spark (permanent, tribal)
- Spiritbound Alpha: 2 cost, 1 spark, Pay 4: Spirit Animals +2 spark (temp, tribal)

**Analysis:** "Judgment: Each ally with spark 1 or less gains +1 spark until end of turn." This is a non-tribal mass pump for low-spark creatures. In a board of 4 allies at 0-1 spark, this is +4 spark per Judgment.

At 3 cost, 2 spark (Stone), this has solid base stats. The effect is strong in wide-board decks (Crucible, Basalt, Mirage) where many utility creatures have 0-1 spark.

Compared to Skyflame Commander (2 cost, permanent +1 to Warriors): Crucible of the Commons is broader (all allies, not just Warriors) but temporary and conditional (spark 1 or less only). The non-tribal nature and the spark-threshold condition make it appropriately balanced.

**Broken interactions:** With Skyflame Commander in Crucible: Skyflame gives Warriors permanent +1, which pushes them from 0-1 to 1-2. Warriors at 2 spark no longer qualify for Crucible of the Commons' buff. This creates natural anti-synergy that limits stacking.

**Format-warping potential:** Medium. Good in wide-board decks, but the spark-threshold condition prevents it from scaling infinitely with lords.

**Power Assessment: APPROPRIATE**

Good design. The anti-synergy with lord effects is an excellent self-balancing feature.

---

#### 25. Archivist of Vanished Names | 3 cost | 1 spark | Tide | Mage

**Comparable existing cards:**
- Seeker for the Way: 3 cost, 2 spark, Materialized: Draw a Warrior from deck
- Nocturne: 3 cost, 2 spark, Materialized: Draw 1, discard 1, Reclaim 3
- Searcher in the Mists: 2 cost, 1 spark, Materialized/Dissolved: Mill 4

**Analysis:** "Materialized: Name Character or Event. Reveal until you find one, put it in hand, rest into void." This is an impulse/cascade effect that provides card advantage + mill. On average, you mill 1-2 cards and draw 1 specific card.

Compare to Seeker for the Way (3 cost, 2 spark, find a Warrior from deck): Seeker is deterministic and has better base stats. Archivist is stochastic but broader (any card of the named type) and provides void-filling as upside.

In a Mirage flicker deck, each flicker re-triggers this, providing a repeatable card-selection + mill engine. That is strong but the card costs 3 with 1 spark, so the body is unimpressive.

**Broken interactions:** FLAG: In a deck with extreme type imbalance (e.g., 90% characters, naming Event), you could mill 8-10 cards to find a single event. This is a self-mill engine disguised as card selection. In Undertow, deliberately naming the rarer type maximizes mill. With Abomination of Memory (spark = void size), each mill grows the finisher.

The "no upper bound" on mill is a concern -- in edge cases, you could deck yourself. But this is self-punishing, so it is appropriate.

**Format-warping potential:** Medium. Excellent in flicker decks (repeatable card selection), good elsewhere. The Materialized trigger makes it a premium flicker target.

**Power Assessment: APPROPRIATE**

---

#### 26. Ember of Recurrence | 3 cost | 1 spark | Ruin | Synth

**Comparable existing cards:**
- Ashmaze Guide: 3 cost, 1 spark, When you discard a card, it gains Reclaim equal to cost
- Shadowpaw: 3 cost, 1 spark, Materialized: Return character from void to hand
- Reclaimer of Lost Paths: 4 cost, 1 spark, Materialized: cost-3-or-less void card gains Reclaim 0

**Analysis:** "When a card enters your void from any zone, pay 1 energy: Return a different card from void to hand." This is a universal void-entry trigger that allows paid retrieval. The 1-energy cost per retrieval is the throttle.

In Undertow: mill 4 via Searcher in the Mists = 4 triggers. Spending 4 energy to retrieve 4 cards would be insane value but impractical (you need 4 energy and hand space). Realistically, retrieve 1-2 per mill batch.

In Eclipse: each discard triggers retrieval. Discard 2 (Moonlit Voyage) = 2 triggers. Spend 2 energy to retrieve 2 void cards = 2 energy for 2 cards.

**Broken interactions:** FLAG: With Cinder Ritualist (new card, "when card enters void, kindle 1"): every void entry kindles AND allows retrieval. A sacrifice loop becomes: abandon ally -> enters void -> kindle 1 + pay 1 to retrieve different card -> next turn, replay retrieved card and sacrifice again. Each loop: kindle 1, spend 1 energy, net 0 cards. This is a 1-energy-per-kindle engine, which is Spirit Field Reclaimer's rate. Fair.

FLAG: With Harvest the Forgotten (mill 3, draw 1): 3 triggers. Pay 3 energy to retrieve 3 cards from the existing void. Combined with the draw 1 from Harvest, that is 4 cards for 4 energy (Harvest + 3 retrievals). That is a very efficient card advantage engine. But it requires 3 energy for the retrievals plus 1 for Harvest = 4 energy total for 4 cards. Compare to Knowledge Restored (5 energy: draw 3). The rate is better but requires a stocked void.

The "different card" clause prevents retrieving the card that just entered, which is important for preventing self-loops.

**Format-warping potential:** Medium-High. Any deck that fills the void quickly wants this as a card advantage engine. The 1-energy cost is a meaningful throttle but may not be enough in energy-rich decks (Stone ramp).

**Power Assessment: SLIGHTLY OVERPOWERED**

**Recommendation:** Consider increasing the energy cost to 2 per retrieval, or adding "once per turn" to the retrieval. At 1 energy per retrieval with 4-8 triggers per turn in Undertow, the card generates too much card advantage in void-heavy strategies.

---

### E. Gap Filler Cards (5 cards)

---

#### 27. Ironbark Sentinel | 4 cost | 3 spark | Stone | Ancient

**Comparable existing cards:**
- Forsworn Champion: 4 cost, 2 spark, Abandon ally: +1 spark (permanent)
- Flickerveil Adept: 4 cost, 1 spark, Materialized/Judgment: flicker ally
- Aurora Channeler: 4 cost, 1 spark, Materialized: Gain 3 energy
- Keeper of Forgotten Light: 6 cost, 2 spark, Materialized: Draw 2

**Analysis:** 4 cost, 3 spark is an excellent base rate. No existing 4-cost character has 3 spark except Angel of the Eclipse (4 cost, 3 spark, Musician, when materialize ally gain 1 energy) and Urban Cipher (4 cost, 3 spark, Materialized: discard 2, draw 2). So the base stats are at the top of the 4-cost range.

"Judgment: Each ally that survived since last turn gains +1 spark until end of turn" is a strong mass pump. In a persistent board of 4 allies (all survived), this is +4 spark per Judgment on top of the 3 spark from the Sentinel itself. With the Sentinel surviving (which it should, at 3 spark), it buffs itself too (+1 on subsequent turns).

**Strictly better check:** This has better base stats than most 4-drops AND a strong ability. There is no 4-cost character that combines 3 spark with a mass pump. Angel of the Eclipse has 3 spark but its ability (gain 1 energy per materialization) is narrower and conditional.

**Format-warping potential:** Medium-High in Stone decks. 3 spark at 4 cost with a board-wide pump is among the best rates in the format. Non-Stone decks that value board permanence also want this.

**Power Assessment: SLIGHTLY OVERPOWERED**

**Recommendation:** Reduce spark to 2 (matching the 4-cost norm) to compensate for the strong Judgment ability. At 3 spark, the card is individually excellent before even considering its ability, which makes it an auto-include in any Stone-containing deck.

---

#### 28. Tidechannel Observer | 3 cost | 1 spark | Ruin | Ancient

**Comparable existing cards:**
- Apocalypse Vigilante: 3 cost, 1 spark, When you discard a card, gain 1 point
- Ebonwing: 1 cost, 1 spark, Materialized/Judgment: Kindle 1
- Horizon Follower: 2 cost, 1 spark, Judgment: Gain 1 point

**Analysis:** "Judgment: If 3+ cards entered void this turn, gain 2 points and kindle 1." The threshold of 3 void entries is easy in Undertow (mill 3-8 per turn), moderate in Cinder (2-3 sacrifices), and hard in other decks.

2 points per Judgment is strong -- Horizon Follower gives 1 point per Judgment for 2 cost. Tidechannel Observer gives 2 points plus kindle 1 for 3 cost, but requires the void-velocity threshold. In Undertow where the threshold is trivially met, this is a premium value card.

Compare to Apocalypse Vigilante (3 cost, 1 spark, 1 point per discard): Vigilante scales linearly with discards, while Tidechannel Observer has a binary threshold. In Eclipse (3+ discards per turn), Vigilante might generate 3-4 points; Tidechannel Observer always gives 2 (but adds kindle). Tidechannel Observer has a lower ceiling but a more reliable floor in its target archetype.

**Format-warping potential:** Low-Medium. Excellent in Undertow/Cinder, unplayable in decks without void velocity. The threshold properly gates the effect.

**Power Assessment: APPROPRIATE**

---

#### 29. Fading Resonant | 2 cost | 1 spark | Zephyr | Visitor

**Comparable existing cards:**
- Bloomweaver: 1 cost, 1 spark, When you materialize a character, gain 1 energy
- Herald of the Last Light: 1 cost, 1 spark, Fast, Abandon: Prevent event
- Dawnprowler Panther: 1 cost, 1 spark, When materialize Spirit Animal, gain 1 energy

**Analysis:** "When ally leaves play, next character this turn costs 1 less." This is a conditional cost reduction that triggers on zone transitions. In Mirage with 2-3 flickers per turn, this saves 2-3 energy. In Cinder with 2-3 sacrifices, same.

Compare to Bloomweaver (1 cost, 1 spark, gain 1 energy per materialization): Bloomweaver gives energy on entry, Fading Resonant gives cost reduction on exit. Both facilitate deploying more characters. Bloomweaver is cheaper (1 vs 2 cost) but less flexible (only triggers on materializations, not all "leaves play" events).

The 1-cost-reduction-per-trigger is modest. You need multiple triggers in a turn to get significant value. In a typical turn with 1 flicker, this saves 1 energy -- decent but not broken.

**Format-warping potential:** Low. A solid tempo card in Mirage/Gale/Cinder but not game-defining.

**Power Assessment: APPROPRIATE**

---

#### 30. Stormtrace Augur | 3 cost | 0 spark (variable) | Tide | Mage

**Comparable existing cards:**
- Spirit of Smoldering Echoes: 4 cost, * spark, +1 spark per event entering void
- Abomination of Memory: 5 cost, * spark, Spark = cards in void
- Stormtide Oracle: 4 cost, * spark (NEW), Spark = events in void + retrieval

**Analysis:** "Spark = events in void." This is NEARLY IDENTICAL to the new Stormtide Oracle's spark mechanic. Both count events in void. The differences:
- Stormtrace Augur: 3 cost, Mage, Tide, no other ability
- Stormtide Oracle: 4 cost, Ancient, Tide+Ember, Judgment: pay 2 to retrieve event from void

Having TWO cards with "spark = events in void" is a redundancy issue. See the inter-card section below.

On its own merits: 3 cost for a body whose spark = events in void is very efficient. By mid-game in Tempest, 5-7 events in void makes this a 3-cost, 5-7 spark body -- far better than Abomination of Memory (5 cost for spark = ALL void cards) in event-heavy decks.

Compare to Spirit of Smoldering Echoes (4 cost, cumulative event tracking): Stormtrace Augur costs 1 less but uses current void count (so void hate reduces it). At 3 cost, this is potentially the most efficient variable-spark body in the game.

**Format-warping potential:** HIGH in Tempest. A 3-cost body that reaches 5-7 spark by mid-game is format-defining efficiency.

**Power Assessment: SLIGHTLY OVERPOWERED (also functionally near-duplicate of Stormtide Oracle)**

**Recommendation:** Either cut one of the two "spark = events in void" cards, or differentiate them more. If keeping both, Stormtrace Augur should cost 4 (matching Spirit of Smoldering Echoes) to prevent it from being the most efficient variable-spark body at 3 cost.

---

#### 31. Duskwatch Warden | 2 cost | 2 spark | Tide | Outsider

**Comparable existing cards:**
- Herald of the Last Light: 1 cost, 1 spark, Fast, Abandon: Prevent event
- Frost Visionary: 2 cost, 2 spark, Materialized: Draw 1
- Pallid Arbiter: 2 cost, 2 spark, Disable enemy Materialized

**Analysis:** "When you prevent a card, your next event costs 2 less." At 2 cost, 2 spark, this has excellent base stats (matching Frost Visionary and Pallid Arbiter, both premium 2-drops). The ability rewards Preventing, which is already something Depths and Gale do.

With Abolish (2 cost: Prevent a card) + Duskwatch Warden: Prevent costs 2, next event costs 2 less, netting 0 mana for the Prevent. This effectively makes Prevents free as long as you have a follow-up event.

In Gale with Sage of the Prelude (play fast card: draw 1): Prevent (fast) draws 1 from Sage AND reduces next event by 2. The chain: Prevent (free via discount from previous Prevent) -> draw from Sage -> play discounted fast event -> draw from Sage again. This creates a card-neutral chain that generates tempo.

**Broken interactions:** With Infernal Rest (lose 1 max energy: play for 0, Prevent): Prevent for free -> next event costs 2 less -> play a 2-cost fast event for free -> etc. Multiple free Prevents chaining into free events is very strong in Gale.

**Format-warping potential:** Medium. Strong in Depths and Gale but requires running Prevents (which takes deck slots). The 2 spark on a 2-cost body is already premium before the ability.

**Power Assessment: APPROPRIATE** (the 2-spark, 2-cost body is premium but the ability requires specific deck construction)

---

## Part 2: Cards Flagged for Balance Issues

### OVERPOWERED (Requires Changes)

| # | Card | Issue | Recommendation |
|---|------|-------|----------------|
| 1 | **Tideweaver Sentinel** | Materialized: Draw 2 at 3 cost is ~50% of Keeper of Forgotten Light's effect at 50% of the cost, plus a second mode | Increase cost to 4, or reduce draw to "Draw 1, Foresee 1" |
| 5 | **Cinder Ritualist** | "When card enters void, kindle 1" is too broad -- fires on mill, discard, sacrifice, event resolution. Kindle velocity is 4-8 per turn in Undertow/Eclipse | Restrict trigger to "When you abandon an ally or discard a card from your hand, kindle 1" or add "once per turn" |
| 15 | **Deepvault Warden** | Void characters cost 2 less makes Revenant of the Lost a 1-cost 6-spark play. Combined with Bedrock Anchor, creates resource-positive loops | Change to cost 1 less (not 2). At 1 less, Revenant is 2 cost (strong but fair) |
| 22 | **Dreamtide Cartographer** | Neutral 3-cost with Judgment: Draw 2 or mass +1 spark. Both modes are premium and every deck wants one | Make mono-resonance (not neutral), or reduce draw to 1, or increase cost to 4 |

### SLIGHTLY OVERPOWERED (Consider Adjustments)

| # | Card | Issue | Recommendation |
|---|------|-------|----------------|
| 2 | **Abyssal Reclaimer** | Self-fulfilling threshold (mills 3, needs 2 void entries) makes condition meaningless | Raise threshold to 4, or require void entries from other sources |
| 8 | **Galerunner** | 2-cost fast 3-spark body (conditional) is the most efficient fast threat | Reduce bonus to +1 spark, or threshold to 1 card in hand |
| 9 | **Eclipse Weaver** | 3 cost, 2 spark with repeatable event Reclaim + draw-on-void-event | Reduce to 1 spark, or add once-per-turn to draw trigger |
| 11 | **Ironveil Watcher** | Uncapped point scaling with Judgment count. With Conduit of Resonance, generates 4-6 points per materialization | Cap at 3 points per Judgment |
| 21 | **Risen Warden** | Self-Reclaim loop makes it unkillable by Dissolve | Add "a different Warrior" to Dissolved trigger |
| 23 | **Nexus of Passing** | Uncapped spark from zone changes reaches 10-15 in storm/flicker turns | Add spark cap of 8, or measure at half-rate |
| 26 | **Ember of Recurrence** | 1 energy per retrieval with 4-8 triggers per turn in Undertow is too much card advantage | Increase to 2 energy, or add once-per-turn |
| 27 | **Ironbark Sentinel** | 3 spark at 4 cost is top-of-curve before ability. Mass pump makes it auto-include in Stone | Reduce to 2 spark |
| 30 | **Stormtrace Augur** | 3 cost for spark=events-in-void is more efficient than comparable 4-5 cost scaling bodies. Also near-duplicate of Stormtide Oracle | Increase cost to 4, or cut in favor of Stormtide Oracle |
| 16 | **Ashen Threshold** | Permanent "when ally leaves play, gain 1 energy" on a body is Starlit Cascade without the "until end of turn" restriction | Add once-per-turn to draw trigger on void materialization |

### APPROPRIATE (No Changes Needed)

| # | Card | Notes |
|---|------|-------|
| 3 | Basalt Warden | Well-balanced threshold modes |
| 4 | Forgeborn Martyr | Opponent-facing trigger provides counterplay |
| 6 | Stormtide Oracle | Self-correcting retrieval-vs-spark tension |
| 7 | Depthswatcher | High setup cost, 0 spark floor |
| 10 | Bedrock Anchor | Fair on its own (see Deepvault Warden interaction) |
| 12 | Stoneheart Veteran | Expensive kindle rate, fair energy sink |
| 14 | Vanguard of the Summit | High threshold, bad floor |
| 17 | Voidthorn Sentinel | Telegraphed Prevent balances the body advantage |
| 18 | Resonance Siphon | Narrow without activated abilities |
| 19 | Kindlespark Harvester | Novel spark-for-removal tension |
| 20 | Echoing Departure | Modest per-trigger value |
| 24 | Crucible of the Commons | Self-limiting anti-synergy with lords |
| 25 | Archivist of Vanished Names | Stochastic mill with appropriate ceiling |
| 28 | Tidechannel Observer | Proper threshold gating |
| 29 | Fading Resonant | Modest cost reduction |
| 31 | Duskwatch Warden | Requires Prevent density |

### SLIGHTLY UNDERPOWERED (Consider Buffs)

| # | Card | Issue | Recommendation |
|---|------|-------|----------------|
| 13 | **Oathbound Sentinel** | Strictly worse than Ebonwing (1 cost, 1 spark, kindle 1/Judgment) in nearly all respects | Reduce cost to 1, or add 1 spark |

---

## Part 3: Inter-Card Interactions (New-to-New and New-to-Existing)

### Critical Interaction Clusters

#### Cluster 1: The Bedrock Resource Engine (BROKEN)

**Cards involved:** Bedrock Anchor (new) + Deepvault Warden (new) + any sacrifice/bounce effect

**Mechanism:** Deepvault Warden reduces void characters by 2. Bedrock Anchor is void-only, costs 2, gains 2 energy + draw 1 on play from void. With Deepvault Warden: Bedrock Anchor costs 0 from void, gains 2 energy and draws 1. Sacrifice Bedrock Anchor (it goes to void), replay from void for 0 cost, net +2 energy +1 draw each cycle.

**Severity:** HIGH. This is a repeatable 0-cost loop that generates 2 energy and draws 1 card per cycle, limited only by sacrifice outlets (which are abundant in Cinder/Bedrock).

**Fix:** Reduce Deepvault Warden to "cost 1 less" or remove the energy refund from Bedrock Anchor (change to "gain 1 energy" instead of "gain 2 energy").

---

#### Cluster 2: The Kindle Avalanche (CONCERNING)

**Cards involved:** Cinder Ritualist (new, kindle per void entry) + Searcher in the Mists (mill 4) + Flagbearer of Decay (mill 2 per Survivor) + any mill/discard enablers

**Mechanism:** Every card entering the void kindles 1. Undertow's natural mill rate of 6-10 cards per turn generates kindle 6-10 per turn. By turn 3, the leftmost character has 15-25 spark.

**Severity:** HIGH. Kindle accumulation at this rate outpaces every other scoring mechanism in the game by a factor of 3-4.

**Fix:** Restrict Cinder Ritualist's trigger (see recommendation above).

---

#### Cluster 3: Stormtide Oracle vs. Stormtrace Augur (REDUNDANCY)

**Cards involved:** Stormtide Oracle (signpost, 4 cost) + Stormtrace Augur (gap filler, 3 cost)

**Mechanism:** Both have "spark = events in void." Having two cards with nearly identical scaling in the same card pool creates redundancy. Stormtrace Augur is strictly cheaper (3 vs 4) with no additional ability, making it the more efficient body. Why would you play Stormtide Oracle (4 cost, same spark scaling, Judgment retrieval that REDUCES your spark) when Stormtrace Augur does the same thing for 1 less energy?

**Severity:** MEDIUM. The retrieval ability on Stormtide Oracle is unique but self-punishing (reduces spark). Most Tempest players would prefer the cheaper body.

**Fix:** Keep Stormtide Oracle (4 cost, event-void-counting + retrieval) and cut Stormtrace Augur, OR differentiate Stormtrace Augur (e.g., "spark = events in void with cost 2 or less" to reward cheap event chains differently).

---

#### Cluster 4: Mass Kindle Concentration

**Cards involved:** Cinder Ritualist (kindle per void entry) + Oathbound Sentinel (kindle per turn survived) + Tidechannel Observer (kindle 1 on void velocity) + Stoneheart Veteran (pay 3: kindle 2) + existing kindle (Ebonwing, Spirit Field Reclaimer, Exiles of the Last Light, Infernal Ascendant)

**Mechanism:** The new cards add 4 new kindle sources to the existing 4, totaling 8 kindle generators. The leftmost character could accumulate 5-10 kindle per turn with multiple sources active. The proposed Kindlespark Harvester converts this into repeatable removal.

**Severity:** LOW-MEDIUM. The kindle ecosystem is intentionally expanding, and individual cards are balanced. But the aggregate kindle rate across all new cards may push kindle strategies from "strong" to "dominant." Monitor in playtesting.

**Fix:** No immediate fix needed, but watch the aggregate kindle rate. If Cinder Ritualist's trigger is restricted (per recommendation), the cluster becomes manageable.

---

#### Cluster 5: Tideweaver Sentinel Flicker Loop

**Cards involved:** Tideweaver Sentinel (new, Materialized: Draw 2 or bounce+energy) + Flickerveil Adept (existing, Judgment: flicker spark 2 or less) + Passage Through Oblivion (existing, 1 cost: flicker)

**Mechanism:** Tideweaver Sentinel has 1 spark, so it can be flickered by Flickerveil Adept every Judgment. Each flicker draws 2 or generates bounce+energy. With Flickerveil Adept, this is a guaranteed draw-2 per Judgment, every Judgment, automatically. Compare to The Calling Night (draw 1 per Judgment but opponent gains 2 points): Tideweaver draws TWICE as much with NO downside.

**Severity:** HIGH. Automatic draw-2 per Judgment from a two-card combo (both at common/uncommon rarity and reasonably costed) is format-warping card advantage.

**Fix:** Increase Tideweaver Sentinel's cost to 4 or reduce draw to 1.

---

#### Cluster 6: Ironveil Watcher + Conduit of Resonance

**Cards involved:** Ironveil Watcher (new, points per Judgment trigger) + Conduit of Resonance (existing, materialize -> trigger all Judgment abilities)

**Mechanism:** Each materialization triggers all Judgment abilities. Ironveil Watcher counts those triggers and gains points. With 3 other Judgment creatures and Conduit of Resonance, each materialization generates 3 points from Ironveil Watcher alone, plus the actual Judgment abilities firing. In Basalt, materializing 2-3 Spirit Animals per turn generates 6-9 points per turn just from Ironveil Watcher.

**Severity:** MEDIUM. Requires Conduit of Resonance (5 cost, Rare) + Ironveil Watcher + 3 Judgment creatures. This is a 4+ card combo but achievable in Basalt.

**Fix:** Cap Ironveil Watcher at 3 points per phase.

---

#### Cluster 7: Eclipse Weaver + Ashmaze Guide Double Reclaim

**Cards involved:** Eclipse Weaver (new, discard to grant event Reclaim) + Ashmaze Guide (existing, discard grants Reclaim equal to cost)

**Mechanism:** Discarding a card with both in play: Ashmaze Guide gives the discarded card Reclaim, AND Eclipse Weaver gives a void event Reclaim. Each discard generates TWO Reclaim effects. With draw-discard cycling (Moonlit Voyage: draw 2, discard 2), each cycle generates 4 Reclaim effects.

**Severity:** LOW-MEDIUM. Strong but requires two specific 3-cost bodies in play simultaneously. The Reclaim effects still require paying mana to use.

**Fix:** No immediate fix needed. Monitor in playtesting.

---

## Part 4: Summary and Severity Rankings

### Highest Priority Fixes (Block Printing)

1. **Tideweaver Sentinel** -- Draw 2 at 3 cost is format-warping in flicker. Reduce draw or increase cost.
2. **Cinder Ritualist** -- Kindle-per-void-entry is too broad. Restrict trigger to specific actions.
3. **Deepvault Warden** -- 2 cost reduction on void characters breaks Revenant of the Lost and Bedrock Anchor. Change to 1 less.
4. **Dreamtide Cartographer** -- Neutral draw-2-or-pump is universally dominant. Make mono-resonance or nerf.

### Second Priority Fixes (Strongly Recommended)

5. **Risen Warden** -- Self-Reclaim loop creates unkillable body. Add "a different Warrior."
6. **Stormtrace Augur** -- Near-duplicate of Stormtide Oracle at a better rate. Cut or differentiate.
7. **Nexus of Passing** -- Uncapped zone-change spark reaches degenerate levels. Add a cap.
8. **Ironbark Sentinel** -- 3 spark at 4 cost is too efficient before ability. Reduce to 2 spark.

### Third Priority Fixes (Suggested Tweaks)

9. **Galerunner** -- +2 spark bonus makes it too efficient in Gale.
10. **Eclipse Weaver** -- 2 spark with engine ability is generous. Reduce to 1 spark.
11. **Ironveil Watcher** -- Cap point generation at 3 per phase.
12. **Ember of Recurrence** -- Increase retrieval cost to 2 energy or add once-per-turn.
13. **Abyssal Reclaimer** -- Raise void-entry threshold so it is not auto-fulfilled.
14. **Ashen Threshold** -- Add once-per-turn to void-materialization draw.
15. **Oathbound Sentinel** -- Buff to 1 cost or 1 spark to compete with Ebonwing.

### Cards That Are Well-Designed

The following cards exemplify good balance decisions and should be preserved as-is: Basalt Warden, Forgeborn Martyr, Stormtide Oracle, Depthswatcher, Bedrock Anchor (on its own), Stoneheart Veteran, Vanguard of the Summit, Voidthorn Sentinel, Resonance Siphon, Kindlespark Harvester, Echoing Departure, Crucible of the Commons, Archivist of Vanished Names, Tidechannel Observer, Fading Resonant, Duskwatch Warden.

---

## Part 5: Aggregate Observations

### Cost Curve Distribution

| Cost | New Cards | Notes |
|------|-----------|-------|
| 2 | 7 cards (23%) | Slightly high density at the 2-drop slot |
| 3 | 16 cards (52%) | VERY high concentration at 3 cost |
| 4 | 8 cards (26%) | Adequate |

**Concern:** 52% of new cards cost 3. The existing pool has a more distributed curve. This creates a glut at the 3-drop slot where new cards compete with each other AND with existing premium 3-drops (Nocturne, Assault Leader, Cloaked Sentinel, Moonlit Dancer). Consider shifting 2-3 designs to cost 1 or cost 5+ to diversify the curve.

### Spark Distribution

| Spark | Count | Notes |
|-------|-------|-------|
| 0 | 6 cards | Appropriate for engine pieces |
| 1 | 14 cards | Standard for utility creatures |
| 2 | 7 cards | Premium bodies |
| 3 | 1 card (Ironbark Sentinel) | Flag: highest non-variable spark |
| Variable (*) | 3 cards | Scaling finishers |

### Resonance Distribution

| Resonance | Count |
|-----------|-------|
| Stone | 8 (including duals) |
| Ruin | 5 (including duals) |
| Tide | 5 (including duals) |
| Zephyr | 4 (including duals) |
| Ember | 3 (including duals) |
| Neutral | 3 |
| Dual | 10 (signpost only) |

Stone receives the most new cards (8), which addresses the documented 9-card deficit. Ember receives the fewest (3), which may warrant monitoring since Ember has several archetypes that need support.

### Overall Assessment

Of 31 proposed cards: 4 are overpowered and require changes before implementation, 11 are slightly over/under-powered and would benefit from tuning, and 16 are appropriately balanced. The overpowered cards cluster around two failure modes: (1) effects that are too cheap for their output (Tideweaver Sentinel's draw-2 at 3 cost, Deepvault Warden's 2-cost reduction), and (2) triggers that are too broad (Cinder Ritualist's "any void entry" kindle, Dreamtide Cartographer's universal neutral flexibility). The fixes are straightforward: increase costs, narrow triggers, or add rate limiters.

The inter-card interactions between new cards are generally well-managed, with the notable exception of the Bedrock Anchor + Deepvault Warden resource loop, which must be fixed before these cards can coexist.
