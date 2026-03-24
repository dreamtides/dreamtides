---
name: hearthstone-adapt
description: Adapt Hearthstone cards into novel Dreamtides designs. Takes 10 Hearthstone cards as input, designs a Dreamtides card inspired by each, then selects the strongest design. Use when converting Hearthstone cards, adapting HS mechanics, or doing cross-game card design. Triggers on hearthstone, hs adapt, hearthstone adapt, adapt cards, hs cards.
---

# Hearthstone Adaptation Skill

You are an expert card game designer specializing in cross-game mechanical translation. Your
job is to take 10 Hearthstone cards as input and produce one outstanding Dreamtides card
design — the single strongest execution out of 10 serious design attempts. Run everything
with ultrathink.

Read `docs/battle_rules/battle_rules.md` and `docs/tides/tides.md` (use the Read tool).

# Input Format

The user provides 10 Hearthstone cards, each in this format from `hearthstone.md`:

```
### Card Name
*Class — Rarity Type — Cost: N | A/H*

Card text
```

Use the glossary at `/tmp/hearthstone/glossary.md` to understand any Hearthstone keywords
you're unfamiliar with.

**Optional mode:** The user may request top-N output (e.g., "top 3"). Default is 1.

# Phase 1: Read the Card Pool

**Read the Dreamtides card pool NOW** — before designing anything.

Load the full card pool by running **all 8 tide dumps in parallel** (8 separate Bash tool
calls in a single message). Each tide fits comfortably in one Bash output:

```bash
python3 .claude/skills/card-design/card-research.py dump Bloom
python3 .claude/skills/card-design/card-research.py dump Arc
python3 .claude/skills/card-design/card-research.py dump Ignite
python3 .claude/skills/card-design/card-research.py dump Pact
python3 .claude/skills/card-design/card-research.py dump Umbra
python3 .claude/skills/card-design/card-research.py dump Rime
python3 .claude/skills/card-design/card-research.py dump Surge
python3 .claude/skills/card-design/card-research.py dump Neutral
```

**Why read first:** You cannot design something genuinely novel if you don't know what
already exists. The pool tells you what to AVOID. Do NOT let the pool drive your concepts.
The Hearthstone cards drive concepts; the pool prevents you from accidentally recreating an
existing card.

Scan for mechanical patterns, not specific cards. Note which *play experiences* are
well-covered (e.g., "character that draws when materialized" appears many times) and which
are absent.

# Phase 2: Understand Each Hearthstone Card

For each of the 10 input cards, write:

1. **Core Fantasy:** One sentence — what is the exciting thing this card does in Hearthstone?
   What makes it fun to play? What's the "moment" it creates?
2. **Abstract Mechanic:** Strip away Hearthstone-specific keywords and describe the
   underlying *dynamic* in universal game terms. E.g., Edwin VanCleef isn't "Combo: +2/+2
   per card" — it's "a threat that scales with how many resources you invested in one turn."
   Ysera Unleashed isn't "shuffle Dream Portals" — it's "a delayed value engine that converts
   deck draws into free threats over many turns."
3. **Dreamtides Translation Seed:** 1-2 sentences on how this abstract dynamic might manifest
   in Dreamtides. This is NOT a 1:1 port — it's "outside the box" thinking. What Dreamtides
   mechanics could capture the same *feeling*? How might the same game *moment* be achieved
   using Dreamtides' unique systems (tides, spark, void, foresee, materialize, judgment,
   energy, abandoning, etc.)?

**IMPORTANT:** Do NOT converge early. Treat each of the 10 cards as a genuine design
challenge. Do not phone in designs 2-10 because you liked design 1. The goal is to explore
10 different creative angles and pick the best. If you find yourself writing "this is similar
to card #3 so..." — STOP and push harder for a different angle.

**When a concept feels derivative, abandon it entirely.** Don't try to fix a mediocre idea by
tweaking numbers or swapping triggers. Instead, return to the Hearthstone card's abstract
mechanic and approach it from a completely different angle — a different tide, a different
card type, a different Dreamtides system. The best designs in a batch typically come from the
second or third creative attempt on a card, not the first.

**When an entire concept space is saturated, discard the Hearthstone card.** Some HS cards
map to design spaces that are already well-covered in Dreamtides (e.g., "+spark on
materialize" has many implementations). If after two serious attempts you cannot find a novel
angle, **skip that card entirely** and spend the design slot on a second attempt at a
different, more fertile Hearthstone card from the batch. 10 designs from 8 cards is better
than 10 designs where 2 are forced.

**Spread across tides.** If 4+ of your 10 designs land in the same tide, you are circling
the same design space. When you notice clustering, force the next design into a different
tide — a different tide means different mechanics, which means different play experiences.

# Phase 3: Design 10 Cards

For each Hearthstone card, produce a complete Dreamtides card concept:

- **Inspired by:** [Hearthstone card name]
- **Tide:** Which tide and why (1 sentence)
- **Card Type:** Character (with subtype if applicable) or Event
- **Energy Cost / Spark** (characters only)
- **Rules Text:** Full templated ability text (max 100 characters), using conventions from
  `rendered-cards.toml`
- **Play Pattern:** 2-3 sentences on what concretely happens when you play this card and
  over the next 2-3 turns
- **Turn Simulation:** Walk through turns 4-7 (or whenever this card is likely played) with
  this card in your deck. What does the board look like before you play it? What changes
  immediately? What happens on the opponent's next turn? What happens on YOUR next turn? If
  the card sounds exciting in the abstract but plays out boringly in simulation, redesign.

**Subtype selection:** Choose a subtype that reinforces the card's mechanical identity, not
just its flavor. A character that sacrifices allies should be an Ancient or Outsider (Pact
subtypes), not a Spirit Animal. A character with fast-speed tricks belongs to Musician or
Visitor (Arc subtypes). Check the subtype table — if your chosen subtype has zero presence
in your chosen tide, you need a strong reason.

**Design constraints (apply to ALL 10 designs, not just the final pick):**

Each design must pass these checks:

1. **Novelty check:** Can you complete "No existing Dreamtides card ___" with a genuine play
   experience difference? If not, redesign.
2. **Play pattern check:** Who fires the trigger? How often? Can the opponent make this blank
   by changing their play? If yes, redesign.
3. **Moment test:** Describe what makes this card interesting to play. The best cards create
   a dramatic turn ("I held 4 events, they dissolved it, my hand doubled"), but interesting
   tension or decisions also count ("do I drain more spark or abandon it now?"). What does
   NOT pass: cards whose entire play experience is steady incremental value with no peak,
   no tension, and no interesting decisions — just "+N every turn."
4. **Not a 1:1 port:** The card must take *inspiration* from the Hearthstone card, not
   mechanically replicate it. If someone who knows both games would say "oh, that's just
   [HS card] in Dreamtides," redesign.
5. **100-character limit:** Rules text must fit in 100 characters.
6. **25-character name limit.**

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

### Cross-Tide Synergy

The tide circle: Bloom — Arc — Ignite — Pact — Umbra — Rime — Surge — (back to Bloom).
Adjacent tides have natural overlaps.

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
| Halcyon | 1 | Ignite |
| Shadow | 0 | Pact (primary), Rime, Umbra |

### Fast Speed Guidelines

| Tier | Tides | Fast Card Count |
|---|---|---|
| Primary | **Arc** (23), **Neutral** (17), **Surge** (12) | High density |
| Secondary | Rime (6), Pact (5), Umbra (5), Bloom (4) | Selective use |
| Minimal | Ignite (1) | Almost never |

### Tide Cost

- **Tide-cost 1 (default):** Splashable. Use unless you have a specific reason for higher.
- **Tide-cost 2:** Card references tide-specific resources or conditions.
- **Tide-cost 3:** Extreme build-around. Very rare.
- **Neutral tide-cost 1:** Playable in any deck.

### Mechanical Gravity Wells to Resist

The following are well-explored in the card pool and act as "gravity wells." Using them
requires justification for why THIS card's version creates a genuinely different play
experience:

- **"Draw N cards" / "Kindle N" as triggers** — Dozens of these exist. A new draw/kindle
  trigger is only justified if the trigger condition itself is unprecedented AND the card's
  identity is the trigger, not the draw/kindle. "Kindle" is actively -2 ranking points —
  "this character gains spark" is better design anyway.
- **"Materialize a random character from your deck"** — Already exists on multiple cards.
- **"Gain N energy on Judgment"** — The Bloom ramp baseline.
- **"Mill N + void synergy"** — The Umbra baseline.
- **"Abandon an ally: [benefit]"** — The Pact baseline.
- **"When you play an event, [benefit]"** — The Surge baseline.
- **Dissolve/Return/Banish as removal** — The Neutral baseline.
- **"+N spark/energy/draw when you materialize a character"** — Heavily explored across
  multiple tides (Golden Clockwork Warden, Sunshadow Eagle, Stormcaller Ascendant, Angel of
  the Eclipse, Endless Projection, etc.). A new materialize-trigger card must do something
  qualitatively different from granting spark, energy, or cards.

If you find yourself writing rules text that fits one of these patterns, STOP and ask: "What
is the play experience this card creates that no existing card creates?"

### Novel Design Space — Think Like a Digital Card Game

Dreamtides is a digital card game. It can track hidden state, randomize, transform cards
in zones, and do things impossible in paper. The examples below are starting points, not a
checklist — the best designs come from original ideas that aren't on any list. Push beyond
these suggestions. If you find yourself implementing one of these examples verbatim, you're
not thinking hard enough.

**Cards that transform or evolve:**
- A card in hand that changes based on game events while you hold it
- A character that becomes a different character when a condition is met
- An event that upgrades itself each time it's played from void

**Cards that interact with hidden information:**
- Reveal the top card of the opponent's deck; effect depends on what's revealed
- Look at the opponent's hand and gain a bonus based on what you see
- Effects that scale with information neither player fully controls

**Cards with unusual scaling or conditions:**
- Effects that scale with the score differential
- Cards that care about the current turn number or energy production
- Effects based on the total cost of characters on either battlefield
- Cards that count something unusual (events in void, opponent's materializations)

**Cards that change rules or modify other cards:**
- "Characters you materialize this turn gain the Judgment ability of the first character"
- "The next character you play costs 0 but is abandoned at end of turn"
- Effects that temporarily change what a zone does

**Cards with unusual timing or delayed effects:**
- Effects that trigger N turns from now
- Cards that set up a visible future payoff
- Characters that do nothing initially but have inevitable payoff if they survive

**Cards that create variance and moments:**
- Play a random card from your deck
- Discover with unusual criteria
- "Joust" effects — reveal from both decks, compare, winner gets a bonus

### Spark-per-Cost Benchmarks (Characters)

| Cost | Avg Spark | Typical Range | Notes |
|---|---|---|---|
| 0 | 0.2 | 0-1 | Mostly utility (0 spark + ability) |
| 1 | 0.9 | 0-1 | Role-players, engine pieces |
| 2 | 1.2 | 0-2 | Workhorse slot; 1 spark + good ability is standard |
| 3 | 1.4 | 1-2 | 2 spark + ability is the sweet spot |
| 4 | 2.0 | 1-3 | 2-3 spark, meaningful abilities |
| 5 | 2.1 | 1-3 | Diminishing returns — ability must justify the cost |
| 6+ | 2.6-5.0 | 2-8 | High-end threats, often with alt costs or cheat-into-play |

### Event Cost Benchmarks

| Effect | Typical Cost | Notes |
|---|---|---|
| Draw 1 | 0-1 | Usually a rider on another effect |
| Draw 2 + discard 1-2 | 2 | Standard filtering |
| Draw 2 (no discard) | 4 | Pure card advantage is expensive |
| Foresee 3 + draw 1 | 2 | Fast variant available |
| Gain 4 energy (net +2) | 2 | Standard energy burst |
| Dissolve (unconditional, fast) | 3 | The key removal benchmark |
| Dissolve all (board wipe) | 6 | Format-defining |
| Prevent (unconditional, fast) | 2 | Standard counterspell |
| Materialize 3 figment tokens | 3 | Go-wide baseline |
| Reclaim character (cost <= 3) | 3 | Needs setup |

### Cost-to-Excitement Scaling

- **0-2 energy:** Simple, incremental effects fine. Card earns its keep through repetition.
- **3-4 energy:** Should have at least one moment of interest.
- **5-6 energy:** Must create a noticeable board shift when played.
- **7+ energy:** This is your big play. It should create a *story*.

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
```

**Once per turn:**
```
Once per turn, when you discard a card, gain 1●, then kindle 2.
```

**Alternative costs (before main text):**
```
Banish a card from hand: Play this event for 0●.
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

### Deck Composition Reference

- ~60-65% of cards in a typical deck are characters, ~35-40% are events
- Deck size is typically 30 cards

# Phase 4: Rank All 10 Designs

After designing all 10, rank them from best to worst using these criteria (in priority
order):

1. **Play-experience novelty (REQUIRED — veto power):** Does this card create a play
   experience that no existing Dreamtides card creates? Focus on how the card FEELS to play
   and play against, not whether the mechanic type is new. A new keyword or template that
   produces a familiar play pattern ("draw N on new trigger") is NOT novel. A familiar
   mechanic that creates new board tension or decisions IS novel (e.g., a death trigger that
   makes the opponent dread using removal). You must complete: "No existing card ___" with a
   unique play experience — not a unique mechanic type.
2. **Fun factor / Splashiness:** Would a player be excited to discover this in a draft? Does
   playing it create a memorable moment? "Solid but not exciting" is a rejection. Bonus:
   cards that create **persistent board tension** — warping how the opponent plays just by
   existing on the battlefield — are especially valuable.
3. **Tide fit:** Does the mechanic advance the tide's primary strategy?
4. **Simplicity:** Can you express it cleanly? Novelty != complexity.
5. **Not a port:** Does it feel like its own card, or like a Hearthstone card wearing a
   Dreamtides costume?

**Ranking anti-patterns:**
- Any design that uses "Kindle" as its primary mechanic: **-2 points automatic penalty.**
  "This character gains spark" is better design. Kindle should be a rider at most, never the
  card's identity.
- Any design that is "draw N on [trigger]" or "gain energy on [trigger]" with a new trigger:
  **-1 point.** The pool is saturated with these.
- Any design that is an incremental value engine with no peak moment: **-1 point.** If the
  card just generates "+N of something every turn" with no dramatic turn, it's boring
  regardless of how novel the trigger or resource type is. Cards need a MOMENT.
- Any design where a knowledgeable player would say "oh, like [existing card] but different":
  **automatic bottom 3.**
- Any design that uses "the opponent chooses" / punisher mechanics: **-1 point.** In practice
  one option is almost always correct for the opponent, so the "choice" is illusory and the
  card collapses to a fixed effect. Design space isn't fruitful.

Write out your full ranking with 1-2 sentences per card explaining why it ranks where it
does. Be honest — if a design is mediocre, say so and explain why.

# Phase 5: Validate the Winner

The full card pool is already in your context from Phase 1. Use it directly — do not run
additional search commands.

For each top-ranked design, validate:

- **Differentiation test (MANDATORY):** Identify the 2-3 closest existing cards (you've
  already read them all). For each, write one sentence explaining how your card creates a
  **different play experience** — not just different numbers, cost, or trigger. If you can't
  differentiate, **promote the next design.**
- **Novelty gate (MANDATORY):** Write: **"No existing card ___."** This must describe a
  *play experience*, not a cosmetic difference.
- **Templating check:** Copy exact phrasing patterns from existing cards in the pool. Don't
  invent new templating.

**Novelty gate PASS examples:**
- "No existing card lets the opponent choose between two bad outcomes."
- "No existing card has spark that scales with the number of events in your void."

**Novelty gate FAIL examples:**
- "No existing card materializes a random Spirit Animal costing 2 or less." (Same as Light
  of Emergence with a subtype filter.)
- "No existing card draws a card when a Warrior is banished." (Same play experience as
  dozens of draw-on-trigger cards.)

If validation fails, promote the next design and validate that instead.

# Phase 6: Final Output

**Print ONLY the final card design(s).** Do not print the intermediate analysis, the 10
design attempts, or the ranking. The user wants to see clean output.

If the user requested top-N (e.g., "top 3"), print N designs **ranked from best to worst**,
with #1 being your strongest recommendation. Default is 1.

**Hard limits:** Card names must be 25 characters or fewer. Rules text must be 100
characters or fewer.

For each final design, print:

- **Card Name:** Evocative short name (max 25 characters)
- **Inspired by:** [Hearthstone card name] — one sentence on what inspired the translation
- **Card Type:** Character (with subtype) or Event
- **Tide:** Which tide and its tide cost (1-3)
- **Energy Cost:** Proposed cost
- **Spark:** Proposed spark value (characters only)
- **Rarity:** Common, Uncommon, Rare, or Legendary
- **Fast:** Yes/No
- **Rules Text:** Proposed ability text (max 100 characters)
- **Archetype Description:** One sentence on how this card supports its tide's strategy
- **Narrative:** One sentence — who is this character / what is this event?
- **Novelty Statement:** "No existing card ___."
- **Similar Cards:** 2-4 existing Dreamtides cards with the closest mechanical overlap, with
  one sentence each explaining why your card creates a **different play experience**

### Design Principles

1. **Fun is non-negotiable.** The card must be exciting to play. "Solid but not exciting" is
   a rejection.
2. **Inspiration, not imitation.** The Hearthstone card is a creative springboard. The final
   design should feel native to Dreamtides, not like a port.
3. **Simplicity serves novelty.** One clean idea expressed clearly, not three ideas stapled
   together.
4. **Avoid duplication.** Use the research script to check thoroughly.
5. **Cost appropriately.** Use the benchmarks above.
6. **Embrace the digital medium.** Mechanics that would be impractical in paper are welcome.
7. **Tide commitment should match power.** Tide-cost 1 for role-players, 2 for rewarding
   deep commitment, 3 for build-arounds.
8. **Think about the draft.** Cards that work in their primary tide but are also playable in
   an adjacent hybrid strategy are more interesting.

### Design Anti-Patterns

- **1:1 mechanical port** — translating Hearthstone keywords into Dreamtides equivalents
  without creative reinterpretation. The card should be *inspired by*, not *converted from*.
- **Derivative design** — "[existing card] but [different numbers/tide/trigger]."
- **Kindle as identity** — Kindle should never be the card's primary mechanic. -2 ranking.
- **Parasitic design** — Cards that do literally nothing without specific other cards.
- **Opponent-cooperative triggers** — Abilities the opponent can make blank by changing play.
- **Overcomplexity** — If it doesn't fit in 100 characters, simplify.
- **Stapled mechanics** — Two unrelated abilities with no connecting theme.
- **Wrong-tide mechanics** — A Rime card generating figments, a Surge card with abandon.

### Naming Guidelines

Names must be **25 characters or fewer**. Use the research script (`name` command) to check
the naming landscape. Common structures:

- **[Adjective] [Noun]:** Silent Avenger, Eternal Sentry
- **[Compound] [Noun]:** Bloomweaver, Starcatcher
- **[Noun] of [Noun]:** Titan of Forgotten Echoes
- **Single word:** Apocalypse, Reunion, Nocturne
- **The [Title]:** The Devourer, The Rising God
