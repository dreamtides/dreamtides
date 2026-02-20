# Round-Trip Test Plan

## Overview

This document outlines the round-trip test plan for the serializer RLF
migration. The goal is `parse(text) -> serialize(ast) == text` for all valid
card ability texts. The original plan addressed 31 failing round-trip tests;
all have been resolved.

## Current Status (Migration Complete)

- **Total round-trip tests:** 224
- **Passing tests:** 224
- **Ignored tests:** 0
- **Pass rate:** 100%

**COMPLETED:** All round-trip tests pass. All `#[ignore = "Round-trip mismatch"]`
annotations have been removed.

## Migration Gate Outcomes

All gates listed in `docs/plans/serializer_rlf_migration.md` Section 7 are
green as of the final validation commit `c988fc4e`.

| Gate | Final State | Evidence |
|------|-------------|----------|
| Bracket-locale leak detector | 0 render errors, 0 unbracketed text leaks | `bracket_locale_leak_baseline.toml`: `max_allowed_render_errors = 0`, `max_allowed_unbracketed_text_leaks = 0` |
| Translation validation | Passes; `Locale::validate_translations` wired in `translation_validation_tests.rs` | Task 2d commit `8515f26f` |
| Parity gate | All abilities fully resolved; 0 mismatches, 0 unresolved markers | `parity_gate_tests.rs` asserts zero mismatches across full corpus |
| Serializer static analyzer | All violation categories at 0 | `serializer_static_analyzer_baseline.toml`: all `max_allowed_*` = 0 |
| `just review` | Green (clippy + style + all tests) | Validated in commit `c988fc4e` |
| Golden rendered output | Stable; only additive changes (new Dreamwell + test card entries) | See Golden File Delta Annotations below |
| Round-trip tests | 224/224 passing, 0 ignored | All `#[ignore]` annotations removed |

## Serializer Baseline Artifacts

- Bracket-locale leak baseline fixture: `tests/round_trip_tests/fixtures/bracket_locale_leak_baseline.toml`
- Golden rendered output baseline fixture: `tests/round_trip_tests/fixtures/golden_rendered_output.txt`
- Serializer static analyzer baseline fixture: `tests/round_trip_tests/fixtures/serializer_static_analyzer_baseline.toml`
- Final baseline leak counts from `test_full_card_bracket_locale_leak_detector`:
  - `total_abilities = 278`
  - `max_allowed_render_errors = 0`
  - `max_allowed_unbracketed_text_leaks = 0`
- Leak trend artifact file written by the leak harness test:
  - `target/parser_artifacts/bracket_locale_leak_trend.toml`
- `just review` runs the serializer baseline checks via the `parser-baselines` target.

## Golden File Delta Annotations

The golden rendered output file (`golden_rendered_output.txt`) was established
in commit `3d5d2596` and updated once in commit `f4b9f12e`. The diff between
initial and final state contains **only additive entries** -- no existing card
renderings were modified. All additions fall into two categories:

**Dreamwell card entries (5 new lines):** `Dreamwell Draw Discard`,
`Dreamwell Foresee`, `Dreamwell Gain Energy`, `Dreamwell Gain Points`,
`Dreamwell Mill 3`. These were added when the golden file test was expanded to
cover `dreamwell.toml` and `test-dreamwell.toml` corpus files (Task 1
bracket-locale infrastructure). The rendered text for each matches the expected
card behavior exactly.

**Test card entries (45 new lines):** All `Test *` cards from
`test-cards.toml`. These were added when the golden file test was expanded to
cover test card corpus files. Each renders identically to the corresponding
real card it models. Examples: `Test Dissolve` renders the same as any
dissolve-an-enemy card; `Test Counterspell` matches `Abolish`; modal test
cards match `Break the Sequence`.

**No regressions.** Zero existing golden lines were changed. The serializer
migration was output-preserving for all production card abilities.

## Leak-Trend Narrative

Baseline captured in Task 1 (commit `0015d948`):
- `total_abilities = 278`
- `max_allowed_render_errors = 17` (17 abilities could not be serialized at all)
- `max_allowed_unbracketed_text_leaks = 0` (no English text outside brackets for abilities that did render)

The 17 render errors were caused by serializer panics on abilities that used
code paths not yet migrated to phrase-based assembly. As each migration task
landed (Tasks 2a through 7), render errors decreased monotonically:
- Task 2b (predicate internals): reduced render errors as predicate paths stabilized
- Task 3 (effect arms): eliminated panics in standard effect serialization
- Task 5 (static ability + trigger): eliminated remaining render errors
- Task 6 (ability assembly + resolve_rlf removal): brought render errors to 0

Final state (commit `c988fc4e`):
- `total_abilities = 278`
- `max_allowed_render_errors = 0`
- `max_allowed_unbracketed_text_leaks = 0`

The unbracketed text leak count was always 0 throughout migration because the
bracket-locale mechanism correctly wrapped all phrase output from the start.
The migration work eliminated render errors (serializer panics/failures) rather
than text leaks, confirming that the RLF phrase coverage established in Task 1
was comprehensive from the beginning.

## Serializer Static Analyzer Trend

Initial baseline (commit `eb4f7321`):
- `max_allowed_legacy_helper_violations = 0`
- `max_allowed_resolve_rlf_violations = 4`
- `max_allowed_trim_end_period_violations = 14`
- `max_allowed_hardcoded_english_violations = 4`
- `max_allowed_english_grammar_violations = 0`

Final baseline (commit `e00c2cbd`):
- All categories at 0

The `resolve_rlf` category was removed entirely when `resolve_rlf()` was
deleted (commit `64865c82`). The `trim_end_matches` violations were eliminated
by the periodless-fragment convention (Task 3a). Hardcoded English literals
were replaced with `strings::` phrase calls throughout Tasks 3-7.

---

## Canonical Patterns (Validated from cards.toml)

### Effect Joining Rules

| Pattern | When to Use | Example |
|---------|-------------|---------|
| `, then` | Sequential effects in triggered abilities | `"{Judgment} Draw {cards}, then discard {discards}."` |
| `. ` | Independent effects in event abilities | `"Draw {cards}. The opponent gains {points}."` |
| ` and ` | Multiple effects from same cost/condition | `"Pay {e} to {kindle} and {banish} {cards}..."` |

**Key insight:** Event abilities use periods for independent effects. Triggered abilities use `, then` for sequential effects. Shared-cost effects use ` and `.

### "enemy" Usage

- **Correct:** `"an enemy"` (standalone noun for opponent's characters)
- **Wrong:** `"enemy card"`, `"enemy character"`, `"enemy event"`
- **Implicit:** `{Prevent}` always targets opponent's cards - never add "enemy"

### Count Directives

- `{count-allies}` - References `$allies` variable for specific counts (e.g., "Abandon {count-allies}:")
- `"any number of"` - Player chooses freely, no fixed count (e.g., "Abandon any number of allies:")

### "discard a card" Usage

- **Allowed in triggers:** `"When you discard a card, {kindle}."` (describes game event)
- **Allowed in costs:** `"discard a card to {dissolve}..."` (describes cost)
- **Not allowed in effects:** Always use `"Discard {discards}."` with variable binding

---

## The 31 Failing Tests (RESOLVED)

All 31 originally-failing round-trip tests have been fixed and their
`#[ignore]` annotations removed. The fixes were applied across the RLF
migration tasks (Tasks 1-7 in `serializer_rlf_migration.md`). The categories
below are preserved for historical reference.

### Category A: Effect Joining (7 tests) -- FIXED
Resolved by the periodless-fragment convention (Task 3a) and phrase-based
effect list assembly. `serialize_effect_with_context` now uses
`strings::then_joiner()` and `strings::and_joiner()` for triggered-ability
contexts and `strings::sentence_joiner()` for event contexts.

### Category B: Prevent/Counterspell (4 tests) -- FIXED
Resolved during predicate serializer rewrite (Task 2b). The predicate
serializer now correctly omits "enemy" in counterspell/prevent contexts where
it is implicit.

### Category C: Cost/Count Serialization (5 tests) -- FIXED
Resolved by cost serializer migration (Task 1 of the original plan) and
predicate consumer updates (Task 2c). Variable bindings are now correctly
preserved through phrase calls.

### Category D: Plural Form (4 tests) -- FIXED
Resolved by adding plural RLF phrase definitions (commit `25f73179`) and
eliminating the unsafe `plural_phrase()` pattern (commit `c3f352de`).

### Category E: Variable Binding (5 tests) -- FIXED
Resolved by modal serializer phrase migration and removal of unused test
variables.

### Category F: Predicate Form (4 tests) -- FIXED
Resolved during predicate constraint composition refactor (commit `07a009f9`)
and predicate API consolidation (commit `5ac1dc6c`).

### Category G: Temporal/Structural (2 tests) -- FIXED
Resolved by using canonical forms in the serializer output and updating test
templates to match.

---

## Residual Risks

None identified for the round-trip test infrastructure. All 224 tests pass,
all baselines are at their tightest possible values (zero violations), and the
golden file is stable.

## Deferred Follow-Up Work

1. **Phase 2 Phrase composition:** Serializers currently return `String`;
   migrating to `Phrase` return types enables real i18n. See
   `serializer_rlf_migration.md` Section 2.2.
2. **Translation files:** No `.rlf` translation files for non-English locales.
   Requires Phase 2.
3. **`eval_str` removal from display layer:** After Phase 2, the serializer
   produces fully-rendered text, making `eval_str` redundant.
4. **`VariableBindings` removal:** After Phase 2 test restructuring,
   `VariableBindings` can be eliminated from the serializer return path.
5. **Bracket-locale maintenance:** New serializer code paths must use
   `strings::` phrases to maintain zero leaks. The harness catches violations
   automatically.

---

## File Reference

| File | Purpose |
|------|---------|
| `effect_serializer.rs` | Effect joining, standard effect serialization |
| `predicate_serializer.rs` | Predicate serialization, plural forms |
| `cost_serializer.rs` | Cost serialization including discard, abandon |
| `ability_serializer.rs` | Top-level ability serialization |
| `serializer_utils.rs` | Helper functions |
| `bracket_locale_leak_harness_tests.rs` | Bracket-locale leak detection CI gate |
| `parity_gate_tests.rs` | Parity regression gate |
| `serializer_static_analyzer_tests.rs` | Static analyzer CI gate |
| `golden_rendered_output_tests.rs` | Golden file regression detection |

---

## Command Reference

```bash
# Run all round-trip tests
just parser-test

# Run specific test
just parser-test test_name

# Run serializer baseline checks (bracket-locale + golden file)
just parser-baselines

# Full validation (clippy + style + all tests including baselines)
just review
```

---

## Success Criteria (ALL MET)

1. All 31 originally-ignored tests pass -- DONE
2. No regression in 193 pre-existing passing tests -- DONE (total now 224)
3. `just review` passes -- DONE
4. cards.toml validates correctly -- DONE
5. Bracket-locale leaks at zero -- DONE
6. Serializer static analyzer violations at zero -- DONE
7. Parity gate passes (no unresolved RLF markers) -- DONE
8. Golden rendered output stable -- DONE
