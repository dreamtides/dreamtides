# Appendix: Rust Integration

This appendix describes how the `rlt_source!` and `rlt_lang!` macros are
implemented, what Rust code they generate, and how errors are reported.

## Macro Architecture

RLT uses Rust procedural macros to parse `.rlt.rs` files and generate Rust
code. There are two macros:

- **`rlt_source!`**: For the source language (typically English). Generates
  a trait with default implementations and a unit struct.
- **`rlt_lang!(Name)`**: For translation languages. Generates a unit struct
  and a trait implementation.

The process has three phases:

1. **Parsing**: Extract phrase definitions from the macro input
2. **Validation**: Check names, detect undefined references
3. **Code Generation**: Emit Rust functions, trait, and implementations

Cross-language validation (ensuring translations define the same phrases) is
handled by Rust's trait system—missing methods cause compile errors when
`strict-i18n` is enabled.

---

## Phase 1: Parsing

The macro receives tokens from inside the `rlt_source! { ... }` or
`rlt_lang!(Name) { ... }` block. It parses these into an internal representation:

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

### Multi-Key and Wildcard Parsing

Multi-key shorthand expands during parsing:

```rust
// Multi-key syntax
card = { nom, acc: "card", nom.other, acc.other: "cards" };
// Expands to: [("nom", "card"), ("acc", "card"), ("nom.other", "cards"), ...]
```

Wildcard fallbacks use partial keys:

```rust
// Wildcard syntax
card = { nom: "card", nom.other: "cards" };
// Stored as: [("nom", "card"), ("nom.other", "cards")]
// At runtime: nom.one → try "nom.one" (miss) → try "nom" (hit) → "card"
```

---

## Phase 2: Validation

After parsing, the macro validates names and references. This is purely
compile-time checking—no type inference or type checking occurs.

### Name Resolution

Within phrase text, names in `{}` are resolved using these rules:

1. **Parameters first**: If a name matches a declared parameter, it refers to that parameter
2. **Phrases second**: Otherwise, it refers to a phrase defined in the file

**No shadowing allowed:** It is a compile error for a parameter to have the same
name as a phrase. This eliminates ambiguity without requiring special syntax.

```rust
rlt_source! {
    card = "card";

    // ERROR: parameter 'card' shadows phrase 'card'
    play(card) = "Play {card}.";

    // OK: use a different parameter name
    play(c) = "Play {c}.";
}
```

**Selectors follow the same rules:** A selector like `:n` is dynamic (parameter)
if `n` is in the parameter list, otherwise it's a literal variant name.

```rust
rlt_source! {
    card = { one: "card", other: "cards" };

    // 'other' is literal (no parameter named 'other')
    all_cards = "All {card:other}.";

    // 'n' is dynamic (matches parameter)
    draw(n) = "Draw {card:n}.";
}
```

The validator checks that all referenced names exist:

1. **Phrase references**: Every `{phrase_name}` must refer to a defined phrase
2. **Parameter references**: Every `{param}` in a phrase body must be in that
   phrase's parameter list
3. **Selector references**: Every `{phrase:selector}` must use a valid variant
   name (for literal selectors)

### Undefined Reference Detection

The validator builds a set of all phrase names and checks each reference. Errors
include source spans for precise IDE highlighting. For literal selectors like
`{card:accusative}`, the validator also checks that the variant exists in the
phrase's definition. Parameter selectors (`{card:n}`) are validated at runtime.

---

## Phase 3: Code Generation

Based on the validated AST, the macro generates Rust code. The output differs
between `rlt_source!` and `rlt_lang!`.

### The Value Type

All parameters use a single `Value` type that handles runtime dispatch:

```rust
pub enum Value {
    Number(i64),
    Float(f64),
    String(String),
    Phrase(Phrase),
}
```

`Value` provides methods for runtime operations: `as_number()` for plural
selection (returns `None` for phrases), `has_tag(&str)` and `get_variant(&str)`
for tag-based selection (only work on `Phrase` values). The `Display` impl
renders the value's text representation.

### The Phrase Type

Phrases without parameters return a `Phrase` that carries metadata for runtime
operations:

```rust
pub struct Phrase {
    /// Default text (used when displaying without variant selection).
    pub text: Cow<'static, str>,
    /// Variant key → variant text. Keys use dot notation: "nom.one".
    pub variants: HashMap<&'static str, Cow<'static, str>>,
    /// Metadata tags attached to this phrase.
    pub tags: &'static [&'static str],
}

impl Phrase {
    /// Get a specific variant by key, with fallback resolution.
    pub fn variant(&self, key: &str) -> Result<&str, RltError> {
        resolve_variant(&self.variants, key)
    }
}

impl std::fmt::Display for Phrase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}
```

The `Cow<'static, str>` type allows phrases to use static strings when no
interpolation occurs, while supporting owned strings when phrases contain
dynamic content.

### Into<Value> Implementations

Common types implement `Into<Value>` for convenient calling: integers (`i32`,
`i64`, `u32`, etc.) become `Value::Number`, strings (`&str`, `String`) become
`Value::String`, and `Phrase` becomes `Value::Phrase`.

---

## Code Generation: rlt_source!

The source language macro generates two things:

1. A trait with default implementations
2. A unit struct that implements the trait

### Example

```rust
// en.rlt.rs
rlt_source! {
    card = { one: "card", other: "cards" };
    draw(n) = "Draw {n} {card:n}.";
}
```

### Generated Trait

The trait has default implementations for the source language:

```rust
#[cfg(not(feature = "strict-i18n"))]
pub trait RltLang {
    fn card(&self) -> Phrase {
        static VARIANTS: phf::Map<&'static str, &'static str> = phf_map! {
            "one" => "card",
            "other" => "cards",
        };
        Phrase {
            text: Cow::Borrowed("card"),
            variants: &VARIANTS,
            tags: &[],
        }
    }

    fn draw(&self, n: impl Into<Value>) -> Result<String, RltError> {
        let n = n.into();
        let category = plural_category("en", n.as_number().ok_or_else(|| {
            RltError::InvalidSelector { phrase: "card", selector: n.to_string() }
        })?);
        let card_text = self.card().variant(category)?;
        Ok(format!("Draw {} {}.", n, card_text))
    }
}

#[cfg(feature = "strict-i18n")]
pub trait RltLang {
    fn card(&self) -> Phrase;
    fn draw(&self, n: impl Into<Value>) -> Result<String, RltError>;
}
```

With `strict-i18n` disabled (default), missing translations fall back to English.
With `strict-i18n` enabled, missing translations are compile errors.

### Generated Unit Struct

```rust
pub struct En;

impl RltLang for En {
    // Uses trait defaults (source language implementations)
}
```

---

## Code Generation: rlt_lang!

Translation macros generate a unit struct and trait implementation.

### Example

```rust
// ru.rlt.rs
rlt_lang!(Ru) {
    card = {
        one: "карта",
        few: "карты",
        many: "карт",
    };
    draw(n) = "Возьмите {n} {card:n}.";
}
```

### Generated Code

```rust
pub struct Ru;

impl RltLang for Ru {
    fn card(&self) -> Phrase {
        static VARIANTS: phf::Map<&'static str, &'static str> = phf_map! {
            "one" => "карта",
            "few" => "карты",
            "many" => "карт",
        };
        Phrase {
            text: Cow::Borrowed("карта"),
            variants: &VARIANTS,
            tags: &[],
        }
    }

    fn draw(&self, n: impl Into<Value>) -> Result<String, RltError> {
        let n = n.into();
        let category = plural_category("ru", n.as_number().ok_or_else(|| {
            RltError::InvalidSelector { phrase: "card", selector: n.to_string() }
        })?);
        let card_text = self.card().variant(category)?;
        Ok(format!("Возьмите {} {}.", n, card_text))
    }
}
```

### Incomplete Translations

If Russian only defines some phrases:

```rust
// ru.rlt.rs
rlt_lang!(Ru) {
    card = { ... };
    // 'draw' not defined
}
```

Generated code only overrides what's defined:

```rust
impl RltLang for Ru {
    fn card(&self) -> Phrase { ... }
    // draw() not overridden - uses trait default (English)
}
```

With `strict-i18n` enabled, this becomes a compile error because the trait
has no default for `draw()`.

---

## Selection Code Generation

Selection is resolved at runtime by examining the selector value. The runtime
tries exact matches first, then progressively shorter fallback keys. If no match
is found, it returns an error:

```rust
// RLT:
card = { nom: "card", nom.other: "cards" };
draw(n) = "Draw {n} {card:nom:n}.";

// Generated:
pub fn draw(n: impl Into<Value>) -> Result<String, RltError> {
    let n = n.into();

    let category = match n.as_number() {
        Some(num) => plural_category("en", num),  // "one" or "other"
        None => return Err(RltError::InvalidSelector {
            phrase: "card",
            selector: n.to_string(),
        }),
    };
    let full_key = format!("nom.{}", category);
    let card_text = resolve_variant(CARD_VARIANTS, &full_key)?;

    Ok(format!("Draw {} {}.", n, card_text))
}

fn resolve_variant(variants: &[(&str, &str)], key: &str) -> Result<&str, RltError> {
    // Try exact match: "nom.one"
    if let Some((_, v)) = variants.iter().find(|(k, _)| *k == key) {
        return Ok(v);
    }
    // Try fallback: "nom"
    if let Some(dot) = key.rfind('.') {
        let fallback = &key[..dot];
        if let Some((_, v)) = variants.iter().find(|(k, _)| *k == fallback) {
            return Ok(v);
        }
    }
    // No match found
    Err(RltError::MissingVariant {
        phrase: "card",
        requested: key.to_string(),
        available: variants.iter().map(|(k, _)| *k).collect(),
    })
}
```

### Tag-Based Selection

When selecting based on a phrase's tag, the generated code checks for matching
tags and returns an error if none match:

```rust
// RLT:
destroyed = { masc: "destruido", fem: "destruida" };
destroy(target) = "{target} fue {destroyed:target}.";

// Generated:
pub fn destroy(target: impl Into<Value>) -> Result<String, RltError> {
    let target = target.into();

    let destroyed_text = if target.has_tag("fem") {
        "destruida"
    } else if target.has_tag("masc") {
        "destruido"
    } else {
        return Err(RltError::MissingRequiredTag {
            operation: "destroyed",
            expected_tags: &["masc", "fem"],
            value: target.to_string(),
        });
    };

    Ok(format!("{} fue {}.", target, destroyed_text))
}
```

---

## Transform Code Generation

Transforms are compiled to runtime function calls:

```rust
// RLT:
card = :a "card";
draw_one = "Draw {@a card}.";

// Generated:
pub fn draw_one() -> Result<String, RltError> {
    let card = card();
    let card_with_article = transform_a_en(card.into())?;
    Ok(format!("Draw {}.", card_with_article))
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
        operation: "@a",
        expected_tags: &["a", "an"],
        value: text,
    })
}
```

### Transform Aliases

Aliases are resolved at compile time to their primary transform:

```rust
// Transform alias registry (per language)
static EN_TRANSFORM_ALIASES: &[(&str, &str)] = &[
    ("an", "a"),  // @an → @a
];

static DE_TRANSFORM_ALIASES: &[(&str, &str)] = &[
    ("die", "der"),  // @die → @der
    ("das", "der"),  // @das → @der
];
```

During code generation, aliases are replaced with their primary transform:

```rust
// RLT:
play_event = "Play {@an event}.";

// After alias resolution (generated code):
pub fn play_event() -> Result<String, RltError> {
    let event = event();
    let event_with_article = transform_a_en(event.into())?;  // @an → @a
    Ok(format!("Play {}.", event_with_article))
}
```

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

## Macro File Discovery

The `rlt_lang!` macro needs to know which trait to implement. This is handled
through Rust's module system, not cross-file macro communication:

1. **Source file generates trait**: `rlt_source!` in `en.rlt.rs` generates a
   `pub trait RltLang`.

2. **Translation files import trait**: Each translation file must import the
   trait via `use crate::localization::RltLang;` (or similar path).

3. **Macro uses imported trait**: `rlt_lang!(Ru)` generates `impl RltLang for Ru`,
   where `RltLang` refers to whatever trait is in scope.

This design requires no special cross-file communication. The trait name
`RltLang` is conventional; if you rename it, translation macros still work
because they implement whatever trait is imported.

```rust
// mod.rs
mod en;  // Contains rlt_source! → generates RltLang trait + En struct
mod ru;  // Contains rlt_lang!(Ru) → implements RltLang for Ru

pub use en::{En, RltLang};
pub use ru::Ru;
```

---

## Cross-Language Validation

RLT validates cross-language consistency via Rust's trait system, not via
macro introspection.

### How It Works

1. `rlt_source!` generates a trait with all phrase methods
2. `rlt_lang!` generates an `impl` block for that trait
3. If a translation is missing a phrase, the impl is incomplete

### With strict-i18n Disabled (Development)

The trait has default implementations, so missing phrases compile fine:

```rust
// Generated trait (strict-i18n disabled)
pub trait RltLang {
    fn card(&self) -> Phrase { ... }  // default (English)
    fn draw(&self, n: impl Into<Value>) -> Result<String, RltError> { ... }  // default
}

// Russian impl - only overrides card()
impl RltLang for Ru {
    fn card(&self) -> Phrase { ... }
    // draw() uses trait default (English)
}
```

### With strict-i18n Enabled (CI/Release)

The trait has no defaults, so missing phrases are compile errors:

```rust
// Generated trait (strict-i18n enabled)
pub trait RltLang {
    fn card(&self) -> Phrase;  // no default
    fn draw(&self, n: impl Into<Value>) -> Result<String, RltError>;  // no default
}

// Russian impl - missing draw() is an error
impl RltLang for Ru {
    fn card(&self) -> Phrase { ... }
    // ERROR: method `draw` not implemented
}
```

Error message:

```
error[E0046]: not all trait items implemented, missing: `draw`
  --> src/localization/ru.rlt.rs:1:1
   |
   = note: `draw` from trait `RltLang` is not implemented
```

### Parameter Count Validation

If a translation has wrong parameters, it won't match the trait signature:

```rust
// en.rlt.rs
rlt_source! {
    greet(name) = "Hello, {name}!";
}

// ru.rlt.rs
rlt_lang!(Ru) {
    greet(name, title) = "Привет, {title} {name}!";  // ERROR: wrong signature
}
```

The generated impl tries to define `fn greet(&self, name: impl Into<Value>, title: impl Into<Value>)`
but the trait expects `fn greet(&self, name: impl Into<Value>)`, causing a type error.

---

## Runtime Components

### ICU4X Dependencies

RLT uses crates from the [ICU4X](https://github.com/unicode-org/icu4x) project for
Unicode-compliant internationalization:

```toml
[dependencies]
icu_plurals = "2"
icu_locale_core = "2"
```

| Crate | Purpose |
|-------|---------|
| `icu_plurals` | CLDR plural rules for all languages |
| `icu_locale_core` | Locale identifiers and parsing |

### CLDR Plural Rules

RLT uses `icu_plurals` for plural category selection:

```rust
use icu_plurals::{PluralCategory, PluralRuleType, PluralRules};
use icu_locale_core::locale;

/// Returns the CLDR plural category as a string for variant selection.
fn plural_category(lang: &str, n: i64) -> &'static str {
    let locale = match lang {
        "en" => locale!("en"),
        "ru" => locale!("ru"),
        "ar" => locale!("ar"),
        "pl" => locale!("pl"),
        "zh" => locale!("zh"),
        // ... other languages
        _ => locale!("en"),
    };

    let rules = PluralRules::try_new(locale.into(), PluralRuleType::Cardinal)
        .expect("locale should be supported");

    match rules.category_for(n) {
        PluralCategory::Zero => "zero",
        PluralCategory::One => "one",
        PluralCategory::Two => "two",
        PluralCategory::Few => "few",
        PluralCategory::Many => "many",
        PluralCategory::Other => "other",
    }
}
```

This handles all CLDR plural rules correctly:

```rust
// English: one/other
plural_category("en", 1)  // → "one"
plural_category("en", 5)  // → "other"

// Russian: one/few/many
plural_category("ru", 1)  // → "one"
plural_category("ru", 2)  // → "few"
plural_category("ru", 5)  // → "many"
plural_category("ru", 21) // → "one"

// Arabic: zero/one/two/few/many/other
plural_category("ar", 0)  // → "zero"
plural_category("ar", 1)  // → "one"
plural_category("ar", 2)  // → "two"
plural_category("ar", 5)  // → "few"
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

Each language has its transform implementations:

```rust
// English @a transform
fn transform_a_en(value: Value) -> Result<String, RltError> {
    let text = value.to_string();

    if value.has_tag("an") {
        Ok(format!("an {}", text))
    } else if value.has_tag("a") {
        Ok(format!("a {}", text))
    } else {
        Err(RltError::MissingRequiredTag {
            operation: "@a",
            expected_tags: &["a", "an"],
            value: text,
        })
    }
}

// Chinese @count transform - context is the number, phrase has measure word tag
fn transform_count_zh_cn(context: Value, phrase: Value) -> Result<String, RltError> {
    let num = context.as_number().ok_or_else(|| RltError::InvalidSelector {
        phrase: "@count context",
        selector: context.to_string(),
    })?;
    let text = phrase.to_string();

    let measure_word = if phrase.has_tag("zhang") {
        "张"
    } else if phrase.has_tag("ge") {
        "个"
    } else if phrase.has_tag("ming") {
        "名"
    } else {
        return Err(RltError::MissingRequiredTag {
            operation: "@count",
            expected_tags: &["zhang", "ge", "ming"],
            value: text,
        });
    };

    Ok(format!("{}{}{}", num, measure_word, text))
}
```

---

## Runtime Errors

Selection and transforms produce errors when required variants or tags are missing:

```rust
pub enum RltError {
    /// Variant selection failed - no matching variant found
    MissingVariant {
        phrase: &'static str,
        requested: String,
        available: Vec<&'static str>,
    },

    /// Tag-based selection or transform requires a tag that wasn't present
    MissingRequiredTag {
        operation: &'static str,
        expected_tags: &'static [&'static str],
        value: String,
    },

    /// Selector value couldn't be interpreted (e.g., non-numeric string for plural)
    InvalidSelector {
        phrase: &'static str,
        selector: String,
    },
}
```

**Example error messages:**

Missing variant error:
```
RLT error: phrase 'card' has no variant matching 'nom.few'
  Available variants: nom.one, nom.other

Hint: Add the missing variant to the phrase definition, or check that
the selector value is correct.
```

Transform error:
```
RLT error: transform '@a' requires tag [:a, :an] but "uniform" has none

Hint: Add a tag to the phrase definition:
    uniform = :a "uniform";
```

Tag-based selection error:
```
RLT error: selection for 'destroyed' requires tag [:masc, :fem] but "sword" has none

Hint: Pass a phrase with the required tag:
    sword = :fem "sword";
```

Invalid selector error:
```
RLT error: cannot use "hello" as plural selector for 'card' (expected number)
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

### Minimal Allocations

- Phrases without parameters use `Cow::Borrowed` (no allocation)
- Phrases with interpolated parameters use `Cow::Owned` (one allocation)
- Phrases with parameters that produce strings allocate once for the result
- The `Value` type uses `String` for string values (allocation on conversion)

### Minimal External Dependencies

RLT depends on ICU4X for CLDR plural rules. ICU4X compiles locale data into the
binary by default—no runtime data file loading required. The `icu_plurals` crate
adds approximately 100KB to the binary with all locales included, or less with
locale subsetting via the `icu4x-datagen` tool.

---

## IDE Support

Because RLT uses proc-macros, rust-analyzer provides immediate feedback:
autocomplete for phrase functions, go-to-definition on phrase calls (navigates
to the macro invocation), and inline error highlighting for syntax errors and
undefined references.

---

## Summary

| Component | rlt_source! | rlt_lang!(Name) |
|-----------|-------------|-----------------|
| Struct | `pub struct En;` | `pub struct Name;` |
| Trait | `pub trait RltLang { ... }` | (uses existing trait) |
| Impl | `impl RltLang for En { ... }` | `impl RltLang for Name { ... }` |

| Feature | Development | CI (strict-i18n) |
|---------|-------------|------------------|
| Missing translations | Fall back to English | Compile error |
| Trait defaults | Yes | No |
| Cross-language validation | Runtime (fallback used) | Compile-time |

The design prioritizes:

- **Immediate feedback**: Proc-macros enable IDE autocomplete without build steps
- **Incremental translation**: Missing phrases fall back gracefully during development
- **Release safety**: `strict-i18n` ensures complete translations before release
- **Simplicity**: No build scripts, no external tools, just Rust macros
