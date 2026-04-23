# Deterministic Combat Solver Design

## Goal

Build a standalone Go CLI prototype that solves Dreamtides combat positioning
for Core 11 public board states. The solver answers one tactical question: given
the current player's Main phase, where should that player place their characters
before ending the turn if both players can only reposition characters and the
opponent chooses the worst-case reply?

The prototype is intended for balance exploration and representative board
analysis. It lives under a new top-level `prototypes/` directory and does not
integrate with the Rust rules engine or Unity client.

## Scope

The solver models only Core 11 characters already in play. It ignores card
plays, hidden hands, Dreamwell, draw, activated abilities, stack responses, and
Core 11 events. Events may appear in the loaded card list, but scenario
generation and solving filter them out.

The modeled turn sequence is:

1. Current player chooses a legal repositioning during their Main phase.
2. Current player's end-of-turn support gains resolve.
3. Opponent's Judgment resolves.
4. Opponent chooses a legal worst-case repositioning during their Main phase.
5. Opponent's end-of-turn support gains resolve.
6. Current player's next Judgment resolves.
7. The final board is scored from the current player's perspective.

The first implementation supports any partial board with 0-9 characters per
player. Each character has an owner, card identity, stored spark, battlefield
slot, and `can_reposition` flag. Unmovable characters remain fixed; movable
characters can be assigned to any free legal slot on their own battlefield.

## Core 11 Rules Modeled

The CLI loads Core 11 data at runtime from:

- `rules_engine/tabula/card-lists.toml`
- `rules_engine/tabula/rendered-cards.toml`

It filters to character cards and keeps each card's ID, name, base spark, and
rendered text. The solver recognizes the current Core 11 prototype support
abilities:

- `Nocturne Strummer`: while in the back rank, supported front-rank characters
  gain `+2` effective spark. This is a derived spark bonus and stacks across
  multiple Strummers.
- `Runebound Champion`: while in the front rank, at end of turn each occupied
  supporting back-rank character gains `+1` stored spark. Multiple Champions
  stack.

Support topology follows the documented 4-front, 5-back staggered grid:

- `B0` supports `F0`
- `B1` supports `F0` and `F1`
- `B2` supports `F1` and `F2`
- `B3` supports `F2` and `F3`
- `B4` supports `F3`

Judgment follows the battle rules: active front-rank characters attack in lanes
`F0-F3`; non-active front-rank characters block in matching lanes; paired
characters compare effective spark and dissolve the weaker character, with ties
dissolving both; unblocked attackers score points equal to effective spark.

## CLI Behavior

The CLI supports generated and file-backed inputs.

Generated scenarios are representative Core 11 board states for balance review:
support-heavy positions, high-spark combat positions, sparse boards, full-board
stress cases, and asymmetric mobility cases. Generated scenarios default
characters to movable unless the scenario is specifically testing immobility.

JSON input supports custom or captured board states with:

- active player
- each player's 9 battlefield slots
- character card ID or name
- stored spark override
- `can_reposition`

Outputs are terminal-first:

- ASCII board diagrams before and after the recommended line
- ranked root placements
- the opponent's worst-case reply for the chosen placement
- score tuple components
- elapsed time
- `complete: true` if the search evaluated the full relevant tree, otherwise
  `complete: false`

The CLI may also emit machine-readable JSON result files for regression tests
and later tooling. SVG, PNG, and browser output are out of scope for the first
version.

## Solver Architecture

The solver uses exact two-ply minimax as the design center.

For each legal current-player placement:

1. Apply the placement to a copied board.
2. Resolve current-player end-of-turn support gains.
3. Resolve opponent Judgment.
4. Enumerate legal opponent placements from the resulting board.
5. For each opponent placement, resolve opponent end-of-turn support gains and
   current-player Judgment.
6. Score the final state from the current player's perspective.
7. Keep the opponent reply that minimizes that score.

The selected root placement is the one whose worst-case reply has the best
score.

The board representation should stay compact and pure:

- two fixed 9-slot arrays, one per player
- one character table containing identity, owner, stored spark, and mobility
- pure functions for legal placement generation, effective spark, end-of-turn
  support gains, Judgment, scoring, and board rendering

This keeps the solver deterministic and testable without depending on Rust or
Unity runtime state.

## Objective Function

The objective is strict lexicographic comparison, not a weighted score. From the
current player's perspective, maximize:

1. Final surviving allied characters' effective spark multiset, sorted
   descending.
2. Opponent characters dissolved across the two Judgment phases, using their
   spark at the moment they dissolved, sorted descending.
3. Current player's points scored across the simulated line.
4. Negative opponent points scored across the simulated line.

Final survival uses effective spark in the final board position, so static
support from `Nocturne Strummer` matters. Ties are broken deterministically by
slot order and stable character identity.

## Time Budget And Pruning

The target budget is 100ms per solve. The solver attempts exact completion first
and marks the result as complete only when all relevant root placements and
opponent replies were evaluated.

If the budget is reached, the solver returns the best fully evaluated root
candidate so far and reports `complete: false`. Candidate ordering is
deterministic so repeated runs over the same input produce stable partial
results.

Initial pruning should include:

- duplicate placement elimination for equivalent characters with the same card,
  stored spark, mobility, and support-relevant identity
- dominance checks for placements with the same front layout and weaker support
  layout
- branch-and-bound upper bounds using the maximum remaining possible survival,
  dissolve, and point outcomes

The design does not require beam search in the first implementation. Beam search
can be added later only if exact minimax plus pruning cannot produce useful
results within budget on representative full-board cases.

## Testing

Go unit tests cover the pure rules and solver functions:

- staggered support topology
- `Nocturne Strummer` effective spark
- `Runebound Champion` end-of-turn gains
- lane Judgment outcomes
- partial-board legal placement generation
- unmovable character constraints
- lexicographic score comparison
- minimax opponent choice

CLI or golden tests should cover at least one generated scenario and one JSON
input scenario. A full 9-character movable stress case should verify budget
behavior and the `complete` flag rather than requiring every environment to
finish an exhaustive tree inside 100ms.

Validation against the Rust engine is indirect in this prototype. The Go
transition functions are expected to match `docs/battle_rules/battle_rules.md`
and the current Core 11 support hooks in the Rust code, but the CLI does not
call Rust.

## Risks And Follow-Ups

The main correctness risk is drift between the Go prototype and the Rust rules
engine. Runtime TOML loading reduces card-data drift, but transition rules are
still duplicated. Keeping the Go model narrow and heavily tested is the intended
mitigation.

The main performance risk is the two-ply state tree on full, fully movable
boards. The first version should prioritize determinism, instrumentation, and
clear completeness reporting over forcing heuristic answers to look optimal.

Future follow-ups may include captured-state import from battle logs, JSON
golden fixtures generated from Rust engine states, richer scenario generation,
and optional visual output once the terminal workflow proves useful.
