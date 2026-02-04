# RLF

The Rust Localization Framework: a localization DSL embedded in Rust via macros.

## Overview

RLF generates a **language-agnostic API** from phrase definitions. The source
language (typically English) is compiled via the `rlf!` macro into Rust
functions. All other languages are loaded at runtime via the interpreter.

```rust
// strings.rlf.rs - The source language (English)
rlf! {
    hello = "Hello, world!";
    draw(n) = "Draw {n} {card:n}.";
}
```

This generates functions that take a `Locale` parameter:

```rust
// Generated API - usage
let mut locale = Locale::with_language("en");
strings::hello(&locale);       // → "Hello, world!"
strings::draw(&locale, 3);     // → "Draw 3 cards."

// Switch to Russian
locale.set_language("ru");
strings::draw(&locale, 3);     // → "Возьмите 3 карты." (via interpreter)
```

**How it works:**

1. The `rlf!` macro parses the source language and generates one function per phrase
2. The macro also embeds the source phrases as data for the interpreter
3. At startup, the source phrases are registered with the interpreter
4. All evaluation (source and translations) goes through the interpreter

**Key benefit:** When you add a new phrase to `strings.rlf.rs`, it immediately
appears in IDE autocomplete for all Rust code. No build steps, no external tools—
just write the phrase and use it.

---

## Primitives

RLF has four primitives: **phrase**, **parameter**, **variant**, and
**selection**.

### Phrase

A phrase has a name and produces text.

```rust
rlf! {
    hello = "Hello, world!";
    goodbye = "Goodbye!";
}
```

### Parameter

Phrases can accept values. Parameters are declared in parentheses and
interpolated with `{}`.

```rust
rlf! {
    greet(name) = "Hello, {name}!";
    damage(amount, target) = "Deal {amount} damage to {target}.";
}
```

### Variant

A phrase can have multiple forms. Variants are declared in braces after `=`.

```rust
rlf! {
    card = {
        one: "card",
        other: "cards",
    };
}
```

Variants can be multi-dimensional using dot notation:

```rust
// In ru.rlf
card = {
    nom.one: "карта",
    nom.few: "карты",
    nom.many: "карт",
    acc.one: "карту",
    acc.few: "карты",
    acc.many: "карт",
};
```

**Multi-key shorthand:** Assign the same value to multiple keys with commas:

```rust
rlf! {
    card = {
        nom.one, acc.one: "card",
        nom.other, acc.other: "cards",
    };
}
```

**Wildcard fallbacks:** Omit the final dimension to create a fallback:

```rust
// In ru.rlf
card = {
    nom: "карта",        // Fallback for nom.one, nom.few, etc.
    nom.many: "карт",    // Override for nom.many specifically
    acc: "карту",
    acc.many: "карт",
};
```

Resolution order: exact match (`nom.many`) → progressively shorter fallbacks
(`nom`). If no match is found, RLF produces a **runtime error**.

**Irregular forms:** Use variants for unpredictable forms:

```rust
rlf! {
    go = { present: "go", past: "went", participle: "gone" };
    good = { base: "good", comparative: "better", superlative: "best" };
}
```

### Selection

The `:` operator selects a variant.

Literal selection uses a variant name directly:

```rust
rlf! {
    all_cards = "All {card:other}.";
}

// In ru.rlf
take_one = "Возьмите {card:acc.one}.";
```

Derived selection uses a parameter. For numbers, RLF maps to CLDR plural
categories (`zero`, `one`, `two`, `few`, `many`, `other`):

```rust
rlf! {
    draw(n) = "Draw {n} {card:n}.";
}
// n=1 → "Draw 1 card."
// n=5 → "Draw 5 cards."
```

**Escape sequences:** Use doubled characters for literals:

```rust
rlf! {
    syntax_help = "Use {{name}} for interpolation and @@ for transforms.";
    ratio = "The ratio is 1::2.";
}
// → "Use {name} for interpolation and @ for transforms."
// → "The ratio is 1:2."
```

Multi-dimensional selection chains with multiple `:` operators:

```rust
// In ru.rlf
draw(n) = "Возьмите {n} {card:acc:n}.";
// n=1 → "Возьмите 1 карту."
// n=5 → "Возьмите 5 карт."
```

**Selection on phrase parameters:**

```rust
rlf! {
    character = { one: "character", other: "characters" };
    with_cost_less_than_allies(base, counting) =
        "{base} with cost less than the number of allied {counting:other}";
}
// counting=character → "... allied characters"
```

**Dynamic selection:**

```rust
rlf! {
    character = { one: "character", other: "characters" };
    card = { one: "card", other: "cards" };
    draw_entities(n, entity) = "Draw {n} {entity:n}.";
}
// draw_entities(1, character) → "Draw 1 character."
// draw_entities(3, card) → "Draw 3 cards."
```

---

## Metadata Tags

A phrase can declare metadata tags using `:` before its content:

```rust
// In es.rlf
card = :fem "carta";
character = :masc "personaje";
```

Tags serve two purposes:

1. **Selection**: Other phrases can select variants based on tags
2. **Transforms**: Transforms can read tags to determine behavior

**Multiple tags:**

```rust
rlf! {
    // English: article hint for @a transform
    card = :a "card";
    event = :an "event";
    uniform = :a "uniform";   // phonetic exception
    hour = :an "hour";        // silent h exception
}

// In de.rlf (German)
karte = :fem "Karte";
charakter = :masc "Charakter";
ereignis = :neut "Ereignis";

// In zh_cn.rlf (Chinese)
pai = :zhang "牌";
jue_se = :ge "角色";
```

**Selection based on tags:**

```rust
// In es.rlf
card = :fem "carta";
character = :masc "personaje";

destroyed = {
    masc: "destruido",
    fem: "destruida",
};

destroy(thing) = "{thing} fue {destroyed:thing}.";
// thing=card      → "carta fue destruida."
// thing=character → "personaje fue destruido."
```

---

## Transforms

The `@` operator applies a transform. Transforms modify text and apply
right-to-left when chained:

```rust
rlf! {
    card = "card";
    draw_one = "Draw {@a card}.";        // → "Draw a card."
    title = "{@cap card}";               // → "Card"
    heading = "{@cap @a card}";          // → "A card"
}
```

**Automatic capitalization:** Referencing a phrase with an uppercase first letter
applies `@cap` automatically: `{Card}` is equivalent to `{@cap card}`.

Transforms combine with selection:

```rust
rlf! {
    card = { one: "card", other: "cards" };
    draw(n) = "Draw {n} {@cap card:n}.";
}
// n=1 → "Draw 1 Card."
// n=3 → "Draw 3 Cards."
```

**Transform context:** Some transforms need additional information:

```rust
// In de.rlf
destroy_card = "Zerstöre {@der:acc karte}.";      // :acc is literal context

// In es.rlf
return_all(t) = "devuelve {@el:other t} a mano";  // :other is literal context

// In zh_cn.rlf
draw(n) = "抽{@count:n card}";                    // :n is parameter context
```

The first `:` after the transform name is context; subsequent `:` apply to the
phrase:

```rust
get_card = "Nimm {@der:acc karte:one}.";   // :acc is context; :one selects variant
```

### Universal Transforms

| Transform | Effect                   |
| --------- | ------------------------ |
| `@cap`    | Capitalize first letter  |
| `@upper`  | All uppercase            |
| `@lower`  | All lowercase            |

### Metadata-Driven Transforms

Language-specific transforms read metadata tags:

```rust
rlf! {
    card = :a "card";
    event = :an "event";
    draw_one = "Draw {@a card}.";   // → "Draw a card."
    play_one = "Play {@a event}.";  // → "Play an event."
}
```

The `@a` transform reads the `:a` or `:an` tag. Missing tags produce runtime
errors—no phonetic guessing.

Standard transforms per language:

| Transform   | Languages              | Reads Tags                   | Effect                     |
| ----------- | ---------------------- | ---------------------------- | -------------------------- |
| `@a`        | English                | `:a`, `:an`                  | Indefinite article         |
| `@der`      | German                 | `:masc`, `:fem`, `:neut`     | Definite article + case    |
| `@el`       | Spanish                | `:masc`, `:fem`              | Definite article           |
| `@le`       | French                 | `:masc`, `:fem`, `:vowel`    | Definite article           |
| `@un`       | Romance                | `:masc`, `:fem`              | Indefinite article         |
| `@count`    | CJK                    | measure word tags            | Measure word insertion     |

**Transform aliases:** `@an` → `@a`, `@die` → `@der`, `@la` → `@el`, etc.

See **APPENDIX_STDLIB.md** for complete documentation.

---

## File Structure

```
src/
  localization/
    mod.rs
    strings.rlf.rs     # Source language (English) - uses rlf!
  assets/
    localization/
      ru.rlf           # Russian translation - loaded at runtime
      es.rlf           # Spanish translation - loaded at runtime
      zh_cn.rlf        # Chinese translation - loaded at runtime
```

The source language (`strings.rlf.rs`) defines the API via the `rlf!` macro.
Translation files (`.rlf`) use the same syntax but are loaded by the interpreter
at runtime.

**Translation file format:**

```
// Comment
hello = "Привет, мир!";
card = :fem { one: "карта", few: "карты", many: "карт" };
draw(n) = "Возьмите {n} {card:n}.";
```

---

## The Locale Object

The `Locale` object manages language selection and translation data:

```rust
pub struct Locale {
    /// Current language code (e.g., "en", "ru", "es")
    current_language: String,

    /// The source language code (typically "en")
    source_language: &'static str,

    /// Interpreter with loaded translations
    interpreter: RlfInterpreter,
}

impl Locale {
    /// Create a new locale with the source language selected.
    pub fn new() -> Self;

    /// Create a locale with a specific language selected.
    pub fn with_language(language: &str) -> Self;

    /// Get the current language.
    pub fn language(&self) -> &str;

    /// Set the current language.
    pub fn set_language(&mut self, language: &str);

    /// Check if the current language is the source language.
    pub fn is_source(&self) -> bool;

    /// Load translations from a file.
    pub fn load_translations(&mut self, language: &str, path: impl AsRef<Path>) -> Result<(), LoadError>;

    /// Load translations from embedded data.
    pub fn load_translations_str(&mut self, language: &str, content: &str) -> Result<(), LoadError>;

    /// Reload translations for a language (clears existing and reloads from original source).
    pub fn reload_translations(&mut self, language: &str) -> Result<(), LoadError>;
}
```

**Typical initialization:**

```rust
fn setup_localization() -> Locale {
    let mut locale = Locale::new();

    // Register source language phrases (from macro-embedded data)
    strings::register_source_phrases(locale.interpreter_mut());

    // Load translation files
    locale.load_translations("ru", "assets/localization/ru.rlf")?;
    locale.load_translations("es", "assets/localization/es.rlf")?;
    locale.load_translations("zh_cn", "assets/localization/zh_cn.rlf")?;

    // Set initial language from user preferences
    locale.set_language(&user_preferences.language);

    locale
}
```

---

## Generated API

Given:

```rust
// strings.rlf.rs
rlf! {
    card = { one: "card", other: "cards" };
    draw(n) = "Draw {n} {card:n}.";
}
```

RLF generates:

```rust
// strings.rs (generated)

/// Returns the "card" phrase.
pub fn card(locale: &Locale) -> Phrase {
    locale.interpreter()
        .get_phrase(locale.language(), "card")
        .expect("phrase 'card' should exist")
}

/// Evaluates the "draw" phrase with parameter n.
pub fn draw(locale: &Locale, n: impl Into<Value>) -> String {
    locale.interpreter()
        .call_phrase(locale.language(), "draw", &[n.into()])
        .expect("phrase 'draw' should exist")
}

/// Registers source language phrases with the interpreter.
/// Call once at startup.
pub fn register_source_phrases(interpreter: &mut RlfInterpreter) {
    interpreter.load_phrases("en", SOURCE_PHRASES)
        .expect("source phrases should parse successfully");
}

const SOURCE_PHRASES: &str = r#"
    card = { one: "card", other: "cards" };
    draw(n) = "Draw {n} {card:n}.";
"#;
```

**Usage:**

```rust
use localization::strings;

fn render_card_text(locale: &Locale) {
    let text = strings::draw(locale, 3);
    // English: "Draw 3 cards."
    // Russian: "Возьмите 3 карты."
}
```

---

## Runtime Templates

For data-driven content (templates stored in data files), use the interpreter directly:

```rust
let template = "Draw {cards(n)} for each {target}.";
let params = hashmap!{ "n" => 2, "target" => strings::ally(&locale) };
locale.interpreter().eval_str(template, locale.language(), params)?
```

Parameters work identically to phrase parameters. See **APPENDIX_RUNTIME_INTERPRETER.md**.

---

## Runtime Values

All parameters accept a `Value` type:

```rust
strings::draw(&locale, 3);                     // number
strings::draw(&locale, "3");                   // string
strings::greet(&locale, "World");              // string
strings::destroy(&locale, strings::card(&locale));  // phrase
```

**Runtime behavior:**

| Operation              | Value Type      | Behavior                                    |
| ---------------------- | --------------- | ------------------------------------------- |
| `{x}`                  | Any             | Display the value                           |
| `{card:x}` (selection) | Number          | Select plural category                      |
| `{card:x}` (selection) | String          | Parse as number, or error                   |
| `{card:x}` (selection) | Phrase          | Look up matching tag                        |
| `{@a x}`               | Phrase with tag | Use the tag                                 |
| `{@a x}`               | Other           | **Runtime error**                           |

---

## Compile-Time Errors

RLF validates the source file at compile time:

**Unknown phrase:**

```rust
rlf! {
    draw(n) = "Draw {n} {cards:n}.";  // typo
}
```

```
error: unknown phrase 'cards'
  --> strings.rlf.rs:2:28
   |
   = help: did you mean 'card'?
```

**Unknown parameter:**

```rust
rlf! {
    draw(n) = "Draw {count} {card:n}.";
}
```

```
error: unknown parameter 'count'
  --> strings.rlf.rs:2:18
   |
   = help: declared parameters: n
```

**Additional compile-time checks:**

- **Cyclic references**: Phrases that reference each other in a cycle are rejected
- **Parameter shadowing**: A parameter cannot have the same name as a phrase

**Translation files** are validated at load time, not compile time. Load errors
include the file path and line number.

---

## Runtime Errors

Generated functions use `.expect()` and **panic** on evaluation errors. This is
intentional—these are programming errors (missing phrase, wrong argument count,
missing tag) that should be caught during development. The underlying interpreter
returns `Result` for testing and tooling; see **APPENDIX_RUST_INTEGRATION.md**.

---

## Phrase Type

Phrases without parameters return a `Phrase` that carries metadata:

```rust
pub struct Phrase {
    /// Default text.
    pub text: String,
    /// Variant key → variant text.
    pub variants: HashMap<String, String>,
    /// Metadata tags.
    pub tags: Vec<String>,
}

impl Phrase {
    /// Get a specific variant by key, with fallback resolution.
    /// Tries exact match first, then progressively shorter keys.
    /// Panics if no match found.
    pub fn variant(&self, key: &str) -> &str;
}

impl Display for Phrase {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}
```

Use `variant()` to access specific forms:

```rust
let card = strings::card(&locale);
let singular = card.to_string();            // "card"
let plural = card.variant("other");         // "cards"
```

---

## Phrase Identifiers

For scenarios where you need to store a reference to a phrase in serializable
data structures, RLF provides `PhraseId`—a compact, `Copy`-able, 8-byte
identifier based on a hash of the phrase name. The `rlf!` macro generates
`PhraseId` constants for all phrases.

```rust
// Store in serializable data
let card_name: PhraseId = strings::phrase_ids::FIRE_ELEMENTAL;
let draw_phrase: PhraseId = strings::phrase_ids::DRAW;

// Resolve parameterless phrase (returns Result<Phrase, EvalError>)
let phrase = card_name.resolve(&locale).expect("phrase should exist");
let text = phrase.to_string();  // → "Fire Elemental"

// Resolve phrase with parameters (returns Result<String, EvalError>)
let text = draw_phrase.call(&locale, &[3.into()])
    .expect("phrase should exist");  // → "Draw 3 cards."
```

See **APPENDIX_RUST_INTEGRATION.md** for complete details on `PhraseId`
generation, API, and usage patterns.

---

## Design Philosophy

**Unified interpreter, compile-time validation.** All languages (including the
source) are evaluated by the interpreter at runtime. The source language gets
full compile-time syntax and reference checking via the macro. Translations are
loaded at runtime, enabling hot-reloading and community translations.

**Immediate IDE support.** When you add a phrase to `strings.rlf.rs`, it
appears in autocomplete immediately. No external tools, no build steps.

**Language-agnostic API.** Functions take a locale parameter. The same code
works for all languages—Rust identifies what to say, RLF handles how to say it.

**Pass Phrase, not String.** When composing phrases, pass `Phrase` values rather
than pre-rendered strings. This preserves variants and tags so RLF can select
the correct grammatical form. Pre-rendering to `String` strips this metadata.

**Logic in Rust, text in RLF.** Complex branching stays in Rust; RLF provides
atomic text pieces. Translators don't need to understand Rust.

**Keywords and formatting are phrases.** No special syntax—define phrases with
markup (`dissolve = "<k>dissolve</k>";`) and interpolate normally.

**Dynamic typing for simplicity.** Parameters accept any `Value`. Runtime errors
catch type mismatches. Translators don't need Rust types.

---

## Translation Workflow

### Adding a New Phrase

1. Add the phrase to `strings.rlf.rs`:
   ```rust
   rlf! {
       new_ability(n) = "Gain {n} {point:n}.";
   }
   ```

2. Use it immediately in Rust code (autocomplete works):
   ```rust
   let text = strings::new_ability(&locale, 5);
   ```

3. Later, add translations to `.rlf` files:
   ```
   // ru.rlf
   new_ability(n) = "Получите {n} {point:n}.";
   ```

### Updating Translations

1. Edit the `.rlf` file
2. Reload in development (if hot-reload enabled) or restart
3. Changes take effect without recompilation

### Command-Line Tools

The `rlf` binary provides utilities for working with translation files:

```bash
# Validate syntax
rlf check assets/localization/ru.rlf

# Check coverage against source
rlf coverage --source strings.rlf.rs --lang ru,es,zh_cn

# Evaluate a template interactively
rlf eval --lang ru --param n=3 --template "Draw {n} {card:n}."
```

---

## Summary

| Primitive    | Syntax                         | Purpose                                 |
| ------------ | ------------------------------ | --------------------------------------- |
| Phrase       | `name = "text";`               | Define text                             |
| Parameter    | `name(p) = "{p}";`             | Accept values                           |
| Variant      | `name = { a: "x", b: "y" };`   | Multiple forms                          |
| Selection    | `{phrase:selector}`            | Choose a variant                        |
| Metadata tag | `name = :tag "text";`          | Attach metadata                         |
| Transform    | `{@transform:ctx phrase}`      | Modify text                             |

| File Type        | Extension    | Purpose                               |
| ---------------- | ------------ | ------------------------------------- |
| Source language  | `.rlf.rs`    | Compiled via `rlf!` macro             |
| Translations     | `.rlf`       | Loaded at runtime via interpreter     |

| Type             | Purpose                                | Size / Traits                       |
| ---------------- | -------------------------------------- | ----------------------------------- |
| `Phrase`         | Text with variants and tags            | Heap-allocated                      |
| `PhraseId`       | Serializable reference to a phrase     | 8 bytes, `Copy`, `Serialize`        |
| `Value`          | Runtime parameter (number/string/phrase) | Enum                              |

| Component        | Compile-Time              | Runtime                             |
| ---------------- | ------------------------- | ----------------------------------- |
| Source language  | Full validation           | Interpreter evaluation              |
| Translations     | (optional strict check)   | Interpreter evaluation              |

Four primitives, one macro, Rust-compatible syntax, compile-time checking for
the source language, runtime loading for translations, immediate IDE support.
