# Parser Testing Infrastructure

This document describes the testing infrastructure that verifies the correctness
of the Dreamtides card ability parser pipeline. The test suite spans five
distinct verification layers -- round-trip tests, golden output comparison,
static analysis, locale leak detection, and parse-only validation -- all of
which must pass for the build gate to succeed.

## Round-Trip Testing Mechanism

The foundation of parser testing is the dual-path rendered output comparison,
implemented in the `assert_rendered_match` function in `test_helpers.rs`. This
assertion verifies that two independent code paths produce identical display
text for any given card ability.

Path A exercises the full parser-then-serializer pipeline. It takes the raw
ability template text, lexes it into tokens, resolves variable directives into
typed semantic tokens, parses the resolved token stream into an Ability AST
using the Chumsky combinator parser, and then serializes that AST back into
rendered display text through the RLF phrase system.

Path B bypasses the parser and serializer entirely. It takes the same raw
template text and evaluates it directly through the RLF locale system using the
same variable bindings. Since the template text is itself a valid RLF template
(containing phrase references like `{energy($e)}` and `{Dissolve}`), the RLF
engine can render it directly into display text without any intermediate AST.

Both paths must produce byte-identical output. If the parser misinterprets any
part of the ability text, the AST will differ from the original intent, and the
serializer will reconstruct different text than the direct RLF rendering.
Similarly, if the serializer has a bug, its output will diverge from the direct
rendering of the template. This symmetry property means that any defect in
either the parser or the serializer manifests as a concrete string mismatch.

The bulk variant, `assert_rendered_match_for_toml`, follows the same logic but
returns a Result instead of panicking, allowing bulk tests to accumulate all
failures across an entire TOML file and report them together.

## Individual Round-Trip Tests

Many handwritten round-trip tests cover specific ability patterns across all
ability types. These live in the `tests/round_trip_tests/` subdirectory,
organized into files by ability category.

- `event_effect_round_trip_tests.rs` covers one-shot effects such as dissolve,
  banish, prevent, draw, discard, discover, reclaim, gain energy, gain points,
  kindle, foresee, and their compound combinations.
- `triggered_ability_round_trip_tests.rs` covers abilities that fire in response
  to game events, including materialized, dissolved, and judgment phase
  triggers.
- `activated_ability_round_trip_tests.rs` covers abilities with explicit costs
  (energy, abandon, discard, return-to-hand, banish) separated from effects by a
  colon, including the fast keyword prefix and once-per-turn restrictions.
- `static_ability_round_trip_tests.rs` covers passive rule-modifying abilities
  such as cost reductions, spark bonuses, and reclaim granting.
- `judgment_ability_round_trip_tests.rs` covers judgment-phase-specific trigger
  patterns and end-of-turn effects.
- `materialized_ability_round_trip_tests.rs` covers abilities that trigger when
  a character enters or leaves the battlefield.

Each test exercises a specific grammatical pattern the parser must recognize.
The tests serve as living documentation of every supported ability syntax.

## Bulk TOML Round-Trip Tests

Four bulk tests iterate over every card in each of the four TOML data files,
running the dual-path comparison on every ability in the game.

- `cards_toml_round_trip_tests.rs` processes `cards.toml`, the production set.
- `test_cards_toml_round_trip_tests.rs` processes `test-cards.toml`.
- `dreamwell_toml_round_trip_tests.rs` processes `dreamwell.toml`.
- `test_dreamwell_toml_round_trip_tests.rs` processes `test-dreamwell.toml`.

All four follow the same collect-then-report pattern. The test reads the TOML
file, iterates over every card, splits multi-ability rules text on double
newline boundaries, and runs the rendered comparison for each ability block.
Errors are accumulated into a vector rather than causing an immediate panic.
After processing every card, `print_bulk_results` outputs a summary showing
total abilities tested, successes, and failures. Each failure includes the card
name, original rules text, variable bindings, and the specific Path A vs Path B
mismatch. Only after all results are printed does the test panic if errors were
collected. This design ensures a single test run reveals every broken card,
which is essential when changes affect many abilities simultaneously.

## Parse-Only Validation

Parse-only validation tests in `card_toml_validation_tests.rs` verify that every
card can be successfully parsed through the lex, resolve, and parse stages
without exercising the serializer. Three tests cover `cards.toml`,
`test-cards.toml`, and `dreamwell.toml` respectively.

These tests catch parser grammar gaps before the round-trip tests catch
serializer gaps. A card that fails to parse will also fail the round-trip test,
but the parse-only test provides a clearer diagnostic because the failure is
isolated to the parsing stage rather than being conflated with a serialization
mismatch.

## Golden Rendered Output

The golden rendered output test in `golden_rendered_output_tests.rs` provides a
fixture-based snapshot of every rendered ability across all four TOML files. The
test generates rendered output for each ability, sorts entries alphabetically by
card name, and compares against a stored baseline at
`tests/round_trip_tests/fixtures/golden_rendered_output.txt`.

Each line in the golden file follows the format
`CardName|AbilityIndex|RenderedText`, including all Unity rich text formatting
tags. When the golden file does not exist, the test creates it automatically.
When it differs from newly generated output, the test shows a line-by-line diff
of up to 20 differences and fails. To update after intentional changes, the
developer deletes the golden file and reruns the test.

This test catches unintended serializer changes that round-trip tests would not
detect. If both Path A and Path B change in lockstep (due to a shared RLF phrase
modification), the round-trip test still passes. The golden file test catches
such changes because it compares against a fixed historical baseline.

## Serializer Static Analyzer

A custom static analysis tool in `serializer_static_analyzer_tests.rs` scans the
serializer source files for code quality violations that would undermine the
localization architecture. The analyzer checks four categories.

- Banned legacy helpers: functions like `text_phrase`, `make_phrase`,
  `with_article`, and `phrase_plural` that bypass the RLF system.
- Period manipulation patterns: `trim_end_matches('.')`, indicating the effect
  fragment convention has not been followed.
- Hardcoded English string literals: patterns like `"allies".to_string()` that
  embed English directly rather than delegating to RLF phrases.
- English grammar logic: patterns like `starts_with(['a','e','i','o','u'])` that
  implement language-specific code in serializers.

The analyzer uses a ratcheting baseline stored in
`fixtures/serializer_static_analyzer_baseline.toml`. Each violation category has
a maximum allowed count (all currently zero), and the test fails if any category
exceeds its baseline. Baselines can only decrease, never increase. The analyzer
includes self-test logic verifying the detectors work on seeded code and that
allowed patterns (comments, function definitions, `strings::` calls) are not
falsely flagged.

## Locale Tests

Two sets of locale tests verify that serializer output is fully compatible with
the localization system.

The bracket locale leak harness in `bracket_locale_leak_harness_tests.rs`
registers a special `en-x-bracket` locale where every RLF phrase is wrapped in
square brackets. It renders every ability through the parse-then-serialize
pipeline and scans for text not inside brackets, which would indicate English
that bypassed RLF. A ratcheting baseline in
`fixtures/bracket_locale_leak_baseline.toml` tracks the total ability count and
maximum allowed leaks (currently zero). A companion sub-test checks that no
unresolved RLF markers (`{@` or `{$` syntax) appear in serialized output when
running in the English locale. The test also writes a trend artifact to
`target/parser_v2_artifacts/bracket_locale_leak_trend.toml`.

The Russian locale tests in `russian_locale_tests.rs` verify translations end to
end through four sub-tests. A ratcheting translation test compares serialized
Russian output against expected translations in
`fixtures/russian_locale_expected.toml`, ensuring pass count never decreases. A
no-crash test renders all abilities through the Russian locale, catching missing
phrase definitions. A phrase count test verifies Russian loads the same number
of phrases as English. A translation validation gate checks for missing phrases,
orphan phrases, and parameter count mismatches between locales.

## Parser CLI

The parser CLI is a standalone debugging tool at `parser_cli.rs` in the
`parser_v2` crate. It provides direct access to each pipeline stage.

The `parse` subcommand accepts ability text and optional variable bindings. The
`--stage` flag selects the pipeline depth: `lex` shows raw tokens, useful for
verifying directive tokenization; `resolve-variables` shows the typed
ResolvedToken stream after variable resolution; `full` runs the complete parse
and displays the Ability AST. Three output formats are available via `--format`:
`json`, `ron` (default), and `debug`.

The `parse-file` subcommand processes all cards in a TOML file and writes
resolved tokens to JSON. The `parse-abilities` subcommand processes an entire
directory of TOML files, producing `parsed_abilities.json`. The
`verify-abilities` subcommand compares a generated JSON file against a fresh
parse. The `verify` subcommand checks that all cards in a TOML file can be lexed
and resolved, printing per-card results.

Error reporting uses the ariadne crate for rich diagnostics with source spans,
colored labels, and Levenshtein-based "did you mean?" suggestions.

## Infrastructure Details

The test helpers address two practical challenges: parser construction cost and
stack space requirements.

Parser caching uses a `thread_local!` static holding a boxed Chumsky parser. The
`ABILITY_PARSER_CACHE` is constructed once per thread and reused for all parse
operations, avoiding the significant overhead of rebuilding the full combinator
tree for each test case.

Stack space management uses the `stacker` crate for on-demand growth. The
`with_stack` wrapper calls `stacker::maybe_grow` with configured red zone and
growth parameters. When remaining stack drops below the threshold, stacker
allocates a new segment. The `RUST_MIN_STACK` environment variable is set to a
large value for parser tests, complementing the dynamic growth. The deep parser
hierarchy -- nested choice combinators, recursive card predicate parsing, and
multiple effect composition layers -- demands this extra space, especially under
parallel test execution.

The test library lives at `rules_engine/tests/parser_v2_tests/` as a separate
crate with dependencies on `ability_data`, `core_data`, `parser_v2`, `strings`,
`chumsky`, `insta`, `rlf`, `stacker`, and `toml`. Fixture files live in
`tests/round_trip_tests/fixtures/`.

## Build Pipeline Integration

Parser tests are separate from the main test suite due to their stack
requirements. The `just parser-test` command runs all parser_v2 tests with
`RUST_MIN_STACK` configuration. A specific test can be run via
`just parser-test TEST_NAME`. The `just parser-baselines` command runs only the
bracket locale leak harness and golden rendered output tests.

The `just review` command, the full build gate, includes parser tests as part of
its verification. All five test layers must pass for review to succeed.

A critical prerequisite is the `tabula-check` staleness validator, invoked via
`just tabula-check`. It regenerates all build artifacts (parsed_abilities.json,
test_card.rs, card_lists.rs) in memory and compares against on-disk versions. If
any mismatch is found, it fails with a message to run `just tabula-generate`.
This check gates both `just test` and the review pipeline, ensuring parser tests
never run against stale generated files.

The typical development workflow after TOML changes is: run
`just tabula-generate`, run `just fmt`, then run `just review`. A file watching
mode monitors the TOML source directory and auto-regenerates on changes with a
200 ms debounce, eliminating manual regeneration during active development.
