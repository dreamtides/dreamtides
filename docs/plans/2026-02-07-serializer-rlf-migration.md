# Serializer RLF Migration Plan

**Goal:** Migrate all hardcoded English strings in the serializer to RLF phrase calls, producing language-agnostic serializer code that can support translations.

**Architecture:** Serializers currently return `(String, VariableBindings)` where the String contains embedded RLF directives like `{energy($e)}`. After migration, serializers will call RLF phrase functions (e.g., `strings::draw_cards(count)`) that return `Phrase`, then `.to_string()` the result. The serializer continues to return `String` (not `Phrase`) and continues to populate `VariableBindings` for parser round-trip compatibility. Each serializer is migrated independently, bottom-up.

**Tech Stack:** Rust, RLF macro (`rlf::rlf!`), chumsky parser, Dreamtides rules engine

---

## Conceptual Model

At the meta level, the entire serializer is a **compiler from the Ability AST into RLF syntax**. The parser converts card text (which is itself RLF) into structured `Ability` types; the serializer reverses this, converting `Ability` back into RLF. Today the serializer produces RLF via hardcoded English format strings — after this migration, it produces RLF by composing named RLF phrases. This means:

- The **Rust serializer code becomes language-agnostic** — it maps semantic concepts (DrawCards, DissolveCharacter) to named RLF phrases, never touching natural language directly.
- The **RLF phrase definitions contain ALL language-specific logic** — sentence structure, word order, articles, gender agreement, pluralization, case marking.
- **Adding a new language** means writing a new `.rlf` translation file with the same phrase names but different templates. No Rust changes needed.
- The serializer's job is purely **semantic**: "this effect is a dissolve action on a target" → call `strings::dissolve_target(target_phrase)`. How that renders in English, Chinese, or Russian is entirely RLF's concern.

---

## Background Context

### Current Pipeline
```
Card TOML → Parser → Ability AST → Serializer → (String with {directives}, VariableBindings)
                                                         ↓
                                         rlf_helper::eval_str() → final formatted String
```

### Current Serializer Pattern
```rust
// effect_serializer.rs - CURRENT
StandardEffect::DrawCards { count } => {
    bindings.insert("c".to_string(), VariableValue::Integer(*count));
    "draw {cards($c)}.".to_string()
}
```

### Target Serializer Pattern
```rust
// effect_serializer.rs - AFTER MIGRATION
StandardEffect::DrawCards { count } => {
    bindings.insert("c".to_string(), VariableValue::Integer(*count));
    strings::draw_cards(*count).to_string()
}
```

### Key Constraint: Round-Trip Tests
The test helper `assert_round_trip(expected_text, vars)` parses text, serializes it back, and asserts exact string equality. After migration, the serializer output will be *evaluated* RLF text (no more `{directives}`) rather than template text. This means:
1. The round-trip tests will need updating to compare against evaluated output
2. The `eval_str` call in display rendering becomes redundant (serializer already produces final text)
3. We need to keep `VariableBindings` populated for now to avoid breaking the parser side

### Important: VariableBindings Dual Path
During migration, serializers will:
1. **Still populate VariableBindings** (for parser round-trip tests and backward compatibility)
2. **Also call RLF phrases directly** (for localization-ready output)

The serializer return value changes from template text to evaluated text, but the bindings remain.

### File Locations
- **Serializers:** `rules_engine/src/parser_v2/src/serializer/`
- **RLF strings:** `rules_engine/src/strings/src/strings.rs`
- **Round-trip tests:** `rules_engine/tests/parser_v2_tests/tests/round_trip_tests/`
- **Test helpers:** `rules_engine/tests/parser_v2_tests/src/test_helpers.rs`
- **RLF helper (eval_str):** `rules_engine/src/display/src/rendering/rlf_helper.rs`
- **Display callers:** `rules_engine/src/display/src/rendering/card_rendering.rs`, `dreamwell_card_rendering.rs`, `modal_effect_prompt_rendering.rs`

### Commands
- `just fmt` - format code
- `just check` - type check
- `just clippy` - lint
- `just review` - clippy + style + tests
- `just parser-test` - run parser/serializer tests
- `just battle-test <NAME>` - run specific battle test

---

## Task 1: Cost Serializer Migration

**Why first:** Smallest serializer (~130 lines), well-understood from the Spanish appendix doc, self-contained (only calls predicate_serializer internally).

**Files:**
- Modify: `rules_engine/src/strings/src/strings.rs` (add cost phrases)
- Modify: `rules_engine/src/parser_v2/src/serializer/cost_serializer.rs`
- Modify: round-trip test files that test cost-related abilities

### Step 1: Add cost RLF phrases to strings.rs

Add these phrases to `rules_engine/src/strings/src/strings.rs` inside the existing `rlf::rlf!` block. Place them after the existing keyword definitions (around line 70, before the modal card formatting section).

Design principle: Each phrase represents a *semantic intent*, not a grammatical fragment. Phrases that take a `$target` parameter expect a `Phrase` value (for future language support), but for now we pass pre-rendered strings since we haven't migrated predicates yet.

```rust
    // =========================================================================
    // Cost Phrases
    // =========================================================================

    // Abandon costs
    abandon_any_number_of($target) = "abandon any number of {$target}";
    abandon_target($target) = "abandon {$target}";
    abandon_count($count) = "abandon {count_allies($count)}";

    // Discard costs
    discard_cards_cost($d) = "discard {cards($d)}";
    discard_your_hand = "discard your hand";

    // Energy costs (already have energy($e) for display, this is for cost context)
    energy_cost($e) = "{energy($e)}";
    lose_maximum_energy_cost($m) = "lose {maximum_energy($m)}";
    pay_one_or_more_energy = "pay 1 or more {energy_symbol}";

    // Banish costs
    banish_another_card_in_void = "{Banish} another card in your void";
    banish_cards_from_your_void($c) = "{Banish} {cards($c)} from your void";
    banish_cards_from_enemy_void($c) = "{Banish} {cards($c)} from the opponent's void";
    banish_void_with_min_count($n) = "{Banish} your void with {count($n)} or more cards";
    banish_from_hand_cost($target) = "{Banish} {$target} from hand";
    banish_your_void = "{Banish} your void";

    // Return to hand costs
    return_target_to_hand($target) = "return {$target} to hand";
    return_count_to_hand($n, $target) = "return {$n} {$target} to hand";
    return_all_but_one_to_hand($target) = "return all but one {$target} to hand";
    return_all_to_hand($target) = "return all {$target} to hand";
    return_any_number_to_hand($target) = "return any number of {$target} to hand";
    return_up_to_to_hand($n, $target) = "return up to {$n} {$target} to hand";
    return_each_other_to_hand($target) = "return each other {$target} to hand";
    return_or_more_to_hand($n, $target) = "return {$n} or more {$target} to hand";

    // Cost connectors
    cost_or = " or ";
    cost_and = " and ";

    // Trigger cost wrapper
    pay_cost($cost) = "pay {$cost}";
```

Note: Some of these phrases (like `energy_cost`) duplicate existing phrases. That's intentional - they represent different semantic contexts. A translator might render `energy($e)` (display) differently from `energy_cost($e)` (paying a cost). For now they're identical in English.

### Step 2: Refactor cost_serializer.rs

Replace each hardcoded string with the corresponding `strings::` call. Keep the `bindings.insert()` calls intact.

```rust
// cost_serializer.rs - AFTER

use ability_data::collection_expression::CollectionExpression;
use ability_data::cost::Cost;
use ability_data::variable_value::VariableValue;
use strings::strings;

use crate::serializer::predicate_serializer;
use crate::variables::parser_bindings::VariableBindings;

pub fn serialize_cost(cost: &Cost, bindings: &mut VariableBindings) -> String {
    match cost {
        Cost::AbandonCharactersCount { target, count } => match count {
            CollectionExpression::AnyNumberOf => {
                let target_text = predicate_serializer::serialize_predicate_plural(target, bindings);
                strings::abandon_any_number_of(target_text).to_string()
            }
            CollectionExpression::Exactly(1) => {
                let target_text = predicate_serializer::serialize_predicate(target, bindings);
                strings::abandon_target(target_text).to_string()
            }
            CollectionExpression::Exactly(n) => {
                bindings.insert("a".to_string(), VariableValue::Integer(*n));
                strings::abandon_count(*n).to_string()
            }
            _ => strings::abandon_count(0).to_string(), // fallback
        },
        Cost::DiscardCards { count, .. } => {
            bindings.insert("d".to_string(), VariableValue::Integer(*count));
            strings::discard_cards_cost(*count).to_string()
        }
        Cost::DiscardHand => strings::discard_your_hand().to_string(),
        Cost::Energy(energy) => {
            bindings.insert("e".to_string(), VariableValue::Integer(energy.0));
            strings::energy_cost(energy.0).to_string()
        }
        Cost::LoseMaximumEnergy(amount) => {
            bindings.insert("m".to_string(), VariableValue::Integer(*amount));
            strings::lose_maximum_energy_cost(*amount).to_string()
        }
        Cost::BanishCardsFromYourVoid(count) => {
            if *count == 1 {
                strings::banish_another_card_in_void().to_string()
            } else {
                bindings.insert("c".to_string(), VariableValue::Integer(*count));
                strings::banish_cards_from_your_void(*count).to_string()
            }
        }
        Cost::BanishCardsFromEnemyVoid(count) => {
            bindings.insert("c".to_string(), VariableValue::Integer(*count));
            strings::banish_cards_from_enemy_void(*count).to_string()
        }
        Cost::BanishAllCardsFromYourVoidWithMinCount(min_count) => {
            bindings.insert("n".to_string(), VariableValue::Integer(*min_count));
            strings::banish_void_with_min_count(*min_count).to_string()
        }
        Cost::BanishFromHand(predicate) => {
            let target_text = predicate_serializer::serialize_predicate(predicate, bindings);
            strings::banish_from_hand_cost(target_text).to_string()
        }
        Cost::Choice(costs) => {
            let or_str = strings::cost_or().to_string();
            costs.iter().map(|c| serialize_cost(c, bindings)).collect::<Vec<_>>().join(&or_str)
        }
        Cost::ReturnToHand { target, count } => {
            serialize_return_to_hand(target, count, bindings)
        }
        Cost::SpendOneOrMoreEnergy => strings::pay_one_or_more_energy().to_string(),
        Cost::BanishAllCardsFromYourVoid => strings::banish_your_void().to_string(),
        Cost::CostList(costs) => {
            let and_str = strings::cost_and().to_string();
            costs.iter().map(|c| serialize_cost(c, bindings)).collect::<Vec<_>>().join(&and_str)
        }
    }
}

fn serialize_return_to_hand(
    target: &ability_data::predicate::Predicate,
    count: &CollectionExpression,
    bindings: &mut VariableBindings,
) -> String {
    match count {
        CollectionExpression::Exactly(1) => {
            let target_text = predicate_serializer::serialize_predicate(target, bindings);
            strings::return_target_to_hand(target_text).to_string()
        }
        CollectionExpression::Exactly(n) => {
            let target_text = predicate_serializer::serialize_predicate_plural(target, bindings);
            strings::return_count_to_hand(*n, target_text).to_string()
        }
        CollectionExpression::AllButOne => {
            let target_text = predicate_serializer::predicate_base_text(target, bindings);
            strings::return_all_but_one_to_hand(target_text).to_string()
        }
        CollectionExpression::All => {
            let target_text = predicate_serializer::serialize_predicate(target, bindings);
            strings::return_all_to_hand(target_text).to_string()
        }
        CollectionExpression::AnyNumberOf => {
            let target_text = predicate_serializer::serialize_predicate(target, bindings);
            strings::return_any_number_to_hand(target_text).to_string()
        }
        CollectionExpression::UpTo(n) => {
            let target_text = predicate_serializer::serialize_predicate_plural(target, bindings);
            strings::return_up_to_to_hand(*n, target_text).to_string()
        }
        CollectionExpression::EachOther => {
            let target_text = predicate_serializer::serialize_predicate(target, bindings);
            strings::return_each_other_to_hand(target_text).to_string()
        }
        CollectionExpression::OrMore(n) => {
            let target_text = predicate_serializer::serialize_predicate_plural(target, bindings);
            strings::return_or_more_to_hand(*n, target_text).to_string()
        }
    }
}

pub fn serialize_trigger_cost(cost: &Cost, bindings: &mut VariableBindings) -> String {
    match cost {
        Cost::Energy(_) => {
            let cost_text = serialize_cost(cost, bindings);
            strings::pay_cost(cost_text).to_string()
        }
        _ => serialize_cost(cost, bindings),
    }
}
```

### Step 3: Critical change — update round-trip test expectations

The key difference: the serializer output is now *evaluated* RLF, not *template* RLF. For example:
- **Before:** `"{energy($e)}"` (template, later evaluated by `eval_str`)
- **After:** `"<color=#00838F>3●</color>"` (already evaluated by RLF)

This means round-trip tests comparing against template strings will break. **However**, the current serializer output for cost_serializer is mostly plain text with embedded `{directives}`. After migration, the directives are evaluated.

The round-trip test approach needs to change:
- Tests should now compare against **evaluated** output
- OR we create a separate test that validates cost serializer output independently

**Recommended approach:** Update `assert_round_trip` to handle the new evaluated output. Since the serializer now produces final text, the comparison should be against what `eval_str` would have produced.

Actually, wait — this is the key insight for the migration strategy. The serializer output currently contains `{Banish}` which `eval_str` renders as `<color=#AA00FF>Banish</color>`. If the serializer now calls `strings::banish()`, it will produce the rendered version directly.

**But the parser expects `{Banish}` as input, not the rendered version.** So the round-trip `parse → serialize → compare` cycle will break if the serializer now outputs rendered text.

### Step 3 (revised): Keep template output for round-trip, use RLF for semantic structure

Actually, after deeper analysis, the correct approach for this incremental migration is:

**The serializer output must continue to produce template text** (with `{directives}`) because:
1. The parser reads `{Banish}` not `<color=#AA00FF>Banish</color>`
2. Round-trip tests compare serializer output against original parsed text
3. `eval_str` in the display layer converts templates to rendered text

So the RLF phrases for the cost serializer should produce **the same template text** that the hardcoded strings produced. The RLF phrases encapsulate the English sentence structure, but the output is still template format.

This means the RLF phrases need to output template text:
```rust
// strings.rs - cost phrases output TEMPLATE text
banish_another_card_in_void = "{{Banish}} another card in your void";
discard_your_hand = "discard your hand";
```

Wait — this creates a bootstrapping problem. The `{Banish}` in the template is itself an RLF directive that gets evaluated later by `eval_str`. If we put `{Banish}` inside an RLF phrase, the rlf! macro will try to evaluate it as a reference to the `banish` term.

**This is actually correct and desired!** The cost serializer RLF phrase should reference the `banish` term:
```rust
banish_another_card_in_void = "{Banish} another card in your void";
```

When evaluated, `{Banish}` resolves to `<color=#AA00FF>Banish</color>`. So the serializer would output rendered text, not template text.

**This breaks round-trip tests.** We need a different approach.

### Step 3 (final): Two-layer approach

The cleanest incremental approach is:

1. **RLF phrases define the semantic structure** but output "intermediate" template strings
2. **The serializer continues to output template text** for round-trip compatibility
3. **eval_str continues to convert templates to rendered text** in the display layer

To achieve this, the RLF phrases should use escaped braces `{{Banish}}` to output literal `{Banish}`:

```rust
// strings.rs
banish_another_card_in_void = "{{Banish}} another card in your void";
// Evaluates to: "{Banish} another card in your void"
```

But wait — `{{` in RLF is the escape for literal `{`. So `{{Banish}}` would produce `{Banish}` as literal text. This is exactly what we want! The serializer output remains template text, and `eval_str` later evaluates it.

**However**, this means the RLF phrases are just returning the same strings they would have returned before, just routed through RLF. The main value is:
1. All English text is centralized in strings.rs
2. A translator can provide different sentence structures
3. The semantic intent is named (e.g., `abandon_any_number_of`)

For the first migration phase, this is the right trade-off. When we later migrate to Phrase-based composition (Phase 2), the phrases will reference other phrases directly instead of embedding escaped directives.

### Revised RLF Phrases

```rust
    // =========================================================================
    // Cost Phrases (output template text with escaped braces for directives)
    // =========================================================================

    abandon_any_number_of($target) = "abandon any number of {$target}";
    abandon_target($target) = "abandon {$target}";
    abandon_count_allies($a) = "abandon {{count_allies($a)}}";

    discard_cards_cost($d) = "discard {{cards($d)}}";
    discard_your_hand_cost = "discard your hand";

    energy_cost_value($e) = "{{energy($e)}}";
    lose_max_energy_cost($m) = "lose {{maximum_energy($m)}}";
    pay_one_or_more_energy_cost = "pay 1 or more {{energy_symbol}}";

    banish_another_in_void = "{{Banish}} another card in your void";
    banish_cards_from_void($c) = "{{Banish}} {{cards($c)}} from your void";
    banish_cards_from_enemy_void($c) = "{{Banish}} {{cards($c)}} from the opponent's void";
    banish_void_min_count($n) = "{{Banish}} your void with {{count($n)}} or more cards";
    banish_from_hand($target) = "{{Banish}} {$target} from hand";
    banish_your_void_cost = "{{Banish}} your void";

    return_to_hand($target) = "return {$target} to hand";
    return_count_to_hand($n, $target) = "return {$n} {$target} to hand";
    return_all_but_one_to_hand($target) = "return all but one {$target} to hand";
    return_all_to_hand($target) = "return all {$target} to hand";
    return_any_number_to_hand($target) = "return any number of {$target} to hand";
    return_up_to_to_hand($n, $target) = "return up to {$n} {$target} to hand";
    return_each_other_to_hand($target) = "return each other {$target} to hand";
    return_or_more_to_hand($n, $target) = "return {$n} or more {$target} to hand";

    cost_or_connector = " or ";
    cost_and_connector = " and ";
    pay_prefix($cost) = "pay {$cost}";
```

**IMPORTANT NOTE:** The `$target` parameter receives pre-rendered predicate strings (e.g., "an ally", "allies"). The `$a`, `$c`, `$d`, `$e`, `$m`, `$n` parameters receive integers. The double-brace `{{...}}` escaping preserves the directive as literal text in the output.

**IMPORTANT NOTE 2:** We need to verify that RLF handles `{{count_allies($a)}}` correctly — the `$a` inside double-braces should be treated as literal text, not as a parameter reference. If RLF evaluates `$a` inside double-braces, we may need `$$a` escaping or a different approach. Check the RLF DESIGN.md escape rules.

From the RLF DESIGN.md: "Inside `{}` expressions: `$$` → literal `$`". And `{{` → literal `{`. So `{{count_allies($a)}}` should produce `{count_allies($a)}` literally — the `$a` inside the escaped braces should NOT be evaluated because it's inside an escape sequence. The `{{` turns off expression parsing until the matching `}}`.

### Step 4: Update cost_serializer.rs

Replace hardcoded strings with `strings::` calls. Pass pre-rendered predicate strings as `$target` and integers for counts.

The refactored code follows the pattern shown in Step 2 above, but uses the revised phrase names from Step 3.

### Step 5: Run tests, fix any issues

```bash
just parser-test
just review
```

Verify all round-trip tests pass. The output should be identical since the RLF phrases produce the same template text as the original hardcoded strings.

### Step 6: Commit

```bash
git add rules_engine/src/strings/src/strings.rs rules_engine/src/parser_v2/src/serializer/cost_serializer.rs
git commit -m "feat: migrate cost_serializer to RLF phrases"
```

---

## Task 2: Trigger Serializer Migration

**Why second:** Small (~127 lines), self-contained, only calls predicate_serializer.

**Files:**
- Modify: `rules_engine/src/strings/src/strings.rs` (add trigger phrases)
- Modify: `rules_engine/src/parser_v2/src/serializer/trigger_serializer.rs`

### Step 1: Add trigger RLF phrases to strings.rs

```rust
    // =========================================================================
    // Trigger Phrases
    // =========================================================================

    when_you_play($target) = "when you play {$target}, ";
    when_opponent_plays($target) = "when the opponent plays {$target}, ";
    when_you_play_from_hand($target) = "when you play {$target} from your hand, ";
    when_you_play_cards_in_turn($c) = "when you play {{cards_numeral($c)}} in a turn, ";
    when_you_play_in_turn($target) = "when you play {$target} in a turn, ";
    when_you_play_during_enemy_turn($target) = "when you play {$target} during the opponent's turn, ";
    when_you_discard($target) = "when you discard {$target}, ";
    when_you_materialize($target) = "when you {{materialize}} {$target}, ";
    when_dissolved($target) = "when {$target} is {{dissolved}}, ";
    when_banished($target) = "when {$target} is {{banished}}, ";
    when_leaves_play($target) = "when {$target} leaves play, ";
    when_you_abandon($target) = "when you abandon {$target}, ";
    when_you_abandon_count_in_turn($a) = "when you abandon {{count_allies($a)}} in a turn, ";
    when_put_into_void($target) = "when {$target} is put into your void, ";
    when_you_draw_in_turn($c) = "when you draw {{cards_numeral($c)}} in a turn, ";
    at_end_of_your_turn = "at the end of your turn, ";
    when_deck_empty = "when you have no cards in your deck, ";
    when_you_materialize_nth_in_turn($n, $target) = "when you {{materialize}} {{text_number($n)}} {$target} in a turn, ";
    when_you_gain_energy = "when you gain energy, ";
```

### Step 2: Refactor trigger_serializer.rs

Replace hardcoded strings with `strings::` calls. Keep keyword serialization as-is (it produces `{Judgment}`, `{Materialized}`, etc. which are template directives).

### Step 3: Run tests, fix issues

```bash
just parser-test
just review
```

### Step 4: Commit

```bash
git commit -m "feat: migrate trigger_serializer to RLF phrases"
```

---

## Task 3: Condition Serializer Migration

**Why third:** Smallest (~96 lines), simple match arms.

**Files:**
- Modify: `rules_engine/src/strings/src/strings.rs` (add condition phrases)
- Modify: `rules_engine/src/parser_v2/src/serializer/condition_serializer.rs`

### Step 1: Add condition RLF phrases

```rust
    // =========================================================================
    // Condition Phrases
    // =========================================================================

    with_allies_sharing_type($a) = "with {{count_allies($a)}} that share a character type,";
    if_discarded_this_turn($target) = "if you have discarded {$target} this turn";
    if_drawn_count_this_turn($n) = "if you have drawn {{count($n)}} or more cards this turn";
    while_void_count($n) = "while you have {{count($n)}} or more cards in your void,";
    if_character_dissolved = "if a character dissolved this turn";
    with_allied_subtype($t) = "with an allied {{subtype($t)}},";
    with_predicate($pred) = "with {$pred},";
    if_card_in_void = "if this card is in your void,";
```

### Steps 2-4: Refactor, test, commit

Same pattern as Tasks 1-2.

---

## Task 4: Predicate Serializer Migration (Phase 1 - Base Terms)

**Why fourth:** The predicate serializer is the foundation for everything else. It's large (~800 lines) so we split into phases. Phase 1 migrates the base building blocks.

**Files:**
- Modify: `rules_engine/src/strings/src/strings.rs` (add predicate term phrases)
- Modify: `rules_engine/src/parser_v2/src/serializer/predicate_serializer.rs`
- Modify: `rules_engine/src/parser_v2/src/serializer/text_formatting.rs`

### Step 1: Add predicate base terms to strings.rs

Many of these already exist (`card`, `ally`, subtypes). Add missing ones:

```rust
    // =========================================================================
    // Predicate Base Terms
    // =========================================================================

    this_character = "this character";
    that_character = "that character";
    them_pronoun = "them";
    it_pronoun = "it";
    these_characters = "these characters";
    those_characters = "those characters";

    // Card type terms (already have card, but need singular forms for predicates)
    a_card = "a card";
    a_character = "a character";
    an_event = "an event";
    cards_plural = "cards";
    characters_plural = "characters";
    events_plural = "events";

    // Ownership qualifiers
    your_card_singular = "your card";
    your_event_singular = "your event";
    your_cards_plural = "your cards";
    your_events_plural = "your events";

    // Allied terms
    an_ally = "an ally";
    allies_plural = "allies";
    allied_character_singular = "allied character";
    allied_event_singular = "allied event";

    // Enemy terms
    an_enemy = "an enemy";
    enemies_plural = "enemies";
    enemy_card_singular = "enemy card";
    enemy_event_singular = "enemy event";

    // Location qualifiers
    in_your_void($target) = "{$target} in your void";
    in_enemy_void($target) = "{$target} in the opponent's void";
    another_target($target) = "another {$target}";
    other_target($target) = "other {$target}";
```

### Step 2: Refactor text_formatting.rs

Replace `FormattedText` construction with RLF term lookups where possible. `FormattedText` can remain as a struct but delegate to RLF terms for the actual text.

### Step 3: Refactor predicate_serializer.rs base functions

Start with `serialize_predicate`, `serialize_predicate_plural`, `predicate_base_text` — the top-level dispatch functions that map `Predicate` variants to text.

### Steps 4-5: Test and commit

---

## Task 5: Predicate Serializer Migration (Phase 2 - Card Predicates)

**Files:**
- Modify: `rules_engine/src/strings/src/strings.rs` (add card predicate phrases)
- Modify: `rules_engine/src/parser_v2/src/serializer/predicate_serializer.rs`

### Step 1: Add card predicate phrases

```rust
    // =========================================================================
    // Card Predicate Phrases
    // =========================================================================

    a_fast_target($target) = "a {{fast}} {$target}";
    with_cost($target, $op) = "{$target} with cost {{energy($e)}}{$op}";
    character_with_materialized = "a character with a {{materialized}} ability";
    character_with_activated = "a character with an activated ability";
    character_not_subtype($t) = "a character that is not {{@a subtype($t)}}";
    character_with_spark($s, $op) = "a character with spark {{$s}}{$op}";
    event_could_dissolve($target) = "an event which could {{dissolve}} {$target}";

    // "with cost less than" family
    with_cost_less_than_allies($target, $allies) = "{$target} with cost less than the number of allied {$allies}";
    with_cost_less_than_abandoned($target) = "{$target} with cost less than the abandoned ally's cost";
    with_spark_less_than_abandoned($target) = "{$target} with spark less than the abandoned ally's spark";
    with_spark_less_than_abandon_count($target) = "{$target} with spark less than the number of allies abandoned this turn";
    with_cost_less_than_void($target) = "{$target} with cost less than the number of cards in your void";
    with_spark_less_than_energy($target) = "{$target} with spark less than the amount of {{energy_symbol}} paid";
```

### Step 2: Refactor card predicate functions

Migrate `serialize_card_predicate`, `serialize_card_predicate_plural`, `serialize_card_predicate_without_article`, `serialize_cost_constraint_only`, `serialize_fast_target`.

### Steps 3-4: Test and commit

---

## Task 6: Predicate Serializer Migration (Phase 3 - Your/Enemy/ForEach)

**Files:**
- Modify: `rules_engine/src/strings/src/strings.rs`
- Modify: `rules_engine/src/parser_v2/src/serializer/predicate_serializer.rs`

### Step 1: Add ownership-qualified predicate phrases

```rust
    // Your predicate phrases
    allied_subtype($t) = "allied {{subtype($t)}}";
    allied_subtype_plural($t) = "allied {{@plural subtype($t)}}";
    ally_not_subtype($t) = "ally that is not {{@a subtype($t)}}";
    allies_not_subtype($t) = "allies that are not {{@plural subtype($t)}}";
    ally_with_spark($s, $op) = "ally with spark {{$s}}{$op}";
    allies_with_spark($s, $op) = "allies with spark {{$s}}{$op}";
    ally_with_materialized = "ally with a {{materialized}} ability";
    allies_with_materialized = "allies with {{materialized}} abilities";
    ally_with_activated = "ally with an activated ability";
    allies_with_activated = "allies with activated abilities";

    // Enemy predicate phrases
    enemy_subtype($t) = "enemy {{subtype($t)}}";
    enemy_subtype_plural($t) = "enemy {{@plural subtype($t)}}";
    non_subtype_enemy($t) = "non-{{subtype($t)}} enemy";
    enemy_with_spark($s, $op) = "enemy with spark {{$s}}{$op}";
    enemy_with_materialized = "enemy with a {{materialized}} ability";
    enemy_with_activated = "enemy with an activated ability";
    enemy_with_cost($op) = "enemy with cost {{energy($e)}}{$op}";

    // For-each predicate phrases
    for_each_ally = "ally";
    for_each_enemy = "enemy";
    for_each_character = "character";
    for_each_card = "card";
    for_each_event = "event";
    for_each_card_in_void = "card in your void";
    for_each_character_in_void = "character in your void";
    for_each_event_in_void = "event in your void";
    for_each_card_in_enemy_void = "card in the opponent's void";
    for_each_allied_character = "allied character";
    for_each_other_character = "other character";
    for_each_prefix($target) = "each {$target}";
```

### Steps 2-4: Refactor, test, commit

---

## Task 7: Serializer Utils Migration

**Files:**
- Modify: `rules_engine/src/strings/src/strings.rs` (add operator phrases)
- Modify: `rules_engine/src/parser_v2/src/serializer/serializer_utils.rs`

### Step 1: Add operator and utility phrases

```rust
    // =========================================================================
    // Operator Phrases
    // =========================================================================

    operator_or_less = " or less";
    operator_or_more = " or more";
    operator_lower = " lower";
    operator_higher = " higher";
```

### Step 2: Refactor serialize_operator

The `capitalize_first_letter` and `lowercase_leading_keyword` functions handle template directive casing. These may remain as utility functions since they operate on template syntax, not natural language text. They'll be eliminated in Phase 2 when we move to Phrase-based composition.

### Steps 3-4: Test and commit

---

## Task 8: Effect Serializer Migration (Phase 1 - Simple Effects)

**Why split:** effect_serializer.rs is ~1200 lines with 80+ match arms. We split into phases by complexity.

**Files:**
- Modify: `rules_engine/src/strings/src/strings.rs` (add effect phrases)
- Modify: `rules_engine/src/parser_v2/src/serializer/effect_serializer.rs`

### Step 1: Add simple effect phrases

Start with effects that take only counts (no predicates):

```rust
    // =========================================================================
    // Simple Effect Phrases
    // =========================================================================

    draw_cards_effect($c) = "draw {{cards($c)}}.";
    discard_cards_effect($d) = "discard {{cards($d)}}.";
    gain_energy_effect($e) = "gain {{energy($e)}}.";
    gain_points_effect($p) = "gain {{points($p)}}.";
    lose_points_effect($p) = "you lose {{points($p)}}.";
    opponent_gains_points($p) = "the opponent gains {{points($p)}}.";
    opponent_loses_points($p) = "the opponent loses {{points($p)}}.";
    foresee_effect($f) = "{{foresee($f)}}.";
    kindle_effect($k) = "{{kindle($k)}}.";
    each_player_discards($d) = "each player discards {{cards($d)}}.";
    gain_energy_equal_to_that_cost = "gain {{energy_symbol}} equal to that character's cost.";
    gain_energy_equal_to_this_cost = "gain {{energy_symbol}} equal to this character's cost.";
    opponent_gains_points_equal_spark = "the opponent gains points equal to its spark.";
    prevent_that_card = "{{prevent}} that card.";
    take_extra_turn = "take an extra turn after this one.";
    you_win_the_game = "you win the game.";
    gain_twice_energy_instead = "gain twice that much {{energy_symbol}} instead.";
    then_materialize_it = "then {{materialize}} it.";
    no_effect = "";
    // ... more simple effects
```

### Step 2: Refactor simple effect match arms

### Steps 3-4: Test and commit

---

## Task 9: Effect Serializer Migration (Phase 2 - Target Effects)

Effects that combine an action with a predicate target.

### Step 1: Add target effect phrases

```rust
    // =========================================================================
    // Target Effect Phrases
    // =========================================================================

    dissolve_target($target) = "{{dissolve}} {$target}.";
    banish_target($target) = "{{banish}} {$target}.";
    materialize_target($target) = "{{materialize}} {$target}.";
    reclaim_target($target) = "{{reclaim}} {$target}.";
    discover_target($target) = "{{Discover}} {$target}.";
    gain_control_of($target) = "gain control of {$target}.";
    copy_target($target) = "copy {$target}.";

    // Gains spark
    target_gains_spark($target, $s) = "{$target} gains +{{$s}} spark.";
    each_gains_spark($each, $s) = "have each {$each} gain +{{$s}} spark.";

    // ... more target effects
```

### Steps 2-4: Refactor, test, commit

---

## Task 10: Effect Serializer Migration (Phase 3 - Collection Effects)

Effects with collection expressions (all, N, up to N, any number).

### Step 1: Add collection effect phrases

```rust
    // =========================================================================
    // Collection Effect Phrases
    // =========================================================================

    dissolve_all($target) = "{{dissolve}} all {$target}.";
    dissolve_count($n, $target) = "{{dissolve}} {$n} {$target}.";
    dissolve_up_to($n, $target) = "{{dissolve}} up to {$n} {$target}.";
    dissolve_any_number($target) = "{{dissolve}} any number of {$target}.";

    banish_all($target) = "{{banish}} all {$target}.";
    banish_count($n, $target) = "{{banish}} {$n} {$target}.";
    banish_up_to($n, $target) = "{{banish}} up to {$n} {$target}.";
    banish_any_number($target) = "{{banish}} any number of {$target}.";

    materialize_all($target) = "{{materialize}} all {$target}.";
    materialize_any_number($target) = "{{materialize}} any number of {$target}.";
    materialize_up_to($n, $target) = "{{materialize}} up to {$n} {$target}.";

    // ... and compound actions like banish-then-materialize
```

### Steps 2-4: Refactor, test, commit

---

## Task 11: Effect Serializer Migration (Phase 4 - Complex Effects)

Remaining complex effects: for-each patterns, compound effects, quantity expressions.

### Step 1: Add complex effect phrases

```rust
    // For-each effects
    draw_for_each($c, $for_each) = "draw {{cards($c)}} for each {$for_each}.";
    gain_energy_for_each($e, $for_each) = "gain {{energy($e)}} for each {$for_each}.";
    gain_points_for_each($p, $for_each) = "gain {{points($p)}} for each {$for_each}.";
    gains_spark_for_each($target, $s, $for_each) = "{$target} gains +{{$s}} spark for each {$for_each}.";

    // Compound effects
    banish_then_materialize($target) = "{{banish}} {$target}, then {{materialize}} it.";
    // ... etc
```

### Steps 2-4: Refactor, test, commit

---

## Task 12: Effect Serializer Migration (Phase 5 - Structural Logic)

The `serialize_effect_with_context` function handles `Effect::List`, `Effect::WithOptions`, `Effect::ListWithOptions`, `Effect::Modal`. These involve structural connectors.

### Step 1: Add structural phrases

```rust
    // =========================================================================
    // Structural Phrases
    // =========================================================================

    you_may_prefix = "you may ";
    cost_to_connector($cost) = "{$cost} to ";
    until_end_of_turn_prefix = "until end of turn, ";
    once_per_turn_prefix = "Once per turn, ";
    fast_prefix = "{{Fast}} -- ";
    once_per_turn_suffix = ", once per turn";
    cost_effect_separator = ": ";
    then_joiner = ", then ";
    and_joiner = " and ";
```

### Steps 2-4: Refactor, test, commit

---

## Task 13: Static Ability Serializer Migration

**Files:**
- Modify: `rules_engine/src/strings/src/strings.rs`
- Modify: `rules_engine/src/parser_v2/src/serializer/static_ability_serializer.rs`

### Step 1: Add static ability phrases

```rust
    // =========================================================================
    // Static Ability Phrases
    // =========================================================================

    cards_cost_more($target, $e) = "{$target} cost you {{energy($e)}} more.";
    cards_cost_less($target, $e) = "{$target} cost you {{energy($e)}} less.";
    enemy_cards_cost_more($target, $e) = "the opponent's {$target} cost {{energy($e)}} more.";
    allied_have_spark($target, $s) = "allied {$target} have +{{$s}} spark.";
    to_play_this_card($cost) = "To play this card, {$cost}.";
    characters_in_hand_have_fast = "characters in your hand have {{fast}}.";
    disable_enemy_materialized = "disable the {{Materialized}} abilities of enemies.";
    has_all_character_types = "has all character types.";
    reveal_top_of_deck = "reveal the top card of your deck.";
    look_at_top_of_deck = "you may look at the top card of your deck.";
    play_only_from_void = "you may only play this character from your void.";
    reclaim_equal_to_cost = "they have {{reclaim}} equal to their cost.";
    // ... more static ability phrases
```

### Steps 2-4: Refactor, test, commit

---

## Task 14: Ability Serializer Migration

**Files:**
- Modify: `rules_engine/src/strings/src/strings.rs`
- Modify: `rules_engine/src/parser_v2/src/serializer/ability_serializer.rs`

The ability serializer is the orchestrator. Most of its strings are structural (prefixes, separators). Many were already covered in Task 12.

### Step 1: Add remaining ability phrases

```rust
    // Named ability phrases
    reclaim_for_cost_ability($r) = "{{Reclaim_For_Cost($r)}}";
    reclaim_named_ability($cost) = "{{Reclaim}} -- {$cost}";
```

### Steps 2-4: Refactor, test, commit

---

## Task 15: Clean Up text_formatting.rs

After Tasks 4-6, most of `text_formatting.rs` is redundant. The `FormattedText` struct can potentially be removed if all callers now use RLF terms directly.

### Step 1: Audit remaining callers of FormattedText

### Step 2: Remove or simplify text_formatting.rs

### Step 3: Test and commit

---

## Task 16: Integration Testing

### Step 1: Run full round-trip test suite

```bash
just parser-test
```

### Step 2: Run cards.toml round-trip test

This is the ultimate validation — every card in the game parses and serializes correctly.

### Step 3: Run display rendering tests

Verify that `eval_str` continues to produce correct rendered text from the serializer's template output.

### Step 4: Run full review

```bash
just review
```

### Step 5: Final commit

---

## Future Work (Not Part of This Plan)

### Phase 2: Phrase-Based Composition
- Serializers return `Phrase` instead of `String`
- Predicates pass `Phrase` to effect phrases (enables gender agreement)
- Remove `VariableBindings` from serializer path
- Remove `eval_str` — serializer produces final rendered text

### Phase 3: Translation Files
- Create `.rlf` translation files for each target language
- Add RLF features as needed (case variants, classifiers, etc.)
- Create translation testing infrastructure

### Phase 4: Chinese/Russian/German-Specific RLF Features
- Classifier transforms for Chinese
- Case-based variant selection for Russian/German
- Animacy tags for Russian
- Personal "a" transform for Spanish
- Contraction transforms for Portuguese
