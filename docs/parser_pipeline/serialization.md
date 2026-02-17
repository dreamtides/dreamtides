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

The serialize_ability function in ability_serializer.rs is the top-level entry
point. It accepts a reference to an Ability enum and dispatches on its five
variants.

For Event abilities, the function serializes the effect and wraps it in a
capitalized sentence. The capitalized_sentence function applies the @cap
transform to uppercase the first visible character, skipping any rich text tags.

For Triggered abilities, the function composes a trigger phrase with an effect
phrase. It checks for optional modifiers (once per turn, until end of turn) and
selects among four assembly phrases depending on whether the trigger is a
keyword trigger (like Materialized or Judgment) and whether a prefix modifier is
present.

For Activated abilities, the function joins one or more cost phrases with an
effect phrase. Energy costs are separated from non-energy costs using different
separators. The first non-energy cost is capitalized, and subsequent costs
receive distinct formatting. Four assembly phrases cover the fast and
once-per-turn flag combinations.

For Named abilities, currently limited to Reclaim and its cost variants, the
function produces keyword formatting with optional energy cost values. For
Static abilities, it delegates to the static_ability_serializer and wraps the
result in a capitalized sentence.

Two additional public functions exist alongside serialize_ability. The
serialize_ability_effect function serializes only the effect portion of an
ability, omitting costs. The serialize_modal_choices function extracts each
modal option from a list of abilities and serializes them individually,
returning a map from choice index to serialized text.

## Effect Serialization

The effect_serializer module is the heart of the serializer. Its central
function is serialize_effect_with_context, which handles the five Effect enum
variants.

For a plain Effect containing a single StandardEffect, the function serializes
the standard effect and wraps it with a trailing period via the
effect_with_period phrase.

For a WithOptions effect, the function applies layered wrappers in order. It
serializes the inner standard effect, optionally wraps it with a cost connector
(like "pay 2 energy to draw a card"), optionally prepends "you may," adds a
trailing period, and finally prepends any condition clause.

For a List of effects, joining behavior depends on multiple factors. If all
effects are optional with trigger costs, they are joined with "and" and wrapped
in an optional cost body. If the context is Triggered, mandatory effects are
joined with ", then" for natural triggered ability phrasing. In Event context,
effects become separate capitalized sentences joined with period-space
separators.

For a ListWithOptions effect, each individual effect may carry its own
condition, cost, or optional flag. Per-effect wrappers are applied first, then
effects are joined with "and" (shared trigger cost) or ", then" (no shared
cost). Any shared trigger cost, optional flag, or condition wraps the joined
result as an outer layer.

For a Modal effect, the function produces a "Choose One:" header followed by
bulleted lines, each with an energy cost and serialized effect text.

The AbilityContext enum controls the joining strategy with two variants:
Triggered and Event. In Triggered context, multiple mandatory effects use ",
then" as a joiner, producing text like "draw a card, then dissolve an enemy." In
Event context (the default), each effect becomes its own capitalized sentence,
producing text like "Draw a card. Dissolve an enemy."

The serialize_standard_effect function handles all StandardEffect variants
through a large match expression. Each arm extracts relevant data, delegates
sub-components to sibling serializers (predicates to predicate_serializer, costs
to cost_serializer, triggers to trigger_serializer), and calls the corresponding
RLF phrase function.

## RLF Phrase System

The RLF phrase system is the foundation of all display text. All definitions
live in a single file wrapped in the rlf! procedural macro. The macro generates
one public Rust function per definition, each returning an rlf::Phrase value.

RLF has two definition types. A term has no parameters and produces a fixed
phrase, such as the dissolve keyword or the card noun. A parameterized phrase
accepts arguments, such as draw_cards_effect (which takes a card count) or
dissolve_target (which takes a predicate phrase). Parameterized functions accept
"impl Into Value," so callers can pass integers, strings, or other Phrase
objects.

The Phrase type carries a default display text string, a map of variant keys to
alternative forms (for plural agreement), and a list of tags (grammatical
metadata like :a or :an). The key methods are:

- Phrase::empty returns an empty phrase used as an identity element for
  conditional composition. The build_trigger_prefix function starts with
  Phrase::empty and conditionally replaces it with modifier phrases.
- Phrase::join concatenates multiple phrases with a separator, preserving
  variant keys that appear in all inputs. The effect serializer uses this to
  join effect phrases with ", then" or " and " separators.
- Phrase::map_text transforms the default text while preserving tags and
  variants, used for appending or prepending text such as modifier prefixes.
- capitalized_sentence applies @cap to capitalize the first visible character,
  skipping leading rich text tags.

Serialization typically involves multiple layers of phrase composition.
Producing "dissolve an enemy" involves the predicate serializer creating an
enemy phrase with variants and an :an tag, wrapping it with an indefinite
article for the singular form, combining it with the dissolve keyword through
dissolve_target (which uses :from to propagate variants), adding a trailing
period, and finally capitalizing the sentence. Each layer delegates to the next
without directly manipulating markup.

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

The codebase contains two parallel paths from an Ability AST to displayable
text: the serializer and the builder layer. These serve fundamentally different
purposes.

The serializer performs canonical reconstruction. Given an AST, it produces the
one correct rendering by walking the tree and composing RLF phrases. It does not
use the original template text, variable bindings, or lexer output. This makes
it suitable for localization, since the same AST produces correct output in any
locale. The serializer is the production display path used in the game.

The builder layer performs original-text extraction. Given an AST and the
original LexResult (including the un-lowercased input string and byte-offset
spans), it slices the original authored text into semantically tagged segments
for trigger, cost, and effect. The builder preserves exact original casing,
spacing, and formatting by using spans to recover text that the lexer's
lowercasing discarded.

The builder produces a structured DisplayedAbility type with separate fields for
costs, effects, and triggers. However, it does not support localization, has
fragile assumptions (hardcoded prefix lengths, exactly two modal modes), and is
not currently used in production. It exists as infrastructure for a potential
future UI path that displays authored text with structural annotations rather
than the serializer's canonical reconstruction.
