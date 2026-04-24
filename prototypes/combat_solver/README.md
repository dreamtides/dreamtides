# Combat Solver Prototype

Run commands from this directory:

```sh
go run ./cmd/combat-solver --scenario support --budget 100ms --rank 5
```

To solve a JSON board and write the solver result:

```sh
go run ./cmd/combat-solver --input board.json --json-out result.json
```

`--scenario` selects a generated board. Current generated scenarios include
`support`, `high-spark`, `sparse`, `full-stress`, and `mobility-mixed`.
`--input` loads a JSON board instead of a scenario. JSON input contains an
`active` player plus `player_one` and `player_two` arrays of nine slots; each
occupied slot provides a character ID, card ID, optional name, optional stored
spark, and whether that character can reposition.

## Modeled Scope

This prototype models public in-play character combat positioning only. It has
no hands, card plays, Dreamwell, draw, stack responses, or events.

## Turn Sequence

The solver evaluates:

1. Current player repositions movable characters.
2. Current player's end-of-turn support gains resolve.
3. Opponent Judgment resolves.
4. Opponent chooses the worst-case reposition reply.
5. Opponent's end-of-turn support gains resolve.
6. Current player's next Judgment resolves.
