# DREAMTIDES AGENT GUIDE

Dreamtides is a roguelike deckbuilder card game with a Rust rules engine backend
and Unity3D C# frontend. Players materialize characters (putting them into play)
and play one-time events using energy. Characters have spark which earns victory
points during the judgment phase. Dissolved characters and played events go to
the void (discard pile). A match is called a battle; first to 12 points wins.

## Architecture

The rules engine (~34 Rust crates in rules_engine/) communicates with Unity via
JSON. The core loop: Unity sends GameActions to engine.rs, which mutates
BattleState and returns Commands describing full UI state snapshots. Critical
constraint: max 128 cards per battle (u128 bitsets).

Key crate layers: core_data (IDs, newtypes) -> ability_data (card effects AST)
-> battle_state (central hub) -> battle_queries (reads) -> battle_mutations
(writes) -> display (rendering) -> rules_engine (facade).

Card data lives in TOML files (rules_engine/tabula/cards.toml — too large to
read). The `tabula generate` command parses card rules text through lexer ->
variable resolver -> chumsky parser -> JSON output. Generated files:
parsed_abilities.json and test_card.rs. Regenerate after any TOML changes. Test
cards in test-cards.toml; dreamwell cards in dreamwell.toml and
test-dreamwell.toml.

## Code Style (Common Agent Mistakes)

Naming qualification — the single most common style error:
- Function calls use EXACTLY ONE qualifier: `move_card::to_zone()`
- Struct names use ZERO qualifiers: `BattleState` not `battle::BattleState`
- Enum values use ONE qualifier: `Zone::Battlefield`
- NEVER fully-qualify: `crate::foo::bar::baz()` is always wrong
- NEVER use bare function names to external functions: `to_zone()` is wrong

File ordering (enforced by style_validator): private constants/statics,
thread_local, public type aliases, public constants, public traits, public
structs/enums, public functions, all other private items.

Import rules:
- Use `crate::` not `super::` for imports
- Place all `use` declarations at file top, never inside function bodies
- Never add `pub use` for anything
- Only put module declarations in mod.rs and lib.rs, no other code

General style:
- Prefer inline expressions over `let` bindings where readable
- Add short doc comments to public functions, fields, and types only
- No inline comments in code
- Use modern Rust: let-else, `"{inline:?}"` formatting
- Keep Cargo.toml deps alphabetized: internal deps first, then external
- The chumsky `select!` macro triggers clippy::unnested_or_patterns; suppress
  with `#[expect(clippy::unnested_or_patterns)]` not `#[allow]`

## Testing

Tests live in rules_engine/tests/ as integration tests. NEVER write inline `mod
tests {}` blocks. Do not write code only used by tests — test against the real
public API. TestBattle builds configurations; TestSession drives actions through
the full engine pipeline. Tests operate at the simulated UI level via
`perform_user_action()`.

Run a specific test: `just battle-test <NAME>`

## Build Commands

Always use `just` commands, never raw `cargo`:
- `just fmt` — format code (runs style_validator --fix, rlf-fmt, rustfmt)
- `just check` — type check
- `just clippy` — lint
- `just test` — all tests
- `just review` — full validation gate (build + clippy + style + tests) Takes ~5
  minutes. Run with timeout, keep polling, don't restart.
- `just tabula-generate` — regenerate parsed_abilities.json
- `just schema` — regenerate C# types from Rust

## Acceptance Checklist

After completing any task:
1. Add error handling for edge cases where appropriate
2. Add integration tests in rules_engine/tests/ if appropriate
3. Follow existing logging conventions (battle_trace! macro)
4. Run `just fmt`
5. Run `just review` (allow ~5 min, don't assume it hung)
6. Commit with a detailed description

## Key Subsystems

RLF localization: All UI strings defined in rules_engine/src/strings/ via the
`rlf!` macro. Serializers convert Ability AST back to display text using RLF
phrase functions. Masonry UI builds FlexNode trees for overlay UI, reconciled
via virtual-DOM diffing. AI uses Monte Carlo Tree Search (ai_uct).

## Game Terms

Reclaim: play from void. Kindle: add spark to leftmost character. Foresee N:
look at top N of deck, reorder/void. Prevent: counter.