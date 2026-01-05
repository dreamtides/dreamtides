# Adding Parser Support for New Cards

This guide provides instructions for AI agents extending the Dreamtides rules
text parser to support new card abilities.

**IMPORTANT**: Do not add doc comments to parser functions. Code should be
self-documenting through clear function and variable names.

---

## Quick Start

### Progress Tracking

Use an internal task list or todowrite mcp to track progress. Create a task list for yourself with:

1. Analyze rules text and review similar cards in rules_text_sorted.json
2. Implement parser for new syntax
3. Add parsing tests with insta snapshots
4. Update serialize_ability support
5. Add round-trip tests
6. Update parse_spanned_ability support
7. Add spanned ability tests
8. Consider error handling improvements
9. Run `just fmt` and `just review`
10. Update this guide with improvements discovered

Mark todos as in_progress before starting each step and completed when done.

Do NOT commit the task list to version control, maintain it in memory.

### Code Style Rules (Critical)

Follow these rules at all times:

1. **No inline comments**: Code should be self-documenting. Add short doc
   comments only to top-level public functions. Never delete existing inline
   comments.

2. **Qualifier rules**: Function calls and enum values get exactly one
   qualifier. Struct names and enum types get zero qualifiers:
   - CORRECT: `effect_parser::single_effect_parser()`, `Zone::Battlefield`
   - WRONG: `crate::parser::effect_parser::single_effect_parser()`
   - WRONG: `ability_data::standard_effect::StandardEffect`

3. **No code in mod.rs files**: Use descriptively-named files with unique
   names prefixed by module context (e.g., `card_effect_parsers.rs` instead
   of just `effects.rs`).

### Pre-Implementation

Before writing any code, always read `rules_engine/docs/rules_text_sorted.json`
to understand the full range of rules text patterns. This file contains all
card rules sorted by complexity. Study:

- How similar abilities are phrased
- Common patterns and variations
- Edge cases and special syntax

Think ahead to future parsers. Make design choices that will generalize to
other similar cards rather than over-fitting to the specific card at hand.

### Key Terminology Changes

The following terminology applies in parser v2:

- **"allied"** on characters means "another character you control"
- **"allied"** on events means "character you control"
- **"enemy"** now means an enemy character (not the opponent)
- **Directives** replace all previous variable syntax (like `$2` for costs)
- Directives never take arguments - they are a single identifier in braces

---

## Terminology Notes

The following terminology changes from parser v1 are important to keep in mind:

- The term "reclaim" when used as a *verb* means to return from your void to
  play (`ReturnFromYourVoidToPlay`). There is also a named ability called
  `{ReclaimForCost}` that gives a card the ability to be reclaimed for a cost.
- The term "ally" means a character *other* than this character when it appears
  on character cards. On event cards an "ally" is any character you control.
- The term "enemy" means a character controlled by the opponent. The opposing
  player themself is called "the opponent".


## Critical Pitfalls

### Directive Names Must Be Lowercase

The lexer lowercases ALL input text before parsing. This means directive names
in your parsers **MUST** be lowercase.

```rust
// WRONG - will NEVER match:
directive("Judgment")
directive("Foresee")
directive("Kindle")

// CORRECT:
directive("judgment")  // Matches {Judgment} in rules text
directive("foresee")   // Matches {Foresee} in rules text
directive("kindle")    // Matches {Kindle} in rules text
```

**Why this happens**: The lexer converts `"{Judgment}"` to
`Token::Directive("judgment")` before the parser sees it. If you write
`directive("Judgment")`, it will never match the lowercased token.

### Variable Directives Need Custom Helpers

Some directives are configured in `parser_substitutions.rs` to be **variables**
that require values, not simple directive tokens. Check the `DIRECTIVES` array
to see which directives are variables:

```rust
static DIRECTIVES: &[(&str, &str, VariableConstructor)] = &[
    ("foresee", "foresee", integer),  // {foresee} requires variable value
    ("Foresee", "foresee", integer),  // {Foresee} also requires variable value
    ("kindle", "k", integer),         // {kindle} maps to variable "k"
    ("Kindle", "k", integer),         // {Kindle} maps to variable "k"
];
```

**What this means:**
- `{Foresee}` gets resolved to `ResolvedToken::Integer { directive: "foresee", value: X }`
- You need a helper function to parse this, not `directive()`
- Tests must provide variable values: `parse_ability("{Judgment} {Foresee}.", "foresee: 3")`

**Creating helpers for variable directives:**

```rust
pub fn foresee_count<'a>() -> impl Parser<'a, ParserInput<'a>, u32, ParserExtra<'a>> + Clone {
    select! {
        (ResolvedToken::Integer { directive, value }, _)
            if directive == "foresee" || directive == "Foresee" => value
    }
}

pub fn foresee<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    foresee_count()  // NOT directive("foresee")
        .then_ignore(period())
        .map(|count| StandardEffect::Foresee { count })
}
```

### Compound Directives Are Single Tokens

Directives like `{a-subtype}` are resolved as **single tokens**, not as
`word("a")` followed by something else.

```rust
// WRONG:
directive("discover")
    .ignore_then(word("a"))  // Don't expect "a" as a separate word!
    .ignore_then(subtype())

// CORRECT:
directive("discover")
    .ignore_then(card_predicate_parser::parser())  // Parses the subtype token directly
```

**Why**: `{a-subtype}` gets resolved to `ResolvedToken::Subtype` during variable
resolution, so there's no `word("a")` token to match.

### Test Variable Naming Conventions

When providing variables in tests, use the **variable name** from the DIRECTIVES
array, not the directive name:

```rust
// DIRECTIVES shows: ("Kindle", "k", integer)
parse_ability("{Kindle}.", "k: 1")  // Use "k", not "kindle"

// DIRECTIVES shows: ("a-subtype", "subtype", subtype)
parse_ability("{Discover} {a-subtype}.", "subtype: warrior")  // Use "subtype"
```

**Subtype values must be lowercase:** `"subtype: warrior"` not `"subtype: Warrior"`

### Snapshot Array Formatting

RON snapshots use multi-line array formatting:

```rust
// Expected format:
Keywords([
  Judgment,
])

// Not this:
Keywords([Judgment])
```

Use `sed` or accept snapshots with `INSTA_UPDATE=always` to fix formatting.

### Event Ability Serialization

Don't forget to add Event ability support in `serialize_ability`:

```rust
pub fn serialize_ability(ability: &Ability) -> String {
    match ability {
        Ability::Triggered(triggered) => { /* ... */ }
        Ability::Event(event) => {  // Add this!
            if let ability_data::effect::Effect::Effect(effect) = &event.effect {
                capitalize_first_letter(&serialize_standard_effect(effect))
            } else {
                unimplemented!("Complex event effects")
            }
        }
        _ => unimplemented!(),
    }
}
```

### Import Organization

The nightly formatter groups imports in a specific way. Let it format your
imports rather than trying to organize them manually:

```rust
// After rustfmt, imports will be grouped like this:
use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;

use crate::parser::parser_helpers::{
    directive, foresee_count, period, word, ParserExtra, ParserInput,
};
use crate::parser::{card_predicate_parser, predicate_parser};
```

---

## Implementation Workflow

### Step 1: Implement the Parser

Parser code lives in `rules_engine/src/parser_v2/src/parser/`. Effects are
split across files in the `effect/` subdirectory:

- `card_effect_parsers.rs` - Draw, discard, materialize, etc.
- `spark_effect_parsers.rs` - Kindle, spark gains
- `resource_effect_parsers.rs` - Energy, points
- `control_effects_parsers.rs` - Gain control, disable abilities
- `game_effects_parsers.rs` - Foresee, prevent, discover

Choose the appropriate file based on effect category. If adding a new category,
create a new file with a descriptive name.

#### Parser Pattern

```rust
use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;

use crate::parser::parser_helpers::{
    cards, directive, energy, period, word, ParserExtra, ParserInput,
};

pub fn your_new_effect<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    word("keyword")
        .ignore_then(energy())
        .then_ignore(period())
        .map(|n| StandardEffect::YourNewEffect { amount: n })
}
```

#### Key Parser Helpers

Use helpers from `parser_helpers.rs`:

| Helper | Purpose | Example Usage |
|--------|---------|---------------|
| `word("text")` | Match a specific word | `word("draw")` |
| `words(&["a", "b"])` | Match word sequence | `words(&["at", "the", "end"])` |
| `directive("name")` | Match a directive (MUST be lowercase!) | `directive("judgment")` |
| `period()` | Match period | `.then_ignore(period())` |
| `comma()` | Match comma | `.then_ignore(comma())` |
| `energy()` | Parse {e} directive | `energy()` returns `u32` |
| `cards()` | Parse {cards} directive | `cards()` returns `u32` |
| `discards()` | Parse {discards} | `discards()` returns `u32` |
| `points()` | Parse {points} | `points()` returns `u32` |
| `spark()` | Parse {s} directive | `spark()` returns `u32` |
| `subtype()` | Parse subtype variable | `subtype()` returns `CardSubtype` |
| `literal_number()` | Parse literal numbers | Matches "3" in "Draw 3 cards" |

**Important**: For variable directives like `{Foresee}` or `{Kindle}`, you need
to create custom helpers (see "Variable Directives Need Custom Helpers" above).

#### Register the New Parser

Add your parser to the choice in `effect_parser.rs`:

```rust
pub fn single_effect_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        card_effect_parsers::draw_cards(),
        card_effect_parsers::discard_cards(),
        your_module::your_new_effect(),  // Add here
    ))
    .boxed()
}
```

#### Avoiding Infinite Loops (Left Recursion)

Parser combinators are vulnerable to infinite loops when a parser can call
itself without first consuming input. This is called left recursion.

**The Golden Rule**: Always consume at least one token before any recursive call.

```rust
// Bad pattern (infinite loop):
recursive(|cp| {
    choice((
        cp.clone().then_ignore(word("with")),  // WRONG: recurses first
        word("character"),
    ))
})

// Good pattern:
recursive(|cp| {
    choice((
        directive("fast").ignore_then(cp.clone()),  // RIGHT: consumes first
        word("character"),
    ))
})
```

See section 4.9 of `rules_engine/docs/parser_v2_design.md` for comprehensive
guidance on avoiding left recursion.

#### Boxing for Compile Performance

Box parsers to prevent exponential type complexity:

1. Box every `choice()` with 4+ alternatives
2. Box all recursive parsers
3. Box top-level category parsers
4. Box main entry points

```rust
fn category_effects<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> {
    choice((
        effect_a(),
        effect_b(),
        effect_c(),
        effect_d(),
    ))
    .boxed()  // Essential for compile time
}
```

### Step 2: Add Parsing Tests

Tests go in `rules_engine/tests/parser_v2_tests/tests/`. Use the insta crate
with RON snapshots for parser output assertions.

Create tests for full card text, not individual component parts. Import test
helpers and use `assert_ron_snapshot!`:

```rust
use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_your_new_ability() {
    let result = parse_ability("{Judgment} Your effect {cards}.", "cards: 3");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: Effect(YourNewEffect(
        count: 3,
      )),
    ))
    "###);
}

#[test]
fn test_your_new_event_ability() {
    let result = parse_ability("Your effect {cards}.", "cards: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(YourNewEffect(
        count: 2,
      )),
    ))
    "###);
}
```

**IMPORTANT**: All cargo commands must be run from the `rules_engine` directory.

Run tests: `cargo test -p parser_v2_tests`
Run specific test file: `cargo test -p parser_v2_tests --test effect_parser_tests`
Update snapshots: `cargo insta review`

### Step 3: Update Serialization Support

Serialization converts `Ability` structs back to template text for round-trip
verification. Update `rules_engine/src/parser_v2/src/serializer/parser_formatter.rs`.

Add a match arm for your new effect in `serialize_standard_effect`:

```rust
pub fn serialize_standard_effect(effect: &StandardEffect) -> String {
    match effect {
        StandardEffect::DrawCards { .. } => "draw {cards}.".to_string(),
        StandardEffect::DiscardCards { .. } => "discard {discards}.".to_string(),
        StandardEffect::YourNewEffect { .. } => "your effect {cards}.".to_string(),
        _ => unimplemented!("Serialization not yet implemented"),
    }
}
```

Use these canonical variable names:

| Directive | Variable Name |
|-----------|---------------|
| Energy | `{e}` |
| Card count | `{cards}` |
| Discard count | `{discards}` |
| Points | `{points}` |
| Spark | `{s}` |
| Subtype | `{subtype}` |

### Step 4: Add Round-Trip Tests

Round-trip tests verify that parsing then serializing produces reasonable output.
Add tests in `rules_engine/tests/parser_v2_tests/tests/ability_round_trip_tests.rs`:

```rust
use parser_v2::serializer::parser_formatter;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_your_new_effect() {
    let original = "Your effect {cards}.";
    let parsed = parse_ability(original, "cards: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}
```

**Note**: Round-trips don't need to be 100% exact. When there are multiple valid
phrasings for the same effect (like ", then" vs "."), either output is fine.
Make a best effort to match the original, but don't create special-case code
just for serialization.

### Step 5: Update Spanned Ability Support

Spanned abilities track text spans for UI display segmentation. Update
`rules_engine/src/parser_v2/src/builder/parser_builder.rs` if your new ability
requires special span handling.

### Step 6: Add Spanned Ability Tests

Add tests in `rules_engine/tests/parser_v2_tests/tests/spanned_ability_tests.rs`:

```rust
use parser_v2::builder::parser_spans::{SpannedAbility, SpannedEffect};
use parser_v2_tests::test_helpers::*;

#[test]
fn test_spanned_your_new_effect() {
    let spanned = parse_spanned_ability("Your trigger, your effect {cards}.", "cards: 2");
    if let SpannedAbility::Triggered(triggered) = spanned {
        assert_eq!(triggered.trigger.text, "Your trigger");
        if let SpannedEffect::Effect(effect) = triggered.effect {
            assert!(effect.text.contains("your effect"));
        }
    }
}
```

### Step 7: Consider Error Handling

Review whether parser error suggestions should be added for your new text.
Error suggestions live in `parser_error_suggestions.rs`.

If your new syntax introduces new words, add them to `PARSER_WORDS`:

```rust
static PARSER_WORDS: &[&str] = &[
    "abandon",
    "allied",
    "your_new_keyword",  // Add new keywords here
];
```

### Step 8: Format and Review

After completing all changes, run formatting and validation:

```bash
just fmt      # Apply rustfmt formatting
just review   # Full validation (format check, build, lint, test)
```

---

## Compound Effects

Compound effects (multiple effects separated by periods) are automatically
supported through `effect_or_compound_parser` in `effect_parser.rs`. You
typically do NOT need to write a special parser for compound effects.

When you implement a single effect parser, it automatically works in compound
effect patterns:

```
Gain {e}. Draw {cards}.           // Automatically works
Draw {cards}. Discard {discards}. // Automatically works
{Dissolve} an enemy. You lose {points}. // Automatically works
```

Focus on implementing the single effect parser correctly. Test both the
standalone effect AND compound variations to ensure proper integration.

---

## Optional Effects

Optional effects (prefixed with "You may") are supported through
`EffectWithOptions` in `effect_parser.rs`. The parser automatically wraps
effects prefixed with "you may" in an optional wrapper.

When you implement an effect parser, it automatically works with the optional
prefix:

```
Return an ally to hand.           // Works without optional
You may return an ally to hand.   // Automatically wrapped as optional
```

The parser creates an `Effect::WithOptions` with the `optional` field set to
`true`. No special parser code is needed for individual effects - the optional
handling is done at the effect composition level in `effect_or_compound_parser`.

When serializing optional effects, the formatter automatically adds "you may"
prefix when `options.optional` is true. See `serialize_effect` in
`parser_formatter.rs` for the implementation.

---

## Reference Documentation

For detailed reference information, see:

- **Data types**: `rules_engine/docs/parser_data_types.md` - All ability data
  types (Ability, Effect, StandardEffect, Predicate, etc.)
- **Directives**: `rules_engine/docs/parser_directives.md` - Complete directive
  reference and common patterns
- **Parser architecture**: `rules_engine/docs/parser_v2_design.md` - Full
  parser design and processing flow
- **Environment setup**: `rules_engine/docs/environment_setup.md`
- **Adding effects**: `rules_engine/docs/adding_new_effects.md`
- **Adding triggers**: `rules_engine/docs/adding_new_triggers.md`
- **Chumsky guide**: `docs/chumsky/guide/getting_started.md`
- **Chumsky recursion**: `docs/chumsky/guide/recursion.md`

---

## Validation Commands

All commands must be run from the `rules_engine` directory:

| Command | Purpose |
|---------|---------|
| `just fmt` | Apply rustfmt formatting |
| `just check` | Type check code |
| `just clippy` | Check for lint warnings |
| `cargo test -p parser_v2_tests` | Run all parser tests |
| `cargo test -p parser_v2_tests --test effect_parser_tests` | Run specific test file |
| `just review` | Full validation pipeline |
| `cargo insta review` | Review/update snapshots |
