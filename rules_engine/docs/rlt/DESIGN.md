# RLT

The Rust Language Toolkit: a localization DSL embedded in Rust via macros.

## Overview

RLT files are valid Rust source files with a `.rlt.rs` extension. They
contain an `rlt!` macro invocation that defines phrases for a single
language.

```rust
// en.rlt.rs
rlt! {
    hello = "Hello, world!";
}
```

The macro generates Rust functions with compile-time validation of phrase and
parameter names. Parameter values are dynamically typed at runtime.

---

## Primitives

RLT has four primitives: **phrase**, **parameter**, **variant**, and
**selection**.

### Phrase

A phrase has a name and produces text.

```rust
rlt! {
    hello = "Hello, world!";
    goodbye = "Goodbye!";
}
```

### Parameter

Phrases can accept values. Parameters are declared in parentheses and
interpolated with `{}`.

```rust
rlt! {
    greet(name) = "Hello, {name}!";
    damage(amount, target) = "Deal {amount} damage to {target}.";
}
```

### Variant

A phrase can have multiple forms. Variants are declared in braces after `=`.

```rust
rlt! {
    card = {
        one: "card",
        other: "cards",
    };
}
```

Variants can be multi-dimensional using dot notation:

```rust
rlt! {
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

### Selection

The `:` operator selects a variant.

Literal selection uses a variant name directly:

```rust
rlt! {
    all_cards = "All {card:other}.";
    take_one = "Возьмите {card:acc.one}.";
}
```

Derived selection uses a parameter. For numbers, RLT maps to CLDR plural
categories (`one`, `two`, `few`, `many`, `other`):

```rust
rlt! {
    draw(n) = "Draw {n} {card:n}.";
}
// n=1 → "Draw 1 card."
// n=5 → "Draw 5 cards."
```

Multi-dimensional selection chains with multiple `:` operators:

```rust
rlt! {
    draw(n) = "Возьмите {n} {card:acc:n}.";
}
// n=1 → "Возьмите 1 карту."
// n=5 → "Возьмите 5 карт."
```

**Selection on phrase parameters:**

When a phrase takes another phrase as a parameter, you can select variants from
it:

```rust
rlt! {
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
rlt! {
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

A phrase can declare metadata tags using `:` after its definition. Tags serve
two purposes:

1. **Selection**: Other phrases can select variants based on the tag
2. **Transforms**: Transforms can read tags to determine behavior

```rust
rlt! {
    card = "carta" :fem;
    character = "personaje" :masc;
}
```

**Multiple tags:**

Phrases can have multiple tags for different purposes:

```rust
rlt! {
    // English: article hint for @a transform
    card = "card" :a;
    event = "event" :an;
    ally = "ally" :an;
    uniform = "uniform" :a;   // phonetic exception (not "an uniform")
    hour = "hour" :an;        // silent h exception

    // German: grammatical gender for article transforms
    karte = "Karte" :fem;
    charakter = "Charakter" :masc;
    ereignis = "Ereignis" :neut;

    // Chinese: measure word category for @count transform
    pai = "牌" :zhang;
    jue_se = "角色" :ge;
}
```

**Selection based on tags:**

```rust
rlt! {
    card = "carta" :fem;
    character = "personaje" :masc;

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
modify text.

```rust
rlt! {
    card = "card";

    draw_one = "Draw {@a card}.";        // → "Draw a card."
    title = "{@cap card}";               // → "Card"
    heading = "{@cap @a card}";          // → "A card"
}
```

Transforms combine with selection:

```rust
rlt! {
    card = {
        one: "card",
        other: "cards",
    };

    draw(n) = "Draw {n} {@cap card:n}.";
}
// n=1 → "Draw 1 Card."
// n=3 → "Draw 3 Cards."
```

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
// en.rlt.rs
rlt! {
    card = "card" :a;
    event = "event" :an;
    hour = "hour" :an;      // silent h
    uniform = "uniform" :a;  // phonetic exception

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
// de.rlt.rs - German definite articles
rlt! {
    karte = "Karte" :fem;
    charakter = "Charakter" :masc;
    ereignis = "Ereignis" :neut;

    // @the reads :masc/:fem/:neut → der/die/das
    destroy_card = "Zerstöre {@the karte}.";  // → "Zerstöre die Karte."
}

// zh_cn.rlt.rs - Chinese measure words
rlt! {
    pai = "牌" :zhang;
    jue_se = "角色" :ge;

    // @count reads measure word tags
    draw(n) = "抽{@count n pai}";  // → "抽3张牌"
}
```

### Standard Transform Library

RLT provides language-specific transforms for common patterns. Transforms are
scoped to the language file—`@un` in `es.rlt.rs` uses Spanish rules, while `@un`
in `fr.rlt.rs` uses French rules. The language is inferred from the filename.

| Transform   | Languages                      | Reads Tags                   | Effect                       |
| ----------- | ------------------------------ | ---------------------------- | ---------------------------- |
| `@a`        | English                        | `:a`, `:an`                  | Indefinite article (a/an)    |
| `@the`      | Germanic, Romance              | `:masc`, `:fem`, `:neut`     | Definite article             |
| `@un`       | Romance                        | `:masc`, `:fem`              | Indefinite article           |
| `@contract` | French, Italian, Portuguese    | article + preposition        | Contraction (de+le→du)       |
| `@elide`    | French, Italian                | `:vowel`                     | Vowel elision (le→l')        |
| `@count`    | Chinese, Japanese, Korean      | `:zhang`, `:ge`, etc.        | Measure word insertion       |

See **APPENDIX_STDLIB.md** for complete documentation of transforms per
language.

### Transforms on Phrase Parameters

Transforms read metadata from phrase parameters, enabling abstraction:

```rust
rlt! {
    card = "card" :a;
    event = "event" :an;
    ally = "ally" :an;

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
card = "card" :a;
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
// en.rlt.rs
rlt! {
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
// ru.rlt.rs
rlt! {
    card = {
        one: "карта",
        few: "карты",
        many: "карт",
    };

    draw(n) = "Возьмите {n} {card:n}.";
}
// n=1  → "Возьмите 1 карта."
// n=3  → "Возьмите 3 карты."
// n=5  → "Возьмите 5 карт."
// n=21 → "Возьмите 21 карта."
```

### Case + Number (Russian)

```rust
// ru.rlt.rs
rlt! {
    card = {
        nom.one: "карта",
        nom.few: "карты",
        nom.many: "карт",
        acc.one: "карту",
        acc.few: "карты",
        acc.many: "карт",
        gen.one: "карты",
        gen.few: "карт",
        gen.many: "карт",
    };

    draw(n) = "Возьмите {n} {card:acc:n}.";
    no_cards = "Нет {card:gen.other}.";
}
```

### Gender Agreement (Spanish)

```rust
// es.rlt.rs
rlt! {
    card = "carta" :fem;
    enemy = "enemigo" :masc;

    destroyed = {
        masc: "destruido",
        fem: "destruida",
    };

    destroy(target) = "{@cap target} fue {destroyed:target}.";
}
// target=card  → "Carta fue destruida."
// target=enemy → "Enemigo fue destruido."
```

### Measure Words (Chinese)

```rust
// zh-CN.rlt.rs
rlt! {
    card = "牌";
    character = "角色";

    zhang(n, thing) = "{n}张{thing}";
    ge(n, thing) = "{n}个{thing}";

    draw(n) = "抽{zhang(n, card)}。";
    summon(n) = "召唤{ge(n, character)}。";
}
// draw(3)   → "抽3张牌。"
// summon(2) → "召唤2个角色。"
```

### Articles (English)

```rust
// en.rlt.rs
rlt! {
    card = "card";
    event = "event";
    ally = "ally";

    draw_one = "Draw {@a card}.";      // → "Draw a card."
    play_one = "Play {@a event}.";     // → "Play an event."
    target_one = "Target {@a ally}.";  // → "Target an ally."
}
```

---

## File Structure

Each language has its own `.rlt.rs` file:

```
src/
  localization/
    mod.rs
    en.rlt.rs      # English (source)
    zh_cn.rlt.rs   # Simplified Chinese
    ru.rlt.rs      # Russian
    es.rlt.rs      # Spanish
    pt_br.rlt.rs   # Portuguese (Brazil)
```

The source language defines the contract. Other languages must define the same
phrase names with the same parameters.

---

## Runtime Values

All parameters accept a `Value` type that can represent numbers, strings, or
phrase references. Type checking happens at runtime.

```rust
// All of these work:
en::draw(3);                    // number
en::draw("3");                  // string (converted to display)
en::greet("World");             // string
en::destroy(es::card());        // phrase reference
```

**Runtime behavior:**

| Operation              | Value Type      | Behavior                                    |
| ---------------------- | --------------- | ------------------------------------------- |
| `{x}`                  | Any             | Display the value                           |
| `{card:x}` (plural)    | Number          | Select plural category (one/few/many/other) |
| `{card:x}` (plural)    | String          | Parse as number, or use "other" as fallback |
| `{card:x}` (plural)    | Phrase          | Use "other" as fallback                     |
| `{destroyed:x}` (tags) | Phrase          | Look up tag, select matching variant        |
| `{destroyed:x}` (tags) | String/Number   | Use first variant as fallback               |
| `{@a x}`               | Phrase with tag | Use the tag                                 |
| `{@a x}`               | Other           | **Runtime error** (missing required tag)    |

This design prioritizes simplicity and flexibility over strict type safety.
Selection operations use fallback behavior—the system always produces *some*
output, even if the value type doesn't match the expected usage. However,
metadata-driven transforms (like `@a`) produce runtime errors when required
tags are missing, ensuring predictable results.

---

## Generated API

Given:

```rust
// en.rlt.rs
rlt! {
    card = {
        one: "card",
        other: "cards",
    };

    draw(n) = "Draw {n} {card:n}.";
}
```

RLT generates:

```rust
pub mod en {
    pub fn draw(n: impl Into<Value>) -> String {
        // Runtime plural selection based on n's value
    }
}
```

Usage:

```rust
use localization::en;

let msg = en::draw(3);  // "Draw 3 cards."
```

**Variant accessors:**

For phrases with variants, RLT generates accessor functions for each variant:

```rust
// RLT definition:
card = {
    one: "card",
    other: "cards",
};

// Generated Rust:
pub fn card() -> PhraseRef { ... }          // Default (first variant)
pub fn card_one() -> PhraseRef { ... }
pub fn card_other() -> PhraseRef { ... }
```

The `PhraseRef` type carries the phrase's text, variants, and tags:

```rust
pub struct PhraseRef {
    text: &'static str,
    variants: &'static [(&'static str, &'static str)],
    tags: &'static [&'static str],
}

impl std::fmt::Display for PhraseRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}
```

This allows Rust code to access specific variants when needed:

```rust
// Use default (singular)
let singular = en::card();  // displays as "card"

// Use explicit plural
let plural = en::card_other();  // displays as "cards"
```

**Passing phrase references:**

```rust
// es.rlt.rs
rlt! {
    carta = "carta" :fem;
    enemigo = "enemigo" :masc;
    destruido = { masc: "destruido", fem: "destruida" };

    destroy(target) = "{target} fue {destruido:target}.";
}
```

```rust
use localization::es;

let msg = es::destroy(es::carta());    // "carta fue destruida."
let msg = es::destroy(es::enemigo());  // "enemigo fue destruido."
```

**Multi-language support:**

```rust
use localization::messages;
use localization::Language;

let msg = messages::draw(Language::Ru, 3);  // "Возьмите 3 карты."
```

---

## Compile-Time Errors

RLT validates phrase and parameter *names* at compile time. Type mismatches are
handled at runtime with fallback behavior.

**Unknown phrase:**

```rust
rlt! {
    draw(n) = "Draw {n} {cards:n}.";  // typo
}
```

```
error: unknown phrase 'cards'
  --> en.rlt.rs:2:28
   |
2  |     draw(n) = "Draw {n} {cards:n}.";
   |                          ^^^^^ not defined
   |
   = help: did you mean 'card'?
```

**Unknown parameter:**

```rust
rlt! {
    draw(n) = "Draw {count} {card:n}.";  // 'count' not declared
}
```

```
error: unknown parameter 'count'
  --> en.rlt.rs:2:18
   |
2  |     draw(n) = "Draw {count} {card:n}.";
   |                      ^^^^^ not in parameter list
   |
   = help: declared parameters: n
```

**Missing variant:**

```rust
// ru.rlt.rs
rlt! {
    card = {
        one: "карта",
        other: "карт",  // missing 'few'
    };
}
```

```
error: missing variant 'few' for phrase 'card'
  --> ru.rlt.rs:2:5
   |
   = note: Russian requires: one, few, many
```

**Invalid selector:**

```rust
rlt! {
    take = "{card:accusative}";  // 'accusative' not a variant of card
}
```

```
error: phrase 'card' has no variant 'accusative'
  --> en.rlt.rs:2:17
   |
2  |     take = "{card:accusative}";
   |                   ^^^^^^^^^^ variant not defined
   |
   = note: available variants: one, other
```

**Missing phrase in translation:**

A build script validates all `.rlt.rs` files together:

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

---

## Design Philosophy

**Logic in Rust, text in RLT.**

RLT provides atomic text pieces. Complex branching logic stays in Rust code:

```rust
// Rust code handles the logic
match predicate {
    Predicate::Your(card_predicate) => {
        if is_character(card_predicate) {
            en::ally()  // RLT provides the text
        } else {
            en::your_card()
        }
    }
    Predicate::Enemy(card_predicate) => {
        en::enemy()
    }
}
```

This separation keeps RLT files simple and translator-friendly, while allowing
arbitrarily complex composition logic in Rust.

**Keywords and formatting are just phrases.**

There's no special keyword syntax. Define phrases that return formatted text:

```rust
rlt! {
    dissolve = "<k>dissolve</k>";
    materialized = "<k>materialized</k>";
    energy_symbol = "<e>●</e>";

    could_dissolve(target) = "which could {dissolve} {target}";
}
// → "which could <k>dissolve</k> that character"
```

The formatting markup is part of the phrase text and gets interpolated normally.

**Dynamic typing for simplicity.**

Parameters are dynamically typed to keep the RLT syntax simple and
translator-friendly. The runtime handles type mismatches gracefully with
fallback behavior rather than errors. This trades strict type safety for ease
of use—translators don't need to understand Rust's type system.

---

## Summary

| Primitive    | Syntax                         | Purpose                                 |
| ------------ | ------------------------------ | --------------------------------------- |
| Phrase       | `name = "text";`               | Define text                             |
| Parameter    | `name(p) = "{p}";`             | Accept values                           |
| Variant      | `name = { a: "x", b: "y" };`   | Multiple forms                          |
| Selection    | `{phrase:selector}`            | Choose a variant                        |
| Metadata tag | `name = "text" :tag;`          | Attach metadata for selection/transforms|
| Transform    | `{@transform phrase}`          | Modify text                             |

Four primitives, Rust-compatible syntax, compile-time name checking, runtime
value handling.
