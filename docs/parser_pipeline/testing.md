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

Handwritten round-trip tests cover specific ability patterns across all ability
types. They live in the `tests/round_trip_tests/` subdirectory, organized into
files by ability category (event effects, triggered abilities, activated
abilities, static abilities, judgment abilities, materialized abilities). Each
test exercises a specific grammatical pattern and serves as living documentation
of supported ability syntax.

## Bulk TOML Round-Trip Tests

Four bulk tests iterate over every card in each TOML data file (cards.toml,
test-cards.toml, dreamwell.toml, test-dreamwell.toml), running the dual-path
comparison on every ability. All follow a collect-then-report pattern: errors
are accumulated and reported together after processing all cards, so a single
test run reveals every broken card rather than stopping at the first failure.

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

A custom static analysis tool scans the serializer source files for code quality
violations that would undermine localization: banned legacy helpers, hardcoded
English strings, English grammar logic, and period manipulation patterns. The
analyzer uses a ratcheting baseline (all categories currently at zero allowed
violations) that can only decrease, never increase.

## Locale Tests

Two sets of locale tests verify localization compatibility. The bracket locale
leak harness registers a special locale where every RLF phrase is wrapped in
square brackets, then scans serialized output for text not inside brackets
(indicating English that bypassed RLF). The Russian locale tests verify
end-to-end translations, check for missing or orphan phrases, and ensure
parameter count consistency between locales. Both use ratcheting baselines.

## Parser CLI

The parser CLI at `parser_cli.rs` provides direct access to each pipeline stage
for debugging. The `parse` subcommand accepts ability text with a `--stage` flag
to select pipeline depth (lex, resolve-variables, or full parse) and a
`--format` flag for output format (json, ron, debug). Additional subcommands
handle batch processing and verification of TOML files. Error reporting uses the
ariadne crate for rich diagnostics with "did you mean?" suggestions.

## Infrastructure Details

Parser caching uses a thread_local static holding a boxed Chumsky parser,
constructed once per thread and reused for all parse operations. Stack space
management uses the stacker crate for on-demand growth, complemented by a large
RUST_MIN_STACK environment variable. The test library lives at
`rules_engine/tests/parser_v2_tests/` with fixture files in
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
