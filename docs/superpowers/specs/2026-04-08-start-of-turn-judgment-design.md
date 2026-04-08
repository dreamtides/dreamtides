# Start-of-Turn Judgment Design

## Goal

Restore a single start-of-turn `Judgment` phase and eliminate the `Dawn` name.
Front-rank combat should occur at the start of the active player's turn so the
rules can be described as "your front-rank characters attack at the start of
your turn."

## Problem

The current battle model resolves combat during `Judgment` at the end of the
active player's turn, but the non-active player's front-rank characters are the
attackers. This is mechanically valid but unintuitive to describe and teach,
because a player's attack step happens during the opponent's turn. `Dawn` was
introduced as a separate start-of-turn trigger window, which fixed naming
conflicts but added another phase term that players have to learn.

## Decision

Dreamtides should return to a start-of-turn `Judgment` phase and remove `Dawn`
from rules terminology, parser directives, strings, and card text.

## Turn Structure

The turn order becomes:

`Judgment -> Dreamwell -> Draw -> Main -> Ending`

`Judgment` is the first phase of the active player's turn. There is no separate
`Dawn` phase.

End-of-turn triggers remain separate and still occur after `Ending`.

## Judgment Phase Behavior

`Judgment` contains two substeps in a fixed order:

1. Start-of-turn `Judgment` triggers resolve for the active player.
2. Front-rank combat resolves with the active player as attacker and the
   opponent as blocker.

This preserves the old meaning of start-of-turn triggered abilities while making
combat happen on the same player's turn.

No fast-action window exists inside `Judgment`. Once the phase starts, triggered
effects and combat resolve without new card plays.

## Combat Model

The staggered battlefield and 4-lane front-rank combat rules remain unchanged
except for which player is considered the attacker.

- The active player's front-rank characters are attackers.
- The non-active player's front-rank characters are blockers.
- Each front-rank lane resolves independently.
- Unblocked attackers score points equal to spark.
- Paired attacker/blocker lanes compare spark and dissolve the weaker
  participant, or both on a tie.
- Surviving characters remain in place after combat.

Summoning-sickness behavior remains unchanged: characters played this turn still
cannot move to the front rank on that same turn, so front-rank attackers must
already be in place when the turn begins.

## Trigger Semantics

Current `Dawn` trigger semantics move back onto `Judgment`.

- `TriggerKeyword::Dawn` becomes `TriggerKeyword::Judgment`.
- `Trigger::Dawn(PlayerName)` becomes `Trigger::Judgment(PlayerName)` for
  start-of-turn card abilities.
- The separate post-combat `Trigger::Judgment(player)` emission is removed.

This avoids overloading the same trigger name with two different meanings. Cards
that currently read `{Dawn}` should read `{Judgment}` again and trigger at the
start of the controller's turn before combat.

## Rules Engine Changes

- Remove `BattleTurnPhase::Dawn`.
- Reorder the turn state machine so `Starting` transitions into `Judgment`.
- During `Judgment`, push the start-of-turn `Trigger::Judgment` event, resolve
  triggered effects and prompts, then iterate combat lanes.
- After `Judgment` finishes, transition to `Dreamwell`.
- Remove the current transition path `Draw -> Dawn -> Main`.
- Remove the post-combat `Trigger::Judgment` push that currently happens when
  combat finishes.

## Parser, Strings, and Data Changes

- Rename parser directives and serialized phrases from `dawn` back to
  `judgment`.
- Rename localized strings such as `dawn_phase_name` back to `judgment`.
- Update all card data in `client/Assets/StreamingAssets/Tabula/cards.toml` from
  `{Dawn}` to `{Judgment}`.
- Regenerate generated parser/card artifacts with `just tabula-generate`.

## Documentation Changes

- Update `docs/battle_rules/battle_rules.md` to describe start-of-turn
  `Judgment`.
- Remove `Dawn` terminology from player-facing rule explanations where it refers
  to the start-of-turn trigger window.
- Keep the explanation that end-of-turn triggers remain distinct from
  `Judgment`.

## Risks

- Some current cards may have been designed assuming `Dawn` happens after
  Dreamwell and Draw. Moving them to the first phase of the turn means they
  resolve before new energy and the normal draw.
- Parser and localization changes touch a broad surface area and require careful
  regeneration to keep rendered text fixtures in sync.
- Existing tests may implicitly assume that `Trigger::Judgment` is a post-combat
  event or that `Dawn` exists as a standalone phase.

## Testing

The implementation should cover:

- Turn-order tests proving `Judgment -> Dreamwell -> Draw -> Main -> Ending`.
- Tests proving start-of-turn judgment abilities fire before combat.
- Tests proving the active player's front-rank characters attack during their
  own `Judgment`.
- Parser and round-trip tests proving `{Judgment}` replaces `{Dawn}` in card
  text and directives.
- Battle-rules doc updates and generated-data refresh validation via
  `just tabula-generate`, `just fmt`, and `just review`.
