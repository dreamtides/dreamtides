# Serializer RLF Migration — Technical Design Document

---

## 1. Goal

Migrate all hardcoded English strings in `rules_engine/src/parser_v2/src/serializer/` to named RLF phrase calls in `strings.rs`. The serializer code becomes language-agnostic in *intent* (every output is a named semantic phrase) while continuing to produce English template text for round-trip test compatibility.

**This is Phase 1.5 — semantic cataloging and code organization.** It is explicitly NOT full localization. Real multilingual support requires Phase 2 (Phrase-based composition), which is sketched at the end of this document. Phase 1.5 produces durable value: phrase names, parameter signatures, and call-site refactoring survive into Phase 2. The escaped-brace phrase *bodies* (~65% of phrases) are temporary and will be rewritten. The primary value of Phase 1.5 is the **semantic inventory** — enumerating every string the serializer produces and assigning it a name. This cataloging work is the hardest intellectual part of the migration and survives fully into Phase 2.

**Durability caveats:** Call sites will need minor changes in Phase 2 (removing `bindings.insert()` calls when RLF evaluates parameters directly). Serializer return types will change from `String` to `Phrase`. Phrase bodies for ~65% of phrases will be rewritten. But phrase names and parameter counts are stable.

**Target languages for Dreamtides (informing design decisions):**
English, Simplified Chinese, Russian, Spanish, Portuguese-Brazil, German

---

## 2. Architecture

### 2.1 Current Pipeline

```
Card TOML → Parser → Ability AST → Serializer → (template String, VariableBindings)
                                                          ↓
                                          rlf_helper::eval_str() → rendered String
```

The serializer produces template text containing `{directives}` like `{Banish}`, `{cards($c)}`, `{energy($e)}`. The display layer's `eval_str()` resolves these against RLF definitions in `strings.rs` and `VariableBindings` to produce final rendered text with colors and symbols.

### 2.2 Phase 1.5 Pipeline (after this migration)

```
Card TOML → Parser → Ability AST → Serializer → (template String, VariableBindings)
                                     ↓ calls                    ↓
                               strings::phrase()     rlf_helper::eval_str() → rendered String
```

The serializer calls named RLF phrases instead of using inline format strings. Phrases use escaped braces `{{...}}` to output literal `{directives}` that `eval_str()` resolves in a second pass. **The serializer output is identical to today** — same template text, same `VariableBindings`. No changes to the parser, display layer, or test infrastructure.

### 2.3 Why Escaped Braces

The parser expects `{Banish}` as input, not `<color=#AA00FF>Banish</color>`. Round-trip tests compare serializer output against original parsed text. If RLF phrases evaluated `{Banish}` directly, the serializer would produce rendered text, breaking all round-trip tests.

The escaped-brace approach (`{{Banish}}` → literal `{Banish}`) lets RLF phrases define sentence structure while preserving the template text format. This is confirmed working — `$param` inside `{{...}}` is NOT evaluated; it becomes literal text. (Verified against RLF test suite: `test_escape_braces()`, `test_escaped_braces_with_dollar_param()`, and end-to-end in `escape_sequences.rs`.)

**Important: Two escape mechanisms.** The current serializer code uses two different mechanisms that produce `{`:
1. **Rust format strings**: `format!("{{materialize}}")` uses Rust's `{{` escaping to produce literal `{materialize}` in the format output.
2. **Plain strings**: `"draw {cards($c)}.".to_string()` — `{` is literal because `.to_string()` is not a format string.

Phase 1.5 introduces a third mechanism: **RLF escape syntax** in phrase bodies where `{{cards($c)}}` evaluates to literal `{cards($c)}`. These are three different systems (Rust formatting, string literals, RLF escaping) that all produce the same output. After migration, only the RLF mechanism remains.

### 2.4 Phrase Categories

**Any phrase containing `{{...}}` is Category B**, regardless of how simple it appears. This is the sole criterion.

**Category A — Final phrases (survive unchanged into Phase 2):**
- Self-contained text with NO escaped braces
- Parameters are only `$target` (pre-rendered predicate strings) or simple text
- Examples: `discard_your_hand_cost`, `take_extra_turn_effect`, `cost_or_connector`

**Category B — Temporary phrases (escaped braces, rewritten in Phase 2):**
- Contain `{{directive}}` patterns that produce literal template text
- Parameters with `$` inside `{{...}}` are not evaluated — they pass through as literal text for `eval_str()`
- Marked with `// Phase 2: requires Phrase composition` comments
- Examples: `draw_cards_effect`, `banish_your_void_cost`, `prevent_that_card_effect`

### 2.5 Key Constraint: Phantom Parameters in Category B

In Category B phrases like `draw_cards_effect($c) = "draw {{cards($c)}}.";`, the `$c` parameter is **never evaluated** by RLF. The output is always `draw {cards($c)}.` regardless of what integer is passed. The parameter exists for API stability — when Phase 2 replaces `{{cards($c)}}` with `{cards($c)}`, the parameter will be evaluated. For Phase 1.5, the actual value comes from `VariableBindings` via `eval_str()`.

### 2.6 Variable Name Coupling Risk

Literal `$c`, `$e`, etc. inside escaped braces must match `VariableBindings` keys. There is no compile-time validation of this coupling. A typo produces a runtime error when `eval_str()` can't resolve the variable.

**Mitigation:** Round-trip tests serve as the primary consistency check — the `cards_toml_round_trip_tests` test uses byte-for-byte comparison of template text AND exact `VariableBindings` equality. A typo in a phrase body produces different template text, which the test catches immediately with an error message showing the card name, expected text, and actual text side-by-side.

**Additional mitigation:** When writing each phrase, verify each `$var` inside `{{...}}` matches the corresponding `bindings.insert("var", ...)` call. A mechanical check: grep for all `bindings.insert` calls in each serializer and verify they appear in the corresponding phrases.

**Complete variable name inventory:**

| Variable | Meaning | Used By |
|----------|---------|---------|
| `$e` | Energy amount | cost, effect, trigger costs |
| `$c` | Card count | cost, effect, trigger |
| `$d` | Discard count | cost, effect |
| `$a` | Ally count | cost, condition |
| `$m` | Max energy | cost |
| `$n` | Generic count | cost, condition, effect, trigger |
| `$t` | Subtype | condition, effect |
| `$f` | Foresee count | effect |
| `$k` | Kindle amount | effect |
| `$p` | Points amount | effect |
| `$s` | Spark amount | effect |
| `$v` | Void card count | effect |

---

## 3. Scope

### 3.1 In Scope (Phase 1.5)

| File | Lines | Arms | Coverage |
|------|-------|------|----------|
| `cost_serializer.rs` | 131 | ~28 (14 top-level + nested) | Full |
| `trigger_serializer.rs` | 127 | 22 in `serialize_trigger_event` | Full (keyword arms kept as Rust format strings) |
| `condition_serializer.rs` | 96 | 22 (9 in `serialize_condition` + 13 in `serialize_predicate_count`) | Full |
| `serializer_utils.rs` | 86 | 5 (`serialize_operator` only) | Partial |
| `effect_serializer.rs` simple arms | ~200 | ~26 StandardEffect variants | Partial |
| `strings.rs` additions | — | — | New phrases |

### 3.2 Out of Scope (deferred to Phase 2)

| File | Reason |
|------|--------|
| `predicate_serializer.rs` (803 lines, 16 functions) | Returns `String` with English articles; must return `Phrase` in Phase 2. |
| `text_formatting.rs` (79 lines, `FormattedText` struct) | Poor man's `Phrase` — replaced by real RLF terms in Phase 2. |
| `static_ability_serializer.rs` (222 lines) | Circular dependency with effect_serializer; complex conditional string building. |
| `ability_serializer.rs` (176 lines) | Orchestrator with conditional capitalization; depends on all other serializers. |
| `effect_serializer.rs` complex arms | Arms calling predicate_serializer, FormattedText, or serialize_gains_reclaim. |
| `effect_serializer.rs` structural logic | `serialize_effect_with_context` (4 Effect::List branches, ListWithOptions, Modal). |
| `effect_serializer.rs` helpers | `serialize_gains_reclaim`, `serialize_void_gains_reclaim`, `serialize_for_count_expression` (public, 15 arms), `serialize_allied_card_predicate`. |
| `capitalize_first_letter` / `lowercase_leading_keyword` | Template-text operations; kept as-is until Phase 2. |

### 3.3 Decision: Why Skip Predicate Serializer

The predicate serializer is 803 lines with 16 functions, ~100 match arms, deep `FormattedText` coupling, and is called by every other serializer (~80 call sites). In Phase 1.5, predicate phrases would receive pre-rendered English strings as `$target` parameters — fundamentally blocking i18n for Russian (case declension on both nouns AND adjectives), German (article declension across 4 cases), Spanish (personal "a" before animate direct objects, subjunctive mood in temporal clauses), Chinese (classifiers, prenominal modifiers), and Portuguese (contractions, future subjunctive). All predicate phrases would be rewritten in Phase 2 when predicates return `Phrase` objects. Migrating it now doubles the work for no lasting benefit.

---

## 4. Phase 2 Prep Work (zero-risk, do during Phase 1.5)

### 4.1 Add Animacy Tags to English Predicate Terms

Enrich existing terms in `strings.rs` with animacy tags. These don't affect English output but signal metadata for future translations. Gender tags (`:masc`/`:fem`/`:neut`) are language-specific and will appear only in translation files (e.g., `ru.rlf`), not in the English source.

```rust
// Current:
ally = :an { one: "ally", other: "allies" };
card = :a { one: "card", other: "cards" };

// Enriched with animacy:
ally = :an :anim { one: "ally", other: "allies" };
enemy = :an :anim { one: "enemy", other: "enemies" };
character = :a :anim { one: "character", other: "characters" };
event = :an :inan { one: "event", other: "events" };
card = :a :inan { one: "card", other: "cards" };
```

Translation files will add language-specific gender:
```
// ru.rlf — gender + animacy + case declension
card = :fem :inan {
    nom: "карта", nom.few: "карты", nom.many: "карт",
    acc: "карту", acc.few: "карты", acc.many: "карт",
};
character = :masc :anim {
    nom: "персонаж", nom.few: "персонажа", nom.many: "персонажей",
    acc: "персонажа", acc.few: "персонажей", acc.many: "персонажей",
};

// de.rlf — gender + case (no animacy distinction)
card = :fem { nom: "Karte", acc: "Karte", dat: "Karte", nom.other: "Karten" };
```

### 4.2 Add Missing Keywords

`{Aegis}` is used in effect_serializer output (line 668) but not defined in `strings.rs`. Add it:

```rust
aegis = <color={keyword_color}><b>Aegis</b></color>;
```

---

## 5. Validation Protocol

After **every single task** (not just at the end):

```bash
just review    # clippy + style validator + ALL tests including cards_toml_round_trip_tests
```

If `just review` fails, stop and fix before proceeding. The `cards_toml_round_trip_tests` test serializes every card in the game — it is the ultimate validation that no serializer output changed. It uses byte-for-byte comparison and shows the card name, expected text, and actual text in the error message, making diagnosis straightforward.

**Optional phrase unit tests:** Consider adding a small test file exercising Category B phrases to verify escaped-brace output in isolation (e.g., `assert_eq!(strings::draw_cards_effect(1).to_string(), "draw {cards($c)}.")`). This is defense-in-depth — the cards_toml test already catches mismatches — but catches typos earlier in the feedback loop.

---

## 6. Task Breakdown

### Task 1: Cost Serializer Migration

**Files:** `strings.rs`, `cost_serializer.rs`
**Risk:** Low — leaf serializer, no callers depend on internal structure.

#### Step 1: Add cost phrases to strings.rs

```rust
    // =========================================================================
    // Cost Phrases — Category B (Phase 2: requires Phrase composition)
    // =========================================================================

    // Phase 2: $param is literal inside {{...}}, resolved by eval_str
    abandon_count_allies($a) = "abandon {{count_allies($a)}}";
    discard_cards_cost($d) = "discard {{cards($d)}}";
    energy_cost_value($e) = "{{energy($e)}}";
    lose_max_energy_cost($m) = "lose {{maximum_energy($m)}}";
    banish_your_void_cost = "{{Banish}} your void";
    banish_another_in_void = "{{Banish}} another card in your void";
    banish_cards_from_void($c) = "{{Banish}} {{cards($c)}} from your void";
    banish_cards_from_enemy_void($c) = "{{Banish}} {{cards($c)}} from the opponent's void";
    banish_void_min_count($n) = "{{Banish}} your void with {{count($n)}} or more cards";
    banish_from_hand_cost($target) = "{{Banish}} {$target} from hand";

    // =========================================================================
    // Cost Phrases — Category A (Final)
    // =========================================================================

    discard_your_hand_cost = "discard your hand";
    pay_one_or_more_energy_cost = "pay 1 or more {energy_symbol}";
    cost_or_connector = " or ";
    cost_and_connector = " and ";
    pay_prefix($cost) = "pay {$cost}";

    // Cost target phrases (Category A — $target is pre-rendered, no escaped braces)
    abandon_any_number_of($target) = "abandon any number of {$target}";
    abandon_target($target) = "abandon {$target}";
    return_target_to_hand($target) = "return {$target} to hand";
    return_count_to_hand($n, $target) = "return {$n} {$target} to hand";
    return_all_but_one_to_hand($target) = "return all but one {$target} to hand";
    return_all_to_hand($target) = "return all {$target} to hand";
    return_any_number_to_hand($target) = "return any number of {$target} to hand";
    return_up_to_to_hand($n, $target) = "return up to {$n} {$target} to hand";
    return_each_other_to_hand($target) = "return each other {$target} to hand";
    return_or_more_to_hand($n, $target) = "return {$n} or more {$target} to hand";
```

#### Step 2: Refactor cost_serializer.rs

Replace every hardcoded string with the corresponding `strings::` call. Keep all `bindings.insert(...)` calls unchanged. The function signature and return type do not change.

All ~28 match arms (including nested `CollectionExpression` arms in `AbandonCharactersCount` with 4 sub-arms, `ReturnToHand` with 8 sub-arms, and the `BanishCardsFromYourVoid` if-else branch) must be migrated. `serialize_trigger_cost` wraps `serialize_cost` with the `pay_prefix` phrase for energy costs.

Cross-serializer calls to `predicate_serializer::serialize_predicate()`, `serialize_predicate_plural()`, and `predicate_base_text()` remain as-is — their return values are passed as `$target` string parameters to the new phrases.

Note: `cost_or_connector` and `cost_and_connector` are used in `.join()` calls. Use `strings::cost_or_connector().to_string()` as the join separator.

#### Step 3: Validate

```bash
just review
```

#### Step 4: Commit

---

### Task 2: Trigger Serializer Migration

**Files:** `strings.rs`, `trigger_serializer.rs`
**Risk:** Low — leaf serializer, only calls predicate_serializer for target text.

#### Step 1: Add trigger phrases to strings.rs

```rust
    // =========================================================================
    // Trigger Phrases — Category A (Final)
    // =========================================================================

    at_end_of_your_turn_trigger = "at the end of your turn, ";
    when_deck_empty_trigger = "when you have no cards in your deck, ";
    when_you_gain_energy_trigger = "when you gain energy, ";

    // Trigger target phrases (Category A)
    when_you_play_trigger($target) = "when you play {$target}, ";
    when_opponent_plays_trigger($target) = "when the opponent plays {$target}, ";
    when_you_play_from_hand_trigger($target) = "when you play {$target} from your hand, ";
    when_you_play_in_turn_trigger($target) = "when you play {$target} in a turn, ";
    when_you_play_during_enemy_turn_trigger($target) = "when you play {$target} during the opponent's turn, ";
    when_you_discard_trigger($target) = "when you discard {$target}, ";
    when_leaves_play_trigger($target) = "when {$target} leaves play, ";
    when_you_abandon_trigger($target) = "when you abandon {$target}, ";
    when_put_into_void_trigger($target) = "when {$target} is put into your void, ";

    // =========================================================================
    // Trigger Phrases — Category B (Phase 2: requires Phrase composition)
    // =========================================================================

    when_you_materialize_trigger($target) = "when you {{materialize}} {$target}, ";
    when_dissolved_trigger($target) = "when {$target} is {{dissolved}}, ";
    when_banished_trigger($target) = "when {$target} is {{banished}}, ";
    when_you_play_cards_in_turn_trigger($c) = "when you play {{$c}} {{card:$c}} in a turn, ";
    when_you_abandon_count_in_turn_trigger($a) = "when you abandon {{count_allies($a)}} in a turn, ";
    when_you_draw_in_turn_trigger($c) = "when you draw {{$c}} {{card:$c}} in a turn, ";
    when_you_materialize_nth_in_turn_trigger($n, $target) = "when you {{materialize}} {{text_number($n)}} {$target} in a turn, ";
```

**Note on `PlayCardsInTurn` and `DrawCardsInTurn`:** The serializer outputs `{$c} {card:$c}` (inline RLF parameterized selection), NOT `{cards_numeral($c)}` (which was removed in commit 5c41d958). The phrase bodies must match current serializer output exactly: `"when you play {{$c}} {{card:$c}} in a turn, "`. The `{{$c}}` and `{{card:$c}}` produce literal `{$c}` and `{card:$c}` which eval_str resolves.

#### Step 2: Refactor trigger_serializer.rs

Migrate all 22 match arms in `serialize_trigger_event`. For the keyword arms (`Keywords` with len==1, len==2, fallback), these produce `{Judgment}`, `{Materialized_Judgment}`, etc. via `format!("{{{}}}", keyword)` — Rust triple-brace escaping to produce template directives. Since the keyword text is dynamic (from `serialize_keyword`), keep these arms as Rust format strings. Document why in a comment.

`serialize_keyword` returns plain strings ("Judgment", "Materialized", "Dissolved") — no migration needed.

#### Step 3: Validate and commit

---

### Task 3: Condition Serializer Migration

**Files:** `strings.rs`, `condition_serializer.rs`
**Risk:** Low — small file, well-defined.

#### Step 1: Add condition phrases to strings.rs

```rust
    // =========================================================================
    // Condition Phrases — Category A (Final)
    // =========================================================================

    if_character_dissolved_this_turn = "if a character dissolved this turn";
    if_card_in_your_void = "if this card is in your void,";

    // Condition target phrases (Category A)
    if_discarded_this_turn($target) = "if you have discarded {$target} this turn";
    with_predicate_condition($pred) = "with {$pred},";

    // =========================================================================
    // Condition Phrases — Category B (Phase 2: requires Phrase composition)
    // =========================================================================

    with_allies_sharing_type($a) = "with {{count_allies($a)}} that share a character type,";
    if_drawn_count_this_turn($n) = "if you have drawn {{count($n)}} or more cards this turn";
    while_void_count($n) = "while you have {{count($n)}} or more cards in your void,";
    with_allied_subtype($t) = "with an allied {{subtype($t)}},";
    with_count_allied_subtype($a, $t) = "{{count_allied_subtype($a, $t)}}";
    with_count_allies($a) = "{{count_allies($a)}}";
```

#### Step 2: Refactor condition_serializer.rs

Migrate all 9 match arms in `serialize_condition` AND all 13 match arms in `serialize_predicate_count`. For the delegation arms in `serialize_predicate_count` (that call `predicate_serializer::serialize_predicate_plural`), no phrase is needed — the call stays as-is. Only the `Another(CharacterType(_))` and `Another(Character)` arms that produce `{count_allied_subtype($a, $t)}` and `{count_allies($a)}` need phrases.

#### Step 3: Validate and commit

---

### Task 4: Serializer Utils — Operator Phrases

**Files:** `strings.rs`, `serializer_utils.rs`
**Risk:** Low — purely additive.

#### Step 1: Add operator phrases

```rust
    // =========================================================================
    // Operator Phrases — Category A (Final)
    // =========================================================================

    operator_or_less = " or less";
    operator_or_more = " or more";
    operator_lower = " lower";
    operator_higher = " higher";
```

#### Step 2: Refactor `serialize_operator`

Replace the 5 match arms. `Operator::Exactly` returns empty string — use `String::new()` or a phrase that evaluates to empty.

**Do NOT migrate** `capitalize_first_letter` or `lowercase_leading_keyword`. These operate on template syntax (`{keyword}` patterns) and are essential for the current pipeline. Note: `capitalize_first_letter` has special title-case logic for underscore-separated keywords (e.g., `reclaim_for_cost` → `Reclaim_For_Cost`). RLF's `@cap` only capitalizes the first letter, so this logic has no direct RLF equivalent. Phase 2 should ensure all keyword terms are defined with correct capitalization in their RLF definitions, eliminating the need for runtime capitalization.

#### Step 3: Validate and commit

---

### Task 5: Effect Serializer — Simple Count-Only Effects

**Files:** `strings.rs`, `effect_serializer.rs`
**Risk:** Low-Medium — modifying a large file (1193 lines) but only touching self-contained arms.

#### Step 1: Add simple effect phrases

These are `StandardEffect` variants that take only counts (no predicates, no FormattedText, no cross-serializer composition):

```rust
    // =========================================================================
    // Simple Effect Phrases — Category B (Phase 2: requires Phrase composition)
    // =========================================================================

    draw_cards_effect($c) = "draw {{cards($c)}}.";
    discard_cards_effect($d) = "discard {{cards($d)}}.";
    gain_energy_effect($e) = "gain {{energy($e)}}.";
    gain_points_effect($p) = "gain {{points($p)}}.";
    lose_points_effect($p) = "you lose {{points($p)}}.";
    opponent_gains_points_effect($p) = "the opponent gains {{points($p)}}.";
    opponent_loses_points_effect($p) = "the opponent loses {{points($p)}}.";
    foresee_effect($f) = "{{foresee($f)}}.";
    kindle_effect($k) = "{{kindle($k)}}.";
    each_player_discards_effect($d) = "each player discards {{cards($d)}}.";
    prevent_that_card_effect = "{{prevent}} that card.";
    then_materialize_it_effect = "then {{materialize}} it.";
    gain_twice_energy_instead_effect = "gain twice that much {{energy_symbol}} instead.";
    gain_energy_equal_to_that_cost_effect = "gain {{energy_symbol}} equal to that character's cost.";
    gain_energy_equal_to_this_cost_effect = "gain {{energy_symbol}} equal to this character's cost.";
    put_deck_into_void_effect($v) = "put the {{top_n_cards($v)}} of your deck into your void.";
    banish_cards_from_enemy_void_effect($c) = "{{banish}} {{cards($c)}} from the opponent's void.";
    banish_enemy_void_effect = "{{banish}} the opponent's void.";
    judgment_phase_at_end_of_turn_effect = "at the end of this turn, trigger an additional {{judgment_phase_name}} phase.";
    multiply_energy_effect($n) = "{{multiply_by($n)}} the amount of {{energy_symbol}} you have.";
    spend_all_energy_dissolve_effect = "spend all your {{energy_symbol}}. {{dissolve}} an enemy with cost less than or equal to the amount spent.";
    spend_all_energy_draw_discard_effect = "spend all your {{energy_symbol}}. Draw cards equal to the amount spent, then discard that many cards.";
    each_player_shuffles_and_draws_effect($c) = "each player shuffles their hand and void into their deck and then draws {{cards($c)}}.";
    return_up_to_events_from_void_effect($n) = "return {{up_to_n_events($n)}} from your void to your hand.";
    fast_prefix = "{{Fast}} -- ";

    // =========================================================================
    // Simple Effect Phrases — Category A (Final)
    // =========================================================================

    opponent_gains_points_equal_spark = "the opponent gains points equal to its spark.";
    take_extra_turn_effect = "take an extra turn after this one.";
    you_win_the_game_effect = "you win the game.";
    no_effect = "";
```

#### Step 2: Refactor simple effect arms

Migrate only the following `StandardEffect` variants (those that don't call `predicate_serializer` or use `FormattedText`):

- `DrawCards`, `DiscardCards`, `GainEnergy`, `GainPoints`, `LosePoints`
- `EnemyGainsPoints`, `EnemyGainsPointsEqualToItsSpark`, `EnemyLosesPoints`
- `Foresee`, `Kindle`
- `EachPlayerDiscardCards`, `EachPlayerShufflesHandAndVoidIntoDeckAndDraws`
- `PutCardsFromYourDeckIntoVoid`
- `BanishCardsFromEnemyVoid`, `BanishEnemyVoid`
- `MultiplyYourEnergy`
- `TakeExtraTurn`, `YouWinTheGame`
- `GainTwiceThatMuchEnergyInstead`, `ThenMaterializeIt`, `NoEffect`
- `SpendAllEnergyDissolveEnemy`, `SpendAllEnergyDrawAndDiscard`
- `TriggerAdditionalJudgmentPhaseAtEndOfTurn`
- `ReturnUpToCountFromYourVoidToHand`
- `GainEnergyEqualToCost` (the `It`/`That` and `This` sub-cases only — the fallback `_` calls `predicate_serializer` and is deferred)
- `Counterspell` — the `Predicate::That | Predicate::It` branch only (produces `{prevent} that card.`; the other branch calls `predicate_serializer` and is deferred)

**Do NOT migrate** any arm that calls `predicate_serializer::*`, `text_formatting::*`, `serialize_for_count_expression`, `serialize_gains_reclaim`, or any other complex helper. Those arms stay as hardcoded strings until Phase 2.

#### Step 3: Validate and commit

---

### Task 6: Effect Serializer — Structural Connectors

**Files:** `strings.rs`
**Risk:** Very low — adding phrases only, no code changes.

Add structural connector phrases that will be used by `ability_serializer` and `serialize_effect_with_context`. These are defined now but not wired into code — those files are deferred to Phase 2. Having the phrases pre-defined means Phase 2 can use them directly.

```rust
    // =========================================================================
    // Structural Phrases — Category A (Final)
    // =========================================================================

    you_may_prefix = "you may ";
    cost_to_connector($cost) = "{$cost} to ";
    until_end_of_turn_prefix = "Until end of turn, ";
    once_per_turn_prefix = "Once per turn, ";
    once_per_turn_suffix = ", once per turn";
    cost_effect_separator = ": ";
    then_joiner = ", then ";
    and_joiner = " and ";
    period_suffix = ".";
```

**Validate and commit.**

---

### Task 7: Phase 2 Prep — Enrich Predicate Terms with Tags

**Files:** `strings.rs`
**Risk:** Zero — tags don't affect English output.

Add `:anim`/`:inan` tags to all card-type and character terms. Add missing terms if not present. Add `aegis` keyword.

**Validate and commit.**

---

## 7. Escaped-Brace Phrase Tracking List

Every Category B phrase must be tracked for Phase 2 rewrite. When Phase 2 begins, search `strings.rs` for `{{` to find all temporary phrases.

### Cost Phrases (Category B)
- `abandon_count_allies($a)`
- `discard_cards_cost($d)`
- `energy_cost_value($e)`
- `lose_max_energy_cost($m)`
- `banish_your_void_cost`
- `banish_another_in_void`
- `banish_cards_from_void($c)`
- `banish_cards_from_enemy_void($c)`
- `banish_void_min_count($n)`
- `banish_from_hand_cost($target)`

### Trigger Phrases (Category B)
- `when_you_materialize_trigger($target)`
- `when_dissolved_trigger($target)`
- `when_banished_trigger($target)`
- `when_you_play_cards_in_turn_trigger($c)`
- `when_you_abandon_count_in_turn_trigger($a)`
- `when_you_draw_in_turn_trigger($c)`
- `when_you_materialize_nth_in_turn_trigger($n, $target)`

### Condition Phrases (Category B)
- `with_allies_sharing_type($a)`
- `if_drawn_count_this_turn($n)`
- `while_void_count($n)`
- `with_allied_subtype($t)`
- `with_count_allied_subtype($a, $t)`
- `with_count_allies($a)`

### Simple Effect Phrases (Category B)
- All `{{directive}}` phrases listed in Task 5 Step 1 (including `prevent_that_card_effect`, `then_materialize_it_effect`, `gain_twice_energy_instead_effect`, `gain_energy_equal_to_*_cost_effect`, `fast_prefix`, and all phrases with `{{energy_symbol}}`, `{{banish}}`, `{{dissolve}}`, etc.)

---

## 8. Cross-Serializer Dependency Graph

```
ability_serializer                    [Phase 2]
  ├── trigger_serializer              [Phase 1.5 — Task 2]
  │     └── predicate_serializer      [Phase 2]
  ├── cost_serializer                 [Phase 1.5 — Task 1]
  │     └── predicate_serializer      [Phase 2]
  ├── effect_serializer               [Phase 1.5 partial — Task 5]
  │     ├── predicate_serializer      [Phase 2]
  │     ├── cost_serializer           [Phase 1.5 — Task 1]
  │     ├── condition_serializer      [Phase 1.5 — Task 3]
  │     │     └── predicate_serializer [Phase 2]
  │     ├── trigger_serializer        [Phase 1.5 — Task 2]
  │     ├── static_ability_serializer [Phase 2]
  │     │     ├── predicate_serializer
  │     │     ├── cost_serializer
  │     │     ├── condition_serializer
  │     │     ├── effect_serializer   [CIRCULAR]
  │     │     └── text_formatting
  │     ├── text_formatting           [Phase 2]
  │     └── serializer_utils          [Phase 1.5 — Task 4]
  ├── serializer_utils                [Phase 1.5 — Task 4]
  └── static_ability_serializer       [Phase 2]
```

**Key insight:** The `static_ability_serializer ↔ effect_serializer` circular dependency means they must be migrated together in Phase 2. The specific coupling points: `effect_serializer` calls `static_ability_serializer::serialize_standard_static_ability()` (line 32); `static_ability_serializer` calls `effect_serializer::serialize_for_count_expression()` (line 185, note: this is a `pub fn`, not private) and `effect_serializer::serialize_effect()` (line 214).

**Note:** `ability_serializer` also depends on `serializer_utils` (for `capitalize_first_letter` and `lowercase_leading_keyword`), not shown in plan's original graph.

---

## 9. What's NOT Covered (Phase 2 Scope)

The following code paths are explicitly deferred. This section serves as the starting inventory for Phase 2 planning.

### 9.1 Predicate Serializer (803 lines, 16 functions)

All 11 public and 5 private functions. This is the **critical path** for Phase 2 — every non-trivial localization pattern depends on predicates returning `Phrase` with gender/case metadata.

### 9.2 Effect Serializer Complex Arms (~40 StandardEffect variants)

All arms calling predicate_serializer or other complex helpers (full list in prior version of this document, preserved in git history).

### 9.3 Effect Serializer Structural Logic

`serialize_effect_with_context`: `Effect::WithOptions`, `Effect::List` (4 code paths), `Effect::ListWithOptions`, `Effect::Modal`.

### 9.4 Effect Serializer Helpers

- `serialize_for_count_expression` (**public**, 15 arms, called by static_ability_serializer)
- `serialize_gains_reclaim` (~116 lines)
- `serialize_void_gains_reclaim` (8 CollectionExpression arms)
- `serialize_allied_card_predicate` / `serialize_allied_card_predicate_plural` (2 arms each)

### 9.5 Static Ability Serializer (222 lines)

Must migrate together with effect_serializer due to circular dependency.

### 9.6 Ability Serializer (176 lines)

Conditional capitalization logic must be restructured for RLF `@cap` transforms.

### 9.7 Text Formatting (79 lines)

`FormattedText` maps directly to RLF concepts: `.with_article()` → `:a`/`:an` tags + `@a` transform, `.plural()` → `one`/`other` variants, `.capitalized()` → `@cap`. Must be replaced simultaneously with predicate_serializer (they are deeply entangled).

---

## 10. Phase 2 Plan Sketch

Phase 2 is a separate planning effort. The recommended order based on team analysis:

### Phase 2.0: Parallel Round-Trip Test Strategy

**Do this FIRST, before any other Phase 2 work.** Create a second round-trip test that compares at the AST level (`parse(text) == parse(serialize(parse(text)))`) alongside the existing byte-for-byte text comparison. Run both strategies in parallel. This de-risks Phase 2.1 — once the AST-level test is proven reliable, you can change serializer output format without breaking the text-equality test being a hard blocker.

**Risk level: HIGH** — this is the single point of failure. If the new test strategy is wrong or incomplete, you lose confidence that serializer changes are correct.

### Phase 2.1: Predicate Serializer + FormattedText → Phrase

Replace `FormattedText` with real RLF terms simultaneously with changing `serialize_predicate()` to return `Phrase`. These are deeply entangled — `FormattedText` is consumed exclusively by predicate functions, and predicate functions produce `FormattedText`.

All predicate terms carry `:a`/`:an` tags and `one`/`other` variants. `:from` is the key enabling mechanism — `subtype($s) = :from($s) "<b>{$s}</b>";` already propagates tags and variants through the composition chain.

This is the hardest single step: 803 lines, 16 functions, ~80 call sites across all serializers. But it unlocks everything else.

### Phase 2.2: Remove Text-Equality Round-Trip Tests

Once Phase 2.0's AST-level tests have run in parallel long enough to build confidence, switch to AST-level comparison as the primary strategy. This removes the constraint that serializer output must exactly match parser input text.

### Phase 2.3: Migrate Remaining Serializers

Cost, trigger, condition: change `$target: String` → `$target: Phrase`. Rewrite Category B phrase bodies to use real RLF references (`{Banish}` instead of `{{Banish}}`). Effect serializer complex arms: now possible with Phrase composition.

### Phase 2.4: Structural Composition (static_ability + effect together)

`Effect::List`, `Effect::ListWithOptions`, `Effect::Modal`. `ability_serializer` orchestration. `serialize_gains_reclaim` and complex helpers. **Plan static_ability ↔ effect migration as a single atomic step** — document which functions in each file depend on the other (see Section 8).

### Phase 2.5: Remove eval_str

Serializer returns fully rendered text via `Phrase.to_string()`. `SerializedAbility` struct changes: drop `variables` field or change `text` to `Phrase`. Display layer calls `.to_string()` directly.

**This must be an atomic switchover** — you can't have some abilities producing template text and others producing rendered text, because `serialize_abilities_text()` iterates over ALL abilities with a single code path. Once all serializers produce Phrase, flip the switch in `card_rendering.rs` all at once.

**Mechanism:** Consider using `PhraseId` (compact, Copy-able, 16-byte identifier from RLF) to store references to definitions in serializable data, enabling `id.call(&locale, &args)` instead of string template evaluation.

### Phase 2.6: Remove VariableBindings

Variables become internal to serialization. `SerializedAbility` no longer exposes `VariableBindings`. Incremental, well-contained.

### Phase 2.7: Remove Capitalization Helpers

`capitalize_first_letter`, `lowercase_leading_keyword` — replaced by RLF `@cap` transforms and correctly-capitalized keyword term definitions.

---

## 11. Multilingual Design Considerations

The following issues were identified by i18n stress testing across all 6 target languages. They MUST be addressed during Phase 2 planning.

### 11.1 Case Declension (Russian, German)

Russian has 6 cases × 3 CLDR plural categories = 18 forms per noun. German has 4 cases × 2 numbers = 8 forms. RLF's multi-dimensional variants with wildcard fallbacks handle this:

```
// ru.rlf
card = :fem :inan {
    nom: "карта", nom.few: "карты", nom.many: "карт",
    acc: "карту", acc.few: "карты", acc.many: "карт",
};

// Usage: "Возьмите {card:acc:$n}."
// n=1 → "Возьмите карту."  (acc + CLDR "one" → wildcard acc)
// n=3 → "Возьмите карты."  (acc + CLDR "few" → acc.few)
```

### 11.2 Gender Agreement on Participles (Russian, Spanish, Portuguese, German)

"when X is dissolved" requires participle agreement with X's gender. Handled by `:match` on gender tags:

```
// ru.rlf
when_dissolved_trigger($target) = :match($target) {
    masc: "когда {$target:nom} растворён, ",
    fem: "когда {$target:nom} растворена, ",
    *neut: "когда {$target:nom} растворено, ",
};

// es.rlf
when_dissolved_trigger($target) = :match($target) {
    masc: "cuando {$target} sea disuelto, ",
    *fem: "cuando {$target} sea disuelta, ",
};
```

### 11.3 Personal "a" (Spanish)

"dissolve an enemy" → "disolver **a** un enemigo" (animate direct object marker). No new RLF features needed — handle with `:match` on `:anim` tag:

```
// es.rlf
dissolve_target($target) = :match($target) {
    anim: "{@cap dissolve} a {@un $target:acc}.",
    *inan: "{@cap dissolve} {@un $target:acc}.",
};
```

### 11.4 Chinese Classifiers and Word Order

Every noun requires a measure word: 一**张**牌 (zhāng for cards), 一**个**角色 (gè for characters). Chinese also uses prenominal modifiers and different word order ("from your void banish cards" instead of "banish cards from your void"). Each Chinese phrase defines its own word order:

```
// zh.rlf
banish_cards_from_void($c) = "从你的虚空中{@cap banish}{@count($c) card}";
draw_cards_effect($c) = "抽{cards($c)}。";
cards($n) = :match($n) { 1: "一张牌", *other: "{$n}张牌" };
```

### 11.5 German Separable Verbs

"auflösen" (dissolve) → "löse...auf" (verb stem + particle split around object). Handled by German phrase structure:

```
// de.rlf
dissolve_target($target) = "Löse {@ein:acc $target} auf.";
```

### 11.6 Contraction Transforms (Portuguese)

"de o" → "do", "em a" → "na". RLF's `@de` and `@em` transforms handle this.

### 11.7 Subjunctive Mood (Spanish, Portuguese)

Spanish uses subjunctive in temporal clauses ("cuando materialices" not "cuando materializas"). Portuguese uses future subjunctive ("quando materializar"). These are purely translation concerns — the translator writes the correct verb form. No RLF feature needed.

### 11.8 Tag System Design

English source terms carry `:a`/`:an` (for English articles) and `:anim`/`:inan` (for cross-language use). Gender tags (`:masc`/`:fem`/`:neut`) are language-specific and appear only in translation files. Animacy and gender are orthogonal dimensions — a Russian word can be `:masc :anim` (персонаж/character) or `:fem :inan` (карта/card).

### 11.9 RLF Feature Verification Checklist

Before Phase 2 begins, verify these RLF features are fully implemented:
- [ ] `@count` transform for CJK classifiers
- [ ] `@der`/`@ein` for German articles + case
- [ ] `@el`/`@un` for Spanish articles
- [ ] `@o` for Portuguese articles
- [ ] Multi-parameter `:match` (e.g., `:match($n, $entity)`)
- [ ] `:from` with multi-dimensional variant propagation

---

## 12. Migration Ordering and Risk Assessment

### 12.1 Task Independence

Tasks 1-4 (cost, trigger, condition, utils) are truly independent leaf serializers with no cross-dependencies. Task 5 (simple effect arms) is also independent — the simple arms being migrated don't call other serializers. Tasks 6-7 are additive-only (new phrases, no code changes).

**However:** All tasks add phrases to `strings.rs`. If done in parallel branches, merge conflicts are guaranteed. Do tasks sequentially (1→2→3→4→5→6→7) to avoid merge conflicts, or designate non-overlapping sections of `strings.rs` for each task.

### 12.2 Performance

Negligible concern. Card serialization is not a hot path (runs once per card display, not per frame). Category B phrases don't even evaluate parameters — they produce static strings. The eval_str two-pass adds no overhead vs. today.

### 12.3 Error Handling

`eval_str()` panics on failure (existing behavior, unchanged by migration). Round-trip tests catch all mismatches before they reach production. No changes to error handling needed for Phase 1.5.

### 12.4 Phase 2 Risk Matrix

| Step | Risk | Reason |
|------|------|--------|
| 2.0 Parallel tests | HIGH | Single point of failure for all subsequent work |
| 2.1 Predicate→Phrase | HIGH | 803 lines, 16 functions, ~80 call sites, entangled with FormattedText |
| 2.2 Remove text tests | MEDIUM | Must prove AST-level tests are sufficient first |
| 2.3 Remaining serializers | LOW | Straightforward once predicates return Phrase |
| 2.4 Structural composition | MEDIUM | Circular dependency requires atomic migration |
| 2.5 Remove eval_str | LOW | Atomic switchover, straightforward once upstream done |
| 2.6 Remove VariableBindings | LOW | Incremental, well-contained |
| 2.7 Remove capitalization | LOW | Straightforward once `@cap` transforms are in place |

---

## Appendix A: File Reference

| File | Path | Lines |
|------|------|-------|
| Serializer directory | `rules_engine/src/parser_v2/src/serializer/` | — |
| RLF strings | `rules_engine/src/strings/src/strings.rs` | ~456 |
| Round-trip test helpers | `rules_engine/tests/parser_v2_tests/src/test_helpers.rs` | 113 |
| Round-trip tests | `rules_engine/tests/parser_v2_tests/tests/round_trip_tests/` | ~multiple files |
| Display eval_str | `rules_engine/src/display/src/rendering/rlf_helper.rs` | 75 |
| Display card rendering | `rules_engine/src/display/src/rendering/card_rendering.rs` | 515 |
| RLF design doc | `~/rlf/docs/DESIGN.md` | 766 |

## Appendix B: Commands

```bash
just fmt          # Format code
just check        # Type check
just clippy       # Lint
just review       # clippy + style + ALL tests (use after every task)
just parser-test  # Parser/serializer tests only
just battle-test <NAME>  # Specific battle test
```

## Appendix C: Multilingual Case Studies

### "Draw 3 cards." across all languages

| Language | n=1 | n=3 | Key Features |
|----------|-----|-----|--------------|
| EN | "Draw a card." | "Draw 3 cards." | `:match` for 1 vs other |
| ZH | "抽一张牌。" | "抽三张牌。" | Classifier 张, number words |
| RU | "Возьмите 1 карту." | "Возьмите 3 карты." | Accusative case, CLDR one/few/many |
| ES | "Roba una carta." | "Roba 3 cartas." | Gender agreement on article |
| PT-BR | "Compre uma carta." | "Compre 3 cartas." | Gender agreement on article |
| DE | "Ziehe eine Karte." | "Ziehe 3 Karten." | Accusative feminine article |

### "Dissolve an enemy Ancient." across all languages

| Language | Output | Key Features |
|----------|--------|--------------|
| EN | "Dissolve an enemy Ancient." | `@a` reads `:an` tag |
| ZH | "消解一个敌方远古。" | Classifier 个, no articles |
| RU | "Растворите вражеского Древнего." | Accusative on BOTH adjective and noun (masc.anim → acc=gen) |
| ES | "Disuelve a un Antiguo enemigo." | Personal "a", reversed adjective order |
| PT-BR | "Dissolva um Ancião inimigo." | Reversed adjective order |
| DE | "Löse einen feindlichen Uralten auf." | Separable verb, accusative article, adjective declension |

These case studies confirm the architecture handles all 6 languages. The critical enabler is Phase 2.1 (predicates returning `Phrase` with gender/case metadata).
