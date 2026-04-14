# Dreamcaller Tide Assignment Postmortem

This note summarizes what happened during the 32-dreamcaller synthesis pass and
what it implies for the next round of tide work.

## Method Summary

The dreamcallers were designed against the real anonymized card pool, then
validated with the exact pool-construction algorithm from `notes/tides_v2.md`:

1. Start with a Dreamcaller's `mandatory-tides`.
2. Enumerate every 3-tide and 4-tide subset of its `optional-tides`.
3. Keep only subsets that land in the legal `175-225` card band.
4. Prefer subsets that land in the tighter `190-210` band.

The biggest lesson from the pass was that qualitative archetype logic was not
enough. Many early dreamcaller packages sounded correct, but failed the exact
subset math because the mandatory shell was too large, or because a few generic
optionals were doing all the work while the thematic optionals were effectively
dead.

## 1) Difficult Tides And Needed Revisions

### `ally_formation`

This was the hardest structural tide to work with. It appeared in 18
dreamcallers total, 11 times as mandatory. It is carrying too many jobs:

- true pair-scoring and multi-lane payoff cards
- generic non-figment go-wide payoffs
- broadly playable cheap bodies that any wide deck wants

Needed revision:

- Tighten it around "multiple real allies on board" payoffs.
- Push more generic battlefield-filler cards into `go_wide_enablers` or
  `cheap_curve`.
- Be stricter about whether a card is rewarding ally counts, or merely helping a
  deck deploy bodies.

### `character_velocity`

This also showed up 15 times, 11 as mandatory. The tide is supposed to be about
double-deploy turns, rebates, and character-chain turns, but in practice it is
too close to "good cheap character deck."

Needed revision:

- Keep cards that explicitly reward extra character plays, replaying
  characters, or chaining deployments.
- Move generic low-curve value characters out unless they materially support
  second-character turns.
- Consider splitting the generic low-curve shell from the real
  chain-deployment engine.

### `materialize_value` vs `materialize_tempo`

This boundary was unstable. Early packages that used both as mandatory almost
always overfilled. `materialize_value` is supposed to own ETB advantage and
`materialize_tempo` is supposed to own bounce / replay / pressure timing, but
the current card assignments still let both tides ingest too much of the same
space.

Needed revision:

- `materialize_value`: ETB card, energy, removal, and steady advantage.
- `materialize_tempo`: return, banish-and-return, replay timing, attack-pressure
  sequencing.
- Remove cards that are merely "good ETB cards" from `materialize_tempo` unless
  the tempo shell actively wants them early.

### `void_recursion`

This tide was powerful but awkward. It showed up only 6 times, all as mandatory.
That means it is functioning as an anchor shell, but not as a flexible side
package. The main issue is that it currently covers too many different void
plans:

- reclaim / void-as-hand play
- void threshold scaling
- self-mill setup
- recursive late-game threats

Needed revision:

- Separate threshold / "cards in your void matter" cards from straight reclaim
  recursion.
- Keep `void_setup` as setup only; do not let it quietly carry threshold payoffs
  or reclamation density.
- Consider a dedicated threshold-facing structural or support tide if the void
  count shell is meant to be a recurring dreamcaller lane.

### `abandon_furnace`

This tide was narrower than expected as a mandatory anchor, but the spec still
bundles two different archetypes:

- abandon for energy / cards / attrition conversion
- abandon ladder / upgrade / deck-cheat chains

Needed revision:

- Split the straight sacrifice-value shell from the ladder shell.
- If kept unified, drastically tighten what counts as true `abandon_furnace`
  support, because current dreamcaller design kept wanting one mode but not the
  other.

### `prevent_control`

This tide worked better than some of the others, but its scope is still too
wide. It currently owns:

- prevents
- taxes
- discard / hand denial
- denial-as-value engines

That makes it hard to tell whether a dreamcaller is drafting a real
counter-control shell or just a general opponent-denial shell.

Needed revision:

- Keep hard reactive prevent cards and true deny-on-stack payoffs in
  `prevent_control`.
- Let `hand_disruption` carry more of the proactive denial pressure.
- Be careful about tax cards that are actually generic tempo tools rather than
  prevent-shell cards.

## 2) Tides That Need To Shrink Or Be Split

### Shrink

These tides were too effective as generic pool-padding, which means they are
probably claiming too many cards:

- `resource_burst`: used in 25 dreamcallers, all optional. This is the clearest
  filler tide in the current system. It should remain useful, but right now it
  is too often the card-count fix when a shell is short.
- `cheap_curve`: used in 27 dreamcallers. Some of that is expected, but the
  tide is broad enough that it can make many shells legal almost by itself.
- `premium_removal`: used in 18 dreamcallers, always optional. This likely
  contains too many cards that are simply "always good" rather than the slower,
  narrower removal the spec calls for.
- `cheap_removal`: same problem, though slightly less severe. It is acting as
  both texture and size ballast.
- `character_tutors`: used in 16 dreamcallers, always optional. If it keeps
  showing up as a generic fix for synergy shells, it is probably too broad.
- `materialized_staples`: used in 9 dreamcallers, often as a math patch for
  character-heavy shells that were otherwise coming up short. It should be a
  real utility tide, not a hidden rescue package for every materialize-adjacent
  dreamcaller.

### Split

- `character_velocity`: split true chain-deployment support from generic cheap
  character efficiency.
- `void_recursion`: split reclaim recursion from void-threshold scaling.
- `abandon_furnace`: split sacrifice-value from ladder / deck-cheat.
- `prevent_control`: consider splitting stack denial from proactive tax /
  discard pressure if those cards continue to draft like different decks.

### Tighten Without Full Split

- `ally_formation`: keep it structural, but remove cards that belong to
  `go_wide_enablers`, `cheap_curve`, or `point_pressure`.
- `materialize_value` / `materialize_tempo`: preserve the two-tide model, but
  sharpen the assignment boundary.

## 3) Tides That Were Not Useful Or Were Unassigned

### Completely Unused

- `spark_disruption`
- `sweepers`

These did not appear in any dreamcaller, even as optional tides.

Interpretation:

- `spark_disruption` is probably too reactive / opponent-facing to function as a
  dreamcaller package on its own. It may still be a useful card-assignment tide,
  but it is not currently pulling dreamcaller identity.
- `sweepers` is likely too anti-synergy for how dreamcallers are currently being
  built. It reads more like a sideboard-style control texture tide than a real
  pool-construction choice.

### Nearly Unused

- `midcurve_glue`: used exactly once, and only as an optional tide.
- `defensive_curve`: used twice, both mandatory, only for the two most defensive
  shells.
- `warrior_bastion`: used twice.
- `event_chain`: used twice, both mandatory, and only for the event-centric
  dreamcallers.

Interpretation:

- `midcurve_glue` is not currently doing enough to justify dreamcaller slots.
  It may still be fine as a broad assignment tide, but it is not affecting
  dreamcaller identity.
- `defensive_curve` is real, but very narrow.
- `event_chain` looks healthy as a structural anchor, but not as a side package.
  That is probably correct.

### Tides That Were Useful Only As Anchors

These appeared as mandatory only, never optional:

- `discard_velocity`
- `void_recursion`
- `prevent_control`
- `abandon_furnace`
- `survivor_dissolve`

Interpretation:

- These are acting like real structural shells, which is good.
- But it also suggests the support slices around them may not be strong enough,
  or are currently being absorbed by other generic tides instead.

## 4) What To Consider In Future Rounds

### Do Exact Subset Math Early, Not At The End

This is the main process lesson. Many subagent proposals were design-sound in
the abstract but failed the real pool-size math. Future rounds should require:

- mandatory-only size
- legal subset count
- preferred subset count
- optional frequency within chosen subsets

before a dreamcaller is treated as viable.

### Watch For Dead Optionals

Several optionals were effectively dead during validation. If an optional tide
has frequency `0.0` in preferred subsets, it is not a real option. If it has
frequency near `1.0`, it is probably mandatory in disguise. Future rounds
should explicitly reject both cases unless there is a strong reason.

### Design From Tide Topology, Not Ability Fantasy

The cleanest dreamcallers were the ones that followed a consistent shape:

- 1 primary structural anchor
- 1 secondary structural or strong support bridge
- 1-2 utility floors
- optionals that genuinely branch the shell rather than just adding card count

Dreamcallers designed from "cool ability first" often ended up needing generic
patch tides to become legal.

### Audit Generic Optionals As A Group

`cheap_curve`, `resource_burst`, `premium_removal`, `cheap_removal`, and
`character_tutors` were frequently doing emergency stabilization work. That is a
warning sign. Future rounds should check whether a dreamcaller is only viable
because the same generic package cluster keeps bailing it out.

### Use Dreamcaller Design To Pressure-Test The Tide Library

The tide system is not finished when the cards are assigned. Dreamcaller
construction is a second validation layer:

- If a tide is never chosen, it may be too weak or too off-axis.
- If a tide is always chosen as optional ballast, it is probably too broad.
- If a tide can only appear as mandatory, it may need a stronger support slice.

Dreamcaller allocation should therefore be treated as a core validation pass for
the tide library, not just a downstream content task.

### Separate Executable Rules Support From Design Data

One operational lesson from this pass: several dreamcaller abilities do not fit
the current parser-backed card text DSL cleanly. Future rounds should decide up
front whether dreamcaller text is:

- display-only design data
- parser-backed executable rules text

If dreamcallers are going to become executable in the same pipeline as cards,
the parser and effect system will need explicit dreamcaller-oriented support.

### For Future Subagent Rounds

- Give each worker the exact pool-size evaluator up front.
- Require them to report failed package attempts, not just the final answer.
- Ask for one short list of likely overlap-problem tides per dreamcaller.
- Reserve final synthesis for a single pass that normalizes awakenings, removes
  duplicate names, and replaces dead optionals.

## Bottom Line

The current dreamcaller roster is workable, but it exposed a real structural
issue in the tide library: a handful of generic support and utility tides are
too broad, while several structural tides still blur important archetype
boundaries. The next revision should focus less on inventing new dreamcallers
and more on tightening tide ownership so dreamcaller packages stop relying on
the same generic rescue kit.
