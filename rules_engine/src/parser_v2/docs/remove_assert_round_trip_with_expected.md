# Project Plan: Remove `assert_round_trip_with_expected` from Parser V2 Tests

## Overview

This document outlines the plan to remove all 18 usages of `assert_round_trip_with_expected` by fixing serializers, updating card text, and making parsers reject non-canonical forms.

## Test Failure Analysis

### Category A: Output is Correct, Update Card Text, Reject in Parser

These tests have correct serializer output. The input form should be rejected by the parser.

| # | Test | Input | Output | Fix |
|---|------|-------|--------|-----|
| 1 | `test_round_trip_abandon_an_ally_once_per_turn_reclaim_subtype` | `{Reclaim} a {subtype}.` | `{Reclaim} {a-subtype}.` | Reject `a {subtype}` pattern in parser |
| 3 | `test_round_trip_spend_one_or_more_energy_draw_for_each_energy_spent` | `Spend 1 or more...` | `Pay 1 or more...` | Reject "Spend" in `cost_parser.rs` |
| 6 | `test_round_trip_dissolved_lowercase_subtype_directive_serializes_to_capital` | `{a-subtype} in your void` | `{ASubtype} in your void` | Reject `{a-subtype}` in dissolved subject position |

### Category B: Output is Correct, Update Card Text, Hard to Reject

| # | Test | Input | Output | Notes |
|---|------|-------|--------|-------|
| 2 | `test_round_trip_energy_discard_kindle` | `{kindle}.` | `{Kindle}.` | Case insensitivity makes rejection difficult |

### Category C: Input is Correct, Serializer Wrong - Don't Capitalize After Trigger Cost

The serializer is incorrectly capitalizing keywords that appear after "You may" or trigger costs. These should remain lowercase.

| # | Test | Input (correct) | Output (wrong) |
|---|------|-----------------|----------------|
| 5 | `test_round_trip_when_you_discard_this_character_materialize_it` | `{materialize} it` | `{Materialize} it` |
| 7 | `test_round_trip_materialized_you_may_banish_ally_then_materialize_it` | `{banish}...{materialize}` | `{Banish}...{Materialize}` |
| 9 | `test_round_trip_judgment_you_may_banish_ally_then_materialize_it` | `{banish}...{materialize}` | `{Banish}...{Materialize}` |
| 10 | `test_round_trip_judgment_you_may_discard_dissolve_enemy` | `{dissolve}` | `{Dissolve}` |
| 11 | `test_round_trip_judgment_banish_cards_from_your_void_to_dissolve_enemy_with_cost` | `{banish}...{dissolve}` | `{Banish}...{Dissolve}` |
| 12 | `test_round_trip_judgment_banish_cards_from_opponent_void_to_gain_energy` | `{banish}` | `{Banish}` |
| 14 | `test_round_trip_judgment_pay_to_banish_allies_then_materialize` | `{banish}...{materialize}` | `{Banish}...{Materialize}` |

**Root cause**: The serializer capitalizes effects after keyword triggers (`{Judgment}`, `{Materialized}`), but it should NOT capitalize when the effect follows "You may" or a trigger cost like "pay {e} to".

### Category D: Input is Correct, Serializer Wrong - Compound Effect Joining

| # | Test | Input (correct) | Output (wrong) | Fix |
|---|------|-----------------|----------------|-----|
| 4 | `test_round_trip_once_per_turn_when_you_discard_a_card_gain_energy_and_kindle` | `gain {e} and {kindle}.` | `Gain {e}. {Kindle}.` | Use "and" for effects from same trigger |
| 8 | `test_round_trip_materialized_banish_any_number_of_allies_then_materialize_them` | `{Banish}..., then {materialize} them.` | `{Banish}.... {Materialize} them.` | Special case banish-then-materialize to use ", then" |

### Category E: Serializer Has Specific Bugs

| # | Test | Input | Output | Bug |
|---|------|-------|--------|-----|
| 13 | `test_round_trip_judgment_abandon_to_discover_and_materialize` | `{a-subtype}...cost {e} higher` | `{subtype}...cost {e} or more` | Two bugs: (1) loses `{a-subtype}` article, (2) "higher" becomes "or more" |
| 15 | `test_round_trip_materialized_event_in_void_gains_reclaim` | `gains {reclaim}...this turn.` | `gains {reclaim}...` (no "this turn") | Missing "this turn" for reclaim-until-end-of-turn |

### Category F: Consistency Issue with "then"

| # | Test | Input | Output | Decision |
|---|------|-------|--------|----------|
| 16 | `test_round_trip_judgment_draw_then_discard` | `Draw..., then discard...` | `Draw.... Discard...` | Output is fine, update card text |
| 17 | `test_round_trip_materialized_discard_then_draw` | `Discard..., then draw...` | `Discard.... Draw...` | Output is fine, update card text |
| 18 | `test_round_trip_you_may_return_character_from_void_draw_cards` | `...to your hand. Draw...` | `...to your hand, then draw...` | Inconsistent! Should match #16/#17 |

**Decision**: Standardize on `. ` separation for #16, #17, #18. Fix #18 to NOT insert "then".

---

## Serializer Fixes Required

### Fix 1: Don't Capitalize After "You may" or Trigger Costs

**File**: `effect_serializer.rs` and/or `ability_serializer.rs`

**Problem**: Currently capitalizes effects after keyword triggers unconditionally. Should NOT capitalize when effect follows:
- "You may"
- A trigger cost like "pay {e} to" or "discard a card to"

**Affects tests**: #5, #7, #9, #10, #11, #12, #14

**Location in `ability_serializer.rs:44-48`**:
```rust
// CURRENT (wrong for "You may" cases):
if is_keyword_trigger {
    result.push(' ');
    result.push_str(&serializer_utils::capitalize_first_letter(
        &effect_serializer::serialize_effect(&triggered.effect, &mut variables),
    ));
}
```

The issue is that the effect itself may contain "You may" which should not be followed by capitalization. Need to check if effect starts with "you may" or similar and not capitalize in that case.

### Fix 2: Compound Effect Joining with "and"

**File**: `effect_serializer.rs:898-919`

**Problem**: Effects from the same trigger are joined with `. ` and capitalized. Should use "and".

**Affects tests**: #4

**Current code (wrong)**:
```rust
let effect_str = effects
    .iter()
    .map(|e| {
        serializer_utils::capitalize_first_letter(&serialize_standard_effect(
            &e.effect, bindings,
        ))
    })
    .collect::<Vec<_>>()
    .join(" ");
```

**Fix**:
```rust
let effect_strings: Vec<String> = effects
    .iter()
    .map(|e| {
        serialize_standard_effect(&e.effect, bindings)
            .trim_end_matches('.')
            .to_string()
    })
    .collect();
let effect_str = if effect_strings.len() == 2 {
    format!("{} and {}.", effect_strings[0], effect_strings[1])
} else if effect_strings.len() > 2 {
    let last = effect_strings.last().unwrap();
    let rest = &effect_strings[..effect_strings.len() - 1];
    format!("{}, and {}.", rest.join(", "), last)
} else {
    effect_strings.join("")
};
```

### Fix 3: Special Case Banish-Then-Materialize

**File**: `effect_serializer.rs`

**Problem**: "banish X, then materialize" pattern should serialize back with ", then".

**Affects tests**: #8

**Approach**: In `Effect::List` or `Effect::ListWithOptions`, detect when we have a banish followed by materialize of the same target, and use ", then" joining.

### Fix 4: Preserve `{a-subtype}` Article

**File**: `predicate_serializer.rs`

**Problem**: `{a-subtype}` becomes `{subtype}`, losing the article.

**Affects tests**: #13

**Also in #13**: "higher" becomes "or more" - check `serializer_utils.rs` for `Operator::HigherBy` serialization.

### Fix 5: Preserve "this turn" for Reclaim Until End of Turn

**File**: `effect_serializer.rs`

**Problem**: The "this turn" phrase is dropped for reclaim-until-end-of-turn effects.

**Affects tests**: #15

**Location**: Check `CardsInVoidGainReclaimThisTurn` serialization (lines 1066-1118) - some cases may be missing "this turn".

### Fix 6: Don't Insert "then" Inconsistently

**File**: `effect_serializer.rs`

**Problem**: Test #18 inserts ", then" but tests #16, #17 use `. ` for similar patterns.

**Affects tests**: #18

**Decision**: Use `. ` consistently. Find where ", then" is being inserted for the #18 pattern and change to `. `.

---

## Parser Rejections Required

### Rejection 1: `a {subtype}` Pattern

**File**: Investigate where this is parsed

**Change**: Reject literal "a" followed by `{subtype}` variable. Must use `{a-subtype}`.

### Rejection 2: "Spend" Keyword

**File**: `cost_parser.rs`

**Change**: Remove "spend" as alternative to "pay". Only accept "pay".

### Rejection 3: `{a-subtype}` in Dissolved Subject Position

**File**: Investigate dissolved trigger parsing

**Change**: In `{Dissolved} {a-subtype} in your void...`, the subject should use `{ASubtype}` not `{a-subtype}`.

---

## Card Text Updates Required

After serializer fixes and parser rejections are in place, update card text in `cards.toml`:

1. `a {subtype}` → `{a-subtype}` where applicable
2. `Spend` → `Pay`
3. `{kindle}` → `{Kindle}` at sentence start (if any)
4. `{a-subtype}` → `{ASubtype}` in dissolved subject position
5. `X, then Y` → `X. Y.` for independent effects (#16, #17 patterns)

**Note**: Do NOT read cards.toml directly - it's too large. Use grep to find specific patterns.

---

## Test Infrastructure

### Setup Required
1. Rename `ability_round_trip_tests/` to `round_trip_test_cases/`
2. Create `ability_round_trip_tests.rs` with `mod round_trip_test_cases;`
3. Fix duplicate test name `test_round_trip_dissolve_all_allies_that_are_not_subtype` in `predicate_serialization_round_trip_tests.rs` (lines 43 and 243)

---

## Implementation Order

1. **Fix serializer capitalization** (Fix 1) - affects 7 tests
2. **Fix compound effect joining** (Fix 2) - affects 1 test
3. **Fix banish-then-materialize** (Fix 3) - affects 1 test
4. **Fix {a-subtype} preservation** (Fix 4) - affects 1 test
5. **Fix "this turn" preservation** (Fix 5) - affects 1 test
6. **Fix "then" consistency** (Fix 6) - affects 1 test
7. **Add parser rejections** - affects 3 tests
8. **Update card text** - affects 4 tests
9. **Convert tests to assert_round_trip**
10. **Delete assert_round_trip_with_expected**
11. **Run `just fmt` and `just review`**

---

## File Reference

| File | Purpose |
|------|---------|
| `effect_serializer.rs` | Serializes effects, compound effect joining |
| `ability_serializer.rs` | Serializes abilities, calls capitalize_first_letter |
| `serializer_utils.rs` | Contains capitalize_first_letter, operator serialization |
| `predicate_serializer.rs` | Serializes predicates including {a-subtype} |
| `trigger_serializer.rs` | Serializes triggers like Judgment, Materialized |
| `cost_parser.rs` | Parses costs, contains "spend"/"pay" |
| `parser_helpers.rs` | Contains directive() parser helper |
| `test_helpers.rs` | Contains assert_round_trip functions |

## Command Reference

```bash
# Run all round-trip tests
cargo test --package parser_v2_tests --test ability_round_trip_tests

# Run specific test
cargo test --package parser_v2_tests --test ability_round_trip_tests test_name

# Run parser tests
just parser-test

# Format and validate
just fmt
just review
```
