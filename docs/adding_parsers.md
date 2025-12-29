# Adding Parser Support for New Cards

This guide provides instructions for AI agents extending the Dreamtides rules
text parser to support new card abilities. It includes the complete workflow
from implementation through testing, serialization, and error handling.

---

## Quick Start Instructions for AI Agents

### First-Time Setup

Before beginning work, follow the environment setup in
`rules_engine/docs/environment_setup.md`:

1. Verify Rust toolchain: `rustc --version && cargo --version`
2. Install components: `rustup component add clippy rustfmt`
3. Install nightly rustfmt: `rustup component add rustfmt --toolchain nightly`
4. Install just: `cargo install just`
5. Install workspace lints: `cargo install cargo-workspace-lints`

### Progress Tracking

Use the TodoWrite MCP tool to track progress throughout implementation. Create
a todo list at the start of work with items for each major step:

1. Analyze rules text and review similar cards in rules_text_sorted.json
2. Implement parser for new syntax
3. Add parsing tests with insta snapshots
4. Update serialize_ability support
5. Add round-trip tests
6. Update parse_spanned_ability support
7. Add spanned ability tests
8. Consider error handling improvements
9. Run `just fmt` and `just review`

Mark todos as in_progress before starting each step and completed when done.

### Code Style Rules (Top 3)

Follow these critical code style rules at all times:

1. **No inline comments**: Code should be self-documenting. Add short doc
   comments only to top-level public functions. Never delete existing inline
   comments.

2. **Qualifier rules for names**: Function calls and enum values get exactly
   one qualifier. Struct names and enum types get zero qualifiers:
   - CORRECT: `effect_parser::single_effect_parser()`, `Zone::Battlefield`
   - WRONG: `crate::parser::effect_parser::single_effect_parser()`
   - WRONG: `ability_data::standard_effect::StandardEffect`

3. **No code in mod.rs files**: Use descriptively-named files with unique
   names prefixed by module context (e.g., `card_effect_parsers.rs` instead
   of `effects.rs`).

### Pre-Implementation: Study the Rules Text

Before writing any code, always read `rules_engine/docs/rules_text_sorted.json`
to understand the full range of rules text patterns. This file contains all
card rules sorted by complexity. Study:

- How similar abilities are phrased
- Common patterns and variations
- Edge cases and special syntax

Think ahead to future parsers. Make design choices that will generalize to
other similar cards rather than over-fitting to the specific card at hand.

### Key Terminology Changes in Parser V2

The following terminology has changed from previous versions:

- **"allied"** on characters means "another character you control"
- **"allied"** on events means "character you control"
- **"enemy"** now means an enemy character (not the opponent)
- **Directives** replace all previous variable syntax (like `$2` for costs)
- Directives never take arguments - they are a single identifier in braces

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

#### Parser Helpers

Use helpers from `parser_helpers.rs`:

| Helper | Purpose | Example Usage |
|--------|---------|---------------|
| `word("text")` | Match a specific word | `word("draw")` |
| `words(&["a", "b"])` | Match word sequence | `words(&["at", "the", "end"])` |
| `directive("name")` | Match a directive | `directive("Judgment")` |
| `period()` | Match period | `.then_ignore(period())` |
| `comma()` | Match comma | `.then_ignore(comma())` |
| `energy()` | Parse {e} directive | `energy()` returns `u32` |
| `cards()` | Parse {cards} directive | `cards()` returns `u32` |
| `discards()` | Parse {discards} | `discards()` returns `u32` |
| `points()` | Parse {points} | `points()` returns `u32` |
| `spark()` | Parse {s} directive | `spark()` returns `u32` |
| `subtype()` | Parse subtype variable | `subtype()` returns `CardSubtype` |

#### Register the New Parser

Add your parser to the choice in `effect_parser.rs`:

```rust
pub fn single_effect_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> + Clone {
    choice((
        card_effect_parsers::draw_cards(),
        card_effect_parsers::discard_cards(),
        // Add your new effect here
        your_module::your_new_effect(),
    ))
    .boxed()
}
```

#### Avoiding Infinite Loops (Left Recursion)

Parser combinators are vulnerable to infinite loops when a parser can call
itself without first consuming input. This is called left recursion.

**The Golden Rule**: Always consume at least one token before any recursive
call.

Bad pattern (infinite loop):
```rust
recursive(|cp| {
    choice((
        cp.clone().then_ignore(word("with")),  // WRONG: recurses first
        word("character"),
    ))
})
```

Good pattern:
```rust
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
fn test_your_new_effect() {
    let result = parse_effect("Your effect {cards}.", "cards: 2");
    assert_ron_snapshot!(result, @r###"
    YourNewEffect(
      count: 2,
    )
    "###);
}

#[test]
fn test_your_new_ability_full() {
    let result = parse_ability("{Judgment} Your effect {cards}.", "cards: 3");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([Judgment]),
      effect: Effect(YourNewEffect(
        count: 3,
      )),
    ))
    "###);
}
```

Available test helpers in `test_helpers.rs`:

| Function | Purpose |
|----------|---------|
| `parse_ability(input, vars)` | Parse to `Ability` |
| `parse_effect(input, vars)` | Parse to `StandardEffect` |
| `parse_trigger(input, vars)` | Parse to `TriggerEvent` |
| `parse_predicate(input, vars)` | Parse to `Predicate` |
| `try_parse_effect(input, vars)` | Parse with `Option` result |
| `parse_spanned_ability(input, vars)` | Parse to `SpannedAbility` |

Run tests with:
```bash
cargo test -p parser_v2_tests
```

Update snapshots with:
```bash
cargo insta review
```

### Step 3: Update Serialization Support

Serialization converts `Ability` structs back to template text for round-trip
verification. Update `rules_engine/src/parser_v2/src/serializer/parser_formatter.rs`.

Add a match arm for your new effect in `serialize_standard_effect`:

```rust
pub fn serialize_standard_effect(effect: &StandardEffect) -> String {
    match effect {
        StandardEffect::DrawCards { .. } => "draw {cards}.".to_string(),
        StandardEffect::DiscardCards { .. } => "discard {discards}.".to_string(),
        // Add your new effect
        StandardEffect::YourNewEffect { .. } => "your effect {cards}.".to_string(),
        _ => unimplemented!("Serialization not yet implemented"),
    }
}
```

The serializer produces canonical template text using standard variable names.
Use these canonical names:

| Directive | Variable Name |
|-----------|---------------|
| Energy | `{e}` |
| Card count | `{cards}` |
| Discard count | `{discards}` |
| Points | `{points}` |
| Spark | `{s}` |
| Subtype | `{subtype}` |

### Step 4: Add Round-Trip Tests

Round-trip tests verify that parsing then serializing produces the original
text. Add tests in `rules_engine/tests/parser_v2_tests/tests/ability_round_trip_tests.rs`:

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

### Step 5: Update Spanned Ability Support

Spanned abilities track text spans for UI display segmentation. Update
`rules_engine/src/parser_v2/src/builder/parser_builder.rs` if your new ability
requires special span handling.

The `SpannedAbility` types in `parser_spans.rs`:

```rust
pub enum SpannedAbility {
    Event(SpannedEventAbility),
    Static { text: SpannedText },
    Activated(SpannedActivatedAbility),
    Triggered(SpannedTriggeredAbility),
    Named { name: SpannedText },
}

pub struct SpannedTriggeredAbility {
    pub once_per_turn: Option<SpannedText>,
    pub trigger: SpannedText,
    pub effect: SpannedEffect,
}
```

### Step 6: Add Spanned Ability Tests

Add tests in `rules_engine/tests/parser_v2_tests/tests/spanned_ability_tests.rs`:

```rust
use chumsky::span::Span;
use parser_v2::builder::parser_spans::{SpannedAbility, SpannedEffect};
use parser_v2_tests::test_helpers::*;

#[test]
fn test_spanned_your_new_effect() {
    let spanned = parse_spanned_ability("Your trigger, your effect {cards}.", "cards: 2");

    if let SpannedAbility::Triggered(triggered) = spanned {
        assert_eq!(triggered.trigger.text, "Your trigger");

        if let SpannedEffect::Effect(effect) = triggered.effect {
            assert!(effect.text.contains("your effect"));
        } else {
            panic!("Expected Effect, got Modal");
        }
    } else {
        panic!("Expected Triggered ability");
    }
}
```

### Step 7: Consider Error Handling

Review whether parser error suggestions should be added for your new text.
Error suggestions live in
`rules_engine/src/parser_v2/src/error/parser_error_suggestions.rs`.

The suggestion system uses Levenshtein distance to find close matches for:
- Directives (`suggest_directive`)
- Variables (`suggest_variable`)
- Words (`suggest_word`)

If your new syntax introduces new keywords, add them to `PARSER_WORDS`:

```rust
static PARSER_WORDS: &[&str] = &[
    "abandon",
    "allied",
    // ... existing words ...
    "your_new_keyword",  // Add new keywords here
];
```

### Step 8: Format and Review

After completing all changes, run formatting and validation:

```bash
just fmt      # Apply rustfmt formatting
just review   # Full validation (format check, build, lint, test)
```

The `just review` command runs:
1. `cargo +nightly fmt --check`
2. `cargo build`
3. `cargo workspace-lints`
4. `cargo clippy`
5. `cargo test`

---

## Parser Architecture Reference

### High-Level Processing Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                          INPUT                                       │
│  rules_text: "{Judgment} Draw {cards}."  +  variables: "cards: 2"   │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    LEXER (Manual Implementation)                     │
│  Converts string → Vec<Token>                                       │
│  Tokens: Word, Directive, Punctuation, Newline                      │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    VARIABLE RESOLVER                                 │
│  Substitutes variable directives with concrete values                │
│  {cards} → 2, {e} → Energy(3), {subtype} → Warrior                  │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    CHUMSKY PARSER (0.12)                            │
│  Operates on &[Token] input                                         │
│  Produces: ParsedAbility with spans                                  │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    ABILITY BUILDER                                   │
│  Converts ParsedAbility → Ability                                   │
│  Also produces SpannedAbility for display segmentation              │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         OUTPUT                                       │
│  Result<Vec<Ability>, Vec<ParserError>>                             │
│  Optional: SpannedAbility for UI text segmentation                  │
└─────────────────────────────────────────────────────────────────────┘
```

### Crate Structure

```
rules_engine/src/parser_v2/
├── src/
│   ├── lib.rs                    # Public API + module declarations
│   │
│   ├── lexer/
│   │   ├── lexer_tokenize.rs     # Lexer entry point
│   │   └── lexer_token.rs        # Token enum definition
│   │
│   ├── variables/
│   │   ├── parser_bindings.rs    # Variable binding types
│   │   └── parser_substitutions.rs # Token substitution logic
│   │
│   ├── parser/
│   │   ├── ability_parser.rs     # Top-level ability parsing
│   │   ├── triggered_parser.rs   # Triggered ability parsing
│   │   ├── activated_ability_parser.rs
│   │   ├── static_ability_parser.rs
│   │   ├── named_parser.rs       # Named ability parsing
│   │   ├── effect_parser.rs      # Effect orchestration
│   │   ├── effect/
│   │   │   ├── card_effect_parsers.rs
│   │   │   ├── spark_effect_parsers.rs
│   │   │   ├── resource_effect_parsers.rs
│   │   │   ├── control_effects_parsers.rs
│   │   │   └── game_effects_parsers.rs
│   │   ├── trigger_parser.rs
│   │   ├── cost_parser.rs
│   │   ├── predicate_parser.rs
│   │   ├── card_predicate_parser.rs
│   │   ├── condition_parser.rs
│   │   └── parser_helpers.rs     # Shared combinators
│   │
│   ├── builder/
│   │   ├── parser_builder.rs     # Ability construction
│   │   ├── parser_spans.rs       # SpannedAbility types
│   │   └── parser_display.rs     # Display text extraction
│   │
│   ├── serializer/
│   │   └── parser_formatter.rs   # Ability → String conversion
│   │
│   └── error/
│       ├── parser_errors.rs      # Error types
│       ├── parser_diagnostics.rs # Ariadne integration
│       └── parser_error_suggestions.rs

rules_engine/tests/parser_v2_tests/
├── src/
│   └── test_helpers.rs           # Test utilities
└── tests/
    ├── effect_parser_tests.rs
    ├── triggered_ability_tests.rs
    ├── ability_round_trip_tests.rs
    ├── spanned_ability_tests.rs
    ├── parse_error_tests.rs
    └── ...
```

### Token System

The lexer produces these token types:

```rust
pub enum Token {
    Word(String),      // English word or symbol ("draw", "2", "+")
    Directive(String), // Braced directive: {Judgment}, {cards}
    Period,            // Sentence terminator
    Comma,             // Clause separator
    Colon,             // Cost/effect separator
    Newline,           // Ability separator
}
```

After variable resolution, tokens become:

```rust
pub enum ResolvedToken {
    Token(Token),                              // Non-variable token
    Integer { directive: String, value: u32 }, // Resolved number
    Subtype { directive: String, subtype: CardSubtype },
    FigmentCount { count: u32, figment_type: FigmentType },
    FigmentSingle { figment_type: FigmentType },
}
```

### Parser Hierarchy

The parser follows this priority order (most specific first):

```
ability_parser()
├── triggered_ability_parser()      # "When/Whenever/At...", "{Keyword}: ..."
│   ├── keyword_trigger_parser()    # "{Materialized}, {Judgment}: effect"
│   └── standard_trigger_parser()   # "Once per turn, when event, effect"
│
├── activated_ability_parser()      # "cost: effect"
│
├── named_ability_parser()          # "{ReclaimForCost}", "{Fast}"
│
├── event_ability_parser()          # "[cost:] effect" for events
│
└── static_ability_parser()         # Declarative statements ending in "."
```

---

## Directive Reference

Based on rules_text_sorted.json, these directives are used:

### Trigger Keywords

| Directive | Example |
|-----------|---------|
| `{Judgment}` | `{Judgment} Draw {cards}.` |
| `{Materialized}` | `{Materialized} Gain {e}.` |
| `{Dissolved}` | `{Dissolved} Draw {cards}.` |
| `{MaterializedJudgment}` | `{MaterializedJudgment} Gain {e}.` |
| `{MaterializedDissolved}` | `{MaterializedDissolved} Draw {cards}.` |

### Action Verbs (Capitalized)

| Directive | Example |
|-----------|---------|
| `{Dissolve}` | `{Dissolve} an enemy.` |
| `{Banish}` | `{Banish} an enemy with cost {e} or less.` |
| `{Discover}` | `{Discover} {a-subtype}.` |
| `{Prevent}` | `{Prevent} a card.` |
| `{Foresee}` | `{Foresee}.` |
| `{Materialize}` | `{Materialize} it.` |
| `{Reclaim}` | `{Reclaim} this character.` |
| `{Kindle}` | `{Kindle}.` |

### Action Verbs (Lowercase - in context)

| Directive | Example |
|-----------|---------|
| `{dissolve}` | `when you {dissolve} an ally` |
| `{banish}` | `{banish} {up-to-n-allies}` |
| `{materialize}` | `when you {materialize} a character` |
| `{kindle}` | `once per turn, {kindle}` |
| `{reclaim}` | `it gains {reclaim}` |
| `{foresee}` | `When you play an event, {foresee}.` |

### Numeric Variables

| Directive | Purpose | Example |
|-----------|---------|---------|
| `{e}` | Energy amount | `Gain {e}.` with `e: 3` |
| `{cards}` | Card count | `Draw {cards}.` with `cards: 2` |
| `{discards}` | Discard count | `Discard {discards}.` |
| `{points}` | Points | `Gain {points}.` |
| `{s}` | Spark | `+{s} spark` |
| `{count}` | Generic count | `with {count} or more cards` |

### Subtype Variables

| Directive | Example |
|-----------|---------|
| `{subtype}` | `allied {subtype}` with `subtype: Warrior` |
| `{plural-subtype}` | `allied {plural-subtype}` |
| `{a-subtype}` | `{Discover} {a-subtype}.` |

### Count Expressions

| Directive | Example |
|-----------|---------|
| `{count-allies}` | `Abandon {count-allies}:` |
| `{count-allied-subtype}` | `With {count-allied-subtype},` |
| `{cards-numeral}` | `play {cards-numeral} in a turn` |
| `{top-n-cards}` | `the {top-n-cards} of your deck` |

### Collection Expressions

| Directive | Example |
|-----------|---------|
| `{up-to-n-allies}` | `{banish} {up-to-n-allies}` |
| `{up-to-n-events}` | `Return {up-to-n-events} from your void` |
| `{n-random-characters}` | `{Materialize} {n-random-characters}` |
| `{n-figments}` | `{Materialize} {n-figments}.` |
| `{a-figment}` | `{materialize} {a-figment}` |

### Named Abilities

| Directive | Example |
|-----------|---------|
| `{ReclaimForCost}` | `\n\n{ReclaimForCost}` |
| `{Fast}` | `{Fast} -- cost: effect` |
| `{reclaim-for-cost}` | `gains {reclaim-for-cost}` |

### Modal Markers

| Directive | Example |
|-----------|---------|
| `{ChooseOne}` | `{ChooseOne}\n{bullet}...` |
| `{bullet}` | `{bullet} {e}: effect` |

### Other Directives

| Directive | Purpose |
|-----------|---------|
| `{fast}` | Card with fast ability |
| `{energy-symbol}` | Energy symbol reference |
| `{spark}` | Spark value reference |
| `{it-or-them}` | Pronoun for singular/plural |
| `{this-turn-times}` | Turn count reference |
| `{JudgmentPhaseName}` | Phase name reference |
| `{maximum-energy}` | Maximum energy reference |

---

## Common Patterns in Rules Text

Study these patterns from rules_text_sorted.json to inform parser design:

### Simple Effects
```
Draw a card.
Gain {e}.
Draw {cards}.
```

### Triggered Abilities with Keywords
```
{Judgment} Gain {e}.
{Materialized} Draw {cards}.
{MaterializedJudgment} {Kindle}.
```

### Effects with Targets
```
{Dissolve} an enemy.
{Dissolve} an enemy with spark {s} or less.
{Banish} an enemy with cost {e} or less.
```

### Compound Effects
```
Gain {e}. Draw {cards}.
Draw {cards}. Discard {discards}.
{Dissolve} an enemy. You lose {points}.
```

### Triggers with Conditions
```
When you discard a card, gain {points}.
When you play an event, gain {e}.
When you {materialize} an ally, gain {e}.
Once per turn, when you {materialize} a character, gain {e}.
```

### Activated Abilities
```
{e}: Draw {cards}.
Abandon an ally: Gain {e}.
{e}, Discard {discards}: {kindle}.
```

### Static Abilities
```
Events cost you {e} less.
Allied {plural-subtype} have +{s} spark.
This character's spark is equal to the number of cards in your void.
```

### Complex Patterns
```
{Judgment} You may discard {discards} to draw {cards} and gain {points}.
{Materialize} {n-random-characters} with cost {e} or less from your deck.
{ChooseOne}
{bullet} {e}: Return an enemy to hand.
{bullet} {e-}: Draw {cards}.
```

---

## Validation Commands Reference

| Command | Purpose |
|---------|---------|
| `just fmt` | Apply rustfmt formatting |
| `just check` | Type check code |
| `just clippy` | Check for lint warnings |
| `cargo test -p parser_v2_tests` | Run parser tests |
| `just review` | Full validation pipeline |
| `cargo insta review` | Review/update snapshots |

---

## Additional Documentation

- Environment setup: `rules_engine/docs/environment_setup.md`
- Full parser design: `rules_engine/docs/parser_v2_design.md`
- Adding effects: `rules_engine/docs/adding_new_effects.md`
- Adding triggers: `rules_engine/docs/adding_new_triggers.md`
- Chumsky guide: `docs/chumsky/guide/getting_started.md`
- Chumsky recursion: `docs/chumsky/guide/recursion.md`
