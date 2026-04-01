# Dreamtides Tides

## Introduction

Tides are the mechanical backbone of Dreamtides draft and set structure.

They are **not** factions, colors, or deck restrictions. A tide is not meant to
be a self-contained draft lane. A tide is an **incomplete but coherent package
of tools**: a cluster of mechanics, card types, incentives, and play patterns
that naturally combines with the two tides next to it on the circle.

The tide circle is:

**Bloom -> Arc -> Ignite -> Pact -> Umbra -> Rime -> Surge -> Bloom**

The purpose of this structure is:

- each tide should feel mechanically recognizable,
- each tide should feed both adjacent decks,
- the real deck archetypes should live mainly in the **overlap** between
  neighboring tides,
- and no tide should read as a complete mono-lane by itself.

In other words:

> **A tide should not be a deck. A tide should be a cohesive mechanical**
> **cluster whose cards naturally serve both neighboring decks.**

Neutral remains the neutral / glue space. It is not part of the tide circle.

For full rules definitions of game terms used in this document, see
[battle_rules.md](../battle_rules/battle_rules.md). Key terms in brief:

- **Materialize** — put a character onto the battlefield from any zone.
- **Dreamwell** — shared card drawn each turn that produces energy.
- **Judgment** — start-of-turn phase where spark totals are compared and victory
  points scored.
- **Fast** — a card property allowing play outside normal main-phase timing.
- **Figment** — a token character created by card effects rather than played
  from a deck.

A useful way to read the system is:

- **shared creature families** often live across a pair,
- **"matters" cards** usually live more strongly in one of those two tides,
- **engines and payoffs** are intentionally split,
- and each tide should support both a named neighboring archetype and a few
  broader deck-style identities such as tempo, control, aggro, midrange, combo,
  prison, or reanimator.

______________________________________________________________________

# Mechanical Ownership Matrix

## Mechanic Categories

Not all mechanics in the matrix carry the same weight for tide identity. There
are three categories:

**Neutral infrastructure** — mechanics that every tide needs access to and that
do not define any tide's identity. Any tide can have these cards. A tide being
listed as Primary means it should have the best or most efficient versions, but
these cards do not pull drafters toward that tide specifically. Examples:

- **Raw card draw** — every tide draws cards; Rime and Surge are best at it.
- **Removal** — every tide removes things; Pact has the most density.
- **Minor materialized abilities** — most characters have enter-play effects;
  this is a baseline card quality, not a tide mechanic.

**Distributed with density** — mechanics that can appear in any tide but where
one or two tides should be noticeably denser. The Primary tide has the most
cards with this mechanic and the best payoffs for caring about it, but other
tides will still have individual cards that use it. Example:

- **Materialized abilities** (strong ETBs) — Ignite is Primary and should have
  the highest concentration, but a Bloom ramp creature or a Rime filtering
  creature can still have a strong materialized ability. The difference is that
  Ignite is the tide where you go *because* you want to build around repeated
  materializations.

**Identity-defining** — mechanics that are concentrated in one or two tides and
rarely appear elsewhere. These are the mechanics that make a tide feel like
itself and that pull drafters into specific lanes. Examples:

- **Event copying** — almost exclusively Surge.
- **Warrior Figment generation** — almost exclusively Pact.
- **Permanent ramp** — almost exclusively Bloom.

The matrix below includes all three categories. When evaluating whether each
tide has enough design space, focus on the identity-defining and
distributed-with-density rows. Neutral infrastructure should be roughly balanced
across all tides and does not contribute to a tide feeling mechanically
distinct.

## Ownership Levels

- **Primary** — this tide is the main home for that mechanic. For
  identity-defining mechanics, this means the mechanic is concentrated here. For
  distributed mechanics, this means the highest density is here. For neutral
  infrastructure, this means the best or most efficient versions are here.
- **Secondary** — this tide materially supports that mechanic.
- **Tertiary** — this tide may have glue pieces, but not much density.
- **Not Real** — this mechanic should not meaningfully pull drafters here.

Blm=Bloom, Arc=Arc, Ign=Ignite, Pct=Pact, Umb=Umbra, Rim=Rime, Srg=Surge.

### Identity-Defining Mechanics

| Function                       | Blm   | Arc   | Ign   | Pct   | Umb   | Rim   | Srg   |
| ------------------------------ | ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| Permanent ramp                 | **P** | T     | N     | T     | N     | N     | N     |
| Repeatable energy generation   | **P** | T     | N     | N     | N     | N     | T     |
| Rituals / temporary energy     | **P** | T     | T     | T     | N     | N     | **S** |
| Spirit Animal presence         | **P** | **P** | N     | N     | N     | N     | T     |
| "Spirit Animals matter"        | S     | **P** | N     | N     | N     | N     | T     |
| Flicker / blink / replay       | N     | **P** | **S** | N     | N     | N     | T     |
| Repeated flicker payoff        | N     | **S** | **P** | N     | N     | N     | T     |
| "Warriors matter"              | N     | N     | **P** | S     | N     | N     | N     |
| Go-wide scaling                | T     | T     | **P** | S     | N     | N     | T     |
| Warrior Figment generation     | N     | N     | S     | **P** | N     | N     | N     |
| Individual Warrior quality     | N     | N     | S     | **P** | N     | N     | N     |
| "When you abandon" payoffs     | N     | N     | T     | **P** | S     | N     | N     |
| Profitable-to-abandon chars    | N     | N     | T     | **P** | S     | N     | N     |
| Abandon outlets                | N     | T     | T     | S     | **P** | N     | T     |
| Void recursion / reclaim       | T     | N     | T     | S     | **P** | S     | T     |
| Self-mill / void filling       | N     | T     | N     | T     | S     | **P** | T     |
| Foresee / selection / sculpt   | S     | **S** | N     | T     | T     | **P** | S     |
| Opponent discard / hand attack | N     | N     | N     | T     | T     | **P** | S     |
| "When you discard" payoffs     | N     | N     | T     | N     | T     | **P** | **S** |
| Self-discard enablers          | N     | N     | T     | T     | T     | **S** | **P** |
| "Events matter" payoffs        | T     | T     | N     | N     | N     | T     | **P** |
| Event copying                  | N     | N     | N     | N     | N     | N     | **P** |
| Prevent / counterspell         | N     | S     | N     | N     | N     | T     | **P** |
| Prison / tax / rule-setting    | N     | T     | N     | S     | T     | **P** | **S** |
| Reanimator / recursion combo   | N     | N     | T     | S     | **P** | **S** | T     |

### Distributed With Density

These mechanics appear across all tides but the Primary tide should have the
highest concentration and the strongest reason to build around them.

| Function                      | Blm | Arc   | Ign   | Pct   | Umb   | Rim   | Srg   |
| ----------------------------- | --- | ----- | ----- | ----- | ----- | ----- | ----- |
| Big payoff threats            | T   | **P** | N     | S     | T     | N     | N     |
| Materialized abilities        | T   | **S** | **P** | T     | N     | T     | T     |
| Warrior presence              | T   | N     | **P** | **P** | N     | N     | N     |
| Figment generation            | T   | T     | **S** | **P** | T     | N     | T     |
| Survivor presence             | N   | N     | N     | **P** | **P** | T     | N     |
| Hatebears / disruptive bodies | T   | S     | S     | **P** | T     | **S** | N     |
| Fast cards matter             | T   | **S** | S     | N     | T     | S     | **P** |
| Event density                 | S   | T     | T     | T     | N     | T     | **P** |

### Neutral Infrastructure

Every tide needs these. Primary here means "best at it," not "only home."

| Function              | Blm   | Arc   | Ign   | Pct   | Umb   | Rim   | Srg   |
| --------------------- | ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| Raw card draw         | S     | S     | S     | S     | T     | **P** | **P** |
| Removal               | T     | S     | S     | **P** | S     | S     | T     |
| Reach / inevitability | N     | N     | T     | S     | **S** | **S** | **P** |
| Tempo tools           | T     | **P** | **S** | N     | T     | S     | **S** |
| Midrange glue         | **S** | **S** | S     | **P** | **S** | T     | N     |
| Aggro tools           | T     | S     | **P** | **S** | N     | N     | T     |
| Tap-out control tools | N     | T     | N     | **S** | **S** | **P** | **S** |
| Draw-go control tools | N     | **S** | N     | N     | T     | **S** | **P** |

Legend: **P** = Primary, **S** = Secondary, **T** = Tertiary, **N** = Not Real

## Ownership Summary

### Bloom owns

- the best permanent ramp,
- the best repeatable energy generation,
- the best rituals / temporary energy generation,
- the highest density of kindle and spark growth effects,
- shared Spirit Animal presence with Arc,
- and mana-positive development.

### Arc owns

- the best flicker / blink / replay tools,
- shared Spirit Animal presence with Bloom,
- the best "Spirit Animals matter" cards,
- the best big payoff threats for Bloom ramp,
- and much of the tempo shell.

### Ignite owns

- the best materialized abilities,
- repeated-flicker reward space,
- shared Warrior presence with Pact,
- the strongest "Warriors matter" cards,
- and the cleanest aggro / go-wide scaling.

### Pact owns

- the best "when you abandon" payoffs,
- the best profitable-to-abandon characters,
- shared Warrior presence with Ignite,
- shared Survivor presence with Umbra,
- the best individual Warriors,
- and the best Warrior Figment generation.

### Umbra owns

- the best abandon outlets,
- shared Survivor presence with Pact,
- void recursion,
- reclaim infrastructure,
- and the void as an active resource zone.

### Rime owns

- the best void-filling through self-mill, foresee, and selection,
- the best opponent discard / hand attack,
- the best "when you discard" payoffs,
- card filtering and hand sculpting,
- prison / disruptive control tools,
- and precise void setup.

### Surge owns

- the best cards with discard as a cost (self-discard enablers),
- the best "events matter" payoffs,
- the best event copying,
- the strongest fast-card and draw-go infrastructure,
- event density,
- and critical-mass event turns.

______________________________________________________________________

# The Seven Neighbor Archetypes

## Bloom + Arc — Ramp / Go Tall

**Plan:** Use Bloom's repeatable energy generation and Spirit Animals to jump
the curve, then convert that mana advantage into Arc's large threats and Spirit
Animal payoff cards.

**How it wins:**

- accelerate earlier than the normal Dreamwell energy progression,
- land oversized Arc threats,
- scale a small number of premium Spirit Animals,
- and protect or reuse those threats with Arc reset tools.

**Bloom contributes:**

- the best repeatable energy generation,
- the best rituals,
- shared Spirit Animal presence,
- mana-positive development pieces,
- spark growth cards — especially those triggered by materializing or replaying
  creatures, which naturally pair with Arc's flicker tools.

**Arc contributes:**

- shared Spirit Animal presence,
- the best "Spirit Animals matter" cards,
- the best big ramp payoffs,
- flicker / bounce tools that retrigger premium creatures.

**Overlap in both:**

- Spirit Animals live genuinely in both tides,
- materialized-value creatures,
- tempo-safe reset tools for tall bodies,
- spark growth triggered by materializing bridges both tides — Bloom owns the
  growth card, Arc provides the repeated materializations.

**Mini-archetypes inside the pair:**

- **Tall midrange** — curve ramp into premium threats and protect them.
- **Spirit Animal engine** — more synergy-driven, centered on replaying and
  multiplying tribe payoffs.
- **Tap-out control ramp** — heavier Arc top-end, fewer tribal synergies.
- **Growth engine** — Bloom spark-growth creatures plus Arc flicker to keep
  materializing them and accumulating spark.

**Why both are required:**

- Without Bloom, Arc's payoffs arrive too late.
- Without Arc, Bloom ramps without a distinctive payoff shell.
- Bloom supplies the **mana base**; Arc supplies the **reward layer**.

______________________________________________________________________

## Arc + Ignite — Flicker

**Plan:** Arc supplies the strongest flicker tools. Ignite supplies the best
materialized abilities and the strongest reward cards for repeating those
materialized triggers.

**How it wins:**

- repeatedly flicker high-value materialized creatures,
- turn each replay into fresh board presence or value,
- scale through repeated materialization rather than static size.

**Arc contributes:**

- the best flicker / blink / replay effects,
- bounce and protective reset,
- tempo repositioning,
- the core engine that lets the deck loop materializations.

**Ignite contributes:**

- the best materialized abilities,
- repeated-flicker reward cards,
- creatures that are especially good to re-materialize,
- scaling from repeated materialization.

**Overlap in both:**

- materialized triggers,
- replay-friendly creatures,
- tempo-friendly low-curve bodies,
- some fast-card crossover.

**Mini-archetypes inside the pair:**

- **Tempo** — Arc interaction plus Ignite pressure.
- **Blink value** — slower engine deck with many materialize triggers.
- **Aggro-combo** — cheap threats that snowball from repeated materialization.

**Where fast cards matter lives:**

- mostly **Surge**, secondarily **Arc**,
- but Arc-Ignite can use fast replay as a tempo branch, not as the main home.

**Why both are required:**

- Without Arc, Ignite only has payloads and no true flicker shell.
- Without Ignite, Arc has the engine but not enough premium materialized
  targets.
- Arc provides the **flicker engine**; Ignite provides the **materialized
  payload**.

______________________________________________________________________

## Ignite + Pact — Warriors / Go Wide / Figments

**Plan:** Ignite provides the best "Warriors matter" and board-width rewards.
Pact provides the best individual Warriors and the best ability to create
Warrior Figments.

**How it wins:**

- establish a wide Warrior battlefield,
- use Pact's stronger individual Warriors to anchor the board,
- create Warrior Figments to keep density high,
- convert board mass into overwhelming Judgment pressure.

**Ignite contributes:**

- shared Warrior presence,
- the best "Warriors matter" payoffs,
- go-wide scaling,
- repeated-materialize support for swarming.

**Pact contributes:**

- shared Warrior presence,
- the best individual Warriors,
- Warrior Figment generation,
- abandon payoffs that convert extra bodies into resources.

**Overlap in both:**

- Warriors genuinely live in both tides,
- expendable battlefield bodies,
- some Figment crossover,
- cards that reward width plus turnover.

**Mini-archetypes inside the pair:**

- **Aggro** — low-curve Warrior pressure with scaling rewards.
- **Creature midrange** — stronger Pact anchors plus Ignite snowball.
- **Figments** — Pact-generated Warrior Figments with Ignite-wide payoffs.
- **Hatebears aggro** — Pact disruptive Warriors with Ignite pressure.

**Why both are required:**

- Without Ignite, Pact has strong individual Warriors but not enough payoff for
  going wide.
- Without Pact, Ignite has mass but lacks the best anchor creatures and Warrior
  Figment engine.
- Ignite provides the **tribal scaling**; Pact provides the **quality bodies**
  and **conversion tools**.

______________________________________________________________________

## Pact + Umbra — Sacrifice / Abandon

**Plan:** Pact owns the payoff side of abandonment, while Umbra owns the outlet
side. Umbra lets you abandon characters efficiently; Pact makes that action
profitable.

**How it wins:**

- repeatedly abandon profitable creatures,
- convert leave-play into cards, mana, removal, or recursion,
- outlast the opponent by turning loss into value.

**Pact contributes:**

- the best "when you abandon" payoffs,
- the best profitable-to-abandon characters,
- shared Survivor presence,
- leave-play reward cards,
- bodies that are worth cashing in.

**Umbra contributes:**

- the best abandon outlets,
- shared Survivor presence,
- recursion and reclaim support,
- void-facing infrastructure for repeat use.

**Overlap in both:**

- Survivors live genuinely in both tides,
- leave-play triggers,
- recursive attrition pieces,
- cards that like moving through the void.

**Mini-archetypes inside the pair:**

- **Attrition midrange** — grind through repeated profitable exchanges.
- **Sacrifice engine** — dense outlet/payoff shell.
- **Tap-out control** — removal and recursive inevitability.
- **Stax-lite** — punishing permanents plus recursive fodder.

**Why both are required:**

- Without Pact, Umbra is just movement without enough reward.
- Without Umbra, Pact has payoff text without enough outlets.
- Pact supplies the **profit**; Umbra supplies the **mechanism**.

______________________________________________________________________

## Umbra + Rime — Self-Mill / Void Recursion

**Plan:** Umbra provides Survivor creatures and built-in void recursion. Rime is
the better tide at filling the void through self-mill, foresee, selection, and
careful curation.

**How it wins:**

- load the void efficiently,
- recur the best pieces,
- use Survivors and reclaim to keep material flowing,
- turn the void into a long-game resource engine.

**Umbra contributes:**

- shared Survivor presence,
- built-in void recursion,
- reclaim,
- recursive payoff cards.

**Rime contributes:**

- the best self-mill,
- the best foresee / selection / filtering for void setup,
- discard and sculpting that place the right cards into the void,
- consistency and curation.

**Overlap in both:**

- Survivors bridge the two tides,
- void-facing card flow,
- recursion-adjacent pieces,
- slow inevitability tools.

**Mini-archetypes inside the pair:**

- **Reanimator** — load premium targets, recur ahead of curve.
- **Tap-out control** — trade resources, then win from the void.
- **Recursive midrange** — resilient battlefield plan with less combo focus.
- **Prison-control** — Rime denial plus Umbra inevitability.

**Why both are required:**

- Without Umbra, Rime fills the void but does not convert it well enough.
- Without Rime, Umbra recurs well but loads the void too slowly and too bluntly.
- Umbra provides the **recursion shell**; Rime provides the **fuel and
  curation**.

______________________________________________________________________

## Rime + Surge — Discard Matters

**Plan:** Rime owns the best "when you discard" payoff cards and the best
opponent discard tools. Surge owns the best self-discard enablers, along with
the fast event shell that turns discard into a real engine.

**How it wins:**

- trigger discard payoffs repeatedly,
- use Surge events and effects to enable your own discard,
- use Rime hand pressure to constrain the opponent,
- convert churn into velocity and payoff turns.

**Rime contributes:**

- the best "when you discard" payoffs,
- the best opponent discard / hand attack,
- filtering,
- card quality,
- prison-style disruption.

**Surge contributes:**

- the best self-discard enablers,
- event density,
- event payoff turns,
- fast-card infrastructure,
- draw-go and combo-control scaffolding.

**Overlap in both:**

- draw-discard effects,
- velocity tools,
- cards that turn churn into advantage,
- reactive control support.

**Mini-archetypes inside the pair:**

- **Draw-go control** — fast cards, Prevent, sculpting, hand attack.
- **Combo-control** — hold up interaction, then explode with Surge turns.
- **Discard engine** — repeated self-discard plus payoff permanents.
- **Prison** — Rime hand pressure and taxes backed by Surge reactivity.

**Where Prevent / counterspell lives:**

- primarily **Surge**,
- secondarily **Arc** and **Rime**,
- strongest as part of a Surge-centered reactive shell.

**Why both are required:**

- Without Rime, Surge discards but gets too little reward and too little
  disruption.
- Without Surge, Rime has discard rewards and hand attack but not enough
  self-discard velocity.
- Rime provides the **payoff and disruption layer**; Surge provides the
  **enabler and reactive shell**.

______________________________________________________________________

## Surge + Bloom — Storm

**Plan:** Surge owns the best "events matter" and event-copying payoffs. Bloom
owns the best rituals and temporary energy generation that make true event-chain
turns possible.

**How it wins:**

- generate extra mana with Bloom,
- chain multiple events,
- copy the best ones,
- create one or two overwhelming turns.

**Surge contributes:**

- the best "events matter" payoffs,
- event density,
- event copying,
- fast reactive support,
- event-chain finishers.

**Bloom contributes:**

- the best rituals,
- the best temporary energy generation,
- long-term mana growth,
- spark growth cards — especially those triggered by playing events or chaining
  cards, which naturally pair with Surge's event density.

**Overlap in both:**

- mana-positive setup,
- scaling cards,
- spark growth triggered by events bridges both tides — Bloom owns the growth
  card, Surge provides the event volume,
- cards that reward abundance or sequencing,
- some combo-control crossover.

**Mini-archetypes inside the pair:**

- **Storm combo** — the pure many-cards turn deck.
- **Combo-control** — slower shell with more interaction and sculpting.
- **Ramp-combo** — heavier Bloom board presence, lighter spell density.
- **Tap-out combo** — fewer held-up responses, more main-phase explosions.

**Why both are required:**

- Without Bloom, Surge's big turns are too slow.
- Without Surge, Bloom's mana bursts are just generic ramp.
- Surge provides the **event payoff shell**; Bloom provides the **mana burst**.

______________________________________________________________________

# The Seven Tide Identities

## Neutral

Neutral is the neutral glue space, not part of the circle.

**It does:**

- generic removal,
- discover,
- sweepers,
- flexible standalone utility,
- a small number of universally playable high-rarity effects.

**It does not:**

- carry a hidden archetype,
- out-synergize real tide pairs,
- replace the need for neighboring overlap.

______________________________________________________________________

## Bloom

Bloom is the tide of **resource abundance and living growth**.

**It does:**

- permanent ramp,
- repeatable energy generation,
- rituals / temporary energy generation,
- kindle and spark growth,
- shared Spirit Animal presence with Arc,
- mana-positive development.

Spark growth is a key overlap mechanic with Bloom's neighbors — see the "Kindle
/ Spark Growth" section in the ownership matrix for detailed assignment
guidance.

**Left deck: Surge + Bloom**

- powers event turns with rituals and burst mana.
- Bloom spark-growth cards triggered by events bridge the two tides.

**Right deck: Bloom + Arc**

- ramps into Arc's premium threats and Spirit Animal payoffs.
- Bloom spark-growth cards triggered by materializing bridge the two tides.

**Broader homes:**

- ramp,
- tall midrange,
- ramp-combo.

**Core idea:** Bloom creates more dreamstuff than everyone else — more energy,
more spark, more growth over time.

**It should not be:**

- the full ramp deck by itself,
- or the home of the best giant payoff threats.

______________________________________________________________________

## Arc

Arc is the tide of **flicker, replay, and state-change exploitation**.

**It does:**

- the best flicker / blink / replay,
- bounce and reset,
- shared Spirit Animal presence with Bloom,
- "Spirit Animals matter" cards,
- premium large payoffs for ramp shells,
- much of the tempo backbone.

**Left deck: Bloom + Arc**

- supplies large payoffs and Spirit Animal reward cards.

**Right deck: Arc + Ignite**

- supplies the flicker engine for Ignite payloads.

**Broader homes:**

- tempo,
- blink value,
- some draw-go,
- some protective midrange.

**Core idea:** Arc makes state changes worth repeating.

**It should not be:**

- a full blink deck with both engine and all payloads.

______________________________________________________________________

## Ignite

Ignite is the tide of **materialize payload and battlefield multiplication**.

**It does:**

- the best materialized abilities,
- repeated-flicker reward cards,
- shared Warrior presence with Pact,
- "Warriors matter" scaling,
- repeated-materialize payoffs,
- aggressive board snowball.

**Left deck: Arc + Ignite**

- supplies the best creatures and rewards to flicker repeatedly.

**Right deck: Ignite + Pact**

- supplies Warrior scaling and swarming pressure.

**Broader homes:**

- aggro,
- tempo-aggro,
- go-wide midrange.

**Core idea:** Power comes from things materializing over and over.

**It should not be:**

- just generic aggro,
- or a fully closed Warriors deck.

______________________________________________________________________

## Pact

Pact is the tide of **profitable loss and hard bargains**.

**It does:**

- the best "when you abandon" payoffs,
- profitable-to-abandon characters,
- shared Warrior presence with Ignite,
- shared Survivor presence with Umbra,
- the best individual Warriors,
- Warrior Figment generation,
- disruptive board-centric interaction.

**Left deck: Ignite + Pact**

- supplies better anchor Warriors and Warrior Figment production.

**Right deck: Pact + Umbra**

- supplies the payoff layer for abandon engines.

**Broader homes:**

- aggro-midrange,
- attrition midrange,
- hatebears,
- stax-lite.

**Core idea:** Loss should be converted into advantage.

**It should not be:**

- the best home for abandon outlets,
- or a complete sacrifice deck by itself.

______________________________________________________________________

## Umbra

Umbra is the tide of **void access, Survivors, and deliberate abandonment**.

**It does:**

- the best abandon outlets,
- shared Survivor presence with Pact,
- void recursion,
- reclaim,
- recursive attrition tools,
- reanimator infrastructure.

**Left deck: Pact + Umbra**

- supplies the outlet engine and recursive backbone.

**Right deck: Umbra + Rime**

- supplies built-in recursion and Survivor payoff.

**Broader homes:**

- reanimator,
- attrition control,
- recursive midrange.

**Core idea:** The void is a reusable operating zone.

**It should not be:**

- the main void-filling tide,
- because Rime should be better at loading the void.

______________________________________________________________________

## Rime

Rime is the tide of **curation, void setup, and discard payoff**.

**It does:**

- the best self-mill / void filling,
- the best foresee / selection / sculpting for that role,
- the best opponent discard / hand attack,
- the best "when you discard" payoffs,
- filtering, precision setup, and denial,
- prison / control scaffolding.

**Left deck: Umbra + Rime**

- loads the void efficiently and carefully for Umbra recursion.

**Right deck: Rime + Surge**

- supplies discard payoff, hand attack, and control texture.

**Broader homes:**

- tap-out control,
- prison,
- discard control,
- reanimator support.

**Core idea:** Cards are worth more when you put them in the right place.

**It should not be:**

- only generic control,
- or the primary source of self-discard enablers.

______________________________________________________________________

## Surge

Surge is the tide of **event critical mass and reactive self-churn**.

**It does:**

- the best "events matter" payoffs,
- the best event copying,
- event density,
- the best self-discard enablers,
- fast-card and Prevent infrastructure,
- event payoff turns,
- draw-go and combo-control scaffolding.

**Left deck: Rime + Surge**

- supplies the self-discard enablers and reactive event shell.

**Right deck: Surge + Bloom**

- supplies the storm payoffs and copying engine.

**Broader homes:**

- draw-go control,
- combo-control,
- storm combo,
- fast-cards-matter shells.

**Core idea:** Momentum comes from chaining actions until they avalanche.

**It should not be:**

- the best ritual tide,
- or a self-contained storm deck.

______________________________________________________________________

### Kindle / Spark Growth — Bloom's Growth Mechanic

Bloom is the primary home of cards that add spark to individual characters over
time — kindle effects, "+1 spark" triggers, and scaling spark accumulation.
Bloom is the tide of living growth, and growing characters over time is a core
expression of that identity.

**Default assignment:** A card whose primary purpose is growing spark on
characters belongs to Bloom. "Judgment: this character gains +1 spark," "when
you materialize a character, kindle 1," or "2●: this character gains +1 spark"
are Bloom cards.

**Overlap with allies:** Spark growth is a key mechanic for bridging Bloom with
its neighbors. A Bloom card that grows spark based on banishing or you
materializing your creatures naturally serves the Bloom + Arc archetype (Arc
provides repeated materializations). A Bloom card that grows spark based on
playing events naturally serves the Surge + Bloom archetype (Surge provides
event volume). In these cases the card is still Bloom — the spark growth is the
primary identity, and the trigger creates synergy with the neighbor.

**When spark growth is NOT Bloom:** A card only stops being Bloom when spark
growth is a minor rider on a mechanic that is strongly anchored to a non-ally
tide. "When you abandon a character, gain 1 spark" is a Pact card — the abandon
trigger is the card's core identity. "When you discard a card, gain 1 spark" is
a Rime card. "Warriors you control gain +1 spark" is an Ignite card. In these
cases the card's primary mechanical identity belongs to another tide and the
spark growth is incidental.

This is always a case-by-case judgment when a card has multiple mechanical
elements. A card with spark growth and reclaim could still be Bloom — reclaim
alone does not pull it to Umbra. A card with spark growth and figment generation
could still be Bloom. The question is which mechanic defines the card's
identity, not which mechanics are present.

______________________________________________________________________

# Closing Principle

The tide system is working when drafters say:

- "I need more Bloom mana pieces for my Surge shell."
- "I need more Arc flicker tools for my Ignite payloads."
- "I need more Pact Warriors for my Ignite payoffs."
- "I need more Umbra outlets for my Pact payoffs."
- "I need more Rime setup to make my Umbra recursion cards real."
- "I need more Surge self-discard enablers for my Rime payoffs."
- "I need more Rime hand-attack pieces for my Surge control shell."

It is failing when drafters say:

- "I am Bloom, so I take Bloom."
- "I am Surge, so I take Surge."
- "I am Ignite, so I take the Warrior card."
- "All the Spirit Animals are in Bloom."
- "All the Warriors are in Ignite."
- "All the Survivors are in Umbra."

A tide should be recognizable. A tide should be coherent. A tide should be
powerful.

But above all:

> **A tide should not be a deck.**
