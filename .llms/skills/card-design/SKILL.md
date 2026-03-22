---
name: card-design
description: Design a new Dreamtides card based on art input. Use when designing cards, creating card concepts, or translating art into game mechanics. Triggers on card design, design card, new card, card art, card concept.
---

# Card Design Skill

You are an expert card game designer, creating novel game designs with deep thinking and analysis. Run everything with ultrathink.

Read @docs/battle_rules/battle_rules.md and @docs/tides/tides.md.

## Research Tool

A research script is available at `.claude/skills/card-design/card-research.py`. Use it
directly via bash instead of spawning subagents to read rendered-cards.toml. Run these
commands as needed throughout the design process:

```bash
# Overview of the full card pool
python3 .claude/skills/card-design/card-research.py stats

# All cards in a tide (sorted by cost)
python3 .claude/skills/card-design/card-research.py tide Rime

# Find cards by mechanic keyword(s) — searches name + rules text
python3 .claude/skills/card-design/card-research.py mechanic prevent
python3 .claude/skills/card-design/card-research.py mechanic discard kindle

# Find cards by subtype
python3 .claude/skills/card-design/card-research.py subtype Survivor

# Find cards by name fragment
python3 .claude/skills/card-design/card-research.py name "Storm"

# Find cards at a specific energy cost
python3 .claude/skills/card-design/card-research.py cost 3

# Search rules text for a phrase
python3 .claude/skills/card-design/card-research.py similar "when you discard"
```

# Phase 1: Classify Art

The first step in card design is to classify the card art. There are 3 possible types of card art:

1) **Character.** Does the art show a single figure of a person, creature, animal, monster, etc? Then this is a character card.
2) **Dreamwell.** Does the art depict a landscape? A wide depiction of a wilderness or urban landscape is a dreamwell card.
3) **Event.** Art which does not depict a character or landscape is by default an event card.

### Classification Tips

- Interior scenes (rooms, corridors, vehicles) without a central figure are events, not landscapes.
- Group scenes (armies, crowds, flocks) are always events, never characters. A character card must depict a single figure.
- Landscapes need a sense of scale and openness. A close-up of a single tree or rock formation is not a landscape.

# Phase 2: Identify Art Constraints

Analyze the art and what it depicts, and think about how this would translate into game terms. Many excellent card designs
fail because they don't match the art. For example:

- Art showing a larger-than-human character must correspond to an expensive/high-spark character card
- Art with a positive/uplifting mood should have a positive-coded game effect for the player such as drawing cards
- Art with a horror/destructive mood should have a negative-coded effect such as interacting with the opponent or the void

Connecting the mood of the art to the card design is very important. We are trying to build a coherent narrative
that explains e.g. *what is happening* in this event or *who this character is*.

### Art-to-Mechanic Translation Guide

**Visual elements and their mechanical associations:**

| Visual Element | Mechanical Association |
|---|---|
| Golden light, sparks, radiance | Card draw, Foresee, Discover — revelation/knowledge |
| Darkness, shadow, void imagery | Void interaction, mill, Reclaim — hidden resources |
| Fire, explosion, destruction | Dissolve, Banish, removal — destroying threats |
| Growth, nature, blooming | Ramp, energy gain, max energy increase |
| Speed, motion blur, flight | Fast keyword, tempo, return-to-hand effects |
| Crowds, armies, banners | Token generation, go-wide, tribal effects |
| Sacrifice, altars, blood | Abandon effects, sacrifice-for-value |
| Ice, winter, austerity | Discard effects, taxation, cost increase |
| Machinery, technology, circuits | Repeatable effects, engines, systematic processes |
| Magical cascades, chains, links | Event chaining, copying, storm effects |

**Size-to-cost mapping for characters:**

| Character Scale | Typical Cost | Typical Spark |
|---|---|---|
| Small creature (cat, imp, child) | 1-2 | 0-1 |
| Human-sized figure | 2-4 | 1-3 |
| Large creature/imposing figure | 4-6 | 3-5 |
| Titanic/mythic being | 7-9 | 5-7 |

**Mood-to-effect mapping:**

| Mood | Player Effect | Opponent Effect |
|---|---|---|
| Serene, contemplative | Foresee, card selection | — |
| Joyful, triumphant | Draw cards, gain energy | — |
| Mysterious, hidden | Discover, look at top of deck | — |
| Aggressive, intense | — | Dissolve, forced sacrifice |
| Eerie, haunting | Void recursion, Reclaim | Discard from hand |
| Chaotic, explosive | — | Mass removal, sweepers |

# Phase 3: Connect to Tides

Each card needs to be assigned to one of the 7 tides (or neutral) and should support the primary game plan of that
tide. **The primary goal is making a great card for its tide.** If a design also happens to
work well in an adjacent tide's hybrid deck, that's a nice bonus — but never compromise a
card's core identity to chase cross-tide appeal. A card that's excellent in one tide is
better than a card that's mediocre in two.

### Tide Quick Reference

| Tide | Primary Strategy | Key Mechanics | Wants to... |
|---|---|---|---|
| Bloom | Ramp | Energy gain, Spirit Animals, Voltron | Deploy one massive threat |
| Arc | Blink | Materialize triggers, temporary banish, fast | Re-trigger enter-play effects |
| Ignite | Go-wide | Warriors, Figment tokens, mass pump | Flood the board with bodies |
| Pact | Sacrifice | Abandon-for-value, reanimation | Sacrifice cheap things for expensive things |
| Umbra | Self-mill | Void count matters, Reclaim, Survivors | Stock the void, then exploit it |
| Rime | Self-discard | Discard-for-value, hand disruption, taxation | Turn excess cards into resources |
| Surge | Storm | Events matter, Prevent, event copying, energy burst | Chain events in one explosive turn |
| Neutral | Utility | Removal, Discover, Foresee, sweepers | Flexible answers for any deck |

### Cross-Tide Synergy (Secondary Consideration)

The tide circle is: Bloom — Arc — Ignite — Pact — Umbra — Rime — Surge — (back to Bloom).
Adjacent tides have natural overlaps. If your design happens to bridge two neighbors, that's
a bonus for draftability — but don't force it. Some tide pairs (notably Bloom + Arc) have
limited mechanical overlap, and trying to serve both can produce worse designs than focusing
on one.

- **Bloom + Arc:** Limited overlap — Bloom's ramp and Arc's blink don't naturally combine
- **Arc + Ignite:** Token tempo (blink token-producers)
- **Ignite + Pact:** Aristocrats (generate expendable bodies, sacrifice for value)
- **Pact + Umbra:** Void engine (fill void from two angles)
- **Umbra + Rime:** Grindy midrange (discard and mill to void, recur)
- **Rime + Surge:** Full control (proactive disruption + reactive counterspells)
- **Surge + Bloom:** Ramp combo (accelerate energy, chain events)

### Tide Cost

Tide cost is a deckbuilding constraint — higher tide cost requires deeper commitment to that tide.

- **Tide-cost 1:** Lightly committed; splashable in hybrid decks
- **Tide-cost 2:** Requires real commitment to this tide
- **Tide-cost 3:** Build-around card, only for deep single-tide decks
- **Neutral tide-cost 1:** Playable in any deck

### Character Subtypes

Characters can have a subtype that enables tribal synergies. Choose a subtype based on tide
fit and art flavor. Not every character needs a subtype — leave it blank if none fits.

| Subtype | Primary Tides | Thematic Flavor |
|---|---|---|
| Warrior | **Ignite** (22), Umbra (4), Pact (4) | Soldiers, fighters, martial figures |
| Survivor | **Umbra** (20), Pact (8), Rime (6) | Enduring figures, post-apocalyptic, haunted |
| Ancient | **Pact** (11), Umbra (6), Arc (5) | Mythic beings, eldritch entities, old powers |
| Spirit Animal | **Bloom** (22), Arc (5) | Animals, nature spirits, magical creatures |
| Mage | Spread across all tides (5 Rime, 5 Arc) | Spellcasters, sorcerers, mystics |
| Tinkerer | Spread across all tides (5 Ignite, 4 Surge) | Inventors, engineers, craftspeople |
| Explorer | Spread across all tides (5 Arc, 4 Rime) | Travelers, seekers, wanderers |
| Visionary | Spread across all tides (5 Neutral, 5 Umbra) | Prophets, seers, dreamers |
| Synth | Arc (5), Ignite (5), Pact (3) | Artificial beings, constructs, cyborgs |
| Outsider | **Pact** (8), Umbra (4), Rime (3) | Aliens, outcasts, otherworldly beings |
| Musician | **Arc** (5) | Performers — Arc's tribal subtype for fast-matters |
| Visitor | Arc (4), Neutral (3), Surge (3) | Otherworldly guests, travelers from beyond |
| Renegade | Rime (4), Pact (3) | Rebels, outlaws, defiant figures |
| Guide | Bloom (2), spread | Mentors, leaders, pathfinders |
| Monster | Rime (3) | Beasts, horrors, aberrations |
| Robot | Spread (1 each) | Mechanical beings |
| Child | Rime (1), Umbra (1) | Young figures |

### Figment Types

Figments are token characters created by card effects. Four types exist:

| Figment | Spark | Primary Tides |
|---|---|---|
| Celestial | 1 | Arc, Bloom, Surge |
| Radiant | 1 | Ignite, Surge |
| Halcyon | 1 | Ignite |
| Shadow | 0 | Pact (primary), Rime, Umbra |

### Fast Speed Guidelines

The `fast` keyword allows cards/abilities to be used outside the main phase. Distribution
by tide:

| Tier | Tides | Fast Card Count |
|---|---|---|
| Primary | **Arc** (23), **Neutral** (17), **Surge** (12) | High density |
| Secondary | Rime (6), Pact (5), Umbra (5), Bloom (4) | Selective use |
| Minimal | Ignite (1) | Almost never |

- **Events:** Fast events are primarily removal (Dissolve, Banish), prevention (Prevent),
  and card draw. Non-fast events are proactive plays (token generation, mass pump, draw
  engines).
- **Activated abilities:** Fast activated abilities are used for reactive plays — preventing
  dissolution, abandoning in response, bouncing. Written as `↯fast -- cost: effect`.
- **Characters:** Fast characters (via `is-fast`) can be played reactively. Primarily in Arc.
- **General rule:** If the effect is reactive (responds to opponent's actions), it should
  probably be fast. If it's proactive (builds your board), it should not be.

# Phase 4: Research Existing Cards

Use the research script to explore the existing card pool. Run at minimum:

1. `stats` — understand the overall card pool shape
2. `tide <your_tide>` — see all existing cards in your chosen tide
3. `mechanic <keyword>` — search for cards with similar mechanics to your concept
4. `similar <phrase>` — find cards with similar rules text

The goal is to (a) avoid duplicating existing designs, (b) understand the templating syntax,
and (c) calibrate your cost/spark values against comparable cards.

### Spark-per-Cost Benchmarks

Average spark values by energy cost across all characters:

| Cost | Avg Spark | Typical Range | Notes |
|---|---|---|---|
| 0 | 0.2 | 0-1 | Mostly utility (0 spark + ability) |
| 1 | 0.9 | 0-1 | Role-players, engine pieces |
| 2 | 1.2 | 0-2 | Workhorse slot; 1 spark + good ability is standard |
| 3 | 1.4 | 1-2 | 2 spark + ability is the sweet spot |
| 4 | 2.0 | 1-3 | 2-3 spark, meaningful abilities |
| 5 | 2.1 | 1-3 | Diminishing returns — ability must justify the cost |
| 6+ | 2.6-5.0 | 2-8 | High-end threats, often with alt costs or cheat-into-play |

Cards with powerful abilities trade spark downward (e.g., cost 3, spark 0-1 with a strong
engine effect). Cards with weak or no abilities get above-curve spark.

### Templating Conventions

Abilities in rendered-cards.toml use these conventions:

**Triggered abilities:**
```
▸ Materialized: Draw a card.
▸ Judgment: Gain 1●.
▸ Dissolved: Kindle 2.
▸ Materialized, Judgment: Gain 2●.
```

**Activated abilities:**
```
2●: This character gains +1 spark.
1●, Discard a card: Kindle 2.
Abandon an ally: Gain 1●.
```

**Fast activated abilities:**
```
↯fast -- Abandon this character: Prevent a played event.
↯fast -- Discard 2 cards: Return an enemy to hand.
```

**Static abilities (always-on):**
```
Allied Warriors have +1 spark.
The opponent's events cost 1● more.
Events cost you 1● less.
```

**Once per turn:**
```
Once per turn, when you discard a card, gain 1●, then kindle 2.
```

**Alternative costs (before main text):**
```
Banish a card from hand: Play this event for 0●.
Return an ally to hand: Play this event for 0●.
```

**Conditional cost reduction:**
```
This character costs 1● if you have discarded a card this turn.
Costs 0● if you discarded a card this turn.
```

**Reclaim (separate line at end):**
```
Reclaim 2●
Reclaim -- Abandon an ally
```

**Modal / Choose:**
```
Choose one: Draw 2 cards or Dissolve an enemy with spark 3 or less.
```

**Symbols:** `●` = energy, `✪` = victory points, `▸` = trigger, `↯` = fast

# Phase 5: Explore Concepts

Before committing to a final design, generate **2-3 one-line concept sketches**. Each
concept should be:

- A single sentence describing the mechanic
- The tide and approximate cost
- How it connects to the art

Present these concepts briefly, then pick the best one to develop fully. This avoids the
trap of over-investing in the first idea that comes to mind.

**Example concepts for art showing a hooded figure in rain catching glowing fragments:**
1. "Rime 3●/2✦ — discard 2 to prevent a played card (umbrella as shield, fragments as discarded cards)"
2. "Umbra 3●/1✦ — Judgment: mill 2, return a non-character from void (catching fragments from the void)"
3. "Rime 2●/1✦ — when you discard, foresee 1 (fragments reveal glimpses of the future)"

### Concept Evaluation Criteria

Pick the concept that best satisfies ALL of these:
- **Art match:** Does the mechanic tell the same story as the art?
- **Tide fit:** Does it advance the tide's primary strategy?
- **Novelty:** Is it meaningfully different from existing cards?
- **Simplicity:** Can you express it in one clean rules text block?
- **Draftability:** Would you pick this in a draft? Is it appealing on its own merits?

# Phase 6: Final Design

Develop your chosen concept into a complete card. Write your response with the following:

- **Card Name:** Evocative short name for this card
- **Card Type:** Character (with subtype) or Event
- **Tide:** Which tide and its tide cost (1-3)
- **Energy Cost:** Proposed cost
- **Spark:** Proposed spark value (characters only)
- **Rarity:** Common, Uncommon, Rare, or Legendary
- **Fast:** Yes/No (whether the card itself has `is-fast = true`)
- **Rules Text:** Proposed ability text, using the templating conventions above
- **Art Description:** One sentence description of the card art.
- **Archetype Description:** One sentence on how this card supports its tide's strategy
  (or for neutral, how it supports many strategies).
- **Narrative:** One sentence connecting the art to the mechanics. For characters, who is
  this person and why do they have this ability? For events, what is happening? For
  dreamwell, where is this place?
- **Similar Cards:** 2-4 existing cards with similar effects, with brief comparison.

### Dreamwell Design (if applicable)

Dreamwell cards have a different structure than regular cards:

- **Energy produced:** Phase 0 cards produce 2 energy. Phase 1 cards produce 1 energy.
- **Phase:** 0 (early game only, no bonus) or 1 (every cycle, has a bonus effect).
- **Bonus effects (phase 1 only):** Simple one-line effects. Existing effects are: Foresee 1,
  Gain 1 point, Gain 1 energy, Draw 1/Discard 1, Mill top 3 to void.
- **Naming:** Location names — evocative place names (Skypath, Autumn Glade, Twilight Radiance,
  Auroral Passage).
- Dreamwell cards have no tide, no cost, no spark, and no rarity.

### Design Principles

1. **Simplicity first.** The best cards have one clean mechanic, not three stapled together.
   If you can express the card in one sentence of rules text, do so. A card should have at
   most 2 mechanical elements (e.g., a trigger + an effect, or a cost reduction + an ability).

2. **Match the art.** The narrative must be believable. A serene forest spirit should not
   dissolve enemies. A war machine should not draw cards peacefully. If the art and mechanic
   don't tell the same story, redesign.

3. **Avoid duplication.** If an existing card already does what you're designing, find a
   different angle. Use the research script to check thoroughly.

4. **Cost appropriately.** Compare to existing cards at similar power levels using the
   spark-per-cost benchmarks. Card draw is roughly 1 card per 1-2 energy. Dissolve effects
   cost 2-4 energy depending on restrictions. Prevent effects cost 1-3 energy.

5. **Consider the digital medium.** Digital card games can use mechanics with interesting
   variance (e.g., "discover a character" from a random set, scaling effects based on game
   state). Embrace mechanics that would be impractical in paper.

6. **Tide commitment should match power.** Tide-cost 1 cards are good role-players.
   Tide-cost 2 cards should meaningfully reward being in that tide. Tide-cost 3 cards
   should be build-arounds.

7. **Think about the draft.** Cards that only work in one specific deck are less interesting
   than cards that are good in their primary tide but also playable in an adjacent hybrid
   strategy.

8. **Rarity guides complexity.** Common cards should have simple, clean effects. Uncommon
   cards can have one conditional or synergy-based ability. Rare cards can have more complex
   or build-around effects. Legendary cards are format-defining and uniquely powerful.

### Design Anti-Patterns to Avoid

- **Parasitic design:** Cards that do literally nothing without specific other cards. Every
  card should have a baseline of usefulness even without synergy.
- **Strict duplicates:** If an existing card does the same thing at the same cost, your
  design needs a different angle.
- **Stapled mechanics:** Two unrelated abilities on one card (e.g., "Draw 2 cards. Dissolve
  an enemy.") with no connecting theme. The abilities should form a cohesive whole.
- **Wrong-tide mechanics:** A Rime card that generates figment tokens, or a Surge card with
  abandon synergies. Mechanics should belong to the card's tide or an adjacent ally.
- **Overcomplexity:** If the rules text is more than 3 lines, simplify. The best designs are
  often the most elegant.

### Naming Guidelines

Names should be evocative and creative. Use the research script (`name` command) to check
the existing naming landscape and avoid collisions, but don't feel constrained to follow
rigid patterns. Some common structures that appear in the card pool:

- **[Adjective] [Noun]:** Silent Avenger, Eternal Sentry
- **[Compound] [Noun]:** Bloomweaver, Starcatcher
- **[Place] [Role]:** Neon Street Wanderer
- **[Noun] of [Noun]:** Titan of Forgotten Echoes, Blade of Oblivion
- **Single word:** Apocalypse, Reunion, Nocturne
- **The [Title]:** The Devourer, The Rising God

These are guidelines, not rules. Prioritize names that are memorable, evocative of the
card's identity, and feel natural. Creative names that don't fit any pattern are fine.

### Worked Example

**Art:** A massive glowing stag standing in an ancient forest, golden light radiating from its antlers.

**Phase 1 — Classification:** Single figure (animal) → Character.

**Phase 2 — Art constraints:** Large creature → cost 4-6, spark 3-5. Golden light/radiance → card draw or revelation. Nature/growth → Bloom. Serene, majestic mood → positive player effect.

**Phase 3 — Tide:** Bloom (nature, spirit animal, ramp). Focus on making a great Bloom card first.

**Phase 4 — Research:** Run `tide Bloom`, `subtype "Spirit Animal"`, `similar "draw a card"`. Found existing energy-producing spirit animals at various costs. No existing 5-cost spirit animal that conditionally draws cards.

**Phase 5 — Concepts:**
1. "Bloom 5●/3✦ Spirit Animal — Materialized, Judgment: Draw a card if you have 8+ energy (radiant knowledge rewards ramp)"
2. "Bloom 4●/2✦ Spirit Animal — Your Spirit Animals cost 1 less (ancient forest patron)"
3. "Bloom 5●/3✦ Spirit Animal — Materialized: Gain 2 max energy (permanent growth, not temporary)"

Concept 1 best matches the golden radiance (knowledge/revelation) and rewards Bloom's core ramp strategy — you only draw if you've been ramping, which is exactly what Bloom wants to do. The Judgment trigger gives it ongoing value. Pick concept 1.

**Phase 6 — Final Design:**
- **Card Name:** Luminheart Stag
- **Card Type:** Character — Spirit Animal
- **Tide:** Bloom (tide cost 2)
- **Energy Cost:** 5
- **Spark:** 3
- **Rarity:** Rare
- **Fast:** No
- **Rules Text:** `▸ Materialized, Judgment: With 8 or more maximum energy, draw a card.`
- **Art Description:** A massive glowing stag stands in an ancient forest, golden light radiating from its antlers.
- **Archetype Description:** Rewards Bloom's ramp investment with card advantage — once you've built your energy high enough, this stag keeps your hand full to deploy more threats.
- **Narrative:** The Luminheart Stag is an ancient guardian of the deep forest who reveals hidden truths to those who have proven their connection to the land — only dreamers who have cultivated enough energy can perceive the visions its antlers illuminate.
- **Similar Cards:** Looming Oracle (Bloom, 4●/2✦, Judgment: With 3 allied Spirit Animals, draw a card — tribal condition instead of ramp condition). Spiritbound Alpha (Bloom, 5●/3✦, Materialized: Gain 2 max energy — ramp payoff but energy instead of cards). This card is a dedicated ramp payoff that converts Bloom's energy investment into card flow.
