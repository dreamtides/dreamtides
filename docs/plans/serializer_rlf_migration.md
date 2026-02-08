# Serializer RLF Migration — Phase 2 Technical Design Document

---

## 1. Goal

Make the serializer Rust code 100% language-neutral. After Phase 2, every piece of text the serializer produces flows through a named RLF phrase. The serializer outputs **final rendered display strings** — no more template text, no more `VariableBindings`, no more `eval_str()` two-pass rendering. Adding a new language requires only writing a `.rlf` translation file, with zero Rust code changes.

**What Phase 1.5 accomplished:** Every hardcoded string in the leaf serializers (cost, trigger, condition, utils, simple effect arms) was replaced with a named `strings::` phrase call. However, the serializer still produces template text with `{directives}` and `VariableBindings`, using `{{...}}` escaped braces as a compatibility bridge. The predicate serializer, FormattedText, static ability serializer, ability serializer orchestration, and complex effect arms remain unmigrated.

**What Phase 2 does:**
1. Replace `FormattedText` with proper RLF `Phrase` returns carrying metadata (tags, variants)
2. Migrate the predicate serializer to return `Phrase` instead of `String`
3. Migrate all remaining serializers to return `Phrase`
4. Remove `{{...}}` escaped braces — phrases produce final rendered text directly
5. Remove `VariableBindings` — values are passed directly to phrase functions
6. Remove `eval_str()` — the display layer receives final text from the serializer
7. Remove `capitalize_first_letter` / `lowercase_leading_keyword` — use RLF `@cap`

**What is NOT in scope:** Writing translation files for non-English languages. We are building the language-neutral Rust infrastructure; actual translations come later.

**Target languages (informing design decisions):**
English, Simplified Chinese, Russian, Spanish, Portuguese-Brazil, German

---

## 2. Architecture

### 2.1 Current Pipeline (Post Phase 1.5)

```
Card TOML → Parser → Ability AST → Serializer → (template String, VariableBindings)
                                     ↓ calls                    ↓
                               strings::phrase()     rlf_helper::eval_str() → rendered String
```

The serializer calls named RLF phrases but uses `{{...}}` escapes to produce literal template text (e.g., `"draw {cards($c)}."` with `{c: 3}` in bindings). The display layer's `eval_str()` resolves these against RLF definitions to produce final rendered text with colors and symbols.

### 2.2 Phase 2 Pipeline (Target)

```
Card TOML → Parser → Ability AST → Serializer → rendered String
                                     ↓ calls
                               strings::phrase(real_values) → Phrase → .to_string()
```

The serializer calls RLF phrase functions with **real values** (not phantom 0s) and receives fully-rendered `Phrase` objects. Intermediate serializers return `Phrase` to preserve metadata (tags, variants) for composition. The ability serializer calls `.to_string()` at the top level to produce the final display string.

**Key changes from Phase 1.5:**
- `SerializedAbility` loses its `variables` field — it just holds a `String`
- `eval_str()` is deleted — the serializer output IS the final display text
- `VariableBindings` is no longer threaded through serializer functions
- Category B phrases have `{{ }}` replaced with `{ }` — they now evaluate directly
- `FormattedText` is deleted — replaced by `Phrase` with `:a`/`:an` tags and `one`/`other` variants

### 2.3 Phrase Composition Strategy

The fundamental challenge: serializers currently compose text via `format!()` and string concatenation. In Phase 2, intermediate results are `Phrase` objects with metadata. Composition works at two levels:

**Level 1 — Phrase parameters (metadata-preserving):**
Predicate serializers return `Phrase` values that carry `:a`/`:an` tags and `one`/`other` variants. These are passed as `Value::Phrase(p)` arguments to consuming phrases. The consuming phrase can use `{@a $target}` to add the correct article, `{$target:other}` to select the plural form, or `:match($target)` for gender agreement. This is the critical path for localization.

```rust
// Predicate returns Phrase with :an tag and one/other variants
let target = predicate_serializer::serialize_predicate(pred);

// RLF phrase receives it as a Phrase value and can use @a, :other, :match
// when_you_play_trigger($target) = "when you play {@a $target}, ";
strings::when_you_play_trigger(Value::Phrase(target)).to_string()
```

**Level 2 — String concatenation (top-level assembly):**
The ability serializer assembles final text from already-rendered pieces using string concatenation. This is acceptable because: (a) the ability serializer is the top-level compositor where metadata propagation is complete, and (b) each assembled piece is itself an RLF phrase that translation files can reorder internally. For structural connectors (": ", ", then ", etc.) the ability serializer uses named phrases.

```rust
// Ability serializer concatenates rendered pieces
let trigger_text = strings::when_you_play_trigger(target_phrase).to_string();
let effect_text = strings::draw_cards_effect(count).to_string();
format!("{trigger_text}{effect_text}")
```

**Why not a single top-level phrase for each ability structure?**
Ability structures are highly variable (optional costs, optional once-per-turn, keyword vs non-keyword triggers, multiple cost slots, etc.). Expressing every combination as a single RLF phrase would require dozens of conditional phrases. String concatenation at the top level is simpler, and for languages that need to reorder trigger vs effect, the translation can restructure at the phrase level below.

### 2.4 FormattedText → Phrase Mapping

`FormattedText` currently provides five operations. Each maps directly to an RLF feature:

| FormattedText method | RLF equivalent | Example |
|---------------------|----------------|---------|
| `.with_article()` → `"a card"`, `"an ally"` | `{@a $phrase}` reads `:a`/`:an` tag | `ally = :an { one: "ally", other: "allies" };` |
| `.without_article()` → `"card"`, `"ally"` | Direct `{$phrase}` reference | `{$target}` |
| `.plural()` → `"cards"`, `"allies"` | Variant selection `{$phrase:other}` | `{$target:other}` |
| `.capitalized()` → `"Card"`, `"Ally"` | `{@cap $phrase}` transform | `{@cap $target}` |
| `.capitalized_with_article()` → `"A card"`, `"An ally"` | `{@cap @a $phrase}` | `{@cap @a $target}` |

The `FormattedText::new()` constructor auto-detects vowel sounds. In Phase 2, this is handled by `:a`/`:an` tags on the phrase definitions themselves. The `FormattedText::with_plural()` constructor for custom plurals maps to `one`/`other` variants on the phrase.

### 2.5 Predicate System Redesign

Currently, `serialize_predicate()` returns strings like `"a character"`, `"an enemy"`, `"{@a subtype($t)}"` — baking in the article and ownership context. In Phase 2, predicates return `Phrase` objects and the consuming phrase decides presentation:

**Current (Phase 1.5):**
```rust
// predicate_serializer returns pre-baked String
fn serialize_predicate(pred: &Predicate, bindings: &mut VariableBindings) -> String {
    match pred {
        Another(Character) => "an ally".to_string(),       // article baked in
        Enemy(Character) => "an enemy".to_string(),        // article baked in
        Your(CharacterType(t)) => {
            bindings.insert("t", Subtype(*t));
            "{@a subtype($t)}".to_string()                 // deferred to eval_str
        }
    }
}
```

**Phase 2:**
```rust
// predicate_serializer returns Phrase with metadata
fn serialize_predicate(pred: &Predicate) -> Phrase {
    match pred {
        Another(Character) => strings::ally(),             // Phrase with :an tag
        Enemy(Character) => strings::enemy(),              // Phrase with :an tag
        Your(CharacterType(t)) => {
            strings::subtype(subtype_phrase(*t))            // Phrase inherits tags via :from
        }
    }
}

// Consuming phrase applies article/plural as needed:
// dissolve_target($target) = "{@cap dissolve} {@a $target}.";
// → "Dissolve an ally." or "Dissolve an enemy Ancient."
```

The key insight: predicates no longer decide whether to include an article — they return a bare `Phrase` with metadata, and the consuming phrase template uses `{@a $target}` or `{$target}` or `{$target:other}` to control presentation. This is essential for localization because different languages need different article/case forms at different call sites.

### 2.6 Keyword Capitalization Strategy

Currently, `capitalize_first_letter()` handles two patterns:
1. Regular text: uppercase first character
2. Keywords in braces: `"{kindle($k)} ..."` → `"{Kindle($k)} ..."` with title-case logic for underscore-separated keywords

In Phase 2, keywords are always rendered (no more template `{keyword}` syntax). All keyword phrases in `strings.rs` are defined with their display formatting. Capitalization is handled by:
- RLF `@cap` transform for sentence-initial capitalization
- Keyword phrases are defined lowercase by convention; `@cap` is applied at the call site when needed

`capitalize_first_letter()` and `lowercase_leading_keyword()` are deleted. Any serializer function that capitalizes its output uses `@cap` in its phrase template instead.

---

## 3. Current State Inventory

### 3.1 Serializer Files

| File | Lines | Migration Status (Post Phase 1.5) |
|------|-------|----|
| `ability_serializer.rs` | 176 | Not migrated. Uses `strings::fast_prefix()` only. Hardcoded structural text. |
| `cost_serializer.rs` | 119 | Fully migrated to `strings::` phrases (Category B with `{{ }}`). |
| `trigger_serializer.rs` | 108 | Fully migrated (keyword arms stay as `format!` by design). |
| `condition_serializer.rs` | 99 | Fully migrated to `strings::` phrases. |
| `effect_serializer.rs` | 1138 | ~20 arms migrated; ~50 arms still use `format!()`. |
| `predicate_serializer.rs` | 802 | Not migrated. All 16 functions return hardcoded `String`. |
| `static_ability_serializer.rs` | 221 | Not migrated. Zero `strings::` usage. |
| `text_formatting.rs` | 78 | Not migrated. `FormattedText` to be replaced by `Phrase`. |
| `serializer_utils.rs` | 86 | `serialize_operator` migrated. `capitalize_first_letter`/`lowercase_leading_keyword` to be deleted. |

### 3.2 Display Layer Call Sites

All call sites follow the same pattern — serialize then eval:
- `card_rendering.rs`: `serialize_abilities_text()` and `ability_token_text()`
- `dreamwell_card_rendering.rs`: ability rendering
- `modal_effect_prompt_rendering.rs`: modal choice text

### 3.3 strings.rs Phrase Inventory

**225 total phrases** defined in `rlf!` macro:
- **175 Category A** (no `{{ }}`): Final phrases that survive into Phase 2 unchanged
- **50 Category B** (contain `{{ }}`): Temporary phrases whose `{{ }}` escapes will be removed

### 3.4 Types That Need PartialEq

`Ability` derives `Debug, Clone, Serialize, Deserialize` but **not** `PartialEq`. The inner types (`EventAbility`, `TriggeredAbility`, `ActivatedAbility`, `StaticAbility`) do derive `PartialEq, Eq`, but `NamedAbility` and `Effect` do not. Adding `PartialEq` to the full AST tree is a prerequisite for AST-level round-trip tests.

---

## 4. Cross-Serializer Dependency Graph

```
ability_serializer
  ├── trigger_serializer
  │     └── predicate_serializer
  ├── cost_serializer
  │     └── predicate_serializer
  ├── effect_serializer
  │     ├── predicate_serializer
  │     ├── cost_serializer
  │     ├── condition_serializer
  │     │     └── predicate_serializer
  │     ├── trigger_serializer
  │     ├── static_ability_serializer  ←──┐
  │     │     ├── predicate_serializer    │
  │     │     ├── cost_serializer         │  CIRCULAR
  │     │     ├── condition_serializer    │
  │     │     ├── effect_serializer  ─────┘
  │     │     └── text_formatting
  │     ├── text_formatting
  │     └── serializer_utils
  ├── serializer_utils
  └── static_ability_serializer
```

**Circular dependency:** `effect_serializer` calls `static_ability_serializer::serialize_standard_static_ability()`. In return, `static_ability_serializer` calls `effect_serializer::serialize_for_count_expression()` and `effect_serializer::serialize_effect()`. These must be migrated together.

**Migration order constraint:** `predicate_serializer` + `text_formatting` must be migrated first (they're leaves). Then the mid-level serializers (cost, trigger, condition, effect + static_ability together). Finally `ability_serializer` at the top.

---

## 5. Validation Protocol

After **every single task**:

```bash
just review    # clippy + style validator + ALL tests
```

### 5.1 Round-Trip Test Evolution

**Phase 2 begins** with text-equality round-trip tests (current). After Task 1 adds AST-level tests, **both strategies run in parallel**. After Task 3 proves AST tests are reliable, text-equality tests are removed, unblocking serializer output format changes.

### 5.2 Test Coverage

- `cards_toml_round_trip_tests` — serializes every card in the game (byte-for-byte or AST-level)
- `dreamwell_toml_round_trip_tests` — same for dreamwell cards
- Unit round-trip tests in 6 files — individual ability patterns
- `card_effect_parser_tests.rs` — insta snapshot tests for parser AST

---

## 6. Task Breakdown

### Task 1: AST-Level Round-Trip Tests

**Files:** `ability_data` types, `test_helpers.rs`, new test file
**Risk:** HIGH — this is the single point of failure for all subsequent work.

#### Step 1: Add PartialEq to the Ability AST

Add `PartialEq, Eq` to all types in the ability AST tree that don't already have it. The types that need it:
- `Ability` (ability.rs) — currently `Debug, Clone, Serialize, Deserialize`
- `NamedAbility` (named_ability.rs) — currently `Debug, Clone, Serialize, Deserialize`
- `Effect` (effect.rs) — currently `Debug, Clone, Serialize, Deserialize`
- Check and add to any nested types that are missing it

These are simple `derive` additions. If any type contains `f32`/`f64` (unlikely for a card game AST), use `PartialEq` with appropriate handling.

#### Step 2: Add `assert_ast_round_trip` helper

Add to `test_helpers.rs`:

```rust
pub fn assert_ast_round_trip(input_text: &str, vars: &str) {
    let parsed = parse_ability(input_text, vars);
    let serialized = ability_serializer::serialize_ability(&parsed);
    let reparsed = parse_ability(&serialized.text, &serialized.variables.to_string());
    assert_eq!(parsed, reparsed,
        "AST mismatch for input: {input_text}\n  serialized: {}\n  vars: {}",
        serialized.text, serialized.variables
    );
}
```

#### Step 3: Add parallel AST round-trip test suite

For every existing `assert_round_trip(text, vars)` call, add a corresponding `assert_ast_round_trip(text, vars)`. Run both in the same test function. The existing text-equality test catches output format changes (catches regressions now); the AST test catches semantic equivalence (needed for Phase 2 when output format changes).

#### Step 4: Add AST-level comparison to TOML bulk tests

In `cards_toml_round_trip_tests.rs` and `dreamwell_toml_round_trip_tests.rs`, add AST comparison alongside the existing text comparison. Both must pass.

#### Step 5: Validate and commit

---

### Task 2: Predicate Serializer + FormattedText → Phrase

**Files:** `predicate_serializer.rs`, `text_formatting.rs`, `strings.rs`
**Risk:** HIGH — 802 lines, 16 functions, ~80 call sites across all other serializers.
**Prerequisite:** Task 1

This is the hardest single step. It changes the predicate serializer from returning `String` to returning `Phrase`, replacing `FormattedText` with RLF metadata.

#### Step 1: Add predicate noun phrases to strings.rs

Define RLF terms for all composite predicate noun phrases that `FormattedText` currently constructs. These carry `:a`/`:an` tags and `one`/`other` variants for article and plural selection:

```rust
// =========================================================================
// Predicate Noun Terms
// =========================================================================

// Ownership-qualified character terms
your_card = :a :inan { one: "your card", other: "your cards" };
your_event = :an :inan { one: "your event", other: "your events" };
enemy_card = :an :inan { one: "enemy card", other: "enemy cards" };
enemy_event = :an :inan { one: "enemy event", other: "enemy events" };

// The "ally" and "enemy" terms already exist with proper tags.
// SubType terms use :from to inherit tags from the subtype Phrase.

allied_subtype($t) = :from($t) { one: "allied {$t}", other: "allied {$t:other}" };
enemy_subtype($t) = :from($t) { one: "enemy {$t}", other: "enemy {$t:other}" };
```

The exact set of phrases depends on which `CardPredicate` arms exist. Each arm in `your_predicate_formatted` and `enemy_predicate_formatted` maps to a phrase.

#### Step 2: Refactor predicate_serializer return types

Change all public functions to return `Phrase` instead of `String`:
- `serialize_predicate()` → `Phrase`
- `serialize_predicate_plural()` → `Phrase` (or `String` since it's always the plural text)
- `predicate_base_text()` → `Phrase`
- `serialize_card_predicate()` → `Phrase`
- All other public functions

The private `your_predicate_formatted()` and `enemy_predicate_formatted()` currently return `FormattedText`. Change them to return `Phrase` directly (the RLF term already carries article tags and plural variants).

#### Step 3: Update all call sites

This is the bulk of the work (~80 call sites). Each call site currently does one of:
1. Passes predicate `String` to a `strings::` phrase as `&str` → change to pass `Value::Phrase(p)`
2. Uses predicate `String` in `format!()` → change to use `p.to_string()` for now, or create a new phrase
3. Calls `.with_article()` / `.without_article()` / `.plural()` on the result → use `@a`, direct ref, or `:other` in the phrase template

**Key pattern change for consuming phrases:**

Category A phrases like `when_you_play_trigger($target)` currently take `&str`. They need to accept `Value` (Phrase) instead so they can use `{@a $target}` etc. This means changing the `rlf!` parameter types.

Category B phrases with `$target` inside `{{ }}` must also be updated — the `$target` is now a real `Phrase` value, not a pre-rendered string.

#### Step 4: Delete text_formatting.rs

Remove the `FormattedText` struct and `card_predicate_base_text()` function. All their functionality is now provided by RLF phrases with proper tags and variants.

Update `mod.rs` to remove the module declaration.

#### Step 5: Validate and commit

**Critical:** Both text-equality AND AST round-trip tests must pass. The serializer output text may change (e.g., article selection may differ slightly if tag-based `@a` produces different results than vowel-detection). The AST test ensures semantic equivalence is preserved.

---

### Task 3: Remove Text-Equality Round-Trip Tests

**Files:** `test_helpers.rs`, all round-trip test files, TOML round-trip tests
**Risk:** MEDIUM — must be confident AST tests are sufficient.
**Prerequisite:** Task 2 (proves AST tests catch real issues)

#### Step 1: Verify AST test coverage

Review all existing text-equality tests. For each one, confirm the AST test would catch the same failure. Check edge cases: empty abilities, abilities with multiple effects, modal choices, named abilities.

#### Step 2: Remove text-equality assertions

In `assert_round_trip`, remove the `assert_eq!(expected_text, serialized.text)` and `VariableBindings` comparison. Replace with just the AST comparison.

In TOML bulk tests, remove the byte-for-byte text comparison. Keep only AST-level comparison.

#### Step 3: Simplify SerializedAbility usage in tests

Test helpers no longer need to compare `VariableBindings`. Simplify `assert_round_trip` to:

```rust
pub fn assert_round_trip(input_text: &str, vars: &str) {
    let parsed = parse_ability(input_text, vars);
    let serialized = ability_serializer::serialize_ability(&parsed);
    // Re-parse the serialized output and compare ASTs
    let reparsed = parse_from_rendered(&serialized.text);
    assert_eq!(parsed, reparsed);
}
```

Note: `parse_from_rendered` must handle the fact that serializer output is now **rendered** text (with HTML color tags etc.) rather than template text. The parser needs to handle this, OR the serializer provides both a rendered string and a parseable string. See Design Decision 3.1 below.

#### Step 4: Validate and commit

---

### Task 4: Remove `{{ }}` Escapes and VariableBindings from Leaf Serializers

**Files:** `strings.rs`, `cost_serializer.rs`, `trigger_serializer.rs`, `condition_serializer.rs`, `serializer_utils.rs`, simple `effect_serializer.rs` arms
**Risk:** LOW — straightforward mechanical changes.
**Prerequisite:** Task 3

#### Step 1: Rewrite Category B phrases in strings.rs

For every phrase containing `{{ }}`, remove the escape braces so RLF evaluates directly:

```rust
// Before (Phase 1.5):
draw_cards_effect($c) = "draw {{cards($c)}}.";

// After (Phase 2):
draw_cards_effect($c) = "draw {cards($c)}.";
```

All ~50 Category B phrases are rewritten. The phantom parameter problem is eliminated — `$c` is now actually evaluated by RLF.

#### Step 2: Pass real values to phrase functions

In each serializer, replace the pattern:
```rust
bindings.insert("c".to_string(), VariableValue::Integer(count));
strings::draw_cards_effect(0).to_string()   // 0 is phantom
```
with:
```rust
strings::draw_cards_effect(count).to_string()   // real value
```

Remove all `bindings.insert()` calls for leaf serializers (cost, trigger, condition, simple effects). The `bindings: &mut VariableBindings` parameter is still passed through but no longer written to by these functions.

#### Step 3: Update phrases that take $target

Phrases like `when_you_play_trigger($target)` currently receive pre-rendered `String` targets. After Task 2, they receive `Phrase` values. Update the phrase function calls to pass `Value::Phrase(target_phrase)` where the predicate serializer now returns a `Phrase`.

#### Step 4: Validate and commit

---

### Task 5: Migrate Remaining Effect Serializer Arms

**Files:** `strings.rs`, `effect_serializer.rs`
**Risk:** MEDIUM — large file, many arms.
**Prerequisite:** Task 4

#### Step 1: Add phrases for all remaining StandardEffect arms

Define RLF phrases for all ~50 unmigrated `StandardEffect` arms. These arms now call the predicate serializer which returns `Phrase`, so `$target` parameters are `Phrase` values:

```rust
// Examples of new phrases taking Phrase targets:
dissolve_target_effect($target) = "{@cap dissolve} {@a $target}.";
banish_target_effect($target) = "{@cap banish} {@a $target}.";
gain_control_effect($target) = "gain control of {@a $target}.";
draw_for_each_effect($c, $for_each) = "draw {cards($c)} for each {$for_each}.";
materialize_target_effect($target) = "{@cap materialize} {@a $target}.";
```

#### Step 2: Migrate effect_serializer.rs arms

Replace all remaining `format!()` arms with `strings::` phrase calls. Key groups:

**Predicate-consuming arms** (~30): `DissolveCharacter`, `BanishCharacter`, `GainControl`, `MaterializeCharacter`, `Discover`, `ReturnToHand`, `Counterspell` (all variants), etc. These now pass `Phrase` values from the predicate serializer to new RLF phrases.

**For-each pattern arms** (~8): `DrawCardsForEach`, `GainEnergyForEach`, `GainPointsForEach`, etc. These combine a count phrase with a for-each predicate phrase.

**Spark-related arms** (~6): `GainsSpark`, `EachMatchingGainsSpark`, `SparkBecomes`, etc.

**Collection arms** (~10): `DissolveCharactersCount`, `BanishCollection`, `MaterializeCollection`, `MaterializeSilentCopy`, etc. These have nested `CollectionExpression` matches.

#### Step 3: Migrate `serialize_for_count_expression`

This public function (15 arms) returns quantity descriptions used by multiple serializers. Migrate all arms to use RLF phrases. Since it's called by `static_ability_serializer`, coordinate with Task 6.

#### Step 4: Migrate helper functions

- `serialize_gains_reclaim` (~116 lines) — complex but self-contained
- `serialize_void_gains_reclaim` (8 CollectionExpression arms)
- `serialize_allied_card_predicate` / `serialize_allied_card_predicate_plural` (2 arms each)

#### Step 5: Migrate `serialize_effect_with_context` structural logic

The `Effect::WithOptions`, `Effect::List` (4 branches), `Effect::ListWithOptions`, and `Effect::Modal` arms. These use structural connector phrases already defined in strings.rs (`then_joiner`, `and_joiner`, `period_suffix`, `you_may_prefix`, etc.).

#### Step 6: Validate and commit

---

### Task 6: Migrate Static Ability Serializer (Atomic with Effect Serializer)

**Files:** `strings.rs`, `static_ability_serializer.rs`
**Risk:** MEDIUM — circular dependency with effect_serializer.
**Prerequisite:** Task 5 (or done concurrently as a single atomic step)

**This task MUST be coordinated with Task 5** due to the circular dependency:
- `effect_serializer` calls `static_ability_serializer::serialize_standard_static_ability()`
- `static_ability_serializer` calls `effect_serializer::serialize_for_count_expression()` and `effect_serializer::serialize_effect()`

#### Step 1: Add static ability phrases to strings.rs

Define phrases for all ~20 `StandardStaticAbility` variants. These include conditional text placement logic (condition prepended vs appended).

#### Step 2: Migrate `serialize_standard_static_ability`

Replace all `format!()` arms with `strings::` phrase calls. Handle the condition placement logic: for `StaticAbility::WithOptions`, conditions can be prepended (`"if this card is in your void, ..."`) or appended (`"... with allied subtype"`) depending on the condition type. This logic stays in Rust code, calling different phrases for each layout variant.

#### Step 3: Migrate `serialize_static_ability`

Handle `StaticAbility::StaticAbility` (simple delegation) and `StaticAbility::WithOptions` (condition+ability composition).

#### Step 4: Validate and commit

---

### Task 7: Migrate Ability Serializer

**Files:** `ability_serializer.rs`, `strings.rs`
**Risk:** LOW — top-level orchestrator, straightforward once everything below returns Phrase.
**Prerequisite:** Tasks 5 and 6

#### Step 1: Add ability structure phrases

```rust
// Ability-level structural phrases
triggered_ability_keyword_effect($trigger, $effect) = "{$trigger} {@cap $effect}";
activated_cost_separator = ": ";
activated_once_per_turn_suffix = ", once per turn";
```

Most structural phrases already exist from Phase 1.5 (`until_end_of_turn_prefix`, `once_per_turn_prefix`, `fast_prefix`, `cost_effect_separator`).

#### Step 2: Refactor ability_serializer.rs

Replace string concatenation with phrase calls. The key change: instead of calling `serializer_utils::capitalize_first_letter()`, use `@cap` in phrase templates.

For `Ability::Triggered`: compose trigger + effect, with proper capitalization handled by phrases.
For `Ability::Activated`: compose costs + separator + effect, with structural phrases.
For `Ability::Event`: capitalize and return effect.
For `Ability::Named`: handle Reclaim variants with proper phrases.
For `Ability::Static`: capitalize and return static ability.

#### Step 3: Update `serialize_named_ability`

Currently produces template text like `"{Reclaim_For_Cost($r)}"`. Change to call the phrase directly with the real cost value.

#### Step 4: Validate and commit

---

### Task 8: Remove eval_str and VariableBindings

**Files:** `ability_serializer.rs`, `rlf_helper.rs`, `card_rendering.rs`, `dreamwell_card_rendering.rs`, `modal_effect_prompt_rendering.rs`, `parser_bindings.rs`
**Risk:** LOW — straightforward cleanup once all serializers produce final text.
**Prerequisite:** Task 7

**This is an atomic switchover.** All display layer call sites iterate over abilities with a single code path. Once the serializer produces final rendered text, all call sites must stop calling `eval_str()` simultaneously.

#### Step 1: Change SerializedAbility

```rust
// Before:
pub struct SerializedAbility {
    pub text: String,
    pub variables: VariableBindings,
}

// After:
pub struct SerializedAbility {
    pub text: String,  // Now contains final rendered text, not template text
}
```

#### Step 2: Remove VariableBindings from serializer functions

Remove the `bindings: &mut VariableBindings` parameter from all serializer function signatures. This is a large mechanical change touching every function. Remove `VariableBindings::new()` creation in `serialize_ability()`.

#### Step 3: Update display layer call sites

In `card_rendering.rs`, `dreamwell_card_rendering.rs`, and `modal_effect_prompt_rendering.rs`, change:
```rust
// Before:
let serialized = ability_serializer::serialize_ability(ability);
rlf_helper::eval_str(&serialized.text, &serialized.variables)

// After:
ability_serializer::serialize_ability(ability).text
```

#### Step 4: Delete eval_str and build_params

Delete `rlf_helper::eval_str()` and `rlf_helper::build_params()`. The `subtype_phrase()` and `figment_phrase()` functions in `rlf_helper.rs` are still needed (they're used by the predicate serializer now), so keep them.

#### Step 5: Remove VariableBindings type

If `VariableBindings` and `VariableValue` are no longer used anywhere, delete them. Check for any remaining references outside the serializer (e.g., `parser_substitutions` may use `VariableBindings` for parsing — keep that if so, but remove the serializer's dependency on it).

#### Step 6: Update test infrastructure

Switch to the dual-path rendered output comparison strategy (Section 7.1):

```rust
pub fn assert_rendered_match(input_text: &str, vars: &str) {
    // Path A: parse → serialize → rendered string
    let parsed = parse_ability(input_text, vars);
    let serialized = ability_serializer::serialize_ability(&parsed);
    // Path B: evaluate template text directly through RLF
    let rendered = rlf_eval(input_text, vars);
    assert_eq!(serialized.text, rendered);
}
```

Keep `rlf_eval` as a test-only helper even after removing it from production code. It serves as an independent oracle for verifying serializer output.

#### Step 7: Validate and commit

---

### Task 9: Remove Capitalization Helpers

**Files:** `serializer_utils.rs`
**Risk:** LOW — straightforward deletion.
**Prerequisite:** Task 8

#### Step 1: Verify no remaining callers

Grep for `capitalize_first_letter` and `lowercase_leading_keyword` across the codebase. After Tasks 7-8, there should be zero callers.

#### Step 2: Delete the functions

Remove `capitalize_first_letter`, `capitalize_string`, `title_case_keyword`, `is_capitalizable_keyword`, and `lowercase_leading_keyword` from `serializer_utils.rs`.

#### Step 3: Validate and commit

---

## 7. Design Decisions

### 7.1 Testing Strategy: Dual-Path Rendered Output Comparison

The serializer does not need to support round-trip testing (parse → serialize → re-parse). Instead, we compare the **final rendered English output** of two independent paths:

```
Path A: input_string → parse() → AST → serialize() → rendered string
Path B: input_string → rlf_eval()                   → rendered string
Assert: Path A == Path B
```

Both paths start from the same input string (template text with variables) and should produce identical final rendered text (with HTML color tags, Unicode symbols, etc.). This validates that the serializer produces correct rendered output without needing to re-parse rendered text.

**How this works at each phase:**

- **Before Task 8** (eval_str still exists): Path B is `rlf_helper::eval_str(input_string, vars)`. Path A is `serialize(parse(input_string, vars)).text`. Both should produce the same rendered string.
- **After Task 8** (eval_str removed): Path B can be preserved as a test-only helper that evaluates template text through RLF directly. It doesn't need to exist in production code.

**What this replaces:** The current text-equality round-trip test (`input == serialize(parse(input)).text`) compares template text. The new strategy compares rendered text from two independent paths, which is strictly more powerful — it validates both parse/serialize correctness AND rendered output correctness simultaneously.

**Task 3 impact:** When text-equality tests are removed, they're replaced by this dual-path comparison. The AST-level tests from Task 1 provide a second safety net: `parse(input) == parse(serialize(parse(input)))` at the AST level (once serializer output is parseable template text; before Task 8). After Task 8, the dual-path rendered comparison is the primary test strategy.

### 7.2 Phrase Parameter Types

Currently, `rlf!` macro-generated functions accept specific Rust types (integers → `i64`, strings → `&str`). For Phase 2, predicate parameters need to accept `Phrase` values (via `Value::Phrase(p)`). Verify that the `rlf!` macro supports this, or determine how to pass `Value` enum variants to phrase functions.

If the macro-generated functions only accept primitives, an alternative is to call `locale.eval_str()` with `Value::Phrase` in the params map, or to extend the `rlf!` macro to support Phrase parameters.

### 7.3 register_source_phrases() Call Site

Currently, `rlf_helper::eval_str()` calls `strings::register_source_phrases()` on every invocation. In Phase 2, the serializer calls `strings::` functions directly. Ensure phrase registration happens before serialization. Options:
- Call `register_source_phrases()` once at serializer initialization
- The `rlf!` macro handles registration lazily on first phrase access
- Move registration to a global init that runs before any serialization

---

## 8. Risk Assessment

| Task | Risk | Reason |
|------|------|--------|
| 1. AST round-trip tests | HIGH | Single point of failure for all subsequent work |
| 2. Predicate → Phrase | HIGH | 802 lines, 16 functions, ~80 call sites, entangled with FormattedText |
| 3. Remove text tests | MEDIUM | Must prove AST tests are sufficient first |
| 4. Remove `{{ }}` escapes | LOW | Mechanical changes to leaf serializers |
| 5. Remaining effect arms | MEDIUM | Large file (~50 arms), but each arm is self-contained |
| 6. Static ability serializer | MEDIUM | Circular dependency with effect_serializer |
| 7. Ability serializer | LOW | Straightforward top-level orchestration |
| 8. Remove eval_str | MEDIUM | Atomic switchover, round-trip test strategy change |
| 9. Remove capitalization | LOW | Simple deletion |
| 10. Animacy tags + `:from` | LOW | Additive changes only, no English behavioral impact |
| 11. RLF framework changes | LOW | Independent of Dreamtides code, follows established patterns |

---

## 9. Multilingual Design Considerations

These were identified by i18n stress testing across all 6 target languages. They inform the Rust code structure even though we're not writing translations yet.

### 9.1 Case Declension (Russian, German)

Russian: 6 cases × 3 CLDR plural categories = 18 forms per noun. German: 4 cases × 2 numbers. RLF handles this via multi-dimensional variants with wildcard fallbacks:

```
// ru.rlf
card = :fem :inan {
    nom: "карта", nom.few: "карты", nom.many: "карт",
    acc: "карту", acc.few: "карты", acc.many: "карт",
};
```

**Rust code implication:** Predicate phrases must accept `Phrase` values (not strings) so translation files can apply case selectors like `{$target:acc:$n}`.

### 9.2 Gender Agreement (Russian, Spanish, Portuguese, German)

"when X is dissolved" requires participle agreement with X's gender. Handled by `:match` on gender tags:

```
// ru.rlf
when_dissolved_trigger($target) = :match($target) {
    masc: "когда {$target:nom} растворён, ",
    fem: "когда {$target:nom} растворена, ",
    *neut: "когда {$target:nom} растворено, ",
};
```

**Rust code implication:** Predicate `Phrase` values must carry gender tags (`:masc`/`:fem`/`:neut`). English source terms carry animacy (`:anim`/`:inan`); translation files add gender.

### 9.3 Personal "a" (Spanish)

"dissolve an enemy" → "disolver **a** un enemigo". Handled by `:match` on `:anim` tag:

```
// es.rlf
dissolve_target($target) = :match($target) {
    anim: "{@cap dissolve} a {@un $target:acc}.",
    *inan: "{@cap dissolve} {@un $target:acc}.",
};
```

**Rust code implication:** English terms need `:anim`/`:inan` tags (to be added in Task 10; not yet present in `strings.rs`).

### 9.4 Chinese Classifiers and Word Order

Different classifiers per noun (张 for cards, 个 for characters). Different word order ("from void banish" not "banish from void"):

```
// zh.rlf
draw_cards_effect($c) = "抽{cards($c)}。";
cards($n) = :match($n) { 1: "一张牌", *other: "{$n}张牌" };
```

**Rust code implication:** Each phrase defines its own word order in the translation file. The Rust code passes the same parameters regardless of language.

### 9.5 German Separable Verbs

"auflösen" (dissolve) splits: "Löse ... auf". Handled entirely in translation file:

```
// de.rlf
dissolve_target($target) = "Löse {@ein:acc $target} auf.";
```

### 9.6 Tag System Design

| Tag | Source | Purpose |
|-----|--------|---------|
| `:a` / `:an` | English source | English indefinite article selection |
| `:anim` / `:inan` | English source | Cross-language animacy (Spanish personal "a", Russian acc=gen) |
| `:masc` / `:fem` / `:neut` | Translation files | Language-specific gender agreement |
| `:zhang` / `:ge` / `:ming` etc. | Chinese translation | Chinese classifier tags for `@count` |

**Animacy tag requirements (Task 2):** All entity noun terms in `strings.rs` MUST carry `:anim` or `:inan` tags. This is critical for Spanish (personal "a" before animate direct objects) and Russian (masculine animate accusative = genitive). Specifically:
- `:anim` on: `ally`, all character subtypes (agent, ancient, avatar, child, detective, dreamer, explorer, mystic, ruler, scholar, spy, trickster, warrior), `character`
- `:inan` on: `card`, `event`, figment types, location-like terms

**Compound phrase `:from` requirement (Task 2):** Compound phrases that wrap entity references MUST use `:from($entity)` to preserve tag propagation through the composition chain. Without this, downstream `:match` for gender/animacy breaks. Specifically:
```rust
allied($entity) = :from($entity) "allied {$entity:one}";
enemy_modified($entity) = :from($entity) "enemy {$entity:one}";
```

**`:from` tag propagation guarantee:** `:from($param)` MUST propagate ALL tags unconditionally from the parameter. This includes `:a`/`:an`, `:anim`/`:inan`, and any language-specific tags (`:masc`/`:fem`/`:neut`, classifier tags). This is a hard requirement — verify and document as a guarantee.

### 9.7 RLF Feature Verification Checklist

Before writing translation files, verify these RLF features are fully implemented:
- [ ] `@count` transform for CJK classifiers
- [ ] `@count:word` modifier for CJK number words (一/两/三 instead of 1/2/3) — see Section 9.8
- [ ] `@der`/`@ein` for German articles + case
- [ ] `@ein` returns empty string for plural context (German has no plural indefinite article)
- [ ] `@el`/`@un` for Spanish articles
- [ ] `@del`/`@al` for Spanish preposition+article contractions (de+el=del, a+el=al)
- [ ] `@o`/`@um` for Portuguese articles
- [ ] `@um:other` produces plural indefinite articles (uns/umas) for Portuguese
- [ ] `@para` (a+article contraction ao/à/aos/às) for Portuguese
- [ ] `@por` (por+article contraction pelo/pela/pelos/pelas) for Portuguese
- [ ] Multi-parameter `:match` (e.g., `:match($n, $entity)`)
- [ ] `:from` with multi-dimensional variant propagation (4 cases × 2 numbers for German)
- [ ] `:from` propagates ALL tags unconditionally (`:a`/`:an`, `:anim`/`:inan`, gender, classifiers)
- [ ] Phrase parameter types in `rlf!` macro-generated functions
- [ ] `@cap` is a no-op on CJK characters (Chinese/Japanese/Korean)
- [ ] `@cap` skips leading HTML markup tags to find first visible character

### 9.8 RLF Framework Changes Required

These changes to the RLF framework (`rlf` crate) were identified by the 5-language i18n review. See `docs/plans/i18n-review-*.md` for full analysis. Changes are ordered by priority.

#### 9.8.1 Portuguese `@para` Transform — HIGH

Add preposition "a" + article contraction (ao/à/aos/às). Needed for dynamic "to X" phrases like "devolva à mão" (return to the hand). ~30 lines following the exact pattern of existing `@de` and `@em` transforms. See `i18n-review-pt-br.md` Section 14.1.

#### 9.8.2 Portuguese `@por` Transform — MEDIUM

Add preposition "por" + article contraction (pelo/pela/pelos/pelas). Needed for "by/for each" patterns. ~30 lines. See `i18n-review-pt-br.md` Section 14.2.

#### 9.8.3 Portuguese `@um` Plural Context — MEDIUM

Enhance `@um` to accept `:other` context for plural indefinite articles (uns/umas). Currently only produces singular (um/uma). ~10 lines. See `i18n-review-pt-br.md` Section 14.3.

#### 9.8.4 Chinese `@count:word` Modifier — MEDIUM

Add a `:word` context modifier to `@count` that substitutes CJK number words (一/两/三) instead of digits (1/2/3) for small numbers (1-10). Falls back to digits for larger numbers. Without this, Chinese translators must use verbose `:match` wrappers for every count phrase. ~30 lines per CJK language. See `i18n-review-zh.md` Recommendation 1.

#### 9.8.5 German `@ein` Empty Plural — LOW

Define `@ein` behavior for plural context: return empty string. German has no plural indefinite article. Currently undefined behavior. ~5 lines. See `i18n-review-de.md` Recommendation 2.

#### 9.8.6 Spanish `@del`/`@al` Contractions — LOW

Add preposition+article contraction transforms: "de"+"el"="del", "a"+"el"="al". Only applies to masculine singular definite article. Low priority because most card text uses possessives ("tu") not articles for locations. ~20 lines each. See `i18n-review-es.md` RFC-ES-3/RFC-ES-7.

#### 9.8.7 Changes NOT Recommended

The following were evaluated and rejected by the i18n review:
- **Classifiers as first-class concept** (Chinese) — tags + `@count` already handle this
- **`@sep` for separable verbs** (German) — phrase templates handle this perfectly
- **`@part` for participle agreement** (all gendered languages) — `:match` is simpler and more flexible
- **`@kein` negative article** (German) — too few use cases, inline works
- **Verb conjugation transforms** (Portuguese, Spanish) — only ~15 fixed verbs in the game
- **Implicit `:from` on compound phrases** — explicit is better, avoid ambiguity with multi-parameter phrases
- **Language-specific `:from` behavior** — `:from` is correctly language-neutral by design
- **Declension context on `@der`/`@ein`** (German) — pragmatic alternative (hardcoded adjective forms per known context) works for Dreamtides domain

### 9.9 Cross-Language Design Conventions

These conventions were identified by the i18n review and MUST be followed during Phase 2:

1. **Effect phrases MUST NOT include trailing periods.** The period is added by `period_suffix` at the assembly level. This prevents double punctuation when effects are joined by `then_joiner`. (German agent: separable verb prefix must come before the period/connector.)

2. **Each phrase MUST control its own capitalization via `@cap`.** The Rust code MUST NOT apply `capitalize_first_letter()` to already-rendered strings. The ability serializer concatenates pre-capitalized pieces. (German agent: V2 word order requires verb-initial effects; Chinese agent: `@cap` is a no-op on CJK.)

3. **Keyword terms MUST have separate imperative and participial forms.** `dissolve` (imperative, for effects) and `dissolved` (participle, for triggers) are distinct terms. Spanish/Portuguese need different verb forms; Russian/German need different gender-agreeing participles. (Spanish agent: BLOCK-ES-2.)

4. **Structural connectors MUST be named RLF phrases.** All punctuation, separators, and joiners used by `ability_serializer.rs` go through named phrases: `then_joiner`, `and_joiner`, `period_suffix`, `cost_effect_separator`, `once_per_turn_prefix`, `until_end_of_turn_prefix`, etc. (Chinese agent: full-width punctuation; Russian agent: em-dash separator convention.)

5. **`text_number` is English-specific.** Translators for gendered languages (Spanish, Portuguese, German) should inline number words in their phrase templates using `:match`, not call `text_number`. Only "1" varies by gender in most languages.

---

### Task 10: Add Animacy Tags and `:from` to Compound Phrases

**Files:** `strings.rs`
**Risk:** LOW — additive changes only, no behavioral impact on English.
**Prerequisite:** Task 2 (predicate phrases must exist before tagging them)

This task implements the animacy tagging and compound phrase `:from` propagation identified by the 5-language i18n review.

#### Step 1: Add `:anim`/`:inan` tags to all entity terms

Every noun term in `strings.rs` that represents a game entity needs an animacy tag:

```rust
// Character types — animate
ally = :an :anim { one: "ally", other: "allies" };
agent = :an :anim { one: "Agent", other: "Agents" };
ancient = :an :anim { one: "Ancient", other: "Ancients" };
avatar = :an :anim { one: "Avatar", other: "Avatars" };
child = :a :anim { one: "Child", other: "Children" };
detective = :a :anim { one: "Detective", other: "Detectives" };
dreamer = :a :anim { one: "Dreamer", other: "Dreamers" };
explorer = :an :anim { one: "Explorer", other: "Explorers" };
mystic = :a :anim { one: "Mystic", other: "Mystics" };
ruler = :a :anim { one: "Ruler", other: "Rulers" };
scholar = :a :anim { one: "Scholar", other: "Scholars" };
spy = :a :anim { one: "Spy", other: "Spies" };
trickster = :a :anim { one: "Trickster", other: "Tricksters" };
warrior = :a :anim { one: "Warrior", other: "Warriors" };
character = :a :anim { one: "character", other: "characters" };

// Non-character types — inanimate
card = :a :inan { one: "card", other: "cards" };
event = :an :inan { one: "event", other: "events" };
// figment types also :inan
```

English doesn't read these tags, so this is purely additive. The tags enable Spanish `:match($target) { anim: "... a ...", *inan: "..." }` and Russian masculine animate accusative=genitive.

#### Step 2: Add `:from($entity)` to compound predicate phrases

Ensure tag propagation through the composition chain:

```rust
allied($entity) = :from($entity) "allied {$entity:one}";
enemy_modified($entity) = :from($entity) "enemy {$entity:one}";
allied_subtype($t) = :from($t) { one: "allied {$t}", other: "allied {$t:other}" };
enemy_subtype($t) = :from($t) { one: "enemy {$t}", other: "enemy {$t:other}" };
```

Without `:from`, downstream phrases that do `:match(allied_result)` for gender/animacy would fail because tags are lost in the composition.

#### Step 3: Verify keyword terms have separate imperative/participial forms

Audit all keyword terms to ensure imperative (effect text) and participial (trigger text) uses are separate terms:
- `dissolve` (imperative) + `dissolved` (participle) ✓
- `banish` (imperative) + `banished` (participle) ✓
- `materialize`, `reclaim`, `prevent`, `discover` — verify participial forms exist if used in triggers

#### Step 4: Validate and commit

---

### Task 11: RLF Framework Changes for Localization

**Files:** RLF crate (`~/rlf/`) — `transforms.rs`, stdlib definitions
**Risk:** LOW — additive changes following established patterns.
**Prerequisite:** None (can be done in parallel with Phase 2 tasks)

This task implements the RLF framework changes identified by the 5-language i18n review (Section 9.8).

#### Step 1: Add Portuguese `@para` transform (HIGH)

Add "a" + article contraction (ao/à/aos/às) following the exact pattern of existing `@de` and `@em` transforms. Register under `("pt", "para")` with alias `@ao`. ~30 lines.

#### Step 2: Add Portuguese `@por` transform (MEDIUM)

Add "por" + article contraction (pelo/pela/pelos/pelas). Same pattern. Register under `("pt", "por")`. ~30 lines.

#### Step 3: Enhance Portuguese `@um` for plural context (MEDIUM)

Modify `portuguese_um_transform` to accept context parameter, producing "uns"/"umas" for `:other` context. ~10 lines.

#### Step 4: Add `@count:word` modifier for CJK (MEDIUM)

Add a `:word` context to the Chinese `@count` transform that substitutes CJK number words (一/两/三/四/五/六/七/八/九/十) for small numbers (1-10), falling back to digits for larger numbers. Uses 两 (not 二) for counting form of "two". ~30 lines.

#### Step 5: Define German `@ein` plural behavior (LOW)

Specify that `@ein` with `.other` (plural) context returns empty string. Prevents undefined behavior. ~5 lines.

#### Step 6: Add Spanish `@del`/`@al` contractions (LOW)

Add "de"+"el"="del" and "a"+"el"="al" contraction transforms. Only contract with masculine singular definite article. ~20 lines each.

#### Step 7: Update APPENDIX_STDLIB.md documentation

- Document `@count:word` modifier and the canonical `:match` fallback pattern for CJK
- Document `@para`, `@por` in Portuguese transform table
- Document `@um:other` plural behavior
- Document `@ein` empty-string plural behavior
- Document `@del`/`@al` in Spanish transform table
- Add note that `text_number` is English-specific; gendered languages inline number words

#### Step 8: Add verification tests

Add test cases for:
- `:from` propagates all tags unconditionally (including `:anim`/`:inan`)
- `:from` preserves multi-dimensional variants (4 cases × 2 numbers for German)
- `@count:word` produces correct CJK number words
- `@ein:acc.other` returns empty string
- `@um:other` produces plural forms
- All new preposition+article transforms produce correct contractions

#### Step 9: Validate and commit

---

## 10. Migration Ordering Summary

```
Task 1: AST Round-Trip Tests
    │
    ▼
Task 2: Predicate Serializer → Phrase  ◄── HARDEST STEP
    │
    ▼
Task 3: Remove Text-Equality Tests
    │
    ▼
Task 4: Remove {{ }} Escapes + VariableBindings (leaf serializers)
    │
    ▼
Task 5: Remaining Effect Arms  ◄──┐
    │                              │  ATOMIC (circular dep)
Task 6: Static Ability Serializer ◄┘
    │
    ▼
Task 7: Ability Serializer
    │
    ▼
Task 8: Remove eval_str + VariableBindings
    │
    ▼
Task 9: Remove Capitalization Helpers
    │
    ▼
Task 10: Add Animacy Tags + :from to Compound Phrases  ◄── i18n prep
```

**Parallel track (can run alongside any task):**
```
Task 11: RLF Framework Changes  ◄── independent of Dreamtides code
```

Tasks 1-9 are sequential — each depends on the previous. Task 10 depends on Task 2 (predicate phrases must exist). Task 11 is independent and can be done at any time since it modifies the RLF crate, not the Dreamtides codebase.

---

## Appendix A: File Reference

| File | Path | Lines |
|------|------|-------|
| i18n review (Chinese) | `docs/plans/i18n-review-zh.md` | — |
| i18n review (Russian) | `docs/plans/i18n-review-ru.md` | — |
| i18n review (Spanish) | `docs/plans/i18n-review-es.md` | — |
| i18n review (PT-BR) | `docs/plans/i18n-review-pt-br.md` | — |
| i18n review (German) | `docs/plans/i18n-review-de.md` | — |
| Serializer directory | `rules_engine/src/parser_v2/src/serializer/` | — |
| RLF strings | `rules_engine/src/strings/src/strings.rs` | ~695 |
| Round-trip test helpers | `rules_engine/tests/parser_v2_tests/src/test_helpers.rs` | 113 |
| Round-trip tests | `rules_engine/tests/parser_v2_tests/tests/round_trip_tests/` | multiple |
| Display eval_str | `rules_engine/src/display/src/rendering/rlf_helper.rs` | 75 |
| Display card rendering | `rules_engine/src/display/src/rendering/card_rendering.rs` | 515 |
| Dreamwell rendering | `rules_engine/src/display/src/rendering/dreamwell_card_rendering.rs` | — |
| Modal rendering | `rules_engine/src/display/src/rendering/modal_effect_prompt_rendering.rs` | — |
| Ability AST types | `rules_engine/src/ability_data/src/` | multiple |
| VariableBindings | `rules_engine/src/parser_v2/src/variables/parser_bindings.rs` | — |

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

These case studies confirm the architecture handles all 6 languages. The critical enabler is Task 2 (predicates returning `Phrase` with gender/case metadata).

## Appendix D: Complete Category B Phrase List (to remove {{ }})

All phrases in strings.rs containing `{{ }}` escapes. Task 4 removes the escapes from all of these:

### Cost Phrases (10)
`abandon_count_allies`, `discard_cards_cost`, `energy_cost_value`, `lose_max_energy_cost`, `banish_your_void_cost`, `banish_another_in_void`, `banish_cards_from_void`, `banish_cards_from_enemy_void`, `banish_void_min_count`, `banish_from_hand_cost`

### Trigger Phrases (7)
`when_you_materialize_trigger`, `when_dissolved_trigger`, `when_banished_trigger`, `when_you_play_cards_in_turn_trigger`, `when_you_abandon_count_in_turn_trigger`, `when_you_draw_in_turn_trigger`, `when_you_materialize_nth_in_turn_trigger`

### Condition Phrases (6)
`with_allies_sharing_type`, `if_drawn_count_this_turn`, `while_void_count`, `with_allied_subtype`, `with_count_allied_subtype`, `with_count_allies`

### Effect Phrases (25)
`draw_cards_effect`, `discard_cards_effect`, `gain_energy_effect`, `gain_points_effect`, `lose_points_effect`, `opponent_gains_points_effect`, `opponent_loses_points_effect`, `foresee_effect`, `kindle_effect`, `each_player_discards_effect`, `prevent_that_card_effect`, `then_materialize_it_effect`, `gain_twice_energy_instead_effect`, `gain_energy_equal_to_that_cost_effect`, `gain_energy_equal_to_this_cost_effect`, `put_deck_into_void_effect`, `banish_cards_from_enemy_void_effect`, `banish_enemy_void_effect`, `judgment_phase_at_end_of_turn_effect`, `multiply_energy_effect`, `spend_all_energy_dissolve_effect`, `spend_all_energy_draw_discard_effect`, `each_player_shuffles_and_draws_effect`, `return_up_to_events_from_void_effect`, `fast_prefix`

### Structural (1)
`pay_one_or_more_energy_cost`
