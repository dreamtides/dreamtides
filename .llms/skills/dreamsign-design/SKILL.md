---
name: dreamsign-design
description: Design one Dreamtides Dreamsign from a Dreamcaller prompt or a pool of MTG cards, Slay the Spire relics, or Monster Train artifacts. Use when creating a new dreamsign, designing a quest relic, or turning outside inspiration into a battle, quest, or hybrid dreamsign.
---

# Dreamsign Design Skill

You are an expert Dreamtides designer creating **one final Dreamsign**.

Dreamsigns are persistent run objects that can affect battles from turn 1 and
cannot be removed. They are closer to relics than to Dreamcallers: they should
usually **augment an existing plan** rather than define an entire deck by
themselves.

Run everything with strong independent judgment. Do not ask follow-up questions
unless the user has omitted the actual inspiration input. Make reasonable
assumptions and proceed.

## Required Reading

Read all three documents before designing:

- `docs/plans/quests/quests.md`
- `docs/battle_rules/battle_rules.md`
- `docs/tides/tides.md`

These are mandatory because Dreamsigns can live in battle rules, quest rules, or
both.

## Core Dreamsign Rules

- A Dreamsign is **persistent**, begins active immediately, and **cannot be
  removed**.
- A Dreamsign is **not** a Dreamcaller. It should usually be narrower, smaller,
  and less build-defining.
- Dreamsigns may be:
  - **Battle-facing**
  - **Quest-facing**
  - **Hybrid**
- **Battle-facing Dreamsigns must connect to the tides system.** The design
  should clearly support a real tide package, battlefield plan, or tide bridge.
- **Quest-facing Dreamsigns do not need tide linkage.** They may instead modify
  draft offers, map rewards, dreamsign offers, shops, essence flow, deck
  shaping, battle rewards, or other run-level systems from `quests.md`.
- Never design a battle Dreamsign with an activated ability.
- Use battle triggered abilities only rarely. Prefer static text.
- Triggered abilities outside battle are fine.
- Avoid narrative writing. Output ability text and mechanical justification only.

## What Good Dreamsigns Do

- Make an existing deck or run pattern feel sharper, cleaner, or more distinct.
- Change incentives without monopolizing the whole run.
- Create interesting deckbuilding or map-routing texture.
- Stay understandable at a glance despite being always-on.
- Feel worth acquiring, but not mandatory in every deck that can cast cards.

## What To Avoid

- Dreamcaller-scale engines or all-purpose value machines.
- Effects that would be generically best in nearly every run.
- Battle text that asks for repeated clicks, timing prompts, or constant memory
  tracking.
- Activated battle abilities.
- Heavy battle trigger bookkeeping unless the payoff is unusually elegant.
- Designs that ignore tides while still trying to be battle-facing.
- 1:1 imports from MTG, Slay the Spire, or Monster Train.

## Operating Modes

Choose the mode from the user input and follow the matching section.

### Mode 1: Dreamcaller Inspired

Input: a Dreamcaller description.

Goal: design a Dreamsign that would naturally belong in decks attracted to that
Dreamcaller, without simply repeating the Dreamcaller text at smaller numbers.

Guidelines:

- Identify what the Dreamcaller is really asking the drafter to do.
- Support the surrounding deck texture, not the exact same reward loop.
- Prefer enabling, smoothing, backup payoffs, or side incentives over direct
  duplication.
- If the Dreamcaller is already highly synergistic and narrow, make the
  Dreamsign a stabilizer or bridge piece.
- If the Dreamcaller is broad, the Dreamsign can be more specifically tide
  pointed.

### Mode 2: Magic the Gathering Inspired

Input: a pool of roughly 25 MTG cards.

Goal: use the pool as a creativity spark, then convert the interesting dynamics
into a Dreamtides-native Dreamsign.

Guidelines:

- Do not search for literal copies. Extract the useful gameplay dynamics.
- Translate inspiration into Dreamtides concepts: battlefield geometry, energy
  pacing, Judgment timing, materialize/reclaim/void play, quest reward shaping,
  and the tide package system.
- For battle Dreamsigns, ask which tides would actually want this and why.
- For quest Dreamsigns, ask what run-level behavior becomes more fun, not merely
  more efficient.
- Prefer designs that feel native to Dreamtides even if the spark came from MTG.

### Mode 3: Monster Train / Slay the Spire Inspired

Input: a pool of roughly 10 relics or artifacts from those games.

Goal: capture the appealing run-shaping pattern of the relics while designing a
Dreamsign that fits Dreamtides battles and quests.

Guidelines:

- Treat relics and artifacts as examples of pacing, incentives, and run texture.
- Do not port numbers or wording directly.
- Lean toward compact, elegant augmentation effects.
- Quest-facing or hybrid designs are especially natural in this mode, but
  battle-facing designs are valid if they still obey Dreamtides battle
  constraints.

## Design Workflow

### Phase 1: Understand the hook

Classify the best destination for the design:

- **Battle-facing** if the idea is strongest as a persistent board or deck
  modifier.
- **Quest-facing** if the idea is strongest as a run-economy, offer-shaping, or
  map-behavior modifier.
- **Hybrid** only if both halves are genuinely pulling in the same direction.

Then identify the design's job:

- smoother
- enabler
- bridge
- side payoff
- economy shaper
- reward shaper
- risk/reward modifier

If the design starts looking like a full archetype engine, shrink it.

### Phase 2: Internal brainstorm

Generate **5 to 7 rough Dreamsign concepts** internally.

For each concept, sanity check:

- Is this smaller than a Dreamcaller?
- If battle-facing, what tides or tide bridges want it?
- If quest-facing, what quest decision does it change?
- Is the text mainly static?
- Is it too generic?
- Is it too much bookkeeping for too little payoff?

Do not print the full brainstorm pool unless the user asks for alternatives.

### Phase 3: Select and refine

Choose the single best concept using this priority order:

1. Feels native to Dreamtides
2. Supports a real deck or run pattern
3. Elegant always-on play pattern
4. Fun decision pressure
5. Low bookkeeping
6. Novel enough to be worth existing

Refine until the Dreamsign text is concise and stable.

## Design Heuristics

### Battle-facing heuristics

- Prefer static bonuses, rule changes, or conditionally modified defaults over
  repeated reward triggers.
- Tie the effect to real Dreamtides structures such as:
  - deployed versus reserve positioning
  - support relationships
  - Judgment incentives
  - materialize timing
  - event density
  - void usage
  - subtype pressure
  - cheap-character curves
  - pace and tempo shells
- The best battle Dreamsigns often make a tide package play a little cleaner,
  safer, or more explosive in its preferred board states.
- If a battle design would be equally perfect in every deck, it has failed.

### Quest-facing heuristics

- Use the actual quest systems from `quests.md`.
- Good quest Dreamsign hooks include:
  - shop behavior
  - essence gains or discounts
  - draft pick shaping
  - dreamsign offer behavior
  - map site incentives
  - battle reward changes
  - run inventory pressure
  - bane handling
- Favor effects that influence meaningful run choices instead of passively
  granting raw value every time something happens.

### Hybrid heuristics

- Only use hybrid if the battle and quest pieces clearly express one unified
  gameplay idea.
- Keep each half modest. Two medium effects are usually too much on a permanent,
  unremovable object.

## Output

By default, output concise prose with exactly these sections:

### Dreamsign

One line naming the design direction, if helpful.

### Type

State `Battle`, `Quest`, or `Hybrid`.

### Ability Text

Print only the final Dreamsign rules text.

### Justification

Explain:

- why this design is strong
- why it is fun
- what deck or run pattern it supports
- for battle Dreamsigns, which tides or tide families it reinforces
- why it is Dreamsign-scale rather than Dreamcaller-scale

### Rejected Alternatives

List 2 or 3 short bullets describing the best discarded concepts and why the
final design beat them.

## Final Checks

Before answering, verify:

- no activated battle ability
- battle triggers used only if clearly worth it
- battle-facing design is tide-connected
- quest-facing design uses real quest systems
- the effect is persistent and appropriate from turn 1
- the design augments more than it defines
- the final output contains only one Dreamsign design unless the user asks for
  more
