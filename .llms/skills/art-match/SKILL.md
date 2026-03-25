---
name: art-match
description: Match card art to existing rules text from the anonymized card pool. Use when pairing art to an existing card design, selecting rules text for an image, or matching art to mechanics. Triggers on art match, match art, pick rules text, pair art, select card for art.
---

# Art-to-Rules-Text Matching Skill

You are an expert card game designer matching art to existing card rules text. Your goal is
to find the strongest thematic and narrative fit between a piece of art and an existing
unassigned card from the pool. Run everything with ultrathink.

Read `docs/battle_rules/battle_rules.md` and `docs/tides/tides.md` (use the Read tool).

# Phase 0: Load the Art

The user will provide an image ID number. Use the lookup script to find the image path and
description:

```bash
python3 .llms/skills/art-match/art-lookup.py <image_id>
```

This prints the local file path and the art description. Use the Read tool on the file path
to view the image, and use the description as additional context for your analysis. If the
image cannot be read for any reason (permission denied, file not found, etc.), STOP
immediately and report the error — do not proceed with the skill using only the text
description.

# Phase 1: Classify Art

The first step is to classify the card art. There are 3 possible types:

1) **Character.** Does the art show a single figure of a person, creature, animal, monster,
   etc. as the primary subject? Then this is a character card.
2) **Landscape.** Does the art depict a landscape? A wide depiction of a wilderness or urban
   landscape is a dreamwell/landscape card. **If the art is a landscape, state this clearly
   and EXIT. We do not handle dreamwell cards in this process.**
3) **Abstract.** Is the art an abstract texture, pattern, or color field with no discernible
   subject, scene, or action? Examples: paint textures, color gradients, geometric patterns,
   marble/stone surfaces, bokeh/light effects with no scene context. Abstract art is not
   suitable for use as card art. **If the art is abstract, state this clearly and EXIT. We do
   not handle abstract art in this process.**
4) **Event.** Art which does not depict a character, landscape, or abstract texture is by
   default an event card.

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
- **Two figures in conflict** (a warrior fighting a monster, two dueling figures) are events
  — the confrontation is the subject, not either participant. But a secondary figure that
  provides *scale* or *context* for a dominant creature (e.g., a tiny diver near a massive
  manta ray, a traveler gazing up at a colossus) does not make it an event; classify based
  on the dominant figure as a character.

**If the art is a landscape:** Print "This art depicts a landscape and would be a dreamwell
card. Dreamwell cards are not handled by this process." Then STOP. Do not continue.

### Mechanical Subtype Classification (Characters Only)

If you classified the art as a **character**, immediately determine whether the art depicts
one of the three **mechanically relevant** subtypes. These subtypes are hardcoded into card
rules text (e.g. "allied Warriors gain +1 spark", "for each allied Spirit Animal") and
cannot be changed — a Spirit Animal card must show an animal, a Warrior card must show a
fighter, etc.

| Subtype | The art clearly shows... |
|---|---|
| **Spirit Animal** | A natural animal (wolf, bear, eagle, fox, deer, etc.) or animal-like magical creature. The subject is unambiguously an animal, not a humanoid. |
| **Warrior** | A fighter, soldier, magic-wielder, or combat-ready figure — wielding a weapon or magic, wearing armor, in a martial pose, or in the midst of combat. Fantasy warriors, mages, and modern soldiers all count. |
| **Survivor** | A post-apocalyptic figure — gas masks, hazmat gear, improvised armor, ruined environments, radiation symbols, wasteland scavenger aesthetics. |

**Classification rules:**
- If the art **clearly and unambiguously** matches one of the three types above, lock the
  subtype. In Phase 3, you will filter the pool to only cards of that subtype.
- If the art is ambiguous (e.g., a figure with a weapon in ruins — Warrior or Survivor?),
  note both possibilities but do NOT lock. You'll consider cards from both subtypes plus
  the general "Char" pool.
- If the art does not match any of these three types (e.g., a child, a robot, a person at
  a computer), the subtype is **open**. In Phase 3, you will search the "Char" pool (cards
  with no mechanical subtype dependency). The subtype will be assigned later in Phase 4.

Record your classification:
- **Locked subtype:** "This art depicts a [Spirit Animal/Warrior/Survivor]."
- **Ambiguous:** "This art could be a [Type A] or [Type B]. Will search both."
- **Open:** "This art does not match a mechanical subtype. Will search Char pool."

# Phase 2: Read the Art's Story

Before thinking about mechanics, produce a detailed visual breakdown of the art, then
distill it into a narrative.

### Detailed Art Breakdown

Write a structured description of the art covering each of these elements (skip any that
don't apply):

- **The Subject:** What is the central figure, creature, object, or action? Describe their
  posture, clothing, equipment, expression, and any distinguishing features. Note their
  apparent size relative to the scene. Do not infer age from scale alone — a figure that
  appears small may simply be dwarfed by a massive environment; default to adult unless
  childlike proportions (large head-to-body ratio, short limbs) are clearly visible.
- **The Setting/Terrain:** What environment surrounds the subject? Describe the ground,
  structures, vegetation, or lack thereof. Is it natural, urban, alien, ruined?
- **The Sky/Background:** What's behind or above the scene? Weather, celestial bodies,
  distant structures, atmospheric effects?
- **Color Palette:** What are the dominant colors? Warm or cool? High contrast or muted?
- **Atmosphere & Mood:** What emotional tone does the art convey? Lonely, triumphant,
  ominous, serene, chaotic? What about the lighting — dawn, dusk, harsh noon, ethereal glow?
- **Art Style:** Painterly, photorealistic, impressionistic, stylized? Soft edges or sharp
  detail?

**Grounding rule — describe only what you see.** Do not infer weather, particles, or
environmental effects that are not clearly depicted. Ambiguous visual elements (floating
specks, haze, atmospheric effects) must be described neutrally — "floating particles" not
"snow," "atmospheric haze" not "fog," "light specks" not "fireflies." If you are uncertain
what something is, say so explicitly: "particles that could be dust, pollen, or ash." Never
commit to a specific interpretation of an ambiguous element. This is critical because
downstream phases (naming, narrative) will build on your description, and a wrong label here
(e.g., calling dust "snow") will propagate through the entire output.

**Example breakdown:** "The painting depicts a sci-fi scene on an alien, Mars-like planet
with a warm red-orange color palette. At the center-right stands a large robotic walker — a
mechanized vehicle with insect-like legs and a glowing yellowish-orange visor suggesting a
pilot inside. The mech is perched atop a rocky ridge. The landscape is dominated by rugged,
eroded rock formations — tall mesa-like pillars in deep reds and burnt sienna. A dusky
blue-purple sky contrasts with the warm terrain, and a massive planet looms in the upper-left.
The overall mood is solitary exploration — a lone pilot traversing a vast, desolate alien
landscape."

### Literal Reading and Narrative Anchor

**Literal reading:** One sentence about what is physically happening in the art. "A figure
is striking a bear with a pickaxe." "A woman floats above a glowing pool." This grounds
you in what a viewer actually *sees*.

**Narrative anchor:** 2-3 sentences describing what you see in pure story terms — no game
vocabulary. What emotion does it evoke? What's happening? What kind of person/creature/moment
is this? The narrative anchor is a creative prompt that guides your matching.

**Example narrative anchors:**
- "A lone traveler stands in an endless golden field, watching in quiet awe as impossibly
  vast runestones descend from the heavens. It's not destruction — it's revelation, ancient
  knowledge returning to the earth."
- "A feral wolf-spirit erupts from a thicket of thorns, trailing wisps of violet light.
  It's hunting, but what it hunts is not prey — it's something lost, something it needs
  to reclaim."
- "Two figures stand back-to-back on a crumbling bridge over a void. They're not fighting
  each other — they're the last defense against something unseen below."

Then identify the **practical constraints** the art places on matching:

- Art showing a larger-than-human character must correspond to an expensive/high-spark
  character card
- Art with a positive/uplifting mood should match a positive-coded game effect for the player
  such as drawing cards
- Art with a horror/destructive mood should match a negative-coded effect such as interacting
  with the opponent or the void
- Art showing a small creature should match a cheap/low-cost character
- Art showing a massive, dramatic event should match a high-cost event
- Art showing a subtle, quiet moment should match a low-cost or utility event

### Art-to-Dynamic Translation Guide

The table below maps visual elements to the **dynamics and feelings** they suggest. Use this
to identify which rules text entries resonate with the art's mood and story.

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

**Size-to-cost mapping for characters:**

| Character Scale | Typical Cost | Typical Spark |
|---|---|---|
| Small creature (cat, imp, child) | 1-2 | 0-1 |
| Human-sized figure | 2-4 | 1-3 |
| Large creature/imposing figure | 4-6 | 3-5 |
| Titanic/mythic being | 7-9 | 5-7 |

These ranges are guidelines for the art's visual weight, not minimums — a card whose power comes from its entrance effect (e.g., a sweeper) rather than its stats can have spark well below the range and still be a strong match for imposing art.

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

# Phase 3: Search and Match

This phase is iterative. You will search the card pool, evaluate candidates, and refine
your search until you find a strong match — or determine that no strong match exists.

## The Pool Filter Tool

Use the pool filter script to load candidate cards. The script reads from
`cards_anonymized.txt` (pre-generated anonymized card data). Run it from the
repo root:

```bash
python3 .llms/skills/art-match/pool-filter.py <command> [filters...]
```

**Commands:**
- `characters` — all anonymized characters
- `events` — all anonymized events

**Filters:**
- `--tide TIDE` — filter to one tide (e.g. `--tide Bloom`)
- `--subtype SUBTYPE` — filter to a character subtype: `Warrior`, `"Spirit Animal"`,
  `Survivor`, or `Char` (cards with no mechanical subtype)

Each output line has the format:
```
TideCost | Cost/Spark | Type | Rarity | Rules Text
```
For example: `Bloom1 | 3●/2✦ | Spirit Animal | R | ▸ Dissolved: Put this card on top of your deck with +2 spark.`

## Round 1: Initial Search

Run your first queries based on the classification from Phase 1 and your narrative from
Phase 2. **Browse broadly** — read through full card lists by type and tide, letting the
rules text surprise you. Do NOT pre-filter by cost, mechanic, or rarity. The best matches
often come from unexpected cards you wouldn't have searched for.

**For characters**, start with the subtype classification:

```bash
# Locked subtype — ONLY search that subtype:
python3 .llms/skills/art-match/pool-filter.py characters --subtype Warrior

# Ambiguous — search both possible subtypes:
python3 .llms/skills/art-match/pool-filter.py characters --subtype Warrior
python3 .llms/skills/art-match/pool-filter.py characters --subtype Survivor

# Open subtype — search Char (no mechanical subtype):
python3 .llms/skills/art-match/pool-filter.py characters --subtype Char

# Event art:
python3 .llms/skills/art-match/pool-filter.py events
```

Then browse tides that fit the art's mood and color:

```bash
python3 .llms/skills/art-match/pool-filter.py characters --tide Umbra
python3 .llms/skills/art-match/pool-filter.py characters --tide Pact
```

## Evaluate and Iterate

Scan the results from Round 1. For each card that catches your eye, ask the **"aha" test:**

> If a player saw this art on this card, would they say "of course this character/event
> does that"? Does the mechanic complete the story the art is telling?

**If you spot a strong candidate:** Develop it. Write 2-3 sentences explaining the
art-mechanic connection. Check that:
- The cost/spark matches the visual weight of the subject
- The mood of the mechanic matches the mood of the art
- The tide philosophy fits the art's worldview
- You can explain "this character does X because..." without straining

**If nothing in Round 1 is compelling:** Widen your search. Try different tides — maybe
the mood maps to a tide you didn't initially consider.

**If you find multiple strong candidates:** Good — hold them and compare. You don't need
to force a quota, but having 2-3 genuine contenders makes for a more confident final pick.

Continue iterating until you either:
1. **Find a match that passes the "aha" test** — the art and mechanic feel inseparable
2. **Exhaust the reasonable search space** — you've tried multiple tides and nothing fits
   well. If this happens, say so honestly. Not every piece of art will have a perfect
   match in a finite pool.

## What Makes a Strong Match

The best matches create an "aha" moment where the art and the mechanic feel inseparable.

- **The mechanic IS the character's story.** A card that reads "▸ Dissolved: Return each
  ally to hand" paired with art of a guardian figure creates the narrative: "When the
  guardian falls, they use their last strength to save everyone else." The mechanic doesn't
  just coexist with the art — it completes the story.
- **The cost/spark matches the visual weight.** A massive, imposing creature should feel
  expensive and impactful. A small, nimble figure should feel cheap and tricky.
- **The tide philosophy matches the art's worldview.** A nature scene with growth imagery
  naturally fits Bloom. A figure surrounded by spectral ruins fits Umbra. Dark sacrifice
  imagery fits Pact. Bright, energetic imagery fits Arc or Surge.

**Reject matches where:**
- The mechanic actively contradicts the art (a peaceful scene matched with mass destruction)
- The cost/scale is wildly mismatched (a tiny creature on a 7-cost card)
- The narrative requires too many logical leaps to explain
- A card has a secondary mechanic (e.g. abandon, sacrifice, discard) that contradicts the art's mood even if the primary mechanic fits — every part of the rules text must be justifiable from the art, not just the headline ability
- The effect requires you to invent a hypothetical scenario ("when the beast breaks free…", "when the guardian falls…") rather than being visually suggested by what the art actually depicts — if the art shows a predator standing in a lab, the mechanic should feel like what a predator in a lab *does*, not what might happen in a future scene you imagined

## Tide Color Bias

Each tide has an associated color palette. This is a **tiebreaker only** — when choosing
between otherwise-equal candidates, prefer the tide whose color matches the art:

| Tide | Color |
|---|---|
| Bloom | Green |
| Ignite | Red |
| Arc | Yellow |
| Pact | Orange/Brown |
| Umbra | Purple |
| Rime | Blue |
| Surge | Gray |

A strong narrative match in the "wrong" color always beats a weak match in the "right" color.

## Select the Winner

**Guard against primacy bias.** LLMs tend to favor the first candidate they evaluate in
detail — the one that gets labeled "Candidate A." Before selecting a winner, you MUST:
1. Identify your top 2-3 candidates.
2. For each candidate *other than* the first one you noticed, write a **steel-man argument**
   (2-3 sentences) for why it could be the best match.
3. For the first candidate you noticed, write a **devil's advocate argument** (2-3 sentences)
   for why it might NOT be the best match — what's weak about it?
4. Only after completing steps 2-3, make your final selection.

It is completely fine to still pick the first candidate after this exercise — but you must
genuinely consider whether a later candidate is stronger.

Once you've selected the winner, briefly explain:
- What is the narrative connection — the "story" this card tells?
- Why does this match beat any alternatives you considered?
- Any concerns or compromises?

# Phase 4: Assign Subtype and Name

### Subtype Assignment (Characters Only)

If the selected card already has a specific subtype (e.g., "Spirit Animal", "Warrior"),
keep it — these are mechanical dependencies that cannot be changed.

If the card has "Char" as its type (meaning no assigned subtype), assign one based on the
art. Use the following type descriptions:

| Subtype | Description |
|---|---|
| Warrior | A fantasy character wielding a sword or magic, or modern soldier |
| Spirit Animal | Any kind of natural/non-mythical animal |
| Survivor | A post-apocalyptic character, e.g. wearing a gasmask or living in ruins |
| Ancient | A massive monster or humanoid |
| Child | A child character |
| Explorer | A human in some new environment |
| Tinkerer | A human interacting with technology |
| Monster | A non-human horrifying creature at normal human scale (not an Ancient) |
| Synth | A robot character type |
| Musician | A human playing an instrument |
| Visitor | Fallback character type when other types don't fit |
| Outsider | A human figure with surreal/horror distortions |
| Renegade | A rebel, outlaw, or defiant figure |
| Detective | An investigator or analyst figure |

**Type redirects — do NOT use these, use the redirect instead:**
- Visionary → use Warrior or Explorer
- Super → use Warrior
- Guide → use Explorer
- Hacker → use Tinkerer
- Trooper → use Warrior
- Robot → use Synth
- Mage → use Warrior

It is acceptable to create a **new subtype** if the art clearly depicts something that
doesn't fit any existing type. Do this carefully and only when the art strongly demands it.

### Card Naming

The card needs a new, evocative name. Think through at least 3 name candidates before
picking one. For each candidate, explain how the name connects back to the art.

**Hard limit:** Card names must be 25 characters or fewer.

**No proper names.** Dreamtides preserves a dreamlike, anonymous feel — card names must
never include specific character names (e.g., "Voss", "Kael", "Aria"). Use titles, roles,
descriptors, or abstract phrases instead. The characters are unnamed figures in a dream.

Name guidelines:
- Character cards must be named as a person/creature (a role, title, or descriptor), never as an object, weapon, body part, or action they happen to be performing. Event-style names that describe a phenomenon, force, or occurrence (e.g., "Emerald Pyre", "Meltdown", "Cataclysm") are wrong for characters — the name must identify *who* the figure is, not *what is happening around them*.
- Prefer poetic, evocative names over literal descriptions of what the art depicts — "The Undaunted" is better than "The Sunleaper," and names should suggest mood and identity rather than narrating the visual scene.
- The name should evoke the **primary subject** of the art (the figure, creature, or action), not minor background elements like ambient lighting, secondary environmental effects, or background details that a viewer would not identify as the focal point
- It should feel natural alongside the card's rules text
- It should hint at the narrative you've constructed
- **Do not reference weather, particles, or environmental details in the name unless they
  are unambiguously depicted in the art.** If the art breakdown described something as
  ambiguous (e.g., "floating particles"), do not name the card after a specific
  interpretation of that element (e.g., "Snowfall," "Ashrain"). Ground the name in the
  subject, setting, and mood — not in uncertain details.

Common name structures (use variety — don't default to the first few):
- **[Adjective] [Noun]:** Silent Avenger, Eternal Sentry
- **[Compound] [Noun]:** Bloomweaver, Starcatcher
- **[Place] [Role]:** Neon Street Wanderer
- **[Noun] of [Noun]:** Titan of Forgotten Echoes, Blade of Oblivion
- **Single word:** Apocalypse, Reunion, Nocturne
- **The [Title]:** The Devourer, The Rising God
- **[Verb]-er / [Verb]-ing [Noun]:** Flickering Lantern, Dreamstalker
- **[Possessive] [Noun]:** Siren's Lament, Nobody's Herald
- **[Noun] [Verb]:** Starfall, Duskwalk, Nightbloom
- **[Role] [Participle]:** Sentinel Unbound, Pilgrim Returned
- **Abstract / Poetic:** Hollow Meridian, Pale Crescendo, Rust and Reverie

# Phase 5: Final Output

Present the final card assignment:

- **Card Name:** [name, max 25 characters]
- **Rules Text:** [exact rules text from the pool entry]
- **Card Type:** Character or Event
- **Subtype:** [subtype, for characters only]
- **Tide:** [tide and tide cost from the pool entry]
- **Energy Cost:** [from the pool entry]
- **Spark:** [from the pool entry, characters only]
- **Rarity:** [from the pool entry]
- **Narrative:** A 2-3 sentence description explaining the art-mechanic connection:
  - For characters: Who is this character? What is their story? Why do they have this
    ability?
  - For events: What is happening in this event? How did it cause these game effects?
