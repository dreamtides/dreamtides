# Phraselet

A localization DSL embedded in Rust via macros.

## Overview

Phraselet files are valid Rust source files with a `.phr.rs` extension. They
contain a `phraselet!` macro invocation that defines phrases for a single
language.

```rust
// en.phr.rs
phraselet! {
    hello = "Hello, world!";
}
```

The macro generates strongly-typed Rust functions, with errors caught at compile
time.

---

## Primitives

Phraselet has four primitives: **phrase**, **parameter**, **variant**, and
**selection**.

### Phrase

A phrase has a name and produces text.

```rust
phraselet! {
    hello = "Hello, world!";
    goodbye = "Goodbye!";
}
```

### Parameter

Phrases can accept values. Parameters are declared in parentheses and
interpolated with `{}`.

```rust
phraselet! {
    greet(name) = "Hello, {name}!";
    damage(amount, target) = "Deal {amount} damage to {target}.";
}
```

### Variant

A phrase can have multiple forms. Variants are declared in braces after `=`.

```rust
phraselet! {
    card = {
        one: "card",
        other: "cards",
    };
}
```

Variants can be multi-dimensional using dot notation:

```rust
phraselet! {
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
phraselet! {
    all_cards = "All {card:other}.";
    take_one = "Возьмите {card:acc.one}.";
}
```

Derived selection uses a parameter. For numbers, Phraselet maps to CLDR plural
categories (`one`, `two`, `few`, `many`, `other`):

```rust
phraselet! {
    draw(n) = "Draw {n} {card:n}.";
}
// n=1 → "Draw 1 card."
// n=5 → "Draw 5 cards."
```

Multi-dimensional selection chains with multiple `:` operators:

```rust
phraselet! {
    draw(n) = "Возьмите {n} {card:acc:n}.";
}
// n=1 → "Возьмите 1 карту."
// n=5 → "Возьмите 5 карт."
```

**Selection on phrase parameters:**

When a phrase takes another phrase as a parameter, you can select variants from
it:

```rust
phraselet! {
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

---

## Metadata Tags

A phrase can declare metadata tags using `:` after its definition. Tags serve two purposes:
1. **Selection**: Other phrases can select variants based on the tag
2. **Transforms**: Transforms can read tags to determine behavior

```rust
phraselet! {
    carta = "carta" :fem;
    personaje = "personaje" :masc;
}
```

**Multiple tags:**

Phrases can have multiple tags for different purposes:

```rust
phraselet! {
    // English: article hint for @a transform
    card = "card" :a;
    event = "event" :an;
    ally = "ally" :an;
    uniform = "uniform" :a;   // phonetic exception (not "an uniform")
    hour = "hour" :an;        // silent h exception

    // German: grammatical gender for article transforms
    Karte = "Karte" :fem;
    Charakter = "Charakter" :masc;
    Ereignis = "Ereignis" :neut;

    // Chinese: measure word category for @count transform
    牌 = "牌" :zhang;
    角色 = "角色" :ge;
}
```

**Selection based on tags:**

```rust
phraselet! {
    carta = "carta" :fem;
    personaje = "personaje" :masc;

    destruido = {
        masc: "destruido",
        fem: "destruida",
    };

    destroyed(thing) = "{thing} fue {destruido:thing}.";
}
// thing=carta     → "carta fue destruida."
// thing=personaje → "personaje fue destruido."
```

---

## Transforms

The `@` operator applies a transform. Transforms are prefix operations that
modify text.

```rust
phraselet! {
    card = "card";

    draw_one = "Draw {@a card}.";        // → "Draw a card."
    title = "{@cap card}";               // → "Card"
    heading = "{@cap @a card}";          // → "A card"
}
```

Transforms combine with selection:

```rust
phraselet! {
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

| Transform | Effect |
|-----------|--------|
| `@cap` | Capitalize first letter |
| `@upper` | All uppercase |
| `@lower` | All lowercase |

### Metadata-Driven Transforms

Language-specific transforms read metadata tags to determine behavior:

```rust
// en.phr.rs
phraselet! {
    card = "card" :a;
    event = "event" :an;
    hour = "hour" :an;      // silent h
    uniform = "uniform" :a;  // phonetic exception

    draw_one = "Draw {@a card}.";   // → "Draw a card."
    play_one = "Play {@a event}.";  // → "Play an event."
}
```

The `@a` transform:
1. Checks if the argument has `:a` or `:an` tag → uses that
2. Falls back to phonetic heuristics for untagged text

This pattern applies to other language-specific transforms:

```rust
// de.phr.rs - German definite articles
phraselet! {
    Karte = "Karte" :fem;
    Charakter = "Charakter" :masc;
    Ereignis = "Ereignis" :neut;

    // @the reads :masc/:fem/:neut → der/die/das
    destroy_card = "Zerstöre {@the Karte}.";  // → "Zerstöre die Karte."
}

// zh_cn.phr.rs - Chinese measure words
phraselet! {
    牌 = "牌" :zhang;
    角色 = "角色" :ge;

    // @count reads measure word tags
    draw(n) = "抽{@count n 牌}";  // → "抽3张牌"
}
```

### Standard Transform Library

Phraselet provides language-specific transforms for common patterns:

| Transform | Languages | Reads Tags | Effect |
|-----------|-----------|------------|--------|
| `@a` | English | `:a`, `:an` | Indefinite article (a/an) |
| `@the` | Germanic, Romance | `:masc`, `:fem`, `:neut` | Definite article |
| `@un` | Romance | `:masc`, `:fem` | Indefinite article |
| `@contract` | French, Italian, Portuguese | article + preposition | Contraction (de+le→du) |
| `@elide` | French, Italian | `:vowel` | Vowel elision (le→l') |
| `@count` | Chinese, Japanese, Korean | `:zhang`, `:ge`, etc. | Measure word insertion |

See **APPENDIX_STDLIB.md** for complete documentation of transforms per language.

### Transforms on Dynamic Values

Transforms work on any displayable value, not just phrase references. For
untagged runtime strings, transforms use heuristics:

```rust
phraselet! {
    // subtype is a runtime string like "Warrior" or "Ancient"
    not_subtype(subtype) = "that is not {@a subtype}";
}
// subtype="Warrior" → "that is not a Warrior"
// subtype="Ancient" → "that is not an Ancient" (heuristic: starts with vowel)
```

For predictable behavior with edge cases, define phrases with explicit tags.

---

## Composition Examples

### Pluralization (English)

```rust
// en.phr.rs
phraselet! {
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
// ru.phr.rs
phraselet! {
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
// ru.phr.rs
phraselet! {
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
// es.phr.rs
phraselet! {
    carta = "carta" :fem;
    enemigo = "enemigo" :masc;

    destruido = {
        masc: "destruido",
        fem: "destruida",
    };

    destroy(target) = "{@cap target} fue {destruido:target}.";
}
// target=carta   → "Carta fue destruida."
// target=enemigo → "Enemigo fue destruido."
```

### Measure Words (Chinese)

```rust
// zh-CN.phr.rs
phraselet! {
    card = "牌";
    character = "角色";

    张(n, thing) = "{n}张{thing}";
    个(n, thing) = "{n}个{thing}";

    draw(n) = "抽{张(n, card)}。";
    summon(n) = "召唤{个(n, character)}。";
}
// draw(3)   → "抽3张牌。"
// summon(2) → "召唤2个角色。"
```

### Articles (English)

```rust
// en.phr.rs
phraselet! {
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

Each language has its own `.phr.rs` file:

```
src/
  localization/
    mod.rs
    en.phr.rs      # English (source)
    zh_cn.phr.rs   # Simplified Chinese
    ru.phr.rs      # Russian
    es.phr.rs      # Spanish
    pt_br.phr.rs   # Portuguese (Brazil)
```

The source language defines the contract. Other languages must define the same
phrase names with the same parameters.

---

## Type Inference

Parameters don't need type annotations. Phraselet infers types from how
parameters are used.

**Inference rules:**

| Usage | Inferred type | Rust type |
|-------|---------------|-----------|
| `{card:n}` with plural variants | Numeric | `i64` |
| `{destruido:x}` with inherent selector | Phrase reference | Generated enum |
| `{name}` interpolation only | Displayable | `impl Display` |

**Example:**

```rust
phraselet! {
    card = { one: "card", other: "cards" };
    carta = "carta" :fem;
    destruido = { masc: "destruido", fem: "destruida" };

    draw(n) = "Draw {n} {card:n}.";              // n: i64
    greet(name) = "Hello, {name}!";              // name: impl Display
    destroy(target) = "{destruido:target}";      // target: phrase with :fem/:masc
}
```

**Generated Rust:**

```rust
pub fn draw(n: i64) -> String { ... }
pub fn greet(name: impl std::fmt::Display) -> String { ... }
pub fn destroy(target: Gendered) -> String { ... }

// Enum for phrases with :fem/:masc selectors
pub enum Gendered {
    Carta,
}
```

**Conflicting usage is an error:**

```rust
phraselet! {
    foo(x) = "{card:x} {destruido:x}";  // x used for both plural and gender?
}
```

```
error: parameter 'x' has conflicting usage
  --> en.phr.rs:2:15
   |
2  |     foo(x) = "{card:x} {destruido:x}";
   |               ^^^^^^^^ plural selection (requires numeric)
   |                        ^^^^^^^^^^^^^ gender selection (requires phrase)
```

---

## Generated API

Given:

```rust
// en.phr.rs
phraselet! {
    card = {
        one: "card",
        other: "cards",
    };

    draw(n) = "Draw {n} {card:n}.";
}
```

Phraselet generates:

```rust
pub mod en {
    pub fn draw(n: i64) -> String {
        // ...
    }
}
```

Usage:

```rust
use localization::en;

let msg = en::draw(3);  // "Draw 3 cards."
```

**Variant accessors:**

For phrases with variants, Phraselet generates accessor functions for each
variant:

```rust
// Phraselet definition:
card = {
    one: "card",
    other: "cards",
};

// Generated Rust:
pub fn card() -> &'static str { "card" }        // Default (first variant)
pub fn card_one() -> &'static str { "card" }
pub fn card_other() -> &'static str { "cards" }
```

This allows Rust code to access specific variants when needed:

```rust
// Use default (singular)
let singular = en::card();  // "card"

// Use explicit plural
let plural = en::card_other();  // "cards"
```

**Passing phrase references:**

For parameters that require a phrase (inferred from inherent selector usage):

```rust
// es.phr.rs
phraselet! {
    carta = "carta" :fem;
    enemigo = "enemigo" :masc;
    destruido = { masc: "destruido", fem: "destruida" };

    destroy(target) = "{target} fue {destruido:target}.";
}
```

```rust
use localization::es;

let msg = es::destroy(es::Gendered::Carta);    // "carta fue destruida."
let msg = es::destroy(es::Gendered::Enemigo);  // "enemigo fue destruido."
```

**Multi-language support:**

```rust
use localization::messages;
use localization::Language;

let msg = messages::draw(Language::Ru, 3);  // "Возьмите 3 карты."
```

---

## Compile-Time Errors

**Unknown phrase:**

```rust
phraselet! {
    draw(n) = "Draw {n} {cards:n}.";  // typo
}
```

```
error: unknown phrase 'cards'
  --> en.phr.rs:2:28
   |
2  |     draw(n) = "Draw {n} {cards:n}.";
   |                          ^^^^^ not defined
   |
   = help: did you mean 'card'?
```

**Missing variant:**

```rust
// ru.phr.rs
phraselet! {
    card = {
        one: "карта",
        other: "карт",  // missing 'few'
    };
}
```

```
error: missing variant 'few' for phrase 'card'
  --> ru.phr.rs:2:5
   |
   = note: Russian requires: one, few, many
```

**Wrong parameter type in calling code:**

```rust
en::draw("three")
```

```
error[E0308]: mismatched types
  --> src/main.rs:5:14
   |
5  |     en::draw("three")
   |              ^^^^^^^ expected `i64`, found `&str`
```

**Missing phrase in translation:**

A build script validates all `.phr.rs` files together:

```
error: phrase 'draw' not defined in ru.phr.rs
  --> en.phr.rs:5:5
   |
5  |     draw(n) = "Draw {n} {card:n}.";
   |     ^^^^ defined in source language
   |
   = help: add to ru.phr.rs:
   |     draw(n) = "...";
```

---

## Design Philosophy

**Logic in Rust, text in Phraselet.**

Phraselet provides atomic text pieces. Complex branching logic stays in Rust
code:

```rust
// Rust code handles the logic
match predicate {
    Predicate::Your(card_predicate) => {
        if is_character(card_predicate) {
            en::ally()  // Phraselet provides the text
        } else {
            en::your_card()
        }
    }
    Predicate::Enemy(card_predicate) => {
        en::enemy()
    }
}
```

This separation keeps Phraselet files simple and translator-friendly, while
allowing arbitrarily complex composition logic in Rust.

**Keywords and formatting are just phrases.**

There's no special keyword syntax. Define phrases that return formatted text:

```rust
phraselet! {
    dissolve = "<k>dissolve</k>";
    materialized = "<k>materialized</k>";
    energy_symbol = "<e>●</e>";

    could_dissolve(target) = "which could {dissolve} {target}";
}
// → "which could <k>dissolve</k> that character"
```

The formatting markup is part of the phrase text and gets interpolated normally.

---

## Summary

| Primitive | Syntax | Purpose |
|-----------|--------|---------|
| Phrase | `name = "text";` | Define text |
| Parameter | `name(p) = "{p}";` | Accept values |
| Variant | `name = { a: "x", b: "y" };` | Multiple forms |
| Selection | `{phrase:selector}` | Choose a variant |
| Metadata tag | `name = "text" :tag;` | Attach metadata for selection/transforms |
| Transform | `{@transform phrase}` | Modify text |

Four primitives, Rust-compatible syntax, compile-time type checking.
