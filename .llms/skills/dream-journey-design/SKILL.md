---
name: dream-journey-design
description: Design one Dreamtides Dream Journey from agent choice, a specified category, a category + cost structure, or a pool of third-party game events. Use when creating a new dream journey, designing a quest event, or translating outside event inspiration into a journey.
---

# Dream Journey Design Skill

You are an expert Dreamtides designer creating **one final Dream Journey**.

A Dream Journey is a random effect that modifies a quest in some way. It is
the home of the most dramatic effects that can change a player's experience
within a single run, and the only place where neutral or negative effects
belong (Dreamsigns and Dreamcallers are usually only positive). Journeys are
also the primary vehicle for adding Bane cards to the player's deck.

A Dream Journey is the Dreamtides equivalent of an event in a game like Slay
the Spire or Monster Train. The player encounters a journey, sees a short
ability text describing what it offers, and decides whether to embark on it.

Run everything with strong independent judgment. Do not ask follow-up
questions unless the user has omitted required input for the chosen mode.
Make reasonable assumptions and proceed.

## Terminology

Two different texts exist on every journey and serve different audiences:

- **Ability text** — the user-facing text shown on the journey card,
  written as natural prose. Short enough to read at a glance, but
  coherent-sentence readability always wins over brevity. No hard length
  target. This is what the player actually reads before deciding to
  embark.
- **Ability description** — the resolved mechanical specification for
  engineers implementing the journey. Unbounded length; not player-facing.

These were previously called "preview text" and "ability text" respectively;
the renaming is deliberate. Ability text **is** the player's rules text, and
the description is the engine spec behind it.

## Required Reading

Read all three documents before designing:

- `docs/plans/quests/quests.md`
- `docs/battle_rules/battle_rules.md`
- `docs/plans/quests/banes.md`

These cover quest mode rules, battle rules, and Bane card definitions. All
three are mandatory.

## Core Dream Journey Rules

- A journey resolves at the moment the player encounters or accepts it.
  Effects can be immediate, ongoing, delayed, or banked for
  player-controlled timing.
- A journey is **usually opt-in**: the player sees the short ability
  text and chooses whether to embark, picking from a small set of journeys
  at the site. Around a quarter of the time, a journey is **forced** —
  presented alone with no choice to skip. Design with both contexts in mind.
- The ability text must accurately represent the journey. Specific
  identities can be randomized ("a random Dreamsign," "a random card you
  control"), but magnitudes, ranges, and timing windows must always be
  stated — write "2-4 Nightmares will be added" rather than "Nightmares
  will be added." Hiding the existence of a cost is never acceptable.
- **Pure-negative journeys are allowed but only work when forced.** They
  should feel **weirdly run-changing** rather than actively terrible.
  "All event cards were removed from my deck — how do I adapt?" is good.
  "You lose 200 essence" is not. The interesting question is how the
  player reshapes around the change, not how much they got punished.
- Journeys are larger and more dramatic than Dreamsigns. They can rewrite
  battle rules, modify the run economy, or introduce mechanics that
  wouldn't belong on a permanent run-modifier.
- Symmetrical effects (rules that affect both players) belong on journeys,
  not Dreamsigns.
- A journey has two texts with different audiences: the **ability text**
  is what the player sees on the card, and the **ability description** is
  the engine spec. Both must be correct, but they answer different
  questions and obey different constraints.
- Journeys can carry Banes, deck damage, lost cards, future-resource
  reductions, or conduct restrictions — costs that would feel punitive on
  a Dreamsign.

## The 7 Categories

Every journey expresses one **primary frame**. A journey can have secondary
features from other frames, but it should be classifiable into one of these
seven by its dominant design question.

### Pact — trade something real for something real
Major reward paired with a meaningful cost. The interesting design work is
balancing the two halves so the choice has tension. Default home for any
costed journey that doesn't fit a more specific category. Most journeys
with explicit cost+benefit structure live here.

### Reshape — modify pieces you already own
Transform, prune, upgrade, swap, or remove existing deck or run objects.
The journey changes what the player has rather than adding new things.
Subtractive cleanup and transformative upgrades are both Reshapes.

### Gamble — embrace random outcomes for upside
Randomness is the central appeal. The player accepts not knowing the
result in exchange for a higher ceiling, weirder possibilities, or chaotic
upside. State-evaluating conditional outcomes ("if your deck has 30 or
fewer cards, A; else B") also live here — they're structured gambles.

### Decree — alter the ongoing rules of play
A new restriction, math change, or rule rewrite that governs upcoming
battles or the rest of the run. Scope (one battle, several, full run) is a
parameter inside the category. Includes both restrictions on player action
and changes to game-state math (energy, draws, judgment timing, economy).

### Crossroads — multiple visible options, player picks
The journey IS the choice. Two or three explicitly stated branches with
distinct payoffs. The interesting work is making each branch feel like the
right call for a different kind of run.

### Horizon — open new directions or pull future value forward
Either expands what the run can become (new card types unlock, off-package
option appears, a new map path opens) or compresses time (immediate extra
picks, accelerated rewards). The unifying frame is "the run's reach grows."

### Echo — plant something that resolves later
Delayed blessing, reckoning, prophecy, or banked reward whose timing is
the core content. The temporal displacement is the design — what feels
different about this journey is when its effect lands.

### Cross-cutting pattern: state-evaluating
Conditional outcomes based on current run state are a layerable pattern,
not a category. They can appear in any of the 7 frames.

## Cost Patterns

Available to any journey. Most central to Pacts and the negative half of
Echoes. Mix and match as appropriate.

- **Bane** — add a Bane card to the player's deck (see banes.md)
- **Card loss** — remove a chosen or random card from the deck permanently
- **Deck damage** — duplicate a weak card, add filler, raise minimum size
- **Choice surrender** — reduce future draft pick count or option count
- **Conduct rule** — temporary or permanent restriction on play
- **Consistency loss** — make draws or deck access worse
- **Opponent benefit** — give the next opponent something
- **Future income loss** — reduce essence gain after future victories
- **Site sacrifice** — replace a future map site with a worse one
- **Delayed punishment** — cost arrives after a later event
- **Action tax** — make a common action more expensive
- **Card downgrade** — corrupt an owned card (cost up, weaker effect)
- **Doomed mark** — chosen card will be banished later
- **Style commitment** — boon disappears if player breaks a rule
- **Volatile permanence** — gain a powerful thing that vanishes after a battle

## Information Design

### Ability text must be literal, mechanical, AND comprehensible

The ability text is **rules text, not flavor text**. It must clearly
describe the actual mechanical effect in concrete game terms — what gets
added, removed, changed, or triggered. Prose, metaphor, poetic framing,
and implication are **not allowed**.

The ability text is **also the player's reading experience**. Write it
as natural prose — one or two coherent English sentences that parse on
first read. Being literal and mechanical is not license to write
patch-note shorthand or compressed keyword lists.

**Do not compress language to hit a length target.** There is no hard
character cap. Keep the text as short as it can be *while remaining a
natural sentence*, and no shorter. A clear 160-character sentence beats
a cryptic 90-character one every time. If shortening damages the flow —
forces slashes, colons stacked mid-sentence, run-on keyword lists,
telegraphic syntax, or awkward phrasing — lengthen it. If the mechanics
are genuinely complex enough to resist prose, use partial info-reveal
(see below) and let the sub-choice UI carry the detail, rather than
squeezing everything into one compressed line.

As a rough calibration: most journeys land in the 60-180 character
range. That is a consequence of writing good prose for the mechanics at
hand, not a target to aim at.

- GOOD: "Add 3 Lunar Voyage cards to your deck."
- GOOD: "Purge 3 random cards from your deck."
- GOOD: "Gain a random Dreamsign. Add 2 Nightmare banes to your deck."
- GOOD: "Pick 2 of 3 run-long battle rules affecting both players, or
  instead gain {dreamsign}." (partial reveal — rule identities shown at
  sub-choice; shape, scope, and timing all stated)
- BAD: "A card awakens to new power. A memory will fade in return."
  (flavor)
- BAD: "Receive a blessing in your next battle." (no mechanics)
- BAD: "One of your lesser memories will be lost." (metaphor, hidden
  magnitude)
- BAD: "Add Nightmares to your deck." (hidden magnitude — say "2-4
  Nightmares")
- BAD: "Pick 2 of 3 run-long mods (both players): Unbound materialize /
  +1 draw / +1 Dreamwell energy. Or gain {dreamsign}." (patch-notes
  shorthand — fails the comprehensibility bar even though every piece is
  technically mechanical)
- BAD: "Reach up to 4 times, ending on success. Each adds 1 Nightmare;
  Dreamsign chance: 25/50/75/100%." (compressed — the slash-delimited
  probability list and mid-sentence colon trade sentence flow for a few
  saved characters)
- GOOD: "Reach up to four times, stopping whenever you like. Each reach
  adds a Nightmare bane to your deck, then offers a random Dreamsign
  with a 25%, 50%, 75%, or 100% chance on the first through fourth
  reach respectively; success ends the event." (prose — same mechanics,
  reads as English sentences, and the length is earned by clarity)

Use "add" (not "shuffle in") for deck insertions — shuffling is battle
vocabulary, and the quest layer presents deck changes as additions.

For run-long Decrees with symmetric scope, template as: "For the duration
of this dream quest, [scope as subject] [active game-verb]." e.g. "For
the duration of this dream quest, both players' characters materialize
with Unbound" — not "For the rest of your run, in your battles all
characters enter the battlefield Unbound (both players)."

If the player can't tell from the ability text *what will happen
mechanically*, or can't parse the sentence on first read, the ability
text is wrong. Flavor belongs in the journey's title or art, not the
rules line the player uses to make a decision; patch-note compression
belongs nowhere.

### Choosing what to specify vs. leave variable

Randomized identities and targets are fine, but they must be expressed
in concrete mechanical terms — "random," "chosen at random," "a card you
choose," "a random Dreamsign from your package." Magnitudes, ranges, and
timing windows must always be stated (even if they are themselves rolled
from a range — say "2-4", not "some"). The player should know exactly
what kind of thing is happening, how much of it, when, and what is or
isn't under their control.

- **Identity** — "Gain *Moon-Eater Idol*" vs "Gain a random Dreamsign" vs
  "Gain a random Warrior Dreamsign" (specificity of the reward; any of
  these forms is fine)
- **Target** — "Banish a chosen card from your deck" vs "Banish a random
  card from your deck" — player choice vs. random; both are fine
- **Magnitude** — always stated. "Your first character each turn gets +2
  spark for your next battle." If rolled, write the range: "2-4 cards
  are purged from your deck," not "cards are purged."
- **Range** — always stated. "Gain one of 3 random Dreamsigns" shows
  the pick count.
- **Timing** — always stated. "After your next battle, banish a random
  card from your deck" vs "Banish a random card at the start of each of
  your next 3 battles" — be explicit about when effects trigger; never
  leave timing windows implied or "later."
- **Type of price** — "Add 2 random banes to your deck" vs "Add 2
  Nightmare banes to your deck" — specificity of the cost *type* is a
  design lever, but the count still must be stated.

The ability text is where these choices are made visible. Two journeys
with identical mechanics but different information design are genuinely
different journeys — but both must be written in mechanical, literal,
readable terms with all magnitudes, ranges, and timing windows stated.

### Information-reveal styles

Every journey commits to one of two information-reveal styles.
Magnitudes, ranges, and timing windows are always stated in both; the
buckets differ only in whether the specific identity/target is shown.

- **Transparent** — every axis concrete: magnitudes, targets, timings,
  types, and identities are fully specified. Inherent randomness (roll
  ranges, shuffled order) is still allowed, but the range is always
  visible: "Add 2-4 Nightmare banes to your deck" is transparent,
  "Gain 2 random Dreamsigns from your pool" is transparent.
- **Partial** — identity or target is randomized or left to the engine,
  everything else concrete. "Add 2 random Banes to your deck" or "Gain a
  random Dreamsign" are partial — the player knows exactly what KIND of
  thing happens, how much, and when, but not which specific piece.

Default to transparent. Choose partial when the randomness of identity
or target is the interesting part of the design (e.g. a Gamble journey),
**or** when full transparency would force the ability text into
unreadable shorthand (e.g. a Crossroads with 3+ named options). Readable
partial reveal beats unreadable transparency. Hiding magnitude, range,
or timing is **not** a valid style in either bucket — it creates a
memorization burden without producing meaningful tension.

### Placeholder syntax for variable ability-text content

When the ability text includes content that the engine substitutes at
display time (a rolled Dreamsign name, a chosen card, a randomly picked
Bane), use **curly-brace placeholders** with a short descriptive label:

- `Gain {dreamsign}.` — engine rolls a Dreamsign and substitutes its name
- `Lose {random card from deck}.` — engine picks a card and shows its name
- `Add 2 {bane} to your deck.` — engine selects the Bane(s)
- `Transform {chosen starter card} into {tide-aligned upgrade}.`

The placeholder names what the engine will fill in, not the literal text
shown to the player. This lets information-reveal choices stay explicit:
a placeholder always means the value will be visible to the player at
display time. If a value is supposed to remain hidden (random card lost
without revealing identity), say so in the ability description and write
the ability text without a placeholder.

## Bane Handling

Journeys are the primary vehicle for adding Banes to the player's deck.
Read banes.md to know which Banes exist before using one as a cost.

- Match Bane severity to reward magnitude. A minor Bane is appropriate
  payment for a modest boon; a major Bane should buy something genuinely
  build-changing.
- Be specific: "add 2 [specific Bane] to your deck" reads better than
  "add 2 random Banes to your deck" unless the randomness is intentional
  Gamble flavor. The count must always be stated (or a range, e.g. "2-4").
- Multiple Banes in one journey is fine for big-effect Pacts. Single Bane
  is the default.

## Operating Modes

The user invokes the skill with `mode N` (or `mode 2a`, `mode 2b`). If the
mode tag is missing, default to Mode 0.

### Mode 0: Free design

No prompt. The agent picks everything.

To avoid defaulting to familiar patterns, run two random rolls before
brainstorming. Use shell commands for actual randomness — do not pick
"randomly" from your own judgment.

```bash
shuf -i 1-7 -n 1   # category index
shuf -i 1-2 -n 1   # information-reveal style
```

Map the category roll: 1=Pact, 2=Reshape, 3=Gamble, 4=Decree,
5=Crossroads, 6=Horizon, 7=Echo. Map the info-reveal roll: 1=transparent,
2=partial. See the Information-reveal styles section for what each
bucket means — both are literal, mechanical, and readable with
magnitudes, ranges, and timing always stated; they differ only in
whether specific identity/target is shown or randomized.

Commit to both rolls before starting Phase 2. Do not re-roll if the
combination feels awkward — design through it.

### Mode 1: Big Effects

The agent designs a journey that modifies the quest or battles in a
memorable, dramatic way. Examples of "big" effects: "current energy no
longer resets each turn," "draft sites show 15 cards from all tides and
you pick 2," "the battlefield is inverted: reserved characters are the
only ones that can attack or block."

Mode 1 still uses the 7 categories. Most big effects are Decrees, but
sweeping Reshapes, Crossroads, and Horizons are equally valid. Pick the
category that frames the design best and bias toward sweeping change
within it.

Mode 1 accepts an **optional preset category**. If the caller specifies
one (recognized labels match Mode 2A), use that category instead of
picking — and still bias toward sweeping change within it.

### Mode 2A: Category-based

The user passes a category label (or omits it). Recognized labels:

- `pact`
- `reshape`
- `gamble`
- `decree`
- `crossroads`
- `horizon`
- `echo`

If the label is omitted, the agent picks one. If a fuzzy phrase is passed
("transform stuff"), match it to the closest label and proceed.

### Mode 2B: Category-based with cost

Same as 2A, but the design must have an explicit cost+benefit structure.
The Cost / Benefit output section is required in this mode. Pact is the
natural fit, but a costed Reshape ("transform a card; add a Bane") or
costed Decree is also valid.

### Mode 3: Third-party inspired

The user provides one or more event descriptions from another game (Slay
the Spire, Monster Train, or similar) as inspiration. A single event is
a valid input — treat it the same as a pool of size one.

- Do not search for literal copies. Extract the useful gameplay dynamics.
- If multiple events are provided, do not treat them as a checklist of
  patterns to cover. Extract the strongest shared dynamic.
- Translate the inspiration into Dreamtides concepts: the tide system,
  judgment, materialize/reclaim/void play, quest reward shaping, the map.
- The final journey should feel native to Dreamtides even if the spark
  came from elsewhere.

If no pool is provided in Mode 3, ask the user for one before proceeding.

## Design Workflow

### Phase 1: Understand the hook

Determine these in order:

1. **Mode** — which operating mode applies (from the user input).
2. **Category** — which of the 7 frames. In Mode 0 this comes from the
   die roll. In Modes 1, 2A, 2B, 3 the agent picks from the inputs.
3. **Information-reveal style** — what the ability text shows vs. hides.
   In Mode 0 this comes from the die roll. Otherwise the agent picks.
4. **Cost shape** (if Mode 2B or if the journey naturally wants one) —
   pick from the cost patterns list.
5. **Single player decision the journey changes** — state in one sentence
   what the player must weigh, hope for, or commit to. If you can't
   articulate this, the concept is too diffuse.

### Phase 2: Brainstorm

Generate **5-7 rough journey concepts** internally, all within the
committed category and information-reveal style.

For each concept, sanity check:

- Does this match the category's central design question?
- Is the cost (if any) proportional to the benefit?
- Does the ability text read as a coherent, natural English sentence
  (one or two sentences) that conveys the right information without
  slashes, colons stacked mid-sentence, or keyword-list compression?
- Is the player choice meaningful — would different runs make different
  calls?
- Is this a 1:1 import from another game, or a Dreamtides-native design?

Do not print the full brainstorm pool. The Rejected Alternatives section
of the output includes 2-3 of the strongest discards; the rest stay
internal unless the user asks for more.

### Phase 3: Select and refine

Choose the single best concept using this priority order:

1. Creates a real, interesting decision for the player
2. Feels native to Dreamtides
3. Matches the committed category cleanly
4. Ability text and ability description are both clear at a glance
5. Cost (if any) is proportional and well-targeted
6. Novel — not a near-copy of an existing journey or event

Refine until the ability text reads as natural prose — a coherent
English sentence (or at most two) with no patch-note shorthand — and
the ability description fully specifies the effect. Do not trim for
length at the cost of readability.

## Output Format

The format below is the **default and subject to change**. If the user
requests JSON output, produce a structured JSON object with the same
fields (one top-level object with `mode`, `category`, `ability_text`,
`ability_description`, `cost_benefit` (nullable), `justification`, and
`rejected_alternatives` keys).

By default, output prose with these sections, in this order:

### Mode
One line — `Mode N` plus a short label.

### Category
One line — one of the 7 category names.

### Ability Text
This is what the player sees on the journey card before opting in.
Write it as natural prose — one or two coherent English sentences. No
hard length cap; readability as a sentence is the priority, and brevity
is earned only insofar as the prose stays natural. Do not compress to
hit a target. It must state every magnitude, range, and timing window,
and information-reveal choices are baked in here.

### Ability Description
The resolved mechanical specification. No length limit — this is for
engineers, not players. Describe what happens when the player accepts:
which dice are rolled, what state changes, what triggers fire later. Be
explicit enough that someone implementing the journey wouldn't need to
ask follow-up questions.

### Cost / Benefit *(conditional)*
Required in Mode 2B. Optional in any mode where the journey has an
explicit costed structure (most Pacts, some Echoes). Skip when the
journey is a pure Decree, Crossroads, Gamble without cost, or Horizon
without cost. One-line cost summary, one-line benefit summary.

### Justification
2-4 sentences. Why this is strong and fun. What player decision it
forces. What kind of run benefits or suffers most. If Mode 3, briefly
note what was extracted from the inspiration.

### Rejected Alternatives
2-3 short bullets — discarded brainstorm concepts and why the final beat
them.

## What to Avoid

- **1:1 ports from Slay the Spire or Monster Train.** Extract the
  dynamic, don't rename the event.
- **Pure value with no decision.** A journey every player should always
  accept isn't a journey, it's a reward.
- **Punishing core gameplay.** Costs that tax basic actions
  (materializing, attacking, playing cards) feel anti-fun rather than
  tense. Costs should bite the player's strategy, not the act of playing.
- **Actively terrible negatives.** Pure-negative journeys should be
  weirdly run-changing, not just punishing. "Lose 200 essence" is bad;
  "all event cards leave your deck" is good — the player has something
  to figure out.
- **Bane spam.** More than 2 Banes for a single journey should be
  reserved for genuinely build-changing rewards.
- **Hidden magnitudes, ranges, or timing.** "Nightmares will be added"
  or "cards will be purged later" create a memorization/wiki burden
  without producing real tension. Always state counts (or roll ranges
  like "2-4") and timing windows explicitly.
- **Patch-note ability text.** Slash-delimited keyword lists
  ("Unbound materialize / +1 draw / +1 Dreamwell energy") are not
  acceptable even when every piece is technically mechanical. The
  ability text is what the player reads — it must be English.

## Final Checks

Before printing the design, verify:

- Mode and category are stated and match the input
- Ability text reads as one or two coherent English sentences — natural
  prose, not compressed. If you had to use slashes, stacked mid-sentence
  colons, or keyword-list shorthand to shorten it, lengthen it back out
- Ability text is literal and mechanical — no flavor, metaphor, or
  implication. A reader should know exactly what happens mechanically
  from the ability text alone, including all magnitudes (or roll
  ranges), all timing windows, and whether each identity/target is
  specific, chosen, or random
- Ability description fully specifies the effect — an implementer
  wouldn't have questions
- If Mode 2B, the Cost / Benefit section is present
- The journey makes the player make a real decision, not a default-yes
- Information-reveal choices in the ability text are deliberate
- No 1:1 port; the design is Dreamtides-native
- Output contains exactly one final journey design unless the user asked
  for alternatives
