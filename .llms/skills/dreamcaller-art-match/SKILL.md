---
name: dreamcaller-art-match
description: Match one dreamcaller art image to the best-fitting Dreamtides dreamcaller ability, then invent a proper-name title and story that make the mechanic feel native to the character. Use when pairing dreamcaller portrait art to `notes/dreamcallers.md`, naming a dreamcaller from a single image, or writing narrative justification for why a pictured figure has a specific dreamcaller ability.
---

# Dreamcaller Art Match

Match a single dreamcaller image to one existing dreamcaller ability and turn that
match into a convincing character identity.

This skill is optimized for dreamcaller portrait sheets: one humanoid figure in
a neutral or near-neutral pose, usually with little or no meaningful setting or
action. Prioritize costume, silhouette, props, bearing, and expression over
background storytelling.

Run with strong reasoning. `medium` is acceptable for straightforward portraits;
use `high` when the art is ambiguous or several abilities fit.

## Inputs

Use this skill for exactly one image at a time.

Dreamcaller art candidates live in:

`~/Documents/synty/dreamcallers`

Dreamcaller rules text lives in:

`notes/dreamcallers.md`

Accept any of these as the image input:

- a `local_image` attachment
- a full local path
- a basename such as `0042.png`, resolved under `~/Documents/synty/dreamcallers`

If you cannot open the actual image, stop and report the problem. Do not match
from the filename alone.

## Suitability Check

Dreamcallers are character identities, so the art should depict a single
person-like figure with clear individual presence.

A single humanoid portrait in a neutral pose is the normal case for this skill,
not a weakness in the source art.

Reject the image and stop if the art is primarily:

- a landscape or environment
- an abstract texture or effect
- a crowd or multi-character scene
- an event or action tableau where no single person is the subject
- a non-person creature that cannot plausibly read as a named dream-person

If the image is borderline but still feels like one identifiable humanoid
presence, proceed and note the ambiguity.

## Required Sources

Read `notes/dreamcallers.md` every time. That file is the ability pool.

If a Dreamtides keyword in the chosen ability is unclear, briefly inspect local
project docs before finalizing the match. Do not guess on core mechanic terms.

Game rules are in `docs/battle_rules/battle_rules.md`.

## Local Registry

Every run must produce a totally unique dreamcaller name and title, and should
avoid overused abilities.

Use this local-only registry file:

`/tmp/dreamcaller_art_match_registry.json`

That file lives outside the repo. It is the centralized source of truth for
previously used dreamcaller naming words and prior art-to-ability matches.

Before finalizing any candidate, check it with:

`python3 .llms/skills/dreamcaller-art-match/scripts/name_registry.py check --name "Proper Name, Title"`

Before finalizing any ability, check its current usage with:

`python3 .llms/skills/dreamcaller-art-match/scripts/name_registry.py check-ability --ability "<exact ability quote>"`

After choosing the final name, immediately claim it with:

`python3 .llms/skills/dreamcaller-art-match/scripts/name_registry.py claim --name "Proper Name, Title" --image "<image path or label>" --ability "<short ability excerpt>"`

The registry normalizes to lowercase, strips punctuation, and splits hyphenated
words. Every normalized word counts. If a word has appeared in any earlier
generated dreamcaller name or title, do not reuse it. This rule is strict: if a
candidate overlaps on even one word, discard the whole candidate and invent a
different name and title.

Because the registry is in `/tmp`, uniqueness lasts until that temp file is
cleared. If `/tmp/dreamcaller_art_match_registry.json` disappears, the naming
history has been reset and must begin fresh.

If `claim` reports a conflict, assume another run claimed that word first.
Generate a new name, re-check it, and only then continue.

Ability usage counts are tracked by distinct art image, not by raw run count.
This prevents reruns of the same image from consuming extra quota.

Ability repetition caps:

- Soft cap: once an ability has already been used for 3 distinct art images, treat it as overused and actively prefer a different strong fit.
- Hard cap: once an ability has already been used for 5 distinct art images, it is banned for future runs.

The soft cap is a steering rule, not an automatic rejection. If an over-soft-cap
ability is still the best candidate, you may keep it only after checking other
serious fits and concluding they are materially worse on visual and narrative
grounds. The hard cap is absolute; do not use that ability.

## Workflow

### 1. Ground the image in what is visible

Describe only what the image actually shows:

- subject
- posture and gesture
- costume, props, symbols, and silhouette
- minimal background or setting cues, if any
- palette and mood
- anything that implies status, ritual, violence, patience, command, loss, or recurrence

Do not hallucinate lore from ambiguous details. Build from visible facts first.
For these dreamcaller portrait sheets, most of the usable signal is in wardrobe,
ornament, stance, and facial or bodily bearing rather than scene context.

When the image strongly signals a recognizable real-world or genre-coded visual
language through armor, weapons, dress, architecture, styling, or naming
patterns, name that influence directly in qualified terms such as
`samurai-coded`, `Japanese-coded court dress`, or `Roman military silhouette`.
Do not flatten specific visual identity into generic fantasy language when the
signal is strong. Prefer `samurai-coded armored retainer` over `formal warrior`
if the art clearly supports it.

### 2. Extract the dramatic role

Write a short narrative anchor in plain story terms:

- who this person seems to be
- what kind of presented identity the image captures
- what emotional or symbolic role they occupy in the dream

Think in terms like commander, mourner, scavenger, conspirator, herald, judge,
caretaker, revenant, witness, thief, or martyr.

Do not force an action beat if the art is just a posed figure. In many cases the
"moment" is simply self-presentation, office, rank, or ritual identity.

### 3. Search the ability list for resonance

Read all abilities in `notes/dreamcallers.md` and identify the strongest thematic
fits. Compare the art against the mechanic's implied fantasy:

- discard / void / reclaim: loss, memory, scavenging, return, salvage, grief
- materialize / deploy / repeated arrivals: summoning, leadership, coordination, procession
- spark buffs / wide boards: banners, inspiration, collective fervor, command
- event chaining / event discounts: scheming, ritual fluency, improvisation, momentum
- Survivor hooks: wasteland endurance, ruin-born pragmatism, salvage identity
- Warrior hooks: martial posture, dueling presence, command through force
- Judgment hooks: verdict, destiny, ceremony, final reckoning
- prevent / copy / reactiveness: foresight, interdiction, countermagic, trap-setting

Internally shortlist several candidates, then choose the single strongest final
match. Do not pick by mechanic alone; pick by combined visual, emotional, and
story fit.

After building the shortlist, run `check-ability` on each serious finalist using
the exact ability text from `notes/dreamcallers.md`.

- If an ability is below the soft cap, treat it normally.
- If an ability has hit the soft cap, downgrade it and look for a fresher fit.
- If multiple candidates fit similarly well, break the tie in favor of the less-used ability.
- If an ability has hit the hard cap, remove it from consideration entirely.

Do not let one visually broad ability become the default answer for every armored
warrior, judge, or lone champion portrait just because it is an easy thematic
fit. Repetition pressure is part of the selection problem.

### 4. Invent the dreamcaller identity

Produce a dreamcaller name in this format:

`Proper Name, Title`

The name is always a proper name plus a title separated by a comma. It is not
literally always four words, but it must read like a specific person, not a
card label.

Good pattern:

- `Ilya, Keeper of the Last Choir`
- `Seren Vale, Twice-Bound Herald`
- `Morrow, Judge of Empty Thrones`
- `Kazue Tsuki, Champion of the Solitary Vow`

Bad pattern:

- `Void Reclaimer`
- `The Discard Person`
- `Event Discount Mage`

Before finalizing, ask whether the title sounds like an earned office, epithet,
vow, sobriquet, or ceremonial role that a person in the world could actually
bear. If it reads like a deck label, a custom keyword, or a compressed mechanic
summary, discard it and try again.

### 5. Enforce total naming uniqueness

Before outputting any candidate:

- run `python3 .llms/skills/dreamcaller-art-match/scripts/name_registry.py check --name "Proper Name, Title"`
- reject the candidate if the command reports any reused word
- keep iterating until the candidate uses only fresh words

After you have the final match:

- run `python3 .llms/skills/dreamcaller-art-match/scripts/name_registry.py claim --name "Proper Name, Title" --image "<image path or label>" --ability "<short ability excerpt>"`
- if the claim fails, generate a different name and title and try again
- if the claim fails because the ability has reached the hard cap, choose a different ability

Never present an unclaimed name as final output.

**Examples in this skill are illustrative, not a vocabulary to draw from.**
Every word like `ragpicker`, `ferryman`, `mourner`, `herald`, `keeper`,
`magistrate`, `standard-bearer`, `choir-leader`, `arbiter`, `witness`,
`midwife`, `oath-bearer`, etc. that appears anywhere in this SKILL.md is shown
to demonstrate the *shape* of a good title — a diegetic role that implies a
mechanic without paraphrasing it. Do not lift those words into your final
answer. If you find yourself reaching for a word because you just read it in
the skill text, reject that word and invent a different role-noun grounded in
the specific art and ability in front of you. Treat the example list as a
worked exercise you have already seen the answers to, not as a menu.

### 6. Make the title carry the mechanic

The title must be mechanically linked to the ability, especially the part that
makes this dreamcaller distinct.

Do not paraphrase the rules text in ugly game language. Translate the mechanic
into a diegetic role:

- repeated `▸ Materialized:` triggers might suggest a caller, midwife, herald, or stage-director of arrivals
- discard causing return might suggest a ragpicker, mourner, archivist, or ferryman of the lost
- group spark scaling might suggest a standard-bearer, choir-leader, field marshal, or saint of the host
- prevent copying a card might suggest an interceptor, mirror-magistrate, oath-thief, or counterseer
- Judgment Echo might suggest a judge, oracle, final witness, or keeper of verdicts

The title must evoke the fantasy of the mechanic, not encode its condition
literally. Do not build titles out of disguised rules text, exact counts,
target restrictions, or deployment conditions.

Bad title habits:

- invented compounds whose only job is to smuggle mechanics into the title, such as `Singlegate`
- titles that secretly mean `cares about exactly one ally`, `discounts events`, or `copies prevented cards`
- phrases that only make sense if the reader already knows the exact ability text

Prefer titles that imply the story logic behind the mechanic: champion,
arbiter, witness, mourner, herald, keeper, magistrate, ferryman, executioner,
oath-bearer, or standard-bearer. A good title should make the ability feel
inevitable without sounding like paraphrased rules text.

If the art strongly suggests a specific cultural register, let that shape the
name and title choices as well. Do not default to vague pan-fantasy naming when
the visual language clearly points somewhere more specific.

The title should make a player feel, "Of course this person has that ability."

### 7. Write the narrative justification

Explain all of this directly in the output:

- who this person is
- why they exist in the dream
- why the pictured art is the right depiction of them
- why the chosen ability is their signature power
- why the title, specifically, encodes that power

The justification should read like dream-world character logic, not just game
analysis with flavor pasted on top.

## Output Format

Return these sections in order:

1. `Chosen Ability`
   Quote the exact selected ability from `notes/dreamcallers.md`.
2. `Dreamcaller Name`
   Give the final `Proper Name, Title`.
3. `Art Reading`
   Give a concise grounded description of the image and its mood.
4. `Narrative Match`
   Explain who this person is in the dream and why this art fits them.
5. `Title Justification`
   Explain why the title is mechanically linked to the ability.

Keep the output focused on one final match, not a menu of options, unless the
user explicitly asks for alternates.

## Quality Bar

The final pairing should satisfy all of these:

- the ability feels plausible for the pictured figure
- the title points at the mechanic without sounding like rules text
- the character feels like a person with a role in the dream, not a trope stub
- the story justification explains why this exact ability belongs to them
- the answer is anchored in visible art details

Failure modes to avoid:

- flattening strong cultural visual cues into generic fantasy description
- inventing clever-looking title words that are not plausible human titles
- hiding rules conditions inside pseudo-poetic compounds or abstract jargon
- choosing a title that explains the mechanic instead of expressing its fantasy

If none of the abilities fit well, say so plainly and explain what kind of
dreamcaller art or ability would fit better. Do not force a weak match.
