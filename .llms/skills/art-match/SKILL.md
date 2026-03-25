---
name: art-match
description: Match card art to existing rules text from the anonymized card pool. Use when pairing art to an existing card design, selecting rules text for an image, or matching art to mechanics. Triggers on art match, match art, pick rules text, pair art, select card for art.
---

# Art-to-Rules-Text Matching Skill

You are an expert card game designer matching art to existing card rules text. Your goal is
to find the strongest thematic and narrative fit between a piece of art and an existing
unassigned card from the pool. Run everything with ultrathink.

Read `docs/battle_rules/battle_rules.md` and `docs/tides/tides.md` (use the Read tool).

# Phase 1: Classify Art

The first step is to classify the card art. There are 3 possible types:

1) **Character.** Does the art show a single figure of a person, creature, animal, monster,
   etc. as the primary subject? Then this is a character card.
2) **Landscape.** Does the art depict a landscape? A wide depiction of a wilderness or urban
   landscape is a dreamwell/landscape card. **If the art is a landscape, state this clearly
   and EXIT. We do not handle dreamwell cards in this process.**
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
  with no mechanical subtype dependency). The subtype will be assigned later in Phase 6.

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
  apparent size relative to the scene.
- **The Setting/Terrain:** What environment surrounds the subject? Describe the ground,
  structures, vegetation, or lack thereof. Is it natural, urban, alien, ruined?
- **The Sky/Background:** What's behind or above the scene? Weather, celestial bodies,
  distant structures, atmospheric effects?
- **Color Palette:** What are the dominant colors? Warm or cool? High contrast or muted?
- **Atmosphere & Mood:** What emotional tone does the art convey? Lonely, triumphant,
  ominous, serene, chaotic? What about the lighting — dawn, dusk, harsh noon, ethereal glow?
- **Art Style:** Painterly, photorealistic, impressionistic, stylized? Soft edges or sharp
  detail?

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

# Phase 3: Load the Card Pool

Use the pool filter script to load candidate cards. The script reads from
`rendered-cards.toml` and outputs anonymized entries (names stripped). Run it from the
repo root:

```bash
python3 .llms/skills/art-match/pool-filter.py <command> [filters...]
```

**Commands:**
- `characters` — all anonymized characters
- `events` — all anonymized events
- `unassigned-characters` — characters without art assigned (preferred)
- `unassigned-events` — events without art assigned (preferred)

**Filters (combinable):**
- `--cost LOW-HIGH` — energy cost range (e.g. `--cost 2-4` or `--cost 3`)
- `--spark LOW-HIGH` — spark range, characters only (e.g. `--spark 1-3`)
- `--tide TIDE` — filter to one tide (e.g. `--tide Bloom`)
- `--rarity RARITY` — filter to a rarity (e.g. `--rarity Rare`)
- `--mechanic KEYWORD` — filter to cards whose rules text contains keyword
- `--subtype SUBTYPE` — filter to a character subtype: `Warrior`, `"Spirit Animal"`,
  `Survivor`, or `Char` (cards with no mechanical subtype)

Each output line has the format:
```
TideCost | Cost/Spark | Type | Rarity | Rules Text
```
For example: `Bloom1 | 3●/2✦ | Spirit Animal | R | ▸ Dissolved: Put this card on top of your deck with +2 spark.`

**Step 1: Load based on classification.** Use the subtype classification from Phase 1 to
determine which pool to search:

```bash
# Locked subtype — ONLY search that subtype:
python3 .llms/skills/art-match/pool-filter.py unassigned-characters --subtype Warrior
python3 .llms/skills/art-match/pool-filter.py unassigned-characters --subtype "Spirit Animal"
python3 .llms/skills/art-match/pool-filter.py unassigned-characters --subtype Survivor

# Ambiguous — search both possible subtypes:
python3 .llms/skills/art-match/pool-filter.py unassigned-characters --subtype Warrior
python3 .llms/skills/art-match/pool-filter.py unassigned-characters --subtype Survivor

# Open subtype — search Char (no mechanical subtype):
python3 .llms/skills/art-match/pool-filter.py unassigned-characters --subtype Char

# Event art:
python3 .llms/skills/art-match/pool-filter.py unassigned-events
```

**Step 2: Narrow with filters.** Use the practical constraints from Phase 2 to run
focused queries. Run multiple filtered queries in parallel to explore efficiently:

```bash
# Example: small creature → cheap characters
python3 .llms/skills/art-match/pool-filter.py unassigned-characters --cost 1-2

# Example: dark/void mood → Umbra or Pact characters
python3 .llms/skills/art-match/pool-filter.py unassigned-characters --tide Umbra
python3 .llms/skills/art-match/pool-filter.py unassigned-characters --tide Pact

# Example: dramatic event → high-cost events
python3 .llms/skills/art-match/pool-filter.py unassigned-events --cost 4-7

# Example: events with specific mechanic
python3 .llms/skills/art-match/pool-filter.py unassigned-events --mechanic "dissolve"
```

Review the filtered results and identify your working set of candidates. If the unassigned
pool is too small or has no good matches, fall back to the full pool (without `unassigned-`
prefix) — assigned cards can be reassigned if the match is compelling enough.

# Phase 4: Generate Matching Concepts

Generate **at least 5 candidate matches** between the art and cards from the pool. Each
concept should:

1. **Identify the card** from the pool (cite the full line)
2. **Write a narrative anchor** explaining the connection: Why does this rules text fit this
   art? How does the mechanic tell the story of what's happening in the image?
   - For characters: Who is this character? What is their story? Why do they have this
     ability? How does the ability reflect what we see?
   - For events: What is happening in this event? How did it cause these game effects? Why
     does the mechanical effect match the visual moment?
3. **Rate the fit** on three dimensions:
   - **Mood alignment** (1-5): Does the emotional tone of the art match the feel of the
     mechanic? Uplifting art with draw/gain effects = high. Horror art with dissolve = high.
     Mismatch = low.
   - **Scale alignment** (1-5): Does the visual "size" of the subject match the card's cost
     and impact? A towering figure matching a 6-cost powerhouse = high. A small bird matching
     a 7-cost bomb = low.
   - **Narrative coherence** (1-5): How naturally does the mechanic tell a story about the
     art? Can you explain "this character does X because..." without straining? A protector
     figure matching a prevent effect = high. A serene meditator matching an aggressive
     dissolve = low.

### What Makes a Strong Match

The best matches create an "aha" moment where the art and the mechanic feel inseparable —
where a player seeing the card would say "of course this character does that." Aim for
matches where:

- **The mechanic IS the character's story.** A card that reads "▸ Dissolved: Return each
  ally to hand" paired with art of a guardian figure creates the narrative: "When the
  guardian falls, they use their last strength to save everyone else." The mechanic doesn't
  just coexist with the art — it completes the story.

- **The cost/spark matches the visual weight.** A massive, imposing creature should feel
  expensive and impactful. A small, nimble figure should feel cheap and tricky. A dramatic
  catastrophe should be a high-cost event.

- **The tide philosophy matches the art's worldview.** A nature scene with growth imagery
  naturally fits Bloom's patient accumulation. A figure surrounded by spectral ruins fits
  Umbra's relationship with the void. Dark sacrifice imagery fits Pact. Bright, fast,
  energetic imagery fits Arc or Surge.

### What Makes a Weak Match

Avoid matches where:

- The mechanic actively contradicts the art (a peaceful scene matched with mass destruction)
- The cost/scale is wildly mismatched (a tiny creature on a 7-cost card)
- The narrative requires too many logical leaps to explain
- The mood of the mechanic clashes with the mood of the art

### Priority Hierarchy

1. **Narrative coherence** is king — the art-mechanic connection should feel natural and
   tell a compelling story
2. **Scale alignment** is critical — the visual weight must match the mechanical weight
3. **Mood alignment** supports the whole — the emotional tone should be consistent
4. **Tide philosophy** is a bonus — a natural tide fit enhances the match

# Phase 5: Select the Winner

Review your 5+ concepts and select the strongest match. Explain your reasoning:

- Why does this match beat the alternatives?
- What is the narrative connection — the "story" this card tells?
- Are there any concerns or compromises in this match?

If no match feels genuinely strong (all concepts score below 3 on narrative coherence),
say so honestly. Not every piece of art will have a perfect match in a finite pool.

# Phase 6: Assign Subtype and Name

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
- The name should evoke what a viewer sees in the art
- It should feel natural alongside the card's rules text
- It should hint at the narrative you've constructed

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

# Phase 7: Final Output

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
