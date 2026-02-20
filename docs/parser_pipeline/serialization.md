# Serialization and RLF Display Text

This document describes how the Dreamtides parser pipeline converts structured
Ability ASTs back into rich display text. The serializer is the inverse of the
parser: where the parser consumes card text to produce an abstract syntax tree,
the serializer walks that tree to reconstruct formatted rules text suitable for
rendering in the Unity client. All display text passes through the RLF (Rust
Localization Framework) phrase system, which handles color coding, plural
agreement, grammatical transforms, and localization concerns.

## Serializer Architecture

The serializer lives in a module directory containing eight files that mirror
the parser's structure module-for-module. Just as the parser has separate files
for abilities, effects, predicates, triggers, costs, conditions, and static
abilities, the serializer has a corresponding file for each. This symmetry is
intentional: every grammar rule the parser recognizes has a matching
serialization path that can reconstruct display text for that same construct.

Five of the eight modules are public: ability_serializer (the entry point),
effect_serializer (the largest module), cost_serializer, predicate_serializer,
and trigger_serializer. The remaining three are private to the serializer
directory: condition_serializer, serializer_utils, and
static_ability_serializer. The serializer_utils module provides shared utility
functions that map comparison operators to phrases like "or less" and "or more,"
convert CardSubtype enum values to their RLF subtype phrases, and convert
FigmentType values to figment phrases.

All eight modules share a common pattern: they accept ability_data AST types as
input and produce rlf::Phrase values as output, delegating to the strings crate
for all text formatting. The serializer never constructs rich text markup
directly. Every color tag, bold marker, and Unicode symbol originates from an
RLF phrase definition.

## Entry Point

The serialize_ability function in ability_serializer.rs dispatches on the five
Ability variants, composing the appropriate trigger, cost, and effect phrases
for each. Each variant has its own assembly logic — triggered abilities compose
trigger phrases with effects, activated abilities join cost phrases with
effects, and event abilities simply capitalize the effect as a sentence. Two
additional public functions handle partial serialization:
serialize_ability_effect (effect only, omitting costs) and
serialize_modal_choices (per-mode text for modal cards).

## Effect Serialization

The effect_serializer module handles the five Effect enum variants (plain,
WithOptions, List, ListWithOptions, Modal). Each variant has its own composition
logic for wrapping effects with costs, conditions, optional flags, and joining
multiple effects.

The AbilityContext enum controls joining strategy: in Triggered context, effects
are joined with ", then" (e.g., "draw a card, then dissolve an enemy"); in Event
context, each effect becomes a separate capitalized sentence (e.g., "Draw a
card. Dissolve an enemy.").

The serialize_standard_effect function matches on all StandardEffect variants,
delegating sub-components to sibling serializers and calling the corresponding
RLF phrase functions.

## RLF Phrase System

All display text definitions live in a single file wrapped in the rlf!
procedural macro, which generates one public Rust function per definition. RLF
has two definition types: terms (fixed phrases like the dissolve keyword) and
parameterized phrases (like draw_cards_effect, which takes a card count).

Phrases compose through nesting, joining, and variant propagation. Key Phrase
methods include join (concatenating with separators), map_text (transforming
text while preserving metadata), and the @cap transform (capitalizing the first
visible character, skipping rich text tags). The :from modifier propagates
variant information through composition layers, ensuring plural agreement works
across nested phrases.

## Color Conventions

The display text uses four color channels defined as Unity rich text color tags,
applied exclusively through RLF phrase definitions.

- Teal (#00838F) is used for energy symbols and values. The energy phrase wraps
  the numeric value and filled circle symbol in teal.
- Gold (#F57F17) is used for points symbols, points values, and figment token
  names. Figment tokens additionally receive bold and underline formatting.
- Purple (#AA00FF) is used for game keywords: dissolve, banish, reclaim,
  materialize, prevent, kindle, foresee, and discover, along with participial
  forms like dissolved, banished, and reclaimed.
- Green (#2E7D32) is used for character subtypes like Warrior, Agent, and Mage.
  Subtypes additionally receive bold formatting.

Two formatting conventions exist outside the color system. Trigger names
(Materialized, Judgment, Dissolved) are rendered in bold without color, preceded
by a bullet symbol. The fast keyword is rendered in bold without color, preceded
by a lightning bolt Unicode character. All formatting uses Unity-style rich text
tags.

## Plural Agreement and Variants

The RLF variant system handles plural agreement throughout serializer output.
Nouns carry "one" and "other" variants for singular and plural forms. The card
term carries "card" and "cards," the character term carries "character" and
"characters," and each character subtype carries its own forms, with irregular
plurals handled individually (Child/Children, Visionary/Visionaries).

Variant selection uses three mechanisms. The :match directive selects a branch
based on a numeric parameter: the cards phrase uses :match to produce "a card"
when count is 1 and "2 cards" otherwise. Static selection uses the colon
operator with a fixed key ({card:other} always selects the plural). Dynamic
selection uses a variable ({card:$n}) which selects based on the plural category
of the value.

The :from modifier is essential for variant propagation through the composition
chain. When a phrase declares :from($param), its output inherits the tags and
variants of the parameter. The subtype phrase inherits the variants of whichever
subtype term is passed to it, so downstream phrases can select singular or
plural forms even after the subtype has been wrapped in color and bold tags.
Without :from, variant information would be lost at each composition layer.

Two transforms handle grammatical concerns. The @cap transform capitalizes the
first grapheme cluster, skipping rich text tags. The @a transform adds an
indefinite article by reading the :a or :an tag and prepending "a" or "an." The
predicate_with_indefinite_article phrase uses @a to produce "a character" or "an
enemy" for singular predicates, while plural predicates skip the article by
selecting their other variant.

## The Round-Trip Property

The parser and serializer are designed to be exact inverses, verified through
round-trip testing. Two independent paths exist from template text to rendered
display text, and both must produce identical output.

Path A (parse then serialize) runs the template through the full forward
pipeline to produce an Ability AST, then runs the serializer to produce display
text. Path B (direct template rendering) evaluates the original template string
directly through the RLF interpreter with the card's variable bindings.

Both paths produce identical output because both use the same RLF phrase
definitions. The TOML template text contains directive references like
{energy($e)} and {dissolve} that resolve to the same phrases the serializer
calls programmatically. The serializer reconstructs formatting entirely from
scratch by walking the AST and calling the appropriate RLF phrase functions. It
does not preserve or reference the original template text. The original text,
variable bindings, and casing are all discarded during parsing; the serializer
works exclusively from typed AST values.

This reconstruction approach is fundamental to localization. Because the
serializer calls RLF phrases rather than preserving original text, switching the
active locale causes it to produce output in a different language without
changes to the serialization logic. The same AST walk occurs regardless of
locale; only the phrase definitions change.

Extensive individual round-trip tests plus bulk tests covering every card verify
this property. A golden rendered output test captures a baseline of all
serialized abilities and detects unintended changes. A bracket locale leak test
wraps every phrase in square brackets to detect English text that bypasses the
localization system.

## Relationship to the Builder Layer

The serializer is the sole path from an Ability AST to displayable text. Given
an AST, it produces the correct rendering by walking the tree and composing RLF
phrases. It does not use the original template text, variable bindings, or lexer
output. This makes it suitable for localization, since the same AST produces
correct output in any locale.

The builder layer (`parser_builder.rs`) constructs semantic `Ability` values
from the parser AST. The `parser_spans.rs` module defines `SpannedAbility` types
that preserve original-text byte-offset spans from the lexer, used during
building.
