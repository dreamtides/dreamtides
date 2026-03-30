# Dreamtides Tides

## Introduction

Tides are the mechanical backbone of Dreamtides draft and set structure.

They are **not** factions, colors, or deck restrictions. A tide is an
**incomplete but coherent package of tools**: a cluster of mechanics, card
types, incentives, and play patterns that naturally combines with the two tides
next to it on the circle.

The tide circle is:

**Bloom -> Arc -> Ignite -> Pact -> Umbra -> Rime -> Surge -> Bloom**

> **A tide should not be a deck. A tide should be a cohesive mechanical cluster
> whose cards naturally serve both neighboring decks.**

Wild remains the neutral / glue space. It is not part of the tide circle.

For full rules definitions of game terms used in this document, see
[battle_rules.md](../battle_rules/battle_rules.md). Key terms in brief:

- **Materialize** — put a character onto the battlefield from any zone.
- **Dreamwell** — shared card drawn each turn that produces energy.
- **Judgment** — start-of-turn phase where spark totals are compared and victory
  points scored.
- **Fast** — a card property allowing play outside normal main-phase timing.
- **Figment** — a token character created by card effects rather than played
  from a deck.

Key structural principles:

- **Shared creature families** often live across a pair.
- **"Matters" cards** usually live more strongly in one of the two tides.
- **Engines and payoffs** are intentionally split across neighbors.

______________________________________________________________________

# Mechanical Ownership Matrix

- **P** (Primary) — main home for that mechanic.
- **S** (Secondary) — materially supports that mechanic.
- **T** (Tertiary) — may have glue pieces, but not much density.
- **N** (Not Real) — should not meaningfully pull drafters here.

Blm=Bloom, Arc=Arc, Ign=Ignite, Pct=Pact, Umb=Umbra, Rim=Rime, Srg=Surge.

| Function                       | Blm   | Arc   | Ign   | Pct   | Umb   | Rim   | Srg   |
| ------------------------------ | ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| Permanent ramp                 | **P** | T     | N     | T     | N     | N     | N     |
| Repeatable energy generation   | **P** | T     | N     | N     | N     | N     | T     |
| Rituals / temporary energy     | **P** | T     | T     | T     | N     | N     | **S** |
| Big payoff threats             | T     | **P** | N     | S     | T     | N     | N     |
| Spirit Animal presence         | **P** | **P** | N     | N     | N     | N     | T     |
| "Spirit Animals matter"        | S     | **P** | N     | N     | N     | N     | T     |
| Flicker / blink / replay       | N     | **P** | **S** | N     | N     | N     | T     |
| Repeated flicker payoff        | N     | **S** | **P** | N     | N     | N     | T     |
| Materialized abilities         | T     | **S** | **P** | T     | N     | T     | T     |
| Warrior presence               | T     | N     | **P** | **P** | N     | N     | N     |
| "Warriors matter"              | N     | N     | **P** | S     | N     | N     | N     |
| Figment generation             | T     | T     | **S** | **P** | T     | N     | T     |
| Warrior Figment generation     | N     | N     | S     | **P** | N     | N     | N     |
| Go-wide scaling                | T     | T     | **P** | S     | N     | N     | T     |
| Individual Warrior quality     | N     | N     | S     | **P** | N     | N     | N     |
| Abandon outlets                | N     | T     | T     | S     | **P** | N     | T     |
| "When you abandon" payoffs     | N     | N     | T     | **P** | S     | N     | N     |
| Profitable-to-abandon chars    | N     | N     | T     | **P** | S     | N     | N     |
| Survivor presence              | N     | N     | N     | **P** | **P** | T     | N     |
| Void recursion / reclaim       | T     | N     | T     | S     | **P** | S     | T     |
| Self-mill / void filling       | N     | T     | N     | T     | S     | **P** | T     |
| Foresee / selection / sculpt   | S     | **S** | N     | T     | T     | **P** | S     |
| Self-discard enablers          | N     | N     | T     | T     | T     | **S** | **P** |
| Opponent discard / hand attack | N     | N     | N     | T     | T     | **P** | S     |
| "When you discard" payoffs     | N     | N     | T     | N     | T     | **P** | **S** |
| Raw card draw                  | S     | S     | S     | S     | T     | **P** | **P** |
| Event density                  | S     | T     | T     | T     | N     | T     | **P** |
| "Events matter" payoffs        | T     | T     | N     | N     | N     | T     | **P** |
| Event copying                  | N     | N     | N     | N     | N     | N     | **P** |
| Fast cards matter              | T     | **S** | S     | N     | T     | S     | **P** |
| Prevent / counterspell         | N     | S     | N     | N     | N     | T     | **P** |
| Removal                        | T     | S     | S     | **P** | S     | S     | T     |
| Reach / inevitability          | N     | N     | T     | S     | **S** | **S** | **P** |
| Prison / tax / rule-setting    | N     | T     | N     | S     | T     | **P** | **S** |
| Hatebears / disruptive bodies  | T     | S     | S     | **P** | T     | **S** | N     |
| Tempo tools                    | T     | **P** | **S** | N     | T     | S     | **S** |
| Midrange glue                  | **S** | **S** | S     | **P** | **S** | T     | N     |
| Aggro tools                    | T     | S     | **P** | **S** | N     | N     | T     |
| Tap-out control tools          | N     | T     | N     | **S** | **S** | **P** | **S** |
| Draw-go control tools          | N     | **S** | N     | N     | T     | **S** | **P** |
| Reanimator / recursion combo   | N     | N     | T     | S     | **P** | **S** | T     |

______________________________________________________________________

# The Seven Neighbor Archetypes

## Bloom + Arc — Ramp / Go Tall

Bloom's repeatable energy and rituals jump the curve; Arc converts that into
large threats and Spirit Animal payoffs. Spirit Animals live genuinely in both
tides.

- **Bloom provides:** ramp, rituals, Spirit Animals, mana-positive development.
- **Arc provides:** Spirit Animal "matters" cards, big ramp payoffs, flicker /
  bounce that retriggers premium creatures, tempo backbone.
- **Mini-archetypes:** tall midrange, Spirit Animal engine, tap-out control
  ramp.
- **Why both:** Without Bloom, Arc's payoffs arrive too late. Without Arc, Bloom
  ramps without a distinctive reward shell.

## Arc + Ignite — Flicker

Arc supplies the flicker engine; Ignite supplies the best materialized abilities
and the strongest reward cards for repeating those triggers.

- **Arc provides:** flicker / blink / replay, bounce, tempo repositioning.
- **Ignite provides:** materialized abilities, repeated-flicker rewards,
  creatures worth re-materializing.
- **Mini-archetypes:** tempo, blink value, aggro-combo.
- **Why both:** Without Arc, Ignite has payloads but no flicker shell. Without
  Ignite, Arc has the engine but not enough premium materialized targets.

## Ignite + Pact — Warriors / Go Wide / Figments

Ignite provides "Warriors matter" and board-width rewards. Pact provides the
best individual Warriors and Warrior Figment generation. Warriors live genuinely
in both tides.

- **Ignite provides:** "Warriors matter" payoffs, go-wide scaling,
  repeated-materialize support.
- **Pact provides:** best individual Warriors, Warrior Figment generation,
  abandon payoffs that convert extra bodies.
- **Mini-archetypes:** aggro, creature midrange, Figments, hatebears aggro.
- **Why both:** Without Ignite, Pact lacks width payoffs. Without Pact, Ignite
  lacks anchor creatures and the Warrior Figment engine.

## Pact + Umbra — Sacrifice / Abandon

Pact owns the payoff side of abandonment; Umbra owns the outlet side. Survivors
live genuinely in both tides.

- **Pact provides:** "when you abandon" payoffs, profitable-to-abandon
  characters, Survivor presence, leave-play rewards.
- **Umbra provides:** abandon outlets, Survivor presence, recursion and reclaim,
  void-facing infrastructure.
- **Mini-archetypes:** attrition midrange, sacrifice engine, tap-out control,
  stax-lite.
- **Why both:** Without Pact, Umbra abandons without reward. Without Umbra, Pact
  has payoff text without enough outlets.

## Umbra + Rime — Self-Mill / Void Recursion

Umbra provides Survivors and built-in void recursion. Rime fills the void
efficiently through self-mill, foresee, and selection. Survivors bridge the two
tides.

- **Umbra provides:** void recursion, reclaim, recursive payoff cards.
- **Rime provides:** self-mill, foresee / selection / filtering for void setup,
  sculpting and curation.
- **Mini-archetypes:** reanimator, tap-out control, recursive midrange,
  prison-control.
- **Why both:** Without Umbra, Rime fills the void but can't convert it. Without
  Rime, Umbra recurs but loads the void too slowly.

## Rime + Surge — Discard Matters

Rime owns "when you discard" payoffs and opponent hand attack. Surge owns
self-discard enablers and the fast event shell that makes discard a real engine.

- **Rime provides:** discard payoffs, opponent discard / hand attack, filtering,
  prison-style disruption.
- **Surge provides:** self-discard enablers, event density, fast-card
  infrastructure, draw-go scaffolding.
- **Mini-archetypes:** draw-go control, combo-control, discard engine, prison.
- **Prevent / counterspell** lives primarily in Surge, secondarily in Arc and
  Rime.
- **Why both:** Without Rime, Surge discards without enough reward. Without
  Surge, Rime lacks self-discard velocity.

## Surge + Bloom — Storm

Surge owns "events matter" and event-copying payoffs. Bloom owns the rituals and
temporary energy that make true event-chain turns possible.

- **Surge provides:** "events matter" payoffs, event copying, event density,
  fast reactive support.
- **Bloom provides:** rituals, temporary energy generation, long-term mana
  growth.
- **Mini-archetypes:** storm combo, combo-control, ramp-combo, tap-out combo.
- **Why both:** Without Bloom, Surge's big turns are too slow. Without Surge,
  Bloom's mana bursts are just generic ramp.

______________________________________________________________________

# The Seven Tide Identities

## Wild

Neutral glue space: generic removal, discover, sweepers, flexible utility. Does
not carry a hidden archetype or out-synergize real tide pairs.

## Bloom — Resource Abundance

Permanent ramp, repeatable energy, rituals, shared Spirit Animals with Arc,
mana-positive development. Core idea: Bloom creates more dreamstuff than
everyone else. **Should not be** the full ramp deck by itself or the home of the
best giant payoff threats. Broader homes: ramp, tall midrange, ramp-combo.

## Arc — Flicker and Replay

Best flicker / blink / replay, bounce and reset, shared Spirit Animals with
Bloom, "Spirit Animals matter" cards, premium large payoffs, tempo backbone.
Core idea: Arc makes state changes worth repeating. **Should not be** a full
blink deck with both engine and all payloads. Broader homes: tempo, blink value,
some draw-go.

## Ignite — Materialize Payload

Best materialized abilities, repeated-flicker rewards, shared Warriors with
Pact, "Warriors matter" scaling, aggressive board snowball. Core idea: power
comes from things materializing over and over. **Should not be** just generic
aggro or a fully closed Warriors deck. Broader homes: aggro, tempo-aggro,
go-wide midrange.

## Pact — Profitable Loss

Best "when you abandon" payoffs, profitable-to-abandon characters, shared
Warriors with Ignite, shared Survivors with Umbra, best individual Warriors,
Warrior Figment generation, disruptive interaction. Core idea: loss should be
converted into advantage. **Should not be** the best home for abandon outlets or
a complete sacrifice deck by itself. Broader homes: aggro-midrange, attrition
midrange, hatebears, stax-lite.

## Umbra — Void Access

Best abandon outlets, shared Survivors with Pact, void recursion, reclaim,
recursive attrition, reanimator infrastructure. Core idea: the void is a
reusable operating zone. **Should not be** the main void-filling tide — Rime
should be better at loading the void. Broader homes: reanimator, attrition
control, recursive midrange.

## Rime — Curation and Denial

Best self-mill / void filling, best foresee / selection / sculpting, best
opponent discard / hand attack, best "when you discard" payoffs, prison /
control scaffolding. Core idea: cards are worth more when you put them in the
right place. **Should not be** only generic control or the primary source of
self-discard enablers. Broader homes: tap-out control, prison, discard control,
reanimator support.

## Surge — Event Critical Mass

Best "events matter" payoffs, event copying, event density, best self-discard
enablers, fast-card and Prevent infrastructure, draw-go and combo-control
scaffolding. Core idea: momentum comes from chaining actions until they
avalanche. **Should not be** the best ritual tide or a self-contained storm
deck. Broader homes: draw-go control, combo-control, storm combo,
fast-cards-matter shells.

______________________________________________________________________

# Closing Principle

The tide system is working when drafters say:

- "I need more Bloom mana pieces for my Surge shell."
- "I need more Arc flicker tools for my Ignite payloads."
- "I need more Pact Warriors for my Ignite payoffs."
- "I need more Umbra outlets for my Pact payoffs."
- "I need more Rime setup to make my Umbra recursion cards real."
- "I need more Surge self-discard enablers for my Rime payoffs."

It is failing when drafters say:

- "I am Bloom, so I take Bloom."
- "All the Spirit Animals are in Bloom."
- "All the Warriors are in Ignite."
- "All the Survivors are in Umbra."

> **A tide should not be a deck.**
