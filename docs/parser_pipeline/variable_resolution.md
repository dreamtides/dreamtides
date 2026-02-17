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
meaning.

**PHRASES** is the main table for integer-valued concepts. Each entry is a
triple of phrase name, default variable name, and constructor function that
wraps a u32 into the appropriate ResolvedToken variant.

| Phrase name         | Default variable | ResolvedToken variant |
| ------------------- | ---------------- | --------------------- |
| cards               | c                | CardCount             |
| copies              | n                | Copies                |
| count               | n                | Count                 |
| count_allies        | a                | CountAllies           |
| discards            | d                | DiscardCount          |
| energy              | e                | Energy                |
| foresee             | f                | ForeseeCount          |
| kindle              | k                | KindleAmount          |
| maximum_energy      | m                | MaximumEnergy         |
| multiply_by         | n                | Number                |
| n_random_characters | n                | Number                |
| points              | p                | PointCount            |
| reclaim_for_cost    | r                | ReclaimCost           |
| spark               | s                | SparkAmount           |
| text_number         | n                | TextNumber            |
| this_turn_times     | n                | ThisTurnTimes         |
| top_n_cards         | v                | CardCount             |
| up_to_n_allies      | n                | UpToNAllies           |
| up_to_n_events      | n                | UpToNEvents           |
| c (short-form)      | c                | CardCount             |
| e (short-form)      | e                | Energy                |
| s (short-form)      | s                | SparkAmount           |

**BARE_PHRASES** lists directive names that carry no variable data: choose_one
(modal choice marker), energy_symbol (energy icon in display text), and
judgment_phase_name (the text "Judgment"). These pass through as
ResolvedToken::Token(Token::Directive) without binding lookup.

**SUBTYPE_PHRASES** maps four subtype-related phrase names (subtype, a_subtype,
asubtype, plural_subtype) to default variable "t". All resolve to
ResolvedToken::Subtype(CardSubtype). The different names correspond to display
forms in the RLF system; the resolver does not distinguish between them since
the parser only needs the subtype value.

**FIGMENT_PHRASES** maps two figment-related phrase names (figment, figments) to
default variable "g". Both resolve to ResolvedToken::Figment(FigmentType).

## The ResolvedToken Enum

ResolvedToken has approximately 22 variants in four categories.

**Pass-through.** Token wraps an original lexer Token unchanged, covering words,
punctuation, bare directives, and unrecognized directives.

**Integer-valued.** Eighteen variants each wrap a u32:

- Energy, Mode1Energy, Mode2Energy -- energy costs; modal variants for
  choose-one cards with separate costs bound to e1 and e2
- CardCount -- number of cards for drawing or other quantity effects
- DiscardCount, PointCount, SparkAmount, ForeseeCount, KindleAmount -- amounts
  for their respective game mechanics
- MaximumEnergy -- energy threshold for cost comparisons
- Count, CountAllies -- generic and allied character counts
- UpToNAllies, UpToNEvents -- upper limits on ally or event counts
- Number -- generic number for multiply_by and n_random_characters
- ReclaimCost, ThisTurnTimes, Copies -- reclaim cost, per-turn limits, copies
- TextNumber -- number rendered as a word ("two") rather than a digit

**Type-valued.** Subtype wraps a CardSubtype (Warrior, Mage, Survivor, etc.);
Figment wraps a FigmentType (Celestial, Radiant, Halcyon, Shadow).

**Compound.** FigmentCount holds a u32 count and FigmentType (for "3 Radiant
figments"); SubtypeCount holds a u32 count and CardSubtype (for "2 allied
Warriors").

Each variant represents a distinct semantic concept rather than a generic
key-value pair, enabling the parser to use select! pattern matches that are
concise and type-safe.

## The Resolution Priority Chain

The resolve_directive function implements a 12-step priority chain, tried in
order until one succeeds.

- **Step 1: Dollar-prefix stripping.** Names starting with $ (bare variable
  references like $s) have the prefix stripped, then resolution restarts.
- **Step 2: Transform prefix stripping.** RLF transform prefixes (@cap @a, @cap,
  @a, @plural) are stripped and resolution restarts. These are display-only
  instructions for capitalization, article insertion, or pluralization.
- **Step 3: Pronoun selection.** The pattern pronoun:$n reads variable n; if the
  value is 1, produces Word("it"), otherwise Word("them").
- **Step 4: PHRASES table lookup.** Direct name lookup; if found, reads an
  integer from the default variable binding and applies the constructor.
- **Step 5: BARE_PHRASES lookup.** If found, passes through as
  Token::Directive(name).
- **Step 6: SUBTYPE_PHRASES lookup.** If found, reads CardSubtype from the
  default variable binding.
- **Step 7: FIGMENT_PHRASES lookup.** If found, reads FigmentType from the
  default variable binding.
- **Step 8: Compound n_figments.** Reads figment from "g" and count from "n",
  producing FigmentCount.
- **Step 9: Compound count_allied_subtype.** Reads subtype from "t" and count
  from "a", producing SubtypeCount.
- **Step 10: Numbered variant lookup.** Names ending in digits (like e1, e2) are
  split by split_numbered_suffix. Energy base "e" with suffix "1" maps to
  Mode1Energy, "2" to Mode2Energy. Other bases look up PHRASES and use the full
  numbered name as binding key.
- **Step 11: RLF function call syntax.** Delegates to resolve_rlf_syntax for
  phrase(args) patterns; see next section.
- **Step 12: Fallback pass-through.** Unrecognized directives pass through as
  Token::Directive(name) for the parser to handle or reject.

Before this chain runs, display-only directives are filtered out.
is_display_only_directive matches the pattern card:$variable, which represents
RLF parameterized selections for plural-aware rendering.

## resolve_rlf_syntax

This function handles the explicit phrase(args) notation found in TOML
directives. It returns None if the name lacks function-call syntax, allowing the
caller to fall through to other strategies.

Processing steps:

- Strip transform prefixes (@cap, @a, @plural) from the name
- Strip any colon-separated selector suffix (like :other) used for RLF
  plural/gender selection
- Extract phrase name and argument list from the parenthesized portion; split
  arguments on commas, trim whitespace, strip dollar-sign prefixes

The function then tries phrase tables in order. Subtype phrases resolve the
first argument (or default "t") as a CardSubtype. Figment phrases resolve the
first argument (or default "g") as a FigmentType. The compound n_figments always
reads figment from "g" and uses the first argument (or "n") for count. The
compound count_allied_subtype always reads subtype from "t" and uses the first
argument (or "a") for count.

For integer phrases from the main PHRASES table, the first argument (or the
table's default variable) serves as binding key. Numbered variable names like e1
or e2 within energy phrases produce Mode1Energy or Mode2Energy respectively.

As a final fallback, if the phrase name is unrecognized but there is a single
argument, the function tries treating that argument as a phrase name in all
tables. This handles RLF wrapper functions that name a phrase reference as their
argument.

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
keys to VariableValue values, bridging TOML card data and the resolution system.

**Parsing format.** VariableBindings::parse accepts the raw variables string.
Entries are separated by commas or newlines, each in key: value format with
trimmed whitespace. Empty entries are silently skipped.

**Auto-detection of value types.** Each value is tried as a u32 integer first,
then as a CardSubtype via case-insensitive strum matching, then as a FigmentType
via lowercase name matching (celestial, radiant, halcyon, shadow). If none
succeed, parsing fails with ParseError::InvalidValue.

**Typed accessors.** Four getters enforce type safety: get returns the raw
VariableValue, get_integer returns Some(n) only for Integer(n), get_subtype
returns Some(s) only for Subtype(s), and get_figment returns Some(f) only for
Figment(f). The insert method supports programmatic binding addition, and iter
provides iteration over all entries.

## Error Handling

When a phrase is found in a table but the required binding is missing,
resolution produces an UnresolvedVariable error carrying the variable name and
source span. The diagnostic system in parser_diagnostics.rs uses the ariadne
crate to format rich error reports with red source location highlighting.

The suggestion system in parser_error_suggestions.rs provides "did you mean?"
hints via Levenshtein distance. The suggest_variable function collects all known
variable names from all four tables (via variable_names) and returns candidates
within edit distance 3, sorted by distance, capped at 5 suggestions. When
available, suggestions appear in both the error label ("Did you mean 'e'?") and
a note line ("Available variables include: e, s, c").

A parallel suggest_directive function operates on phrase names for unrecognized
directives that cause parse errors. A third function, suggest_word, covers plain
English parser vocabulary from a static list of approximately 75 known words.

The phrase_names and variable_names iterators aggregate names from all four
tables, the two compound phrase names, and hardcoded variable names (g, n, t, a,
e1, e2), ensuring suggestion matching covers the full vocabulary.

**Key file references:**

- parser_v2/src/variables/parser_substitutions.rs -- ResolvedToken enum, four
  PHRASES tables, resolve_variables, resolve_directive, resolve_rlf_syntax
- parser_v2/src/variables/parser_bindings.rs -- VariableBindings struct,
  parsing, typed accessors
- parser_v2/src/error/parser_error_suggestions.rs -- Levenshtein distance,
  suggest_variable, suggest_directive, suggest_word
- parser_v2/src/error/parser_diagnostics.rs -- ariadne-based error formatting
