# Tides V2

## Scope

This spec defines the tide system for quest drafting. A Dreamcaller points at a
curated set of tide mini-decks. The run's draft pool is built once, up front,
by combining those tides into a multiset of about 200 cards.

Runtime data is intentionally minimal:

- Per card: `tides = [tide_id, ...]`
- Per Dreamcaller: `mandatory_tides = [...]`, `optional_tides = [...]`

No other per-card labels, scores, or weights are used at runtime.
`notes/all_input_batches.json` remains a design-time aid only.

## Core Position

A tide is a draft package, not a faction and not a micro-tag. A good tide
changes what a run feels like to draft.

This system uses three layers:

- Structural tides: full deck shells with a real plan and real finishers
- Support tides: smaller, overlap-friendly splash packages that skew a shell
- Utility tides: broad floor packages for curve, interaction, and texture

Support tides are intentionally allowed to overlap heavily with structural
tides. That is not a bug. One of their jobs is to let a Dreamcaller borrow some
setup, early plays, interaction, or glue from a neighboring archetype without
importing that archetype's full closer suite.

This means a support tide may be more enabler-heavy than a structural tide. That
is also fine. Support tides should usually not contain the core build-arounds
that pay you for repeatedly doing the package's signature thing. If a card
mostly reads as "when you do the tide thing, get paid," it probably belongs in
the structural tide, not the support tide. What is not fine is a shipping tide
that is so tiny or abstract that it does not draft like a useful package.
"Discard outlets" by itself is too atomized. "Event setup" or "void setup" is
acceptable if it gives another deck a real play pattern, not just a keyword
pile.

## Tide Library

Structural tides are the main identity packages. These are the only tides that
may anchor a Dreamcaller. Each should be independently draftable if paired with
a few utility tides.

### Structural Tides

- `warrior_pressure`: low-curve Warriors, direct spark buffs, point racing,
  aggressive battlefield snowball
- `warrior_bastion`: sticky Warriors, favorable trades, attrition tools,
  defensive board control
- `spirit_growth`: Spirit Animals, ramp, top-of-deck play, board snowball
- `spirit_judgment`: Spirit boards that turn Judgment triggers into energy,
  spark growth, and repeated phase value
- `materialize_value`: ETB/materialized value, copies, repeat triggers, steady
  advantage
- `materialize_tempo`: bounce, blink, temporary banish, fast pressure, and
  replay timing
- `ally_formation`: generic multi-ally battlefield decks, non-figment go-wide
  payoffs, pair scoring, and formation-style spark scaling
- `fast_tempo`: dense fast cards, hand-fast enablers, opponent-turn plays, and
  explicit fast-payoff bodies
- `event_chain`: event density, cost reduction, copying, burst sequencing,
  spell-heavy turns
- `play_many`: cards-played-this-turn payoff turns, cross-card burst chains,
  sequencing rewards, and storm-style finishers that are not event-only
- `prevent_control`: prevent chains, counterspell pressure, taxes, reactive
  events, and pace control
- `discard_velocity`: self-discard, hand churn, burst draws, and discard-fueled
  tempo
- `void_recursion`: self-mill, reclaim, void-as-hand, recursive threats, and
  void threshold payoffs
- `deck_ladder`: top-of-deck access, cost-step upgrades, deck-to-battlefield
  cheats, and deck-as-a-resource combo turns
- `abandon_furnace`: abandon outlets, sacrifice value, leave-play conversion,
  death-for-resource turns, and sacrifice loops that are not mainly deck-ladder
  chains
- `figment_swarm`: figment generation, figment multiplication, figment-tribal
  payoffs, and dedicated token-board finishes
- `survivor_dissolve`: Survivors, Dissolved triggers, death loops, void rebuys,
  sticky attrition, and allied-dissolve payoff shells
- `judgment_engines`: extra Judgment phases, repeated Judgment triggers,
  phase-scaling bodies, and phase-centric payoff turns
- `character_velocity`: low-curve characters, deploy chaining, cost rebates, and
  character-dense turns that stay centered on character count rather than
  generic card-count burst
- `spark_tall`: kindle, concentrated spark growth, board compression, and
  single-threat or two-threat pressure

Structural lane notes:

- `ally_formation` is the generic wide-board shell. `figment_swarm` is for
  figment-specific token decks.
- `event_chain` is event-density. `play_many` is for cards whose main reward is
  how many total cards you played this turn.
- `spirit_growth` may borrow some top-of-deck texture, but `deck_ladder` owns
  the cards whose primary job is chaining off the top of the deck or upgrading
  bodies from deck hits.
- `abandon_furnace` owns sacrifice and resource-conversion loops. `deck_ladder`
  owns abandon-to-upgrade or abandon-to-deck-cheat chains.

Structural tide target size: 55-70 cards.

Each structural tide should roughly contain:

- 8-14 low-cost starters
- 10-16 engine cards
- 6-10 interaction cards
- 6-10 payoffs or closers
- 8-14 bridge cards that connect to support or utility tides

### Support Tides

Support tides are smaller packages that tilt the same Dreamcaller into different
versions of its plan. They are allowed to overlap with structural tides on
purpose. In practice, many support tides will function as the splashable slice
of a larger shell.

Typical support tide contents:

- starters and setup cards
- low-commitment glue cards
- smoothing, rebates, or interaction that reinforce the package's texture
- generic or low-scaling role-players that are still playable as a splash

Support tides should not be the home for build-around payoffs. In particular, do
not place these card classes into support tides:

- cards whose main value is "when" or "whenever" you discard, abandon, banish,
  dissolve, reclaim, materialize, flicker, prevent, or play fast cards or events
- cards that care how many allies, figments, Warriors, Spirit Animals, or other
  character types you have
- cards that care how many cards or events you played this turn
- cards that only become good once a shell reaches a dedicated density or
  threshold
- most exclusive closers from the parent structural tide

If a support tide carries too many of those cards, it is probably a hidden
structural tide.

Support tides:

- `big_energy`: temporary and permanent energy bursts plus flexible sinks
- `fast_setup`: cheap fast cards, reactive timing tools, and opponent-turn
  texture without fast-payoff bodies
- `hand_cycling`: looting, rummaging, and hand sculpting without discard reward
  cards
- `reclaim_characters`: character rebuy and replay setup without reclaim payoff
  cards
- `reclaim_events`: event rebuy and replay setup without event-recursion payoff
  cards
- `spark_growth`: direct spark buffs, kindle tools, and tall-board setup without
  spark-scaling reward bodies
- `spark_disruption`: shrink, flatten, steal, or otherwise manipulate enemy
  spark totals
- `go_wide_enablers`: cheap extra bodies, token makers, deployment smoothing,
  and non-scaling board support without ally-count or tribal payoff cards
- `leave_play_enablers`: sacrifice, bounce, banish, and dissolve bridges that
  let other shells exploit leave-play patterns without leave-play reward cards
- `bounce_blink_tools`: ally return, temporary banish, replay setup, and cheap
  blink infrastructure without most materialized payoff cards
- `topdeck_setup`: top-of-deck access, deck smoothing, deck stocking, and light
  deck-cheat setup without ladder payoffs or repeat-chain closers
- `void_setup`: self-mill, discard-to-void, threshold setup, and void stocking
  without void-threshold or reclaim payoff concentration
- `judgment_repeaters`: extra phases, trigger-copying, and generic Judgment
  setup that can splash into non-Judgment decks without Judgment payoff cards
- `event_setup`: cheap events, cost smoothing, cantrips, and sequencing tools
  without event-count, second-event, or cards-played-this-turn payoff cards

Support tide target size: 28-45 cards.

### Utility Tides

Utility tides are the draft floor. They should be broadly good, but each still
needs one clean role. If a card is only good in one shell, it belongs in a
structural or support tide instead.

Utility tides:

- `cheap_curve`: generically good 0-2 cost starters
- `defensive_curve`: blockers, reserve-friendly bodies, and stabilizers
- `midcurve_glue`: generic 3-5 cost role-players that fill turns without heavy
  synergy demands
- `card_flow`: generic draw and hand refuel
- `foresee_selection`: smoothing, selection, and setup
- `resource_burst`: broadly useful energy gain, rebates, and flexible sinks
- `cheap_removal`: efficient but conditional answers
- `premium_removal`: slower or rarer unconditional answers
- `fast_interaction`: prevents, bounce, and combat-speed disruption
- `hand_disruption`: discard, taxes, and card-denial pressure
- `sweepers`: reset buttons and anti-wide punishment
- `finishers`: top-end threats and closing tools
- `void_denial`: banish, void hate, and anti-recursion tools

Utility tide target size: 18-32 cards.

## Card Metadata

Every main-pool card gets a tide membership list and nothing else: `tides =
["discard_velocity", "void_setup", "cheap_removal"]`

Assignment targets:

- narrow build-arounds or tribal signposts: 1-2 tides
- ordinary synergy pieces: 2-4 tides
- splashable setup cards or true bridge cards: 3-5 tides
- generic utility cards: 4-6 tides
- exceptional all-purpose staples: 6-8 tides, very rare

Hard rules:

- every card must belong to at least 1 tide
- most cards should stop at 5 tides
- if a card wants 6+ tides, ask whether its role should instead be captured by a
  support or utility tide
- do not add a tide just because the card mentions a mechanic
- add a structural or support tide only if a drafter in that package would be
  happy to take the card in the first half of a run
- add a support tide only if another shell can actively use that card without
  needing the source shell's full payoff package
- if the card needs density, counts, or repeated triggers from the source shell
  to be worth drafting, it belongs in the structural tide, not the support tide

## Dreamcaller Data

Every Dreamcaller gets:

- 4 mandatory tides on average
- 10 optional tides on average

Mandatory tide rules:

- at least 2 must be structural tides
- at least 1 must provide curve, interaction, or setup floor through support or
  utility
- mandatory-only pool must already represent a coherent deck shell
- mandatory-only pool size target: 110-150 cards after duplicate capping

Optional tide rules:

- they create run variance, not basic functionality
- at least 2 should reinforce the main plan
- at least 2 should pull in overlapping support slices from neighboring shells
- at least 2 should improve generic texture, not identity
- if the same optional tide appears in nearly every legal subset through pure
  size pressure, it should probably be mandatory instead

Desired feel:

- same Dreamcaller, same mechanical spine every run
- different Dreamcaller, clearly different spine
- repeat runs with one Dreamcaller, noticeably different side packages
- support tides should often change how a shell drafts without changing what the
  shell fundamentally is

## Pool Construction

The run's pool is fixed at Dreamcaller selection time. There is no later tide
weighting based on draft picks.

Definitions:

- `S`: selected tides for the run
- `count(card, S)`: number of tides in `S` that contain the card
- `copies(card, S) = min(2, count(card, S))`
- `pool_size(S) = sum over all cards of copies(card, S)`

Algorithm:

1. The player chooses a Dreamcaller.
2. Set `mandatory = dreamcaller.mandatory_tides`.
3. Enumerate every subset of `dreamcaller.optional_tides` of size 3 and 4.
4. For each subset `O`, compute `pool_size(mandatory union O)`.
5. Keep only subsets whose final size is in the legal band `175-225`.
6. Preferred subsets are the legal subsets in the tighter band `190-210`.
7. If at least one preferred subset exists, choose one uniformly at random from
   the preferred set using the run seed.
8. Otherwise, choose uniformly at random from the full legal set.
9. If no legal subset exists, the Dreamcaller data is invalid and must be
   redesigned; do not ship runtime fallback logic.
10. Build the draft multiset with `copies(card, S)`.

With 10 optional tides, the largest search is only `C(10, 3) + C(10, 4) = 330`
subsets, so exact subset search is still cheap and easy to audit.

## Offer Generation

After the pool is built, drafting is uniform from the remaining multiset. For
each 4-card offer:

1. Sample card names from the remaining pool with probability proportional to
   remaining copies.
2. A single offer may not contain the same card name twice.
3. Remove one copy of each shown card from the pool, whether picked or not.
4. Continue until the quest draft ends or the pool cannot form another offer.

Duplicate cards therefore matter across the run, not inside one offer.

## Assignment Process

Card assignment should be done top-down.

1. Write tide briefs. For each tide, write one short brief covering what the
package is trying to do, what battlefield pattern it wants, what cards it needs
early, mid, and late, and what it deliberately does not try to include.

2. Build structural shells first. Each structural tide needs enough curve,
interaction, and closers to feel like an actual deck when paired with a few
utility tides.

3. Cut support slices out of the structural spaces. For each major archetype,
identify what another deck would want to borrow: setup, early plays, role
compression, light recursion, safe interaction, smoothing, or cost help. Move
those cards into support tides without moving most of the parent shell's most
exclusive win conditions. Build-around rewards, count payoffs, and
repeated-trigger payoffs should stay behind in the structural tide.

4. Anchor every card in one best home. Ask what package most wants this card and
what package misses something important if this card is absent. That answer
becomes the card's first tide.

5. Add genuine secondary homes. Add a second, third, or fourth tide only when
the card materially changes how another package can draft.

Good reasons:

  - it is a true bridge card between two structural plans
  - it is the splashable setup piece of one shell and real glue in another
  - it is a payoff in one plan and a stabilizer in another
  - it is the kind of broad role-player a specific utility tide is meant to
    provide

Bad reasons:

  - it contains a keyword that appears in the tide's theme
  - it can technically be played there
  - it needs to be somewhere and "this is close enough"

6. Audit subset pressure. Optional tide selection should change the pool's
texture, not just re-add the same staples twice. If a support tide is always
selected because it is the only way to make the math work, the Dreamcaller
package is not actually well-shaped.

## Validation

Global validation:

- every card has 1-8 tides
- fewer than 20 cards in the full game should have 7-8 tides
- every structural tide is independently recognizable in packs
- every support tide has a clear job as a splash package, not just a mechanic
  bucket
- no support tide should be mostly dead without one specific structural tide
- no support tide should contain cards whose main text is "when you do the tide
  thing, get paid"
- no support tide should contain the shell's ally-count, tribal-count,
  cards-played-this-turn, or second-event style payoff cards
- no support tide should contain most of one shell's exclusive closers
- no utility tide contains cards that are dead outside one shell

Dreamcaller validation:

- mandatory-only size is `110-150`
- at least 12 preferred optional subsets exist in the `190-210` size band
- average final pool size across preferred subsets is `195-205`
- average number of doubled cards in a final pool is `8-24`
- no optional tide appears in every preferred subset through pure size pressure
- at least some preferred subsets should differ because of support-overlap
  choices, not only because of swapping one structural tide for another

## Example Dreamcaller Packages

Self-discard Dreamcaller:

- mandatory: `discard_velocity`, `void_recursion`, `cheap_removal`, `card_flow`
- optional: `hand_cycling`, `void_setup`, `event_setup`, `reclaim_events`,
  `fast_interaction`, `finishers`, `judgment_repeaters`, `resource_burst`,
  `void_denial`

Result: some runs stay pure discard-void attrition, while others borrow cheap
event texture and replay setup without importing most `event_chain` closers.

Warrior control Dreamcaller:

- mandatory: `warrior_bastion`, `defensive_curve`, `cheap_removal`, `card_flow`
- optional: `warrior_pressure`, `spark_tall`, `go_wide_enablers`,
  `figment_swarm`, `prevent_control`, `premium_removal`, `sweepers`,
  `finishers`, `resource_burst`

Result: some runs stay pure attrition-control, while others branch into wider
Warrior floods or a taller spark endgame.

Materialize tempo Dreamcaller:

- mandatory: `materialize_tempo`, `materialize_value`, `fast_interaction`,
  `card_flow`
- optional: `bounce_blink_tools`, `fast_setup`, `judgment_repeaters`,
  `go_wide_enablers`, `premium_removal`, `finishers`, `big_energy`,
  `event_setup`, `spark_growth`

Result: some runs stay blink-tempo, while others borrow extra ETB reuse, fast
play texture, or board-flood side plans without changing the core identity.

## Rejected Models

- Do not use a seven-tide color wheel. This system is about curated packages,
  not faction identity.
- Do not ban overlap between support and structural tides. That overlap is one
  of the main reasons support tides exist.
- Do not use ultra-small "mechanic atom" tides as shipping content. Support
  tides may be enabler-leaning, but they still need to draft as coherent splash
  packages.
- Do not aim for 10 tides per card. That destroys variance and turns optional
  selection into duplicate inflation.
- Do not use runtime card weights based on earlier picks in the draft. The
  signal source should be the pool that the Dreamcaller built, not a hidden
  current.
