# TOML Card Definition Format

Reference for the TOML file format used to define cards in Dreamtides. Card
definitions are the primary input to the parser pipeline, which transforms
rules-text fields into structured ability ASTs. The format covers two card
categories -- regular cards and dreamwell cards -- each with its own schema.

## Table of Contents

- [Regular Card Fields](#regular-card-fields)
- [Dreamwell Card Fields](#dreamwell-card-fields)
- [Rules-Text Directive Syntax](#rules-text-directive-syntax)
- [Variables Field Format](#variables-field-format)
- [Multi-Paragraph Abilities](#multi-paragraph-abilities)
- [Modal Card Conventions](#modal-card-conventions)
- [Prompts Field](#prompts-field)
- [File Locations and Tooling](#file-locations-and-tooling)

## Regular Card Fields

Regular cards are defined as entries in TOML array-of-tables sections.
Production cards use the table name `[[cards]]` while test cards use
`[[test-cards]]`. All fields use kebab-case naming. Every field is optional at
the TOML deserialization level; validation happens during the build phase.

- **name** -- Display name of the card.
- **id** -- UUID string uniquely identifying the card. Used as the key in the
  pre-parsed abilities JSON and as a compile-time constant in generated Rust
  code.
- **energy-cost** -- Integer energy required to play the card, or "\*" for modal
  cards with no fixed cost. Empty string or omitted treated as no cost.
- **rules-text** -- Ability text using the directive syntax described below.
  Empty string for vanilla cards. Triple-quoted TOML strings for multi-line
  text.
- **variables** -- Key-value bindings that parameterize directives in the rules
  text. Empty string when there are no variables.
- **card-type** -- Either "Character" or "Event". Characters persist on the
  battlefield; events go to the void after resolution.
- **subtype** -- Character subtype such as "Warrior" or "Ancient". Empty string
  for events and characters with no subtype.
- **is-fast** -- Boolean for fast-speed play (during the opponent's turn).
  Defaults to false. An activated ability can be independently marked fast via
  the rules-text fast prefix, even when the card itself has is-fast set to
  false.
- **spark** -- Integer spark value earning victory points at judgment. Empty
  string or "\*" treated as no spark value. Events typically use empty string.
- **image-number** -- Numeric identifier referencing the card's art asset
  (Shutterstock image ID). Resolves to an asset path under
  Assets/ThirdParty/GameAssets/CardImages/Standard/.
- **rarity** -- One of "Common", "Uncommon", "Rare", "Legendary", or "Special".
  Present on production cards; omitted from test cards.
- **prompts** -- Player-facing UI text for targeting choices. Empty string when
  no choices are needed.

Production cards include two additional fields: **art-owned** (boolean tracking
asset licensing) and **card-number** (sequential identifier within the set).

### Special Deserialization Rules

The spark field uses a custom deserializer that handles multiple input types:
missing or None becomes no value, integers convert directly, empty string
becomes no value, and "\*" becomes no value. This allows TOML authors to use
whichever form is clearest for the card type.

## Dreamwell Card Fields

Dreamwell cards are energy-producing cards analogous to lands in other card
games. They use a different schema with the table name `[[dreamwell]]` for
production or `[[test-dreamwell]]` for tests.

- **name** -- Display name, same as regular cards.
- **id** -- UUID identifier, same as regular cards.
- **energy-produced** -- Integer specifying how much energy the dreamwell
  provides when drawn. Required for dreamwell cards.
- **rules-text** -- Optional ability text using the same directive syntax as
  regular cards. Many dreamwell cards have no abilities and either set this to
  an empty string or omit it entirely.
- **variables** -- Variable bindings, same format as regular cards.
- **phase** -- Integer controlling deck ordering. Phase 0 cards are starter
  dreamwells available at game start (typically no abilities, just energy
  production). Phase 1 cards appear later in the shuffled deck and often carry
  bonus effects like Foresee or point gain. Defaults to 0 if omitted.
- **image-number** -- Art asset reference. Dreamwell images resolve to a
  separate asset path under Assets/ThirdParty/GameAssets/CardImages/Dreamwell/.
- **prompts** -- Player-facing choice text, same as regular cards.

Dreamwell cards lack energy-cost, card-type, subtype, spark, is-fast, and
rarity. Production dreamwell entries may omit rules-text, variables, and prompts
when the card has no abilities, while test entries include all fields
explicitly.

## Rules-Text Directive Syntax

The rules-text field uses a directive syntax where game concepts are wrapped in
curly braces. Everything outside curly braces is treated as plain English text.

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

### Phrase Tables

The parser resolves directives through four phrase tables defined in
parser_substitutions.rs. Each table maps a phrase name to a default variable
name and a typed ResolvedToken constructor.

**PHRASES** -- Integer-valued concepts. Maps names like "energy", "cards",
"spark", "foresee", "kindle", "points", "reclaim_for_cost", "copies", "count",
"discards", "maximum_energy", "top_n_cards", "up_to_n_allies", "up_to_n_events",
"text_number", "this_turn_times", and "multiply_by" to their respective
ResolvedToken variants. Default variables follow single-letter conventions (e
for energy, c for cards, s for spark, f for foresee, k for kindle, p for points,
r for reclaim cost, d for discards, n for counts, m for maximum energy, v for
top cards). Short-form aliases "c", "e", and "s" are also supported.

**BARE_PHRASES** -- Keywords requiring no variable: "choose_one",
"energy_symbol", and "judgment_phase_name". These pass through as directive
tokens.

**SUBTYPE_PHRASES** -- Card subtype references: "subtype", "a_subtype",
"asubtype", and "plural_subtype". All default to variable "t" and produce a
Subtype token resolved from the CardSubtype enum.

**FIGMENT_PHRASES** -- Figment type references: "figment" and "figments". Both
default to variable "g" and produce a Figment token resolved from the
FigmentType enum.

**Compound phrases** combine a count with a type: "n_figments" (defaults n and
g, produces FigmentCount with count and type) and "count_allied_subtype"
(defaults a and t, produces SubtypeCount with count and subtype).

## Variables Field Format

The variables field contains bindings that provide concrete values for the
dollar-sign-prefixed variable references in directives. The format is one or
more key-value pairs, where each pair is written as a short name, a colon, and a
value. Multiple pairs are separated by newlines (using TOML triple-quoted
strings) or by commas.

### Value Types

Variable values are interpreted in priority order: first as an unsigned integer,
then as a card subtype name (case-insensitive, matching CardSubtype enum values
such as Warrior, Explorer, Musician, Ancient, Mage, etc.), then as a figment
type name (lowercase: celestial, radiant, halcyon, shadow). An unrecognized
value produces a parse error.

### Naming Conventions

Variable names follow single-letter conventions matching the PHRASES table
defaults: e for energy, c for cards, s for spark, f for foresee, k for kindle, p
for points, r for reclaim cost, d for discards, n for counts, t for subtype, g
for figment type, m for maximum energy, v for top cards, a for ally count.

Modal cards use numbered suffixes to distinguish per-mode values: e1 and e2 for
energy costs, c1 and c2 for card counts. The variable resolution stage maps e1
to Mode1Energy and e2 to Mode2Energy (distinct from the standard Energy
variant), allowing the parser to associate each cost with the correct mode.
Other numbered variants (c1, c2, etc.) use the full variable name for lookup but
produce the same token type as their base name.

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

## File Locations and Tooling

### TOML Source Files

The TOML card data files live in the Unity project at
client/Assets/StreamingAssets/Tabula/. The rules engine accesses these files
through a symlink at rules_engine/tabula that points to the same directory, so
both the Rust and C# codebases read from a single source of truth with no manual
synchronization needed.

- **cards.toml** -- Production card definitions. This file is large and should
  not be read directly; use test card files or the parsed abilities JSON for
  inspection.
- **test-cards.toml** -- Test card definitions used by the integration test
  suite. These cards are designed to exercise specific parser and rules engine
  behaviors.
- **dreamwell.toml** -- Production dreamwell card definitions.
- **test-dreamwell.toml** -- Test dreamwell definitions for the test suite.
- **card-lists.toml** -- Defines named groups of cards for gameplay features
  like starter decks.

The directory also contains supplementary TOML files (card-fx.toml,
effect-types.toml, trigger-types.toml, predicate-types.toml, sheets.toml) that
serve the Tabula editor tool and other subsystems but are not part of the
ability parser pipeline's input.

### Generated Files

The `just tabula-generate` command runs the full parse path offline, producing
three artifacts:

- **parsed_abilities.json** -- Pre-parsed ability ASTs as JSON, written to the
  Tabula directory alongside the TOML source files. Loaded at game runtime
  instead of re-parsing. The JSON maps card UUIDs to arrays of Ability objects.
- **test_card.rs** -- Compile-time BaseCardId and DreamwellCardId constants for
  test cards, with doc comments from card rules text. Written to
  tabula_generated/src/. Also generates ALL_TEST_CARD_IDS and
  ALL_TEST_DREAMWELL_CARD_IDS arrays.
- **card_lists.rs** -- Compile-time card list constants and lookup functions.
  Written to tabula_generated/src/.

### Staleness Check

`just tabula-check` regenerates all artifacts in memory and compares them
against the files on disk. Rust files use byte-for-byte comparison; JSON uses
structural comparison (ignoring formatting). This runs as a prerequisite of
`just test` and `just review`. After modifying any TOML card data file, run
`just tabula-generate` before tests. The staleness check blocks test execution
if files are out of date.

### The TV App

The tv app is a Tauri-based desktop application for viewing and editing TOML
card data in a spreadsheet format. It provides features like sorting, filtering,
derived columns, image rendering, file watching, and cell-level validation. Run
`just tv-dev` to launch it, or `just tv-cards` to open it with the main
cards.toml file. This is the recommended way to inspect and modify the large
cards.toml file rather than reading it directly.

### Key Source Files

- tabula_data/src/card_definition_raw.rs -- Raw field definitions for TOML
  deserialization.
- tabula_data/src/card_definition_builder.rs -- Validation and construction of
  CardDefinition from raw data.
- tabula_data/src/toml_loader.rs -- TOML file loading and deserialization entry
  points.
- tabula_data/src/dreamwell_definition.rs -- Dreamwell-specific data structures.
- tabula_cli/src/commands/generate.rs -- The tabula generate command
  implementation.
- tabula_cli/src/commands/check.rs -- The staleness check implementation.
- parser/src/variables/parser_substitutions.rs -- Phrase tables and directive
  resolution.
