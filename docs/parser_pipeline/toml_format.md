# TOML Card Data Format

This document describes the TOML file format used to define cards in Dreamtides.
TOML card definitions are the primary input to the
[parser pipeline](pipeline_overview.md), which transforms rules-text fields into
structured ability ASTs. The format covers two card categories -- regular cards
and dreamwell cards -- each with its own schema and field set.

## Card Definition Structure

### Regular Cards

Regular cards are defined as entries in TOML array-of-tables sections.
Production cards use the table name "cards" while test cards use "test-cards".
Each entry contains the following fields.

- **name** -- Display name of the card.
- **id** -- UUID string uniquely identifying the card. Used as the key in the
  pre-parsed abilities JSON and as a compile-time constant in generated Rust
  code.
- **energy-cost** -- Integer energy required to play the card, or "\*" for modal
  cards with no fixed cost.
- **rules-text** -- Ability text using the directive syntax described below.
  Empty string for vanilla cards. Triple-quoted TOML strings for multi-line
  text.
- **variables** -- Key-value bindings that parameterize directives in the rules
  text. Empty string when there are no variables.
- **card-type** -- Either "Character" or "Event". Characters persist on the
  battlefield; events go to the void after resolution.
- **subtype** -- Creature subtype such as "Warrior" or "Ancient". Empty string
  for events and characters with no subtype.
- **is-fast** -- Boolean for fast-speed play (during the opponent's turn). An
  activated ability can be independently marked fast via the rules-text fast
  prefix, even when the card itself has is-fast set to false.
- **spark** -- Integer spark value earning victory points at judgment. Empty
  string for events.
- **image-number** -- Numeric identifier referencing the card's art asset.
- **rarity** -- One of "Common", "Uncommon", "Rare", "Legendary", or "Special".
  Present on production cards; omitted from test cards.
- **prompts** -- Player-facing UI text for targeting choices. Empty string when
  no choices are needed.

Production cards include two additional fields: **art-owned** (boolean tracking
asset licensing) and **card-number** (sequential identifier within the set).

### Dreamwell Cards

Dreamwell cards are energy-producing cards analogous to lands in other card
games. They use a different schema with the following fields.

- **name** -- Display name, same as regular cards.
- **id** -- UUID identifier, same as regular cards.
- **energy-produced** -- Integer specifying how much energy the dreamwell
  provides when played.
- **rules-text** -- Optional ability text using the same directive syntax as
  regular cards. Many dreamwell cards have no abilities and either set this to
  an empty string or omit it entirely.
- **variables** -- Variable bindings, same format as regular cards.
- **phase** -- Integer where 0 indicates a starter dreamwell (available at game
  start) and 1 indicates a normal dreamwell (acquired during play).
- **image-number** -- Art asset reference, same as regular cards.
- **prompts** -- Player-facing choice text, same as regular cards.

Dreamwell cards lack energy-cost, card-type, subtype, spark, is-fast, and
rarity. Production dreamwell entries may omit rules-text, variables, and prompts
when the card has no abilities, while test entries include all fields
explicitly.

## Rules-Text Directive Syntax

The rules-text field uses a directive syntax where game concepts are wrapped in
curly braces. Everything outside curly braces is treated as plain English text.
The following table catalogues all directive patterns used in the system.

| Pattern               | Form                    | Description                                                                                                                            |
| --------------------- | ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| Bare keyword          | {Keyword}               | A game keyword with no variable data, such as {Dissolve}, {Prevent}, {Reclaim}, {Banish}, {Materialize}                                |
| Bare symbol           | {energy_symbol}         | A display-only icon reference carrying no variable                                                                                     |
| Keyword with variable | {keyword($var)}         | A phrase name followed by a parenthesized variable reference, such as {energy($e)}, {cards($c)}, {Foresee($f)}                         |
| Multi-argument phrase | {keyword($v1, $v2)}     | A compound phrase with two or more variable arguments, such as {n_figments($n, $g)} or {count_allied_subtype($a, $t)}                  |
| Article transform     | {@a keyword($var)}      | Prepends an indefinite article ("a" or "an") to the phrase output                                                                      |
| Plural transform      | {@plural keyword($var)} | Applies pluralization to the phrase output                                                                                             |
| Capitalize transform  | {@cap keyword($var)}    | Capitalizes the first letter of the phrase output                                                                                      |
| Stacked transforms    | {@cap @a keyword($var)} | Multiple transforms composed left to right, such as capitalizing an article-prefixed phrase                                            |
| Selector suffix       | {keyword:$var}          | Links the phrase to a variable for grammatical agreement, such as {card:$c} for singular/plural and {pronoun:$n} for pronoun selection |
| Raw variable          | {$var}                  | Inserts the numeric value of the variable directly into the text                                                                       |
| Compound keyword      | {Keyword1_Keyword2}     | Underscore-joined keywords forming a compound trigger, such as {Materialized_Judgment} or {Materialized_Dissolved}                     |
| Fast prefix           | {Fast} --               | Marks an activated ability as fast-speed; always followed by a double-dash separator and then the cost-colon-effect structure          |
| Modal header          | {choose_one}            | Introduces a modal choice block; appears on its own line                                                                               |
| Modal bullet          | {bullet}                | Prefixes each individual option within a modal choice block                                                                            |

### Capitalization Convention

Directives that begin a sentence or name an ability use an uppercase first
letter in the TOML source, such as {Dissolve}, {Foresee($f)}, or {Materialized}.
When the same keyword appears inline within a sentence (often in prompts or
after other words), it uses a lowercase first letter, such as {dissolve},
{materialize}, or {reclaim}. The lexer lowercases all input before tokenization,
so this distinction affects only the display path that preserves original casing
-- the parser treats both forms identically.

### Trigger and Tense Keywords

Trigger keywords appear at the start of a rules-text paragraph and indicate when
an ability fires: {Materialized} (character enters play), {Judgment} (judgment
phase), {Dissolved} (character is destroyed), {Banished} (character is
banished). Compound triggers join two keywords with an underscore, such as
{Materialized_Judgment} (fires on both events).

Some keywords have both present and past-tense forms. Present tense
({materialize}, {dissolve}, {banish}) describes an action being performed; past
tense ({materialized}, {dissolved}, {banished}) describes a trigger condition.
Both are bare keywords with no variable argument.

## Variables Field Format

The variables field contains bindings that provide concrete values for the
dollar-sign-prefixed variable references in directives. The format is one or
more key-value pairs, where each pair is written as a short name, a colon, and a
value. Multiple pairs are separated by newlines (using TOML triple-quoted
strings) or by commas.

### Value Types

Variable values are interpreted in priority order: first as an unsigned integer,
then as a card subtype name (case-insensitive, matching CardSubtype), then as a
figment type name (lowercase: celestial, radiant, halcyon, shadow). An
unrecognized value produces a parse error.

Variable names follow single-letter conventions matching the PHRASES table
defaults (e for energy, c for cards, s for spark, etc.). Modal cards use
numbered suffixes (e1, e2) to distinguish per-mode values. See the PHRASES
tables in parser_substitutions.rs for the complete mapping.

## Multi-Paragraph Abilities

Double newlines within a triple-quoted TOML string create paragraph breaks, and
each paragraph is parsed as a completely independent ability. The parser
pipeline splits the rules text on double newlines before any other processing;
each resulting paragraph goes through lexing, variable resolution, and parsing
as a separate unit. All paragraphs on a card share the same variable bindings.

Multi-paragraph rules text is used in several situations.

- **Event effect plus named ability.** The main effect in the first paragraph
  and a Reclaim keyword in the second, allowing the card to be replayed from the
  void.
- **Alternative cost plus effect.** A static ability providing a cost reduction
  in the first paragraph, with the card's main effect in the second.
- **Multiple abilities on a character.** A triggered ability in one paragraph
  and an activated ability in another, or two separate activated abilities.
- **Sequential effects in one paragraph.** By contrast, multiple effects
  separated by periods or ", then" connectors within the same paragraph are part
  of a single ability and execute sequentially.

If any paragraph of a card fails to parse, all abilities for that card are
discarded.

## Modal Card Conventions

Modal cards allow the player to choose one of several effects when playing the
card. They follow a specific set of formatting conventions.

The energy-cost field is set to the string "\*" rather than an integer,
indicating that the card has no single fixed cost. Each mode specifies its own
energy cost within the rules text.

The rules text begins with {choose_one} on its own line, followed by one line
per mode. Each mode line starts with {bullet} and then provides its own energy
cost and effect. The modes are separated by single newlines (not double
newlines), so the entire modal block remains within a single paragraph and is
parsed as one ability.

Variables for modal cards use numbered suffixes to distinguish per-mode values:
e1 and e2 for energy costs, c1 and c2 for card counts. Variables shared across
modes (or used by only one mode) can use un-suffixed names. The variable
resolution stage maps e1 to Mode1Energy and e2 to Mode2Energy (distinct from the
standard Energy variant), allowing the parser to associate each cost with the
correct mode.

## Prompts Field

The prompts field provides player-facing UI text displayed when the card
requires a targeting choice or other player decision. This text appears in the
game interface as an instruction to the player.

Prompts use the same curly-brace directive syntax as rules text, but with a
different capitalization convention. Where rules text uses uppercase keywords at
the start of sentences ({Dissolve}, {Prevent}), prompts typically use lowercase
inline references ({dissolve}, {prevent}) because the keyword appears
mid-sentence after verbs like "choose" or "select."

Common prompt patterns include target selection ("Choose an enemy character to
{dissolve}."), prevention targeting ("Choose a card to {prevent}."), void
selection ("Choose a card in your void."), and variable-cost instructions ("Pay
one or more energy: Draw cards for each energy spent."). The field is an empty
string when no player choices are needed.

## File Locations

The TOML card data files live in the Unity project at
client/Assets/StreamingAssets/Tabula/. The rules engine accesses these files
through a symlink at rules_engine/tabula that points to the same directory, so
both the Rust and C# codebases read from a single source of truth with no manual
synchronization needed.

The directory contains the following card data files.

- **cards.toml** -- Production card definitions. This file is large and should
  not be read directly; use test card files or the parsed abilities JSON for
  inspection.
- **test-cards.toml** -- Test card definitions used by the integration test
  suite. These cards are designed to exercise specific parser and rules engine
  behaviors.
- **dreamwell.toml** -- Production dreamwell card definitions.
- **test-dreamwell.toml** -- Test dreamwell definitions for the test suite.

The directory also contains supplementary TOML files (card-lists.toml,
card-fx.toml, effect-types.toml, trigger-types.toml, predicate-types.toml,
sheets.toml) that serve the Tabula editor tool and other subsystems but are not
part of the ability parser pipeline's input.

The generated file parsed_abilities.json is written to the same Tabula
directory. It contains pre-parsed ability ASTs for every card and is loaded at
game runtime, avoiding any parsing at startup. The generated Rust files
test_card.rs and card_lists.rs are written to
rules_engine/src/tabula_generated/src/. After modifying any TOML card data file,
"just tabula-generate" must be run to regenerate all derived artifacts. The
staleness check "just tabula-check" verifies generated files match current
sources and blocks test execution if they are out of date.
