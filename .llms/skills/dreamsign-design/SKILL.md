---
name: dreamsign-design
description: Design one Dreamtides Dreamsign from a Dreamcaller prompt, a pool of MTG cards, or Slay the Spire / Monster Train relic or artifact inspiration. Use when creating a new dreamsign, designing a quest relic, or translating outside inspiration into a battle, quest, or hybrid dreamsign.
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
- Never design a Dreamsign with an activated ability.
- Use battle triggered abilities rarely. Prefer static text.
- Triggered abilities outside battle are fine.
- Avoid narrative writing. Output ability text and mechanical justification only.

## What Good Dreamsigns Do

- Make an existing deck or run pattern feel sharper, cleaner, or more distinct.
- Change the quest metagame in interesting ways
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

Input: a pool of MTG cards.

Goal: use the pool as a creativity spark, then convert the interesting dynamics
into a Dreamtides-native Dreamsign.

Guidelines:

- Do not search for literal copies. Extract the useful gameplay dynamics.
- Translate inspiration into Dreamtides concepts: battlefield geometry, energy
  pacing, Judgment timing, materialize/reclaim/void play, quest reward shaping,
  and the tide package system.
- Think about how concepts like attacking/blocking/damage map to the judgment system
- For battle Dreamsigns, ask which tides would actually want this and why.
- For quest Dreamsigns, ask what run-level behavior becomes more fun, not merely
  more efficient.
- Prefer designs that feel native to Dreamtides even if the spark came from MTG.

### Mode 3A: Monster Train / Slay the Spire Inspired, Battle-Level

Input: a pool of relics or artifacts whose appeal is mainly in-battle.

Goal: capture the appealing combat texture of the relics while designing a
battle-facing Dreamsign that fits Dreamtides combat and tide structure.

Guidelines:

- Treat relics and artifacts as examples of combat pacing, board incentives,
  and tactical texture.
- Do not port numbers, cadence, or wording directly.
- Start by asking what makes the source relic feel strong in combat:
  smoothing, tempo, survivability, payoff concentration, or a specific combat
  stance.
- Then ask which Dreamtides tide package, board pattern, or judgment pattern
  wants that feeling.
- A strong battle Dreamsign in this mode should make a real tide shell feel
  cleaner, sharper, or more threatening from turn 1 without becoming a generic
  auto-pick.
- Prefer effects that improve a deck's preferred board states over effects that
  simply spray value every turn.
- If the idea would fit equally well in nearly every battle deck, narrow it.

### Mode 3B: Monster Train / Slay the Spire Inspired, Quest-Level

Input: a pool of relics or artifacts whose appeal is mainly run-level.

Goal: capture the appealing run-shaping pattern of the relics while designing a
quest-facing Dreamsign that fits Dreamtides run structure.

Guidelines:

- Treat relics and artifacts as examples of pacing, incentives, and run
  texture.
- Do not port numbers or wording directly.
- Start by asking what strategic behavior the source relic rewards across a
  run: greed, risk-taking, drafting narrow cards, conserving health, hitting
  shops, routing toward elites, and so on.
- Then map that behavior onto real Dreamtides quest systems such as draft
  shaping, essence flow, map routing, reward modification, shops, or dreamsign
  offers.
- A strong quest Dreamsign in this mode should create better decisions, not
  just more resources. It should make the player want to route, draft, or spend
  differently.
- Favor hooks that create a run identity with modest numbers over passive value
  text that always pays out.
- Only make this hybrid if the quest behavior and battle payoff clearly express
  one unified idea.

## Design Workflow

### Phase 0: Rules fidelity

Before designing, write a short internal rules brief for every keyword, timing
window, or zone interaction the Dreamsign touches.

- Restate the exact gameplay meaning in your own words from the docs; do not
  design from surface memory.
- If the design uses or implies `Foresee`, `Reclaim`, `Materialize`,
  `Dissolve`, `Banish`, `Judgment`, `Ending`, or reserve/deployed positioning,
  verify that term before drafting text.
- Do not add reminder text for functionality already contained in a keyword
  unless the Dreamsign is intentionally modifying that functionality.
- If any wording depends on a rules assumption, reopen the relevant doc and
  resolve it before selecting a final concept.

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

For relic/artifact inspiration specifically:

- If the source's appeal is mostly about turn-to-turn combat states, use
  **Mode 3A**.
- If the source's appeal is mostly about run pacing, drafting, rewards, or map
  choices, use **Mode 3B**.
- If both are present, choose the stronger half first and only keep the other
  half if the final design still reads as one compact idea.

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
- For STS / Monster Train combat inspiration, preserve the source relic's
  feeling of strength by identifying the exact battle problem it solves, then
  solving that problem in a tide-linked Dreamtides way.
- Strong battle Dreamsigns from relic inspiration usually do one of three
  things well: stabilize an archetype's weak draws, intensify its payoff turns,
  or reward a board pattern that already matters.

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
- For STS / Monster Train run-level inspiration, preserve the source relic's
  feeling of strength by turning it into a sharper decision economy rather than
  a permanent fountain of free resources.
- Strong quest Dreamsigns from relic inspiration usually make the player
  meaningfully better at one style of run navigation: greedier, narrower,
  more adaptive, or more committed.

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
final design beat them, unless more specific output instructions are given.

## Final Checks

Before answering, verify:

- no activated battle ability
- battle triggers used only if clearly worth it
- battle-facing design is tide-connected
- quest-facing design uses real quest systems
- the effect is persistent and appropriate from turn 1
- the design augments more than it defines
- no clause redundantly re-states an existing keyword's built-in functionality
- the final output contains only one Dreamsign design unless the user asks for
  more
