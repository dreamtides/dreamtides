# Round-Trip Test Plan

## Overview

This document outlines a comprehensive plan to fix all round-trip test failures in the parser_v2 codebase, enabling the removal of all `#[ignore = "Round-trip mismatch"]` annotations. The goal is to ensure that `parse(text) -> serialize(ast) == text` for all valid card ability texts.

## Current State

There are approximately **116 failing tests** in the round-trip test suite (in `round_trip_tests/`).

---

## Completed Fixes

### Fix 1: Don't Capitalize After "You may" or Trigger Costs (DONE)

**Problem**: Serializer was capitalizing effects after keyword triggers unconditionally. Should NOT capitalize when effect follows:
- "You may"
- A trigger cost like "pay {e} to" or "discard a card to"

**Solution implemented**: Added `lowercase_leading_keyword()` helper function to `serializer_utils.rs` that lowercases the first `{Keyword}` in a string. Modified `effect_serializer.rs` to use this function in the `Effect::WithOptions` and `Effect::List` branches when `optional=true` or `trigger_cost=Some`.

**Files modified**:
- `serializer_utils.rs` - Added `lowercase_leading_keyword()` function
- `effect_serializer.rs` - Applied lowercase in WithOptions and List branches

### Fix 2: Compound Effect Joining with "and" (DONE)

**Problem**: Effects from the same trigger were joined with `. ` and capitalized. Should use "and".

**Solution implemented**: Modified the `else` branch in `Effect::List` serialization (in `effect_serializer.rs`) to join effects with " and " instead of ". ". Only the first effect is capitalized; subsequent effects remain lowercase.

### Fix 3: BanishThenMaterialize Standard Effect (DONE)

The "banish X, then materialize it/them" pattern is now parsed as a **single `BanishThenMaterialize` standard effect** instead of two separate effects. Implemented in commit `cffbc794`.

**What changed:**
- Added `BanishThenMaterialize { target: Predicate, count: CollectionExpression }` to `StandardEffect` enum
- Added parsers: `banish_then_materialize()`, `banish_collection_then_materialize()`, `banish_up_to_n_then_materialize()`
- Added serializer support in `effect_serializer.rs`

**Supported patterns:**
| Pattern | Example | CollectionExpression |
|---------|---------|---------------------|
| Single target | `{banish} an ally, then {materialize} it` | `Exactly(1)` |
| Any number | `{Banish} any number of allies, then {materialize} them` | `AnyNumberOf` |
| Up to N | `{banish} {up-to-n-allies}, then {materialize} {it-or-them}` | `UpTo(n)` |

---

## Root Cause Analysis

After running the failing tests and analyzing the actual mismatches, I've identified **8 distinct categories** of failures:

### Category 1: Keyword Capitalization (HIGH IMPACT - ~40% of failures)

**Pattern**: Lowercase keyword directives become capitalized.

| Input | Output | Status |
|-------|--------|--------|
| `{kindle}` | `{Kindle}` | SERIALIZER BUG |
| `{reclaim}` | `{Reclaim}` | SERIALIZER BUG |
| `{materialize}` | `{Materialize}` | SERIALIZER BUG |
| `{dissolve}` | `{Dissolve}` | SERIALIZER BUG |
| `{banish}` | `{Banish}` | SERIALIZER BUG |
| `{prevent}` | `{Prevent}` | SERIALIZER BUG |
| `{foresee}` | `{Foresee}` | SERIALIZER BUG |

**Root Cause**: The effect serializers in `effect_serializer.rs` output capitalized keywords unconditionally (e.g., `"{Kindle}."` at line 182, `"{Foresee}."` at line 174). The `ability_serializer.rs` then capitalizes the first letter of the effect, but the keyword is already capitalized.

**Example Test Failures**:
- `test_discard_trigger_kindle`: `"When you discard a card, {kindle}."` → `"When you discard a your card, {Kindle}."`
- `test_play_count_trigger_reclaim_self`: `"{reclaim} this character."` → `"{Reclaim} this character."`
- `test_discard_self_trigger_materialize`: `"{materialize} it."` → `"{Materialize} it."`

**Files Involved**:
- `rules_engine/src/parser_v2/src/serializer/effect_serializer.rs:168-183` (Foresee, Kindle)
- All `StandardEffect` serialization that uses keywords

**Fix Strategy**:
1. Output lowercase keywords in `serialize_standard_effect()` for effects that start with keywords
2. Let `ability_serializer.rs` handle capitalization based on position in sentence
3. This requires careful coordination with Fix 1 (already implemented) which lowercases keywords after "you may"

**Decision Point**: Should the canonical form be lowercase (`{kindle}`) or uppercase (`{Kindle}`)?
- **Recommendation**: Lowercase is canonical in input; serializer should match input form based on context.

---

### Category 2: Predicate Ownership/Scope Errors (HIGH IMPACT - ~25% of failures)

**Pattern**: Generic predicates get explicit ownership added incorrectly.

| Input | Output | Issue |
|-------|--------|-------|
| `a card` | `a your card` | Adds incorrect "your" |
| `a character` | `an ally` | Canonicalizes to "ally" |
| `a played card` | `a played enemy card` | Adds incorrect "enemy" |

**Root Cause**: The parser parses "a card" as `Predicate::Your(CardPredicate::Card)` but the serializer outputs "a your card" instead of "a card". Similarly, "a character" parses to `Predicate::Ally(CardPredicate::Character)` which serializes as "an ally".

**Example Test Failures**:
- `test_discard_trigger_gain_points`: `"When you discard a card, gain {points}."` → `"When you discard a your card, gain {points}."`
- `test_materialize_character_trigger_gain_spark`: `"a character"` → `"an ally"`
- `test_prevent_played_card`: `"a played card"` → `"a played enemy card"`

**Files Involved**:
- `rules_engine/src/parser_v2/src/serializer/predicate_serializer.rs:8-40` (`serialize_predicate()`)
- `rules_engine/src/parser_v2/src/parser/predicate_parser.rs` (parsing logic)

**Fix Strategy**:
1. **Option A (Serializer Fix)**: Add special cases in `serialize_predicate()` to omit "your" for generic cards:
   - `Predicate::Your(CardPredicate::Card)` → `"a card"` (not `"a your card"`)
   - `Predicate::Your(CardPredicate::Character)` → `"a character"` (not `"a your character"`)

2. **Option B (Parser Fix)**: Parse "a card" to a new `Predicate::AnyCard` variant that serializes correctly

3. **Option C (Both)**: Add `Predicate::Generic(CardPredicate)` variant for unqualified predicates

**Recommendation**: Option A is simplest. The serializer should recognize that `Your(Card)` in the context of "a card" should serialize without "your".

---

### Category 3: Operator Serialization (MEDIUM IMPACT - ~10% of failures)

**Pattern**: Missing or extra operators in cost/spark comparisons.

| Input | Output | Issue |
|-------|--------|-------|
| `cost {e}` | `cost {e} exactly` | Adds unnecessary "exactly" |
| `cost {e} higher` | `cost {e} or more` | Wrong operator |

**Example Test Failures**:
- `test_discover_card_by_cost`: `"a card with cost {e}."` → `"a card with cost {e} exactly."`

**Root Cause**: The `Operator::Exactly` variant serializes to `"{value} exactly"` but the parser accepts both `"cost {e}"` and `"cost {e} exactly"` as `Exactly`. The serializer always outputs the explicit form.

**Files Involved**:
- `rules_engine/src/parser_v2/src/serializer/serializer_utils.rs:22-30` (`serialize_operator()`)
- `rules_engine/src/parser_v2/src/parser/predicate_suffix_parser.rs`

**Fix Strategy**:
1. Make `Operator::Exactly` serialize to just `"{value}"` without "exactly"
2. Or add a flag to distinguish "implicit exactly" from "explicit exactly"

**Recommendation**: Treat bare `cost {e}` as the canonical form. Modify `serialize_operator()` to not append "exactly".

---

### Category 4: Variable Binding Preservation (MEDIUM IMPACT - ~8% of failures)

**Pattern**: Input variables not present in serialized output.

**Example Test Failures**:
- `test_abandon_allies_count_reclaim_self`: Input has `allies: 2` but serialized output doesn't bind `allies`
- `test_judgment_draw_one`: Input has `e: 3, cards: 1` but output only has `cards: 1`

**Root Cause**: The test input specifies variables that aren't actually used in the ability text. For example, `test_judgment_draw_one` passes `e: 3` but the ability text `"{Judgment} Draw {cards}."` doesn't use `{e}`.

**Files Involved**:
- Test files themselves (incorrect test data)
- `rules_engine/tests/parser_v2_tests/src/test_helpers.rs:107-112`

**Fix Strategy**:
1. **Fix test data**: Remove unused variables from test inputs
2. **Stricter validation**: Error if input specifies variables not used in text

**Recommendation**: Fix test data. The serializer correctly only outputs variables that are used.

---

### Category 5: Effect Joining and Punctuation (MEDIUM IMPACT - ~7% of failures)

**Pattern**: Effects joined incorrectly or with wrong punctuation.

| Input | Output | Issue |
|-------|--------|-------|
| `X. Put it Y.` | `X and put it Y.` | Uses "and" instead of ". " |
| `X, then Y.` | `X. Y.` | Uses ". " instead of ", then" |

**Example Test Failures**:
- `test_prevent_played_card_put_on_deck`: `"{Prevent} a played card. Put it on top of..."` → `"{Prevent} a played enemy card and put it on top of..."`

**Root Cause**: The `Effect::List` serialization in `effect_serializer.rs:878-1010` uses different joining strategies based on effect types, but doesn't match the original input form.

**Files Involved**:
- `rules_engine/src/parser_v2/src/serializer/effect_serializer.rs:878-1010`

**Remaining Fix (Fix 6)**: Don't insert ", then" inconsistently. Use `. ` consistently for independent effects.

---

### Category 6: Article and Subtype Directive Issues (LOW IMPACT - ~5% of failures)

**Pattern**: Article directives lose their article form.

| Input | Output | Issue |
|-------|--------|-------|
| `{a-subtype}` | `{subtype}` | Loses "a-" prefix |
| `{ASubtype}` | `{a-subtype}` | Wrong capitalization |

**Files Involved**:
- `rules_engine/src/parser_v2/src/serializer/predicate_serializer.rs:123-201`

**Remaining Fix (Fix 4)**: Preserve `{a-subtype}` article in `predicate_serializer.rs`.

**Also**: "higher" becomes "or more" - check `serializer_utils.rs` for `Operator::HigherBy` serialization.

---

### Category 7: "this turn" Temporal Modifier (LOW IMPACT - ~3% of failures)

**Pattern**: "this turn" phrase dropped from temporal effects.

| Input | Output | Issue |
|-------|--------|-------|
| `gains {reclaim}...this turn.` | `gains {reclaim}...` | Missing "this turn" |

**Files Involved**:
- `rules_engine/src/parser_v2/src/serializer/effect_serializer.rs` (GainsReclaimUntilEndOfTurn)

**Remaining Fix (Fix 5)**: Check `CardsInVoidGainReclaimThisTurn` serialization - some cases may be missing "this turn".

---

### Category 8: Parser Acceptance of Non-Canonical Forms (LOW IMPACT - ~2% of failures)

**Pattern**: Parser accepts multiple input forms that should be rejected.

| Input | Canonical | Status |
|-------|-----------|--------|
| `Spend 1 or more...` | `Pay 1 or more...` | REJECT "Spend" |
| `a {subtype}` | `{a-subtype}` | REJECT literal "a" before variable |
| `{a-subtype}` in dissolved subject | `{ASubtype}` | REJECT lowercase in subject position |

---

## Remaining Serializer Fixes

### Fix 4: Preserve `{a-subtype}` Article

**File**: `predicate_serializer.rs`

**Problem**: `{a-subtype}` becomes `{subtype}`, losing the article.

**Also**: "higher" becomes "or more" - check `serializer_utils.rs` for `Operator::HigherBy` serialization.

### Fix 5: Preserve "this turn" for Reclaim Until End of Turn

**File**: `effect_serializer.rs`

**Problem**: The "this turn" phrase is dropped for reclaim-until-end-of-turn effects.

**Location**: Check `CardsInVoidGainReclaimThisTurn` serialization - some cases may be missing "this turn".

### Fix 6: Don't Insert "then" Inconsistently

**File**: `effect_serializer.rs`

**Problem**: Some patterns insert ", then" but others use `. ` for similar patterns.

**Decision**: Use `. ` consistently. Find where ", then" is being inserted and change to `. `.

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
5. `X, then Y` → `X. Y.` for independent effects

**Note**: Do NOT read cards.toml directly - it's too large. Use grep to find specific patterns.

---

## Implementation Plan

### Phase 1: Fix Predicate Serialization (Highest Impact)

**Files to modify**:
- `rules_engine/src/parser_v2/src/serializer/predicate_serializer.rs`

**Changes**:
1. In `serialize_predicate()`, add special handling for common generic patterns:
   - `Predicate::Your(CardPredicate::Card)` in "discard" context → `"a card"`
   - `Predicate::Your(CardPredicate::Character)` in "materialize" context → `"a character"`
   - `Predicate::Allied(CardPredicate::Character)` → check if original was "a character"

2. Consider adding a `Predicate::Generic(CardPredicate)` variant to distinguish:
   - "a card" (any card) from "a your card" (explicitly your card)
   - "a character" from "an ally"

**Tests to verify**: `test_discard_trigger_kindle`, `test_discard_trigger_gain_points`, `test_materialize_character_trigger_gain_spark`

### Phase 2: Fix Keyword Capitalization

**Files to modify**:
- `rules_engine/src/parser_v2/src/serializer/effect_serializer.rs`
- `rules_engine/src/parser_v2/src/serializer/ability_serializer.rs`

**Changes**:
1. Change `serialize_standard_effect()` to output lowercase keywords:
   - `"{Kindle}."` → `"{kindle}."`
   - `"{Foresee}."` → `"{foresee}."`
   - `"{Dissolve} an enemy."` → `"{dissolve} an enemy."`

2. Modify `serialize_ability()` to capitalize appropriately based on position:
   - Sentence start → capitalize first letter (may capitalize `{k}` to `{K}`)
   - After trigger → lowercase
   - After "you may" → lowercase (already implemented in Fix 1)

**Alternative Approach**: Keep serializer outputting capitalized keywords, but modify the test expectations and card text to use capitalized keywords as canonical. This is a larger blast radius but simpler code change.

**Tests to verify**: `test_play_count_trigger_reclaim_self`, `test_discard_self_trigger_materialize`, `test_discard_trigger_kindle`

### Phase 3: Fix Operator Serialization

**Files to modify**:
- `rules_engine/src/parser_v2/src/serializer/serializer_utils.rs`

**Changes**:
1. Modify `serialize_operator()` for `Operator::Exactly`:
   - Current: `"{value} exactly"`
   - New: `"{value}"` (implicit exactly)

2. Or add `Operator::ImplicitExactly` vs `Operator::ExplicitExactly` distinction in AST

**Tests to verify**: `test_discover_card_by_cost`

### Phase 4: Fix Effect Joining (Fix 6)

**Files to modify**:
- `rules_engine/src/parser_v2/src/serializer/effect_serializer.rs`

**Changes**:
1. In `Effect::List` serialization, standardize joining:
   - Independent effects: `. ` (period space)
   - Related effects on same target: `, then` or `and`

2. Track effect relationships through parsing to preserve original joining style

**Tests to verify**: `test_prevent_played_card_put_on_deck`

### Phase 5: Fix Remaining Serializer Issues

- **Fix 4**: Preserve `{a-subtype}` article in `predicate_serializer.rs`
- **Fix 5**: Preserve "this turn" for reclaim effects in `effect_serializer.rs`

### Phase 6: Add Parser Rejections

- **Rejection 1**: `a {subtype}` → require `{a-subtype}`
- **Rejection 2**: `Spend` → require `Pay`
- **Rejection 3**: `{a-subtype}` in dissolved subject position → require `{ASubtype}`

### Phase 7: Fix Test Data and Card Text

1. Remove unused variables from test inputs
2. Update card text in `cards.toml` where canonical form differs

---

## Decision Points Requiring Clarification

### Decision 1: Canonical Keyword Capitalization

**Question**: Should the canonical form of keyword directives be lowercase (`{kindle}`) or capitalized (`{Kindle}`)?

**Option A**: Lowercase is canonical
- Pro: Matches current test inputs
- Pro: More natural for in-sentence usage
- Con: Requires serializer changes

**Option B**: Capitalized is canonical
- Pro: Matches current serializer output
- Pro: Keywords are proper nouns in game terminology
- Con: Requires updating all card text and tests

**Recommendation**: Choose Option A (lowercase canonical). The serializer should adapt to context.

### Decision 2: Generic vs Explicit Predicates

**Question**: How should `"a card"` be represented in the AST?

**Option A**: Keep `Predicate::Your(CardPredicate::Card)`, fix serializer
- Pro: Smaller AST change
- Con: Loses semantic distinction

**Option B**: Add `Predicate::Generic(CardPredicate)`
- Pro: Preserves semantic meaning
- Con: Larger AST change, needs parser updates

**Recommendation**: Start with Option A; consider Option B if needed for other features.

### Decision 3: Operator Defaults

**Question**: Should `cost {e}` imply "exactly" or should operator always be explicit?

**Option A**: Implicit "exactly" is default
- Pro: Matches card text style
- Con: Ambiguous meaning

**Option B**: Require explicit operators
- Pro: Clear semantics
- Con: Verbose card text

**Recommendation**: Option A - implicit "exactly" matches natural card text.

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

---

## Command Reference

```bash
# Run all round-trip tests (including ignored)
cargo test --package parser_v2_tests --test round_trip_tests -- --ignored

# Run specific test
cargo test --package parser_v2_tests --test round_trip_tests test_name -- --ignored --nocapture

# Run non-ignored tests only
cargo test --package parser_v2_tests --test round_trip_tests

# Run parser tests
just parser-test

# Format and validate
just fmt
just review

# Regenerate tabula after parser changes
just tabula-generate
```

---

## Success Criteria

1. All tests in `round_trip_tests/` pass without `#[ignore]` annotations
2. `just review` passes
3. No regression in existing passing tests
