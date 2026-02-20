# Parser V2 Technical Design Document

## Executive Summary

This document specifies Parser V2 for the Dreamtides card game rules engine. V2 replaces the existing Chumsky 0.10-based parser with a new implementation using Chumsky 0.12 and a manually-implemented lexer pass. Primary goals: support new template syntax with variables, improve compile and runtime performance, enable round-trip serialization, and consolidate `parser` and `parser_cli` crates.

---

## How to Use This Document
1. Read sections 1-5 once at project start
2. When starting milestone N, read only section 13.N
3. Reference appendices when you encounter specific syntax questions
4. Record progress in docs/parser/PROGRESS.md

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
rules_engine/src/parser/
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
│   │   ├── predicate_parser.rs   # Predicate parsing
|   |   ├── card_predicate_parser.rs # Card predicate parsing
│   │   ├── condition_parser.rs   # Condition parsing
│   │   └── parser_helpers.rs     # Shared parser combinators
│   │
│   ├── builder/
│   │   ├── mod.rs                # Module declarations
│   │   ├── parser_builder.rs     # Ability construction
│   │   └── parser_spans.rs       # SpannedAbility types
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

rules_engine/tests/parser_tests/
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

rules_engine/benchmarks/parser_benchmarks/
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


### 4.9 Avoiding Infinite Loops (Left Recursion)

**Critical:** Parser combinators are vulnerable to infinite loops caused by left recursion. A parser has left recursion when it can attempt to match itself as the first action without consuming any input.

#### 4.9.1 The Problem

Left recursion occurs when a recursive parser can immediately call itself before consuming any tokens:

```rust
// DANGER: This will cause an infinite loop!
fn card_predicate_parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> {
    recursive(|cp| {
        choice((
            // This alternative starts with the recursive parser 'cp' without consuming input first
            cp.clone().then_ignore(word("with")).then(word("cost")),  // INFINITE LOOP!
            word("character").to(CardPredicate::Character),
        ))
    })
}
```

**Why this loops forever:**
1. Parser tries to match first alternative
2. First alternative immediately tries to match `cp` (the recursive parser)
3. This recursively tries step 1 again **without consuming any tokens**
4. Loop repeats infinitely

#### 4.9.2 The Golden Rule

**Always consume at least one token before any recursive call.**

Every alternative in a `choice()` that uses a recursive parser must begin with a non-recursive parser that consumes input:

```rust
// CORRECT: Each alternative consumes tokens before recursing
fn card_predicate_parser<'a>() -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> {
    recursive(|cp| {
        choice((
            // ✓ Consumes {fast} directive BEFORE recursing
            directive("fast").ignore_then(cp.clone()),
            // ✓ Consumes "character" word before returning
            word("character").to(CardPredicate::Character),
        ))
    })
}
```

#### 4.9.3 Common Patterns to Avoid

**Dangerous Pattern 1: Recursive parameter used first**
```rust
// BAD
fn with_cost_parser<'a>(
    target: impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> + Clone,
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> {
    choice((
        words(&["character", "with", "cost"]).ignore_then(energy()),  // OK
        target.then_ignore(words(&["with", "cost"])),  // LEFT RECURSION if target is recursive!
    ))
}
```

**Dangerous Pattern 2: Optional matching before recursion**
```rust
// BAD
recursive(|cp| {
    word("fast").or_not()  // Can match nothing!
        .ignore_then(cp)   // Then immediately recurse - INFINITE LOOP!
})
```

**Dangerous Pattern 3: Empty alternative before recursion**
```rust
// BAD
choice((
    empty().ignore_then(cp),  // Can match nothing, then recurse - INFINITE LOOP!
    word("character"),
))
```

#### 4.9.4 Safe Patterns

**Pattern 1: Token consumption before recursion**
```rust
// GOOD: directive() consumes a token before recursing
directive("fast").ignore_then(cp.clone())
```

**Pattern 2: Recursion after required tokens**
```rust
// GOOD: Multiple tokens consumed before any recursion
words(&["character", "with", "cost"])
    .ignore_then(energy())
    .then(cp.clone())
```

**Pattern 3: Non-recursive alternatives**
```rust
// GOOD: No recursion in this alternative
word("character").to(CardPredicate::Character)
```

**Pattern 4: Recursion at the end of a chain**
```rust
// GOOD: Tokens consumed, THEN recursive parser used
word("fast").ignore_then(
    cp.clone().or(just(&[]).to(CardPredicate::Card))
)
```

#### 4.9.5 How to Identify Left Recursion

When adding a recursive parser, check each alternative in the `choice()`:

1. **Trace the first action**: What's the first parser combinator that executes?
2. **Does it consume input?**: Check if it matches `word()`, `directive()`, `words()`, or other consuming parsers
3. **Can it match empty?**: Parsers like `empty()`, `or_not()`, or default alternatives can match without consuming
4. **Is it the recursive parser?**: If the first thing is `cp` or a parameter that might be recursive, you have left recursion

**Example audit:**
```rust
recursive(|cp| {
    choice((
        directive("fast").ignore_then(cp.clone()),  // ✓ directive() consumes first
        character_with_cost_parser(cp.clone()),     // ⚠️  Check character_with_cost_parser!
        word("character").to(CardPredicate::Character),  // ✓ word() consumes first
    ))
})
```

If `character_with_cost_parser` has an alternative that starts with its `target` parameter, and you pass `cp` as that target, you have left recursion.

#### 4.9.6 How to Fix Left Recursion

**Strategy 1: Remove the recursive parameter**

If the problematic function accepts a recursive parser as a parameter, remove alternatives that use it first:

```rust
// BEFORE (has left recursion when called with recursive parser)
fn character_with_cost_parser<'a>(
    target: impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>>,
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> {
    choice((
        words(&["character", "with", "cost"]).ignore_then(energy()),
        target.then_ignore(words(&["with", "cost"])),  // Problem!
    ))
}

// AFTER (no recursion parameter)
fn character_with_cost_parser<'a>(
) -> impl Parser<'a, ParserInput<'a>, CardPredicate, ParserExtra<'a>> {
    choice((
        words(&["character", "with", "cost"]).ignore_then(energy()),
        // Removed the problematic alternative
    ))
}
```

**Strategy 2: Consume tokens first**

Restructure to ensure token consumption before recursion:

```rust
// BEFORE
recursive(|cp| cp.clone().then(word("modifier")))  // BAD

// AFTER
recursive(|cp| word("base").then(cp.clone().then(word("modifier")).or_not()))  // GOOD
```

**Strategy 3: Move recursion to parent parser**

Let the parent parser handle the composition:

```rust
// Parent handles {fast} + target composition
recursive(|cp| {
    choice((
        directive("fast").ignore_then(character_with_cost_parser()),  // Composes here
        character_with_cost_parser(),  // No recursion in this function
    ))
})
```

#### 4.9.7 Testing for Infinite Loops

After implementing any recursive parser:

1. **Add a simple test** that exercises the parser
2. **Run with timeout**: `cargo test <test_name> --timeout 5`
3. **If it hangs**: You likely have left recursion
4. **Audit all alternatives** using the checklist in 4.9.5

```rust
#[test]
fn test_predicate_terminates() {
    // This test should complete in milliseconds
    let result = parse_predicate("it", "");
    assert_eq!(result, Predicate::It);
}
```

If this test hangs, trace through the parser logic to find where recursion occurs without token consumption.

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


---

## 11. Agent Workflow Instructions

### 11.0 Token Efficiency

Minimize token usage by reading only essential files specified in the milestone deliverables rather than broadly exploring the codebase—the design doc already provides the necessary file structure and imports. Always Read files before attempting Write or Edit operations to avoid tool call errors, and use TodoWrite at the start of each milestone to track deliverables systematically.

### 11.1 Validation Checklist

After every code change:

1. **Format:** `just fmt`
2. **Type check:** `just check`
3. **Lint:** `just clippy`
4. **Run parser tests:** `cargo test -p parser_tests`
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
