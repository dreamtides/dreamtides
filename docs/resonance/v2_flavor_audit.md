# V2 Flavor Coherence Audit -- Agent 3

## Methodology

Every proposed card (31 total) is evaluated against the existing 222-card pool for:
1. **Name quality** -- Does it match Dreamtides' naming conventions?
2. **Mechanical complexity** -- Appropriate for its rarity?
3. **Resonance fit** -- Do the mechanics genuinely feel like the assigned resonance(s)?
4. **Dual-resonance coherence** (10 duals only) -- Emergent bridge or stapled keywords?
5. **Parasitic design** -- Does it need other new cards to function?
6. **Keyword density** -- How many distinct mechanics does it reference?

### Naming Convention Reference

Dreamtides names cluster into several patterns drawn from the existing 222:

- **[Nature/Cosmic Noun] + [of/the] + [Poetic Abstraction]:** Keeper of Forgotten Light, Spirit of Smoldering Echoes, Whisper of the Past, Seeker of the Radiant Wilds, Torchbearer of the Abyss, Avatar of Cosmic Reckoning, Wraith of Twisting Shadows, Prophet of the Consumed, Angel of the Eclipse, Oracle of Shifting Skies, Beacon of Tomorrow
- **Compound Adjective + Role Noun:** Moonlit Dancer, Astral Navigators, Ethereal Trailblazer, Cosmic Puppeteer, Dawnblade Wanderer, Flickerveil Adept, Dreamborne Leviathan, Soulflame Predator
- **Simple Evocative Name:** Nocturne, Lumineth, Ebonwing, Shadowpaw, Apocalypse, Desperation, Immolate
- **The [Title]:** The Rising God, The Devourer, The Bondweaver, The Forsaker, The Ringleader, The Waking Titan, The Dread Sovereign, The Calling Night
- **Resonance/Dream-themed:** Dreamscatter, Starlit Cascade, Passage Through Oblivion, Portal of Twin Paths

Key observations: Names tend toward the ethereal, dreamlike, or cosmic. They avoid pure-fantasy tropes (no "Dragonslayer" or "Goblin King"). Creature names often suggest transience, memory, or liminality. Event names tend toward the abstract or atmospheric.

---

## Per-Card Evaluations

### A. Signpost Dual Cards (10 cards)

#### 1. Tideweaver Sentinel (Tide+Zephyr, Uncommon, 3 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Tideweaver" follows the compound-adjective pattern (cf. Flickerveil Adept, Dreamborne Leviathan). "Sentinel" is a common but fitting role noun. Evokes the Tide resonance directly. |
| Complexity | Appropriate | Choose-one modal at uncommon is clean. Two distinct modes, each straightforward. Comparable to Break the Sequence (uncommon, choose-one). |
| Resonance Fit | Strong | Draw-2 is Tide. Bounce-to-hand + energy is Zephyr motion + resource generation. Both modes feel natural. |
| Dual Coherence | Strong | The two modes represent the two resonances, but the CHOICE between them creates emergent decision-making based on game state. Not stapled -- the modal structure means you never get both at once, forcing you to read the game. |
| Parasitic | No | Works with existing flicker targets (Frost Visionary, Looming Oracle) and bounce-replay patterns already in the pool. |
| Keyword Density | 2 (Materialized, draw/bounce) | Clean. |

**Verdict: Pass. Well-designed signpost.**

---

#### 2. Abyssal Reclaimer (Tide+Ruin, Uncommon, 3 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Abyssal" fits the void/depth themes of Ruin. "Reclaimer" is a resonant word in Dreamtides vocabulary (cf. Reclaimer of Lost Paths, Chronicle Reclaimer, Scrap Reclaimer, Grim Reclaimer). Consistent naming. |
| Complexity | Appropriate | Mill-3 plus a conditional reclaim at uncommon. One condition to track ("2 or more cards entered void this turn"), trivially self-satisfied. This is comparable to Nocturne (common) in text density, reasonable for uncommon. |
| Resonance Fit | Strong | Self-mill is the Tide+Ruin boundary. Void-to-hand reclaim is Ruin. The threshold mechanic rewards void velocity, which is the Undertow identity. |
| Dual Coherence | OK | The mill is Tide-adjacent (Undertow), the reclaim is Ruin. The threshold ties them together: the mill itself satisfies the threshold for reclaim. This is coherent but somewhat linear -- mill triggers reclaim triggers more mill on next flicker. The "emergent" behavior is a self-fueling loop, which is functional but not deeply surprising. |
| Parasitic | No | Synergizes with Abomination of Memory, Weight of Memory, Searcher in the Mists -- all existing cards. |
| Keyword Density | 2 (Materialized, mill/reclaim) | Clean. |

**Verdict: Pass. Solid if unspectacular Undertow signpost.**

---

#### 3. Basalt Warden (Zephyr+Stone, Uncommon, 3 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | OK | "Basalt" directly names the archetype, which is unusual for existing cards -- no existing card is called "Mirage [X]" or "Crucible [X]." This is a minor flag: it reads more like a label than an evocative name. Compare to existing Spirit Animals like "Ghostlight Wolves" or "Sunshadow Eagle" -- those evoke imagery, while "Basalt Warden" evokes the archetype name. However, basalt IS a real geological term, so it works as a nature-word too. |
| Complexity | Appropriate | Threshold-gated binary at uncommon. One board-state check (3+ allies), two clean outcomes. Comparable to Wolfbond Chieftain's conditional (2+ Warriors). |
| Resonance Fit | Strong | Energy generation on Judgment is quintessential Stone. Temporary spark pump is Stone's Judgment-phase identity. The board-width check rewards Zephyr's go-wide strategy. |
| Dual Coherence | Strong | The threshold reads the board state that both resonances naturally create (Zephyr goes wide, Stone generates energy). The mode switch is automatic and context-dependent, not player-chosen, which makes the card feel organic rather than stapled. A wide board gets energy (fuel for Stone's activated abilities); a thin board gets spark (emergency scoring). This IS emergent. |
| Parasitic | No | Works with Ghostlight Wolves, Spiritbound Alpha, Assault Leader -- all existing. |
| Keyword Density | 2 (Judgment, energy/spark) | Clean. |

**Verdict: Pass. The name could be more evocative but the design is sound.**

---

#### 4. Forgeborn Martyr (Stone+Ember, Rare, 3 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Forgeborn" compounds forge (Ember/Stone craft imagery) with born (emergence). "Martyr" signals the death-matters trigger. Fits the slightly mythic Dreamtides register. Compare to "Ashen Avenger" -- similar construction, similar resonance. |
| Complexity | Appropriate | Two abilities (Judgment pump + dissolved trigger) at rare. Each ability is individually simple. The cognitive load comes from the strategic interaction between them (keep Warriors alive for pump vs. accept deaths for resources), which is desirable complexity at rare. Compare to Speaker for the Forgotten (rare, two triggered abilities). |
| Resonance Fit | Strong | Judgment pump is Stone. Death-trigger resource generation is Ember's sacrifice-adjacent space. The "by the opponent" clause correctly prevents Stone from gaining self-sacrifice payoffs. |
| Dual Coherence | Strong | This is a genuinely emergent design. The Judgment pump makes Warriors worth keeping alive; the death trigger makes their death less painful. The OPPONENT faces a dilemma (kill Warriors and fuel the Crucible player, or leave them alive and watch them grow). Neither ability alone creates this tension -- it requires both on one card. This is not stapled design; it is interlocking design. |
| Parasitic | No | Skyflame Commander, Blade of Unity, Wolfbond Chieftain -- the entire existing Warrior infrastructure synergizes. |
| Keyword Density | 3 (Judgment, spark pump, dissolved trigger) | Acceptable for rare. |

**Verdict: Pass. One of the best-designed cards in the batch.**

---

#### 5. Cinder Ritualist (Ember+Ruin, Rare, 3 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Cinder" names the archetype (same concern as Basalt Warden), but "Cinder Ritualist" reads more naturally as a character concept -- a ritualist working with cinders/ashes. "Ritualist" fits the dark-magic register. Compare to "Rebirth Ritualist" in the existing pool -- very close naming, which is either a good echo or a collision. |
| Complexity | High | Two distinct abilities (void-entry kindle + once-per-turn abandon-to-reclaim), each with conditions and constraints. The "different character" clause on the reclaim adds tracking. For a rare this is acceptable, but it is at the upper bound. Compare to Obliterator of Worlds (rare, two abilities with conditions) -- similar density. |
| Resonance Fit | Strong | Abandon-sacrifice is Ember. Void-entry and reclaim from void are Ruin. Kindle bridges both (Ember aggression expressed through Ruin's void cycling). |
| Dual Coherence | Strong | The two abilities create a genuine loop: abandon an ally (enters void, kindle 1), then retrieve a different character from void. The "any zone" on the kindle trigger means it works differently in Cinder (sacrifice), Undertow (mill), and Eclipse (discard). This is not stapled -- the kindle trigger and the retrieval create a sacrifice-recursion engine that neither ability enables alone. |
| Parasitic | No | Infernal Ascendant, Exiles of the Last Light, Harvester of Despair -- extensive existing sacrifice infrastructure. |
| Keyword Density | 3 (kindle, abandon, reclaim-from-void) | Acceptable for rare. |

**FLAG: Name collision.** "Cinder Ritualist" vs. existing "Rebirth Ritualist" -- two Ritualists is fine but the "Cinder" prefix directly names the archetype. Recommend considering a rename like "Ashfire Ritualist" or "Ritualist of Smoldering Echoes."

**Verdict: Pass with minor name concern.**

---

#### 6. Stormtide Oracle (Tide+Ember, Rare, 4 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Stormtide" evokes the Tempest (Tide+Ember) archetype without naming it. "Oracle" fits the Ancient subtype and Dreamtides' mythic register. Compare to "Oracle of Shifting Skies" -- similar pattern, complementary rather than colliding. |
| Complexity | Appropriate | Variable spark (common in the pool: Abomination of Memory, Blade of Unity, Spirit of Smoldering Echoes) plus a Judgment activated ability. The tension between retrieving events (shrinks void = less spark) and keeping spark is elegant and rare-appropriate. |
| Resonance Fit | Strong | Counting events in void rewards Tide's event play. The Judgment retrieval is a controlled form of recursion. The * spark scaling is Ember's "intensity" expressed through accumulated event history. |
| Dual Coherence | Strong | This card creates a unique tension: events in void grow the body (rewarding cumulative play), but retrieving events shrinks it (costing spark for tactical flexibility). This tension does not exist on any other card and requires the intersection of Tide (events, retrieval) and Ember (aggressive spark scaling). Excellent emergent design. |
| Parasitic | No | Spirit of Smoldering Echoes, Starcatcher, Keeper of the Lightpath -- existing Tempest infrastructure feeds this naturally. |
| Keyword Density | 2 (variable spark, Judgment retrieval) | Clean for rare. |

**Verdict: Pass. Strong design with genuine decision-making.**

---

#### 7. Depthswatcher (Tide+Stone, Rare, 4 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | OK | Single compound word, evocative but somewhat generic. Compare to existing single-word names like "Ebonwing," "Shadowpaw," "Nocturne" -- those have stronger imagery. "Depthswatcher" is functional but slightly flat. It sounds more like a video game mob than a Dreamtides character. Consider "Sentinel of the Fathoms" or "Watcher of the Abyss" for a more atmospheric register. |
| Complexity | High | Three mechanical elements: hand-size threshold for kindle, Prevent-trigger for energy, and 0 starting spark requiring kindle to score. Each element is simple but there are three interactions to track. At rare this is acceptable but dense. |
| Resonance Fit | Strong | Hand-size-matters is Tide (card accumulation). Prevent-trigger is Tide (reactive denial). Kindle and energy generation are Stone. 0 starting spark requiring patience to grow is Stone's permanence theme. |
| Dual Coherence | Strong | The hand-size kindle rewards Tide's natural card accumulation. The Prevent-trigger converts Tide's reactive play into Stone's resource generation. The 0 starting spark means the card must stay in play to matter -- Stone's permanence. All three elements reinforce the Depths play pattern (hold cards, play reactively, accumulate value over time). This is not stapled; it is a card that makes you WANT to play the Depths game. |
| Parasitic | No | 9 existing Prevent effects in the pool. Existing draw effects maintain hand size. |
| Keyword Density | 3 (kindle, Prevent-trigger, hand-size threshold) | Upper bound for rare, acceptable. |

**Verdict: Pass with a name quality note. Strong mechanically.**

---

#### 8. Galerunner (Zephyr+Ember, Uncommon, 2 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | OK | "Galerunner" directly names the Gale archetype. Same concern as Basalt Warden and Cinder Ritualist. In isolation it reads as a nature-compound (one who runs on gales), but in context it is archetype-labeling. The existing pool avoids this -- "Sage of the Prelude" does not label itself as a "Gale Sage." Consider "Windstride Runner" or "Galecrest Courier." |
| Complexity | Appropriate | Hellbent threshold (+2 spark) plus an activated ability (abandon 0-spark for draw). Two clean mechanics at uncommon. Compare to Dreadcall Warden (rare, similar cost/activation structure). This might actually be slightly high for uncommon -- the hellbent tracking + activated ability + 0-spark targeting creates three things to monitor. |
| Resonance Fit | Strong | Fast Character is Zephyr. Hellbent (empty hand) rewards is Ember's reckless aggression. Abandon-for-draw bridges Ember sacrifice to Zephyr motion. |
| Dual Coherence | Strong | The tension between keeping hand empty (for +2 spark) and drawing (via abandon) is genuinely interesting. The card punishes you for drawing (turns off the bonus) but rewards you for sacrificing (generates draw). This creates a push-pull dynamic unique to the Zephyr+Ember intersection. |
| Parasitic | No | Existing 0-spark bodies (Wolfbond Chieftain, Ethereal Trailblazer, Aspiring Guardian, Oathbound Sentinel if also in the set) provide abandon targets. |
| Keyword Density | 3 (Fast, hellbent threshold, abandon activated) | Upper bound for uncommon. |

**FLAG: Slight complexity concern for uncommon.** Three mechanical elements is pushing it. Consider whether the hellbent threshold or the abandon ability alone would carry the card, and simplify.

**Verdict: Pass with complexity and naming notes.**

---

#### 9. Eclipse Weaver (Zephyr+Ruin, Rare, 3 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | OK | "Eclipse" directly names the archetype. Same issue as above. "Weaver" is a fine role noun. But "Eclipse Weaver" reads as archetype + generic noun. Compare to the existing "Angel of the Eclipse" -- that card uses Eclipse as a cosmic phenomenon, not an archetype label. "Eclipse Weaver" could similarly be read as weaving eclipses (cosmic imagery), which is better. Borderline acceptable. |
| Complexity | High | Once-per-turn activated discard, grants Reclaim to an event, then a separate trigger (draw on void-event play). Two distinct mechanics with memory tracking ("this turn" on the Reclaim, "from your void" condition on draw). Dense for rare but within bounds. |
| Resonance Fit | Strong | Discard-as-cost is Zephyr cycling. Granting Reclaim is Ruin recursion. Draw on playing from void bridges both. |
| Dual Coherence | Strong | The discard-to-Reclaim loop creates a genuine engine: discard (Zephyr) -> grant Reclaim (Ruin) -> play from void (Ruin) -> draw (cycle continues). Each step feeds the next. This is not stapled -- it is a three-step engine that requires both resonances to function. |
| Parasitic | No | Ashmaze Guide exists as a comparable effect. Existing events with Reclaim already establish the pattern. |
| Keyword Density | 3 (discard, Reclaim, draw-on-void-play) | Acceptable for rare. |

**Verdict: Pass. Good engine design, borderline name.**

---

#### 10. Bedrock Anchor (Stone+Ruin, Uncommon, 2 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | OK | "Bedrock" directly names the archetype. "Anchor" fits the Stone permanence theme but is a bit generic. Compare to "Revenant of the Lost" (the other void-only character) -- that name is far more evocative. "Bedrock Anchor" sounds like a fantasy-game item name, not a character. Consider "Anchor of the Forgotten" or "Rootheart Sentinel." |
| Complexity | High | Three mechanics: void-only play restriction, Materialized-from-void energy+draw trigger, and Judgment kindle with a "since your last turn" presence check. This is a lot for uncommon. Revenant of the Lost (uncommon, void-only) has one mechanic; this has three. |
| Resonance Fit | Strong | Void-only is Ruin. Energy generation and kindle-on-persistence are Stone. The "since your last turn" check is a novel Stone mechanic that rewards board permanence. |
| Dual Coherence | OK | The void-only restriction (Ruin) combines with persistence-kindle (Stone) to create a card that must enter from the void but rewards staying. The energy+draw on void-entry is a sweetener. The design is coherent but the three separate mechanics feel more like a checklist than an emergent interaction. The kindle-on-persistence could stand alone as a card; the void-only + ETB trigger could stand alone as a different card. Putting them together does not create a behavior that neither creates alone. |
| Parasitic | No | Works with existing void-filling (Harvest the Forgotten, Searcher in the Mists) and Reclaim effects. |
| Keyword Density | 4 (void-only, Materialized conditional, energy+draw, Judgment kindle with presence check) | **Overloaded for uncommon.** |

**FLAG: Complexity overload for uncommon.** Four distinct mechanics on an uncommon card is too much. Recommend removing one element -- either the Materialized energy+draw trigger or the Judgment kindle. The card tries to do too many things. Compare to Revenant of the Lost (uncommon): one mechanic, clean, memorable.

**Verdict: Flagged. Needs simplification for its rarity.**

---

### B. Mono-Stone Cards (5 cards)

#### 11. Ironveil Watcher (Stone, Uncommon, 2 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Ironveil" is a strong compound (iron = Stone, veil = dreamlike mystery). "Watcher" is a common role noun but fits the Ancient subtype. Compare to "Pallid Arbiter" or "Cloaked Sentinel" -- similar pattern, similar quality. |
| Complexity | Appropriate | One Judgment trigger that counts other Judgment triggers. Simple to understand, scales with board state. A clean uncommon. |
| Resonance Fit | Strong | Judgment-phase scaling is quintessential Stone. Points generation from board infrastructure fits Stone's incremental value theme. |
| Parasitic | No | Existing Judgment triggers are abundant (Wolfbond Chieftain, Dawnblade Wanderer, Spirit Field Reclaimer, Ghostlight Wolves, Luminwings, etc.). |
| Keyword Density | 1 (Judgment) | Minimal. Clean. |

**Verdict: Pass. Clean design, good name.**

---

#### 12. Stoneheart Veteran (Stone, Uncommon, 3 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Stoneheart" evokes Stone's permanence. "Veteran" fits the Warrior subtype and suggests experience/endurance. Compare to "Dawnblade Wanderer" or "Frost Visionary" -- similar compound-adjective + noun pattern. |
| Complexity | Appropriate | One activated ability on Judgment ("may pay 3: kindle 2"). Clean decision point at uncommon. Compare to Spirit Field Reclaimer (uncommon, "may pay 1: kindle 1 + banish void card"). |
| Resonance Fit | Strong | Energy-to-kindle conversion on Judgment is pure Stone (ramp + permanence + incremental growth). |
| Parasitic | No | Any energy-generating card feeds this. |
| Keyword Density | 2 (Judgment, kindle) | Clean. |

**Verdict: Pass. Solid Stone design.**

---

#### 13. Oathbound Sentinel (Stone, Common, 2 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Oathbound" evokes commitment and permanence -- Stone's theme. "Sentinel" is the third Sentinel in this batch (along with Tideweaver Sentinel, Ironbark Sentinel), and the existing pool has zero Sentinels. Three new Sentinels is a naming collision that dilutes distinctiveness. |
| Complexity | Appropriate | One trigger: start-of-turn kindle if it survived. Very simple for common. Comparable to Ethereal Trailblazer (common, "Judgment: gain 1 energy") in simplicity. |
| Resonance Fit | Strong | Continuous-presence reward is the purest possible expression of Stone's permanence theme. The "since your last turn" check anti-synergizes with Zephyr flicker, creating clean resonance separation. |
| Parasitic | No | Works on any board. Just needs to survive. |
| Keyword Density | 1 (kindle) | Minimal. |

**FLAG: Naming collision.** Three Sentinels in one batch (Tideweaver Sentinel, Oathbound Sentinel, Ironbark Sentinel) plus Voidthorn Sentinel from cross-pollination. Four Sentinels total. The existing pool has zero. This is excessive repetition. Recommend renaming at least two of the four to use different role nouns.

**Verdict: Pass mechanically. Naming collision needs addressing.**

---

#### 14. Vanguard of the Summit (Stone, Rare, 4 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Vanguard of the Summit" follows the "[Role] of the [Place]" pattern (cf. "Keeper of Forgotten Light," "Seeker of the Radiant Wilds"). "Summit" evokes Stone's height/permanence. Strong, evocative name. |
| Complexity | Appropriate | One conditional trigger: "when you play your third character this turn, draw 2 and gain 2 energy." Single threshold, clean payoff. The planning puzzle (holding characters for burst deployment) is desirable rare complexity. |
| Resonance Fit | Strong | Rewarding multiple deployments through cost reduction/ramp is Stone's territory. Draw-2 bends Stone's "no primary draw" constraint, but the 3-character threshold is high enough that it functions as a reward for Stone's infrastructure, not as a draw engine. |
| Parasitic | No | Nexus Wayfinder (existing cost reduction) and cheap Warriors already enable the threshold. |
| Keyword Density | 1 (deployment threshold) | Clean. |

**Verdict: Pass. Strong design and name.**

---

#### 15. Deepvault Warden (Stone, Uncommon, 3 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Deepvault" is evocative -- a deep vault suggests Stone's depth and permanence, with a hint of Ruin's underground. "Warden" is the FIFTH "Warden" in this batch (Basalt Warden, Duskwatch Warden, Deepvault Warden, Ironbark Sentinel... actually just three Wardens). Still, three Wardens plus four Sentinels is seven guard-role names out of 31 cards. |
| Complexity | Appropriate | One static effect: "Characters you play from your void cost 2 less." Clean, parseable, uncommon-appropriate. Compare to Nexus Wayfinder (rare, "Characters cost you 2 less" -- more general). |
| Resonance Fit | OK | Cost reduction is Stone. But "characters from your void" is a void-interaction mechanic, which the resonance identity document says Stone does NOT do ("Not void interaction"). The design document justifies this by saying the cost reduction is Stone (where the reduction applies is incidental), but this feels like a stretch. A Stone card that specifically rewards void play is functionally a Bedrock card that happens to be mono-Stone. |
| Parasitic | No | Revenant of the Lost, Ashen Avenger, Kindred Sparks -- existing void-play characters benefit immediately. |
| Keyword Density | 1 (cost reduction) | Clean. |

**FLAG: Resonance boundary tension.** A mono-Stone card whose entire purpose is enabling void plays sits uncomfortably on the Stone/Ruin boundary. The resonance identity document explicitly states Stone is "Not void interaction." While the designer argues cost reduction is inherently Stone, the card's value is zero without void interactions, making it functionally a Ruin card in Stone clothing. Consider making this Stone+Ruin dual instead of mono-Stone, or reframing the cost reduction to not be void-specific (e.g., "Characters you play that did not start in your hand cost 2 less" -- covers void AND top-of-deck play for Basalt's Dreamborne Leviathan).

**Verdict: Flagged for resonance fit concern.**

---

### C. Cross-Pollination Cards (6 cards)

#### 16. Ashen Threshold (Ember, Uncommon, 3 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Ashen" evokes Ember's destructive aftermath. "Threshold" suggests a doorway between states -- fitting for a card about zone transitions. Dreamtides uses "Ashen" in "Ashen Avenger," "Ashen Remnant," and "Ashmaze Guide," so the prefix is well-established. |
| Complexity | Appropriate | Two triggered abilities, each individually simple. "Ally leaves play" and "materialize from void" are clean zone-change triggers. Uncommon-appropriate. |
| Resonance Fit | Strong | "Ally leaves play" is the zone change Ember's sacrifice engine produces. Void-materialization draw bridges to Ruin territory through Ember's transient nature. |
| Parasitic | No | Works with any existing flicker, bounce, sacrifice, or recursion effect. |
| Keyword Density | 2 (leaves-play trigger, void-materialize trigger) | Clean. |

**Verdict: Pass. Well-designed bridge card.**

---

#### 17. Voidthorn Sentinel (Ruin, Uncommon, 2 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Voidthorn" is evocative -- thorns from the void, defensive and dangerous. Fourth Sentinel in the batch (see naming collision flag above). |
| Complexity | Appropriate | Self-sacrifice Prevent plus a Prevent-trigger void-fill rider. Two mechanics that chain naturally. Uncommon-appropriate. Compare to Herald of the Last Light (uncommon, "Fast -- Abandon this: Prevent a played event"). |
| Resonance Fit | OK | Prevent is normally Tide's territory, not Ruin's. The designer justifies it through body-sacrifice (Ember-adjacent) on a Ruin body. This is a deliberate cross-resonance statement. The self-mill rider on Prevent grounds it in Ruin. This works but is intentionally transgressive -- a Prevent in Ruin is surprising. If that surprise is intended, it succeeds. |
| Parasitic | No | Works with any board state where you want to counter a character. |
| Keyword Density | 3 (Abandon, Prevent, self-mill) | Acceptable for uncommon. |

**Verdict: Pass. The Prevent-in-Ruin is unconventional but justified by the self-sacrifice cost.**

---

#### 18. Resonance Siphon (Neutral, Uncommon, 2 cost Fast Event)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | OK | "Resonance" is a game term in Dreamtides (the color system). Using it in a card name is like naming an MTG card "Mana Siphon" -- it breaks the fourth wall slightly. The existing pool does not use "Resonance" in any card name. "Siphon" is fine. Consider "Aetheric Siphon" or "Dreamflow Siphon" to avoid the game-term issue. |
| Complexity | Appropriate | One static effect until end of turn + cantrip. Simple for uncommon. |
| Resonance Fit | N/A (Neutral) | Neutral is correct for a universal activated-ability enabler. |
| Parasitic | No | Spiritbound Alpha, Mystic Runefish, Assault Leader, Minstrel of Falling Light -- many existing expensive activated abilities. |
| Keyword Density | 1 (cost reduction) | Minimal. |

**FLAG: Fourth-wall name.** "Resonance" is the game's color system terminology. Using it in a card name risks feeling meta rather than immersive. Minor issue.

**Verdict: Pass with naming note.**

---

#### 19. Kindlespark Harvester (Stone, Rare, 3 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Kindlespark" compounds two game mechanics (kindle + spark) into a flavor word that reads as "harvesting kindled sparks" -- evocative and mechanically suggestive without being literal. "Harvester" fits the role. Compare to "Harvester of Despair" in the existing pool. Two Harvesters is fine. |
| Complexity | Appropriate | One Judgment activated ability: remove spark to dissolve + kindle. Clean cost-benefit decision. Rare-appropriate. |
| Resonance Fit | Strong | Judgment trigger is Stone. Kindle is Stone's incremental growth. The spark-removal cost creates a Stone-appropriate tension between scoring and board control. |
| Parasitic | No | Any deck that accumulates kindle/spark on the leftmost ally can use this. |
| Keyword Density | 3 (Judgment, kindle, dissolve with spark cost) | Acceptable for rare. |

**Verdict: Pass. Novel "spark as currency" design is excellent.**

---

#### 20. Echoing Departure (Zephyr, Common, 2 cost Event)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Echoing" suggests repetition/reverberation (fitting for a leaves-play trigger). "Departure" directly evokes leaving play. Compare to "Passage Through Oblivion" or "Starlit Cascade" -- similar atmospheric event naming. |
| Complexity | High | "Until end of turn, when an ally leaves play, the next character you play costs 1 less and you may Foresee 1." This is a lot for a common. It sets up a until-end-of-turn triggered ability with two effects (cost reduction + Foresee). Commons should be simpler -- compare to Harvest the Forgotten (common, "Mill 3. Draw 1") or Starlit Cascade (common, "Until end of turn, when ally leaves play, gain 2 energy"). Starlit Cascade is the closest parallel and has one effect per trigger, not two. |
| Resonance Fit | Strong | Leaves-play trigger is Zephyr motion. Cost reduction enables more deployment (Zephyr tempo). Foresee is Zephyr/Tide card selection. |
| Parasitic | No | Starlit Cascade exists as the only other "ally leaves play" trigger, so this expands a thin axis using existing infrastructure. |
| Keyword Density | 3 (leaves-play trigger, cost reduction, Foresee) | **High for common.** |

**FLAG: Complexity for common.** Two effects per trigger (cost reduction AND Foresee) is one effect too many for common. Recommend removing the Foresee to bring it in line with Starlit Cascade's simplicity. "Until end of turn, when an ally leaves play, the next character you play costs 1 less" is a clean common.

**Verdict: Flagged. Too complex for common rarity.**

---

#### 21. Risen Warden (Ruin, Uncommon, 4 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Risen" evokes resurrection/void-return. "Warden" is the third Warden (see naming collision concerns). The name works on its own -- a warden who has risen from death. |
| Complexity | Appropriate | Void-only play restriction + Dissolved trigger. Two clean mechanics. Compare to Revenant of the Lost (uncommon, void-only). |
| Resonance Fit | Strong | Void-only play and Dissolved-trigger Reclaim are pure Ruin mechanics. The Warrior typing in Ruin creates the intended cross-resonance signal. |
| Parasitic | No | Existing mill and Reclaim effects (Harvest the Forgotten, Path to Redemption) get it into the void. |
| Keyword Density | 2 (void-only, Dissolved Reclaim) | Clean. |

**Verdict: Pass. Good cross-resonance Warrior.**

---

### D. Modular Engine Cards (5 cards)

#### 22. Dreamtide Cartographer (Neutral, Uncommon, 3 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Dreamtide" is a compound that evokes Dreamtides' world. "Cartographer" is a distinctive role noun not used elsewhere -- it suggests mapping/exploring the dreamscape. Compare to "Dimensional Pathfinder" or "Starsea Traveler" -- similar exploration theme. |
| Complexity | Appropriate | Threshold-gated binary (3 or fewer cards: draw 2; otherwise: +1 spark to allies). Two clean modes. Uncommon-appropriate. |
| Resonance Fit | N/A (Neutral) | The hand-size threshold and dual modes feel appropriately neutral -- any archetype can use either mode. |
| Parasitic | No | Universal applicability. |
| Keyword Density | 2 (hand-size threshold, draw/spark) | Clean. |

**Verdict: Pass. Excellent engine design with wide applicability.**

---

#### 23. Nexus of Passing (Neutral, Rare, 4 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Nexus" suggests a convergence point (fitting for a zone-change counter). "of Passing" evokes transition/movement. Compare to "Nexus Wayfinder" in the existing pool -- the "Nexus" prefix is established but creates a pair. Two Nexuses is fine. |
| Complexity | High | "This character's spark is equal to the number of cards that have changed zones this turn." The concept is simple but the TRACKING is complex. Players must count every draw, every discard, every mill, every sacrifice, every event resolution, every bounce, every flicker -- all zone changes in a single turn. This is a bookkeeping nightmare in practice. Compare to Abomination of Memory (spark = cards in void) -- that is a single count of a stable zone. Nexus of Passing requires tracking a running total of transient events during a turn. |
| Resonance Fit | N/A (Neutral) | Zone-change universality is intentionally neutral. |
| Parasitic | No | Every archetype generates zone changes naturally. |
| Keyword Density | 1 (variable spark) | Text is simple but execution is complex. |

**FLAG: Tracking burden.** Counting "cards that have changed zones this turn" is extremely difficult to track in practice. Does drawing a card count as a zone change (deck to hand)? Does playing an event count as two changes (hand to stack, stack to void)? Does flickering count as two (battlefield to banish, banish to battlefield)? The design document provides examples, but players at the table will constantly dispute the count. This is a rules-nightmare card. Compare to Abomination of Memory, which counts a static zone -- much easier.

Recommendation: Simplify to a countable subset of zone changes, e.g., "This character's spark is equal to the number of times a card entered or left the battlefield this turn." Battlefield-specific zone changes are easier to track visually.

**Verdict: Flagged. Excellent concept, problematic execution due to tracking complexity.**

---

#### 24. Crucible of the Commons (Stone, Uncommon, 3 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | OK | "Crucible" directly names the archetype. "of the Commons" is an unusual qualifier -- it could read as "a crucible that belongs to common people" or as a gameplay reference to low-spark commons. The name is more abstract than most Dreamtides names and risks sounding like a gameplay label. Compare to "Spiritbound Alpha" or "Skyflame Commander" -- those are characters, while "Crucible of the Commons" sounds like a place or artifact. Given the card is a Visitor character, the name is incongruent with the type. |
| Complexity | Appropriate | One Judgment trigger with a spark-threshold condition. Clean for uncommon. |
| Resonance Fit | Strong | Judgment-phase incremental board buff is pure Stone. |
| Parasitic | No | Every archetype has low-spark utility creatures. |
| Keyword Density | 1 (Judgment spark buff) | Clean. |

**FLAG: Name incongruence.** The card is a Visitor character, but "Crucible of the Commons" sounds like a location or artifact, not a person. This is unlike any existing Visitor name (Moonlit Dancer, Keeper of the Lightpath, The Bondweaver). Recommend a character-appropriate name like "Champion of the Commons" or "Hearthkeeper of the Low."

**Verdict: Pass mechanically. Name needs work.**

---

#### 25. Archivist of Vanished Names (Tide, Rare, 3 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Archivist of Vanished Names" follows the "[Role] of [Poetic Abstraction]" pattern perfectly. Evokes knowledge, loss, and memory -- Tide's themes. Compare to "Keeper of Forgotten Light" or "Seeker of the Radiant Wilds." One of the best names in the batch. |
| Complexity | High | "Name a card type, reveal until you hit it, put it in hand, rest to void." The "reveal until" mechanic is unbounded and stochastic. While the average case is 1-3 cards, the variance can be extreme. This is appropriate for rare but requires careful rules implementation. |
| Resonance Fit | Strong | Draw/filter/selection is Tide. The incidental self-mill bridges to Ruin territory, which is appropriate for a Tide card that Undertow poaches. |
| Parasitic | No | Universal utility -- any deck has characters and events. |
| Keyword Density | 2 (named-type choice, reveal-until mill) | Reasonable for rare. |

**Verdict: Pass. Beautiful name, strong design.**

---

#### 26. Ember of Recurrence (Ruin, Rare, 3 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Ember of Recurrence" is poetic and evocative -- a dying fire that keeps returning. The "Ember" prefix might confuse (is this an Ember-resonance card?), but it is Ruin-resonance. In context, "ember" is a common English word, not necessarily the resonance name. Compare to "Echoes of Eternity" -- similar abstract-poetic naming. |
| Complexity | High | "When a card enters your void from any zone, you may pay 1 energy: Return a different card from your void to your hand." This triggers on EVERY void entry, which can be extremely frequent (mill 4 = 4 triggers). The once-per-trigger payment and "different card" clause add decision complexity. Rare-appropriate but demands high game-literacy. |
| Resonance Fit | Strong | Void-entry trigger and void recursion are pure Ruin. |
| Parasitic | No | Every archetype sends cards to the void. |
| Keyword Density | 1 (void-entry triggered recursion) | Deceptively simple text, complex execution. |

**FLAG: Potential confusion with resonance name.** "Ember of Recurrence" is Ruin-resonance but uses "Ember" in its name. A player might assume it is Ember-resonance. This is a minor flavor concern -- "ember" as an English word works, but the game has a resonance called Ember. Consider "Cinder of Recurrence" or "Spark of Recurrence" to avoid ambiguity. Wait -- "Cinder" is also an archetype name. "Glimmer of Recurrence" or "Echo of Recurrence" would be safer.

**Verdict: Pass with minor naming concern about resonance-word confusion.**

---

### E. Gap Filler Cards (5 cards)

#### 27. Ironbark Sentinel (Stone, Rare, 4 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Ironbark" is a strong nature compound (iron + bark = Stone's endurance). Fourth Sentinel in the batch (see naming collision). |
| Complexity | Appropriate | One Judgment trigger with a "since your last turn" persistence check. Clean for rare. |
| Resonance Fit | Strong | Continuous-presence reward on Judgment is quintessential Stone. Anti-synergy with Zephyr flicker is a bonus that reinforces resonance boundaries. |
| Parasitic | No | Works with any board that can keep allies alive. |
| Keyword Density | 1 (Judgment persistence buff) | Clean. |

**Verdict: Pass. Strong design, name collision with Sentinels.**

---

#### 28. Tidechannel Observer (Ruin, Uncommon, 3 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Tidechannel" is an evocative compound (a channel through which tides flow). "Observer" is a clean role noun. Compare to "Pattern Seeker" or "Oracle of Shifting Skies" -- similar knowledge-watching theme. |
| Complexity | Appropriate | One Judgment trigger with a void-velocity threshold (3+ cards entered void). Clean for uncommon. |
| Resonance Fit | Strong | Void-velocity counting is Ruin. Points and kindle reward are appropriate Ruin payoffs. |
| Parasitic | No | Existing mill, sacrifice, and discard effects all feed the threshold. |
| Keyword Density | 2 (Judgment threshold, kindle+points) | Clean. |

**Verdict: Pass. Good differentiation tool for Undertow.**

---

#### 29. Fading Resonant (Zephyr, Uncommon, 2 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | OK | "Fading" evokes Zephyr's transience. "Resonant" uses the game's resonance terminology as a noun (a being that resonates). This is slightly awkward -- "Resonant" as a noun is uncommon in English. Compare to "Flickerveil Adept" or "Blooming Path Wanderer." "Fading Resonant" also risks the resonance-terminology confusion. Consider "Fading Whisper" or "Fading Presence." |
| Complexity | Appropriate | One triggered ability: "when ally leaves play, next character costs 1 less." Clean tempo effect at uncommon. |
| Resonance Fit | Strong | Leaves-play trigger is Zephyr motion. Cost reduction enables more deployment (Zephyr tempo). |
| Parasitic | No | Starlit Cascade is the only other leaves-play trigger, so this expands a thin axis using existing flicker/bounce infrastructure. |
| Keyword Density | 1 (leaves-play cost reduction) | Clean. |

**Verdict: Pass with minor naming note. Good design for an underserved mechanic.**

---

#### 30. Stormtrace Augur (Tide, Rare, 3 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Stormtrace" is evocative -- tracing/tracking storms. "Augur" fits the Mage subtype and Tide's knowledge theme. Compare to "Oracle of Shifting Skies." |
| Complexity | Appropriate | Variable spark (events in void). This is identical in mechanical template to Abomination of Memory (spark = cards in void) and Spirit of Smoldering Echoes (spark grows with events entering void). Established pattern at rare. |
| Resonance Fit | Strong | Event-counting is Tide. Void-counting bridges to Ruin. The combination rewards Tide's natural event play while creating a proactive win condition. |
| Parasitic | No | Any deck that plays events fills its void with events naturally. |
| Keyword Density | 1 (variable spark) | Clean. |

**FLAG: Redundancy with Stormtide Oracle.** Both Stormtrace Augur (Tide, rare, "spark = events in void") and Stormtide Oracle (Tide+Ember, rare, "spark = events in void" + Judgment retrieval) from the signpost duals have nearly identical core mechanics. Having two cards with "spark = events in void" in a 31-card batch is redundant. One should be redesigned or cut. The Stormtide Oracle is the stronger design because the retrieval creates a decision (shrink spark vs. recover event), while Stormtrace Augur is a simpler version. Recommend cutting Stormtrace Augur or differentiating it significantly (e.g., count ALL cards in void, not just events, to make it more like Abomination of Memory with a different cost point).

**Verdict: Flagged for mechanical redundancy with Stormtide Oracle (Card 6).**

---

#### 31. Duskwatch Warden (Tide, Uncommon, 2 cost)

| Criterion | Rating | Notes |
|-----------|--------|-------|
| Name Quality | Good | "Duskwatch" is atmospheric -- watching at dusk, the transition time. Fits the Outsider subtype's liminal feel. Third Warden in the batch (see naming collision). |
| Complexity | Appropriate | One triggered ability: "when you prevent, next event costs 2 less." Clean for uncommon. |
| Resonance Fit | Strong | Prevent-trigger payoff is Tide. Cost reduction bridges to Stone's economic identity but stays within Tide's reactive-to-proactive conversion theme. |
| Parasitic | No | 9 existing Prevent effects in the pool. |
| Keyword Density | 2 (Prevent trigger, cost reduction) | Clean. |

**Verdict: Pass. Good Depths/Gale bridge design.**

---

## Flagged Cards Summary

### Cards Requiring Attention

| # | Card | Issue | Severity | Recommendation |
|---|------|-------|----------|----------------|
| 10 | Bedrock Anchor | Overloaded for uncommon (4 mechanics) | High | Remove one mechanic. Either cut the Materialized energy+draw OR the Judgment kindle. |
| 20 | Echoing Departure | Too complex for common (3 effects per trigger) | Medium | Remove the Foresee rider. Keep just the cost reduction on ally-leaves-play. |
| 23 | Nexus of Passing | Tracking burden (counting ALL zone changes per turn) | High | Restrict to battlefield zone changes only, or pick a specific countable subset. |
| 30 | Stormtrace Augur | Mechanically redundant with Stormtide Oracle (Card 6) | High | Cut or significantly redesign. Two "spark = events in void" cards in one batch is wasteful. |
| 15 | Deepvault Warden | Resonance fit tension (Stone card that rewards void play) | Medium | Make it Stone+Ruin dual, or broaden the cost reduction beyond void-only plays. |
| 24 | Crucible of the Commons | Name incongruence (location/artifact name on a Visitor character) | Low | Rename to a character-appropriate name. |

### Naming Issues

| Issue | Cards Affected | Recommendation |
|-------|---------------|----------------|
| **Sentinel overuse** (4 cards) | Tideweaver Sentinel, Oathbound Sentinel, Voidthorn Sentinel, Ironbark Sentinel | Rename at least 2. Options: Guardian, Keeper, Watcher, Protector, Ward. |
| **Warden overuse** (3 cards) | Basalt Warden, Deepvault Warden, Duskwatch Warden, (+ Risen Warden) | Rename at least 1-2. "Duskwatch Vigil" or "Deepvault Keeper" work. |
| **Archetype-naming** (4 cards) | Basalt Warden, Galerunner, Cinder Ritualist, Crucible of the Commons | These directly name their archetypes. Minor issue if archetype names are also natural English words (basalt, gale, cinder, crucible all are), but the cumulative effect is noticeable. |
| **Resonance-word confusion** | Ember of Recurrence (Ruin), Fading Resonant (Zephyr), Resonance Siphon (Neutral) | Using resonance terminology (Ember, Resonance) in card names risks confusion about the card's actual resonance. |
| **Ritualist collision** | Cinder Ritualist vs. Rebirth Ritualist (existing) | Two Ritualists is fine but worth noting. |

---

## Overall Set Coherence Assessment

### Do these 31 cards feel like they belong together?

**Mostly yes, with caveats.**

**Strengths:**
- The mechanical themes are well-unified. The batch collectively explores under-utilized axes in the existing pool: leaves-play triggers (V19), hand-size-matters (V01), void velocity (V29), continuous-presence rewards (V27), and Prevent-trigger payoffs (V02). These are genuine gaps in the 222-card pool that the new cards address.
- The subtype distribution is reasonable: Ancient (7), Warrior (3), Survivor (3), Visitor (4), Explorer (3), Outsider (3), Mage (3), Synth (1), Musician (0). This roughly mirrors the existing pool's proportions without over-saturating any type.
- The cost curve is appropriate: six 2-cost cards, fourteen 3-cost cards, seven 4-cost cards, zero 5+ cost cards. The batch fills the 2-4 cost range where engine pieces belong.
- The rarity distribution (6 common-appropriate designs at uncommon or lower, 12 uncommon, 13 rare) is reasonable for a supplemental batch.

**Weaknesses:**
- **Role-noun monotony.** Seven guard-type role nouns (4 Sentinels + 3 Wardens) out of 31 cards gives the batch a militaristic flavor that does not match the existing pool's diversity. The existing 222 cards use Wanderer, Dancer, Caller, Rider, Channeler, Puppeteer, Leviathan, Wolves, Eagle, Stag, Panther, Oracle, Guide, etc. The new batch is heavy on guards and light on the dreamlike, natural, or cosmic roles that define Dreamtides' tone.
- **Archetype-naming in card names.** Four cards use archetype names (Basalt, Gale, Cinder, Crucible) as card-name prefixes. The existing 222 cards never do this. While these words have natural-language meanings, the pattern is noticeable in aggregate and makes the batch feel more like a game-design exercise than organic worldbuilding.
- **The Stormtrace Augur / Stormtide Oracle redundancy** undermines the sense that each card fills a unique role. Two nearly-identical "spark = events in void" cards in one batch feels like a coordination failure between agents.
- **No events in the signpost duals.** All 10 signpost duals are characters. The existing dual-resonance cards in the pool include events. A batch of 10 character-only signposts feels slightly homogeneous in card type, though this is a minor concern since signpost cards are typically characters.

### Do these 31 cards feel like they belong with the existing 222?

**Largely yes.**

- The mechanical complexity levels are generally appropriate -- the new cards use the same keywords (kindle, Foresee, Prevent, Materialized, Judgment, dissolve, Reclaim, banish) as the existing pool. No new keywords are introduced.
- The spark values and cost curves are in line with existing cards at similar rarities.
- The new mechanical axes (continuous-presence, void-velocity, hand-size-matters, zone-change counting) are genuine innovations that do not contradict any existing mechanic. They extend the game's design space rather than replacing it.
- The concern about Nexus of Passing's tracking burden is the most significant "does not fit" issue -- the existing pool avoids mechanics that require counting transient events across a turn. Abomination of Memory counts a stable zone (void size). Blade of Unity counts a static board state (allied Warriors). Nexus of Passing counts a dynamic, turn-long running total that resets each turn. This is a mechanical outlier that would be difficult to implement in both the Rust rules engine and the Unity client.

### Rarity Appropriateness Summary

| Rarity | Cards | Complexity Assessment |
|--------|-------|----------------------|
| Common | Oathbound Sentinel, Echoing Departure | Oathbound Sentinel is appropriate. Echoing Departure is too complex. |
| Uncommon | Tideweaver Sentinel, Abyssal Reclaimer, Basalt Warden, Galerunner, Bedrock Anchor, Ironveil Watcher, Stoneheart Veteran, Deepvault Warden, Ashen Threshold, Voidthorn Sentinel, Resonance Siphon, Echoing Departure (if downgraded), Risen Warden, Dreamtide Cartographer, Crucible of the Commons, Tidechannel Observer, Fading Resonant, Duskwatch Warden | Bedrock Anchor is overloaded. Galerunner is at the upper bound. Others are appropriate. |
| Rare | Forgeborn Martyr, Cinder Ritualist, Stormtide Oracle, Depthswatcher, Eclipse Weaver, Vanguard of the Summit, Kindlespark Harvester, Nexus of Passing, Archivist of Vanished Names, Ember of Recurrence, Ironbark Sentinel, Stormtrace Augur | Nexus of Passing has tracking concerns. Stormtrace Augur is redundant. Others are appropriate. |

---

## Recommendations Summary

### Must-Fix (High Severity)
1. **Bedrock Anchor:** Simplify to 2-3 mechanics for uncommon. Remove either the Materialized energy+draw or the Judgment kindle.
2. **Nexus of Passing:** Restrict zone-change counting to a trackable subset (battlefield entries/exits only) or redesign the scaling metric entirely.
3. **Stormtrace Augur:** Cut from the batch (redundant with Stormtide Oracle) and replace with a mechanically distinct Tempest/Depths tool, or differentiate it by counting a different metric.

### Should-Fix (Medium Severity)
4. **Echoing Departure:** Remove the Foresee rider for common-appropriate simplicity.
5. **Deepvault Warden:** Either make it Stone+Ruin dual (acknowledging its void-play identity) or broaden the cost reduction to cover non-void sources too.
6. **Naming: Sentinel/Warden overuse.** Rename at least 3 of the 7 guard-role-noun cards to use more diverse, Dreamtides-appropriate role names.

### Nice-to-Fix (Low Severity)
7. **Crucible of the Commons:** Rename to match a character, not a place/artifact.
8. **Archetype names in card names:** Consider renaming Basalt Warden, Galerunner, and Cinder Ritualist to use non-archetype words.
9. **Resonance Siphon:** Rename to avoid the game-term "Resonance" in a card name.
10. **Ember of Recurrence:** Consider renaming to avoid "Ember" on a Ruin card.
