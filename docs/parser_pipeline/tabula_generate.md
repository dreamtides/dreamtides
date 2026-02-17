# Tabula Generate and Runtime Loading

This document describes the offline code generation pipeline that transforms
TOML card data into pre-compiled artifacts, the staleness verification system
that keeps those artifacts in sync, and the runtime loading path that assembles
card definitions at game startup without any parsing.

## The Generate Command

The generate command is invoked via `just tabula-generate`, which runs
`cargo run -p tabula_cli -- generate`. The implementation lives in
`tabula_cli/src/commands/generate.rs`. It produces three artifacts from the TOML
source files in the Tabula directory.

- `parsed_abilities.json` contains pre-parsed ability ASTs for every card with a
  rules-text field, serialized as JSON. This is the artifact loaded at game
  runtime.
- `test_card.rs` contains compile-time Rust constants mapping test card names to
  their UUID identifiers, enabling tests to reference cards by symbolic name
  rather than raw UUID strings.
- `card_lists.rs` contains compile-time Rust arrays, enum types, and lookup
  functions grouping cards into named lists for deck building and game
  configuration.

The Rust source files are written to `tabula_generated/src/`, while
`parsed_abilities.json` is written alongside the TOML sources in the Tabula
directory itself. The generate function creates the output directory if needed,
writes each artifact sequentially, and prints a confirmation for each file.

## Generating parsed_abilities.json

The generate command delegates ability parsing to
`ability_directory_parser::parse_abilities_from_directory` in the parser_v2
crate. This function scans every `.toml` file in the Tabula directory using raw
`toml::Value` parsing rather than typed deserialization. This generic approach
means the parser does not need to know the TOML table name; it simply iterates
over every array-of-tables section and extracts any entry that has both an `id`
field and a non-empty `rules-text` field.

For each qualifying entry, the full parse pipeline runs. First, the rules text
is split on double newlines into individual ability paragraphs. Each paragraph
is then independently lexed into a token stream, resolved against the card's
variable bindings, and parsed by the Chumsky combinator parser into an Ability
AST. A single parser instance is constructed once and reused for all entries
across all files, avoiding the expense of repeated parser construction.

Error handling during batch parsing is lenient at the card level. If any stage
fails for any paragraph of a card, all abilities for that card are discarded
with a diagnostic printed to stderr. Other cards continue processing, so a
single malformed card does not prevent the rest from being generated.

The output is a BTreeMap mapping card UUID strings to their parsed ability
vectors. The BTreeMap ensures deterministic key ordering in the output. This map
is serialized to pretty-printed JSON via serde_json and written to
`parsed_abilities.json`.

## Generating test_card.rs

The test card generator loads test-cards.toml and test-dreamwell.toml and emits
Rust `pub const` declarations mapping card names (converted to UPPER_SNAKE_CASE)
to their UUID identifiers. Base cards produce BaseCardId constants, dreamwell
cards produce DreamwellCardId constants. Aggregate arrays list all test card
IDs.

## Generating card_lists.rs

The card lists generator loads card-lists.toml and emits const arrays grouping
cards into named lists, enum types for list selection, and lookup functions
mapping enum variants to their corresponding arrays.

## The Staleness Check

The staleness check (`just tabula-check`) uses a regenerate-and-compare
strategy: it calls the same generate functions to produce all three artifacts in
memory, then compares against disk. Rust files use byte-for-byte comparison;
JSON uses structural comparison (ignoring formatting). All mismatches are
reported before exit.

## Build Pipeline Integration

The `tabula-check` recipe is a just dependency of both `just test` and
`just review-core-test`. The staleness check runs before any cargo test
invocation, and if it fails, tests never start. Within `just review`, the check
runs transitively through the review-core-test step. It is not a separately
named step in the scope system and cannot be independently skipped.

The review scope system, configured in `review_scope_config.json`, treats the
`rules_engine/tabula/` directory as a global_full_trigger. Any change to TOML
files in the tabula directory forces a complete review run with all steps
enabled. On success, the just recipe prints "Tabula check passed" and allows
downstream steps to proceed. On failure, it prints the mismatch output and exits
with code 1, blocking whatever recipe depends on it.

## Runtime Loading

At game startup, the Tabula struct's load method assembles the complete card
database without parsing. It deserializes parsed_abilities.json into a map of
UUID to Ability ASTs, reads TOML card definitions (selecting production or test
files based on a TabulaSource enum), and joins them by UUID to produce
CardDefinition structs. Strict mode treats build failures as fatal; lenient mode
collects warnings and skips affected cards.

## Watch Mode

The `just watch-tabula` command monitors the Tabula source directory and
auto-regenerates all artifacts on `.toml` or `.ftl` file changes with a 200ms
debounce.

## File Location Reference

The TOML source files live in `client/Assets/StreamingAssets/Tabula/`. This
directory is shared with the Unity client and contains all card definition
files: cards.toml (the main production card set), test-cards.toml,
dreamwell.toml, test-dreamwell.toml, card-lists.toml, card-fx.toml, and several
smaller type-definition files.

A symlink at `rules_engine/tabula/` points to
`../client/Assets/StreamingAssets/Tabula`, giving the Rust code convenient
access to the TOML sources without duplicating files.

The parsed_abilities.json file is written into the Tabula source directory
alongside the TOML files (not into the generated code directory). This placement
means it is accessible both through the symlink from the rules engine and
directly from the Unity streaming assets path. The file is tracked in git.

The two generated Rust files (test_card.rs and card_lists.rs) are written to
`rules_engine/src/tabula_generated/src/`, which is a Rust crate that other
rules_engine crates depend on at compile time. The tabula_generated crate
provides the generated constants and enums to the rest of the Rust codebase.

The generate command resolves both directories using compile-time
CARGO_MANIFEST_DIR, navigating relative to the tabula_cli crate's location in
the source tree.
