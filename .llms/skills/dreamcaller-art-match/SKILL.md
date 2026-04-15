---
name: dreamcaller-art-match
description: Match one dreamcaller art image to the best-fitting Dreamtides dreamcaller ability, then invent a proper-name title and story that make the mechanic feel native to the character. Use when pairing dreamcaller portrait art to `notes/dreamcallers.md`, naming a dreamcaller from a single image, or writing narrative justification for why a pictured figure has a specific dreamcaller ability.
---

# Dreamcaller Art Match

Match a single dreamcaller image to one existing dreamcaller ability and turn that
match into a convincing character identity.

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

## Workflow

### 1. Ground the image in what is visible

Describe only what the image actually shows:

- subject
- posture and gesture
- costume, props, symbols, and silhouette
- setting cues
- palette and mood
- anything that implies status, ritual, violence, patience, command, loss, or recurrence

Do not hallucinate lore from ambiguous details. Build from visible facts first.

### 2. Extract the dramatic role

Write a short narrative anchor in plain story terms:

- who this person seems to be
- what kind of moment the image captures
- what emotional or symbolic role they occupy in the dream

Think in terms like commander, mourner, scavenger, conspirator, herald, judge,
caretaker, revenant, witness, thief, or martyr.

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

Bad pattern:

- `Void Reclaimer`
- `The Discard Person`
- `Event Discount Mage`

### 5. Make the title carry the mechanic

The title must be mechanically linked to the ability, especially the part that
makes this dreamcaller distinct.

Do not paraphrase the rules text in ugly game language. Translate the mechanic
into a diegetic role:

- repeated `▸ Materialized:` triggers might suggest a caller, midwife, herald, or stage-director of arrivals
- discard causing return might suggest a ragpicker, mourner, archivist, or ferryman of the lost
- group spark scaling might suggest a standard-bearer, choir-leader, field marshal, or saint of the host
- prevent copying a card might suggest an interceptor, mirror-magistrate, oath-thief, or counterseer
- Judgment Echo might suggest a judge, oracle, final witness, or keeper of verdicts

The title should make a player feel, "Of course this person has that ability."

### 6. Write the narrative justification

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

If none of the abilities fit well, say so plainly and explain what kind of
dreamcaller art or ability would fit better. Do not force a weak match.
