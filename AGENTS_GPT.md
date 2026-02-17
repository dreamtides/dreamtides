# Dreamtides Agent Guide (GPT)

Use this as the default orientation for AI agents working in this repo.
Prioritize correctness, performance, and adherence to project conventions.

## Game And Domain Basics

Dreamtides is a TCG-like game.
- Characters are put into play by "materializing" them.
- Events are one-shot cards.
- Energy pays costs.
- Spark on characters generates victory points.
- Destroyed characters and played events go to the void (discard pile).
- A full game/match is a battle.

Key keywords:
- Reclaim: play from void.
- Kindle: add spark to your leftmost character.
- Foresee: inspect top N cards of deck, reorder and/or send to void.
- Prevent: counter a card before resolution.

## Architecture You Must Understand

Dreamtides is split:
- Client: Unity/C# (`client/`).
- Rules engine: Rust workspace (`rules_engine/`).

Core runtime flow:
- Client sends JSON-serialized game actions.
- Engine entrypoint routes actions and mutates authoritative game state.
- Engine returns display commands that describe UI state.
- Prefer React-style full-state description over imperative UI mutations.

High-value engine concepts:
- `BattleState` is performance critical.
- Battle card count is capped at 128; card sets are represented with `u128`.
- Mutations belong in `battle_mutations`; reads belong in `battle_queries`.
- Logging is important; `battle_trace!` is a key mechanism.
- UI/localized text should use RLF strings.

Data and parsing:
- Card definitions come from Tabula TOML data.
- Rules text is parsed into ability data used by the engine.
- Do not read `rules_engine/tabula/cards.toml` directly (too large).

## Testing And Quality Bar

Testing philosophy is mostly black-box integration behavior.
- For Rust rules engine changes, add integration tests in
  `rules_engine/tests/`.
- Avoid test-only production code.
- Test both success and error paths where relevant.

Always do these after changes:
1. Add edge-case error handling where needed.
2. Add logging where it clarifies behavior and debugging.
3. Run `just fmt`.
4. Run `just review` (use long waits; it may be quiet for minutes).

Prefer `just` commands over raw `cargo` commands.

## Non-Negotiable Rust Style Rules

These are common agent failure points:
- No fully-qualified calls or enum values in code.
- Function calls and enum values use exactly one qualifier.
- Struct names and enum type names use zero qualifiers.
- Do not put `use` inside function bodies.
- Qualify imports through `crate::`, not `super::`.
- Do not add code to `mod.rs` or `lib.rs` except module declarations.
- Do not write inline `mod tests`; use integration tests in `/tests/`.
- Do not add `pub use` re-exports.
- Add short doc comments to top-level public items.
- Prefer inline expressions over unnecessary temporary `let` bindings.
- Keep file item order:
  private consts/statics/thread_local, public aliases/constants/traits,
  public structs/enums/functions, then private items.
- Keep `Cargo.toml` dependencies alphabetized in two blocks:
  internal deps first, external deps second.

## Practical Workflow

Before editing, identify the target crate and boundary (mutation/query/display).
When changing behavior, validate the action -> state -> command pipeline.
When unsure, favor minimal, composable changes that preserve engine
performance.
