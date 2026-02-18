# RLF Localization System

The RLF (Rust Localization Framework) system handles all user-visible text in
Dreamtides. It is an in-house DSL that generates typed Rust functions from
phrase definitions, producing rich text with Unity-compatible markup. Every UI
string — card abilities, keywords, help text, interface labels — flows through
RLF.

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Phrase Definition Syntax](#phrase-definition-syntax)
- [Phrase Composition and the Phrase Type](#phrase-composition-and-the-phrase-type)
- [Rich Text Conventions](#rich-text-conventions)
- [Serializer Integration](#serializer-integration)
- [Parser Integration](#parser-integration)
- [Locales](#locales)
- [Tooling](#tooling)
- [Adding a New Phrase](#adding-a-new-phrase)

## Architecture Overview

RLF is an external crate (`github.com/thurn/rlf`) with three sub-crates: `rlf`
(runtime library), `rlf-macros` (proc macro), and `rlf-semantics`. The project
uses these through a single `rlf::rlf!` macro invocation in
`strings/src/strings.rlf.rs`, which defines all phrase templates and generates
public Rust functions in the `strings` module.

The system has two consumers:

- **Serializers** (display path): Walk parsed Ability ASTs and call
  `strings::*()` functions to produce rich text for the game UI. This is the
  primary runtime path.
- **Parser variable resolution** (parsing path): Resolves RLF function-call
  syntax in TOML rules-text directives (like `{energy($e)}`) into typed
  `ResolvedToken` values. This happens at parse time during `tabula generate`,
  not at runtime.

Key files:

- `strings/src/strings.rlf.rs` — all phrase definitions (the single source of
  truth for UI text)
- `strings/locales/ru.rlf` — Russian locale override
- `strings/locales/bracket.rlf` — bracket-wrapped test locale
- `parser_v2/src/serializer/` — serializer modules that call `strings::*`
  functions
- `parser_v2/src/variables/parser_substitutions.rs` — parser-side RLF syntax
  resolution

## Phrase Definition Syntax

All phrases are defined inside a single `rlf::rlf! { ... }` block in
`strings.rlf.rs`. The `rlf!` proc macro parses each definition and generates a
corresponding public Rust function that returns an `rlf::Phrase`.

### Constant Phrases

A phrase with no parameters generates a no-argument function.

Definition: `energy_symbol = "<color=#00838F>\u{25CF}</color>";`

Generated: `pub fn energy_symbol() -> Phrase`

### Parameterized Phrases

Parameters are prefixed with `$` in both the signature and template
interpolations. Generated functions accept `impl Into<Value>`, so callers can
pass integers, strings, or other Phrases directly.

Definition: `energy($e) = "<color=#00838F>{$e}\u{25CF}</color>";`

Generated: `pub fn energy(e: impl Into<Value>) -> Phrase`

### Inline References

Curly braces reference other phrases by name. Parameterized phrases can be
called inline with arguments.

Definition: `maximum_energy($max) = "{$max} maximum {energy_symbol}";`

Here `{energy_symbol}` inlines the constant phrase, and `{$max}` interpolates
the parameter.

### Variant Blocks (Singular/Plural)

Curly-brace blocks define variant entries keyed by CLDR plural categories
(`one`, `other`). The first entry is the default unless one is marked with `*`.

Definition: `card = :a { one: "card", other: "cards" };`

The `:a` tag is metadata (see Tags below). `one` is the singular form, `other`
is the plural.

### Match Expressions

`:match($param)` dispatches on a parameter's value. Integer keys match exact
values; named keys use CLDR plural categories. Exactly one branch must be marked
`*` as the default.

Definition: `cards($n) = :match($n) { 1: "a card", *other: "{$n} cards" };`

When `$n` is 1, produces "a card". For any other value, produces "{n} cards".

### Tags (Variant Metadata)

Tags are metadata annotations that appear after `=` and before the body. They
carry grammatical information used by transforms.

- `:a` — indicates the indefinite article is "a"
- `:an` — indicates the indefinite article is "an"

Definition: `ally = :an { one: "ally", other: "allies" };`

Tags are stored on the resulting Phrase and accessible at runtime via
`phrase.has_tag("a")`. The `@a` transform reads these to choose the correct
article.

### The `:from` Modifier

`:from($param)` causes the resulting Phrase to inherit tags and variants from
the parameter. When the source has variants (like `one`/`other`), the template
is evaluated once per variant, propagating singular/plural behavior through
composition.

Definition: `dissolve_target($target) = :from($target) "{dissolve} {$target}";`

When called with a Phrase that has `one`/"ally" and `other`/"allies" variants,
the result inherits both variants: "dissolve ally" / "dissolve allies", plus the
`:an` tag from the source.

`:from` can also combine with explicit variant blocks for per-variant templates,
or with `:match` for combined variant inheritance and plural dispatch.

### Variant Selection

The `:selector` suffix on interpolations selects a specific variant from a
Phrase.

- `{card:$n}` — selects the variant matching `$n`'s CLDR plural category
- `{$target:other}` — selects the "other" (plural) variant explicitly
- `{pronoun:$n}` — selects a pronoun variant based on count

When the selector is a numeric `$param`, CLDR rules determine the key (for
English: 1 maps to "one", everything else to "other"). Literal selectors like
`:other` select directly.

### The `*` Default Marker

Within a variant block, prefixing a key with `*` marks it as the default variant
(the text used when the phrase is rendered without a selector). Without `*`, the
first entry is the default.

### Transforms

Transforms are prefixed with `@` inside interpolations and modify the output.

- `@cap` — capitalizes the first visible character, skipping over rich text
  markup tags. Uses ICU4X locale-aware case mapping.
- `@a` — prepends the English indefinite article ("a" or "an") based on the
  phrase's `:a`/`:an` tag.

Definition: `capitalized_sentence($s) = "{@cap $s}";`

This is the most commonly used composition phrase in the codebase — nearly every
ability's final text passes through it.

Auto-capitalization shorthand: an interpolation starting with an uppercase
letter (like `{Judgment}`) is automatically rewritten to `{@cap judgment}` by
the macro parser.

## Phrase Composition and the Phrase Type

### The Phrase Struct

An `rlf::Phrase` has three fields:

- `text: String` — the default display text
- `variants: HashMap<VariantKey, String>` — variant key to variant text
- `tags: Vec<Tag>` — metadata tags

### Key Methods

- `Phrase::empty()` — returns a phrase with no text, variants, or tags. Used as
  an identity value in serializers.
- `map_text(f)` — transforms the text while preserving variants and tags. Used
  for stripping trailing periods or appending suffixes.
- `variant(key)` — looks up a variant with progressive fallback (strips trailing
  dot-segments). Panics if no match found.
- `has_tag(tag)` — checks for a metadata tag.
- `join(phrases, separator)` — joins multiple phrases with a separator. Only
  variant keys present in all phrases are preserved. Used for effect lists.
- `to_string()` / `Display` — returns the `text` field.

### The Value Type

`rlf::Value` is the dynamic type for phrase parameters:

- `Value::Number(i64)` — for counts and numeric parameters
- `Value::Float(f64)` — for decimal values
- `Value::String(String)` — for text parameters
- `Value::Phrase(Phrase)` — for composed phrase parameters

Extensive `From` implementations allow passing `u32`, `i32`, `String`, `&str`,
or `Phrase` directly to generated functions via `impl Into<Value>`.

### Composition Flow

A typical composition chain for card ability text:

- Leaf phrases define vocabulary with variants and tags (e.g., `card` with
  one/other)
- Parameterized phrases compose leaves with `:match` for pluralization (e.g.,
  `cards($n)`)
- Effect phrases use `:from` for variant propagation (e.g.,
  `dissolve_target($target)`)
- Assembly phrases capitalize and punctuate (e.g., `capitalized_sentence($s)`)
- Rust serializer code calls generated functions, composes Phrases, and calls
  `.to_string()` at the end

## Rich Text Conventions

RLF phrases embed Unity-compatible rich text tags (`<color=#HEX>`, `<b>`, `<u>`)
directly in their template strings. These tags pass through unchanged to Unity's
TextMeshPro for rendering. Color hex values and styling choices are hardcoded
inline in the phrase definitions — refer to `strings.rlf.rs` for the current
values. Different semantic categories (keywords, energy, subtypes, etc.) have
distinct color and formatting treatments.

The `@cap` transform is aware of rich text tags — it skips over markup to
capitalize the first visible character.

The utility function `strip_colors()` in `display/src/core/text_utils.rs`
removes color markup while preserving inner text, used for UI contexts that need
uncolored output.

## Serializer Integration

The serializer system is the primary consumer of RLF phrases at runtime. It
converts parsed Ability ASTs into rich display text by pattern-matching on AST
nodes and calling `strings::*` functions.

### Serializer Modules

Each ability type has a dedicated serializer:

- `ability_serializer.rs` — entry point, dispatches by Ability variant
- `effect_serializer.rs` — StandardEffect to Phrase (the largest module)
- `trigger_serializer.rs` — TriggerEvent to Phrase
- `cost_serializer.rs` — Cost to Phrase
- `predicate_serializer.rs` — Predicate/CardPredicate to Phrase (variant-aware)
- `static_ability_serializer.rs` — StaticAbility to Phrase
- `condition_serializer.rs` — Condition to Phrase
- `serializer_utils.rs` — helpers for operators, subtypes, figments

### How Serializers Produce Text

The entry point `serialize_ability()` takes an `&Ability` and returns a
`SerializedAbility` with a `.text: String` field. Each branch delegates to the
appropriate sub-serializer, wraps the result in
`strings::capitalized_sentence()` for proper capitalization, and calls
`.to_string()`.

Sub-serializers follow a consistent pattern: pattern-match on the AST node, then
call a `strings::*` function with concrete Rust values (integers, sub-phrases).
The generated functions handle pluralization, variant selection, and rich text
markup internally.

Phrases compose hierarchically — inner serializer calls return Phrases that are
passed as parameters to outer phrase functions. The final `.to_string()` at the
top level extracts the rendered text with all markup embedded.

### The Display Layer

The display crate's `card_rendering.rs` calls
`ability_serializer::serialize_ability()` to build per-ability rules text lines,
joined with line breaks for the final card text display. A separate
`serialize_ability_effect()` entry point returns just the effect portion (no
costs) for ability token text.

## Parser Integration

The parser's variable resolution stage also uses RLF-like syntax, but this is a
separate path from the display serialization. The function
`resolve_rlf_syntax()` in `parser_substitutions.rs` handles directive strings
that appear in TOML rules-text.

### What It Resolves

When a card's rules-text contains directives like `{energy($e)}`,
`{@a subtype($t)}`, or `{subtype($t):other}`, the variable resolver needs to
convert these into typed `ResolvedToken` values for the parser.
`resolve_rlf_syntax()` handles the RLF function-call syntax specifically.

### Resolution Steps

The function processes directive strings through these steps:

- Strip transform prefixes (`@cap`, `@a`, `@plural`)
- Strip selector suffixes (`:other`, `:$n`)
- Parse the parenthesized function call to extract phrase name and arguments
- Look up the phrase name against the four phrase tables (`PHRASES`,
  `BARE_PHRASES`, `SUBTYPE_PHRASES`, `FIGMENT_PHRASES`)
- Resolve variable bindings from the card's `variables` field to produce
  concrete `ResolvedToken` values

For example, `energy($e)` with binding `e: 2` resolves to
`ResolvedToken::Energy(2)`. This is a parse-time operation that produces typed
AST data, entirely separate from the display-time serialization that produces
formatted text.

### Phrase Tables

The four static tables in `parser_substitutions.rs` map phrase names to their
default variable names and typed constructors:

- `PHRASES` — integer-valued phrases (energy, cards, spark, points, etc.)
- `BARE_PHRASES` — no-parameter phrases (choose_one, energy_symbol,
  judgment_phase_name)
- `SUBTYPE_PHRASES` — subtype-valued phrases (subtype, a_subtype,
  plural_subtype)
- `FIGMENT_PHRASES` — figment-valued phrases (figment, figments)

## Locales

RLF supports locale overrides through separate `.rlf` files. The project has two
additional locale files in `strings/locales/`:

- `ru.rlf` — Russian locale with translated phrase overrides
- `bracket.rlf` — test locale that wraps all text in brackets for visual
  verification of localization coverage

Locale files override specific phrases from the main `strings.rlf.rs` source.
Phrases not overridden fall through to the English defaults. The global locale
is initialized lazily via `std::sync::Once` on first phrase access.

## Tooling

### rlf_fmt (Formatter)

The `rlf_fmt` binary formats RLF phrase definition files. Run via `just fmt`
(which also runs `style_validator --fix` and `cargo +nightly fmt`).

It formats two kinds of files:

- `.rlf` locale files in `strings/locales/`
- The `strings.rlf.rs` Rust source file (extracts and reformats the `rlf!` macro
  body)

Formatting rules include: 100-character max line width, block expansion to
multi-line when needed, tag spacing normalization (`:tag{` becomes `:tag {`),
trailing comma cleanup, and break-at-`=` for long single-line definitions. The
formatter preserves comments and blank lines and is idempotent.

### rlf_lint (Linter)

The `rlf_lint` binary validates RLF phrase definitions for common issues. Run as
part of `just review` (the full pre-push validation gate).

Static lints (AST analysis):

- **Redundant passthrough blocks** — `:from` phrases with variant blocks where
  every entry just passes its key through to the parameter
- **Redundant from-selectors** — explicit variant selectors inside variant
  blocks that match the enclosing key
- **Likely missing `:from`** — phrases using variant selectors without `:from`
  or tags, which silently loses variant metadata
- **Verbose transparent wrappers** — `:from($p) "{$p}"` patterns that can be
  simplified to `:from($p);`

Runtime lints:

- Evaluates each phrase with dummy arguments to catch warnings that only
  manifest during evaluation

The linter exits with status 1 if any warnings are found.

## Adding a New Phrase

To add a new phrase to the RLF system:

1. **Define the phrase** in `strings/src/strings.rlf.rs` inside the `rlf!` macro
   block. Choose the appropriate syntax based on whether it needs parameters,
   pluralization, variant inheritance, or transforms.

2. **Use the generated function** — the `rlf!` macro generates
   `strings::phrase_name()` (or `strings::phrase_name(param)` for parameterized
   phrases) returning `rlf::Phrase`.

3. **Connect to serializers** — in the appropriate serializer module under
   `parser_v2/src/serializer/`, add a match arm that calls the new
   `strings::*()` function and returns the resulting Phrase.

4. **Run `just fmt`** to format the new phrase definition.

5. **Run `just review`** to verify the linter accepts the new phrase and all
   tests pass.

If the phrase introduces a new keyword visible in card rules-text, additional
changes are needed in the parser pipeline (see the parser pipeline
documentation).
