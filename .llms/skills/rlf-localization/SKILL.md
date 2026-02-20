---
name: rlf-localization
description: Work with the RLF (Rust Localization Framework) system for defining and using localized strings. Use when adding new strings, modifying existing phrases, working with the rlf! macro, or connecting serializers to RLF output. Triggers on mentions of RLF, localization, strings, phrases, rlf! macro, or card text display.
---

# RLF Localization System

## Key Files

| Purpose | Location |
|---------|----------|
| Phrase definitions (rlf! macro) | `rules_engine/src/strings/src/strings.rs` |
| Module root | `rules_engine/src/strings/src/lib.rs` |
| RLF helper (bridge to serializers) | `rules_engine/src/display/src/rendering/rlf_helper.rs` |
| UI label rendering | `rules_engine/src/display/src/rendering/interface_rendering.rs` |
| Help text rendering | `rules_engine/src/display/src/rendering/ability_help_text.rs` |
| Label rendering | `rules_engine/src/display/src/rendering/labels.rs` |
| Bracket locale (leak detection) | `rules_engine/src/strings/locales/bracket.rlf` |
| Russian locale | `rules_engine/src/strings/locales/ru.rlf` |
| Bracket locale tests | `rules_engine/tests/parser_tests/tests/bracket_locale_tests.rs` |
| Bracket leak harness | `rules_engine/tests/parser_tests/tests/bracket_locale_leak_harness_tests.rs` |
| RLF lint binary | `rules_engine/src/rlf-lint/` |
| RLF docs | `~/rlf/docs/` |

## Overview

RLF is a localization DSL embedded in Rust via the `rlf!` procedural macro. English phrases are defined at compile-time with full IDE autocomplete. Translations can be loaded at runtime.

The project uses the `global-locale` feature, so generated functions don't take a locale parameter:
```rust
strings::card()          // Returns rlf::Phrase
strings::energy(5)       // Returns rlf::Phrase with parameter
```

## CRITICAL: Each Locale Is Completely Independent

**`strings.rs` defines English only.** When a translation locale (e.g., `ru.rlf`)
is active, every phrase body in `strings.rs` is completely replaced. The Russian
definition of a phrase has zero relationship to the English definition of the
same phrase — different tags, different variants, different transforms, different
text. They share only the phrase **name** and **parameter count**.

Concretely:
- English `card = :a { one: "card", other: "cards" };` uses `:a` tag and
  `one`/`other` variants. The Russian `card` in `ru.rlf` uses `:fem :inan` tags
  and `nom`/`acc`/`gen` × `one`/`few`/`many` variants. The `:a` tag and
  `one`/`other` scheme are irrelevant to Russian.
- English `dissolve_subtype($s) = "Dissolve {@a subtype($s)}."` uses the `@a`
  transform. The Russian definition of `dissolve_subtype` can use completely
  different logic — `{$s:acc}` case selection, `:match` on gender, whatever
  Russian needs. The English `@a` does not exist in the Russian file.
- English pluralization uses `one`/`other`. Russian uses `one`/`few`/`many`/`other`.
  These are independent — the Russian locale file defines its own `:match`
  branches with Russian CLDR categories.

**Do not read `strings.rs` to understand what a non-English locale can or must
do.** Read `strings.rs` only for phrase names and parameter counts. Read the
target `.rlf` file for that locale's actual definitions.

## Core Concepts

### Terms (no parameters)

Static strings, optionally with variants and metadata tags:
```rust
card = :a { one: "card", other: "cards" };
event = :an "event";
energy_symbol = "<color=#00838F>\u{25CF}</color>";
```

- `:a` / `:an` are **tags** (metadata for transforms like `@a`)
- `{ one: "...", other: "..." }` are **variants** (plural forms)

### Phrases (with `$` parameters)

Parameterized templates:
```rust
energy($e) = "<color=#00838F>{$e}\u{25CF}</color>";
cards($n) = :match($n) {
    1: "a card",
    *other: "{$n} cards",
};
```

### Variant Selection with `:`

```rust
{card}        // default → "card"
{card:other}  // named variant → "cards"
{card:$n}     // parameterized (CLDR plural rules)
```

### `:match` — Branch on parameter value

```rust
text_number($n) = :match($n) {
    1: "one",
    2: "two",
    *other: "{$n}",
};
```

Supports numeric literals, CLDR plural categories (`one`, `few`, `many`, `other`), and tag-based branching.

### `:from` — Metadata inheritance

Lets a phrase-returning-phrase pass through tags and variants from its argument:
```rust
subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";
```

When called as `subtype(ancient)`, the result inherits the `:an` tag and `one`/`other` variants from `ancient`. This enables:
```rust
// @a reads the :an tag inherited through subtype → "an Ancient"
dissolve_subtype($s) = "Dissolve {@a subtype($s)}.";
```

`:from` automatically propagates the parameter's variants to the result. If
`warrior` has `one` and `other` variants, `subtype(warrior)` automatically gets
both variants without explicit variant blocks.

**Body-less `:from`** (transparent passthrough): `= :from($p);` passes through
text, tags, and all variants unchanged. Essential for Russian localization where
English-specific phrases (articles, a/an) become identity functions:
```rust
predicate_with_article($p) = :from($p);
```

`:from` also supports **variant blocks** for per-variant templates, and
`:match` inside variant entries for tag-based branching. Compound tag keys like
`masc.anim` match when the parameter has ALL listed tags. This enables
modifiers to agree in both case and gender with the noun:
```rust
enemy_subtype($s) = :from($s) {
    nom: :match($s) { masc: "вражеский {$s}", *fem: "вражеская {$s}" },
    acc: :match($s) { masc.anim: "вражеского {$s}", *fem: "вражескую {$s}" },
    *gen: :match($s) { masc: "вражеского {$s}", *fem: "вражеской {$s}" },
};
```

### Transforms — The `@` operator

Modify text output. Universal transforms:
- `@cap` — capitalize first letter (skips `<...>` markup tags, works with Cyrillic)
- `@upper` / `@lower` — locale-aware case conversion (works with Cyrillic)

English-specific:
- `@a` — indefinite article (reads `:a`/`:an` tags)
- `@the` — definite article

Usage in templates:
```rust
help_text_dissolve = "{@cap dissolve}: Send a character to the void";
// "Dissolve: Send a character to the void"
```

## Adding a New String

### Step 1: Add to `strings.rs`

Add in the appropriate section of the `rlf!` macro block:

**Simple term (no parameters):**
```rust
// Doc comment describing the string.
my_new_label = "Some text";
```

**With plural variants:**
```rust
// Doc comment.
thing = :a { one: "thing", other: "things" };
```

**Parameterized phrase:**
```rust
// Doc comment.
gain_points($p) = "Gain {points($p)}.";
```

**With match branching:**
```rust
// Doc comment.
n_things($n) = :match($n) {
    1: "a thing",
    *other: "{$n} things",
};
```

### Step 2: Use from Rust code

**Direct use (UI labels, static text):**
```rust
use strings::strings;

let label = strings::my_new_label().to_string();
let text = strings::gain_points(3).to_string();
```

**Via rlf_helper for serializer templates:**
```rust
// In effect_serializer.rs or similar:
// Template strings reference phrases by name
let template = "Gain {points($p)}.";
rlf_helper::eval_str(template, &bindings)
```

### Step 3: Ensure registration

Any code path using RLF must call `strings::register_source_phrases()` first. This is idempotent (uses `std::sync::Once` internally). Already called in:
- `rlf_helper::eval_str()`
- `labels::choice_label()`

## How Serializers Use RLF

Serializers convert effect data structures into display text. Two patterns:

### Pattern 1: Direct function calls (preferred for UI)
```rust
strings::energy(cost.0).to_string()
strings::dissolve().to_string()
```

### Pattern 2: Template evaluation (for card ability text)

Templates reference RLF phrases by name. Variable bindings provide parameter values:
```rust
// Template: "Draw {cards($n)}."
// Bindings: { "n" => Integer(2) }
rlf_helper::eval_str(template, &bindings)
// Result: "Draw 2 cards."
```

The `rlf_helper::build_params()` function converts `VariableBindings` (with `VariableValue::Integer`, `VariableValue::Subtype`, `VariableValue::Figment`) into RLF `Value` types.

## Subtype and Figment Phrases

Subtypes and figments have special handling:

```rust
// In rlf_helper.rs:
pub fn subtype_phrase(subtype: CardSubtype) -> Phrase { ... }
fn figment_phrase(figment: FigmentType) -> Phrase { ... }
```

When adding a new subtype:
1. Add term in `strings.rs` with `:a`/`:an` tag and `one`/`other` variants
2. Add match arm in `rlf_helper::subtype_phrase()`

When adding a new figment type:
1. Add term in `strings.rs`
2. Add match arm in `rlf_helper::figment_phrase()`

## Common Patterns in strings.rs

| Pattern | Example | Use For |
|---------|---------|---------|
| Colored symbol | `energy_symbol = "<color=...>..."` | Icons |
| Parameterized format | `energy($e) = "<color=...>{$e}..."` | Inline values |
| Keyword | `dissolve = "<color=#AA00FF>dissolve</color>"` | Game keywords |
| Plural-aware count | `:match($n) { 1: "a X", *other: "{$n} Xs" }` | Quantities |
| Subtype term | `warrior = :a { one: "Warrior", other: "Warriors" }` | Card subtypes |
| Metadata inheritance | `subtype($s) = :from($s) "<b>{$s}</b>"` | Formatted pass-through |
| Text numbers | `text_number($n) = :match($n) { 1: "one", ... }` | Readable counts |

## Translation Files & Locale API

### Creating a translation file (`.rlf`)

Translation files use the same syntax as `rlf!` but in standalone files. Every
phrase from `strings.rs` must have a definition with matching name and parameter
count. Located in `rules_engine/src/strings/locales/`.

**Reminder:** Each definition completely replaces the English one. The tags,
variants, transforms, and text are entirely yours to define. Nothing from the
English definition carries over except the name and parameter count.

Example Russian definitions:
```
card = :fem :inan { nom: "карта", acc: "карту", gen: "карты" };
dissolve_target($t) = :from($t) "{dissolve} {$t:acc}";
predicate_with_article($p) = :from($p);
```

### Loading and switching locales

```rust
strings::register_source_phrases();
rlf::with_locale_mut(|locale| {
    locale.load_translations("ru", "path/to/ru.rlf")?;
    locale.set_language("ru");  // All strings::foo() now resolve to Russian
    Ok(())
});
// Switch back: locale.set_language("en")
```

### Validation and linting

```rust
// Check translation completeness/correctness
locale.validate_translations("en", "ru")  // Returns Vec<Warning>

// Static linting (AST-based, no evaluation needed)
rlf::lint_definitions(defs, "ru")  // Returns Vec<LintWarning>
```

The `rlf-lint` binary crate (`rules_engine/src/rlf-lint/`) runs both static
and runtime linting. Integrated into `just review`.

Key lint rules: `InvalidTag` (wrong tags for language), `InvalidVariantKey`
(wrong case/plural names), `LikelyMissingFrom` (parameter used with selector
but no `:from`), `VerboseTransparentWrapper` (`:from($p) "{$p}"` should use
body-less form), `ParameterCountMismatch`, `UnknownPhrase`.

### Multi-dimensional variants for Russian

Russian nouns need case × number variants with gender/animacy tags:
```
card = :fem :inan {
    nom: "карта", nom.few: "карты", nom.many: "карт",
    acc: "карту", acc.few: "карты", acc.many: "карт",
    gen: "карты", gen.few: "карт", gen.many: "карт",
};
```

Wildcard fallback: selecting `nom.few` first tries exact match, then falls back
to `nom` if `nom.few` isn't defined. Only define sub-variants where the form
differs.

Valid Russian tags: `:masc`, `:fem`, `:neut`, `:anim`, `:inan`.
Valid Russian variant keys: `nom`, `acc`, `gen`, `dat`, `ins`, `prep` ×
`one`, `few`, `many`, `other`.

Russian CLDR plurals: `one` (1, 21, 31…), `few` (2-4, 22-24…),
`many` (5-20, 25-30…), `other` (fractional).

### Bracket locale (leak detection)

`bracket.rlf` wraps every string literal in `[brackets]`. The leak harness
(`bracket_locale_leak_harness_tests.rs`) renders all card abilities through the
bracket locale and detects unbracketed text — strings that bypass RLF entirely
(hardcoded in Rust). This catches serializer bugs where text doesn't flow
through phrase definitions.

## RLF Language Reference

For the full RLF syntax specification, variant selection rules, and
multi-language support details, see
[references/rlf-syntax.md](references/rlf-syntax.md).
