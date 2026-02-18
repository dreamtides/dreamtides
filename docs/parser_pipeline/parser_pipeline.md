# Card Ability Parser Pipeline

The parser pipeline transforms human-authored card text in TOML data files into
structured ability ASTs consumed by the rules engine, and provides an inverse
serialization path from AST back to rich display text for the Unity client.

## Table of Contents

- [Pipeline Overview](#pipeline-overview)
- [TOML Input](#toml-input)
- [Stage 1: Lexing](#stage-1-lexing)
- [Stage 2: Variable Resolution](#stage-2-variable-resolution)
- [Stage 3: Parsing](#stage-3-parsing)
- [Stage 4: Serialization](#stage-4-serialization)
- [Tabula Generate](#tabula-generate)
- [How to Add a New Effect](#how-to-add-a-new-effect)
- [Common Pitfalls](#common-pitfalls)
- [Detail Documents](#detail-documents)

## Pipeline Overview

The pipeline has two directional flows: a forward parse path and a reverse
serialization path, plus an offline code generation step.

**Forward (parse) path:**

- Rules text in a TOML card definition is split on double newlines into
  individual ability paragraphs. Each paragraph becomes one ability.
- Each paragraph is lexed into a flat token stream. The lexer lowercases all
  input, producing six token types (Word, Directive, Period, Comma, Colon,
  Newline). The original un-lowercased text is preserved for display.
- The token stream undergoes variable resolution. Directive tokens (curly-brace
  content) are looked up in four phrase tables and resolved against the card's
  variable bindings, producing semantically typed ResolvedToken values. Display-
  only directives are filtered out.
- The resolved token stream is consumed by a Chumsky combinator parser that
  recognizes five ability types in priority order: triggered, activated, named,
  static, and event (fallback). The parser produces an Ability AST using types
  from the ability_data crate.

**Reverse (serialization) path:**

- The serializer recursively walks an Ability AST, calling RLF phrase functions
  from the strings crate at each node to produce rich display text with colored
  keywords, plural agreement, articles, and capitalization.
- The serializer's module structure mirrors the parser's. Both paths use the
  same RLF phrase definitions, ensuring the round-trip property: parsing then
  serializing produces identical output to direct template rendering.

**Offline generation:**

- The `just tabula-generate` command runs the parse path over every TOML file
  and writes results to parsed_abilities.json. This pre-parsed JSON is loaded at
  game runtime, avoiding any parsing at startup.
- A staleness check (`just tabula-check`) verifies generated files match current
  TOML sources before tests can run.

## TOML Input

Card definitions in TOML files contain a `rules-text` field with directive
syntax and a `variables` field with bindings.

**Directive syntax patterns:**

| Pattern               | Example                     | Purpose                              |
| --------------------- | --------------------------- | ------------------------------------ |
| Bare keyword          | `{Dissolve}`                | Game keyword, no variable            |
| Keyword with variable | `{energy($e)}`              | Parameterized phrase                 |
| Multi-argument        | `{n_figments($n, $g)}`      | Compound concept                     |
| Transform prefix      | `{@a subtype($t)}`          | Article/plural/capitalize            |
| Selector suffix       | `{card:$c}`                 | Grammatical agreement (display-only) |
| Raw variable          | `{$s}`                      | Direct numeric insertion             |
| Compound keyword      | `{Materialized_Judgment}`   | Combined trigger                     |
| Modal                 | `{choose_one}` / `{bullet}` | Choice card structure                |

**Variables field:** Comma-or-newline-separated key:value pairs using
conventional abbreviations (e for energy, c for cards, s for spark, t for
subtype, g for figment). Values are integers, CardSubtype names, or FigmentType
names.

See [toml_card_format.md](../toml_card_format/toml_card_format.md) for the
complete TOML reference.

## Stage 1: Lexing

The lexer (parser_v2/src/lexer/) lowercases all input and scans it into a flat
token stream.

- **Lowercasing:** The entire input is lowercased before scanning. All Word and
  Directive tokens contain lowercase text. The original string is preserved in
  LexResult.original for display use. Since card text is ASCII, byte offsets
  work as indices into both strings.
- **Six token types:** Word (contiguous non-delimiter characters, including
  numbers and hyphens), Directive (content inside curly braces, captured
  verbatim), Period, Comma, Colon, Newline.
- **Directive handling:** Everything between `{` and `}` is captured as a raw
  string. No internal parsing happens at lex time — RLF function-call syntax,
  transform prefixes, and selectors are all resolved in the next stage.
- **Span tracking:** Every token carries a SimpleSpan with byte-offset start and
  end positions for diagnostic reporting.
- **Display-only directives:** Directives matching the pattern card:$variable
  are identified as display-only and filtered during variable resolution.
- **Double newline splitting** happens before the lexer is called, not during
  lexing. Single newlines are preserved as Newline tokens.

## Stage 2: Variable Resolution

Variable resolution (parser_v2/src/variables/) transforms Token streams into
ResolvedToken streams, bridging lexing and parsing.

**The four PHRASES tables** in parser_substitutions.rs map directive names to
(default_variable, ResolvedToken constructor) pairs: PHRASES for integer-valued
concepts, BARE_PHRASES for no-variable directives, SUBTYPE_PHRASES for
CardSubtype values, and FIGMENT_PHRASES for FigmentType values.

**The ResolvedToken enum** has one variant per semantic concept: a Token
pass-through, integer-valued variants for game quantities, type-valued variants
for subtypes and figments, and compound variants bundling a count with a type.

**Resolution priority:** Directives are resolved through a priority chain
covering syntactic preprocessing (prefix/transform stripping), phrase table
lookups, compound phrases, numbered variants (e1/e2), RLF function-call syntax,
and a pass-through fallback.

See [variable_resolution.md](variable_resolution.md) for the complete reference.

## Stage 3: Parsing

The Chumsky parser (parser_v2/src/parser/) consumes ResolvedToken streams and
produces Ability AST nodes.

**Five ability types in priority order:**

1. **Triggered** — begins with "when", "at", "once per turn", or keyword
   directives like materialized/judgment/dissolved. Combines a trigger event
   with an effect.
2. **Activated** — optional fast prefix, one or more costs separated by commas,
   a colon, then an effect. Costs include energy, abandon, discard, return,
   banish.
3. **Named** — currently only Reclaim and ReclaimForCost.
4. **Static** — rule-modification patterns (cost modifications, spark bonuses,
   reclaim granting, play-from-void rules, etc.).
5. **Event** — the fallback. Any text not matching the above is parsed as an
   event effect.

**Effect composition** uses two levels: single_effect_parser dispatches to five
domain-specific sub-parser modules, while effect_or_compound_parser wraps
singles with modal, optional, conditional, or cost-gated structure.

**The five effect sub-parser modules:**

| Module                  | Domain                                                          |
| ----------------------- | --------------------------------------------------------------- |
| card_effect_parsers     | Card movement, draw, discard, energy, points                    |
| spark_effect_parsers    | Kindle, spark gain, spark manipulation                          |
| control_effects_parsers | Gain control, deck manipulation, disable abilities              |
| resource_effect_parsers | Resource multipliers, point manipulation                        |
| game_effects_parsers    | Foresee, discover, dissolve, banish, materialize, prevent, copy |

**Ordering is critical.** Chumsky's choice() commits to the first matching
alternative. More specific patterns must precede less specific ones within each
sub-parser.

**Predicate/targeting** uses a two-layer architecture: Predicate (ownership
scope — This, Enemy, Another, Your, Any, etc.) wrapping CardPredicate (type
constraints — Character, Event, subtype, cost/spark comparisons, etc.).

See [parser_structure.md](parser_structure.md) and
[predicates.md](predicates.md) for complete references.

## Stage 4: Serialization

The serializer (parser_v2/src/serializer/) walks an Ability AST and produces
rich display text through RLF phrase composition.

**Architecture:** Eight files mirror the parser structure. The entry point
serialize_ability dispatches on the Ability variant. The effect serializer
handles all five Effect enum variants (plain, WithOptions, List,
ListWithOptions, Modal).

**RLF phrase system:** The strings crate defines all display phrases using the
rlf! macro, which generates typed functions returning Phrase objects. The
serializer never produces markup directly — it exclusively calls strings::
functions. Phrases compose through nesting, joining, and variant propagation.

**Color conventions:**

| Color           | Hex     | Used for                                       |
| --------------- | ------- | ---------------------------------------------- |
| Teal            | #00838F | Energy symbols and values                      |
| Gold            | #F57F17 | Points, figment tokens                         |
| Purple          | #AA00FF | Game keywords (dissolve, banish, kindle, etc.) |
| Green           | #2E7D32 | Character subtypes (bold)                      |
| Bold (no color) | —       | Trigger names, fast keyword                    |

**Round-trip property:** Both paths (parse-then-serialize and direct template
rendering) use identical RLF phrase definitions, so they produce identical
output. Extensive individual and bulk round-trip tests verify this.

See [serialization.md](serialization.md) for the complete reference.

## Tabula Generate

The `just tabula-generate` command runs the full parse path offline, producing
three artifacts:

- **parsed_abilities.json** — pre-parsed ability ASTs as JSON, loaded at runtime
  instead of re-parsing. Written alongside the TOML source files.
- **test_card.rs** — compile-time BaseCardId/DreamwellCardId constants for test
  cards. Written to tabula_generated/src/.
- **card_lists.rs** — compile-time card list constants and lookup functions.
  Written to tabula_generated/src/.

**Staleness check:** `just tabula-check` regenerates all artifacts in memory and
compares against disk. Rust files use byte-for-byte comparison; JSON uses
structural comparison (ignoring formatting). This runs as a prerequisite of
`just test` and `just review`.

**After any TOML change,** run `just tabula-generate` before tests. The
staleness check will block test execution if files are out of date.

**Watch mode:** `just watch-tabula` auto-regenerates on file changes with 200ms
debounce.

See [tabula_generate.md](tabula_generate.md) for the complete reference.

## How to Add a New Effect

Step-by-step checklist for adding a new game effect to the pipeline. Based on
tracing the Kindle keyword through every stage.

01. **Define TOML card data.** Add cards to test-cards.toml with rules-text
    using the new directive and a variables entry with appropriate bindings.

02. **Add StandardEffect variant.** In ability_data/src/standard_effect.rs, add
    a new variant with appropriate payload fields using domain newtypes (Spark,
    Energy, Points) rather than bare u32.

03. **Add ResolvedToken variant (if needed).** If the effect introduces a new
    semantic concept, add a variant to ResolvedToken in parser_substitutions.rs.
    Skip if reusing an existing concept like energy or cards.

04. **Add PHRASES table entry (if needed).** Add to the appropriate PHRASES
    table with phrase name, default variable name, and constructor function.

05. **Add parser helper (if needed).** In parser_helpers.rs, add a select!-based
    helper to match the new ResolvedToken variant.

06. **Add parser function.** In the appropriate effect sub-parser module, add a
    parser function producing the new StandardEffect variant. Add it to the
    module's choice list at the correct position (specific before general).

07. **Add RLF phrase definitions.** In strings/src/strings.rlf.rs, add keyword
    and effect phrases with color formatting. Add locale entries if needed.

08. **Add serializer arm.** In serializer/effect_serializer.rs, add a match arm
    mapping the new StandardEffect variant to the strings:: function call.

09. **Add target predicate handling.** In
    battle_queries/src/card_ability_queries/target_predicates.rs, add a match
    arm indicating whether the effect requires a targeting prompt.

10. **Add tests.** Write parser tests (parse_ability + assert_ron_snapshot) and
    round-trip tests (assert_rendered_match). Add test cards for bulk coverage.

11. **Regenerate and verify.** Run `just tabula-generate`, then `just fmt`, then
    `just review`.

**Files that typically need changes:** standard_effect.rs,
parser_substitutions.rs, parser_helpers.rs, the appropriate effect sub-parser
module, strings.rlf.rs, effect_serializer.rs, target_predicates.rs, and test
files.

## Common Pitfalls

- **Lexer lowercases everything.** Directive matching must use lowercase forms.
  `{Judgment}` becomes `Directive("judgment")`.

- **Double newlines split abilities.** Each paragraph is parsed independently.
  If any paragraph fails, the entire card is skipped.

- **Display-only directives are filtered.** Directives like `{card:$c}` never
  reach the parser. New display-only patterns need handling in
  is_display_only_directive().

- **Parser ordering matters.** More specific patterns must precede less specific
  in choice() calls. Getting this wrong causes the wrong parser to consume
  tokens.

- **Predicate::Your vs Predicate::Any.** Bare card types parse as Any (either
  player). In trigger contexts, the trigger parser overrides this to Your.

- **Another excludes self, Your includes self.** "Ally" and "another" map to
  Another. Your only appears in trigger contexts.

- **CardPredicate defaults to Character.** When a suffix parser has no explicit
  base type, it defaults to Character.

- **Directive vs word keywords.** Game keywords with special rendering use
  directives (dissolve, banish); plain English words use word matching (draw,
  gain). Mixing them up causes parse failures.

- **Serializer reconstructs, does not preserve.** The serializer builds text
  from scratch using RLF phrases. Original template text is not preserved.

- **Stack requirements.** Deep Chumsky parser hierarchy needs extra stack space.
  Tests use RUST_MIN_STACK and the stacker crate for additional stack.

- **Regenerate after TOML changes.** Always run `just tabula-generate` after
  modifying card data. The staleness check blocks all tests otherwise.

## Detail Documents

- [toml_card_format.md](../toml_card_format/toml_card_format.md): TOML card
  definition format, directive syntax patterns, variables field, modal card
  conventions, dreamwell format, file locations, and tooling. Read when
  authoring or modifying card data.

- [variable_resolution.md](variable_resolution.md): PHRASES tables, the
  ResolvedToken enum, the resolution priority chain, and VariableBindings. Read
  when adding new phrase types or debugging variable resolution.

- [parser_structure.md](parser_structure.md): Chumsky parser architecture,
  ability types, effect sub-parsers, effect composition, and ordering rules.
  Read when adding parser grammar or debugging parse failures.

- [predicates.md](predicates.md): Two-layer Predicate/CardPredicate
  architecture, suffix system, trigger dual-parse pattern, and targeting
  pitfalls. Read when working with card targeting.

- [serialization.md](serialization.md): Serializer architecture, RLF phrase
  system, color conventions, and the round-trip property. Read when modifying
  display text or adding RLF phrases.

- [testing.md](testing.md): Round-trip tests, golden output, static analyzer,
  locale leak detection, parser CLI, and build pipeline integration. Read when
  writing parser tests or debugging test failures.

- [tabula_generate.md](tabula_generate.md): Generate command, staleness check,
  runtime loading, watch mode, and file locations. Read when working with the
  build pipeline or card data loading.
