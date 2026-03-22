---
name: card-design
description: Design a new Dreamtides card based on art input. Use when designing cards, creating card concepts, or translating art into game mechanics. Triggers on card design, design card, new card, card art, card concept.
---

# Card Design Skill

You are an expert card game designer, creating novel game designs with deep thinking and analysis. Run everything with ultrathink.

Read `docs/battle_rules/battle_rules.md` and `docs/tides/tides.md` (use the Read tool).

## Research: Card Pool Reference

**Start by reading the entire card pool into context.** Run this command first — it outputs
every card in compact format (~500 lines), grouped by tide and sorted by cost:

```bash
python3 .claude/skills/card-design/card-research.py dump
```

Each line is: `Name | Tide+Cost | Stats | Type | Rarity | Rules Text`

Having the full card pool visible lets you spot gaps, patterns, and naming conventions
organically — rather than only finding what you already thought to search for.

### Supplemental Research Commands

For targeted checks after you've already developed an art-driven concept:

```bash
# Search rules text for a phrase (check for duplicates of your concept)
python3 .claude/skills/card-design/card-research.py similar "when you discard"

# Show cards at a specific cost within a specific tide (saturation check)
python3 .claude/skills/card-design/card-research.py cost-in-tide Umbra 3
```

These commands are for **validation**, not generation. Use them to confirm your concept
doesn't duplicate an existing card, not to find mechanical gaps to fill.

# Phase 1: Classify Art

The first step in card design is to classify the card art. There are 3 possible types of
card art:

1) **Character.** Does the art show a single figure of a person, creature, animal, monster,
   etc. as the primary subject? Then this is a character card.
2) **Dreamwell.** Does the art depict a landscape? A wide depiction of a wilderness or urban
   landscape is a dreamwell card.
3) **Event.** Art which does not depict a character or landscape is by default an event card.

### Classification Tips

- **Figure + dominant object/structure:** If a single figure is present but clearly
  secondary to a larger subject (a portal, monument, explosion, artifact, vehicle), classify
  based on the dominant element. A person dwarfed by a massive shattered monolith is an
  event (the shattering), not a character (the person). A person holding a glowing artifact
  at the center of the frame is a character (the person), not an event.
- **Interior scenes** (rooms, corridors, vehicles) without a central figure are events, not
  landscapes.
- **Group scenes** (armies, crowds, flocks) are always events, never characters. A character
  card must depict a single figure as the primary subject.
- **Landscapes** need a sense of scale and openness. A close-up of a single tree or rock
  formation is not a landscape.
- **Action scenes** showing an ongoing process (a ritual, a storm, a collapse) are events
  even if a single figure is participating, as long as the process is the visual focus.

# Phase 2: Read the Art's Story

Before thinking about mechanics at all, sit with the art and write a **narrative anchor**:
2-3 sentences describing what you see in pure story terms — no game vocabulary. What emotion
does it evoke? What's happening? What kind of person/creature/moment is this? The narrative
anchor is the single most important output of the design process. Every mechanical decision
must serve it.

**Example narrative anchors:**
- "A lone traveler stands in an endless golden field, watching in quiet awe as impossibly
  vast runestones descend from the heavens. It's not destruction — it's revelation, ancient
  knowledge returning to the earth."
- "A feral wolf-spirit erupts from a thicket of thorns, trailing wisps of violet light.
  It's hunting, but what it hunts is not prey — it's something lost, something it needs
  to reclaim."
- "Two figures stand back-to-back on a crumbling bridge over a void. They're not fighting
  each other — they're the last defense against something unseen below."

Then identify the **practical constraints** the art places on the design:

- Art showing a larger-than-human character must correspond to an expensive/high-spark
  character card
- Art with a positive/uplifting mood should have a positive-coded game effect for the player
  such as drawing cards
- Art with a horror/destructive mood should have a negative-coded effect such as interacting
  with the opponent or the void

The narrative anchor is the source of truth. If at any point during the design process you
find yourself justifying a mechanic that doesn't match the anchor, stop and either revise
the mechanic or revise the anchor — don't force-fit.

### Art-to-Mechanic Translation Guide

The tables below are a **starting point to get you thinking**, not an authoritative mapping.
Most art combines multiple visual elements, and the best designs often bridge elements from
different rows — e.g., golden light + ruins might suggest revelation FROM the void (mill +
draw, or kindle from mill), not just one or the other. Use these associations as creative
seeds, then let the specific art guide you to a mechanic that tells its unique story.

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
| Ruins, decay, entropy | Mill, void recursion, scaling with void count |
| Portals, gateways, thresholds | Banish/return, blink, zone transitions |
| Shields, barriers, protection | Prevent, cost increase, immunity |
| Mirrors, reflections, duality | Copy effects, symmetrical effects |

**Size-to-cost mapping for characters:**

| Character Scale | Typical Cost | Typical Spark |
|---|---|---|
| Small creature (cat, imp, child) | 1-2 | 0-1 |
| Human-sized figure | 2-4 | 1-3 |
| Large creature/imposing figure | 4-6 | 3-5 |
| Titanic/mythic being | 7-9 | 5-7 |

**Scale-to-cost mapping for events:**

The visual drama and scale of the art should match the event's cost and impact:

| Art Scale | Typical Cost | Effect Tier |
|---|---|---|
| Small/intimate (a hand gesture, a whisper, a single rune) | 0-1 | Cantrip, minor trick |
| Medium action (a spell being cast, a figure in motion) | 2-3 | Core utility, targeted removal |
| Large/dramatic (a massive structure, a storm, a ritual) | 3-5 | High-impact, sweeper, finisher |
| Cataclysmic (world-altering, apocalyptic) | 5-7 | Board wipe, game-defining |

**Mood-to-effect mapping:**

| Mood | Player Effect | Opponent Effect |
|---|---|---|
| Serene, contemplative | Foresee, card selection | — |
| Joyful, triumphant | Draw cards, gain energy | — |
| Mysterious, hidden | Discover, look at top of deck | — |
| Melancholic, nostalgic | Void recursion, Reclaim, return from void | — |
| Ominous, foreboding | Foresee, taxation, cost increase | — |
| Awe, sublime wonder | Scaling effects, large mill, high-impact one-shots | — |
| Defiant, rebellious | Cost reduction, alt-cost, rule-breaking | — |
| Aggressive, intense | — | Dissolve, forced sacrifice |
| Eerie, haunting | Void recursion, Reclaim | Discard from hand |
| Chaotic, explosive | — | Mass removal, sweepers |

# Phase 3: Connect to Tides

Choose a tide based on **which tide's philosophy resonates with your narrative anchor** —
not which tide's mechanics seem useful. Read the tide philosophies in `docs/tides/tides.md`
and ask: "Which worldview does this art's story belong to?" A traveler witnessing ancient
knowledge descending could be Bloom (patience rewarded), Arc (thresholds and transitions),
or Surge (knowledge as power) — the same art can belong to different tides depending on
which emotional thread you pull. Pick the tide whose *philosophy* makes the art's story
richer.

**The primary goal is making a great card for its tide.** If a design also happens to work
well in an adjacent tide's hybrid deck, that's a nice bonus — but never compromise a card's
core identity to chase cross-tide appeal. A card that's excellent in one tide is better than
a card that's mediocre in two.

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
a bonus for draftability — but don't force it.

- **Bloom + Arc:** Limited overlap — Bloom's ramp and Arc's blink don't naturally combine
- **Arc + Ignite:** Token tempo (blink token-producers)
- **Ignite + Pact:** Aristocrats (generate expendable bodies, sacrifice for value)
- **Pact + Umbra:** Void engine (fill void from two angles)
- **Umbra + Rime:** Grindy midrange (discard and mill to void, recur)
- **Rime + Surge:** Full control (proactive disruption + reactive counterspells)
- **Surge + Bloom:** Ramp combo (accelerate energy, chain events)

When reviewing the card pool dump, glance at whether your chosen tide already uses
the mechanic you're considering. A mechanic that appears in 0 cards in a tide is a yellow
flag — but if the art's story demands it, that's a valid reason to introduce it.

### Tide Cost

Tide cost is a deckbuilding constraint — higher tide cost requires deeper commitment to that
tide. **Most cards should be tide-cost 1** — this is the default. Only increase tide cost
when the card explicitly rewards being deep in a tide's strategy.

- **Tide-cost 1 (default):** Lightly committed; splashable in hybrid decks. Use this unless
  you have a specific reason for higher commitment.
- **Tide-cost 2:** Card references tide-specific resources or conditions (void count,
  discard triggers, warrior count, "events matter" payoffs). The card is clearly stronger
  when surrounded by its tide's other cards.
- **Tide-cost 3:** Extreme build-around or alternate win condition. Very rare — only for
  cards that define an entire deck.
- **Neutral tide-cost 1:** Playable in any deck.

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

# Phase 4: Validate Against Existing Cards

**Note:** These phases are presented linearly but the process is iterative. Research may
reveal that your tide choice (Phase 3) doesn't work, or concept exploration (Phase 5) may
send you back to research. Loop back when earlier decisions need revision.

**Research is for validation, not generation.** You should already have an art-driven concept
in mind from Phases 2-3. The purpose of research is to check that your concept doesn't
duplicate an existing card and to find the right cost/stats. Do NOT use research to find
"gaps" in the card pool and then backfit an art justification — that produces generic
designs disconnected from the art.

You should already have the full card pool dump in context (from the `dump` command run at
the start). Use it to examine your chosen tide's cards, check for name collisions, and
identify comparable cards. Supplement with targeted commands as needed:

1. `similar <phrase>` — find cards with similar rules text to your concept
2. `cost-in-tide <tide> <cost>` — check saturation at your cost point

### What to Look For

- **Duplicate check:** Search for your concept's rules text phrasing. If an existing card
  already does what you're designing, find a different angle — but stay true to the art's
  story. Change the mechanic, not the narrative anchor.
- **Saturation check:** How many cards already exist at your cost point in your tide? If
  there are already 12 cards at 3● in Umbra, consider a different cost.
- **Closest comparables:** Identify the 2-3 existing cards most similar to your concept.
  Your design should be meaningfully different from each of them. These become your "Similar
  Cards" in the final design.
- **Templating:** Copy the exact phrasing patterns from existing cards. Don't invent new
  templating for effects that already have established wording.

### Spark-per-Cost Benchmarks (Characters)

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

### Event Cost Benchmarks

The "going rate" for common event effects in the existing card pool:

| Effect | Typical Cost | Notes |
|---|---|---|
| Draw 1 | 0-1 | Usually a rider on another effect |
| Draw 2 + discard 1-2 ("loot") | 2 | Standard filtering |
| Draw 2 (no discard) | 4 | Pure card advantage is expensive |
| Draw 3 + discard 2 | 3 | Standard 3-cost draw |
| Foresee 3 + draw 1 | 2 | Fast variant available |
| Gain 4● (net +2) | 2 | Standard energy burst rate |
| Gain 6● (net +2) | 4 | Same net rate at larger scale |
| Mill 2-3 + minor bonus | 1 | Mill is cheap because it needs synergy |
| Mill 3 + draw 1 | 1 | Umbra cantrip baseline |
| Dissolve (conditional) | 1-2 | Requires spark/cost restriction |
| Dissolve (unconditional, fast) | 3 | The key removal benchmark |
| Dissolve (unconditional) + drawback | 2 | Must lose 2 max ● or 4 ✪ |
| Dissolve all (board wipe) | 6 | Format-defining at any cost |
| Dissolve all spark ≤ 2 | 4 | Conditional sweeper |
| Prevent (unconditional, fast) | 2 | Standard counterspell rate |
| Prevent (conditional or drawback) | 0-1 | Must have significant downside |
| Return enemy to hand + bonus | 3 fast | Tempo play with rider |
| Return from void to hand | 2 | Baseline recursion |
| Return from void to hand + draw 1 | 2 | Slightly above-rate for Umbra |
| Banish enemy until next main | 3 fast | Tempo removal |
| Materialize 3 figment tokens | 3 | Go-wide baseline |
| Reclaim character (cost ≤ 3) | 3 | Needs setup (card must be in void) |
| Reclaim up to 3 (cost ≤ 2) | 4 | Premium recursion |
| Copy next event played | 3 | Engine setup |
| Kindle 1-2 (one-shot) | 0-1 | Usually a rider on another effect |
| Kindle 3-4 (one-shot) | 1-2 | Scaling/conditional variants |
| Kindle 1/turn (on character) | 1-2● of budget | Part of a character's ability value |

**Event costing principles:**
- **Fast adds ~1●** to an effect's cost, or the card gets a restriction instead.
- **Reclaim costs** on events are typically equal to or slightly above the card's printed
  cost (e.g., a 2● event might have Reclaim 2-3●). Reclaim is priced at a premium because
  replaying from void is inherently card-advantageous.
- **Scaling effects** (e.g., "draw 1 per ally on the battlefield") should be evaluated at
  their average case, not best case. An effect that averages 3 draws should be costed for
  ~3 draws worth of value.
- **Net value** matters more than gross. A 4● event that gains 6● is net +2, same as a 2●
  event that gains 4●. The higher-cost version is worse because it requires more upfront
  energy.

### Deck Composition Reference

When evaluating scaling effects based on card type (e.g., "kindle 1 per character milled"),
use these baselines:

- **~60-65% of cards in a typical deck are characters**, ~35-40% are events
- A mill-5 that scales per character averages ~3-3.5 hits
- A mill-5 that scales per event averages ~1.5-2 hits
- Deck size is typically 30 cards

### Reclaim on Events

Whether to add Reclaim to an event depends on tide and context:

- **Umbra** events frequently have Reclaim — the void is Umbra's second hand, and replaying
  events from the void is core to the tide's identity
- **Surge** events sometimes have Reclaim — replaying events feeds "events matter" triggers
- **Rime** events occasionally have Reclaim — discarding a reclaim event to void sets up a
  future replay
- **Other tides** rarely have Reclaim on events unless there's a specific narrative or
  mechanical reason
- Events that represent a singular, unrepeatable moment narratively should not have Reclaim
- Events with self-mill effects should consider Reclaim since they'll end up in the void
  anyway and replaying them creates a satisfying loop

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

Before committing to a final design, generate **3 concept sketches that tell different
stories about the art**. Each concept must start from a different emotional or narrative
reading of the art — not the same reading with different mechanics bolted on. The goal is
to explore what the art *means* before deciding what it *does*.

Each concept should be:

- A one-sentence narrative interpretation of the art (what's happening, who is this, how
  does it feel)
- The tide that matches that interpretation and approximate cost
- The mechanic that flows from the narrative (in parentheses)

**Example concepts for art showing a hooded figure in rain catching glowing fragments:**
1. "A watcher shields the world from a cascade of broken spells — Rime 3●/2✦ (discard 2
   to prevent a played card; discarded cards are the fragments caught)"
2. "A scavenger of lost dreams, gathering fragments of forgotten power from the rain —
   Umbra 3●/1✦ (Judgment: mill 2, return a non-character from void; catching = retrieving)"
3. "A seer reading omens in the patterns of falling light — Rime 2●/1✦ (when you discard,
   foresee 1; fragments reveal glimpses of the future)"

Notice how each concept tells a *different story* about who the figure is and what they're
doing — not just "same figure, different mechanic."

### Concept Evaluation Criteria

Pick the concept that best satisfies ALL of these, in priority order:

1. **Narrative resonance (most important):** Does the mechanic feel like it *is* what the
   art depicts? The best card designs create an "aha" moment where the mechanic and art
   feel inseparable — you can't imagine different art on this card, or a different mechanic
   for this art. If you have to write more than one sentence explaining how the mechanic
   connects to the art, the connection is too weak.
2. **Tide fit:** Does the narrative interpretation naturally belong to this tide's
   philosophy? Does the mechanic advance the tide's primary strategy?
3. **Simplicity:** Can you express it in one clean rules text block?
4. **Play-pattern appeal:** Is this card fun to play? The best cards create satisfying
   moments: variance that generates excitement, meaningful decisions (modal choices,
   targeting decisions), or deckbuilding rewards (scaling effects that pay off investment).
   Avoid mechanics that feel bad to use — for example, effects that make a player lose
   victory points feel terrible even when they're balanced.
5. **Novelty:** Is it meaningfully different from existing cards? (Check using research.)
   Novelty matters, but it is the *least* important criterion — a card that perfectly
   captures its art in a familiar mechanic is better than a "novel" card that doesn't
   match the art.

### Cost-to-Excitement Scaling

A card's energy cost sets the player's expectations for how dramatic it should feel. A 2●
common can be a simple stat body or a modest trigger — the investment is small, so the
payoff can be small. But when a player spends 7● on a rare, they've committed most of their
turn's energy to one play. That moment needs to feel *worth it*.

**The problem with "just numbers":** A 7● character with "▸ Judgment: Kindle 3" is
mechanically strong but experientially flat — nothing interesting happens when you play it,
nothing interesting happens each turn. It's a number that goes up. Compare this to
"▸ Materialized: Materialize a random Spirit Animal from your deck" — the moment you slam
this card, something exciting and unpredictable happens. The player watches to see what they
hit, the board state visibly changes, and the game creates a *moment*.

**What makes expensive cards feel exciting:**

| Element | Example | Why it works |
|---|---|---|
| Immediate board impact | "▸ Materialized: Dissolve an enemy" | Something happens RIGHT NOW |
| Variance / discovery | "Materialize a random character from your deck" | Anticipation and surprise |
| Scaling with investment | "Each allied Spirit Animal gains +2 spark" | Rewards the work you did getting here |
| Meaningful choices | "Choose one: [3 options]" | Player agency in a big moment |
| Dramatic opponent interaction | "The opponent abandons 3 characters" | Reshapes the entire game state |

**Guidelines by cost:**

- **0-2●:** Simple, incremental, or engine-piece effects are fine. "Judgment: Gain 1●" is
  perfectly acceptable at 1●. The card earns its keep through repetition over many turns.
- **3-4●:** Should have at least one moment of interest — a materialized trigger, a
  meaningful decision, or a conditional payoff that feels good when met.
- **5-6●:** Must create a noticeable board shift when played. If someone watches you play
  this card and nothing visibly changes, it's too passive for the cost.
- **7+●:** This is your big play of the turn. It should create a *story* — a moment the
  player remembers. Materialized triggers are almost mandatory at this cost. Pure
  passive/judgment-only effects need an extremely compelling reason to justify this
  investment with no immediate payoff.

This doesn't mean expensive cards must be *complex* — "▸ Materialized: Draw 3 cards" is
simple but dramatically impactful at 7●. The goal is that the *moment of playing the card*
feels proportional to the energy spent.

### Refine Before Committing

After picking a concept, stress-test it before writing the final design:

- **Narrative anchor check:** Re-read your narrative anchor from Phase 2. Does the mechanic
  still tell that story, or did you drift during ideation? If you can't explain the
  connection in one sentence without straining, the design has drifted. Go back to the art.
- **Power check:** Estimate the average case for variable effects. Use the event/character
  benchmarks to verify the total value is appropriate for the cost.
- **Play pattern check:** For every triggered ability, answer concretely: who causes this
  to fire, how often, and can they choose not to? If the trigger depends on the opponent
  taking an action they can simply avoid (e.g., "when this character is banished" — the
  opponent will just Dissolve it instead), the ability is effectively blank text. If the
  trigger requires a specific other card in play, estimate how often you'll have it. Then
  mentally play the turn you cast this card and the next 2-3 turns: what concretely happens
  each turn? If the answer is "nothing, unless..." the design is too conditional. Every card
  should do something meaningful through actions the player naturally takes in their tide's
  game plan.
- **Is there a simpler version?** If you have two mechanics stapled together, ask whether
  the card would be better with just one of them at a lower cost. Simpler is almost always
  better.

# Phase 6: Final Design

Develop your chosen concept into a complete card. Write your response with the following:

**Hard limits:** Card names must be 25 characters or fewer. Rules text must be 100
characters or fewer.

- **Card Name:** Evocative short name for this card (max 25 characters)
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
- **Naming:** Location names — evocative place names (Skypath, Autumn Glade, Twilight
  Radiance, Auroral Passage).
- Dreamwell cards have no tide, no cost, no spark, and no rarity.

### Design Principles

1. **Simplicity first.** The best cards have one clean mechanic, not three stapled together.
   If you can express the card in one sentence of rules text, do so. A card should have at
   most 2 mechanical elements (e.g., a trigger + an effect, or a cost reduction + an
   ability).

2. **Match the art.** The narrative must be believable. A serene forest spirit should not
   dissolve enemies. A war machine should not draw cards peacefully. If the art and mechanic
   don't tell the same story, redesign.

3. **Avoid duplication.** If an existing card already does what you're designing, find a
   different angle. Use the research script to check thoroughly.

4. **Cost appropriately.** Use the spark-per-cost benchmarks for characters and the event
   cost benchmarks for events. For variable effects, evaluate the average case, not the best
   or worst case. Compare directly to the 2-3 closest existing cards.

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

- **Mechanic-first design:** Finding a "novel" mechanical combination through card pool
  analysis and then backfitting an art justification. If you arrived at your mechanic by
  searching for gaps rather than by reading the art, start over. The art generates the
  concept; research validates it.
- **Parasitic design:** Cards that do literally nothing without specific other cards. Every
  card should have a baseline of usefulness even without synergy.
- **Strict duplicates:** If an existing card does the same thing at the same cost, your
  design needs a different angle.
- **Stapled mechanics:** Two unrelated abilities on one card (e.g., "Draw 2 cards. Dissolve
  an enemy.") with no connecting theme. The abilities should form a cohesive whole.
- **Wrong-tide mechanics:** A Rime card that generates figment tokens, or a Surge card with
  abandon synergies. Mechanics should belong to the card's tide or an adjacent ally.
- **Opponent-cooperative triggers:** Abilities that rely on the opponent taking a specific
  action they can choose to avoid. "When this character is banished" sounds elegant but the
  opponent will Dissolve it instead. "When the opponent discards" gives them veto power over
  your card text. Good triggers fire from things under YOUR control or things the opponent
  MUST do (like playing cards at all). Test: could a smart opponent make this ability blank
  by changing their play? If yes, it's too weak to be a card's primary mechanic.
- **Overcomplexity:** Rules text has a hard limit of 100 characters. If you can't fit the
  effect in 100 characters, simplify. The best designs are often the most elegant.

### Naming Guidelines

Names must be **25 characters or fewer** and should be evocative and creative. Most names
are 2-3 words. Use the research script (`name` command) to check the existing naming
landscape and avoid collisions, but don't feel constrained to follow rigid patterns. Some
common structures that appear in the card pool:

- **[Adjective] [Noun]:** Silent Avenger, Eternal Sentry
- **[Compound] [Noun]:** Bloomweaver, Starcatcher
- **[Place] [Role]:** Neon Street Wanderer
- **[Noun] of [Noun]:** Titan of Forgotten Echoes, Blade of Oblivion
- **Single word:** Apocalypse, Reunion, Nocturne
- **The [Title]:** The Devourer, The Rising God

These are guidelines, not rules. Prioritize names that are memorable, evocative of the
card's identity, and feel natural. Creative names that don't fit any pattern are fine.

