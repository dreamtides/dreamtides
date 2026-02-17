Dreamtides is a roguelike deckbuilding game with a Rust rules engine
(`rules_engine/`) and Unity C# frontend (`client/`). Two major systems: battles
(card combat) and quest mode (overworld/map progression).

## Architecture

Each major system (battles, quests) has its own state/queries/mutations crate
layers. `core_data` and `rules_engine` (facade) are shared.

**Flow:** game actions → mutate game state → return commands (full UI state
snapshots).

Card data in TOML files; do NOT read `cards.toml` directly (too large). Test
cards in `test-cards.toml`, dreamwell in `dreamwell.toml`. Generated files
(`parsed_abilities.json`, `test_card.rs`) come from `just tabula-generate`;
regenerate after any TOML change.

## Rust Style

**Naming qualification — the #1 agent error:**
- Function calls: exactly ONE qualifier
- Struct/enum type names: ZERO qualifiers
- Enum values: ONE qualifier
- WRONG: `crate::zone_mutations::move_card::to_destination_zone()`
- WRONG: `to_destination_zone()`
- WRONG: `battle_state::BattleState`
- CORRECT: `move_card::to_destination_zone()`
- CORRECT: `BattleState`
- CORRECT: `Zone::Battlefield`

**Imports:** `crate::` not `super::`. All `use` at file top, never inside
function bodies. No `pub use`. Only module declarations in `mod.rs`/`lib.rs`.

**File item order:** private consts/statics, thread_local, public type aliases,
public constants, public traits, public structs/enums, public functions, then
all remaining private items.

**General:**
- Prefer inline expressions over `let` bindings
- Short doc comments on public items only; no inline comments
- Cargo.toml deps alphabetized: internal first, then external
- chumsky `select!` triggers unnested_or_patterns; use `#[expect()]` not
  `#[allow()]`

## Testing

Tests live in `rules_engine/tests/` as integration tests. NEVER write inline
`mod tests {}`. No test-only production code — test against the real public API.
Run one test with e.g. `just battle-test <NAME>`.

## Build Commands (always `just`, never raw `cargo`)

- `just fmt` — format (style_validator --fix + rustfmt)
- `just check` — type check
- `just clippy` — lint
- `just review` — full gate (~5 min, keep polling, don't restart)
- `just tabula-generate` — regenerate from TOML
- `just schema` — regenerate C# types from Rust

## Acceptance Checklist

After every task:
1. Follow logging conventions (`battle_trace!`)
2. `just fmt` then `just review` (allow ~5 min)
3. Commit with detailed description. Do not print a summary of changes.

## Vocabulary

Materialize: play a character.
Energy: resource to play cards.
Spark: character stat that earns victory points.
Dissolve: destroy a card.
Void: discard pile.
Reclaim: play from void.
Kindle: add spark to leftmost character.
Foresee N: look at top N of deck, reorder/void.
Prevent: counter a card.