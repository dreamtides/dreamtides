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
{term:*}              // Explicit default selector (first variant)
```

For terms like `card = :a { one: "card", other: "cards" }`:
- `{card}` → "card" (default = first variant)
- `{card:one}` → "card"
- `{card:other}` → "cards"
- `{card:$n}` → CLDR plural category for n (1→"card", 2→"cards", etc.)
- `{card:*}` → "card" (explicit default, same as bare `{card}`)

### Multi-dimensional variants

For languages needing case + number (e.g., Russian):
```
card = :fem :inan {
    nom: "карта", nom.few: "карты", nom.many: "карт",
    acc: "карту", acc.few: "карты", acc.many: "карт",
    gen: "карты", gen.few: "карт", gen.many: "карт",
};
```

Selected with chained `:` operators: `{card:acc:$n}`

### Multi-key shorthand

Multiple variant keys can share one value:
```
card = :fem :inan {
    nom, acc: "карту",
    nom.few, acc.few: "карты",
};
```

### Wildcard fallback resolution

When selecting a multi-dimensional variant like `nom.few`, the resolver tries
in order:
1. Exact match: `nom.few`
2. Prefix match: `nom` (catches `nom.one`, `nom.few`, `nom.many` etc.)
3. Default variant (first defined)

This means you only need to define specific sub-variants where the form differs
from the base case form. For example, if nominative singular and plural share a
base form, just define `nom` and override with `nom.few` and `nom.many` only
where they differ.

## Tags (Metadata)

Tags are `:name` prefixes on definitions. They attach metadata used by
transforms and selection:

```
card = :a { one: "card", other: "cards" };     // English: takes "a"
event = :an "event";                            // English: takes "an"
enemy = :masc :anim { nom: "враг", acc: "врага" };  // Russian: masculine animate
```

Tags enable:
1. **Transforms** read tags: `@a` reads `:a`/`:an` to output correct article
2. **Tag-based selection**: `:match($thing) { a: "...", an: "..." }`
3. **Dynamic selection**: `{adjective:$entity}` reads entity's first tag

### Valid tags by language

| Language | Tags |
|----------|------|
| English | `a`, `an` |
| Russian | `masc`, `fem`, `neut`, `anim`, `inan` |
| German | `masc`, `fem`, `neut` |
| Spanish | `masc`, `fem` |
| French | `masc`, `fem`, `vowel` |

### Valid variant key components by language

| Language | Case keys | Number keys |
|----------|-----------|-------------|
| English | — | `one`, `other` |
| Russian | `nom`, `acc`, `gen`, `dat`, `ins`, `prep` | `one`, `few`, `many`, `other` |
| German | `nom`, `acc`, `dat`, `gen` | `one`, `other` |
| Polish | `nom`, `acc`, `gen`, `dat`, `ins`, `loc`, `voc` | `one`, `few`, `many`, `other` |

## Transforms (`@` operator)

Transforms modify interpolated content. Syntax:

```
{@transform ref}                 // No context
{@transform:literal ref}        // Static context (e.g., @der:acc)
{@transform($param) ref}        // Dynamic context (e.g., @count($n))
```

### Universal transforms

| Transform | Effect |
|-----------|--------|
| `@cap` | Capitalize first letter (skips `<...>` markup tags) |
| `@upper` | All uppercase (locale-aware, works with Cyrillic) |
| `@lower` | All lowercase (locale-aware, works with Cyrillic) |

### English transforms

| Transform | Reads Tag | Output |
|-----------|-----------|--------|
| `@a` | `:a` / `:an` | "a card" / "an event" |
| `@the` | — | "the card" |
| `@plural` | — | Selects `other` variant |

### Other language transforms

See `~/rlf/docs/APPENDIX_STDLIB.md` for transforms available in German (`@der`,
`@ein`), French (`@le`, `@un`, `@de`), Spanish (`@el`, `@un`), Italian, Dutch,
Greek, Arabic, Turkish (`@inflect`), Finnish, Hungarian, Hindi, Korean/Japanese
(`@particle`), and others.

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
- **Tag-based**: `:match($phrase) { a: "...", an: "..." }` — reads phrase's tags
- **Compound tags**: `masc.anim:` — matches when parameter has ALL listed tags

Resolution order: exact numeric → CLDR plural category → tag match → `*` default.

### Compound tag matching

When a `:match` key contains dots (e.g., `masc.anim`), all listed tags must be
present on the parameter. This enables distinctions like Russian masculine
animate vs. masculine inanimate:

```
target($t) = :match($t) {
    masc.anim: "вражеского {$t:acc}",
    *other: "{$t:acc}",
};
```

### Multi-parameter match

`:match` can branch on two parameters using dot notation:

```
phrase($n, $g) = :match($n, $g) {
    1.masc: "один",
    1.fem: "одна",
    *other: "{$n}",
};
```

## `:from` Keyword

Inherit metadata (tags + variants) from a parameter:

```
subtype($s) = :from($s) "<b>{$s}</b>";
```

When called with `subtype(warrior)` where
`warrior = :a { one: "Warrior", other: "Warriors" }`:
- Result text: `<b>Warrior</b>` (default variant)
- Result inherits `:a` tag → `@a` works on it
- Result has `one`/`other` variants → `{subtype($s):other}` works

This is critical for composing formatted terms while preserving grammatical
metadata.

### Automatic variant propagation

`:from` automatically iterates the parameter's variants and generates
corresponding variants on the result. If `warrior` has `one` and `other`
variants, then `subtype(warrior)` automatically produces `one` and `other`
variants containing `<b>Warrior</b>` and `<b>Warriors</b>` respectively —
without needing explicit variant blocks.

### Body-less `:from` (transparent passthrough)

The syntax `= :from($p);` (note the semicolon, no body) creates a transparent
wrapper that passes through text, tags, and all variants unchanged:

```
// English article logic — becomes identity in Russian
predicate_with_article($p) = :from($p);
```

This is essential for Russian localization: many English phrases that apply
English-specific logic (articles, a/an selection) become transparent
passthroughs in Russian.

### Variant blocks in `:from`

`:from` can include explicit variant blocks for per-variant templates. The
variant keys correspond to the parameter's variant dimensions:

```
allied($s) = :from($s) {
    nom: "союзный {$s:nom}",
    acc: "союзного {$s:acc}",
    *gen: "союзного {$s:gen}",
};
```

### `:match` inside `:from` variant entries

Variant entries in a `:from` block can contain `:match` for tag-based
branching. This enables modifiers to agree in both case and gender with the
noun:

```
enemy_subtype($s) = :from($s) {
    nom: :match($s) { masc: "вражеский {$s}", *fem: "вражеская {$s}" },
    acc: :match($s) { masc.anim: "вражеского {$s}", *fem: "вражескую {$s}" },
    *gen: :match($s) { masc: "вражеского {$s}", *fem: "вражеской {$s}" },
};
```

## Phrase Composition

Phrases can call other phrases:

```
text_number($n) = :match($n) { 1: "one", 2: "two", *other: "{$n}" };
copies($n) = :match($n) {
    1: "a copy",
    *other: "{text_number($n)} copies",
};
```

When `copies(3)` is called, `text_number(3)` resolves to "three", producing
"three copies".

## Translation Files (`.rlf`)

Translation files override source phrase definitions for a target language.
They use the same syntax as the `rlf!` macro but in a standalone file.

### File format

Each `.rlf` file contains phrase definitions, one per line (or spanning
multiple lines for variant blocks). Every phrase from the source `rlf!` macro
must have a corresponding definition. Names and parameter counts must match.

```
// Russian translations (ru.rlf)
card = :fem :inan { nom: "карта", acc: "карту", gen: "карты" };
energy($e) = "<color=#00838F>{$e}●</color>";
dissolve_target($t) = :from($t) "{dissolve} {$t:acc}";
predicate_with_article($p) = :from($p);
```

### Loading translations

```rust
strings::register_source_phrases();
rlf::with_locale_mut(|locale| {
    locale.load_translations("ru", "path/to/ru.rlf")?;
    locale.set_language("ru");
    Ok(())
});
```

After `set_language("ru")`, all `strings::foo()` calls resolve to Russian
definitions. Switching back: `locale.set_language("en")`.

### Validation

```rust
rlf::with_locale(|locale| {
    locale.validate_translations("en", "ru")  // Returns Vec<Warning>
});
```

Checks parameter count mismatches, unknown phrases, etc.

## CLDR Plural Categories

| Language | Categories |
|----------|------------|
| English | `one` (1), `other` (0, 2+) |
| Russian | `one` (1, 21, 31…), `few` (2-4, 22-24…), `many` (5-20, 25-30…), `other` (fractional) |
| German | `one` (1), `other` (0, 2+) |
| Polish | `one` (1), `few` (2-4, 22-24…), `many` (5-20, 25-30…), `other` |
| Arabic | `zero` (0), `one` (1), `two` (2), `few` (3-10), `many` (11-99), `other` |

Full list: 23 languages supported. See `rlf::plural_category(lang, n)`.

## Linting

### Static lints (`rlf::lint_definitions`)

Analyzes AST without evaluation. Available lint rules:

| Lint | Description |
|------|-------------|
| `RedundantPassthroughBlock` | `:from` with unnecessary explicit variant blocks |
| `RedundantFromSelector` | Selector that duplicates the enclosing variant key |
| `LikelyMissingFrom` | Parameter used with selector but no `:from` |
| `VerboseTransparentWrapper` | `:from($p) "{$p}"` that could use body-less form |
| `InvalidTag` | Unrecognized tag for the language |
| `InvalidVariantKey` | Unrecognized case/plural category for the language |
| `ParameterCountMismatch` | Translation has different parameter count than source |
| `UnknownPhrase` | Translation defines phrase not in source |

### Runtime warnings

Emitted during evaluation:
- `PhraseArgumentWithoutFrom` — Phrase value passed without `:from`
- `MissingSelectorOnMultiDimensional` — Bare reference to multi-variant Phrase

## Generated Rust API

The `rlf!` macro generates:
- One function per definition (term or phrase)
- `register_source_phrases()` — registers all phrases in the global locale
- `phrase_ids` module with `PhraseId` constants
- `SOURCE_PHRASES` constant with embedded definitions

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

`Phrase` implements `Display` (returns `text`) and `Into<Value>` for use as a
phrase argument.

## Value Type

```rust
pub enum Value {
    Number(i64),
    Float(f64),
    String(String),
    Phrase(Phrase),
}
```

`impl From<u32>`, `impl From<i64>`, `impl From<Phrase>` etc. allow ergonomic
parameter passing.

## Error Handling

Generated functions panic on errors (programming bugs). For data-driven use:
```rust
locale.eval_str(template, params)  // Returns Result<Phrase, EvalError>
locale.get_phrase("name")          // Returns Result<Phrase, EvalError>
```

Error types: `PhraseNotFound`, `MissingVariant`, `MissingTag`, `ArgumentCount`,
`CyclicReference`, `MaxDepthExceeded`, `UnknownTransform`, `ArgumentsToTerm`,
`SelectorOnPhrase`, `MissingMatchDefault`, `UnknownParameter`.
