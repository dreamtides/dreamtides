# Adding New Effects

This document describes the steps to implement a new battle effect. Effects are
defined in:

- `rules_engine/src/ability_data/src/effect.rs`
- `rules_engine/src/ability_data/src/standard_effect.rs`

The `Effect` struct is a wrapper that adds various configuration values
as well as supporting effect lists, while the `StandardEffect` struct is the
comprehensive enumeration of possible effects.

## Step 1: Verify your effect is present in StandardEffect

A large number of effects defined in `StandardEffect` but not yet implemented. If
the effect you want is missing, you will need to add it here first. The effect
payload can contain information about the effect, but try to keep this
lightweight. Effects that apply to other cards often have a "target" predicate
that describes how they apply: `target: Predicate`.

## Step 2: Verify your effect is recognized by the ability parser

All cards in Dreamtides get their ability definitions in code by parsing their
rules text using the ability parser. Effects are parsed by:

- `rules_engine/src/parser/src/effect_parser.rs`
- `rules_engine/src/parser/src/standard_effect_parser.rs`

A large number of effects are already supported here but not implemented, so
most likely no action is required. If your new effect is *not* supported by the
parser, you will need to implement parsing support for it.

Effect parsing has unit tests which live primarily in:

`rules_engine/tests/parser_tests/tests/parser_tests/effect_parsing_tests.rs`

# Step 3: Add logic to implement your effect

Effects are primarily applied by:

- `rules_engine/src/battle_mutations/src/effects/apply_effect.rs`
- `rules_engine/src/battle_mutations/src/effects/apply_standard_effect.rs`

The effect implementation should mutate the BattleState to apply whatever the
desired result of your effect is, optionally modifying some target. Each effect
implementation should live in a separate function in `apply_standard_effect.rs`.
If your effect implementation is more than 10 lines of codes, it must instead be
created in a separate file under the `battle_mutations/src/effects/` directory.

Please keep effect function definitions in `apply_standard_effect.rs` in the
file in alphabetical order.

- Battle State: `rules_engine/src/battle_state/src/battle/battle_state.rs`
- Player State: `rules_engine/src/battle_state/src/battle_player/battle_player_state.rs`
- Move Card: `rules_engine/src/battle_mutations/src/card_mutations/move_card.rs`
- Card State: `rules_engine/src/battle_state/src/battle/all_cards.rs`

# Step 4: Create a test card

In order to test an effect implementation, we use "test cards". Define a new
test card with your desired rules text, then use it to implement a unit test.
Test cards can be added using the `just` command runner as follows:

```
 just tabula-add-card --name "Name" --text "{Dissolve} an enemy character." --cost 2 --card-type "Event"
```

 This will add a new card to the card database in
 `client/Assets/StreamingAssets/tabula.json` named "Name" with the rules text
 "{Dissolve} an enemy character.", suitable for testing the "dissolve" effect.

 Don't directly open `tabula.json` since it's thousands of lines.

 After adding the test card, you must regenerate the tabula database by invoking

 ```
 just tabula
 ```

 This will cause new code to be generated containing your test card's ID. Test
 card ID constants are defined at:

 ```
 rules_engine/src/tabula_ids/src/test_card.rsk
 ```

 The test card ID can now be used.

 # Step 5: Define a new test for your effect

Tests for effects live in the `tests/battle_tests/effect_tests/` directory. Look
at some example tests for reference:

- `rules_engine/tests/battle_tests/tests/battle_tests/effect_tests/dissolve_effect_tests.rs`

Each effect has tests in its own file. A very simple test for an effect would
look like this:

```
let mut s = TestBattle::builder().connect();
let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);
s.create_and_play(DisplayPlayer::User, test_card::TEST_DISSOLVE);
```

This is creating a new test battle and connecting to it to produce a test
session `s`. We add a new enemy character to the battlefield to act as the
target for our "dissolve" effect. We then create the "Dissolve" card in the
player's hand and play it. By default, cards automatically pick their target if
only one option is available, so no further action is required. If there were
multiple characters in play, we would get prompted to pick a target and we would
need to click on the character to target with the "dissolve" effect.

This is making use of the test utilities present in the test client and test
session extension here:

- `rules_engine/src/test_utils/src/client/test_client.rs`
- `rules_engine/src/test_utils/src/session/test_session_battle_extension.rs`

# Step 6: Run your test

You can run a newly created test using `just` as follows, from any project
directory:

```
just battle-test battle_tests::effect_tests::dissolve_effect_tests::dissolve_enemy_character
```

 Using the path to the test from `battle_tests`. You can also run all of the
 tests for a given file:

```
just battle-test battle_tests::effect_tests::dissolve_effect_tests
```

# Step 7: Validate changes

After you test passes, you should run `just fmt` to apply rustfmt formatting
rules to the project codebase. You should then run Clippy and the full project unit test
suite by invoking `just review`.

# Appendix: Code Style

General project code style rules apply to effect definitions:

- Function calls and enum values have exactly one `::` qualifier, enum names and
  struct names have zero `::` qualifiers.
  - `my_module::function()`
  - `MyEnum::Value`
  - `v: MyEnum`
  - `s: MyStruct`
- The order of declarations in a file should always be:
  - Public Structs
  - Public Functions
  - Private Structs
  - Private Functions
- In general the "main" or "most important" definitions for a file are higher up
  in the file.
- In general, avoid nesting declarations. For example avoid putting a struct or
  function declaration within the body of a function.

# Appendix: Coding Environment

This project uses the Rust programming language. If this is not available, you
can install it using the `rustup` tool following the instructions on
`https://www.rust-lang.org/tools/install`. For example, on a Unix OS:

- `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

This project uses `just` to run commands. Just commands can be run from any
working directory within the project and will always have the same result. If
`just` is not available, after installing `cargo` via `rustup` you can get it
via:

- `cargo install just`