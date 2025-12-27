# Parser V2 Technical Design Document

## Executive Summary

This document specifies Parser V2 for the Dreamtides card game rules engine. V2 replaces the existing Chumsky 0.10-based parser with a new implementation using Chumsky 0.12 and a manually-implemented lexer pass. Primary goals: support new template syntax with variables, improve compile and runtime performance, enable round-trip serialization, and consolidate `parser` and `parser_cli` crates.

---

## How to Use This Document
1. Read sections 1-5 once at project start
2. When starting milestone N, read only section 13.N
3. Reference appendices when you encounter specific syntax questions
4. Record progress in docs/parser_v2/PROGRESS.md

---

## 1. Architecture Overview

### 1.1 High-Level Flow

```
┌───────────────────────────────────────────────────────────────────────────┐
│                            INPUT                                          │
│  rules_text: "{Judgment} Draw {cards}."  +  variables: "cards: 2"        │
└─────────────────────────────────┬─────────────────────────────────────────┘
                                  │
                                  ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                      LEXER (Manual Implementation)                        │
│  Converts string → Vec<Token>                                            │
│  Tokens: Word, Directive, Punctuation, Newline                           │
└─────────────────────────────────┬─────────────────────────────────────────┘
                                  │
                                  ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                    VARIABLE RESOLVER                                      │
│  Substitutes variable directives with concrete values                     │
│  {cards} → 2, {e} → Energy(3), {subtype} → Warrior                       │
└─────────────────────────────────┬─────────────────────────────────────────┘
                                  │
                                  ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                    CHUMSKY PARSER (0.12)                                  │
│  Operates on &[Token] input                                              │
│  Produces: ParsedAbility with spans                                       │
└─────────────────────────────────┬─────────────────────────────────────────┘
                                  │
                                  ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                    ABILITY BUILDER                                        │
│  Converts ParsedAbility → Ability                                        │
│  Also produces SpannedAbility for display segmentation                   │
└─────────────────────────────────┬─────────────────────────────────────────┘
                                  │
                                  ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                           OUTPUT                                          │
│  Result<Vec<Ability>, Vec<ParserError>>                                  │
│  Optional: SpannedAbility for UI text segmentation                       │
└───────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Crate Structure

Parser V2 consolidates into a single crate with clear module separation. **Important:** Do not put code into `mod.rs` files. Each module should have a descriptively-named file. All filenames must be globally unique (prefixed with their module context).

```
rules_engine/src/parser_v2/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Public API + module declarations
│   ├── parser_cli.rs             # CLI implementation (bin target)
│   │
│   ├── lexer/
│   │   ├── mod.rs                # Module declarations
│   │   ├── lexer_tokenize.rs     # Lexer entry point and main logic
│   │   └── lexer_token.rs        # Token enum definition
│   │
│   ├── variables/
│   │   ├── mod.rs                # Module declarations
│   │   ├── parser_bindings.rs    # Variable binding types
│   │   └── parser_substitutions.rs # Token substitution logic
│   │
│   ├── parser/
│   │   ├── mod.rs                # Module declarations
│   │   ├── ability_parser.rs     # Top-level ability parsing
│   │   ├── triggered_parser.rs   # Triggered ability parsing
│   │   ├── activated_ability_parser.rs # Activated ability parsing
│   │   ├── static_ability_parser.rs # Static ability parsing
│   │   ├── named_parser.rs       # Named ability parsing
│   │   ├── effect_parser.rs      # Effect orchestration
│   │   ├── effect/
│   │   │   ├── mod.rs            # Module declarations
│   │   │   ├── card_effect_parsers.rs   # Draw, discard, materialize, etc.
│   │   │   ├── spark_effect_parsers.rs  # Kindle, spark gains
│   │   │   ├── resource_effect_parsers.rs # Energy, points
│   │   │   ├── control_effects_parsers.rs  # Gain control, disable
│   │   │   └── game_effects_parsers.rs   # Foresee, prevent, discover
│   │   ├── trigger_parser.rs     # Trigger event parsing
│   │   ├── cost_parser.rs        # Cost parsing
│   │   ├── predicate_parser.rs   # Card predicate parsing
│   │   ├── condition_parser.rs   # Condition parsing
│   │   └── parser_helpers.rs     # Shared parser combinators
│   │
│   ├── builder/
│   │   ├── mod.rs                # Module declarations
│   │   ├── parser_builder.rs     # Ability construction
│   │   ├── parser_spans.rs       # SpannedAbility types
│   │   └── parser_display.rs     # Display text extraction
│   │
│   ├── serializer/
│   │   ├── mod.rs                # Module declarations
│   │   └── parser_formatter.rs   # Round-trip serialization & Ability → String conversion
│   │
│   └── error/
│       ├── mod.rs                # Module declarations
│       ├── parser_errors.rs      # Error types
│       ├── parser_diagnostics.rs # Ariadne integration
│       └── parser_error_suggestions.rs # Error recovery strategies

rules_engine/tests/parser_v2_tests/
├── Cargo.toml
├── src/
│   └── lib.rs                    # Test utilities
└── tests/
    ├── lexer_tests.rs
    ├── variable_tests.rs
    ├── effect_parser_tests.rs
    ├── trigger_parser_tests.rs
    ├── cost_parser_tests.rs
    ├── predicate_parser_tests.rs
    ├── static_ability_tests.rs
    ├── integration_tests.rs
    ├── round_trip_tests.rs
    └── error_tests.rs

rules_engine/benchmarks/parser_v2/
├── Cargo.toml
└── benches/
    ├── parser_bench.rs
    └── full_pipeline_bench.rs
```

---

## 2. Token System

### 2.1 Token Definition

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// English word or symbol treated as word ("+", "2", "or")
    Word(String),

    /// Directive in braces: {Judgment}, {cards}, {subtype}
    /// The name is everything between the braces, e.g. "Judgment", "cards", "plural-subtype"
    Directive(String),

    /// Period character - sentence terminator
    Period,

    /// Comma character - clause separator
    Comma,

    /// Colon character - cost/effect separator
    Colon,

    /// Newline - ability separator
    Newline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

pub type Spanned<T> = (T, Span);
```

### 2.2 Lexer Implementation

The lexer is manually implemented (not using Chumsky) for performance and simplicity.

**Important:** The lexer operates on **lowercased** input for parsing, but the original input is preserved for span extraction and display segmentation. This matches V1 behavior and simplifies keyword matching.

```rust
pub struct LexResult {
    pub tokens: Vec<Spanned<Token>>,
    pub original: String,  // Preserved for span extraction
}

pub fn lex(input: &str) -> Result<LexResult, LexError> {
    let original = input.to_string();
    let lowercased = input.to_lowercase();
    let mut tokens = Vec::new();
    let mut chars = lowercased.char_indices().peekable();

    while let Some((start, ch)) = chars.next() {
        match ch {
            '{' => tokens.push(lex_directive(&mut chars, start)?),
            '.' => tokens.push((Token::Period, Span::new(start, start + 1))),
            ',' => tokens.push((Token::Comma, Span::new(start, start + 1))),
            ':' => tokens.push((Token::Colon, Span::new(start, start + 1))),
            '\n' => tokens.push((Token::Newline, Span::new(start, start + 1))),
            c if c.is_whitespace() => continue,
            _ => tokens.push(lex_word(&mut chars, start, ch)?),
        }
    }

    Ok(LexResult { tokens, original })
}
```

### 2.3 Directive Classification

Directives fall into categories (based on rules_text.json):

| Category | Examples | Handling |
|----------|----------|----------|
| **Trigger Keywords** | `{Judgment}`, `{Materialized}`, `{Dissolved}` | Parsed as trigger keywords |
| **Combined Triggers** | `{MaterializedJudgment}`, `{MaterializedDissolved}` | Parsed as combined triggers |
| **Action Verbs (caps)** | `{Dissolve}`, `{Banish}`, `{Foresee}`, `{Prevent}`, `{Discover}`, `{Materialize}`, `{Reclaim}`, `{Kindle}` | Effect verbs |
| **Action Verbs (lower)** | `{dissolve}`, `{banish}`, `{materialize}`, `{kindle}`, `{reclaim}`, `{foresee}`, `{prevent}` | Effect verbs (in context) |
| **Numeric Variables** | `{e}`, `{cards}`, `{discards}`, `{points}`, `{s}`, `{count}` | Resolved from variable map |
| **Subtype Variables** | `{subtype}`, `{plural-subtype}`, `{a-subtype}` | Resolved to CardSubtype |
| **Count Expressions** | `{count-allies}`, `{count-allied-subtype}`, `{cards-numeral}` | Resolved counting expressions |
| **Named Abilities** | `{ReclaimForCost}`, `{Fast}`, `{reclaim-for-cost}` | Expand to named abilities |
| **Modal** | `{ChooseOne}`, `{bullet}` | Modal effect markers |
| **Card References** | `{fast}`, `{energy-symbol}`, `{spark}` | Card type/symbol references |
| **Collection Exprs** | `{up-to-n-allies}`, `{up-to-n-events}`, `{n-random-characters}` | Variable collection counts |
| **Compound Directives** | `{n-figments}`, `{a-figment}` | Consume multiple variables (see 3.4) |
| **Other** | `{top-n-cards}`, `{it-or-them}`, `{this-turn-times}`, `{JudgmentPhaseName}`, `{maximum-energy}` | Context-specific |

---

## 3. Variable System

### 3.1 Variable Binding Types

```rust
use core_data::card_types::CardSubtype;

#[derive(Debug, Clone)]
pub enum VariableValue {
    /// Integer value (used for counts, energy, points, spark)
    Integer(u32),
    /// Subtype value (Warrior, Explorer, etc.)
    Subtype(CardSubtype),
}

#[derive(Debug, Clone, Default)]
pub struct VariableBindings {
    bindings: HashMap<String, VariableValue>,
}

impl VariableBindings {
    /// Parse "cards: 2, e: 3" or "cards: 2\ne: 3" format
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let mut bindings = HashMap::new();
        // Split on comma or newline
        for part in input.split([',', '\n']) {
            let part = part.trim();
            if part.is_empty() { continue; }
            let (key, value) = part.split_once(':')
                .ok_or(ParseError::InvalidVariableFormat)?;
            let key = key.trim();
            let value = value.trim();
            // Try integer first, then subtype
            if let Ok(n) = value.parse::<u32>() {
                bindings.insert(key.to_string(), VariableValue::Integer(n));
            } else if let Ok(subtype) = value.parse::<CardSubtype>() {
                bindings.insert(key.to_string(), VariableValue::Subtype(subtype));
            } else {
                return Err(ParseError::InvalidVariableValue(value.to_string()));
            }
        }
        Ok(Self { bindings })
    }

    pub fn get(&self, name: &str) -> Option<&VariableValue> {
        self.bindings.get(name)
    }
}
```

### 3.2 Variable Resolution

Variables are resolved during token processing, producing resolved tokens that the parser consumes:

```rust
#[derive(Debug, Clone)]
pub enum ResolvedToken {
    /// Non-variable token, passed through
    Token(Token),
    /// Resolved integer value
    Integer(u32),
    /// Resolved subtype
    Subtype(CardSubtype),
    /// Resolved figment count (compound: number + figment)
    FigmentCount { count: u32, figment_type: FigmentType },
    /// Resolved single figment (compound: figment only)
    FigmentSingle { figment_type: FigmentType },
}

pub fn resolve_variables(
    tokens: &[Spanned<Token>],
    bindings: &VariableBindings,
) -> Result<Vec<Spanned<ResolvedToken>>, UnresolvedVariable> {
    tokens.iter().map(|(token, span)| {
        match token {
            Token::Directive(name) if is_compound_directive(name) => {
                resolve_compound_directive(name, bindings).map(|t| (t, *span))
            }
            Token::Directive(name) if is_variable_directive(name) => {
                let var_name = extract_variable_name(name);
                let value = bindings.get(&var_name)
                    .ok_or(UnresolvedVariable(var_name.clone()))?;
                match value {
                    VariableValue::Integer(n) => Ok((ResolvedToken::Integer(*n), *span)),
                    VariableValue::Subtype(s) => Ok((ResolvedToken::Subtype(*s), *span)),
                    VariableValue::Figment(_) => Err(UnresolvedVariable(var_name)),
                }
            }
            _ => Ok((ResolvedToken::Token(token.clone()), *span))
        }
    }).collect()
}
```

### 3.3 Re-binding with Different Variables

To produce an Ability with different variable values, simply re-parse the original text with new bindings:

```rust
// Original parse
let text = "Draw {cards}.";
let vars1 = VariableBindings::parse("cards: 2").unwrap();
let ability1 = parse(text, &vars1).unwrap();

// Re-bind by re-parsing with different variables
let vars2 = VariableBindings::parse("cards: 5").unwrap();
let ability2 = parse(text, &vars2).unwrap();
```

This approach is simpler than tracking variable origins and avoids the complexity of runtime substitution. The parser is fast enough that re-parsing is acceptable.

### 3.4 Compound Directives

Some directives consume multiple variables to produce a single resolved value. These are identified by their directive name pattern:

| Directive | Variables | Example Resolution |
|-----------|-----------|-------------------|
| `{n-figments}` | `number`, `figment` | "three Radiant Figments" |
| `{a-figment}` | `figment` | "a Radiant Figment" |

**Example:** Parsing `{Materialize} {n-figments}.` with variables `number: 3, figment: radiant`:

```rust
#[derive(Debug, Clone)]
pub enum FigmentType {
    Radiant,
    Shadow,
    // ... other figment types
}

#[derive(Debug, Clone)]
pub enum VariableValue {
    Integer(u32),
    Subtype(CardSubtype),
    Figment(FigmentType),
}

pub fn resolve_compound_directive(
    name: &str,
    bindings: &VariableBindings,
) -> Result<ResolvedToken, UnresolvedVariable> {
    match name {
        "n-figments" => {
            let number = bindings.get_integer("number")?;
            let figment = bindings.get_figment("figment")?;
            Ok(ResolvedToken::FigmentCount { count: number, figment_type: figment })
        }
        "a-figment" => {
            let figment = bindings.get_figment("figment")?;
            Ok(ResolvedToken::FigmentSingle { figment_type: figment })
        }
        _ => Err(UnresolvedVariable(name.to_string()))
    }
}
```

The parser then handles these resolved tokens:

```rust
fn materialize_figments<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> {
    directive("materialize")
        .ignore_then(choice((
            figment_count_token().map(|(count, figment_type)| {
                StandardEffect::MaterializeFigments { count, figment_type }
            }),
            figment_single_token().map(|figment_type| {
                StandardEffect::MaterializeFigments { count: 1, figment_type }
            }),
        )))
}
```

---

## 4. Parser Design

### 4.1 Chumsky 0.12 Setup

```rust
use chumsky::prelude::*;

type ParserInput<'a> = &'a [Spanned<ResolvedToken>];
type ParserExtra<'a> = extra::Err<Rich<'a, ResolvedToken, Span>>;

fn ability_parser<'a>() -> impl Parser<'a, ParserInput<'a>, ParsedAbility, ParserExtra<'a>> {
    choice((
        triggered_ability_parser(),
        activated_ability_parser(),
        named_ability_parser(),
        event_ability_parser(),
        static_ability_parser(),
    ))
    .boxed()
}
```

### 4.2 Parser Hierarchy

The parser follows this priority order (most specific first):

```
ability_parser()
├── triggered_ability_parser()      # "When/Whenever/At...", "{Keyword}: ..."
│   ├── keyword_trigger_parser()    # "{Materialized}, {Judgment}: effect"
│   └── standard_trigger_parser()   # "Once per turn, when event, effect"
│
├── activated_ability_parser()      # "cost: effect"
│   └── parses: cost_parser() + effect_parser()
│
├── named_ability_parser()          # "{ReclaimForCost}", "{Fast}"
│
├── event_ability_parser()          # "[cost:] effect" for events
│   └── parses: optional_cost() + effect_parser()
│
└── static_ability_parser()         # Declarative statements ending in "."
```

### 4.3 Effect Parser Structure

Effects are split across multiple files for maintainability:

```rust
// parser/effect_parser.rs
fn effect_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Effect, ParserExtra<'a>> {
    recursive(|effect| {
        choice((
            modal_effect_parser(),
            effect_list_parser(effect.clone()),
            single_effect_with_options(effect.clone()),
        ))
    })
    .boxed()
}

fn single_effect_parser<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> {
    choice((
        card_effect_parsers::parser(),      // draw, discard, materialize, dissolve
        spark_effect_parsers::parser(),     // gain spark, kindle, spark becomes
        resource_effect_parsers::parser(),  // gain energy, gain points
        control_effects_parsers::parser(),  // gain control, disable abilities
        game_effects_parsers::parser(),     // foresee, prevent, banish, discover
    ))
    .boxed()
}
```

### 4.4 Complete Effect List

V2 must support all StandardEffect variants (see ability_data/src/standard_effect.rs for authoritative list). Key categories:

**Card Movement:** DrawCards, DiscardCards, MaterializeCharacter, DissolveCharacter, BanishCharacter, ReturnToHand, Copy, etc.

**Spark/Power:** GainsSpark, Kindle, SparkBecomes, etc.

**Resources:** GainEnergy, GainPoints, LosePoints, etc.

**Game Effects:** Foresee, Discover, Counterspell, TakeExtraTurn, YouWinTheGame, etc.

### 4.5 Trigger Events

```rust
fn trigger_event_parser<'a>() -> impl Parser<'a, ParserInput<'a>, TriggerEvent, ParserExtra<'a>> {
    choice((
        keyword_trigger(),           // {Materialized}, {Judgment}, {Dissolved}
        combined_keyword_trigger(),  // {MaterializedJudgment}, {MaterializedDissolved}
        materialize_trigger(),
        play_trigger(),
        discard_trigger(),
        dissolved_trigger(),
        banished_trigger(),
        abandon_trigger(),
        end_of_turn_trigger(),
        gain_energy_trigger(),
    ))
    .boxed()
}
```

### 4.6 Cost Parser

```rust
fn cost_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Cost, ParserExtra<'a>> {
    choice((
        energy_cost(),               // {e}
        abandon_cost(),              // "Abandon {count-allies}"
        banish_from_void_cost(),     // "{Banish} {cards} from your void"
        banish_from_hand_cost(),     // "{Banish} a card from hand"
        discard_cost(),              // "Discard {discards}"
        discard_hand_cost(),         // "Discard your hand"
        combined_costs(),            // Multiple costs separated by comma
    ))
    .boxed()
}
```

### 4.7 Predicate Parser

```rust
fn predicate_parser<'a>() -> impl Parser<'a, ParserInput<'a>, Predicate, ParserExtra<'a>> {
    choice((
        this_parser(),               // "this character/event"
        that_parser(),               // "that card"
        it_parser(),                 // "it"
        them_parser(),               // "them"
        enemy_parser(),              // "an enemy"
        another_parser(),            // "another character"
        your_parser(),               // "a character you control"
        any_parser(),                // "a card"
    ))
    .boxed()
}

fn card_predicate_parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> {
    choice((
        subtype_parser(),            // {subtype} resolved to CardSubtype
        character_with_spark(),      // "character with spark {s} or less"
        character_with_cost(),       // "character with cost {e} or less"
        fast_card_parser(),          // "{fast} card"
        event_parser(),              // "event"
        character_parser(),          // "character"
        card_parser(),               // "card"
    ))
    .boxed()
}
```

### 4.8 Static Ability Parser

```rust
fn static_ability_parser<'a>() -> impl Parser<'a, ParserInput<'a>, StaticAbility, ParserExtra<'a>> {
    choice((
        once_per_turn_play_from_void(),     // "Once per turn, you may play..."
        spark_bonus_your_characters(),       // "Allied {plural-subtype} have +{s} spark"
        cost_reduction(),                    // "Events cost you {e} less"
        enemy_cost_increase(),               // "The opponent's events cost {e} more"
        disable_enemy_materialized(),        // "Disable the '{Materialized}' abilities of enemies"
        has_all_character_types(),           // "Has all character types"
        spark_equal_to_count(),              // "This character's spark is equal to..."
        may_play_from_void(),                // "You may play... from your void"
        characters_in_hand_have_fast(),      // "Characters in your hand have {fast}"
        play_only_from_void(),               // "You may only play this character from your void"
        reveal_top_of_deck(),                // "Reveal the top card of your deck"
        play_from_top_of_deck(),             // "You may play characters from the top of your deck"
        void_cards_have_reclaim(),           // "While you have {count}... they have {reclaim}"
        judgment_triggers_on_materialize(),  // "The '{Judgment}' ability... triggers when you {materialize}"
    ))
    .boxed()
}
```

---

## 5. Boxing Strategy for Compile Performance

### 5.1 Boxing Rules

To prevent exponential type complexity and linker memory issues:

1. **Box every `choice()` with 4+ alternatives**
2. **Box all recursive parsers**
3. **Box top-level category parsers** (card_effects, spark_effects, etc.)
4. **Box the main entry points** (effect_parser, trigger_parser, etc.)

```rust
// GOOD: Boxed choice with many alternatives
fn card_effects<'a>() -> impl Parser<'a, ParserInput<'a>, StandardEffect, ParserExtra<'a>> {
    choice((
        draw_cards(),
        discard_cards(),
        materialize_character(),
        dissolve_character(),
        // ... many more
    ))
    .boxed()  // Essential for compile time
}
```

### 5.2 Parser Caching

Create the parser once and reuse:

```rust
use std::sync::OnceLock;

static PARSER: OnceLock<BoxedParser<...>> = OnceLock::new();

pub fn get_parser() -> &'static BoxedParser<...> {
    PARSER.get_or_init(|| ability_parser().boxed())
}
```

---

## 6. Spanned Abilities (Display Segmentation)

### 6.1 SpannedAbility Types

```rust
#[derive(Debug, Clone)]
pub enum SpannedAbility {
    Event(SpannedEventAbility),
    Static { text: SpannedText },
    Activated(SpannedActivatedAbility),
    Triggered(SpannedTriggeredAbility),
    Named { name: SpannedText },
}

#[derive(Debug, Clone)]
pub struct SpannedTriggeredAbility {
    pub once_per_turn: Option<SpannedText>,  // "Once per turn"
    pub trigger: SpannedText,
    pub effect: SpannedEffect,
}

#[derive(Debug, Clone)]
pub struct SpannedText {
    pub text: String,
    pub span: Span,
}
```

### 6.2 Span Extraction

During parsing, capture spans with `map_with`:

```rust
fn effect_with_span<'a>() -> impl Parser<'a, ParserInput<'a>, (StandardEffect, Span), ParserExtra<'a>> {
    single_effect_parser()
        .map_with(|effect, extra| {
            (effect, extra.span())
        })
}
```

---

## 7. Round-Trip Serialization

### 7.1 Serializer Module

The serializer converts Ability structs back into template text. Since we don't track variable origins, the serializer uses canonical variable names based on field semantics:

```rust
pub fn serialize_ability(ability: &Ability) -> String {
    match ability {
        Ability::Event(e) => serialize_event(e),
        Ability::Triggered(t) => serialize_triggered(t),
        // ...
    }
}

fn serialize_standard_effect(effect: &StandardEffect) -> String {
    match effect {
        StandardEffect::DrawCards { count } => format!("Draw {{cards}}."),
        StandardEffect::GainEnergy { gains } => format!("Gain {{e}}."),
        StandardEffect::DissolveCharacter { target } =>
            format!("{{Dissolve}} {}.", serialize_predicate(target)),
        // ... all variants
    }
}
```

The serializer produces canonical template text using standard variable names (`{cards}`, `{e}`, `{s}`, etc.). The actual values are passed separately when re-parsing.

### 7.2 Round-Trip Testing

```rust
#[test]
fn test_round_trip_draw_cards() {
    let original = "Draw {cards}.";
    let vars = VariableBindings::parse("cards: 2").unwrap();

    let parsed = parse(original, &vars).unwrap();
    let serialized = serialize_abilities(&parsed);

    let reparsed = parse(&serialized, &vars).unwrap();
    assert_eq!(parsed, reparsed);
}
```

---

## 8. Error Handling

### 8.1 Error Types

```rust
#[derive(Debug)]
pub enum ParserError {
    Lex(LexError),
    UnresolvedVariable { name: String, span: Span },
    Parse(ParseError),
    Semantic(SemanticError),
}

#[derive(Debug)]
pub enum LexErrorKind {
    UnclosedBrace,
    InvalidCharacter(char),
    EmptyDirective,
}
```

### 8.2 Ariadne Integration

```rust
pub fn format_error(error: &ParserError, source: &str) -> String {
    let mut output = Vec::new();
    // Use ariadne Report builder to create pretty error output
    // ...
    String::from_utf8(output).unwrap()
}
```

---

## 9. CLI Design

### 9.1 Command Line Interface

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "parser", about = "Dreamtides card ability parser")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Parse a single ability string
    Parse {
        /// The ability text to parse
        text: String,
        /// Variable bindings (comma or newline separated, e.g. "cards:2,e:3")
        #[arg(short, long)]
        vars: Option<String>,
        /// Output format (json, ron, debug)
        #[arg(short, long, default_value = "json")]
        format: OutputFormat,
    },

    /// Parse abilities from TOML cards file
    ParseFile {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Verify parser by round-tripping all cards
    Verify {
        input: PathBuf,
    },
}
```

---

## 10. Dependencies

```toml
[package]
name = "parser_v2"
version = "0.1.0"
edition = "2021"

[dependencies]
ability_data = { workspace = true }
core_data = { workspace = true }

ariadne = { workspace = true }
chumsky = { workspace = true }
clap = { workspace = true, features = ["derive"] }
ron = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
toml = { workspace = true }

[dev-dependencies]
criterion = { workspace = true }
insta = { workspace = true }

[[bin]]
name = "parser"
path = "src/parser_cli.rs"
```

---

## 11. Agent Workflow Instructions

### 11.0 Token Efficiency

Minimize token usage by reading only essential files specified in the milestone deliverables rather than broadly exploring the codebase—the design doc already provides the necessary file structure and imports. Always Read files before attempting Write or Edit operations to avoid tool call errors, and use TodoWrite at the start of each milestone to track deliverables systematically.

### 11.1 Validation Checklist

After every code change:

1. **Format:** `just fmt`
2. **Type check:** `just check`
3. **Lint:** `just clippy`
4. **Run parser tests:** `cargo test -p parser_v2_tests`
5. **Full validation:** `just review`

### 11.2 Code Style Rules

- **No inline comments** - code should be self-documenting
- **Single qualifier for functions:** `parser::parse()` not `crate::parser::parse()`
- **Zero qualifiers for structs and enums:** `Ability` not `ability::Ability`
- **Public items at top of file**, private below
- **No `pub use` re-exports**
- **No code in `mod.rs` files** - use descriptively-named files like `lexer_core.rs` instead
- **Globally unique filenames** - prefix with module context (e.g., `variable_binding.rs` not `binding.rs`)
- **Alphabetized Cargo.toml deps** - internal first, then external
- **Use modern Rust features:** if-let, `"{inline:?}"` formatting

### 11.3 Context Recording

Create `rules_engine/src/parser_v2/PROGRESS.md` to track:

```markdown
# Parser V2 Implementation Progress

## Current Milestone: [N]

## Completed:
- [ ] Item with date and notes

## Blockers:
- Issue description and potential solutions

## Observations:
- Performance note
- Design decision rationale

## Next Steps:
1. Specific next action
```

### 11.4 Getting More Information

**Essential Chumsky documentation to read first:**

1. `docs/chumsky/guide/getting_started.md` - Parser creation and basic usage
2. `docs/chumsky/guide/key_concepts.md` - Parser trait, Input trait, Error trait
3. `docs/chumsky/guide/meet_the_parsers.md` - All primitives and combinators reference
4. `docs/chumsky/guide/recursion.md` - How to handle recursive grammars

**API documentation:**
- `target/doc-md/chumsky/primitive.md` - `just`, `one_of`, `none_of`, `any`, `end`
- `target/doc-md/chumsky/chumsky.md` - Main trait methods

**Current implementation reference:**
- `src/ability_data/src/*.rs` - Ability data structures
- `src/parser/src/*.rs` - V1 parser patterns (for reference only)

---

## 12. Representative Test Cards

These cards must parse and round-trip correctly. They are the primary validation target for all milestones:

| # | Rules Text | Variables |
|---|-----------|-----------|
| 1 | `When you play {cards-numeral} in a turn, {reclaim} this character.` | `cards: 2` |
| 2 | `{Discover} a card with cost {e}.` | `e: 2` |
| 3 | `{Judgment} Return this character from your void to your hand.` | - |
| 4 | `{Judgment} You may discard {discards} to draw {cards} and gain {points}.` | `discards: 1, cards: 1, points: 1` |
| 5 | `Draw {cards}. Discard {discards}.\n\n{ReclaimForCost}` | `cards: 2, discards: 2, reclaim: 2` |
| 6 | `{Judgment} You may {banish} {cards} from your void to {dissolve} an enemy with cost {e} or less.` | `cards: 3, e: 2` |
| 7 | `Once per turn, you may play a character with cost {e} or less from your void.` | `e: 2` |
| 8 | `When you discard a card, {kindle}.` | `k: 1` |
| 9 | `{Materialize} {n-figments}.` | `number: 3, figment: radiant` |

---

## 13. Milestones

**IMPORTANT: End-to-End First Strategy**

The milestone order is designed to prove out a working end-to-end solution as early as possible. After implementing the lexer, variables, and basic parser infrastructure (Milestones 1-3), we immediately implement the CLI, display, error handling, and benchmarking infrastructure (Milestones 4-7). This ensures:

1. We can validate the full pipeline works before investing in more parsers
2. Error reporting and diagnostics are available early for debugging
3. We can measure performance impact as we add features
4. Round-trip testing is exercised from the start

Only after proving the end-to-end solution works do we proceed to implement additional parser features (Milestones 8+).

---

### Milestone 1: Lexer and Token System
**Scope:** Implement manual lexer, token types, span tracking

**Deliverables:**
- `lexer/lexer_tokenize.rs` - Main lexer logic
- `lexer/lexer_token.rs` - Token types and Span
- `tests/parser_v2_tests/tests/lexer_tests.rs`

**Test Cards:** Verify lexer correctly tokenizes all 9 representative test cards.

**Round-trip:** N/A (lexer only)

---

### Milestone 2: Variable System
**Scope:** Variable parsing, binding, resolution

**Deliverables:**
- `variables/parser_bindings.rs` - Variable binding types
- `variables/parser_substitutions.rs` - Token substitution logic
- `tests/parser_v2_tests/tests/variable_tests.rs`

**Test Cards:** Verify variables resolve correctly for all 9 representative test cards, including compound directive resolution for card 9.

**Round-trip:** Verify VariableBindings can be serialized and re-parsed.

---

### Milestone 3: Basic Parser Infrastructure + Simple Effects
**Scope:** Chumsky 0.12 setup, error types, DrawCards, GainEnergy, DiscardCards

**Deliverables:**
- `parser/parser_helpers.rs` - Common parser combinators (word, directive, integer, etc.)
- `parser/ability_parser.rs` - Parser orchestration
- `parser/effect/card_effect_parsers.rs` - Draw, discard, gain energy parsers
- `parser/effect_parser.rs` - Effect parser orchestration (single_effect_parser)
- `error/parser_errors.rs` - Error types (ParserError, LexError)
- `error/parser_diagnostics.rs` - Ariadne integration
- Serializer stubs for implemented effects

**Test Cards:** Cards 2 and 5 (partial - draw/discard portions)

**Round-trip:** Verify `Draw {cards}.` parses and serializes back identically.

---

### Milestone 4: CLI and File Processing
**Scope:** Command line interface, TOML processing

**Rationale:** We implement the CLI early to prove out the end-to-end pipeline before investing in more parser features. This ensures the architecture works and provides tooling for testing subsequent milestones.

**Deliverables:**
- `parser_cli.rs` - CLI implementation with Parse, ParseFile, Verify commands
- Integration with lexer, variable resolution, and basic parser

**Test Cards:** CLI can parse `Draw {cards}.` with variables.

**Round-trip:** `parser verify` command works for simple effects implemented so far.

---

### Milestone 5: SpannedAbility and Display
**Scope:** Text segmentation for UI

**Rationale:** Display infrastructure is needed early to verify span tracking works correctly before adding more complex parsers.

**Deliverables:**
- `builder/parser_builder.rs` - Ability construction
- `builder/parser_spans.rs` - SpannedAbility types
- `builder/parser_display.rs` - Display text extraction

**Test Cards:** Simple effects from Milestone 3 produce correct spans.

**Round-trip:** N/A (spans are metadata)

---

### Milestone 6: Error Recovery and Polish
**Scope:** Error recovery strategies, edge cases

**Rationale:** Good error messages are essential for debugging as we add more parsers. Implementing this early means all subsequent work benefits from clear diagnostics.

**Deliverables:**
- `error/parser_error_suggestions.rs` - Error recovery strategies
- Enhanced error messages with Ariadne

**Test Cards:** Test malformed versions of simple effects produce helpful errors.

**Round-trip:** N/A

---

### Milestone 7: Benchmarking and Optimization
**Scope:** Performance validation infrastructure

**Rationale:** Establishing benchmarks early lets us track performance impact as we add features. If the basic pipeline is slow, we want to know before building on top of it.

**Deliverables:**
- `benchmarks/parser_v2/benches/parser_bench.rs`
- `benchmarks/parser_v2/benches/full_pipeline_bench.rs`

**Test Cards:** Benchmark simple effects implemented so far.

**Round-trip:** Verify round-trip performance is acceptable.

---

### Milestone 8: Trigger Parser
**Scope:** All trigger events, keyword triggers

**Deliverables:**
- `parser/trigger_parser.rs`
- Tests for trigger parsing

**Test Cards:** Cards 1, 3, 4, 6, 7, 8 (trigger portions only)

**Round-trip:** Verify triggered ability trigger text round-trips.

**Checklist**: Ensure this milestone works with
- Command line interface
- Display parsing via `SpannedAbility`
- Error recovery
- Round trip

---


### Milestone 8B: Release Mode Compiler Performance
**Scope:** Implement scripts to identify track compiler performance

### Milestone 9: Predicate Parser
**Scope:** Card predicates, determiners, targets

**Deliverables:**
- `parser/predicate_parser.rs`
- Tests for predicate parsing

**Test Cards:** Cards 2, 6, 7 (predicate portions)

**Round-trip:** Verify predicate text round-trips (e.g., "an enemy with cost {e} or less").

---

### Milestone 10: Cost Parser
**Scope:** All cost types

**Deliverables:**
- `parser/cost_parser.rs`
- Tests for cost parsing

**Test Cards:** Card 6 (banish cost), card 4 (discard cost trigger cost)

**Round-trip:** Verify cost text round-trips.

---

### Milestone 11: Complete Effect Parsers
**Scope:** All remaining StandardEffect variants

**Deliverables:**
- `parser/effect/spark_effect_parsers.rs`
- `parser/effect/resource_effect_parsers.rs`
- `parser/effect/control_effects_parsers.rs`
- `parser/effect/game_effects_parsers.rs`
- Complete serializer for all effects

**Test Cards:** All 9 representative test cards should now have all effects parsing, including compound directive card 9.

**Round-trip:** All effect types round-trip correctly.

---

### Milestone 12: Triggered Ability Parser (Complete)
**Scope:** Full triggered abilities including "Once per turn"

**Deliverables:**
- `parser/triggered_parser.rs`
- Integration tests

**Test Cards:** Cards 1, 3, 4, 6, 7, 8 should fully parse as triggered abilities.

**Round-trip:** Full triggered ability round-trips.

---

### Milestone 13: Static Ability Parser
**Scope:** Static abilities

**Deliverables:**
- `parser/static_ability_parser.rs`
- Tests for static abilities

**Test Cards:** Card 7 (static: once per turn play from void)

**Round-trip:** Static ability text round-trips.

---

### Milestone 14: Effect Options and Conditions
**Scope:** Optional effects, conditional effects, trigger costs

**Deliverables:**
- `parser/condition_parser.rs`
- Updates to effect parser

**Test Cards:** Cards 4 and 6 ("You may...")

**Round-trip:** Optional effect text round-trips.

---

### Milestone 15: Named Abilities
**Scope:** {ReclaimForCost}, {Fast}

**Deliverables:**
- `parser/named_parser.rs`

**Test Cards:** Card 5 ({ReclaimForCost})

**Round-trip:** Named ability round-trips.

---

### Milestone 16: Activated Ability Parser
**Scope:** Activated abilities

**Deliverables:**
- `parser/activated_ability_parser.rs`

**Test Cards:** None in representative set, but add test for `{e}: Draw {cards}.`

**Round-trip:** Activated ability round-trips.

---

### Milestone 17: Modal Effects
**Scope:** {ChooseOne}, {bullet}

**Deliverables:**
- Modal parser in `parser/effect_parser.rs`

**Test Cards:** Add modal test card from rules_text.json:
`{ChooseOne}\n{bullet} {e}: Return an enemy to hand.\n{bullet} {e-}: Draw {cards}.`

**Round-trip:** Modal effect round-trips.

---

## 14. Expected ability_data Changes

Minimal changes expected. Potentially new StandardEffect variants if template syntax requires effects not currently supported.

No breaking changes to existing Ability structure.

---

## 15. Unit Testing

We need extensive unit tests for parsers, covering both success and failure
cases. Parsing tests should use the `insta` crate along with the
`assert_ron_snapshot!()` function to make assertions about parser output.

```
#[test]
fn test_draw_cards() {
    let result = parse_effect("Draw {cards}.", "cards: 2");
    assert_ron_snapshot!(result, @r###"
    DrawCards(
      count: 2,
    )
    "###);
}
```


## Appendix A: Template Syntax Reference

Based on rules_text.json, the following directives exist:

| Directive | Type | Example Usage |
|-----------|------|---------------|
| `{Judgment}` | Trigger keyword | `{Judgment} Draw {cards}.` |
| `{Materialized}` | Trigger keyword | `{Materialized} Gain {e}.` |
| `{Dissolved}` | Trigger keyword | `{Dissolved} Draw {cards}.` |
| `{MaterializedJudgment}` | Combined trigger | `{MaterializedJudgment} Gain {e}.` |
| `{MaterializedDissolved}` | Combined trigger | `{MaterializedDissolved} Draw {cards}.` |
| `{Dissolve}` | Action verb | `{Dissolve} an enemy.` |
| `{Banish}` | Action verb | `{Banish} an enemy.` |
| `{Discover}` | Action verb | `{Discover} a card.` |
| `{Prevent}` | Action verb | `{Prevent} a card.` |
| `{Foresee}` | Action verb | `{Foresee}.` (value from vars) |
| `{Materialize}` | Action verb | `{Materialize} it.` |
| `{Reclaim}` | Action verb | `{Reclaim} this character.` |
| `{Kindle}` | Action verb | `{Kindle}.` (value from vars) |
| `{e}` | Variable (energy) | `Gain {e}.` with `e: 3` |
| `{cards}` | Variable (count) | `Draw {cards}.` with `cards: 2` |
| `{discards}` | Variable (count) | `Discard {discards}.` |
| `{points}` | Variable (count) | `Gain {points}.` |
| `{s}` | Variable (spark) | `+{s} spark` |
| `{subtype}` | Variable (CardSubtype) | `allied {subtype}` with `subtype: Warrior` |
| `{plural-subtype}` | Variable (CardSubtype) | `allied {plural-subtype}` |
| `{a-subtype}` | Variable (CardSubtype) | `{Discover} {a-subtype}.` |
| `{count-allies}` | Count expression | `Abandon {count-allies}:` |
| `{count-allied-subtype}` | Count expression | `With {count-allied-subtype},` |
| `{cards-numeral}` | Count as word | `play {cards-numeral} in a turn` |
| `{ReclaimForCost}` | Named ability | `\n\n{ReclaimForCost}` |
| `{Fast}` | Named ability | `{Fast} -- cost: effect` |
| `{reclaim}` | Keyword (lower) | `it gains {reclaim}` |
| `{kindle}` | Keyword (lower) | `{kindle}` (action, value from vars) |
| `{fast}` | Card type | `a {fast} card` |
| `{ChooseOne}` | Modal marker | `{ChooseOne}\n{bullet}...` |
| `{bullet}` | Modal choice | `{bullet} {e}: effect` |
| `{energy-symbol}` | Symbol reference | `for each {energy-symbol} spent` |
| `{top-n-cards}` | Count expression | `the {top-n-cards} of your deck` |
| `{up-to-n-allies}` | Collection | `{banish} {up-to-n-allies}` |
| `{it-or-them}` | Pronoun | `{materialize} {it-or-them}` |
| `{reclaim-for-cost}` | Named ability | `gains {reclaim-for-cost}` |
| `{n-random-characters}` | Count expression | `{Materialize} {n-random-characters}` |
| `{n-figments}` | Compound (number, figment) | `{Materialize} {n-figments}.` → "three Radiant Figments" |
| `{a-figment}` | Compound (figment) | `{materialize} {a-figment}` → "a Radiant Figment" |

## Appendix B: Chumsky 0.12 Key Differences from 0.10

1. **No more `Empty` error type** - use `extra::Err<Rich<...>>`
2. **`recursive()` API unchanged**
3. **`boxed()` returns `Boxed<...>` type**
4. **Token input via `&[T]` where T: Clone + PartialEq**
5. **`map_with` provides `MapExtra` with `.span()` method**
6. **Recovery via `recover_with(via_parser(...))`**
