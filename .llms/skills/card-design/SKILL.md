---
name: card-design
description: Design a new Dreamtides card based on art input. Use when designing cards, creating card concepts, or translating art into game mechanics. Triggers on card design, design card, new card, card art, card concept.
---

# Card Design Skill

You are an expert card game designer, creating novel game designs with deep thinking and analysis. Run everything with ultrathink.

Read @docs/battle_rules/battle_rules.md and @docs/tides/tides.md.

Please feel free to run as many Opus subagents as you would like for research/analysis to support this task.

# Phase 1: Classify Art

The first step in card design is to classify the card art. There are 3 possible types of card art:

1) Character. Does the art show a single figure of a person, creature, animal, monster, etc? Then this is a character card.
2) Dreamwell. Does the art depict a landscape? A wide depiction of a wilderness or urban landscape is a dreamwell card.
3) Event. Art which does not depict a character or landscape is by default an event card.

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

The best card design in the world will fail if it doesn't thematically connect to one of the archetypes in the game.
Each card needs to be assigned to one of the 7 tides (or neutral) and should support the primary game plan of that
tide. Great card designs are often ones that support game plans from multiple tides. Think about how your design
can connect to a tide.

Note: supporting 2+ tides does NOT mean stapling two unrelated mechanics to a card. The best cards are a single
cohesive structure. This is much more important than multi-archetype support.

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

### Cross-Tide Synergy (Adjacent Allies)

When a design naturally bridges two allied tides, it becomes more draftable:

- **Bloom + Arc:** Blink value (re-trigger materialized abilities on spirit animals for repeated ETB value)
- **Arc + Ignite:** Token tempo (blink token-producers)
- **Ignite + Pact:** Aristocrats (generate expendable bodies, sacrifice for value)
- **Pact + Umbra:** Void engine (fill void from two angles)
- **Umbra + Rime:** Grindy midrange (discard and mill to void, recur)
- **Rime + Surge:** Full control (proactive disruption + reactive counterspells)
- **Surge + Bloom:** Ramp combo (accelerate energy, chain events)

### Tide Cost Guidelines

- **Tide-cost 1:** Lightly committed; splashable in hybrid decks
- **Tide-cost 2:** Requires real commitment to this tide
- **Tide-cost 3:** Build-around card, only for deep single-tide decks
- **Neutral tide-cost 1:** Playable in any deck

# Phase 4: Connect to Existing Designs

There are over 400 existing card designs in rules_engine/tabula/rendered-cards.toml. The
file is 8000 lines total.

Read this file to understand how we write abilities in Dreamtides and follow existing card templating conventions.
Understand what designs already exist, try not to duplicate them. Remember Dreamtides is a digital card game,
meaning we have access to tools that are not available to paper card games.

### Research Strategy

The rendered-cards.toml file is large. Use an Explore subagent to read it comprehensively and extract:

1. All cards in the tide you're designing for — understand the existing design space
2. All cards with mechanics similar to what you're considering — avoid duplication
3. Cards with similar themes (knowledge, destruction, nature, etc.) — understand naming and flavor conventions
4. The exact templating syntax for abilities (triggers use `▸`, costs use `●` for energy and `✪` for spark, etc.)

### Templating Conventions

- Triggered abilities: `▸ Judgment:`, `▸ Materialized:`, `▸ Dissolved:`
- Combined triggers: `▸ Materialized, Judgment:`
- Costs in text: `●` for energy, `✪` for victory points
- Activated abilities: `N●: Effect` or `N●, cost: Effect`
- Reclaim keyword: `Reclaim N●` on its own line at the end
- Alternative costs: `Cost: Play for 0.` before the main effect
- Conditional: `If condition, effect.` or `When condition, effect.`
- Scaling: `per allied Warrior`, `equal to cards in your void`, `for each card played this turn`

# Phase 5: Design

Given information about the tides and existing cards, design a new card which matches the given art. Write your
response with the following:

- Card Name: Evocative short name for this card
- Card Type: Type of card & subtype if character
- Tide: Which tide this card is associated with & its tide cost
- Energy Cost: Proposed cost for this card
- Spark: Proposed spark value for a character card
- Rules Text: Proposed ability text for this card.
- Art Description: One sentence description of the card art.
- Archetype Description: One sentence description of how this card supports its Tide achieve its primary strategy
  for winning the game, or for neutral cards how it supports many strategies.
- Narrative: One sentence description of how the art connects to the game mechanics. For characters, explain
  who this dream character is and why they have this ability. For events, explain what is happening in the event
  and why it has this effect on the game. For dreamwell cards explain where this location is and why it has this
  effect.
- Similar Cards: Briefly mention cards with similar effects to this one which you analyzed.

### Design Principles

1. **Simplicity first.** The best cards have one clean mechanic, not three stapled together. If you can express the card in one sentence of rules text, do so.

2. **Match the art.** The narrative must be believable. A serene forest spirit should not dissolve enemies. A war machine should not draw cards peacefully. If the art and mechanic don't tell the same story, redesign.

3. **Avoid duplication.** If an existing card already does what you're designing, find a different angle. The game has 400+ cards — check thoroughly.

4. **Cost appropriately.** Compare to existing cards at similar power levels. Card draw is roughly 1 card per 1-2 energy. Dissolve effects cost 2-4 energy depending on restrictions. Prevent effects cost 1-3 energy.

5. **Consider variance.** Digital card games can use mechanics with interesting variance (e.g., "put any events into your hand" from a set of revealed cards). Embrace the digital medium.

6. **Tide commitment should match power.** Tide-cost 1 cards are good role-players. Tide-cost 2 cards should meaningfully reward being in that tide. Tide-cost 3 cards should be build-arounds.

7. **Think about the draft.** Cards that only work in one specific deck are less interesting than cards that are good in their primary tide but also playable in an adjacent hybrid strategy.
