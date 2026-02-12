# RLF Syntax Reference

## Definitions

### Terms (no parameters)

```
name = "literal text";
name = :tag "text with tag";
name = :tag1 :tag2 { variant1: "text1", variant2: "text2" };
```

### Phrases (with parameters)

```
name($param) = "text with {$param}";
name($a, $b) = "text {$a} and {$b}";
```

## Interpolation

Inside template strings, `{...}` is an interpolation:

```
{$param}              // Parameter reference
{term_name}           // Term reference (returns default variant)
{phrase_name($arg)}   // Phrase call
```

## Variant Selection

The `:` operator selects among variants:

```
{term:variant_name}   // Static literal selector
{term:$param}         // Dynamic parameterized selector (CLDR plural rules)
{term:other}          // Select "other" variant explicitly
```

For terms like `card = :a { one: "card", other: "cards" }`:
- `{card}` → "card" (default = first variant)
- `{card:one}` → "card"
- `{card:other}` → "cards"
- `{card:$n}` → CLDR plural category for n (1→"card", 2→"cards", etc.)

### Multi-dimensional variants

For languages needing case + number (e.g., Russian):
```
card = :fem {
    nom.one: "карта",
    nom.other: "карты",
    acc.one: "карту",
    gen.many: "карт",
};
```

Selected with chained `:` operators: `{card:acc:$n}`

## Tags (Metadata)

Tags are `:name` prefixes on definitions. They attach metadata used by transforms and selection:

```
card = :a { one: "card", other: "cards" };     // English: takes "a"
event = :an "event";                            // English: takes "an"
```

Tags enable:
1. **Transforms** read tags: `@a` reads `:a`/`:an` to output correct article
2. **Tag-based selection**: `:match($thing) { a: "...", an: "..." }`
3. **Dynamic selection**: `{adjective:$entity}` reads entity's first tag

## Transforms (`@` operator)

Transforms modify interpolated content. Syntax:

```
{@transform ref}                 // No context
{@transform:literal ref}        // Static context
{@transform($param) ref}        // Dynamic context
```

### Universal transforms

| Transform | Effect |
|-----------|--------|
| `@cap` | Capitalize first letter |
| `@upper` | All uppercase |
| `@lower` | All lowercase |

### English transforms

| Transform | Reads Tag | Output |
|-----------|-----------|--------|
| `@a` | `:a` / `:an` | "a card" / "an event" |
| `@the` | — | "the card" |
| `@plural` | — | Selects `other` variant |

### Nesting transforms

Transforms apply right-to-left (innermost first):
```
{@cap @a card}  → "A card"   // @a produces "a card", @cap capitalizes
```

## `:match` Keyword

Branch on a parameter value. Must include `*` default branch:

```
cards($n) = :match($n) {
    0: "no cards",
    1: "a card",
    *other: "{$n} cards",
};
```

### Match types

- **Numeric**: `0:`, `1:`, `42:` — exact number match
- **CLDR plural**: `one:`, `few:`, `many:`, `other:` — language plural rules
- **Tag-based**: `:match($phrase) { a: "...", an: "..." }` — reads phrase's tag

The `*` prefix marks the default branch (required).

## `:from` Keyword

Inherit metadata (tags + variants) from a parameter:

```
subtype($s) = :from($s) "<b>{$s}</b>";
```

When called with `subtype(warrior)` where `warrior = :a { one: "Warrior", other: "Warriors" }`:
- Result text: `<b>Warrior</b>` (default variant)
- Result inherits `:a` tag → `@a` works on it
- Result has `one`/`other` variants → `{subtype($s):other}` works

This is critical for composing formatted terms while preserving grammatical metadata.

## Phrase Composition

Phrases can call other phrases:

```
text_number($n) = :match($n) { 1: "one", 2: "two", *other: "{$n}" };
copies($n) = :match($n) {
    1: "a copy",
    *other: "{text_number($n)} copies",
};
```

When `copies(3)` is called, `text_number(3)` resolves to "three", producing "three copies".

## Generated Rust API

The `rlf!` macro generates:
- One function per definition (term or phrase)
- `register_source_phrases()` — registers all phrases in the global locale
- `phrase_ids` module with `PhraseId` constants

With `global-locale` feature (used in Dreamtides):
```rust
// Terms → no-arg functions returning Phrase
strings::card()              // → Phrase { text: "card", ... }

// Phrases → functions taking parameters, returning Phrase
strings::energy(5)           // → Phrase { text: "<color=...>5●</color>", ... }
strings::cards(2)            // → Phrase { text: "2 cards", ... }
strings::subtype(ancient)    // → Phrase with inherited tags

// Dynamic evaluation
rlf::with_locale(|locale| {
    locale.eval_str("Draw {cards($n)}.", params)
})
```

## Phrase Type

```rust
pub struct Phrase {
    pub text: String,                           // Rendered default text
    pub variants: HashMap<VariantKey, String>,   // Available variants
    pub tags: Vec<Tag>,                          // Metadata tags
}
```

`Phrase` implements `Display` (returns `text`) and `Into<Value>` for use as a phrase argument.

## Value Type

```rust
pub enum Value {
    Number(i64),
    Float(f64),
    String(String),
    Phrase(Phrase),
}
```

`impl From<u32>`, `impl From<i64>`, `impl From<Phrase>` etc. allow ergonomic parameter passing.

## CLDR Plural Categories (English)

| Category | Numbers |
|----------|---------|
| `one` | 1 |
| `other` | 0, 2, 3, 4, 5, ... |

Other languages have additional categories (`few`, `many`). RLF uses the CLDR plural rules database.

## Error Handling

Generated functions panic on errors (programming bugs). For data-driven use:
```rust
locale.eval_str(template, params)  // Returns Result<Phrase, EvalError>
locale.get_phrase("name")          // Returns Result<Phrase, EvalError>
```

Error types: `PhraseNotFound`, `MissingVariant`, `MissingTag`, `ArgumentCount`, `CyclicReference`, `MaxDepthExceeded`, `UnknownTransform`.
