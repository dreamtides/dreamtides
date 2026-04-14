# Tides CO V2

## Scope

This spec replaces the old faction-like tide system for quest drafting.
In the new system, a Dreamcaller points at a curated set of tide mini-decks.
The run's draft pool is built once, up front, by combining those tides into a
multiset of about 200 cards.
Runtime data is intentionally minimal:
- Per card: `tides = [tide_id, ...]`
- Per Dreamcaller: `mandatory_tides = [...]`, `optional_tides = [...]`
No other per-card labels, scores, or weights are used at runtime.
`notes/all_input_batches.json` remains a design-time aid only.

## Core Position

A tide is a draft package, not a faction and not a micro-tag.
A good tide contains enough of a real deck to matter: game plan, curve,
interaction, enablers, and payoffs.
A bad tide is a half-engine like "discard outlets" or "discard payoffs."
Those are assignment heuristics, not shipping tides.
This system uses hybrid granularity:
- 12 structural tides: large, self-contained deck shells
- 8 support tides: secondary packages that change how a shell plays
- 10 utility tides: curve, interaction, selection, and finishers
This starts at 30 tides total.
The rough idea of "25-50 cards per tide" and "about 10 tides per card" does
not fit a 580-card pool; it implies well over 100 tides and too much duplicate
pressure. The target is instead:
- 30 tides total
- mean tide size about 55 cards
- mean card membership about 2.8-3.0 tides
That gives real overlap without making every good card appear in every run.

## Tide Library

Structural tides are the only tides that may anchor a Dreamcaller package.
Each should be independently draftable if paired with basic utility tides.
Structural tides:

- `warrior_pressure`: low-curve Warriors, spark buffs, direct scoring pressure
- `warrior_bastion`: sticky Warriors, favorable trades, slower board control
- `spirit_growth`: Spirit Animals, ramp, top-of-deck play, board snowball
- `spirit_judgment`: Spirit boards that turn Judgment triggers into engines
- `materialize_value`: ETB/materialized value, copies, repeat triggers
- `materialize_tempo`: bounce, blink, temporary banish, fast tempo play
- `event_chain`: event density, cost reduction, copying, burst turns
- `event_control`: reactive events, prevents, taxes, pace control
- `discard_velocity`: self-discard, hand churn, discard rewards, burst turns
- `void_recursion`: self-mill, reclaim, void-as-hand, recursive threats
- `abandon_furnace`: abandon outlets, sacrifice value, leave-play conversion
- `figment_swarm`: token generation, token multiplication, go-wide payoffs

Structural tide target size: 65-85 cards.

Each structural tide should roughly contain 10-16 low-cost starters, 12-20
engine cards, 8-14 interaction cards, 8-14 payoffs/closers, and 10-20 bridge
cards that connect to support or utility tides.

Support tides are smaller draft packages that can tilt the same Dreamcaller
into different versions of its plan. A support tide may not ship as only
enablers or only payoffs.

Support tides:
- `big_energy`: temporary and permanent energy bursts plus energy sinks
- `fast_matters`: rewards playing on the opponent's turn or with fast cards
- `hand_cycling`: extra draw-discard loops and hand sculpting
- `reclaim_characters`: repeated recovery and replay of characters
- `reclaim_events`: repeated recovery and replay of events
- `spark_control`: shrink, swap, steal, or flatten spark values
- `wide_board_payoffs`: ally-count rewards for wide battlefield plans
- `leave_play_payoffs`: value when allies are abandoned, banished, dissolved

Support tide target size: 40-65 cards.

Utility tides are the floor that keeps a pool draftable. They should be broadly
good, but each still needs one clean role.

Utility tides:
- `cheap_curve`: generically good 0-2 cost starters
- `defensive_curve`: blockers, reserve-friendly bodies, stabilizers
- `card_flow`: generic draw and hand refuel
- `foresee_selection`: smoothing, selection, setup
- `cheap_removal`: efficient but conditional answers
- `premium_removal`: slower or rarer unconditional answers
- `fast_interaction`: prevents, bounce, combat-speed disruption
- `sweepers`: reset buttons and anti-wide punishment
- `finishers`: top-end threats and closing tools
- `void_denial`: banish, void hate, anti-recursion tools

Utility tide target size: 25-50 cards.

If a card is only good in one shell, it belongs in a structural or support
tide instead.

## Card Metadata

Every main-pool card gets a tide membership list and nothing else:
`tides = ["discard_velocity", "hand_cycling", "cheap_removal"]`

Assignment targets:
- narrow build-arounds or tribal signposts: 1-2 tides
- ordinary synergy pieces: 2-3 tides
- true bridge cards: 3-4 tides
- generic utility cards: 4-6 tides
- exceptional all-purpose staples: 7-8 tides, very rare

Global target: average 2.8-3.0 tides per card.

Hard rules:
- every card must belong to at least 1 tide
- most cards should stop at 4 tides
- if a card wants 5+ tides, ask whether it should instead live in a utility
  tide that many Dreamcallers can include
- do not add a tide just because the card mentions a mechanic
- add a tide only if a drafter in that package would be happy to take the card
  in the first half of a run

## Dreamcaller Data

Every Dreamcaller gets:
- 4 mandatory tides
- 8 optional tides

Mandatory tide rules:
- at least 2 must be structural tides
- at least 1 must provide curve or interaction floor
- mandatory-only pool must already represent a coherent deck shell
- mandatory-only pool size target: 110-150 cards after duplicate capping

Optional tide rules:
- they create run variance, not basic functionality
- at least 2 should reinforce the main plan
- at least 2 should open real side branches
- at least 2 should improve generic texture, not identity

Desired feel: same Dreamcaller, same mechanical spine every run; different
Dreamcaller, clearly different spine; repeat runs with one Dreamcaller,
noticeably different side packages.

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
7. If at least one preferred subset exists, choose one uniformly at random
   from the preferred set using the run seed.
8. Otherwise, choose uniformly at random from the full legal set.
9. If no legal subset exists, the Dreamcaller data is invalid and must be
   redesigned; do not ship runtime fallback logic.
10. Build the draft multiset with `copies(card, S)`.
With 8 optional tides, the largest search is only
`C(8, 3) + C(8, 4) = 126` subsets, so exact subset search is cheap and easy to
audit.

## Offer Generation

After the pool is built, drafting is uniform from the remaining multiset.
For each 4-card offer:
1. Sample card names from the remaining pool with probability proportional to
   remaining copies.
2. A single offer may not contain the same card name twice.
3. Remove one copy of each shown card from the pool, whether picked or not.
4. Continue until the quest draft ends or the pool cannot form another offer.

Duplicate cards therefore matter across the run, not inside one offer.

## Assignment Process
Card assignment should be done top-down.
1. Write tide briefs.
For each of the 30 tides, write one short brief covering what the deck is
trying to do, what battlefield pattern it wants, what cards it needs early,
mid, and late, and what support tides naturally pair with it.
2. Anchor structural membership.
Give every card one best home first. Ask what deck most wants this card and
what deck misses something important if this card is absent. That answer
becomes the card's first tide.
3. Add genuine secondary homes.
Add a second or third tide only when the card changes how another package can
draft. Good reasons:
- it is a true bridge card between two structural plans
- it is a payoff inside one plan and a stabilizer inside another
- it is the kind of glue a specific utility tide is meant to provide
Bad reasons:
- it contains a keyword that appears in the tide's theme
- it can technically be played there
- it needs to be somewhere and "this is close enough"
4. Add utility membership last.
This prevents good generic cards from being dumped into too many shells before
real archetype identity is clear.
5. Audit overlap.
Overlap is a tool, not a goal. Remove memberships that do not create meaningful
run-to-run differences. Optional tide selection should change the pool's
texture, not just re-add the same staples twice.
## Validation
Global validation:
- every card has 1-8 tides
- fewer than 15 cards in the full game should have 7-8 tides
- every structural tide is independently recognizable in packs
- no support tide is a half-engine
- no utility tide contains cards that are dead outside one shell
Dreamcaller validation:
- mandatory-only size is `110-150`
- at least 12 preferred optional subsets exist in the `190-210` size band
- average final pool size across preferred subsets is `195-205`
- average number of doubled cards in a final pool is `8-20`
- no optional tide appears in every preferred subset through pure size pressure;
  if that happens, it should probably be mandatory
## Example Dreamcaller Packages
Self-discard Dreamcaller:
- mandatory: `discard_velocity`, `hand_cycling`, `card_flow`,
  `cheap_removal`
- optional: `void_recursion`, `reclaim_events`, `event_chain`,
  `fast_matters`, `premium_removal`, `finishers`, `leave_play_payoffs`,
  `spark_control`
Result: some runs lean hard into void recursion, others become event-heavy
velocity decks, but the discard core always exists.
Warrior control Dreamcaller:
- mandatory: `warrior_bastion`, `defensive_curve`, `cheap_removal`,
  `card_flow`
- optional: `warrior_pressure`, `figment_swarm`, `wide_board_payoffs`,
  `event_control`, `premium_removal`, `sweepers`, `finishers`,
  `spark_control`
Result: some runs stay pure attrition-control, while others branch into wider
token boards or a more proactive Warrior finish.
## Rejected Models
- Do not recreate the old seven-tide color wheel. This system is about curated
  packages, not faction identity.
- Do not use ultra-small "mechanic atom" tides as shipping content. They create
  bookkeeping, not draft meaning.
- Do not aim for 10 tides per card. That destroys variance and turns optional
  selection into duplicate inflation.
- Do not use runtime card weights based on prior picks. The signal source
  should be the pool that the Dreamcaller built, not a hidden current.
