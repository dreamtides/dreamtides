# Round-Trip Test Plan

## Overview

This document outlines a comprehensive plan to fix all 31 remaining round-trip test failures, enabling removal of all `#[ignore = "Round-trip mismatch"]` annotations. The goal is `parse(text) -> serialize(ast) == text` for all valid card ability texts.

## Current Status

- **Total round-trip tests:** 224
- **Passing tests:** 193
- **Ignored tests:** 31
- **Pass rate:** 86%

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

## The 31 Failing Tests - Categorized Fixes

### Category A: Effect Joining (7 tests)

**Problem:** Serializer uses wrong join separator for triggered ability effects.

| Test | Input (Correct) | Output (Wrong) | Fix Type |
|------|-----------------|----------------|----------|
| `test_judgment_draw_then_discard` | `", then discard"` | `". Discard"` | Serializer |
| `test_materialized_discard_then_draw` | `", then draw"` | `". Draw"` | Serializer |
| `test_pay_variable_energy_draw_per_energy` | `", then discard"` | `". Discard"` | Serializer |
| `test_judgment_pay_to_kindle_and_banish` | `" and {banish}"` | `", then {banish}"` | Serializer |
| `test_may_return_character_from_void_draw` | `". Draw"` | `", then draw"` | Template |
| `test_judgment_may_discard_to_draw_and_gain` | `"to draw and gain"` | broken parse | Parser |
| `test_return_all_but_one_ally_draw_per_returned` | structural | `"all but one an ally"` | Serializer |

**Root cause:** `effect_serializer.rs:977-997` joins mandatory `Effect::List` with `. ` instead of `, then`.

**Fix for serializer:**
```rust
// In Effect::List else branch (mandatory effects)
// Change from: effect_strings.join(" ")  (with periods on each)
// To: effect_strings.join(", then ")
```

**Fix for `test_may_return_character_from_void_draw`:** Update test template to use `, then`:
```
// From: "You may return a character from your void to your hand. Draw {cards}."
// To:   "You may return a character from your void to your hand, then draw {cards}."
```

---

### Category B: Prevent Adds Spurious "enemy" (4 tests)

**Problem:** Serializer outputs "a played enemy" instead of "a played character/card".

| Test | Input (Correct) | Output (Wrong) |
|------|-----------------|----------------|
| `test_prevent_played_character` | `"a played character"` | `"a played enemy"` |
| `test_prevent_dissolve_event` | `"a played event...an ally"` | `"a played enemy event...ally"` |
| `test_materialized_prevent_played_card_by_cost` | `"a played card"` | `"a played enemy"` |

**Root cause:** `effect_serializer.rs:293-305` uses `predicate_base_text()` which calls `serialize_enemy_predicate()`, adding "enemy".

**Fix location:** `predicate_serializer.rs` - `predicate_base_text()` function.

**Fix approach:** For Counterspell/Prevent context, don't use "enemy" prefix:
```rust
// In predicate_base_text(), for Predicate::Enemy:
// Return "character", "event", or "card" instead of "enemy", "enemy event", etc.
// The "enemy" is implicit for Prevent effects.
```

---

### Category C: Cost/Count Serialization (5 tests)

**Problem:** Serializer converts variable templates to literals or loses predicates.

| Test | Input | Output | Issue |
|------|-------|--------|-------|
| `test_pay_energy_discard_kindle` | `"Discard {discards}"` | `"Discard a card"` | Variable lost |
| `test_abandon_any_allies_draw_per_abandoned` | `"any number of allies"` | `"{count-allies}"` | Wrong directive |
| `test_discard_chosen_from_opponent_hand_by_cost` | `"cost {e} or less"` | predicate lost | Predicate lost |
| `test_materialize_random_characters_from_deck` | `"cost {e} or less"` | predicate lost | Predicate lost |
| `test_play_from_void_by_cost` | `"cost {e} or less"` | predicate lost | Predicate lost |

**Fix 1 - Discard cost (cost_serializer.rs:25-36):**
```rust
// Current: if *count == 1 { format!("discard {}", serialize_predicate(...)) }
// Fix: Always use variable template:
Cost::DiscardCards { target, count } => {
    bindings.insert("discards".to_string(), VariableValue::Integer(*count));
    "discard {discards}".to_string()
}
```

**Fix 2 - "any number of" (cost_serializer.rs:11-24):**
```rust
// Add explicit case for AnyNumberOf:
CollectionExpression::AnyNumberOf => {
    format!("abandon any number of {}",
        predicate_serializer::serialize_predicate_plural(target, bindings))
}
```

**Fix 3 - Cost predicates:** Investigate why cost predicates are lost in `MaterializeFromDeck` and `DiscardFromOpponentHand` serialization.

---

### Category D: Plural Form Lost (4 tests)

**Problem:** Serializer outputs singular where plural is required.

| Test | Input | Output |
|------|-------|--------|
| `test_spark_equals_subtype_count` | `"{plural-subtype}"` | `"{subtype}"` |
| `test_subtype_gains_spark_equal_count` | `"{plural-subtype}"` | `"{subtype}"` |
| `test_judgment_triggers_on_materialize` | `"allies"` | `"ally"` |
| `test_reveal_top_card_play_characters_from_top` | `"characters"` | `"character"` |

**Root cause:** Serializer uses singular form in counting contexts.

**Fix:** In `predicate_serializer.rs`, detect counting contexts ("number of", "each") and use `{plural-subtype}` or plural nouns.

---

### Category E: Variable Binding Issues (5 tests)

**Problem:** Test provides variables that serializer doesn't preserve.

| Test | Missing Variables | Root Cause |
|------|-------------------|------------|
| `test_choose_one_return_or_draw` | `mode1-cost`, `mode2-cost` | Modal cost serialization |
| `test_conditional_cost_if_dissolved` | `e` | Conditional cost not bound |
| `test_materialized_dissolve_with_abandon_cost` | `e` | Unused in text |
| `test_with_allied_subtype_play_from_hand_or_void` | `subtype` | Not serialized |
| `test_play_event_trigger_copy` | `e` | Unused in text |

**Fix approach:**
1. For unused variables: Remove from test input (test data error)
2. For modal costs: Fix modal serializer to preserve `{mode1-cost}`, `{mode2-cost}`
3. For conditional costs: Fix to preserve `{e}` in conditional cost expressions

---

### Category F: Predicate Form Issues (4 tests)

| Test | Input | Output | Fix |
|------|-------|--------|-----|
| `test_banish_non_subtype_enemy` | `"non-{subtype}"` | `"is not {a-subtype}"` | Serializer |
| `test_require_return_ally_to_play` | `"an ally"` | `"a character"` | Serializer |
| `test_conditional_cost_if_discarded` | `"This character"` | `"This event"` | Parser/Serializer |
| `test_dissolve_by_energy_paid` | `"each character"` | `"all characters"` | Template or Serializer |

**Fix for `non-{subtype}`:** Add special case in predicate serializer to output `"non-{subtype}"` instead of `"that is not {a-subtype}"`.

**Fix for `each` vs `all`:** Decide canonical form. If `all` is canonical, update test template. If `each` is canonical, fix serializer.

---

### Category G: Temporal/Structural (2 tests)

| Test | Input | Output | Issue |
|------|-------|--------|-------|
| `test_subtype_in_void_allies_have_spark` | `"If this card is in"` | `"While this card is in"` | Parser collapses keywords |
| `test_materialized_card_gains_reclaim_for_cost` | `"{reclaim-for-cost}"` | `"{reclaim} equal to its cost"` | Directive expansion |

**Fix:** These require either AST changes to distinguish "If" vs "While", or template updates to use canonical form.

---

## Implementation Plan

### Phase 1: Effect Joining Fix (HIGH PRIORITY - 7 tests)

**File:** `effect_serializer.rs:977-997`

**Change:** In the `else` branch for mandatory `Effect::List`, use `, then` joining:
```rust
// Current:
let effect_strings: Vec<String> = effects.iter().map(|e| {
    let s = serialize_standard_effect(&e.effect, bindings);
    format!("{}.", capitalize_first_letter(s.trim_end_matches('.')))
}).collect();
result.push_str(&effect_strings.join(" "));

// Fixed:
let effect_strings: Vec<String> = effects.iter().enumerate().map(|(i, e)| {
    let s = serialize_standard_effect(&e.effect, bindings)
        .trim_end_matches('.').to_string();
    if i == 0 { capitalize_first_letter(&s) } else { s }
}).collect();
result.push_str(&format!("{}.", effect_strings.join(", then ")));
```

**Template update:** `test_may_return_character_from_void_draw` should use `, then` pattern.

**Tests fixed:** 5-6 tests

---

### Phase 2: Prevent/Counterspell Fix (HIGH PRIORITY - 4 tests)

**File:** `predicate_serializer.rs`

**Change:** In `predicate_base_text()`, when handling `Predicate::Enemy` for played card context, return the card type without "enemy":
```rust
Predicate::Enemy(card_predicate) => {
    // For played card contexts, don't add "enemy" - it's implicit
    match card_predicate {
        CardPredicate::Character => "character".to_string(),
        CardPredicate::Card => "card".to_string(),
        CardPredicate::Event => "event".to_string(),
        // ... handle other cases
    }
}
```

**Tests fixed:** 4 tests

---

### Phase 3: Cost Serialization Fix (HIGH PRIORITY - 5 tests)

**File:** `cost_serializer.rs`

**Change 1:** Always use variable template for discard costs:
```rust
Cost::DiscardCards { target, count } => {
    if let Some(var_name) = parser_substitutions::directive_to_integer_variable("discards") {
        bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
    }
    "discard {discards}".to_string()
}
```

**Change 2:** Add `AnyNumberOf` case:
```rust
Cost::AbandonCharactersCount { target, count } => match count {
    CollectionExpression::AnyNumberOf => {
        format!("abandon any number of {}",
            predicate_serializer::serialize_predicate_plural(target, bindings))
    }
    // ... existing cases
}
```

**Change 3:** Investigate cost predicate loss in `MaterializeFromDeck` and related effects.

**Tests fixed:** 3-5 tests

---

### Phase 4: Plural Form Fix (MEDIUM PRIORITY - 4 tests)

**File:** `predicate_serializer.rs`

**Change:** Track context for counting expressions and use plural forms:
- `"the number of allied {plural-subtype}"` - use plural directive
- `"allies"` in counting context - use plural noun

**Tests fixed:** 4 tests

---

### Phase 5: Variable Binding Fix (MEDIUM PRIORITY - 5 tests)

**Files:** Various serializers + test files

**Changes:**
1. Fix modal cost serialization to preserve `{mode1-cost}`, `{mode2-cost}`
2. Remove unused variables from test inputs where appropriate
3. Fix conditional cost serialization

**Tests fixed:** 5 tests

---

### Phase 6: Predicate Form Fix (LOW PRIORITY - 4 tests)

**File:** `predicate_serializer.rs`

**Changes:**
1. Add `"non-{subtype}"` serialization pattern
2. Decide `"each"` vs `"all"` canonical form
3. Fix `"an ally"` vs `"a character"` context

**Tests fixed:** 4 tests

---

### Phase 7: Template Updates (LOW PRIORITY - 2 tests)

**Files:** Test templates

**Changes:**
1. `test_subtype_in_void_allies_have_spark`: Use `"While"` if that's canonical
2. `test_materialized_card_gains_reclaim_for_cost`: Use expanded form if that's canonical

**Tests fixed:** 2 tests

---

## Detailed Test Fixes

### Tests Requiring Serializer Changes

| Test | Category | File to Modify | Change |
|------|----------|----------------|--------|
| `test_judgment_draw_then_discard` | Joining | `effect_serializer.rs` | Use `, then` for mandatory list |
| `test_materialized_discard_then_draw` | Joining | `effect_serializer.rs` | Use `, then` for mandatory list |
| `test_pay_variable_energy_draw_per_energy` | Joining | `effect_serializer.rs` | Use `, then` for mandatory list |
| `test_judgment_pay_to_kindle_and_banish` | Joining | `effect_serializer.rs` | Use ` and ` for shared cost |
| `test_prevent_played_character` | Prevent | `predicate_serializer.rs` | Don't add "enemy" |
| `test_prevent_dissolve_event` | Prevent | `predicate_serializer.rs` | Don't add "enemy" |
| `test_materialized_prevent_played_card_by_cost` | Prevent | `predicate_serializer.rs` | Don't add "enemy" |
| `test_pay_energy_discard_kindle` | Cost | `cost_serializer.rs` | Use variable template |
| `test_abandon_any_allies_draw_per_abandoned` | Cost | `cost_serializer.rs` | Handle AnyNumberOf |
| `test_spark_equals_subtype_count` | Plural | `predicate_serializer.rs` | Use plural directive |
| `test_subtype_gains_spark_equal_count` | Plural | `predicate_serializer.rs` | Use plural directive |
| `test_judgment_triggers_on_materialize` | Plural | `predicate_serializer.rs` | Use plural noun |
| `test_banish_non_subtype_enemy` | Predicate | `predicate_serializer.rs` | Use `non-{subtype}` form |
| `test_return_all_but_one_ally_draw_per_returned` | Predicate | `predicate_serializer.rs` | Fix article insertion |

### Tests Requiring Template Changes

| Test | Current Input | Proposed Change |
|------|---------------|-----------------|
| `test_may_return_character_from_void_draw` | `". Draw {cards}."` | `", then draw {cards}."` |
| `test_dissolve_by_energy_paid` | `"each character"` | `"all characters"` (if canonical) |
| `test_subtype_in_void_allies_have_spark` | `"If this card is in"` | `"While this card is in"` |
| `test_materialized_card_gains_reclaim_for_cost` | `"{reclaim-for-cost}"` | Use expanded form |

### Tests Requiring Test Data Fixes

| Test | Issue | Fix |
|------|-------|-----|
| `test_materialized_dissolve_with_abandon_cost` | Unused `e` variable | Remove from test |
| `test_play_event_trigger_copy` | Unused `e` variable | Remove from test |

### Tests Requiring Investigation

| Test | Issue | Investigation Needed |
|------|-------|---------------------|
| `test_discard_chosen_from_opponent_hand_by_cost` | Cost predicate lost | Check effect serializer |
| `test_materialize_random_characters_from_deck` | Cost predicate lost | Check effect serializer |
| `test_play_from_void_by_cost` | Cost predicate lost | Check static ability serializer |
| `test_conditional_cost_if_discarded` | `"character"` → `"event"` | Check card type context |
| `test_judgment_may_discard_to_draw_and_gain` | Parse structure broken | Check parser for "you may X to Y and Z" |
| `test_choose_one_return_or_draw` | Mode costs not preserved | Check modal serializer |
| `test_with_allied_subtype_play_from_hand_or_void` | Subtype not serialized | Check conditional play serializer |
| `test_reveal_top_card_play_characters_from_top` | Plural lost | Check play permission serializer |
| `test_require_return_ally_to_play` | `"ally"` → `"character"` | Check cost predicate serializer |
| `test_conditional_cost_if_dissolved` | Variable `e` lost | Check conditional cost serializer |

---

## File Reference

| File | Purpose |
|------|---------|
| `effect_serializer.rs` | Effect joining, standard effect serialization |
| `predicate_serializer.rs` | Predicate serialization, plural forms |
| `cost_serializer.rs` | Cost serialization including discard, abandon |
| `ability_serializer.rs` | Top-level ability serialization |
| `serializer_utils.rs` | Helper functions |

---

## Command Reference

```bash
# Run all round-trip tests (including ignored)
cargo test --package parser_v2_tests --test round_trip_tests -- --ignored --nocapture

# Run specific test
cargo test --package parser_v2_tests --test round_trip_tests test_name -- --ignored --nocapture

# Run non-ignored tests only
cargo test --package parser_v2_tests --test round_trip_tests

# Full validation
just review
```

---

## Success Criteria

1. All 31 ignored tests pass
2. No regression in 193 passing tests
3. `just review` passes
4. cards.toml validates correctly
