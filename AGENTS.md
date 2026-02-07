# PROJECT CONTEXT


This project is an implementation of a rules engine for the 'Dreamtides'
card game. Dreamtides is a game similar to TCGs like 'Magic: the Gathering'.
Players put 'character' cards into play and use one-time 'event' cards to affect
the game. Cards are played using a resource called 'energy'. Characters have a
value called 'spark' that lets them generate victory 'points'. Playing a character
is called 'materializing' it and destroying it is called 'dissolving' it.
Dissolved characters and played events go the discard pile, which is called the
'void'. Playing one game or match of Dreamtides is known as a 'battle'.

A few relevant keywords:

- Reclaim: Play from void (discard pile)
- Kindle: Add spark to your leftmost character
- Foresee: Look at top N of deck and reorder or put in void
- Prevent: Stop a card from resolving (counterspell)


# ACCEPTANCE CRITERIA


Please follow this checklist after completing any task:

1) Add error handling where appropriate. Consider edge cases.
2) Add tests where appropriate. Rust tests live in rules_engine/tests/ and
   always test crate public APIs. Test error cases.
3) Add logging where appropriate. Follow existing logging conventions.
4) Run `just fmt` to apply formatting rules.
5) Run `just review` to run clippy, validate style, and run unit tests
6) Create a git commit with a detailed description of your work. 


# CODE STYLE


- Prefer writing code inline (when possible) to creating new variables via "let" statements
- Add a short doc comment to top-level public functions, fields, and types. Don't add inline comments.
- DO NOT fully-qualify names in code ever
- Function calls and enum values have exactly one qualifier, struct names and enum types have
  zero qualifiers:
  - WRONG FUNCTION CALL: crate::zone_mutations::move_card::to_destination_zone()
  - WRONG FUNCTION CALL: to_destination_zone()
  - WRONG ENUM VALUE: battle_state::battle_cards::zone::Zone::Battlefield
  - WRONG STRUCT NAME: battle_state::BattleState
  - CORRECT ENUM VALUE: Zone::Battlefield
  - CORRECT FUNCTION CALL: move_card::to_destination_zone()
  - CORRECT STRUCT NAME: BattleState
- Items in a file must follow this order: private constants, private statics,
  thread_local declarations, public type aliases, public constants, public traits,
  public structs/enums, public functions, then all other private items.
- Don't add "pub use" for anything.
- Keep Cargo.toml dependencies alphabetized in two lists, internal then external dependencies.
- Use modern Rust features such as let-else statements and "{inline:?}" variable formatting
- Do not add code to `mod.rs` or `lib.rs` files except for module declarations
- Do not add `use` declarations within function bodies, only place them at the top of files
- Qualify imports via `crate::`, not via `super::`
- Do not write inline `mod tests {` tests, place them in the `/tests/` directory
- Do not write code only used by tests. Test against real public API.


# JUST COMMANDS


- After completing work, please always run "just fmt" to apply rustfmt
  formatting rules
- Please use `just` commands instead of `cargo`, e.g. `just fmt`, `just check`,
  `just-clippy`, `just-test`, `just parser-test`
- Run `just check` to type check code
- Run `just clippy` to check for lint warnings
- After writing a test, use `just battle-test <TEST NAME>` to run it
- After completing work, please ALWAYS run `just review` to validate changes
- Do not print a summary of changes after completing work.
- Prefer the `just` commands over `cargo` commands since they have project-specific rules


# CODE STRUCTURE


The code is structured as a series of Rust crates using the cargo "workspace" feature.

Dreamtides:
  - `justfile`
  - `rules_engine/`
    - 'Cargo.toml`
  - `client/

Rules engine Rust source code lives in the `rules_engine/` directory.
Client source code lives in the `client/` directory.

Card data lives in `rules_engine/tabula/cards.toml`. Do NOT read this file directly, it is much too large.
