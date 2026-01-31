# Project Plan: Remove `assert_round_trip_with_expected` from Parser V2 Tests

## Overview

This document outlines the plan to remove all 18 usages of `assert_round_trip_with_expected` by fixing serializers, updating card text, and making parsers reject non-canonical forms.

## Recent Changes

### BanishThenMaterialize Standard Effect (Implemented)

The "banish X, then materialize it/them" pattern is now parsed as a **single `BanishThenMaterialize` standard effect** instead of two separate effects. This was implemented in commit `cffbc794`.

**What changed:**
- Added `BanishThenMaterialize { target: Predicate, count: CollectionExpression }` to `StandardEffect` enum
- Added parsers: `banish_then_materialize()`, `banish_collection_then_materialize()`, `banish_up_to_n_then_materialize()`
- Added serializer support in `effect_serializer.rs`

**Tests affected:**
- Test #7, #8, #9, #14 - These now parse as single effects and round-trip correctly
- **Fix 3 is now obsolete** - No longer need special-case serializer logic for banish-then-materialize

**Supported patterns:**
| Pattern | Example | CollectionExpression |
|---------|---------|---------------------|
| Single target | `{banish} an ally, then {materialize} it` | `Exactly(1)` |
| Any number | `{Banish} any number of allies, then {materialize} them` | `AnyNumberOf` |
| Up to N | `{banish} {up-to-n-allies}, then {materialize} {it-or-them}` | `UpTo(n)` |

---

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

The serializer was incorrectly capitalizing keywords that appear after "You may" or trigger costs. These should remain lowercase.

| # | Test | Input (correct) | Output (wrong) | Status |
|---|------|-----------------|----------------|--------|
| ~~5~~ | ~~`test_round_trip_when_you_discard_this_character_materialize_it`~~ | ~~`{materialize} it`~~ | ~~`{Materialize} it`~~ | **FIXED** - Fix 1 |
| ~~7~~ | ~~`test_round_trip_materialized_you_may_banish_ally_then_materialize_it`~~ | ~~`{banish}...{materialize}`~~ | ~~`{Banish}...{Materialize}`~~ | **FIXED** - Now single effect |
| ~~9~~ | ~~`test_round_trip_judgment_you_may_banish_ally_then_materialize_it`~~ | ~~`{banish}...{materialize}`~~ | ~~`{Banish}...{Materialize}`~~ | **FIXED** - Now single effect |
| ~~10~~ | ~~`test_round_trip_judgment_you_may_discard_dissolve_enemy`~~ | ~~`{dissolve}`~~ | ~~`{Dissolve}`~~ | **FIXED** - Fix 1 |
| ~~11~~ | ~~`test_round_trip_judgment_banish_cards_from_your_void_to_dissolve_enemy_with_cost`~~ | ~~`{banish}...{dissolve}`~~ | ~~`{Banish}...{Dissolve}`~~ | **FIXED** - Fix 1 |
| ~~12~~ | ~~`test_round_trip_judgment_banish_cards_from_opponent_void_to_gain_energy`~~ | ~~`{banish}`~~ | ~~`{Banish}`~~ | **FIXED** - Fix 1 |
| ~~14~~ | ~~`test_round_trip_judgment_pay_to_banish_allies_then_materialize`~~ | ~~`{banish}...{materialize}`~~ | ~~`{Banish}...{Materialize}`~~ | **FIXED** - Now single effect |

**Root cause**: The serializer capitalizes effects after keyword triggers (`{Judgment}`, `{Materialized}`), but it should NOT capitalize when the effect follows "You may" or a trigger cost like "pay {e} to". **FIXED** in Fix 1.

### Category D: Input is Correct, Serializer Wrong - Compound Effect Joining

| # | Test | Input (correct) | Output (wrong) | Fix |
|---|------|-----------------|----------------|-----|
| 4 | `test_round_trip_once_per_turn_when_you_discard_a_card_gain_energy_and_kindle` | `gain {e} and {kindle}.` | `Gain {e}. {Kindle}.` | Use "and" for effects from same trigger |
| ~~8~~ | ~~`test_round_trip_materialized_banish_any_number_of_allies_then_materialize_them`~~ | ~~`{Banish}..., then {materialize} them.`~~ | ~~`{Banish}.... {Materialize} them.`~~ | **FIXED** - Now single effect |

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

**Status**: ✅ **IMPLEMENTED**

**Problem**: Serializer was capitalizing effects after keyword triggers unconditionally. Should NOT capitalize when effect follows:
- "You may"
- A trigger cost like "pay {e} to" or "discard a card to"

**Affected tests**: #5, #10, #11, #12 (reduced from 7 due to BanishThenMaterialize fix)

**Solution implemented**:
Added `lowercase_leading_keyword()` helper function to `serializer_utils.rs` that lowercases the first `{Keyword}` in a string. Modified `effect_serializer.rs` to use this function in the `Effect::WithOptions` and `Effect::List` branches when `optional=true` or `trigger_cost=Some`.

**Files modified**:
- `serializer_utils.rs` - Added `lowercase_leading_keyword()` function
- `effect_serializer.rs` - Applied lowercase in WithOptions and List branches
- `triggered_ability_round_trip_tests.rs` - Converted test #5 to `assert_round_trip`
- `judgment_ability_round_trip_tests.rs` - Converted tests #10, #11, #12 to `assert_round_trip`

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

### ~~Fix 3: Special Case Banish-Then-Materialize~~ (COMPLETED)

**Status**: ✅ **Implemented** as `BanishThenMaterialize` standard effect.

This is no longer a serializer fix - it's now handled by parsing "banish X, then materialize Y" as a single `BanishThenMaterialize` effect that serializes correctly.

**Affects tests**: ~~#8~~ → Now passes

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

## Implementation Order

1. ~~**Fix banish-then-materialize** (Fix 3)~~ ✅ **DONE** - Implemented as single effect
2. ~~**Fix serializer capitalization** (Fix 1)~~ ✅ **DONE** - 4 tests now use `assert_round_trip`
3. **Fix compound effect joining** (Fix 2) - affects 1 test
4. **Fix {a-subtype} preservation** (Fix 4) - affects 1 test
5. **Fix "this turn" preservation** (Fix 5) - affects 1 test
6. **Fix "then" consistency** (Fix 6) - affects 1 test
7. **Add parser rejections** - affects 3 tests
8. **Update card text** - affects 4 tests
9. **Convert tests to assert_round_trip**
10. **Delete assert_round_trip_with_expected**
11. **Run `just fmt` and `just review`**

---

## Progress Summary

| Category | Original Count | Fixed | Remaining |
|----------|---------------|-------|-----------|
| A: Update card text, reject in parser | 3 | 0 | 3 |
| B: Hard to reject | 1 | 0 | 1 |
| C: Don't capitalize after trigger cost | 7 | 7 | 0 |
| D: Compound effect joining | 2 | 1 | 1 |
| E: Specific serializer bugs | 2 | 0 | 2 |
| F: "then" consistency | 3 | 0 | 3 |
| **Total** | **18** | **8** | **10** |

---

## Lessons Learned

### Failed Approach: Making All Keywords Lowercase

**What was attempted:**
Changed all action keywords in `effect_serializer.rs` to lowercase (`{dissolve}`, `{banish}`, etc.) with the idea that `ability_serializer.rs` would capitalize them when needed.

**Why it failed:**
1. This broke 263 existing round-trip tests that expected capitalized keywords
2. The existing tests use `assert_round_trip` which expects input == output
3. Changing default serialization behavior has a massive blast radius
4. The fix was supposed to affect only 4 tests but ended up breaking hundreds

**Key insight:**
When fixing serializer bugs, prefer **targeted fixes** that only change behavior for the specific problematic cases. Don't change the default behavior that all other tests depend on.

**Correct approach for Fix 1:**
Instead of changing `effect_serializer.rs` to output lowercase keywords, modify `ability_serializer.rs` to detect when an effect starts with "you may" and handle that case specially - capitalizing only the "y" in "you" rather than potentially capitalizing a keyword that follows later in the string.

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
| `standard_effect.rs` | Contains BanishThenMaterialize and other effect variants |
| `game_effects_parsers.rs` | Contains banish_then_materialize() and related parsers |

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

# Regenerate tabula after parser changes
just tabula-generate
```
