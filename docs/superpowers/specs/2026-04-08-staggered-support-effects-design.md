# Staggered Support Effects Design

Add two prototype-only card effects to the staggered-grid combat prototype:
`Duskborne Sentry` grants passive spark to the front-row characters it supports,
and `Veilward Knight` grants permanent spark at end of turn to the back-row
characters supporting it.

## Scope

This is a prototype implementation for the current staggered-grid battle flow.
It does not go through the full ability parser pipeline and does not add
general-purpose rules text plumbing. The goal is to make the effects real in
the engine and visible in the battle prototype for playtesting.

## Rules Semantics

### Duskborne Sentry

`Duskborne Sentry` has the static effect: "Supported Characters gain +2 spark."

- The effect applies only while Duskborne Sentry is on the battlefield in the
  back rank.
- A front-row character gets the bonus if its slot is supported by the
  Sentry's back-row slot under the staggered-grid topology.
- The bonus is derived, not stored. It should affect all uses of spark,
  including judgment comparisons, scoring, AI evaluation, and rendered combat
  stats.
- Multiple supporting Sentries stack. A character supported by two Sentries
  gains `+4 spark`.

### Veilward Knight

`Veilward Knight` has the triggered effect: "At the end of turn, supporting
characters gain +1 spark."

- The effect applies only while Veilward Knight is on the battlefield in the
  front rank.
- At the end of turn, each Knight grants `+1 spark` to each occupied back-row
  slot that supports the Knight's front-row slot under the staggered-grid
  topology.
- The gain is permanent battlefield spark, using the existing spark mutation
  path.
- Multiple Knights stack. A back-row character behind two Knights gains `+2
  spark` from their end-of-turn triggers.

## Support Topology

The implementation must use the existing staggered-grid support helpers in
`Battlefield`:

- `Battlefield::supported_front_slots(back_slot, back_size)` for effects flowing
  from back to front
- `Battlefield::supporting_back_slots(front_slot, front_size)` for effects
  flowing from front to back

That keeps the effect logic aligned with the current 5-back/4-front prototype
geometry, including the one-supporter edges and two-supporter middle lanes.

## Engine Integration

### Derived Spark Hook

The authoritative read path for battlefield spark is `AllCards::spark`. That is
the correct seam for `Duskborne Sentry` because judgment, scoring, AI, and
display already read through that function or its query wrapper.

`AllCards::spark` should:

- read the stored battlefield spark
- detect the battlefield slot occupied by the character
- if the character is in the front rank, count supporting back-rank Duskborne
  Sentries and add `+2` per Sentry
- return the derived total spark

This avoids adding special handling at every consumer.

### End-of-Turn Trigger Hook

The authoritative end-of-turn transition is the
`EndingPhaseFinished -> FiringEndOfTurnTriggers` branch in
`phase_mutations/turn.rs`.

Before the normal generic end-of-turn trigger queue resolves, the engine should:

- scan the active player's front rank for Veilward Knights
- resolve each Knight independently
- look up the 1-2 supporting back slots for that Knight's lane
- apply `spark::gain(..., Spark(1))` to each occupied supported back-row
  character

Using the existing spark mutation path ensures animations and any downstream
spark consumers remain consistent with other spark gains.

## Prototype UI Expectations

The prototype should expose the effects through the existing rendered spark
values without requiring custom explanatory UI for this spike. If the engine
returns the correct spark values, the current battle prototype should show the
buffed totals where it already renders spark.

If the battle log or action text does not make the source of the gain obvious,
that is acceptable for this prototype as long as the board state is visually
correct and manual QA can verify the effect.

## Error Handling and Constraints

- No parser changes
- No TOML ability regeneration workflow
- No integration tests required for this prototype
- Limit the implementation to card-name-based prototype hooks for these two
  cards only
- Do not introduce a general new ability system unless the existing seams prove
  insufficient

## Manual QA

Validation should be manual in the battle prototype using the `qa` skill and
`agent-browser` CLI.

QA should confirm:

- Duskborne Sentry grants `+2 spark` to exactly the supported front-row
  characters
- edge slots affect one front target and middle slots affect two
- overlapping Sentries stack correctly
- Veilward Knight grants `+1 spark` at end of turn to exactly the supporting
  back-row characters
- overlapping Knights stack correctly
- the derived or gained spark values affect real judgment outcomes and scoring,
  not just rendering

## Out of Scope

- Adding reusable parser-backed support ability syntax
- Localized rules text updates
- Automated tests
- Non-prototype cards using the new support relationships
