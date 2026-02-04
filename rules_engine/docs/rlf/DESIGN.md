# RLF

The Rust Localization Framework: a localization DSL embedded in Rust via macros.

## Overview

RLF files are valid Rust source files with a `.rlf.rs` extension. They contain
macro invocations that define phrases for a single language.

The **source language** (typically English) uses `rlf_source!`:

```rust
// en.rlf.rs
rlf_source! {
    hello = "Hello, world!";
}
```

**Translation languages** use `rlf_lang!`:

```rust
// ru.rlf.rs
rlf_lang!(Ru) {
    hello = "Привет, мир!";
}
```

The source macro generates a trait (`RlfLang`) with default implementations.
Translation macros implement the trait, overriding only the phrases they define.
Missing translations automatically fall back to the source language.

---

## Primitives

RLF has four primitives: **phrase**, **parameter**, **variant**, and
**selection**.

### Phrase

A phrase has a name and produces text.

```rust
rlf_source! {
    hello = "Hello, world!";
    goodbye = "Goodbye!";
}
```

### Parameter

Phrases can accept values. Parameters are declared in parentheses and
interpolated with `{}`.

```rust
rlf_source! {
    greet(name) = "Hello, {name}!";
    damage(amount, target) = "Deal {amount} damage to {target}.";
}
```

### Variant

A phrase can have multiple forms. Variants are declared in braces after `=`.

```rust
rlf_source! {
    card = {
        one: "card",
        other: "cards",
    };
}
```

Variants can be multi-dimensional using dot notation:

```rust
rlf_lang!(Ru) {
    card = {
        nom.one: "карта",
        nom.few: "карты",
        nom.many: "карт",
        acc.one: "карту",
        acc.few: "карты",
        acc.many: "карт",
    };
}
```

**Multi-key shorthand:** Assign the same value to multiple keys with commas:

```rust
rlf_source! {
    card = {
        nom.one, acc.one: "card",
        nom.other, acc.other: "cards",
    };
}
```

**Wildcard fallbacks:** Omit the final dimension to create a fallback for any
unspecified sub-key:

```rust
rlf_lang!(Ru) {
    card = {
        nom: "карта",        // Fallback for nom.one, nom.few, etc.
        nom.many: "карт",    // Override for nom.many specifically
        acc: "карту",
        acc.many: "карт",
    };
}
```

Resolution order: exact match (`nom.many`) → progressively shorter fallbacks (`nom`).
If no match is found after trying all fallbacks, RLF produces a **runtime error**.
This catches bugs where a required variant is missing from a phrase definition.

**Irregular forms (suppletion):** Use variants for words with unpredictable forms:

```rust
rlf_source! {
    go = { present: "go", past: "went", participle: "gone" };
    good = { base: "good", comparative: "better", superlative: "best" };
}
```

These features combine for concise definitions:

```rust
rlf_lang!(Ru) {
    event = :neut :inan {
        nom, acc: "событие",
        nom.many, acc.many: "событий",
        gen: "события",
        gen.many: "событий",
        ins.one: "событием",
        ins: "событиями",
    };
}
```

### Selection

The `:` operator selects a variant.

Literal selection uses a variant name directly:

```rust
rlf_source! {
    all_cards = "All {card:other}.";
}

rlf_lang!(Ru) {
    take_one = "Возьмите {card:acc.one}.";
}
```

Derived selection uses a parameter. For numbers, RLF maps to CLDR plural
categories (`one`, `two`, `few`, `many`, `other`):

```rust
rlf_source! {
    draw(n) = "Draw {n} {card:n}.";
}
// n=1 → "Draw 1 card."
// n=5 → "Draw 5 cards."
```

**Escape sequences:** Use doubled characters to include literal `{`, `}`, `@`, or `:`
in phrase text:

```rust
rlf_source! {
    syntax_help = "Use {{name}} for interpolation and @@ for transforms.";
    ratio = "The ratio is 1::2.";
}
// → "Use {name} for interpolation and @ for transforms."
// → "The ratio is 1:2."
```

Multi-dimensional selection chains with multiple `:` operators:

```rust
rlf_lang!(Ru) {
    draw(n) = "Возьмите {n} {card:acc:n}.";
}
// n=1 → "Возьмите 1 карту."
// n=5 → "Возьмите 5 карт."
```

**Selection on phrase parameters:**

When a phrase takes another phrase as a parameter, you can select variants from
it:

```rust
rlf_source! {
    character = {
        one: "character",
        other: "characters",
    };

    with_cost_less_than_allies(base, counting) =
        "{base} with cost less than the number of allied {counting:other}";
}
// counting=character → "... allied characters"
```

Here `{counting:other}` means "use the 'other' (plural) variant of whatever
phrase `counting` refers to."

**Dynamic selection with numbers:**

Selection also works when both the phrase and selector are parameters:

```rust
rlf_source! {
    character = { one: "character", other: "characters" };
    card = { one: "card", other: "cards" };

    draw_things(n, thing) = "Draw {n} {thing:n}.";
}
// draw_things(1, character) → "Draw 1 character."
// draw_things(3, card) → "Draw 3 cards."
```

The phrase parameter `thing` carries its variants, and `:n` selects based on the
number at runtime.

---

## Metadata Tags

A phrase can declare metadata tags using `:` before its content. Tags serve
two purposes:

1. **Selection**: Other phrases can select variants based on the tag
2. **Transforms**: Transforms can read tags to determine behavior

```rust
rlf_lang!(Es) {
    card = :fem "carta";
    character = :masc "personaje";
}
```

**Multiple tags:**

Phrases can have multiple tags for different purposes:

```rust
rlf_source! {
    // English: article hint for @a transform
    card = :a "card";
    event = :an "event";
    ally = :an "ally";
    uniform = :a "uniform";   // phonetic exception (not "an uniform")
    hour = :an "hour";        // silent h exception
}

rlf_lang!(De) {
    // German: grammatical gender for article transforms
    karte = :fem "Karte";
    charakter = :masc "Charakter";
    ereignis = :neut "Ereignis";
}

rlf_lang!(ZhCn) {
    // Chinese: measure word category for @count transform
    pai = :zhang "牌";
    jue_se = :ge "角色";
}
```

**Selection based on tags:**

```rust
rlf_lang!(Es) {
    card = :fem "carta";
    character = :masc "personaje";

    destroyed = {
        masc: "destruido",
        fem: "destruida",
    };

    destroy(thing) = "{thing} fue {destroyed:thing}.";
}
// thing=card      → "carta fue destruida."
// thing=character → "personaje fue destruido."
```

---

## Transforms

The `@` operator applies a transform. Transforms are prefix operations that
modify text. When chaining multiple transforms, they apply right-to-left
(innermost first):

```rust
rlf_source! {
    card = "card";

    draw_one = "Draw {@a card}.";        // → "Draw a card."
    title = "{@cap card}";               // → "Card"
    heading = "{@cap @a card}";          // @a first → "a card", then @cap → "A card"
}
```

Transforms combine with selection:

```rust
rlf_source! {
    card = {
        one: "card",
        other: "cards",
    };

    draw(n) = "Draw {n} {@cap card:n}.";
}
// n=1 → "Draw 1 Card."
// n=3 → "Draw 3 Cards."
```

**Transform context:** Some transforms need additional information beyond the
phrase they operate on. Context is passed via `@transform:context`, where context
can be a literal value or a parameter reference:

```rust
destroy_card = "Zerstöre {@der:acc karte}.";      // :acc is literal context
return_all(t) = "devuelve {@el:other t} a mano";  // :other is literal context
draw(n) = "抽{@count:n card}";                    // :n is parameter context
```

Here `:acc` tells `@der` to produce the accusative article, `:other` tells `@el`
to produce the plural article, and `:n` tells `@count` how many items. The context
modifies the transform's behavior—it's not selecting from the phrase's variants.

**Disambiguation:** When both transform context and phrase selection appear, the
first `:` after the transform name is context, subsequent `:` operators apply to
the phrase:

```rust
draw(n) = "Draw {@cap card:n}.";           // No context; :n selects from card
get_card = "Nimm {@der:acc karte:one}.";   // :acc is context; :one selects variant
```

The general pattern is `{@transform:context phrase:selector}`. Transforms that
don't use context (like `@cap`) ignore any context provided.

### Universal Transforms

These transforms work on any text in any language:

| Transform | Effect                   |
| --------- | ------------------------ |
| `@cap`    | Capitalize first letter  |
| `@upper`  | All uppercase            |
| `@lower`  | All lowercase            |

### Metadata-Driven Transforms

Language-specific transforms read metadata tags to determine behavior:

```rust
// en.rlf.rs
rlf_source! {
    card = :a "card";
    event = :an "event";
    hour = :an "hour";      // silent h
    uniform = :a "uniform";  // phonetic exception

    draw_one = "Draw {@a card}.";   // → "Draw a card."
    play_one = "Play {@a event}.";  // → "Play an event."
}
```

The `@a` transform reads the `:a` or `:an` tag from its argument. If no tag is
present, it produces a **runtime error**. This ensures predictable results—
phonetic heuristics would silently produce wrong output for words like "uniform"
(uses "a" despite starting with a vowel) or "hour" (uses "an" despite starting
with a consonant).

This pattern applies to other language-specific transforms:

```rust
// de.rlf.rs - German definite articles
rlf_lang!(De) {
    karte = :fem "Karte";
    charakter = :masc "Charakter";
    ereignis = :neut "Ereignis";

    // @der reads :masc/:fem/:neut → der/die/das
    destroy_card = "Zerstöre {@der:acc karte}.";  // → "Zerstöre die Karte."
}

// zh_cn.rlf.rs - Chinese measure words
rlf_lang!(ZhCn) {
    pai = :zhang "牌";
    jue_se = :ge "角色";

    // @count uses context for the number, reads measure word tag from phrase
    draw(n) = "抽{@count:n pai}";  // → "抽3张牌"
}
```

### Standard Transform Library

RLF provides language-specific transforms for common patterns. Transforms are
scoped to the language file—`@un` in `es.rlf.rs` uses Spanish rules, while `@un`
in `fr.rlf.rs` uses French rules. The language is inferred from the filename.

| Transform   | Languages                      | Reads Tags                   | Effect                       |
| ----------- | ------------------------------ | ---------------------------- | ---------------------------- |
| `@a`        | English                        | `:a`, `:an`                  | Indefinite article (a/an)    |
| `@der`      | German                         | `:masc`, `:fem`, `:neut`     | Definite article + case      |
| `@el`       | Spanish                        | `:masc`, `:fem`              | Definite article             |
| `@le`       | French                         | `:masc`, `:fem`, `:vowel`    | Definite article (with elision) |
| `@un`       | Romance                        | `:masc`, `:fem`              | Indefinite article           |
| `@count`    | Chinese, Japanese, Korean      | `:zhang`, `:ge`, etc.        | Measure word insertion       |

### Transform Aliases

Transforms can have aliases for readability. Aliases are interchangeable—they
produce identical behavior:

```rust
// en.rlf.rs - @a and @an are aliases
rlf_source! {
    card = :a "card";
    event = :an "event";

    // Both forms work identically:
    draw_card = "Draw {@a card}.";    // → "Draw a card."
    play_event = "Play {@an event}."; // → "Play an event."

    // The transform reads the tag, not the alias used:
    also_works = "Draw {@an card}.";  // → "Draw a card." (reads :a tag)
}
```

Common aliases by language:

| Language | Primary | Aliases |
| -------- | ------- | ------- |
| English  | `@a`    | `@an`   |
| German   | `@der`  | `@die`, `@das` |
| Spanish  | `@el`   | `@la`   |
| French   | `@le`   | `@la`   |
| Italian  | `@il`   | `@lo`, `@la` |

Aliases let translators write what feels natural for the phrase being modified,
even though the transform behavior is determined by the phrase's metadata tags.

See **APPENDIX_STDLIB.md** for complete documentation of transforms per
language.

### Transforms on Phrase Parameters

Transforms read metadata from phrase parameters, enabling abstraction:

```rust
rlf_source! {
    card = :a "card";
    event = :an "event";
    ally = :an "ally";

    // thing is a phrase parameter - @a reads its :a/:an tag
    play(thing) = "Play {@a thing}.";
}
// play(card) → "Play a card."
// play(event) → "Play an event."
// play(ally) → "Play an ally."
```

All phrases passed to metadata-driven transforms must have the required tags.
Passing an untagged phrase or a raw string to `@a` produces a runtime error.

---

## Selection vs Transforms

**Selection** (`:`) chooses among predefined variants. Use for inherent word
forms: singular/plural, grammatical case, gender agreement.

```rust
card = { one: "card", other: "cards" };
draw(n) = "Draw {card:n}.";  // Picks "card" or "cards"
```

**Transforms** (`@`) compute new text. Use for external operations: adding
articles, capitalization, contractions, measure words.

```rust
card = :a "card";
draw_one = "Draw {@a card}.";  // Produces "a card"
```

The distinction reflects linguistics: selection handles **inflection** (word-
internal changes), transforms handle **syntax** (word-external additions).
Transforms can do things selection cannot—inspect phonetics, combine elements,
apply dynamic rules.

---

## Composition Examples

### Pluralization (English)

```rust
// en.rlf.rs
rlf_source! {
    card = {
        one: "card",
        other: "cards",
    };

    draw(n) = "Draw {n} {card:n}.";
}
```

### Pluralization (Russian)

Russian has three plural categories:

```rust
// ru.rlf.rs
rlf_lang!(Ru) {
    card = {
        one: "карту",
        few: "карты",
        many: "карт",
    };

    draw(n) = "Возьмите {n} {card:n}.";
}
// n=1  → "Возьмите 1 карту."
// n=3  → "Возьмите 3 карты."
// n=5  → "Возьмите 5 карт."
// n=21 → "Возьмите 21 карту."
```

### Case + Number (Russian)

```rust
// ru.rlf.rs
rlf_lang!(Ru) {
    card = {
        nom: "карта",
        nom.many: "карт",
        acc: "карту",
        acc.many: "карт",
        gen: "карты",
        gen.many: "карт",
    };

    draw(n) = "Возьмите {n} {card:acc:n}.";
    no_cards = "Нет {card:gen.other}.";
}
```

### Gender Agreement (Spanish)

```rust
// es.rlf.rs
rlf_lang!(Es) {
    card = :fem "carta";
    enemy = :masc "enemigo";

    destroyed = {
        masc: "destruido",
        fem: "destruida",
    };

    destroy(target) = "{@cap target} fue {destroyed:target}.";
}
// target=card  → "Carta fue destruida."
// target=enemy → "Enemigo fue destruido."
```

### Verb Agreement (Russian)

Verbs that agree with their subject use the same tag-based selection pattern:

```rust
// ru.rlf.rs
rlf_lang!(Ru) {
    card = :fem "карта";
    character = :masc "персонаж";

    // Past tense agrees in gender
    was_destroyed = {
        masc: "был уничтожен",
        fem: "была уничтожена",
        neut: "было уничтожено",
    };

    thing_destroyed(thing) = "{thing:nom.one} {was_destroyed:thing}.";
}
// thing=card      → "карта была уничтожена."
// thing=character → "персонаж был уничтожен."
```

### Measure Words (Chinese)

Chinese requires measure words (classifiers) between numbers and nouns. The
`@count` transform uses context for the number and reads the measure word tag
from the phrase:

```rust
// zh_cn.rlf.rs
rlf_lang!(ZhCn) {
    card = :zhang "牌";       // :zhang = 张 (flat objects)
    character = :ge "角色";   // :ge = 个 (general classifier)

    draw(n) = "抽{@count:n card}。";      // → "抽3张牌。"
    summon(n) = "召唤{@count:n character}。"; // → "召唤2个角色。"
}
```

The `@count:n phrase` pattern uses `n` as context (the count) and reads the
measure word tag from the phrase. This applies to Japanese, Korean, Vietnamese,
Thai, and Bengali, each with their own classifier tags.

---

## File Structure

Each language has its own `.rlf.rs` file:

```
src/
  localization/
    mod.rs
    en.rlf.rs      # English (source) - uses rlf_source!
    zh_cn.rlf.rs   # Simplified Chinese - uses rlf_lang!
    ru.rlf.rs      # Russian - uses rlf_lang!
    es.rlf.rs      # Spanish - uses rlf_lang!
    pt_br.rlf.rs   # Portuguese (Brazil) - uses rlf_lang!
```

The source language defines the API contract via the generated trait. Translation
languages implement the trait, with missing phrases falling back to the source.

---

## Runtime Values

All parameters accept a `Value` type that can represent numbers, strings, or
phrase references. Type checking happens at runtime.

```rust
// All of these work:
En.draw(3);                    // number
En.draw("3");                  // string (converted to display)
En.greet("World");             // string
Es.destroy(Es.card());         // phrase reference
```

**Runtime behavior:**

| Operation              | Value Type      | Behavior                                    |
| ---------------------- | --------------- | ------------------------------------------- |
| `{x}`                  | Any             | Display the value                           |
| `{card:x}` (selection) | Number          | Select plural category (one/few/many/other) |
| `{card:x}` (selection) | String          | Parse as number, or error if invalid        |
| `{card:x}` (selection) | Phrase          | Look up matching tag from phrase            |
| `{card:x}` (selection) | (no match)      | **Runtime error** (missing variant)         |
| `{@a x}`               | Phrase with tag | Use the tag                                 |
| `{@a x}`               | Other           | **Runtime error** (missing required tag)    |

**Design rationale:** All selection operations produce runtime errors when they
cannot find a matching variant. This catches bugs early: a missing `few` variant
in Russian, a missing gender tag for Spanish agreement, or passing a string where
a phrase was expected. Silent fallbacks would produce subtly incorrect output that
might not be noticed until a native speaker reviews the text.

**Error handling:** All phrase functions return `Result<String, RlfError>`.
Callers should handle errors appropriately for their context—log and use a
fallback string, propagate the error, or panic during development. The error
message includes the operation, expected variants/tags, and actual value to
aid debugging.

---

## Generated API

Given:

```rust
// en.rlf.rs
rlf_source! {
    card = {
        one: "card",
        other: "cards",
    };

    draw(n) = "Draw {n} {card:n}.";
}
```

RLF generates:

```rust
// A unit struct for trait dispatch
pub struct En;

// The trait with default implementations (fallback to English)
pub trait RlfLang {
    fn card(&self) -> Phrase { ... }
    fn draw(&self, n: impl Into<Value>) -> Result<String, RlfError> { ... }
}

impl RlfLang for En { ... }
```

For a translation:

```rust
// ru.rlf.rs
rlf_lang!(Ru) {
    card = { ... };
    draw(n) = "...";
}
```

RLF generates:

```rust
pub struct Ru;

impl RlfLang for Ru { ... }
```

**Usage:**

```rust
use localization::{En, Ru, RlfLang};

fn localized_draw(lang: &impl RlfLang, n: i32) -> Result<String, RlfError> {
    lang.draw(n)
}

localized_draw(&En, 3)?;  // "Draw 3 cards."
localized_draw(&Ru, 3)?;  // "Возьмите 3 карты."
```

**Phrase type:**

Phrases without parameters return a `Phrase` that carries text, variants, and
tags. This type is used when passing phrases as parameters to other phrases.

```rust
pub struct Phrase {
    /// Default text (used when displaying without variant selection).
    text: Cow<'static, str>,
    /// Variant key → variant text. Keys use dot notation: "nom.one".
    variants: HashMap<&'static str, Cow<'static, str>>,
    /// Metadata tags attached to this phrase.
    tags: &'static [&'static str],
}
```

The `Cow<'static, str>` type allows phrases to use static strings when no
interpolation occurs, while supporting owned strings for dynamic content.

From Rust code, use the `variant()` method to access specific variants:

```rust
let singular = En.card();                    // displays as "card"
let plural = En.card().variant("other")?;    // "cards"
```

---

## Incomplete Translations

During development, you may want to add phrases to the source language without
immediately translating them. RLF supports this via **fallback behavior**.

### Default: Fallback to Source

The trait's default implementations use the source language text. If Russian
doesn't define a phrase, it inherits the English version:

```rust
// ru.rlf.rs
rlf_lang!(Ru) {
    card = { ... };
    // 'draw' not defined - falls back to English
}

// Usage:
Ru.draw(3)  // Returns English: "Draw 3 cards."
```

### Strict Mode for CI

For release builds, enable the `strict-i18n` feature to require complete
translations:

```toml
# Cargo.toml
[features]
default = []
strict-i18n = []
```

With `strict-i18n` enabled, the trait has no default implementations:

```rust
// When strict-i18n is enabled
pub trait RlfLang {
    fn card(&self) -> Phrase;  // no default - must implement
    fn draw(&self, n: impl Into<Value>) -> String;  // no default
}
```

Missing translations become compile errors:

```
error[E0046]: not all trait items implemented, missing: `draw`
  --> ru.rlf.rs
   |
   = note: `draw` from trait `RlfLang` is not implemented
```

**Workflow:**

- **Development:** `cargo build` — missing translations fall back to English
- **CI/Release:** `cargo build --features strict-i18n` — missing translations fail

### Runtime Detection

For debugging, each language struct provides a `is_fallback(phrase_name: &str)`
method that returns `true` if calling that phrase would use the source language
fallback. This enables debug UIs to highlight untranslated content:

```rust
if Ru.is_fallback("draw") {
    log::warn!("'draw' is not yet translated to Russian");
}
```

---

## Compile-Time Errors

RLF validates phrase and parameter *names* at compile time within each file.
Cross-file validation (missing translations) is enforced via the trait system.

**Unknown phrase:**

```rust
rlf_source! {
    draw(n) = "Draw {n} {cards:n}.";  // typo
}
```

```
error: unknown phrase 'cards'
  --> en.rlf.rs:2:28
   |
2  |     draw(n) = "Draw {n} {cards:n}.";
   |                          ^^^^^ not defined
   |
   = help: did you mean 'card'?
```

**Unknown parameter:**

```rust
rlf_source! {
    draw(n) = "Draw {count} {card:n}.";  // 'count' not declared
}
```

```
error: unknown parameter 'count'
  --> en.rlf.rs:2:18
   |
2  |     draw(n) = "Draw {count} {card:n}.";
   |                      ^^^^^ not in parameter list
   |
   = help: declared parameters: n
```

**Missing variant:**

```rust
// ru.rlf.rs
rlf_lang!(Ru) {
    card = {
        one: "карта",
        other: "карт",  // missing 'few'
    };
}
```

```
error: missing variant 'few' for phrase 'card'
  --> ru.rlf.rs:2:5
   |
   = note: Russian requires: one, few, many
```

**Invalid selector:**

Using a selector that doesn't match any variant produces an error listing
available variants.

**Missing phrase in translation (strict mode):**

With `--features strict-i18n`, missing translations produce standard Rust trait
implementation errors.

---

## Design Philosophy

**Logic in Rust, text in RLF.** Complex branching stays in Rust; RLF provides
atomic text pieces. This keeps RLF files simple and translator-friendly.

**Keywords and formatting are just phrases.** No special syntax—define phrases
with markup (`dissolve = "<k>dissolve</k>";`) and interpolate normally.

**Dynamic typing for simplicity.** Parameters accept any `Value` type. Runtime
errors catch type mismatches. Translators don't need to understand Rust types.

**Immediate IDE support.** Proc-macros enable rust-analyzer autocomplete without
external build tools.

**Edge cases require more variants.** Some language features require extensive
variant tables. This is by design—explicit variants are predictable. When
unwieldy, use language-specific transforms to encapsulate complexity.

---

## Summary

| Primitive    | Syntax                         | Purpose                                 |
| ------------ | ------------------------------ | --------------------------------------- |
| Phrase       | `name = "text";`               | Define text                             |
| Parameter    | `name(p) = "{p}";`             | Accept values                           |
| Variant      | `name = { a: "x", b: "y" };`   | Multiple forms                          |
| Selection    | `{phrase:selector}`            | Choose a variant                        |
| Metadata tag | `name = :tag "text";`          | Attach metadata for selection/transforms|
| Transform    | `{@transform:ctx phrase}`      | Modify text                             |

| Macro          | Purpose                                              |
| -------------- | ---------------------------------------------------- |
| `rlf_source!`  | Define source language; generates trait              |
| `rlf_lang!(X)` | Define translation; generates trait impl             |

Four primitives, two macros, Rust-compatible syntax, compile-time name checking,
runtime error handling via `Result<String, RlfError>`, automatic fallback for
incomplete translations during development.
