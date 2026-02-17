# Variable Resolution and ResolvedToken

## Purpose

Variable resolution is the second stage of the card ability parser pipeline,
positioned between lexing and parsing. It transforms a flat stream of lexer
tokens into semantically typed resolved tokens. The lexer produces generic Token
values where directives are opaque strings (the content between curly braces).
Variable resolution inspects each directive, looks it up in one of four phrase
tables, fetches the corresponding value from the card's variable bindings, and
emits a ResolvedToken carrying a concrete typed value. Non-directive tokens pass
through unchanged, wrapped in ResolvedToken::Token.

This stage bridges raw text and structured parsing. By resolving variables
first, the downstream Chumsky parser operates on a clean stream where each token
already carries its semantic meaning, enabling select! macros that directly
destructure variants and extract values without binding lookup logic.

The entry point is resolve_variables, which takes spanned tokens and a
VariableBindings reference. Display-only directives (such as card:$c patterns
used for plural-aware RLF rendering) are filtered out before resolution.

## The Four PHRASES Tables

Four static tables in parser_substitutions.rs map directive names to semantic
meaning. Each entry is a triple of phrase name, default variable name, and
constructor function.

**PHRASES** is the main table for integer-valued concepts like energy, cards,
spark, foresee, kindle, and points. Each maps a phrase name to a default
variable abbreviation and a ResolvedToken constructor that wraps a u32. Short
forms (e, c, s) are also registered for common concepts.

**BARE_PHRASES** lists directives that carry no variable data (choose_one,
energy_symbol, judgment_phase_name). These pass through as Token::Directive
without binding lookup.

**SUBTYPE_PHRASES** maps subtype-related phrase names to default variable "t",
resolving to ResolvedToken::Subtype(CardSubtype). Multiple names (subtype,
a_subtype, etc.) correspond to different RLF display forms but resolve
identically.

**FIGMENT_PHRASES** maps figment-related phrase names to default variable "g",
resolving to ResolvedToken::Figment(FigmentType).

## The ResolvedToken Enum

ResolvedToken variants fall into four categories: pass-through (wrapping an
original lexer Token unchanged), integer-valued (wrapping a u32 for game
quantities like Energy, CardCount, SparkAmount), type-valued (wrapping a
CardSubtype or FigmentType), and compound (bundling a count with a type, like
FigmentCount or SubtypeCount). Modal cards use distinct variants (Mode1Energy,
Mode2Energy) for per-mode values.

Each variant represents a distinct semantic concept rather than a generic
key-value pair, enabling the parser to use select! pattern matches that are
concise and type-safe.

## The Resolution Priority Chain

The resolve_directive function implements a priority chain tried in order until
one succeeds. Early steps handle syntactic preprocessing: stripping dollar
prefixes from bare variable references, stripping RLF transform prefixes (@cap,
@a, @plural), and handling pronoun selection. The middle steps try each phrase
table in order (PHRASES, BARE_PHRASES, SUBTYPE_PHRASES, FIGMENT_PHRASES), then
compound phrases, then numbered variants (e1, e2 for modal energy). Late steps
handle RLF function-call syntax via resolve_rlf_syntax. Unrecognized directives
fall through as Token::Directive for the parser to handle or reject.

Before the chain runs, display-only directives (matching the pattern
card:$variable) are filtered out entirely.

## resolve_rlf_syntax

This function handles the explicit phrase(args) notation found in TOML
directives like `energy($e)` or `n_figments($n, $g)`. It strips transform
prefixes and selector suffixes, extracts the phrase name and arguments, then
tries each phrase table in order. Arguments override the table's default
variable name. As a final fallback, if the phrase name is unrecognized but there
is a single argument, the function tries treating that argument as a phrase
name, handling RLF wrapper functions.

## The Default Variable Convention

Each phrase table entry includes a default variable name -- a short abbreviation
(typically one letter) serving as the conventional binding key. When a directive
appears as a bare phrase name (e.g., "energy" without function-call syntax), the
resolver uses the default variable ("e") to look up the binding. When
function-call syntax provides an explicit argument (e.g., "energy($e)"), the
argument overrides the default.

This convention enables multiple phrases to coexist on one card without
conflict. A card with both energy and cards directives reads from "e" and "c"
respectively, and the TOML variables field simply declares "e: 3, c: 2". A card
with energy, spark, and foresee reads from "e", "s", and "f" with no ambiguity.

When a card needs two instances of the same concept (such as two energy values
for modal modes), numbered variants are used. The variables field declares e1: 3
and e2: 5, and the directives use energy($e1) and energy($e2), resolving to
Mode1Energy(3) and Mode2Energy(5).

## VariableBindings

The VariableBindings struct in parser_bindings.rs wraps a HashMap from String
keys to VariableValue values. It parses the TOML variables string (comma-or-
newline-separated key:value pairs) with auto-detection of value types: each
value is tried as a u32 integer, then as a CardSubtype, then as a FigmentType.
Typed accessors (get_integer, get_subtype, get_figment) enforce type safety at
lookup time.

## Error Handling

When a phrase is found in a table but the required binding is missing,
resolution produces an UnresolvedVariable error. The diagnostic system uses the
ariadne crate for rich error reports with source spans and "did you mean?"
suggestions via Levenshtein distance, covering variable names, phrase names, and
parser vocabulary.
