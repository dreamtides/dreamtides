---
name: dreamcaller-design
description: Design 5 dreamcaller abilities for a given mechanical theme. Dreamcallers are invulnerable, off-battlefield permanents that start in play and define a player's deck-building strategy. Produces 5 candidate ability designs with synergy analysis. Triggers on dreamcaller design, design dreamcaller, new dreamcaller, dreamcaller ability.
---

# Dreamcaller Design Skill

You are an expert card game designer creating novel Dreamtides dreamcaller abilities from a
mechanical theme prompt. Dreamcallers are the heart of a player's deck — they start in play,
cannot be interacted with by the opponent, have no spark, and exist off the battlefield as pure
ability engines. They are the first card a player sees in a draft and define what they build
around. Your designs must be build-defining, attention-grabbing, and well-supported by the
existing card pool. Run everything with ultrathink.

Read `docs/battle_rules/battle_rules.md` and `docs/tides/tides.md` (use the Read tool).

## Input

The user provides a **mechanical theme prompt** — a word or phrase describing a play pattern
(e.g., "tempo", "storm", "discard matters", "weird", "positional").

Interpret the theme using your judgment:
- **Specific themes** ("discard matters", "warrior tribal") → design abilities that directly
  engage with that mechanic
- **Broad themes** ("tempo", "weird", "aggro") → creatively interpret what that play pattern
  looks like as a dreamcaller ability

Do not ask follow-up questions or request confirmation at any stage of this workflow. Make
reasonable assumptions, proceed with your own judgment, and only change course if the user
voluntarily provides new direction.

No tide is assigned. No card name, cost, spark, rarity, or subtype. You are designing only the
mechanical ability.

## Dreamcaller Design Principles

**What makes a dreamcaller different from a regular card ability:**

- **Invulnerable and permanent.** The opponent cannot remove it. This means the ability will
  be active for the entire game. Design accordingly — effects that would be balanced on a
  removable character may be oppressive on a dreamcaller, and vice versa.
- **Free and automatic.** It starts in play with no cost. It cannot be a pure value machine
  (e.g., "draw a card each turn") — it must reward a specific playstyle rather than being
  generically good.
- **Build-defining.** A player who sees this dreamcaller should immediately think "I need to
  draft [specific kinds of cards] to make this work." If the dreamcaller is equally good in
  every possible deck, it has failed.
- **Mechanically faithful to the theme.** The actual incentive created by the ability must
  clearly align with the prompt. Do not confuse surface flavor with mechanical reality: if the
  ability rewards a different play pattern than the theme names, it is a miss even if the idea
  is clever.
- **First impression.** This is the first card in a draft. It needs to make the player excited
  to build around it, not just acknowledge it as useful.
- **Proactive, not reactive.** A dreamcaller should push the player toward a draft strategy —
  "I need to draft pump spells" or "I want characters with Judgment triggers." Defensive or
  safety-net abilities ("when something bad happens, it's less bad") don't direct drafting and
  are better suited to individual character cards. If a design would work equally well printed
  on a single removable character, it's probably not a good dreamcaller — dreamcallers should
  leverage their permanence to define a whole-game strategy, not just provide passive insurance.
- **Fun to play against.** An invulnerable permanent that creates a miserable play experience
  for the opponent is bad design. The opponent should be able to play around the dreamcaller
  through board positioning, card sequencing, or strategic choices — even though they can't
  remove it.
- **Never punish core gameplay.** Abilities that make the player feel bad for doing the fun
  part of the game (playing cards, attacking, materializing characters) are anti-fun. Good
  tension comes from interesting *choices between appealing options*, not from taxing basic
  actions the player must perform anyway.
- **Simple enough to parse and execute.** Prefer clean incentives, stable board states, and
  concise text over fiddly bookkeeping. If a concept requires frequent micro-adjustments,
  awkward UI handling, or wording that keeps growing exceptions and counters, simplify it.

## Gravity Wells — Obvious Designs

The following are the default dreamcaller templates. **2 of your 5 designs should use these
patterns** — they are the expected, solid foundation of a dreamcaller set. The remaining 3
should aim for more novel territory, but don't force novelty at the expense of fun or
build-around clarity.

- **"When you [game action], draw a card"** — Drawing is the most generic reward.
- **"[Subtype] characters get +N spark"** — Static tribal lord. Functional but uninteresting.
- **"When you [game action], kindle N"** — Kindle as reward trigger.
- **"Your [card type] cost N● less"** — Static cost reduction.
- **"When you [game action], gain N●"** — Energy reward trigger.
- **"When you [game action], materialize a figment"** — Token generation as trigger reward.

**Broader principle:** "When you [theme action], [reward]" is the default dreamcaller template
and should be used sparingly. The agent should resist the gravitational pull of "the theme is
X, so the ability triggers when you do X." A dreamcaller that *enables* or *transforms* the
theme's play pattern is more interesting than one that just *rewards* it.

**Example:** If the theme is "discard": "When you discard a card, draw a card" rewards
discarding. But "Cards in your void have Reclaim equal to their energy cost" *transforms* what
discarding means. The latter is more build-defining.

# Phase 1: Research

Read these three sources in parallel:

```bash
# Read the existing Dreamtides card pool (anonymized, ~584 lines)
cat rules_engine/tabula/rendered_cards_anonymized.txt
```

```bash
# Read the Hearthstone card pool (~1919 one-line ability descriptions)
cat ~/Documents/hearthstone/hearthstone.txt
```

Also re-read `docs/battle_rules/battle_rules.md`, focusing on battlefield position (front/back
rank, lanes F0-F3, back slots B0-B4, support relationships), judgment resolution, and zone
interactions.

After reading, produce a **synergy landscape analysis** (internal working notes, not presented
to user) covering:

1. **Archetypes in the pool** — What deck strategies exist? What cards define each archetype?
   Which archetypes would most benefit from a dreamcaller?
2. **Mechanical gravity wells** — What trigger→reward patterns are already overrepresented?
   What play experiences are deeply explored and should be avoided for the novel designs?
3. **Underexplored design space** — What mechanical territory relevant to the theme is
   underserved? Where are the gaps that a dreamcaller could fill?
4. **Battlefield position opportunities** — What positional mechanics (front/back rank, lanes,
   support relationships, adjacency) could a dreamcaller interact with? This space is
   underexplored — look for creative applications.
5. **Hearthstone inspiration candidates** — Flag 5-10 HS abilities whose abstract dynamics
   resonate with the theme. Extract the abstract dynamic for each (strip HS keywords, describe
   what the mechanic does in universal game terms).
6. **Support density map** — For each promising mechanical hook, estimate how many cards in the
   current anonymized pool actually support it. Use rough buckets:
   - **Broad support**: ~12+ cards
   - **Medium support**: ~6-11 cards
   - **Thin support**: ~3-5 cards
   - **Fragile support**: ~0-2 cards

When estimating support, count cards that would make a player meaningfully happier to draft this
dreamcaller, not generic good cards that every deck would play anyway. Prefer undercounting to
hand-wavy optimism.

# Phase 2: Brainstorm

Generate **8-10 rough ability concepts**, each described in 1-3 sentences. No full designs
yet — just the ability idea and a one-line note on why it's interesting.

**Constraints on the brainstorm pool:**
- At least 1 concept must be inspired by a Hearthstone ability (name the HS source)
- At least 1 concept must interact with battlefield position (front/back rank, lanes, support
  relationships)
- Aim for a mix of ability types (static, triggered, activated) — don't produce 8 triggered
  abilities
- 2-3 concepts should be "obvious" designs (straightforward trigger→reward or static buff) —
  these are expected and healthy as the solid foundation of the set
- The remaining concepts should aim for novel play patterns that go beyond
  "[trigger] → [standard reward]," but never at the expense of fun or draft clarity

**Self-filtering:** After generating all 8-10, evaluate each against this novelty test: "Does
this create a play pattern that no existing card or obvious dreamcaller design already
creates?" Flag concepts that fail, but keep them in the list.

Also evaluate each concept against these quality gates:
- **Theme-fit:** Does the incentive clearly push the named play pattern?
- **Draft pull:** Can you state in one sentence what kinds of cards this makes the player want?
- **Pool support:** Estimate how many cards in the current pool are real matches for this
  concept. Give an approximate count and bucket (`Broad`, `Medium`, `Thin`, `Fragile`). If a
  concept depends on only a few specific enablers, say so explicitly.
- **Simplicity:** Can the core idea be expressed cleanly without fiddly tracking or redundant
  wording?

Present the brainstorm pool to the user as a numbered list. The user may:
- Ask you to proceed with your top 5 picks
- Select specific concepts to keep or drop
- Suggest modifications or new directions

After presenting the brainstorm pool, proceed directly to Phase 3 using your top 5 picks unless
the user immediately interrupts with different instructions. Do not pause to ask for a choice.

For each brainstorm concept, include:
- the rough ability idea
- one-line note on why it is interesting
- **support estimate** in the format `Support: ~N cards (Bucket)`

Selection pressure:
- Concepts with **Fragile** support should almost never advance unless the user explicitly asks
  for that narrow mechanic.
- Concepts with **Thin** support must clear a higher bar on novelty and excitement than concepts
  with **Medium** or **Broad** support.
- If a concept looks cool but the support estimate is weak, say so plainly and deprioritize it.

# Phase 3: Design

Select 5 concepts from the brainstorm pool (self-selected or guided by user feedback) and
flesh each into a full design.

**Each design includes:**

1. **Ability text** — Using templating conventions (▸ triggers, ● energy, ✦ spark, ↯fast,
   etc.). Max 100 characters.
2. **Ability type** — Static, triggered, activated, or combination
3. **Design rationale** — 2-3 sentences on what play pattern this creates, what it asks the
   player to draft, and why it's interesting as a dreamcaller specifically
4. **Synergy citations** — 3-5 specific cards from the anonymized pool that this dreamcaller
   would synergize with, with a brief note on each explaining how
5. **Support estimate** — Approximate number of cards in the current anonymized pool that are
   real matches for this dreamcaller, plus the support bucket (`Broad`, `Medium`, `Thin`,
   `Fragile`) and a one-sentence explanation of what counts toward that estimate
6. **Novelty statement** — "No existing card ___" — the unique play experience this creates.
   For the 1-2 obvious designs, this can describe why the obvious approach is the right one.
7. **Inspiration source** — If from Hearthstone, name the HS ability and the abstract dynamic
   extracted. If positional, explain the interaction. Otherwise, describe what sparked the idea.

**Design constraints:**
- 2 designs should be straightforward "obvious" dreamcaller abilities — solid, clear
  build-arounds that use standard templates well
- The remaining 3 designs should have novel play patterns beyond "trigger when X, reward Y"
- At least 1 design must be from Hearthstone inspiration
- At least 1 design must interact with battlefield position
- Mix ability types — don't produce 5 triggered abilities
- Prefer concepts with **Medium** or **Broad** support. A final design with **Thin** support is
  acceptable only if the set still has strong overall pool coverage and the design is especially
  compelling. Do not present any final design with **Fragile** support unless the user
  explicitly asked for a narrow, low-support mechanic.

**Design anti-patterns:**
- **Opponent chooses / punisher mechanics.** One option is almost always correct for the
  opponent, so the "choice" is illusory. Do not use this design space.
- **Parasitic design.** Abilities that do literally nothing without extremely specific other
  cards. A dreamcaller should work with a *category* of cards, not require a specific combo
  piece.
- **Opponent-cooperative triggers.** Abilities the opponent can make blank by simply changing
  their play. Since the dreamcaller is permanent, a blank ability means the player's entire
  draft identity is neutralized.
- **Pure value machines.** "Draw a card each turn" or "Gain 1● each turn" with no
  build-around requirement. These are generically good, not build-defining.
- **1:1 Hearthstone port.** Translate the abstract dynamic, don't just rename HS keywords.
- **Reactive/defensive designs.** Abilities that mitigate bad outcomes ("when your character
  would be dissolved, instead...") don't tell the player what to draft. They're safety nets,
  not strategies. If the ability would work equally well as a one-off character ability, it
  fails the dreamcaller test — dreamcallers must leverage their permanence to define a
  *proactive* whole-game plan.
- **Punishing core gameplay.** Abilities that tax or penalize basic game actions (playing
  cards, materializing characters, attacking) create feel-bad moments, not interesting
  decisions. If the "tension" comes from the player being punished for doing what the game
  is about, the design is anti-fun.
- **Mechanical mismatch.** Always verify that the mechanical incentive actually pushes toward
  the stated theme. If a "go tall" ability mechanically rewards having many characters, or a
  "go wide" ability only benefits one character, the design contradicts itself regardless of
  how interesting the concept sounds on paper.
- **Incidental rewards mistaken for archetypes.** A nice bonus is not automatically a
  build-around. If the ability would usually be appreciated as a side perk rather than a reason
  to draft toward a plan, it is too weak for a dreamcaller.
- **Thin pool support.** Do not rely on hand-wavy future cards or one narrow combo. If you
  struggle to find strong existing synergy citations from the actual pool, the design is not
  ready. "Cool in theory" is not enough; if the support estimate is only ~0-5 cards, the
  design needs to be cut or clearly labeled as narrow.
- **Fiddly execution.** Avoid designs whose gameplay depends on repeated tiny transfers,
  excessive state tracking, or wording that feels awkward to implement or display. Preserve the
  core idea, then simplify.

**Templating conventions** (same as regular cards):

Triggered abilities:
```
▸ Materialized: Draw a card.
▸ Judgment: Gain 1●.
▸ Dissolved: Kindle 2.
```

Activated abilities:
```
2●: This character gains +1 spark.
1●, Discard a card: Kindle 2.
```

Fast activated abilities:
```
↯fast -- Abandon this character: Prevent a played event.
```

Static abilities:
```
Allied Warriors have +1 spark.
The opponent's events cost 1● more.
```

Once per turn:
```
Once per turn, when you discard a card, gain 1●, then kindle 2.
```

Symbols: `●` = energy, `✦` = spark, `✪` = victory points, `▸` = trigger, `↯` = fast

Before presenting a final design set, do one last silent pass and cut or simplify any design
that fails one of these tests:
- The theme is obvious from the mechanic, not just the explanation
- The draft incentive is strong and specific
- The synergy citations are real and convincing
- The ability text is as short and non-redundant as the concept allows

## Presenting Designs

Present all 5 designs in a clear numbered list. For each design, show:

1. **Ability text**
2. Ability type
3. Design rationale
4. Synergy citations
5. Support estimate
6. Novelty statement
7. Inspiration source

Do not ask the user which designs they'd like to keep, revise, or replace. Simply present the
designs cleanly. If the user later offers feedback, use that to revise the set.

# Phase 4: Iterate

Revise designs based on user feedback when the user chooses to provide it. For each revision:

- Re-check synergy citations against the card pool
- Re-evaluate the novelty statement
- Consider whether the revision makes the dreamcaller more or less build-defining

When the user stops after a design pass, treat the latest presented set as the current output. If
the user later requests revisions, update the set and present the revised output cleanly without
asking for confirmation.
