# Adding New Triggers

This document describes the steps to implement new trigger functionality for
the Dreamtides battle system. Triggers are events that occur during a battle
which can activate triggered abilities on character cards. The trigger system
is defined in:

- `rules_engine/src/battle_state/src/triggers/trigger.rs`
- `rules_engine/src/ability_data/src/trigger_event.rs`
- `rules_engine/src/ability_data/src/triggered_ability.rs`

The `Trigger` enum represents specific event instances that occur during battle,
while the `TriggerEvent` enum represents conditions that triggered abilities
can listen for. The `TriggerName` enum provides a name-only version of triggers
for efficient matching.

Please implement the described new trigger and then add comprehensive unit
tests for this functionality. Please think carefully and make a plan for how
to achieve this. Review all of the instructions in this document thoroughly
before starting.

Please create a TODO list before starting with one entry per step or sub-step
in this document.

## Understanding the Trigger System

### Trigger and TriggerName Structs

The trigger system consists of two main enum types:

1. **`Trigger`** - Represents specific events that occur during battle with their
   associated data:
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
   pub enum Trigger {
       Abandonded(VoidCardId),
       Banished(VoidCardId),
       Discarded(VoidCardId),
       Dissolved(VoidCardId),
       DrewAllCardsInCopyOfDeck(PlayerName),
       EndOfTurn(PlayerName),
       GainedEnergy(PlayerName, Energy),
       Judgment(PlayerName),
       Materialized(CharacterId),
       PlayedCard(StackCardId),
       PlayedCardFromHand(StackCardId),
       PlayedCardFromVoid(StackCardId),
   }
   ```

2. **`TriggerName`** - Represents trigger types without data for efficient matching:
   ```rust
   #[derive(EnumSetType, Debug)]
   pub enum TriggerName {
       Abandonded,
       Banished,
       Discarded,
       Dissolved,
       DrewAllCardsInCopyOfDeck,
       EndOfTurn,
       GainedEnergy,
       Judgment,
       Materialized,
       PlayedCard,
       PlayedCardFromHand,
       PlayedCardFromVoid,
   }
   ```

The `Trigger` struct provides a `name()` method that returns the corresponding
`TriggerName` for efficient categorization and matching.

### How Triggers Work

1. **Firing Triggers** - When events occur during battle, the game engine fires
   triggers by adding them to the `battle.triggers` queue
2. **Listening** - Characters with triggered abilities automatically listen for
   relevant trigger types when they enter the battlefield
3. **Resolution** - After each battle action, if no prompts are active, all
   pending triggers are resolved in first-in-first-out order

## How to Fire Triggers from Code

Triggers are fired throughout the game engine when specific events occur. Here
are examples from `play_card.rs`:

### Example: Playing a Card from Hand
```rust
// After moving a card from hand to stack
battle.triggers.push(source, Trigger::PlayedCard(stack_card_id));
battle.triggers.push(source, Trigger::PlayedCardFromHand(stack_card_id));
```

### Example: Playing a Card from Void
```rust
// After moving a card from void to stack
battle.triggers.push(source, Trigger::PlayedCard(stack_card_id));
battle.triggers.push(source, Trigger::PlayedCardFromVoid(stack_card_id));
```

### Example: Materializing a Character
```rust
// When a character enters the battlefield
battle.triggers.push(source, Trigger::Materialized(character_id));
```

### Example: Gaining Energy
```rust
// When a player gains energy
battle.triggers.push(source, Trigger::GainedEnergy(player, amount));
```

The general pattern is:
```rust
battle.triggers.push(source, Trigger::TriggerVariant(associated_data));
```

Where:
- `source` is an `EffectSource` indicating what caused the trigger
- The trigger variant matches the type of event that occurred
- Associated data provides context (card IDs, player names, amounts, etc.)

## Step 1: Verify your trigger is present in the Trigger enum

Before implementing new trigger functionality, check if your desired trigger
already exists in the `Trigger` enum in
`rules_engine/src/battle_state/src/triggers/trigger.rs`.

If your trigger is missing, you will need to add it to both the `Trigger` enum
and the `TriggerName` enum, along with updating the `name()` method to handle
the new variant.

## Step 2: Verify your trigger event is recognized by the ability parser

Triggered abilities on cards are parsed from rules text using the ability
parser. Trigger events are parsed by:

- `rules_engine/src/parser/src/trigger_event_parser.rs`
- `rules_engine/src/parser/src/triggered_ability_parser.rs`

Most trigger events are already supported by the parser. If your trigger event
is *not* supported, you will need to implement parsing support for it in the
`trigger_event_parser.rs` file.

Trigger parsing has unit tests which live in:

`rules_engine/tests/parser_tests/tests/parser_tests/triggered_ability_parsing_tests.rs`

## Step 3: Add logic to fire your trigger

### Step 3A: Finding where to fire the trigger

Triggers should be fired whenever the corresponding game event occurs. Common
locations include:

- **Card movement**: `rules_engine/src/battle_mutations/src/card_mutations/move_card.rs`
- **Playing cards**: `rules_engine/src/battle_mutations/src/play_cards/play_card.rs`
- **Effect application**: `rules_engine/src/battle_mutations/src/effects/apply_standard_effect.rs`
- **Turn phases**: `rules_engine/src/battle_mutations/src/phase_mutations/`

### Step 3B: Implementing the trigger

Add the trigger at the appropriate location using the pattern:

```rust
battle.triggers.push(source, Trigger::YourTriggerName(relevant_data));
```

Make sure to:
- Use the correct `EffectSource` for the context
- Include relevant data (card IDs, player names, amounts, etc.)
- Fire the trigger at the right time in the sequence of events

### Step 3C: Handling trigger resolution

Trigger resolution is handled automatically by the system in
`rules_engine/src/battle_mutations/src/phase_mutations/fire_triggers.rs`. No
additional code is typically needed unless you have special resolution
requirements.

## Step 4: Create a test card

In order to test a trigger implementation, we use "test cards". Define a new
test card with a triggered ability that responds to your trigger, then use it
to implement a unit test. Test cards can be added using the `just` command
runner as follows:

```
just tabula-add-card --name "Test Trigger Response" --text "Whenever an enemy character is {dissolved}, draw {-drawn-cards(n:1)}." --cost 0 --card-type "Character" --spark 1
```

This will add a new card to the card database in
`client/Assets/StreamingAssets/tabula.json` with a triggered ability suitable
for testing the "dissolved" trigger. Don't directly open `tabula.json` since
it's thousands of lines. By convention, test card names are short and start
with the word "test".

This will cause new code to be generated containing your test card's ID. Test
card ID constants are defined at:

```
rules_engine/src/tabula_ids/src/test_card.rs
```

for example:

```rust
/// Whenever an enemy character is {dissolved}, draw {-drawn-cards(n:1)}.
pub const TEST_TRIGGER_RESPONSE: BaseCardId = BaseCardId(uuid!("d4854b6e-5274-4f6a-8a60-a1ea1c15e9a6"));
```

The test card ID can now be used in tests.

You should *always* create a new test card for each trigger test. Do not use
existing card definitions. Do not use production cards in your test.

## Step 5: Define a new test for your trigger

Tests for triggers live in the `tests/battle_tests/basic_tests/` directory.
Look at some example tests for reference:

- `rules_engine/tests/battle_tests/tests/battle_tests/basic_tests/triggered_ability_tests.rs`

Each trigger functionality should have tests in an appropriate file. A simple
test for a trigger would look like this:

```rust
#[test]
fn trigger_fires_when_event_occurs() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    // Add a character with a triggered ability that responds to our trigger
    let triggered_character_id = s.create_and_play(
        DisplayPlayer::User,
        test_card::TEST_TRIGGER_RESPONSE,
    );

    // Create initial conditions for the test
    let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

    // Perform the action that should fire the trigger
    s.create_and_play(DisplayPlayer::User, test_card::TEST_DISSOLVE);

    // Verify the triggered ability fired correctly
    assert_eq!(s.user_client.cards.user_hand().len(), 1, "should have drawn a card");
}
```

This is making use of the test utilities present in the test client and test
session extension here:

- `rules_engine/src/test_utils/src/client/test_client.rs`
- `rules_engine/src/test_utils/src/session/test_session_battle_extension.rs`

## Step 6: Verify trigger is recognized by the parser

Before writing battle tests, verify that your trigger event is correctly parsed
by adding a parser test to:

`rules_engine/tests/parser_tests/tests/parser_tests/triggered_ability_parsing_tests.rs`

A simple parser test looks like:

```rust
#[test]
fn test_your_trigger_parsing() {
    let result = parse(
        "Whenever an enemy character is {dissolved}, draw {-drawn-cards(n: 1)}.",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Dissolved(Enemy(CharacterType(Any))),
        effect: Effect(DrawCards(
          count: 1,
        )),
      )),
    ]
    "###
    );
}
```

This test verifies that the rules text is correctly parsed into the expected
`TriggeredAbility` structure.

## Step 7: Run your tests

You can run a newly created test using `just` as follows, from any project
directory:

```
just battle-test battle_tests::basic_tests::triggered_ability_tests::trigger_fires_when_event_occurs
```

Using the path to the test from `battle_tests`. You can also run all of the
tests for a given file:

```
just battle-test battle_tests::basic_tests::triggered_ability_tests
```

For parser tests:

```
just parser-test parser_tests::triggered_ability_parsing_tests::test_your_trigger_parsing
```

## Step 8: Validate changes

After your tests pass, you should run `just fmt` to apply rustfmt formatting
rules to the project codebase. You should then run Clippy and the full project
unit test suite by invoking `just review`.

# Appendix: Code Style

General project code style rules apply to trigger definitions:

- Function calls and enum values have exactly one `::` qualifier, enum names
  and struct names have zero `::` qualifiers.
  - `my_module::function()`
  - `MyEnum::Value`
  - `v: MyEnum`
  - `s: MyStruct`
- The order of declarations in a file should always be:
  - Public Structs
  - Public Functions
  - Private Structs
  - Private Functions
- In general the "main" or "most important" definitions for a file are higher
  up in the file.
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