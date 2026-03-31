---
name: card-design
description: Design a new Dreamtides card from art + Hearthstone inspiration. Produces 5 candidate designs for the user to evaluate, iterates on the chosen design, then writes it to rendered-cards.toml and commits. Triggers on card design, design card, new card, card art, card concept.
---

# Card Design Skill

You are an expert card game designer creating novel Dreamtides cards from art input, drawing
mechanical inspiration from Hearthstone. Your design must be mechanically novel in the
Dreamtides pool, fun to play, fitting for its tide, AND narratively matched to the art. All
four are required. Run everything with ultrathink.

Read `docs/battle_rules/battle_rules.md` and `docs/tides/tides.md` (use the Read tool).

# Phase 1: Classify Art

The first step is to classify the card art. There are 3 possible types:

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

Before thinking about mechanics, sit with the art and write two things:

**Literal reading:** One sentence about what is physically happening in the art. "A figure
is striking a bear with a pickaxe." "A woman floats above a glowing pool." This grounds
you in what a viewer actually *sees*.

**Narrative anchor:** 2-3 sentences describing what you see in pure story terms — no game
vocabulary. What emotion does it evoke? What's happening? What kind of person/creature/moment
is this? The narrative anchor is a creative prompt that guides your design — it should
inspire mechanical ideas, but it does not constrain them.

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

### Art-to-Dynamic Translation Guide

The table below maps visual elements to the **dynamics and feelings** they suggest — not to
specific game keywords. The goal is to spark creative thinking, not to channel you into
pre-existing mechanical patterns.

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
dynamics in an unexpected way.

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
and ask: "Which worldview does this art's story belong to?"

**The primary goal is making a great card for its tide.** If a design also happens to work
well in an adjacent tide's hybrid deck, that's a nice bonus — but never compromise a card's
core identity to chase cross-tide appeal.

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

- **Tide-cost 1 (default):** Splashable. Use unless you have a specific reason for higher.
- **Tide-cost 2:** Card references tide-specific resources or conditions.
- **Tide-cost 3:** Extreme build-around. Very rare.
- **Neutral tide-cost 1:** Playable in any deck.

### Character Subtypes

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

| Figment | Spark | Primary Tides |
|---|---|---|
| Celestial | 1 | Arc, Bloom, Surge |
| Radiant | 1 | Ignite, Surge |
| Warrior | 1 | Ignite |
| Shadow | 0 | Pact (primary), Rime, Umbra |

### Fast Speed Guidelines

| Tier | Tides | Fast Card Count |
|---|---|---|
| Primary | **Arc** (23), **Neutral** (17), **Surge** (12) | High density |
| Secondary | Rime (6), Pact (5), Umbra (5), Bloom (4) | Selective use |
| Minimal | Ignite (1) | Almost never |

- **Events:** Fast events are primarily removal, prevention, and card draw. Non-fast events
  are proactive plays.
- **Activated abilities:** Fast activated abilities are used for reactive plays.
- **Characters:** Fast characters can be played reactively. Primarily in Arc.
- **General rule:** If the effect is reactive, it should probably be fast. If proactive, not.

# Phase 4: Read Existing Card Pool & Find Hearthstone Inspiration

This phase has two parallel reads. Run both in the same message:

```bash
# Read the existing Dreamtides card pool (anonymized, ~540 lines)
cat rules_engine/tabula/rendered_cards_anonymized.txt
```

```bash
# Read the Hearthstone card pool (~1919 one-line ability descriptions)
cat ~/Documents/hearthstone/hearthstone.txt
```

**How to use the Dreamtides pool:** Scan for mechanical patterns. Note which *play
experiences* are well-covered (e.g., "draw when materialized," "kindle on trigger," "mill +
void synergy") and which are absent. This tells you what to AVOID. Do NOT let the pool drive
your concepts.

**How to use the Hearthstone pool:** Find 5-10 Hearthstone abilities whose *abstract
mechanics* resonate with the art's narrative anchor and dynamics. You are looking for
mechanical *inspiration*, not cards to port. For each HS ability you flag, extract the
**abstract dynamic** — strip away HS keywords and describe what the mechanic *does* in
universal game terms.

**Example:** Art shows a figure catching glowing fragments in rain.
- HS ability: "After you cast a spell, gain Spell Damage +1" → Abstract: "accumulates power
  from repeated actions, becoming more dangerous over time" → Resonates with "catching
  fragments" = gathering power.
- HS ability: "Whenever a character is healed, deal 1 damage to a random enemy" → Abstract:
  "converts a positive action into offensive output" → Resonates with "catching light and
  redirecting it."

The goal is to find HS mechanics that, when filtered through the art's narrative, could
produce a Dreamtides card that is both mechanically novel AND narratively honest to the art.

# Phase 5: Generate 5 Card Designs

Generate **5 complete card designs**, each inspired by a different Hearthstone ability mapped
through the art's narrative. Each design must include ALL of the following:

1. **HS Inspiration:** Name the HS ability and the abstract dynamic extracted from it
2. **Art Connection:** One sentence on how the art's narrative transforms that dynamic
3. **Card Name:** Evocative short name (max 25 characters)
4. **Card Type:** Character (with subtype) or Event
5. **Tide:** Which tide and its tide cost (1-3)
6. **Energy Cost:** Proposed cost
7. **Spark:** Proposed spark value (characters only; events use "")
8. **Rarity:** Common, Uncommon, Rare, or Legendary
9. **Fast:** Yes/No
10. **Rules Text:** Proposed ability text using templating conventions below (max 100 chars)
11. **Novelty Statement:** "No existing card ___." — the unique play experience
12. **Fun Pitch:** 1-2 sentences on why this card is exciting to play

**Critical requirement: at least 4 of the 5 designs must be "wild"** — mechanics or
mechanical combinations that do not currently exist on any card in the Dreamtides pool.

**What counts as genuinely wild vs. superficially wild:**

- **NOT wild:** "Materialize a random character from your deck" — this mechanic already
  exists. Changing the cost restriction, number, or subtype does not make it wild. The *play
  experience* is the same.
- **NOT wild:** "Draw N cards" or "Kindle N" with a new trigger condition. The play
  experience of "something happens, you draw/kindle" is deeply explored.
- **NOT wild:** Moving an existing mechanic to a different tide.
- **WILD:** A mechanic that creates a play experience no existing card creates. Look for
  mechanics where the *decision space* or *board interaction* is fundamentally different from
  anything in the pool — not just a familiar effect with a new trigger or scaling variable.

**The litmus test for novelty:** Describe the card to a friend who knows the game. If they
say "oh, like [existing card] but different," it's not wild. If they say "wait, you can DO
that?" — it's wild.

**Art/narrative check for each design:** Can you explain in one sentence why this mechanic
belongs on this art? The connection can be metaphorical or abstract, but it must be honest —
a viewer looking at the art should be able to nod and say "yeah, I can see that." If the
mechanic actively contradicts what the art depicts, reject the concept.

### Novel Design Space — Think Like a Digital Card Game

Dreamtides is a digital card game. It can track hidden state, randomize, transform cards
in zones, and do things impossible in paper. When generating designs, actively consider
mechanics from this design space.

**IMPORTANT — Mechanical Gravity Wells to Resist:**
The following mechanics are well-explored and act as "gravity wells." Using them requires
justification for why THIS card's version creates a genuinely different play experience:

- **"Draw N cards" / "Kindle N" as triggers** — Dozens exist. A new draw/kindle trigger
  is only justified if the trigger condition itself is unprecedented AND the card's identity
  is the trigger, not the draw/kindle.
- **"Materialize a random character from your deck"** — Already exists on multiple cards.
- **"Gain N● on Judgment"** — The Bloom ramp baseline.
- **"Mill N + void synergy"** — The Umbra baseline.
- **"Abandon an ally: [benefit]"** — The Pact baseline.
- **"When you play an event, [benefit]"** — The Surge baseline.
- **Dissolve/Return/Banish as removal** — The Neutral baseline.
- **"+N spark/energy/draw when you materialize a character"** — Heavily explored across
  multiple tides.

If you find yourself writing rules text that fits one of these patterns, STOP and ask: "What
is the play experience this card creates that no existing card creates?"

**Avoid "opponent chooses" / punisher mechanics.** In practice one option is almost always
correct for the opponent, so the "choice" is illusory and the card collapses to a fixed
effect. This design space is not fruitful — do not use it.

### Design Evaluation Criteria

All four criteria are required. A design that fails any one is rejected.

1. **Mechanical novelty (REQUIRED — veto power):** Does this card create a play experience
   that no existing card creates? You must complete: "No existing card ___" with a unique
   play experience. If you cannot, reject the concept.
2. **Fun factor (REQUIRED):** Would a player be excited to discover this card in a draft?
   Does playing it create a memorable moment?
3. **Tide fit (REQUIRED):** Does the mechanic advance the tide's primary strategy?
4. **Art/narrative match (REQUIRED):** Does the mechanic connect to the art? The connection
   can be metaphorical or abstract, but a viewer looking at the art must be able to see the
   relationship. The mechanic must not actively contradict what the art depicts.

### Cost-to-Excitement Scaling

- **0-2●:** Simple, incremental effects fine. Card earns its keep through repetition.
- **3-4●:** Should have at least one moment of interest.
- **5-6●:** Must create a noticeable board shift when played.
- **7+●:** This is your big play. It should create a *story*.

### Costing Guidelines

Energy cost and spark are a rough estimate — don't overthink them. Use the anonymized card
pool as a reference: find cards at similar cost points with similar power levels and match
accordingly. Characters typically have spark roughly equal to half their energy cost (e.g.,
cost 4 → spark 2), with powerful abilities trading spark downward. Fast adds ~1● to an
effect's cost. Reclaim is mainly for Umbra, Surge, and Rime events.

### Templating Conventions

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

### Design Principles

1. **Mechanical novelty is the #1 priority.** The card must do something new in the
   Dreamtides pool. If you can describe the card as "[existing card] but [different]," it
   fails.
2. **Fun is non-negotiable.** The card must be exciting to play. "Solid but not exciting"
   is a rejection.
3. **Tide fit matters.** The mechanic should advance the tide's primary strategy.
4. **Art/narrative match is required.** The mechanic must connect to the art. A viewer
   looking at the art must be able to see the relationship. The connection can be
   metaphorical, but it must be honest — not forced or contradictory.
5. **Simplicity serves novelty.** One clean idea expressed clearly, not three ideas stapled
   together.
6. **Cost appropriately.** Use the benchmarks above.
7. **Embrace the digital medium.** Mechanics that would be impractical in paper are welcome.
8. **Rarity guides complexity.** Common = simple. Uncommon = one conditional. Rare = complex
   or build-around. Legendary = format-defining.

### Design Anti-Patterns to Avoid

- **Derivative design (the #1 problem — instant rejection):** "[existing card] but
  [different numbers/tide/trigger]." This includes same mechanic in a different tide, with
  different numbers, a different trigger, or on a different body.
- **1:1 Hearthstone port:** Translating HS keywords into Dreamtides equivalents without
  creative reinterpretation. The card should be *inspired by*, not *converted from*.
- **Art contradiction:** A mechanic that actively contradicts what the art depicts. A serene
  forest spirit probably shouldn't dissolve enemies. A destructive explosion probably
  shouldn't heal.
- **Parasitic design:** Cards that do literally nothing without specific other cards.
- **Opponent-cooperative triggers:** Abilities the opponent can make blank by changing play.
- **Opponent chooses / punisher mechanics:** One option is almost always correct for the
  opponent, so the "choice" is illusory. Do not use this design space.
- **Overcomplexity:** If it doesn't fit in 100 characters, simplify.
- **Stapled mechanics:** Two unrelated abilities with no connecting theme.
- **Wrong-tide mechanics:** A Rime card generating figments, a Surge card with abandon.
- **Kindle as identity:** Kindle should never be the card's primary mechanic.
- **Pure gap-filling:** Mechanically scanning the pool for "what's missing" and generating
  a design to fill the gap, with no creative spark from the art.

### Naming Guidelines

Names must be **25 characters or fewer**. Most names are 2-3 words. Common structures:

- **[Adjective] [Noun]:** Silent Avenger, Eternal Sentry
- **[Compound] [Noun]:** Bloomweaver, Starcatcher
- **[Place] [Role]:** Neon Street Wanderer
- **[Noun] of [Noun]:** Titan of Forgotten Echoes, Blade of Oblivion
- **Single word:** Apocalypse, Reunion, Nocturne
- **The [Title]:** The Devourer, The Rising God

# Phase 6: Present Designs for Evaluation

Present all 5 designs to the user in a clear numbered list format. For each design, show:

1. **Card Name** — Tide, Cost●/Spark✦, Type (Subtype), Rarity
2. Rules text
3. HS Inspiration (1 line)
4. Art connection (1 line)
5. Novelty statement
6. Fun pitch

Then ask the user which design they'd like to proceed with, or if they'd like revisions to
any of the designs.

# Phase 7: Iterate and Finalize

Once the user selects a design (or asks for revisions), iterate until the user is satisfied.
When the user confirms a final design, proceed to Phase 8.

Before finalizing, stress-test the chosen design:

- **Art connection check:** Can you write a one-sentence creative interpretation connecting
  the mechanic to the art? The connection can be loose or metaphorical — it just needs to
  not actively contradict what a viewer sees.
- **Power check:** Estimate the average case for variable effects. Use the costing guidelines
  to verify the total value is appropriate for the cost.
- **Play pattern check:** For every triggered ability, answer concretely: who causes this
  to fire, how often, and can they choose not to? If the trigger depends on the opponent
  taking an action they can simply avoid, the ability is effectively blank text. Mentally
  play the turn you cast this card and the next 2-3 turns: what concretely happens each
  turn?
- **Is there a simpler version?** If you have two mechanics stapled together, ask whether
  the card would be better with just one at a lower cost.
- **Duplicate check:** Scan the anonymized pool for the concept's key mechanical phrases.
  If an existing card already creates the same play experience, flag this to the user.

# Phase 8: Write to rendered-cards.toml and Commit

Once the user confirms the final design, write it to `rendered-cards.toml` and create a
commit.

### Step 1: Determine card-number and image-number

- **card-number:** Find the highest existing `card-number` in
  `client/Assets/StreamingAssets/Tabula/rendered-cards.toml` and increment by 1.
- **image-number:** The user must have provided an image. Extract the Shutterstock image
  number from the filename (the numeric portion of the image filename). If uncertain, ask
  the user.

### Step 2: Generate a UUID

Run `uuidgen | tr '[:upper:]' '[:lower:]'` to generate a new UUID for the card's `id` field.

### Step 3: Write the card entry

Insert a new `[[cards]]` entry into `rendered-cards.toml` **immediately before** the
`[metadata]` line. The entry must use this exact field order:

```toml
[[cards]]
name = "Card Name"
id = "generated-uuid-here"
tide = "Tide"
tide-cost = 1
rendered-text = "Rules text here."
energy-cost = 3
card-type = "Character"
subtype = "Warrior"
rarity = "Rare"
is-fast = false
spark = 2
art-owned = false
card-number = 666
image-number = 1234567890
```

Field notes:
- `rendered-text`: Use the **rendered** rules text with symbols (●, ▸, ↯, ✪), not the
  directive syntax from cards.toml. For multiline text, use TOML triple-quoted strings.
- `spark`: Use an integer for characters. Use `""` (empty string) for events.
- `art-owned`: Set to `false` unless the user specifies otherwise.
- `subtype`: Use `""` (empty string) for events.
- `is-fast`: `true` if the card itself has fast speed; `false` otherwise (even if it has
  a fast activated ability — that's encoded in the rules text via `↯fast`).

### Step 4: Create a commit

Use the `/commit` skill (or equivalent) to commit the change with a message like:
`Add [Card Name] to rendered-cards.toml`

Include in the commit description: the card's tide, type, cost, and a brief note on the
mechanic.
