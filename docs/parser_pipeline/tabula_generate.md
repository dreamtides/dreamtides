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

The test card generator loads two typed TOML files: `test-cards.toml`
(deserialized as TestCardsFile) and `test-dreamwell.toml` (deserialized as
TestDreamwellFile). Unlike the ability parser's generic approach, this uses
serde-based typed deserialization through the toml_loader module.

For each entry that has both an `id` and `name` field, the generator emits a
Rust `pub const` declaration. Card names are converted to UPPER_SNAKE_CASE using
the convert_case crate, with non-alphanumeric characters first replaced by
spaces to handle hyphens and apostrophes cleanly. If the card has a rules-text
field, a doc comment containing the rules text (with newlines collapsed to
spaces) is emitted above the constant.

Base cards produce constants of type BaseCardId, while dreamwell cards produce
constants of type DreamwellCardId. A HashSet tracks seen constant names to skip
duplicates. After all individual constants, the generator produces aggregate
constant arrays: ALL_TEST_CARD_IDS and ALL_TEST_DREAMWELL_CARD_IDS. The file
begins with a header comment warning that it is generated and should not be
edited manually.

## Generating card_lists.rs

The card lists generator loads `card-lists.toml` (deserialized as
CardListsFile), where each row specifies a list name, list type (such as
BaseCardId or DreamwellCardId), card UUID, and copy count. The generator groups
entries by their list type and then by list name, preserving insertion order for
both grouping levels.

Output happens in three passes. The first pass emits a `pub const` array for
each named list, containing the card IDs with duplicates expanded according to
each row's copy count. The second pass emits a Rust enum type for each card ID
type (named with a "List" suffix, such as BaseCardIdList), with one variant per
list name converted to PascalCase. Each enum derives Clone, Debug, Serialize,
Deserialize, and JsonSchema. The third pass emits a lookup function for each
card ID type that maps enum variants to their corresponding const arrays via a
match expression.

## The Staleness Check

The staleness check is invoked via `just tabula-check` and implemented in
`tabula_cli/src/commands/check.rs`. It uses a regenerate-and-compare strategy
rather than checksums or timestamps. The check calls the same generate functions
that the generate command uses, producing all three artifacts entirely in
memory, then reads the existing on-disk versions and compares them.

For the two Rust files (test_card.rs and card_lists.rs), the comparison is
byte-for-byte string equality. Any whitespace difference, trailing newline
change, or formatting variation triggers a mismatch.

For parsed_abilities.json, the comparison is structural. Both strings are
deserialized into serde_json::Value and compared as semantic JSON values. Key
ordering and whitespace differences are ignored; only structural or value
differences trigger a mismatch. This avoids false positives from minor
serde_json formatting changes across versions or platforms.

The check does not short-circuit. All three files are compared and all
mismatches are reported before the process exits. When a mismatch is detected,
the print_diff function shows the first differing line with both the actual and
expected content, or reports that the line counts differ. The final output
instructs the developer to run `tabula generate`.

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

At game startup, the Tabula struct's load method (in
`tabula_data/src/tabula.rs`) assembles the complete card database without
performing any parsing. The load method takes a TabulaSource enum (Production or
Test) and a path to the Tabula data directory.

The first step reads parsed_abilities.json via
`ability_parser::load_parsed_abilities`, which deserializes the JSON file into a
BTreeMap mapping Uuid values to vectors of Ability ASTs. On Android, the loader
handles `jar:file:` URLs to read directly from APK archives.

Next, the loader reads the appropriate TOML card definition files based on the
source selection. For production, it loads cards.toml and dreamwell.toml; for
tests, it loads test-cards.toml and test-dreamwell.toml. These TOML files are
deserialized into typed structs via the toml_loader module using serde.

For each raw card definition from TOML, the loader looks up the card's UUID in
the pre-parsed abilities map to retrieve its ability ASTs, then calls the
card_definition_builder to assemble a complete CardDefinition struct combining
the TOML metadata (name, energy cost, card type, subtype, spark, rarity) with
the pre-parsed abilities. The same join-by-UUID process applies to dreamwell
cards, card lists, and card effects.

The Tabula struct provides two loading modes. The strict mode treats any card
build failure as fatal. The lenient mode (used during development) collects card
build failures as warnings and skips affected cards, returning successfully
loaded cards alongside the warning list.

## Watch Mode

A file-watching command provides auto-regeneration during active development.
The watch subcommand (in `tabula_cli/src/commands/watch.rs`) uses the
notify_debouncer_mini crate to monitor the Tabula source directory for changes.

On startup, watch performs an initial full generation to ensure all artifacts
are current. It then enters a loop that listens for file system events with a
200 millisecond debounce window, filtering for changes to `.toml` and `.ftl`
files only. When a relevant change is detected, a dirty flag is set, and on the
next loop iteration the full generate command runs. If regeneration fails, the
error is printed to stderr but the watch continues running.

The watcher monitors the tabula directory non-recursively. A Ctrl+C handler
allows graceful shutdown. The watcher can be started via `just watch-tabula`.

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
