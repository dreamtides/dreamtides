# Appendix: Rust Integration

This appendix describes how the `rlt!` macro is implemented, what Rust code it
generates, and how errors are reported.

## Macro Architecture

RLT uses a Rust procedural macro to parse `.rlt.rs` files and generate Rust
code. The process has three phases:

1. **Parsing**: Extract phrase definitions from the macro input
2. **Validation**: Check names, detect undefined references
3. **Code Generation**: Emit Rust functions and runtime support

A companion build script validates cross-file consistency (all languages define
the same phrases).

---

## Phase 1: Parsing

The macro receives tokens from inside the `rlt! { ... }` block. It parses these
into an internal representation:

```rust
// Internal AST (conceptual)
struct PhraseDefinition {
    name: Identifier,
    parameters: Vec<Identifier>,
    body: PhraseBody,
    tags: Vec<MetadataTag>,
}

enum PhraseBody {
    Simple(TemplateString),
    Variants(Vec<(VariantKey, TemplateString)>),
}

struct TemplateString {
    segments: Vec<Segment>,
}

enum Segment {
    Literal(String),
    Interpolation {
        transforms: Vec<Transform>,
        reference: Reference,
        selectors: Vec<Selector>,
    },
}

enum Reference {
    Parameter(Identifier),
    Phrase(Identifier),
    PhraseCall { name: Identifier, args: Vec<Reference> },
}
```

### Parsing Interpolations

The parser handles `{...}` blocks within template strings:

| Syntax           | Parsed As                          |
| ---------------- | ---------------------------------- |
| `{name}`         | Parameter or phrase reference      |
| `{card:n}`       | Reference with selector            |
| `{card:acc:n}`   | Reference with chained selectors   |
| `{@cap name}`    | Transform applied to reference     |
| `{@cap @a card}` | Chained transforms                 |
| `{foo(x, y)}`    | Phrase call with arguments         |

### Variant Key Parsing

Variant keys can be simple or multi-dimensional:

```rust
// Simple variants
card = { one: "card", other: "cards" };
// Parsed as: [("one", "card"), ("other", "cards")]

// Multi-dimensional variants
card = { nom.one: "карта", nom.few: "карты", acc.one: "карту", ... };
// Parsed as: [("nom.one", "карта"), ("nom.few", "карты"), ...]
```

---

## Phase 2: Validation

After parsing, the macro validates names and references. This is purely
compile-time checking—no type inference or type checking occurs.

### Name Resolution

The validator checks that all referenced names exist:

1. **Phrase references**: Every `{phrase_name}` must refer to a defined phrase
2. **Parameter references**: Every `{param}` in a phrase body must be in that
   phrase's parameter list
3. **Selector references**: Every `{phrase:selector}` must use a valid variant
   name (for literal selectors)

### Undefined Phrase Detection

```rust
rlt! {
    draw(n) = "Draw {n} {cards:n}.";  // 'cards' not defined
}
```

The validator builds a set of all phrase names, then checks each reference:

```rust
fn validate_reference(ref: &Reference, phrases: &HashSet<&str>, params: &[&str]) {
    match ref {
        Reference::Phrase(name) => {
            if !phrases.contains(name.as_str()) {
                emit_error!(name.span(), "unknown phrase '{}'", name);
            }
        }
        Reference::Parameter(name) => {
            if !params.contains(&name.as_str()) {
                emit_error!(name.span(), "unknown parameter '{}'", name);
            }
        }
        // ...
    }
}
```

### Selector Validation

For literal selectors, the validator checks that the variant exists:

```rust
rlt! {
    card = { one: "card", other: "cards" };
    take = "{card:accusative}";  // Error: no 'accusative' variant
}
```

For parameter selectors (`{card:n}`), no compile-time validation occurs—the
selection happens at runtime based on the parameter's value.

---

## Phase 3: Code Generation

Based on the validated AST, the macro generates Rust code.

### The Value Type

All parameters use a single `Value` type that handles runtime dispatch:

```rust
pub enum Value {
    Number(i64),
    Float(f64),
    String(String),
    Phrase(PhraseRef),
}

impl Value {
    /// Try to interpret this value as a number for plural selection.
    pub fn as_number(&self) -> Option<i64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Float(f) => Some(*f as i64),
            Value::String(s) => s.parse().ok(),
            Value::Phrase(_) => None,
        }
    }

    /// Get a tag from this value (only works for phrases).
    pub fn get_tag(&self, tag_name: &str) -> Option<&str> {
        match self {
            Value::Phrase(p) => p.tags.iter()
                .find(|t| t.starts_with(tag_name))
                .map(|s| s.as_str()),
            _ => None,
        }
    }

    /// Get a variant from this value (only works for phrases).
    pub fn get_variant(&self, variant: &str) -> Option<&str> {
        match self {
            Value::Phrase(p) => p.variants.iter()
                .find(|(k, _)| *k == variant)
                .map(|(_, v)| v.as_str()),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Phrase(p) => write!(f, "{}", p.text),
        }
    }
}
```

### The PhraseRef Type

Phrases return a `PhraseRef` that carries metadata for runtime operations:

```rust
pub struct PhraseRef {
    pub text: &'static str,
    pub variants: &'static [(&'static str, &'static str)],
    pub tags: &'static [&'static str],
}

impl std::fmt::Display for PhraseRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}
```

### Into<Value> Implementations

To allow convenient calling, common types implement `Into<Value>`:

```rust
impl From<i32> for Value {
    fn from(n: i32) -> Self { Value::Number(n as i64) }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self { Value::Number(n) }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self { Value::String(s.to_string()) }
}

impl From<String> for Value {
    fn from(s: String) -> Self { Value::String(s) }
}

impl From<PhraseRef> for Value {
    fn from(p: PhraseRef) -> Self { Value::Phrase(p) }
}
```

### Simple Phrase → Function

```rust
// RLT:
hello = "Hello, world!";

// Generated:
pub fn hello() -> PhraseRef {
    PhraseRef {
        text: "Hello, world!",
        variants: &[],
        tags: &[],
    }
}
```

### Phrase with Parameters → Function with Value Arguments

```rust
// RLT:
greet(name) = "Hello, {name}!";

// Generated:
pub fn greet(name: impl Into<Value>) -> String {
    let name = name.into();
    format!("Hello, {}!", name)
}
```

### Phrase with Variants → Multiple Accessors

```rust
// RLT:
card = {
    one: "card",
    other: "cards",
};

// Generated:
static CARD_VARIANTS: &[(&str, &str)] = &[
    ("one", "card"),
    ("other", "cards"),
];

pub fn card() -> PhraseRef {
    PhraseRef {
        text: "card",  // Default (first variant)
        variants: CARD_VARIANTS,
        tags: &[],
    }
}

pub fn card_one() -> PhraseRef {
    PhraseRef {
        text: "card",
        variants: CARD_VARIANTS,
        tags: &[],
    }
}

pub fn card_other() -> PhraseRef {
    PhraseRef {
        text: "cards",
        variants: CARD_VARIANTS,
        tags: &[],
    }
}
```

### Phrase with Tags

```rust
// RLT:
carta = "carta" :fem;

// Generated:
pub fn carta() -> PhraseRef {
    PhraseRef {
        text: "carta",
        variants: &[],
        tags: &["fem"],
    }
}
```

### Selection Code Generation

Selection is resolved at runtime by examining the selector value:

```rust
// RLT:
draw(n) = "Draw {n} {card:n}.";

// Generated:
pub fn draw(n: impl Into<Value>) -> String {
    let n = n.into();

    // Select card variant based on n
    let card_text = match n.as_number() {
        Some(num) => {
            let category = cldr_plural_en(num);
            CARD_VARIANTS.iter()
                .find(|(k, _)| *k == category)
                .map(|(_, v)| *v)
                .unwrap_or("card")
        }
        None => "cards",  // Fallback for non-numeric
    };

    format!("Draw {} {}.", n, card_text)
}
```

### Tag-Based Selection

When selecting based on a phrase's tag:

```rust
// RLT:
destroyed = { masc: "destruido", fem: "destruida" };
destroy(target) = "{target} fue {destroyed:target}.";

// Generated:
pub fn destroy(target: impl Into<Value>) -> String {
    let target = target.into();

    // Try to get a matching tag from target
    let destroyed_text = if let Some(_) = target.get_tag("fem") {
        "destruida"
    } else if let Some(_) = target.get_tag("masc") {
        "destruido"
    } else {
        "destruido"  // Default fallback
    };

    format!("{} fue {}.", target, destroyed_text)
}
```

### Transform Code Generation

Transforms are compiled to runtime function calls:

```rust
// RLT:
card = "card" :a;
draw_one = "Draw {@a card}.";

// Generated:
pub fn draw_one() -> String {
    let card = card();
    let card_with_article = transform_a_en(card.into());
    format!("Draw {}.", card_with_article)
}

fn transform_a_en(value: Value) -> Result<String, RltError> {
    let text = value.to_string();

    // Check for explicit tag - required for @a transform
    if value.has_tag("a") {
        return Ok(format!("a {}", text));
    }
    if value.has_tag("an") {
        return Ok(format!("an {}", text));
    }

    // No heuristics - missing tag is an error
    Err(RltError::MissingRequiredTag {
        transform: "@a",
        expected_tags: &["a", "an"],
        value: text,
    })
}
```

### Module Structure

Each `.rlt.rs` file generates a module:

```rust
// en.rlt.rs generates:
pub mod en {
    use super::*;

    pub fn hello() -> PhraseRef { ... }
    pub fn draw(n: impl Into<Value>) -> String { ... }
    // etc.
}
```

The module name is derived from the filename (`en.rlt.rs` → `en`).

---

## Error Handling

RLT reports errors as Rust compile-time errors with precise source locations.

### Error Categories

**Syntax Errors:**

```
error: expected '=' after phrase name
  --> en.rlt.rs:3:10
   |
3  |     hello "world";
   |          ^ expected '='
```

**Undefined Phrase Reference:**

```
error: unknown phrase 'cards'
  --> en.rlt.rs:5:28
   |
5  |     draw(n) = "Draw {n} {cards:n}.";
   |                          ^^^^^ not defined
   |
   = help: did you mean 'card'?
```

**Undefined Parameter:**

```
error: unknown parameter 'count'
  --> en.rlt.rs:2:18
   |
2  |     draw(n) = "Draw {count} cards.";
   |                      ^^^^^ not in parameter list
   |
   = help: declared parameters: n
```

**Invalid Literal Selector:**

```
error: phrase 'card' has no variant 'accusative'
  --> en.rlt.rs:5:22
   |
5  |     take = "{card:accusative}";
   |                  ^^^^^^^^^^^ variant not defined
   |
   = note: available variants: one, other
```

### Span Preservation

The macro preserves source spans through parsing so errors point to the exact
location in the `.rlt.rs` file:

```rust
struct Identifier {
    name: String,
    span: Span,  // proc_macro2::Span
}
```

### Helpful Suggestions

For typos, the macro suggests similar names:

```rust
fn suggest_similar(name: &str, candidates: &[&str]) -> Option<&str> {
    candidates.iter()
        .filter(|c| levenshtein_distance(name, c) <= 2)
        .min_by_key(|c| levenshtein_distance(name, c))
        .copied()
}
```

---

## Cross-File Validation

A build script validates that all language files define the same phrases with
the same parameter counts.

### Build Script Operation

1. **Discover files**: Find all `*.rlt.rs` files in `src/localization/`
2. **Parse each file**: Extract phrase names and parameter counts
3. **Compare to source**: Check each file against the source language
4. **Report mismatches**: Generate compile errors for missing/extra phrases

### Validation Rules

**Missing Phrase:**

```
error: phrase 'draw' not defined in ru.rlt.rs
  --> en.rlt.rs:5:5
   |
5  |     draw(n) = "Draw {n} {card:n}.";
   |     ^^^^ defined in source language
   |
   = help: add to ru.rlt.rs:
   |     draw(n) = "...";
```

**Parameter Count Mismatch:**

```
error: phrase 'greet' has different parameter count in fr.rlt.rs
  --> en.rlt.rs:3:5
   |
3  |     greet(name) = "Hello, {name}!";
   |     ^^^^^^^^^^^  1 parameter in source
   |
  --> fr.rlt.rs:3:5
   |
3  |     greet(nom, titre) = "Bonjour, {titre} {nom}!";
   |     ^^^^^^^^^^^^^^^^^  2 parameters in translation
```

---

## Runtime Components

### CLDR Plural Rules

Each language has a plural category function:

```rust
fn cldr_plural_en(n: i64) -> &'static str {
    if n == 1 { "one" } else { "other" }
}

fn cldr_plural_ru(n: i64) -> &'static str {
    let n_mod_10 = n.abs() % 10;
    let n_mod_100 = n.abs() % 100;

    if n_mod_10 == 1 && n_mod_100 != 11 {
        "one"
    } else if n_mod_10 >= 2 && n_mod_10 <= 4
              && !(n_mod_100 >= 12 && n_mod_100 <= 14) {
        "few"
    } else {
        "many"
    }
}

fn cldr_plural_ar(n: i64) -> &'static str {
    match n {
        0 => "zero",
        1 => "one",
        2 => "two",
        _ => {
            let n_mod_100 = n.abs() % 100;
            if n_mod_100 >= 3 && n_mod_100 <= 10 {
                "few"
            } else if n_mod_100 >= 11 && n_mod_100 <= 99 {
                "many"
            } else {
                "other"
            }
        }
    }
}
```

### Universal Transforms

```rust
pub fn transform_cap(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub fn transform_upper(s: &str) -> String {
    s.to_uppercase()
}

pub fn transform_lower(s: &str) -> String {
    s.to_lowercase()
}
```

### Language-Specific Transforms

Each language file includes its transform implementations:

```rust
// en transforms
fn transform_a_en(value: Value) -> Result<String, RltError> {
    let text = value.to_string();

    // Explicit tag required - no heuristics
    if value.has_tag("an") {
        Ok(format!("an {}", text))
    } else if value.has_tag("a") {
        Ok(format!("a {}", text))
    } else {
        Err(RltError::MissingRequiredTag {
            transform: "@a",
            expected_tags: &["a", "an"],
            value: text,
        })
    }
}

// zh_cn transforms
fn transform_count_zh_cn(n: Value, thing: Value) -> String {
    let num = n.as_number().unwrap_or(1);
    let text = thing.to_string();

    let measure_word = thing.get_tag("zhang").map(|_| "张")
        .or(thing.get_tag("ge").map(|_| "个"))
        .or(thing.get_tag("ming").map(|_| "名"))
        .unwrap_or("个");

    format!("{}{}{}", num, measure_word, text)
}
```

---

## Generated API Patterns

### Language-Specific Access

```rust
use localization::en;
use localization::ru;

let msg_en = en::draw(3);  // "Draw 3 cards."
let msg_ru = ru::draw(3);  // "Возьмите 3 карты."
```

### Language-Agnostic Access

The build script generates a unified API:

```rust
pub enum Language { En, Ru, ZhCn, Es, /* ... */ }

pub mod messages {
    use super::*;

    pub fn draw(lang: Language, n: impl Into<Value>) -> String {
        match lang {
            Language::En => en::draw(n),
            Language::Ru => ru::draw(n),
            Language::ZhCn => zh_cn::draw(n),
            // ...
        }
    }
}
```

Usage:

```rust
use localization::{Language, messages};

let msg = messages::draw(Language::Ru, 3);  // "Возьмите 3 карты."
```

---

## Performance Considerations

### Static Data

Phrase text and variant tables are static:

```rust
static CARD_VARIANTS: &[(&str, &str)] = &[
    ("one", "card"),
    ("other", "cards"),
];
```

### Inline Simple Functions

Accessors are `#[inline]`:

```rust
#[inline]
pub fn card() -> PhraseRef {
    PhraseRef { text: "card", variants: CARD_VARIANTS, tags: &[] }
}
```

### Minimal Allocations

- Phrases without parameters return `PhraseRef` (no allocation)
- Phrases with parameters return `String` (one allocation)
- The `Value` type uses `String` for string values (allocation on conversion)

### No External Dependencies

All plural rules and transforms are compiled into the binary. No runtime data
file loading.

---

## Runtime Errors

Metadata-driven transforms produce errors when required tags are missing:

```rust
pub enum RltError {
    /// Transform requires metadata tag that wasn't present
    MissingRequiredTag {
        transform: &'static str,
        expected_tags: &'static [&'static str],
        value: String,
    },
}
```

**Example error message:**

```
RLT error: transform '@a' requires one of tags [:a, :an] but value "uniform" has no matching tag

Hint: Add a tag to the phrase definition:
    uniform = "uniform" :a;
```

This ensures predictable output. Heuristics like "check if starts with vowel"
would silently produce wrong results for words like:
- "uniform" (starts with vowel but uses "a" due to /juː/ sound)
- "hour" (starts with consonant but uses "an" due to silent h)
- "European" (starts with vowel but uses "a" due to /juː/ sound)
- "honest" (starts with consonant but uses "an" due to silent h)

---

## Summary

| Phase           | What Happens                              |
| --------------- | ----------------------------------------- |
| Parsing         | Token stream → Internal AST               |
| Validation      | Check phrase/parameter names exist        |
| Code Generation | AST → Rust functions with runtime dispatch|
| Build Script    | Cross-file consistency validation         |

The design prioritizes:

- **Simplicity**: No type inference, no complex generics
- **Flexibility**: Any value type works at runtime
- **Safety**: Name errors caught at compile time
- **Predictability**: Transforms error on missing tags rather than guessing
- **Graceful degradation**: Selection operations use fallbacks for type mismatches
