---
name: card-design
description: Design a new Dreamtides card based on art input. Use when designing cards, creating card concepts, or translating art into game mechanics. Triggers on card design, design card, new card, card art, card concept.
---

# Card Design Skill

You are an expert card game designer, creating novel game designs with deep thinking and
analysis. Run everything with ultrathink.

Read `docs/battle_rules/battle_rules.md` and `docs/tides/tides.md` (use the Read tool).

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

### Art-to-Dynamic Translation Guide

The table below maps visual elements to the **dynamics and feelings** they suggest — not to
specific game keywords. The goal is to spark creative thinking, not to channel you into
pre-existing mechanical patterns. When you see a portal in the art, don't immediately think
"banish/blink" — think about what a portal *means narratively* (transformation, crossing a
threshold, the moment between before and after) and then invent a mechanic that captures
*that* feeling, whether or not it maps to an existing keyword.

| Visual Element | Dynamic It Suggests |
|---|---|
| Golden light, sparks, radiance | Revelation, uncovering, gaining knowledge or options |
| Darkness, shadow, void imagery | Hidden resources, delayed payoff, things not yet found |
| Fire, explosion, destruction | Decisive action, removing obstacles, clearing the way |
| Growth, nature, blooming | Acceleration, compounding returns, investment that pays off |
| Speed, motion blur, flight | Tempo, responsiveness, being ahead of the moment |
| Crowds, armies, banners | Strength in numbers, collective power, coordination |
| Sacrifice, altars, blood | Trading one thing for something better, transformation through loss |
| Ice, winter, austerity | Discipline, doing more with less, denial |
| Machinery, technology, circuits | Repeatability, precision, systematic advantage |
| Magical cascades, chains, links | Sequences, momentum, each action enabling the next |
| Ruins, decay, entropy | Value in what's been lost, the past as resource |
| Portals, gateways, thresholds | Transformation, conditionality, "before and after" moments |
| Shields, barriers, protection | Control over what happens, authority, selective denial |
| Mirrors, reflections, duality | Echoing, doubling, symmetry or asymmetry |

These dynamics are starting points, not endpoints. The best designs often combine two
dynamics in an unexpected way — e.g., "revelation + loss" might become a card that forces
you to show your hand to gain a powerful effect, or "tempo + sacrifice" might be a card
that returns itself to hand to save an ally but costs you a turn of spark.

**Size-to-cost mapping for characters:**

| Character Scale | Typical Cost | Typical Spark |
|---|---|---|
| Small creature (cat, imp, child) | 1-2 | 0-1 |
| Human-sized figure | 2-4 | 1-3 |
| Large creature/imposing figure | 4-6 | 3-5 |
| Titanic/mythic being | 7-9 | 5-7 |

**Scale-to-cost mapping for events:**

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

# Phase 4: Explore Concepts

**Read the card pool NOW** — before generating concepts. Run the dump command to load every
card:

```bash
python3 .claude/skills/card-design/card-research.py dump
```

**Why read first:** You cannot design something genuinely novel if you don't know what
already exists. The pool read is defensive — it tells you what to AVOID, not what to DO.
Do NOT let the pool drive your concepts. The art and narrative anchor drive concepts; the
pool prevents you from accidentally recreating an existing card.

**How to use the pool:** Scan for mechanical patterns, not specific cards. Note which
*play experiences* are well-covered (e.g., "character that draws when materialized" appears
many times) and which are absent. Then CLOSE THE POOL mentally and return to the art. Your
concepts should flow from the narrative anchor, informed by the knowledge of what the game
has already explored.

Before committing to a final design, generate **4 concept sketches that tell different
stories about the art**. Each concept must start from a different emotional or narrative
reading of the art — not the same reading with different mechanics bolted on. The goal is
to explore what the art *means* before deciding what it *does*.

Each concept should be:

- A one-sentence narrative interpretation of the art (what's happening, who is this, how
  does it feel)
- The tide that matches that interpretation and approximate cost
- The mechanic that flows from the narrative (in parentheses)

**Critical requirement: at least 3 of the 4 concepts must be "wild" concepts** — mechanics
or mechanical combinations that you believe do not currently exist on any card in the pool.
The default expectation is that the final card will use a wild concept. You may only pick a
non-wild concept if you can articulate a specific, compelling reason why every wild concept
fails for this art — and even then, push harder for a wild alternative first.

**What counts as genuinely wild vs. superficially wild:**

- **NOT wild:** "Materialize a random character from your deck" — this mechanic already
  exists (Light of Emergence, Engine Light Dreamer). Changing the cost restriction, the
  number of characters, or adding "Spirit Animal" does not make it wild. The *play
  experience* is the same: you play a card, a random thing appears from your deck.
- **NOT wild:** "Draw N cards" or "Kindle N" with a new trigger condition. The play
  experience of "something happens, you draw/kindle" is deeply explored. A new trigger
  stapled onto draw/kindle is not a new card — it's a new trigger on a familiar chassis.
- **NOT wild:** Moving an existing mechanic to a different tide. "Abandon an ally: Draw a
  card" in Bloom instead of Pact is not novel — it's the same card in different colors.
- **WILD:** A mechanic that creates a play experience no existing card creates. "The
  opponent chooses: you draw 3 or they abandon a character" — no card gives the opponent
  this kind of choice. "When you foresee, you may also look at the opponent's top card" —
  no card ties foresee to opponent information. "This character's spark equals the number
  of events in your void" — no character scales this way.

**The litmus test for novelty:** Describe the card to a friend who knows the game. If they
say "oh, like [existing card] but different," it's not wild. If they say "wait, you can DO
that?" — it's wild.

**Example concepts for art showing a hooded figure in rain catching glowing fragments:**
1. "A seer reading omens in the patterns of falling light — Neutral 2●/1✦ (when you
   foresee, you may also look at the top card of the opponent's deck; fragments reveal
   hidden knowledge)" **(wild — foresee grants opponent information access, entirely new)**
2. "The figure is not catching the fragments — they're *becoming* them, dissolving into pure
   magic to be reborn — Arc 4●/1✦ (Materialized: banish this character and a card from your
   hand. At end of turn, materialize both and they each gain +2 spark; transformation
   through the rain)" **(wild — self-banish + hand-banish with delayed co-materialization)**
3. "A lightning rod absorbing stray magic before it hits the ground — the more spells that
   fly, the more power this figure holds — Surge 3●/0✦ (this character gains +1 spark each
   time any player plays an event; the fragments are spells accumulating)" **(wild — spark
   scales with global event count, no existing card does this)**
4. "A watcher shields the world from a cascade of broken spells — Rime 3●/2✦ (discard 2
   to prevent a played card; discarded cards are the fragments caught)" (not wild — discard
   as prevent cost is a known pattern, included as backup only)

Notice how concepts 1-3 each create a play experience that doesn't exist anywhere in the
card pool. Concept 4 is the lone non-wild backup — included for completeness but not the
preferred pick.

### Novel Design Space — Think Like a Digital Card Game

Dreamtides is a digital card game. It can track hidden state, randomize, transform cards
in zones, and do things that would be impossible in paper. When generating concepts,
actively consider mechanics from this design space.

**IMPORTANT — Mechanical Gravity Wells to Resist:**
The following mechanics are well-explored in the card pool and act as "gravity wells" that
pull designs toward them. They are not banned, but using them requires justification for
why THIS card's version creates a genuinely different play experience:

- **"Draw N cards" / "Kindle N" as triggers** — The pool has dozens of these on every
  trigger type. A new draw/kindle trigger is only justified if the trigger condition itself
  is unprecedented AND the card's identity is the trigger, not the draw/kindle.
- **"Materialize a random character from your deck"** — Already exists on Light of
  Emergence, Engine Light Dreamer, Speaker for the Forgotten, Beacon Gazer (for events).
  Changing the cost cap or card type doesn't create novelty.
- **"Gain N●" on Judgment** — The Bloom ramp baseline. Another gain-energy-on-judgment card
  needs an extraordinary twist to justify its existence.
- **"Mill N + void synergy"** — The Umbra baseline. Mill-then-benefit is the most common
  Umbra pattern. Don't default here.
- **"Abandon an ally: [benefit]"** — The Pact baseline. Another abandon-for-value outlet
  needs to offer a benefit type that doesn't exist yet.
- **"When you play an event, [benefit]"** — The Surge baseline. Another events-matter
  payoff needs to reward in a way no existing card does.
- **Dissolve/Return/Banish as removal** — The Neutral baseline. Another removal variant is
  only justified if the targeting condition or secondary effect is unprecedented.

If you find yourself writing rules text that fits one of these patterns, STOP and ask: "What
is the play experience this card creates that no existing card creates?" If you can't answer
clearly, the concept needs more creative work.

The following are **examples of genuinely unexplored territory** — not a checklist, but a
prompt to think bigger:

**Cards that transform or evolve:**
- A card in your hand that changes (gains abilities, reduces cost, transforms into a
  different card) based on game events while you hold it
- A character that becomes a different character when a condition is met
- An event that upgrades itself each time it's played from void (via Reclaim)

**Cards that interact with hidden information:**
- Reveal the top card of the opponent's deck; effect depends on what's revealed
- Look at the opponent's hand and gain a bonus based on what you see
- Effects that scale with information neither player fully controls (top of deck,
  random selection from deck)

**Cards with unusual scaling or conditions:**
- Effects that scale with the score differential (stronger when you're behind, or when
  you're ahead)
- Cards that care about the current turn number or energy production tier
- Effects based on the total cost of characters on either battlefield
- Cards that count something unusual (events in your void, characters the opponent has
  materialized this game, number of times you've used foresee)

**Cards that give the opponent choices (punisher mechanics):**
- "The opponent chooses: you draw 3 cards, or they abandon a character"
- Effects where the opponent picks which of two bad outcomes happens to them
- Cards that get stronger if the opponent takes a specific public action

**Cards that change rules or modify other cards:**
- "Characters you materialize this turn gain the Judgment ability of the first character
  you materialized"
- "The next character you play this turn costs 0● but is abandoned at end of turn"
- "Until end of turn, your events also kindle 1 when played"
- Effects that temporarily change what a zone does (void becomes a second hand, top of
  deck is revealed)

**Cards with unusual timing or delayed effects:**
- Effects that trigger N turns from now ("At the start of your 3rd turn after this one...")
- Cards that set up a future payoff visible to both players
- Characters that do nothing initially but have an inevitable payoff if they survive

**Cards that create variance and moments:**
- Play a random card from your deck (some versions exist — explore more)
- Effects that generate a random card not in your deck (from a pool)
- Discover with unusual criteria ("discover a card the opponent has in their deck")
- "Joust" effects — reveal from both decks, compare, winner gets a bonus

These mechanics create the memorable moments that players talk about after a game. A card
that says "▸ Judgment: Kindle 2" is forgettable. A card that says "▸ Materialized: Reveal
the top 3 cards of the opponent's deck. Play one of them for 0●" creates a *story*.

### Concept Evaluation Criteria

**Novelty has veto power.** A concept that fails the novelty test is rejected regardless
of how well it scores on other criteria. The game already has 300+ cards — it does not need
another competent-but-familiar design. It needs cards that create new play experiences.

1. **Mechanical novelty (REQUIRED — veto power):** Does this card create a play experience
   that no existing card creates? This is not about the specific numbers, trigger condition,
   or tide — it's about the *experience at the table*. If you can point to an existing card
   and say "playing my card feels like playing that card," the concept fails. You must be
   able to complete this sentence: "No existing card ___" where the blank describes the
   unique play experience. If you cannot, reject the concept and try again.
2. **Narrative resonance:** Does the mechanic feel like it *is* what the art depicts? The
   best card designs create an "aha" moment where the mechanic and art feel inseparable.
   If you have to write more than one sentence explaining how the mechanic connects to the
   art, the connection is too weak.
3. **Tide fit:** Does the narrative interpretation naturally belong to this tide's
   philosophy? Does the mechanic advance the tide's primary strategy?
4. **Simplicity:** Can you express it in one clean rules text block? Novelty does not mean
   complexity — the best novel designs are often elegantly simple.

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
  benchmarks (in Phase 5) to verify the total value is appropriate for the cost.
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

### Novelty Gate (MANDATORY)

Before proceeding to Phase 5, you must pass this gate. Write the following:

**"No existing card ___."** Complete this sentence with the unique play experience your
chosen concept creates. This must describe a *play experience*, not a cosmetic difference.

**PASS examples:**
- "No existing card lets the opponent choose between two bad outcomes."
- "No existing card has spark that scales with the number of events in your void."
- "No existing card transforms into a different card while in your hand."
- "No existing card punishes the opponent for having more characters than you."

**FAIL examples:**
- "No existing card materializes a random Spirit Animal costing 2 or less." (This is
  "materialize random character from deck" with a subtype filter — same play experience
  as Light of Emergence.)
- "No existing card draws a card when a Warrior is banished." (This is "draw a card on
  [trigger]" — same play experience as dozens of existing cards with different triggers.)
- "No existing card in Bloom kindles 3 on materialized." (Kindle-on-materialized exists
  in other tides — different tide is not a different play experience.)

If you cannot pass the gate, **return to Phase 4 and generate new concepts.** Do not
proceed with a derivative design. The purpose of Phase 5 is to refine a novel concept, not
to rescue a derivative one.

# Phase 5: Validate Against Existing Cards

Use targeted commands to validate your chosen concept:

```bash
# Search rules text for a phrase (check for duplicates of your concept)
python3 .claude/skills/card-design/card-research.py similar "when you discard"

# Show cards at a specific cost within a specific tide (saturation check)
python3 .claude/skills/card-design/card-research.py cost-in-tide Umbra 3
```

**Note:** These phases are presented linearly but the process is iterative. Validation may
reveal that your concept duplicates an existing card, your tide doesn't work, or your cost
is wrong. **When this happens, return to Phase 4 and generate entirely new concepts** — do
not try to rescue a derivative concept by adjusting numbers. Carry the creative ambition
forward: if your wild concept collides with an existing card, find a *different* wild angle.
Never fall back to a safe, derivative design just because validation was hard.

### What to Look For

- **Duplicate check:** Search for your concept's key mechanical phrases. If an existing card
  already creates the same play experience, **return to Phase 4** — do not simply adjust
  numbers or cost. A truly novel concept can survive discovering a partial overlap; a
  derivative concept cannot be rescued by tweaking.
- **Differentiation test (MANDATORY):** For each of the 2-3 closest comparable cards, write
  one sentence explaining how your card creates a **different play experience** — not just
  different numbers, cost, tide, or trigger condition. If you find yourself writing "mine
  does X instead of Y" where X and Y are the same type of effect (e.g., "mine draws 2
  instead of 1," "mine targets cost ≤3 instead of ≤2"), the differentiation is
  insufficient. **Return to Phase 4.**
- **Saturation check:** How many cards already exist at your cost point in your tide? If
  there are already 12 cards at 3● in Umbra, consider a different cost.
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
- **Novelty Statement:** "No existing card ___." The unique play experience, carried from
  the Novelty Gate.
- **Similar Cards:** 2-4 existing cards with the closest mechanical overlap, with one
  sentence each explaining why your card creates a **different play experience** (not just
  different numbers).

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

1. **Novelty is non-negotiable.** Every card must create a play experience that doesn't
   already exist in the card pool. "Solid but not exciting" is a rejection. Before
   finalizing, ask: "Could I describe this card to a friend and have them say 'whoa, that's
   cool'?" If the answer is "it's like [existing card] but [minor difference]," the design
   has failed — return to Phase 4. The card pool has 300+ cards. It does not need another
   competent role-player. It needs cards that create new stories.

2. **Match the art.** The narrative must be believable. A serene forest spirit should not
   dissolve enemies. A war machine should not draw cards peacefully. If the art and mechanic
   don't tell the same story, redesign.

3. **Simplicity serves novelty.** The best novel designs are elegantly simple — one clean
   idea expressed clearly, not three ideas stapled together. If you can express the card in
   one sentence of rules text, do so. A card should have at most 2 mechanical elements
   (e.g., a trigger + an effect, or a cost reduction + an ability).

4. **Avoid duplication.** If an existing card already does what you're designing, find a
   different angle. Use the research script to check thoroughly.

5. **Cost appropriately.** Use the spark-per-cost benchmarks for characters and the event
   cost benchmarks for events. For variable effects, evaluate the average case, not the best
   or worst case. Compare directly to the 2-3 closest existing cards.

6. **Consider the digital medium.** Digital card games can use mechanics with interesting
   variance (e.g., "discover a character" from a random set, scaling effects based on game
   state). Embrace mechanics that would be impractical in paper.

7. **Tide commitment should match power.** Tide-cost 1 cards are good role-players.
   Tide-cost 2 cards should meaningfully reward being in that tide. Tide-cost 3 cards
   should be build-arounds.

8. **Think about the draft.** Cards that only work in one specific deck are less interesting
   than cards that are good in their primary tide but also playable in an adjacent hybrid
   strategy.

9. **Rarity guides complexity.** Common cards should have simple, clean effects. Uncommon
   cards can have one conditional or synergy-based ability. Rare cards can have more complex
   or build-around effects. Legendary cards are format-defining and uniquely powerful.

### Design Anti-Patterns to Avoid

- **Derivative design (the #1 problem — instant rejection):** If your card's play
  experience can be described as "[existing card] but [different numbers/tide/trigger]," it
  is derivative and must be rejected. This includes:
  - Same mechanic in a different tide ("abandon-for-value but in Bloom")
  - Same mechanic with a different number ("kindle 3 instead of kindle 2")
  - Same mechanic with a different trigger ("draw on banish instead of draw on materialize")
  - Same mechanic with a different restriction ("cost ≤3 instead of cost ≤2")
  - Same mechanic on a different body ("materialize random from deck, but on a character
    instead of an event")
  The card pool already has hundreds of "draw a card," "kindle N," and "gain N●" triggers.
  These effects can appear as *riders* on a card whose primary identity is novel, but they
  must never BE the card's identity.
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
