Please follow all of the "code style" and "validating changes" rules below at
all times.


# SETUP


- This project uses Rust and Cargo, which can be installed via `rustup`
    - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- This project uses the `just` command runner
    - `cargo install just`
  - A `just` command can be run from any directory of the project and will have the same result
- This project uses `cargo-workspace-lints`
    - `cargo install cargo-workspace-lints`
- If you want to be able to run benchmarks, you will need valgrind:
    - `sudo apt-get install valgrind`


# CODE STYLE


- Prefer writing code inline (when possible) to creating new variables via "let" statements
- DO NOT add inline comments to code
- DO NOT delete existing inline comments
- DO NOT fully-qualify names in code ever
- Function calls and enum values have exactly one qualifier, struct names have
  zero qualifiers:
  - WRONG FUNCTION CALL: crate::zone_mutations::move_card::to_destination_zone()
  - WRONG FUNCTION CALL: to_destination_zone()
  - WRONG ENUM VALUE: battle_state::battle_cards::zone::Zone::Battlefield
  - WRONG STRUCT NAME: battle_state::BattleState
  - CORRECT ENUM VALUE: Zone::Battlefield
  - CORRECT FUNCTION CALL: move_card::to_destination_zone()
  - CORRECT STRUCT NAME: BattleState
- Public functions and types go at the TOP of the file, then private ones.
- Don't add "pub use" for anything.
- Keep Cargo.toml dependencies alphabetized in two lists, internal then external dependencies.
- Use modern Rust features such as if-let statements and "{inline:?}" variable formatting
- Do not add code to `mod.rs` or `lib.rs` files except for module declarations


# VALIDATING CHANGES


- After completing work, please always run "just fmt" to apply rustfmt
  formatting rules
- Run `just check` to type check code
- Run `just clippy` to check for lint warnings
- After writing a test, use `just battle-test <TEST NAME>` to run it
- After completing work, please ALWAYS run `just review` to validate changes
- Do not print a summary of changes after completing work.
- Prefer the `just` commands over `cargo` commands since they have project-specific rules


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


# ADDITIONAL DOCUMENTATION

More task-specification documentation is available:

- Development environment setup: rules_engine/docs/environment_setup.md
- Adding new battle effects: rules_engine/docs/adding_new_effects.md
- Adding new trigger conditions: rules_engine/docs/adding_new_triggers.md
- Running benchmarks: rules_engine/docs/benchmarks.md


# CODE STRUCTURE

The code is structured as a series of Rust crates using the cargo "workspace" feature.

Rules engine source code lives in the rules_engine/ directory.
