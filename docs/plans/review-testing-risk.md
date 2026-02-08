# Testing & Migration Risk Review: Serializer RLF Migration

**Reviewer:** Testing/Risk Analyst
**Date:** 2026-02-07
**Plan:** `docs/plans/2026-02-07-serializer-rlf-migration.md`

---

## 1. Round-Trip Test Concrete Walkthrough

### Example: `test_draw_cards` (event_effect_round_trip_tests.rs:187)

```rust
assert_round_trip("Draw {cards($c)}.", "c: 3");
```

**What happens today:**

1. **Parse:** `"Draw {cards($c)}."` is lexed. The lexer sees `{cards($c)}` as an RLF directive. `VariableBindings::parse("c: 3")` produces `{c: Integer(3)}`. `resolve_variables` substitutes `$c` → `3`, creating resolved tokens including `Cards(3)`.
2. **Serialize:** The parser produces `StandardEffect::DrawCards { count: 3 }`. The serializer runs:
   ```rust
   bindings.insert("c".to_string(), VariableValue::Integer(3));
   "draw {cards($c)}.".to_string()
   ```
   Output text: `"draw {cards($c)}."`, bindings: `{c: Integer(3)}`.
3. **Capitalize:** `ability_serializer` calls `capitalize_first_letter("draw {cards($c)}.")` → `"Draw {cards($c)}."` (capitalizes the first ASCII char since the string doesn't start with `{`).
4. **Compare:** `assert_eq!("Draw {cards($c)}.", "Draw {cards($c)}.")` ✅ passes.

**What would happen after plan migration (Task 8):**

The plan says to replace the hardcoded string with an RLF phrase:
```rust
draw_cards_effect($c) = "draw {{cards($c)}}.";
```

When the serializer calls `strings::draw_cards_effect(3).to_string()`:

1. RLF evaluates the phrase template `"draw {{cards($c)}}."` with `$c = 3`.
2. `{{` → literal `{`, `}}` → literal `}`. But **what about `$c` inside the escaped braces?**

**CRITICAL RISK: `$c` inside `{{...}}` escape behavior is ambiguous.**

Per the RLF DESIGN.md escape rules:
- `{{` → literal `{` (anywhere in text)
- `}}` → literal `}`
- `$` is only special inside `{}` expressions

The key question: Does `{{cards($c)}}` get parsed as:
- (a) A literal `{cards($c)}` string (with `$c` left unevaluated because we're inside an escape), or
- (b) Something that errors because `$c` appears in literal context?

The plan acknowledges this on line 388-390: *"So `{{count_allies($a)}}` should produce `{count_allies($a)}` literally — the `$a` inside the escaped braces should NOT be evaluated because it's inside an escape sequence. The `{{` turns off expression parsing until the matching `}}`."*

**However, this is an assumption that must be verified.** The RLF DESIGN.md escape table says `{{` → literal `{` and `}}` → literal `}`. It does NOT explicitly say that `{{...}}` forms an atomic escape block that suppresses inner `$` evaluation. The escape mechanism works character-by-character (`{{` → `{`), not block-by-block. This means the parser may see:

```
"draw {{cards($c)}}."
       ^^ literal {
              ^^  $c is evaluated (it's just in text, not inside {})
                ^^ literal }
```

This would produce: `"draw {cards(3)}."` — **NOT** `"draw {cards($c)}."`.

If the output is `"draw {cards(3)}."` instead of `"draw {cards($c)}."`, the round-trip test will **fail** because the original text was `"Draw {cards($c)}."`.

**Verdict: HIGH RISK.** The plan's core assumption about escaped braces must be tested against the actual RLF library implementation before any migration work begins. Write a unit test:

```rust
#[test]
fn test_rlf_escaped_braces_preserve_dollar_vars() {
    // Define a phrase: test_escape($c) = "draw {{cards($c)}}.";
    // Call with c=3
    // Assert output is "draw {cards($c)}." NOT "draw {cards(3)}."
}
```

If this test fails (output is `"draw {cards(3)}."` instead), the entire migration strategy is broken and a different approach is needed. **This is a blocking prerequisite for the plan.**

### Second Example: `test_pay_energy_draw` (activated_ability_round_trip_tests.rs:106)

```rust
assert_round_trip("{energy($e)}: Draw {cards($c)}.", "e: 3\nc: 1");
```

Today, the serializer produces: `"{energy($e)}: Draw {cards($c)}."` with bindings `{e: 3, c: 1}`.

After migration, the cost serializer (Task 1) produces `energy_cost_value(3).to_string()` which, using the phrase `energy_cost_value($e) = "{{energy($e)}}"`, should produce `"{energy($e)}"` or `"{energy(3)}"`. This then gets combined with the effect serializer output.

**The dollar-sign-inside-escaped-braces problem applies to EVERY phrase in the plan.** This isn't a one-off issue — it affects literally every phrase that uses `{{directive($var)}}` syntax.

### Capitalization Interaction Risk

Current `capitalize_first_letter()` in `serializer_utils.rs` (line 20-31) has keyword-specific logic:
- It detects leading `{keyword}` and capitalizes the keyword name inside braces
- e.g., `"{kindle($k)}."` → `"{Kindle($k)}."`

After migration, the serializer output from `strings::kindle_effect(2).to_string()` would already contain the text. If the escaped-brace approach works and the output is `"{kindle($k)}."`, then `capitalize_first_letter` would still need to capitalize it to `"{Kindle($k)}."`. **This works correctly** because the capitalization logic operates on the template text.

But if the escaped-brace approach fails and the output is `"{kindle(2)}."`, then `capitalize_first_letter` would try to capitalize `"{kindle(2)}."` → `"{Kindle(2)}."`. This is semantically wrong — `{Kindle(2)}` is not a valid parser input (the parser expects `{Kindle($k)}` with a variable reference). **Round-trip would fail.**

---

## 2. Display Rendering During Migration

### Current Display Pipeline

```
serialize_ability() → SerializedAbility { text: "{template}", variables: VariableBindings }
                                                    ↓
              rlf_helper::eval_str(text, variables) → final rendered String
```

Callers:
- `card_rendering.rs:174` `serialize_abilities_text()`: calls `serialize_ability` → `eval_str`
- `card_rendering.rs:92` `ability_token_text()`: calls `serialize_ability_effect` → `eval_str`
- `dreamwell_card_rendering.rs:78` `rules_text()`: calls `serialize_ability` → `eval_str`
- `modal_effect_prompt_rendering.rs:61` `modal_effect_descriptions()`: calls `serialize_modal_choices` → `eval_str`

### What Happens If Serializer Output Changes

**Scenario A: Escaped braces work correctly (output is template text)**

If the migration produces identical template text (e.g., `"draw {cards($c)}."` stays the same), then:
- `eval_str()` receives the same template string as before → same rendered output
- **No display breakage** during incremental migration
- Individual serializers can be migrated independently

**Scenario B: Escaped braces DON'T work (output is `"draw {cards(3)}."` instead)**

Then `eval_str("{cards(3)}.", bindings)` would attempt to evaluate `{cards(3)}` which is a valid RLF phrase call with a numeric literal argument. This would actually produce the same rendered output as `{cards($c)}` with `c=3`, because `cards(3)` evaluates to `"3 cards"` just like `cards($c)` with `$c=3`.

**So display rendering might NOT break even if round-trip tests do.** This creates a dangerous situation where card displays look correct but round-trip parsing is silently broken.

### Migration Coexistence

The plan structures migration bottom-up (costs → triggers → conditions → predicates → effects → abilities). During migration:

- `effect_serializer` calls `cost_serializer::serialize_cost()` internally (line 215, 697-701)
- `effect_serializer` calls `predicate_serializer::serialize_predicate()` extensively
- `ability_serializer` calls `trigger_serializer`, `effect_serializer`, `cost_serializer`, `static_ability_serializer`

**If cost_serializer is migrated but effect_serializer isn't:**

The cost_serializer would produce template text (via RLF escaped braces). The effect_serializer would embed this in its own format strings. For example:

```rust
// effect_serializer.rs line 697 (NOT yet migrated)
StandardEffect::OpponentPaysCost { cost } => {
    format!("the opponent pays {}.", cost_serializer::serialize_cost(cost, bindings))
}
```

If `cost_serializer::serialize_cost` still produces the same string (because escaped braces work), this is fine. **The output format of each serializer function is the integration contract.** As long as the output string is identical, cross-serializer calls work.

**RISK:** If the output changes even subtly (e.g., trailing/leading whitespace, capitalization), the calling serializer will embed different text, breaking round-trip tests for the composed output.

---

## 3. Incremental Migration Safety

### Cross-Serializer Call Graph

```
ability_serializer
  ├── trigger_serializer
  │     └── predicate_serializer
  ├── cost_serializer
  │     └── predicate_serializer
  ├── effect_serializer
  │     ├── cost_serializer
  │     ├── predicate_serializer
  │     ├── condition_serializer
  │     ├── static_ability_serializer
  │     │     └── predicate_serializer
  │     ├── trigger_serializer
  │     └── text_formatting
  ├── static_ability_serializer
  └── serializer_utils (capitalize_first_letter, lowercase_leading_keyword)
```

### The Bottom-Up Order is Correct

The plan's ordering (cost → trigger → condition → predicate → effect → ability) is the right direction. Each serializer returns `String`, so the interface between serializers is just string composition. As long as the returned string is identical, migration of one serializer doesn't affect others.

### Where Incremental Migration Can Go Wrong

**1. Predicate serializer partial migration (Tasks 4-6) is the most dangerous.**

The predicate serializer is split across 3 tasks. It has ~15 public functions that are called by effect_serializer, trigger_serializer, cost_serializer, and condition_serializer. If `serialize_predicate` is migrated (Task 4) but `serialize_card_predicate` isn't, and both return different formats, callers that use both will get inconsistent output.

**Mitigation:** Within the predicate serializer, migrate ALL public-facing functions atomically or ensure the return type is identical before/after for each function.

**2. `serialize_for_each_predicate` and `serialize_for_count_expression` live in different files.**

`serialize_for_each_predicate` is in `predicate_serializer.rs` (Task 4-6), but `serialize_for_count_expression` is in `effect_serializer.rs` (Task 11). Both produce predicate-style text. If one is migrated and the other isn't, the output format for "for each X" expressions may diverge.

**Mitigation:** Co-migrate these or verify identical output explicitly.

**3. `FormattedText` interactions.**

`text_formatting::card_predicate_base_text()` returns `FormattedText`, which is used by both the predicate_serializer and effect_serializer. If Task 15 (Clean Up text_formatting.rs) happens before all callers are migrated, it will break callers that still depend on `FormattedText`.

**Mitigation:** Task 15 must happen LAST, after all serializers are migrated.

---

## 4. Rollback Story

### Current State: No Rollback Mechanism

The plan has **no feature flag, no toggle, no abstraction layer** for switching between old and new serializer behavior. Each task directly modifies the serializer source code.

### What Happens If Migration Goes Wrong Halfway

Suppose Tasks 1-3 (cost, trigger, condition) succeed, but Task 4 (predicate Phase 1) introduces a bug that isn't caught until Task 8 (effect serializer).

**Rollback options:**
1. **Git revert:** Revert commits back to the last working state. This is straightforward since each task is a separate commit. But it loses all work.
2. **Fix forward:** Debug the issue and fix it. This is the realistic path.
3. **Feature flag:** Not available in the current plan.

### Recommendation: Add a Validation Gate After Each Task

Instead of a feature flag, add a stronger validation step:

```bash
just parser-test  # Round-trip tests
just review       # Full review including cards.toml round-trip
```

The plan already suggests this but doesn't emphasize it enough. **The `cards_toml_round_trip_tests` test (which tests ALL cards in the game) is the critical validation gate.** If this passes after each task, the migration is safe. If it fails, stop and investigate before proceeding.

**RISK:** The plan doesn't mention running `cards_toml_round_trip_tests` after each individual task — only at the end (Task 16). This is insufficient. A bug introduced in Task 4 that only manifests for obscure predicate combinations in `cards.toml` might not be caught until Task 16.

**Mitigation:** Run `just parser-test` (which includes `cards_toml_round_trip_tests`) after EVERY task, not just at the end.

---

## 5. Hidden Coupling

### 5.1. Parser → Serializer Coupling via Round-Trip Tests

The parser expects specific input formats. For example, the parser recognizes `{Banish}` (capital B, inside braces) as a keyword. The serializer must produce exactly this format for round-trip to work. This coupling is tested by round-trip tests but is implicit — there's no shared constant or type defining the format.

**Risk:** If an RLF phrase subtly changes the output (e.g., `{Banish}` becomes `{banish}` due to case normalization), the parser won't recognize it.

**Current protection:** Round-trip tests catch this. But only for cards that are explicitly tested.

### 5.2. `eval_str` Template Syntax Coupling

The display layer's `eval_str()` function evaluates template text against RLF definitions. It expects specific directive names (e.g., `{Banish}`, `{energy($e)}`, `{dissolve}`). The RLF definitions in `strings.rs` define what these directives resolve to.

**Hidden coupling:** The serializer output, the parser input, and the RLF definitions must all agree on directive names. If the serializer starts outputting `{banish_keyword}` instead of `{Banish}`, `eval_str` won't find a matching definition.

**The plan doesn't change directive names** — it only wraps them in escaped braces. So this coupling is preserved. But it's fragile and undocumented.

### 5.3. `capitalize_first_letter` and `lowercase_leading_keyword` Coupling

These functions in `serializer_utils.rs` parse template text to find `{keyword}` patterns and change their case. They are called extensively by `ability_serializer.rs` and `effect_serializer.rs`.

**Risk:** After migration, if the serializer output format changes (e.g., RLF pre-evaluates keywords), these functions will operate on rendered HTML (`<color=#AA00FF>banish</color>`) instead of template text (`{banish}`). This would silently fail — `capitalize_first_letter` looks for a leading `{` but finds `<`.

**Mitigation:** As long as escaped braces work correctly (output remains template text), these functions continue to work. But this is another reason the escaped-brace behavior is a critical prerequisite.

### 5.4. `VariableBindings` Round-Trip Coupling

The test `assert_round_trip` checks BOTH `serialized.text == expected_text` AND `serialized.variables == parsed_bindings`. The serializer must populate `VariableBindings` with exactly the right keys and values.

The plan preserves `bindings.insert(...)` calls alongside the RLF phrase calls. But the RLF phrase also receives the integer value directly (e.g., `strings::draw_cards_effect(3)`). This means the same value is passed twice through different paths:
- Path 1: `bindings.insert("c", Integer(3))` (for round-trip test compatibility)
- Path 2: `strings::draw_cards_effect(3)` (for RLF phrase evaluation)

**Risk:** If these get out of sync (e.g., someone changes the phrase call but not the binding insert, or vice versa), the serializer will produce wrong output or bindings.

### 5.5. Lexer Case Sensitivity

Per MEMORY.md: "Lexer lowercases all input." The parser input is lowercased, but the serializer output preserves original casing (via `capitalize_first_letter`). The round-trip test compares the pre-lowercasing input against the serializer output.

Wait — the round-trip test calls `assert_eq!(expected_text, serialized.text)` where `expected_text` is the original mixed-case string. But the lexer lowercases input before parsing. So the parser sees lowercase tokens, and the serializer must reconstruct the original casing.

**This is already working** and the migration doesn't change the casing logic. But it's worth noting as a fragile coupling point.

---

## 6. VariableBindings Dual-Path Analysis

### What Consumes VariableBindings

1. **Round-trip tests** (`assert_round_trip`): Compare `serialized.variables` against parsed vars.
2. **`cards_toml_round_trip_tests`**: Same comparison for all cards in the game.
3. **Display rendering** (`rlf_helper::eval_str`): Converts template text to rendered text using bindings as RLF parameters.

### Is the Dual Path Necessary?

**Yes, for this incremental migration phase.** Here's why:

- The serializer output is still template text with `{directives}`.
- `eval_str` needs `VariableBindings` to evaluate those directives (e.g., `{cards($c)}` needs to know `$c = 3`).
- If we drop `VariableBindings`, `eval_str` can't evaluate the template.

The dual path becomes unnecessary in **Phase 2** when the serializer returns fully-evaluated text (via `Phrase.to_string()`). At that point, `eval_str` is no longer needed and `VariableBindings` can be removed from the serializer.

### Inconsistency Risk

**The dual path creates an inconsistency surface:**

```rust
// Task 1 cost_serializer.rs after migration
Cost::DiscardCards { count, .. } => {
    bindings.insert("d".to_string(), VariableValue::Integer(*count));  // Path 1
    strings::discard_cards_cost(*count).to_string()                    // Path 2
}
```

Both paths receive `*count` — they can't diverge for this value. But consider a more complex case where the phrase receives a transformed value:

```rust
// Hypothetical: phrase receives count+1 by mistake
strings::discard_cards_cost(*count + 1).to_string()  // Bug: off by one
```

The `bindings` would still have the correct value, so `eval_str` would render correctly, but the direct phrase evaluation would produce wrong text. In the current (Phase 1) design, the phrase output IS the serializer output, so the wrong text would be returned and the round-trip test would catch it.

**This means the round-trip tests serve as a consistency check between the two paths.** If the phrase produces different text than what `eval_str` would produce from the bindings, the round-trip test fails because the expected text (which was parsed from template format) won't match the phrase-evaluated text.

**However**, this only works if the escaped-brace approach actually produces template text. If phrases produce evaluated text (the scenario where escaped braces don't suppress `$` evaluation), then the round-trip test fails for a different reason (format mismatch), and the consistency between paths can't be verified.

---

## Summary of Risks and Mitigations

| # | Risk | Severity | Likelihood | Mitigation |
|---|------|----------|------------|------------|
| 1 | `{{directive($var)}}` doesn't preserve `$var` as literal | **CRITICAL** | **Medium** | Write RLF unit test before ANY migration work. This is a blocking prerequisite. |
| 2 | Individual serializer migration changes output subtly | High | Medium | Run `cards_toml_round_trip_tests` after EVERY task, not just at the end. |
| 3 | Predicate serializer partial migration breaks cross-serializer calls | High | Medium | Migrate all public predicate functions atomically per task, verify output parity. |
| 4 | `capitalize_first_letter` / `lowercase_leading_keyword` break on non-template text | High | Low (if #1 is OK) | Verify these functions still work on migrated output. |
| 5 | Dual-path `VariableBindings` + phrase call diverge | Medium | Low | Round-trip tests serve as consistency check. Enforce both paths use same source values. |
| 6 | No rollback mechanism for partial migration | Medium | Low | Git history provides rollback. Each task is a separate commit. |
| 7 | `serialize_for_count_expression` and `serialize_for_each_predicate` migrated at different times | Medium | Medium | Co-migrate or verify output parity explicitly. |
| 8 | Display rendering silently works even when round-trip is broken (Scenario B from §2) | Medium | Medium | Don't rely on visual testing alone. Round-trip tests are the source of truth. |

### Recommended Action Items

1. **[BLOCKING] Write an RLF unit test for `{{directive($var)}}` escape behavior.** If it doesn't produce `{directive($var)}` as literal text, the entire plan needs restructuring.

2. **[BLOCKING] If the escape test fails, consider an alternative approach:** Have the RLF phrases return the same hardcoded strings directly (without escaped directive references), purely for semantic naming. E.g., `discard_cards_cost($d) = "discard {{cards($$d)}}."` using the `$$` → literal `$` escape, producing `"discard {cards($d)}."`. Verify `$$` inside `{{...}}` works.

3. **Run `just parser-test` after every single task**, not just at the end.

4. **Add a simple smoke test** that serializes a few representative abilities and compares against known-good output, to catch regressions early in the migration.

5. **Document the serializer output contract**: Each serializer function must return the same string as before migration. This is implicit but should be explicit.
