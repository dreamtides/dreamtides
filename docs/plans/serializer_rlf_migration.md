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
English, Simplified Chinese, Russian, Spanish, Portuguese-Brazil, German, Japanese, Arabic, Turkish, Korean, French

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
Predicate serializers return `Phrase` values that carry `:a`/`:an` tags and `one`/`other` variants. These are passed directly to consuming phrases (the `rlf!` macro generates `impl Into<Value>` parameters, so `Phrase` auto-converts — no `Value::Phrase()` wrapper needed). The consuming phrase can use `{@a $target}` to add the correct article, `{$target:other}` to select the plural form, or `:match($target)` for gender agreement. This is the critical path for localization.

```rust
// Predicate returns Phrase with :an tag and one/other variants
let target = predicate_serializer::serialize_predicate(pred);
// Phrase passes directly via impl Into<Value> — no wrapping needed
strings::when_you_play_trigger(target).to_string()
```

Level 1 applies **inside** each `serialize_standard_effect()` arm. Each arm uses Phrase parameters from the predicate serializer to call RLF phrases. The arm itself returns `String` (via `.to_string()`), not `Phrase`, because its output feeds into Level 2 assembly.

**Level 2 — String concatenation (structural assembly):**
Both the ability serializer (top-level) and `serialize_effect_with_context()` (intermediate-level) assemble final text from already-rendered String pieces. This applies to:
- Ability serializer: trigger + effect, costs + separator + effect
- `serialize_effect_with_context()`: sub-effect joining with "then"/"and", "you may" prefixes, period trimming

This is acceptable because each concatenated piece is itself an RLF phrase that translation files can reorder internally. All structural connectors (": ", ", then ", "you may", etc.) MUST be named RLF phrases.

```rust
// Level 2: concatenate rendered pieces
let trigger_text = strings::when_you_play_trigger(target_phrase).to_string();
let effect_text = strings::draw_cards_effect(count).to_string();
format!("{trigger_text}{effect_text}")
```

**Why not a single top-level phrase for each ability structure?**
Ability structures are highly variable (optional costs, optional once-per-turn, keyword vs non-keyword triggers, multiple cost slots, etc.). Expressing every combination as a single RLF phrase would require dozens of conditional phrases. String concatenation at the top level is simpler, and for languages that need to reorder trigger vs effect, the translation can restructure at the phrase level below.

**Why does `serialize_standard_effect()` return String, not Phrase?**
Standard effects are consumed by `serialize_effect_with_context()` which performs string operations like `trim_end_matches('.')` and conditional joining. These require String, not Phrase. Phrase metadata (tags, variants) is only needed at the predicate→consuming-phrase boundary (Level 1), which happens entirely within each standard effect arm.

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

**Note:** `.capitalized()` and `.capitalized_with_article()` are **dead code** — never called outside `text_formatting.rs` itself. Only 3 of the 5 methods are actually used in production: `.with_article()` (3 sites), `.without_article()` (16 sites), `.plural()` (5 sites).

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
fn serialize_predicate(pred: &Predicate) -> Phrase {
    match pred {
        Another(Character) => strings::ally(),             // Phrase with :an tag
        Enemy(Character) => strings::enemy(),              // Phrase with :an tag
        Your(CharacterType(t)) => {
            strings::subtype(subtype_phrase(*t))            // Phrase inherits tags via :from
        }
    }
}

// Consuming phrase: dissolve_target($target) = "{@cap dissolve} {@a $target}.";
// Phrase passes directly via impl Into<Value> — no wrapping needed:
strings::dissolve_target(predicate_serializer::serialize_predicate(pred)).to_string()
```

The key insight: predicates no longer decide whether to include an article — they return a bare `Phrase` with metadata, and the consuming phrase template uses `{@a $target}` or `{$target}` or `{$target:other}` to control presentation. This is essential for localization because different languages need different article/case forms at different call sites.

### 2.6 Keyword Capitalization Strategy

Currently, `capitalize_first_letter()` handles two patterns:
1. Regular text: uppercase first character
2. Keywords in braces: `"{kindle($k)} ..."` → `"{Kindle($k)} ..."` with title-case logic for underscore-separated keywords

In Phase 2, keywords are always rendered (no more template `{keyword}` syntax). All keyword phrases in `strings.rs` are defined with their display formatting. Capitalization is handled by:
- RLF `@cap` transform for sentence-initial capitalization
- Keyword phrases are defined lowercase by convention; `@cap` is applied at the call site when needed

`capitalize_first_letter()` (18 call sites, 2 dead code) and `lowercase_leading_keyword()` (7 call sites) are deleted. Any serializer function that capitalizes its output uses `@cap` in its phrase template instead.

### 2.7 `:from` Constraints

**`:from` + variant body is DISALLOWED.** The RLF evaluator rejects `:from($param)` combined with a variant block `{ one: ..., other: ... }`. This means the following syntax **WILL FAIL**:

```rust
// WRONG — :from + variant body causes runtime error
allied_subtype($t) = :from($t) { one: "allied {$t}", other: "allied {$t:other}" };
```

**Correct approach:** Use `:from($param)` with a simple template. The `:from` mechanism automatically evaluates the template once per variant of `$param`, producing a result with the same variant structure:

```rust
// CORRECT — :from auto-propagates variants from $t
allied_subtype($t) = :from($t) "allied {$t}";
// If $t has {one: "Warrior", other: "Warriors"}, result is:
//   {one: "allied Warrior", other: "allied Warriors"}
```

**`:from` replaces definition tags, does NOT merge.** When a definition has both its own tags AND `:from`, only the inherited tags survive. For example, `decorated($s) = :masc :from($s) "[{$s}]"` would get tags from `$s`, NOT `:masc`. This is by design (`:from` means "be like the source"), but means you cannot add extra tags alongside `:from`.

---

## 3. Current State Inventory

### 3.1 Serializer Files

| File | Lines | Migration Status (Post Phase 1.5) |
|------|-------|----|
| `ability_serializer.rs` | 176 | Not migrated. Uses `strings::fast_prefix()` only. Hardcoded structural text. |
| `cost_serializer.rs` | 119 | Fully migrated to `strings::` phrases (Category B with `{{ }}`). |
| `trigger_serializer.rs` | 108 | Fully migrated (keyword arms stay as `format!` by design). |
| `condition_serializer.rs` | 99 | Fully migrated to `strings::` phrases. |
| `effect_serializer.rs` | 1139 | 25 arms migrated to `strings::`; 54 arms still use `format!()`. Additionally, several arms use raw RLF template text (e.g., `"{materialize} {@a figment($g)}."`) — a third category neither fully migrated nor plain `format!()`. |
| `predicate_serializer.rs` | 803 | Not migrated. 16 functions (11 pub + 5 private) return hardcoded `String`. |
| `static_ability_serializer.rs` | 222 | Not migrated. Zero `strings::` usage. 23 `StandardStaticAbility` variants. |
| `text_formatting.rs` | 78 | Not migrated. `FormattedText` to be replaced by `Phrase`. |
| `serializer_utils.rs` | 86 | `serialize_operator` migrated. `capitalize_first_letter`/`lowercase_leading_keyword` to be deleted. |

### 3.2 Display Layer Call Sites

All 4 call sites follow the same pattern — serialize then eval:
- `card_rendering.rs:95` — `ability_token_text()` calls `eval_str`
- `card_rendering.rs:182` — `serialize_abilities_text()` calls `eval_str` in a loop
- `dreamwell_card_rendering.rs:87` — `rules_text()` calls `eval_str` in a loop
- `modal_effect_prompt_rendering.rs:63` — `modal_effect_descriptions()` calls `eval_str`

Additionally, `tv/src-tauri/src/derived/rlf_integration.rs:128` calls `locale.eval_str()` directly (TOML Viewer app, may be out of scope).

### 3.3 strings.rs Phrase Inventory

**225 total phrases** defined in `rlf!` macro:
- **176 Category A** (no `{{ }}`): Final phrases that survive into Phase 2 unchanged
- **49 Category B** (contain `{{ }}`): Temporary phrases whose `{{ }}` escapes will be removed

### 3.4 Types That Need PartialEq

**~29 types** in the ability AST tree need `PartialEq, Eq` added. Key types: `Ability`, `EventAbility`, `NamedAbility`, `Effect`, `EffectWithOptions`, `ModalEffectChoice`, `ListWithOptions`, `StandardEffect`, `Cost`, `Predicate`, `CardPredicate`, `Operator<T>`, `Condition`, `TriggerEvent`, `TriggerKeyword`, `PlayerTurn`, `TriggeredAbility`, `TriggeredAbilityOptions`, `ActivatedAbility`, `ActivatedAbilityOptions`, `StaticAbility`, `StaticAbilityWithOptions`, `StandardStaticAbility`, `PlayFromVoid`, `PlayFromHandOrVoidForCost`, `CardTypeContext`, `AlternateCost`, `CollectionExpression`, `QuantityExpression`.

No type contains `f32`/`f64`, `HashMap`, or other problematic types — all are simple `derive` additions. `Operator<T>` is only instantiated with `Energy` and `Spark` (both have `PartialEq`). This is a mechanical but thorough change.

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

### 5.1 Test Strategy Evolution

The testing strategy evolves through three phases as the serializer output changes:

**Phase A (Tasks 1-3): Template text era.** Serializer output is template text (with `{...}` RLF syntax). Three test strategies run in parallel:
1. Text-equality round-trip: `input == serialize(parse(input)).text`
2. AST round-trip: `parse(input) == parse(serialize(parse(input)))`
3. Dual-path rendered comparison: `eval_str(input) == serialize(parse(input)).rendered_text`

**Phase B (Tasks 4-7): Transition era.** Serializer output gradually becomes rendered text (with HTML, Unicode). Text-equality tests break as output format changes. AST round-trip also breaks because the parser cannot re-parse rendered text (it expects template text with `{...}` syntax). **The dual-path rendered comparison is the primary safety net.** It remains viable because test inputs are still written in template format.

**Phase C (Task 8+): Rendered text era.** Serializer produces final rendered text directly. `eval_str` is removed from production but preserved as a **test-only helper**. The dual-path comparison continues working: both paths independently convert template input to rendered output.

**IMPORTANT: Re-parsing rendered text is not feasible.** The parser/lexer expects template text with `{Dissolve}`, `{cards($c)}` syntax. It cannot handle rendered text like `<color=#AA00FF>Dissolve</color>`. No `parse_from_rendered` function exists or should be built.

### 5.2 Golden-File Regression Detection

Before starting the migration, generate a golden file of `(card_name, ability_index, rendered_text)` for every card in the game. After each task, regenerate and diff against the golden file. Expected changes are annotated; unexpected changes are flagged. This catches cases where both dual-path comparison paths produce the same wrong output (e.g., an incorrect RLF phrase definition).

### 5.3 Test Coverage

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

Add `PartialEq, Eq` to all ~29 types listed in Section 3.4. These are all simple `derive` additions — no custom implementations needed. No type contains `f32`/`f64` or other problematic types. `Operator<T>` will auto-derive with `where T: PartialEq` bound, which is satisfied for all instantiations.

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
**Risk:** HIGH — 803 lines, 16 functions, ~95+ call sites across all other serializers.
**Prerequisite:** Task 1

This is the hardest single step. It changes the predicate serializer from returning `String` to returning `Phrase`, replacing `FormattedText` with RLF metadata.

#### Step 1: Add predicate noun phrases to strings.rs

Define RLF terms for all composite predicate noun phrases that `FormattedText` currently constructs. These carry `:a`/`:an` tags and `one`/`other` variants for article and plural selection:

```rust
// Ownership-qualified character terms
your_card = :a { one: "your card", other: "your cards" };
your_event = :an { one: "your event", other: "your events" };
enemy_card = :an { one: "enemy card", other: "enemy cards" };
enemy_event = :an { one: "enemy event", other: "enemy events" };

// Compound phrases use :from with simple templates (NOT variant bodies —
// :from + variant body is DISALLOWED, see Section 2.7).
// :from auto-propagates variants from the parameter.
allied_subtype($t) = :from($t) "allied {$t}";
enemy_subtype($t) = :from($t) "enemy {$t}";
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

This is the bulk of the work (~95+ call sites). Each call site currently does one of:
1. Passes predicate `String` to a `strings::` phrase — change to pass `Phrase` directly (auto-converts via `impl Into<Value>`)
2. Uses predicate `String` in `format!()` — change to use `p.to_string()` for now, or create a new phrase
3. Calls `.with_article()` / `.without_article()` / `.plural()` on the result — use `@a`, direct ref, or `:other` in the phrase template

**No `rlf!` parameter type changes needed.** The macro generates `impl Into<Value>` parameters, which already accept `Phrase` directly. No `Value::Phrase()` wrapping is necessary.

**Coupling warning:** Many effect_serializer arms that consume predicate results ALSO have `{{ }}` escapes. The predicate→Phrase change and the `{{ }}`→`{ }` change (Task 4) are tightly coupled for these arms. The `.to_string()` bridge (pattern #2 above) provides a valid transitional approach, but expect some arms to be touched twice (once in Task 2, again in Task 4/5).

#### Step 4: Delete text_formatting.rs

Remove the `FormattedText` struct and `card_predicate_base_text()` function. All their functionality is now provided by RLF phrases with proper tags and variants.

Update `mod.rs` to remove the module declaration.

#### Step 5: Validate and commit

**Critical:** Both text-equality AND AST round-trip tests must pass. The serializer output text may change (e.g., article selection may differ slightly if tag-based `@a` produces different results than vowel-detection). The AST test ensures semantic equivalence is preserved.

---

### Task 3: Convert to Dual-Path Rendered Output Comparison

**Files:** `test_helpers.rs`, all round-trip test files, TOML round-trip tests
**Risk:** MEDIUM — replacing the primary test safety net.
**Prerequisite:** Task 2 (proves dual-path catches real issues)

This task replaces text-equality round-trip tests with the dual-path rendered output comparison (Section 5.1). Re-parsing rendered text is **not feasible** — the parser expects template text with `{...}` syntax, not rendered HTML output.

#### Step 1: Add dual-path rendered comparison helper

```rust
pub fn assert_rendered_match(input_text: &str, vars: &str) {
    // Path A: parse → serialize → rendered string
    let parsed = parse_ability(input_text, vars);
    let serialized = ability_serializer::serialize_ability(&parsed);
    // Path B: evaluate template text directly through RLF
    let rendered = rlf_helper::eval_str(input_text, &parse_bindings(vars));
    assert_eq!(serialized_to_rendered(&serialized), rendered);
}
```

#### Step 2: Replace text-equality assertions with dual-path comparison

In each test function, replace `assert_round_trip(text, vars)` (text equality) with `assert_rendered_match(text, vars)` (dual-path). Keep AST round-trip tests from Task 1 alongside. Both must pass.

In TOML bulk tests, replace byte-for-byte text comparison with dual-path rendered comparison. Keep AST comparison alongside.

#### Step 3: Generate golden-file baseline

Generate a golden file of `(card_name, ability_index, rendered_text)` for every card. This provides an additional regression detection layer (see Section 5.2).

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

All 49 Category B phrases are rewritten. The phantom parameter problem is eliminated — `$c` is now actually evaluated by RLF.

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

Phrases like `when_you_play_trigger($target)` currently receive pre-rendered `String` targets. After Task 2, they receive `Phrase` values. Pass the `Phrase` directly to phrase functions — `impl Into<Value>` handles the conversion automatically.

#### Step 4: Validate and commit

---

### Task 5: Migrate Remaining Effect Serializer Arms

**Files:** `strings.rs`, `effect_serializer.rs`
**Risk:** MEDIUM — large file, many arms.
**Prerequisite:** Task 4

#### Step 1: Add phrases for all remaining StandardEffect arms

Define RLF phrases for all ~54 unmigrated `StandardEffect` arms. These arms now call the predicate serializer which returns `Phrase`, so `$target` parameters are `Phrase` values:

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

**Predicate-consuming arms** (~35): `DissolveCharacter`, `BanishCharacter`, `GainControl`, `MaterializeCharacter`, `Discover`, `ReturnToHand`, `Counterspell` (all variants), etc. These now pass `Phrase` values from the predicate serializer to new RLF phrases.

**For-each pattern arms** (~8): `DrawCardsForEach`, `GainEnergyForEach`, `GainPointsForEach`, etc. These combine a count phrase with a for-each predicate phrase. **IMPORTANT:** The for-each predicate MUST be passed as a `Phrase` parameter (not pre-rendered string) so that languages with different word order (Chinese/Turkish put "for each" before the effect) can restructure internally.

**Spark-related arms** (~6): `GainsSpark`, `EachMatchingGainsSpark`, `SparkBecomes`, etc.

**Collection arms** (~15 including sub-arms): `DissolveCharactersCount` (5 sub-arms), `BanishCollection` (5 sub-arms), `MaterializeCollection` (4 sub-arms), `MaterializeSilentCopy` (5 sub-arms), etc. These are more complex than they appear — each has nested `CollectionExpression`/`QuantityExpression` matches. Budget accordingly.

#### Step 3: Migrate `serialize_for_count_expression`

This public function (15 arms) returns quantity descriptions used by multiple serializers. Migrate all arms to use RLF phrases. Since it's called by `static_ability_serializer`, coordinate with Task 6.

#### Step 4: Migrate helper functions

- `serialize_gains_reclaim` (~34 lines) + `serialize_void_gains_reclaim` (~82 lines) — **most complex compound pattern**: 8 `CollectionExpression` branches × optional cost × optional "this turn" suffix × predicate. Needs ~8 separate RLF phrases for the collection variants. Special attention for Russian (case varies by collection expression) and CJK (word order reversal).
- `serialize_allied_card_predicate` / `serialize_allied_card_predicate_plural` (2 arms each)

**Note on `serialize_for_count_expression` participial agreement:** The 15 arms produce constructions like "ally abandoned this turn" where the participle must agree with the noun's gender in Russian/German. Each arm needs its own RLF phrase so translations can use `:match($entity)` for participial agreement.

#### Step 5: Migrate `serialize_effect_with_context` structural logic

The `Effect::WithOptions`, `Effect::List` (4 branches), `Effect::ListWithOptions`, and `Effect::Modal` arms. `Effect::ListWithOptions` has hardcoded "you may " prefix and " to " connector that are NOT yet named RLF phrases — these MUST be added. `Effect::Modal` has hardcoded newline + bullet structure that should use named phrases like `modal_choice_prefix` for localization.

**Pronoun agreement note:** Compound effects like "banish X, then materialize it" use "it"/"them" pronouns that need gender agreement in Russian/Spanish/etc. Translation files handle this via `:match($target)` on the antecedent's gender tags, selecting the correct pronoun form.

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

Define phrases for all 23 `StandardStaticAbility` variants. These include conditional text placement logic (condition prepended vs appended). `PlayForAlternateCost` and `PlayFromVoid` are particularly complex — each has card_type context, optional additional_cost, and optional if_you_do effect, requiring 3-4 phrase patterns each. ~10 occurrences of hardcoded "this card"/"this character" should become RLF terms for gender-aware translations.

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

Most structural phrases already exist from Phase 1.5 (`until_end_of_turn_prefix`, `once_per_turn_prefix`, `fast_prefix`, `cost_effect_separator`). Also add `modal_choice_prefix` for the bullet/newline structure in modal effects, and ensure `you_may_prefix` / `to_connector` exist for `ListWithOptions`.

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

### Task 8: Remove eval_str and VariableBindings from Serializer

**Files:** `ability_serializer.rs`, `rlf_helper.rs`, `card_rendering.rs`, `dreamwell_card_rendering.rs`, `modal_effect_prompt_rendering.rs`
**Risk:** MEDIUM — atomic switchover of 4 display call sites + ~35 function signature changes + test infrastructure changes.
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

#### Step 5: Verify VariableBindings type is retained for parser

**CRITICAL:** `VariableBindings` and `VariableValue` CANNOT be deleted. They are used by the **parser** (`parser_substitutions.rs: resolve_variables()`, `resolve_directive()`, `resolve_rlf_syntax()`) and throughout the test infrastructure for parsing card text. Only the serializer's dependency on `VariableBindings` is removed in this task — the type itself must survive.

#### Step 6: Preserve eval_str as test-only helper

Move `rlf_helper::eval_str()` and `build_params()` to a test-only module rather than deleting them. They serve as the independent oracle for the dual-path rendered output comparison (Section 5.1). The test infrastructure (already set up in Task 3) continues to use this test-only `eval_str` to validate serializer output.

#### Step 7: Update TOML bulk tests

TOML round-trip tests must switch from text-equality comparison to dual-path rendered comparison. Both paths (serializer output and test-only eval_str output) independently convert template text to rendered text, and should produce identical results.

#### Step 8: Validate and commit

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

See Section 5.1 for the full test strategy evolution. The core principle: compare rendered output from two independent paths:

```
Path A: template_text → parse() → AST → serialize() → rendered string
Path B: template_text → eval_str() (test-only)       → rendered string
Assert: Path A == Path B
```

**Why not re-parse?** After the migration, serializer output contains HTML color tags, Unicode symbols, etc. The parser/lexer cannot handle this — it expects template text with `{Dissolve}`, `{cards($c)}` syntax. Re-parsing rendered text would require a second parser, which is out of scope and unnecessary. The dual-path comparison validates correctness without re-parsing.

**Oracle definition:** Both paths start from the same template-format input string. Path A goes through the full parse→serialize pipeline. Path B goes through RLF evaluation directly. If both produce identical rendered output, the serializer is correct. `eval_str` is preserved as a test-only function even after removal from production code.

**Golden-file complement:** The golden-file snapshot (Section 5.2) catches cases where both paths produce the same wrong output (e.g., an incorrect RLF phrase definition would fool the dual-path comparison).

### 7.2 Phrase Parameter Types — RESOLVED

The `rlf!` macro generates functions with `impl Into<Value>` parameters. `Phrase` implements `From<Phrase> for Value`. Therefore, `Phrase` values can be passed directly to any phrase function — no `Value::Phrase()` wrapping needed. This was verified against the RLF source code (`codegen.rs:88`).

### 7.3 register_source_phrases() — RESOLVED

With `global-locale` (already enabled in Dreamtides — see `rules_engine/Cargo.toml`), registration is **automatic**. Each generated function includes `__RLF_REGISTER.call_once(...)` via `std::sync::Once`, guaranteeing thread-safe one-time initialization on first use. The explicit `strings::register_source_phrases()` call in `rlf_helper::eval_str()` is redundant. After Phase 2 removes `eval_str()`, no explicit registration is needed.

### 7.4 `:from` Constraints — Important

See Section 2.7 for full details. Two constraints discovered during code verification:
1. `:from($param)` + variant body `{ ... }` is **DISALLOWED** — use simple templates instead
2. `:from` **replaces** definition tags, does not merge — a definition's own tags are silently dropped when `:from` is present

---

## 8. Risk Assessment

| Task | Risk | Reason |
|------|------|--------|
| 1. AST round-trip tests | HIGH | Single point of failure; ~29 types need PartialEq |
| 2. Predicate → Phrase | HIGH | 803 lines, 16 functions, ~95+ call sites, entangled with FormattedText |
| 3. Dual-path test conversion | MEDIUM | Replacing primary test safety net; golden-file baseline generation |
| 4. Remove `{{ }}` escapes | LOW | Mechanical changes; tightly coupled with Task 2 for some arms |
| 5. Remaining effect arms | MEDIUM-HIGH | 54 unmigrated arms; collection arms more complex than they appear; `serialize_void_gains_reclaim` needs ~8 phrase variants |
| 6. Static ability serializer | MEDIUM | 23 variants; `PlayForAlternateCost`/`PlayFromVoid` are complex; circular dep with effect_serializer |
| 7. Ability serializer | LOW | Straightforward; add modal/ListWithOptions structural phrases |
| 8. Remove eval_str | MEDIUM | Atomic switchover of 4 display call sites + ~35 function signatures; test infrastructure changes; VariableBindings type must be preserved for parser |
| 9. Remove capitalization | LOW | 18 call sites (2 dead code), 7 lowercase sites |
| 10. `:from` + keyword audit | LOW | Must use simple `:from` templates, NOT `:from` + variant body |
| 11. RLF framework changes | LOW | Independent of Dreamtides code, follows established patterns |

---

## 9. Multilingual Design Considerations

These were identified by i18n stress testing across all 11 target languages (two rounds of 5-language review). They inform the Rust code structure even though we're not writing translations yet.

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

### 9.2 Gender Agreement (Russian, Spanish, Portuguese, German, French, Arabic)

"when X is dissolved" requires participle agreement with X's gender. Handled by `:match` on gender tags:

```
// ru.rlf
when_dissolved_trigger($target) = :match($target) {
    masc: "когда {$target:nom} растворён, ",
    fem: "когда {$target:nom} растворена, ",
    *neut: "когда {$target:nom} растворено, ",
};
```

**Rust code implication:** Predicate `Phrase` values must carry gender tags (`:masc`/`:fem`/`:neut`). These come from the translation file's definition of each term, not the English source. When the locale is Russian, `strings::ally()` returns the Russian `Phrase` which carries `:masc`.

### 9.3 Personal "a" (Spanish)

"dissolve an enemy" → "disolver **a** un enemigo". Handled by `:match` on `:anim` tag:

```
// es.rlf
dissolve_target($target) = :match($target) {
    anim: "{@cap dissolve} a {@un $target:acc}.",
    *inan: "{@cap dissolve} {@un $target:acc}.",
};
```

**Rust code implication:** The Spanish translation file defines entity terms with `:anim`/`:inan` tags. When the locale is Spanish, `strings::ally()` returns the Spanish `Phrase` which carries `:anim`, enabling the `:match` branch. No tags needed on the English source.

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

All grammatical tags are **language-specific** and live in **translation files**, not in the English source. Each language defines whatever tags its grammar requires. When the serializer calls `strings::ally()`, it gets the `Phrase` from the active locale's definition, carrying that locale's tags.

| Tag | Defined in | Purpose |
|-----|-----------|---------|
| `:a` / `:an` | English source (`strings.rs`) | English indefinite article selection |
| `:anim` / `:inan` | Translation files (es, ru, etc.) | Animacy (Spanish personal "a", Russian acc=gen) |
| `:masc` / `:fem` / `:neut` | Translation files (es, ru, de, pt, fr, ar) | Gender agreement |
| `:zhang` / `:ge` / `:ming` etc. | Chinese translation | Chinese classifier tags for `@count` |
| `:mai` / `:tai` / `:ko` etc. | Japanese translation | Japanese counter tags for `@count` |
| `:jang` / `:myeong` / `:gae` etc. | Korean translation | Korean counter tags for `@count` |
| `:sun` / `:moon` | Arabic translation | `@al` definite article assimilation |
| `:front` / `:back` | Turkish translation | Vowel harmony for `@inflect` suffixes |
| `:vowel` | French translation | Elision trigger for `@le`/`@de`/`@au` |
| `:nom` / `:acc` / `:dat` etc. | Translation files (ru, de, ar, tr) | Case variant keys |

**Key principle:** The English source only carries `:a`/`:an` tags (English grammar). All other grammatical metadata (gender, animacy, classifiers, case) is defined by each translation file on its own terms. This keeps the English source clean and gives each language full control over its grammatical annotations.

**Example:** The English source defines `ally = :an { one: "ally", other: "allies" }`. The Russian translation independently defines `ally = :masc :anim { nom: "союзник", ... }`. When the locale is Russian, `strings::ally()` returns the Russian Phrase with `:masc :anim` tags. The English `:an` tag is irrelevant in Russian context.

**Compound phrase `:from` requirement (Task 2):** Compound phrases that wrap entity references MUST use `:from($entity)` with a simple template (NOT a variant body — see Section 2.7). This preserves tag propagation through the composition chain:
```rust
allied($entity) = :from($entity) "allied {$entity}";
enemy_modified($entity) = :from($entity) "enemy {$entity}";
```

When running in Russian, `allied(ally)` inherits `:masc :anim` from the Russian `ally` definition via `:from`. Variants auto-propagate through the simple template. A downstream phrase can then `:match` on those tags.

**`:from` tag propagation guarantee:** `:from($param)` propagates ALL tags unconditionally from the parameter — verified in the RLF evaluator source code (`evaluator.rs:226-243`). Note that `:from` **replaces** the definition's own tags (see Section 2.7), so don't combine `:from` with explicit tags on the same definition.

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
- [ ] `:from` propagates ALL tags unconditionally from the active locale's definition
- [ ] Phrase parameter types in `rlf!` macro-generated functions
- [ ] `@cap` is a no-op on CJK characters (Chinese/Japanese/Korean) and Arabic script
- [ ] `@cap` skips leading HTML markup tags to find first visible character
- [ ] `@cap` uses locale-sensitive Unicode case mapping (Turkish dotted/dotless I)
- [ ] `@inflect` handles Turkish buffer consonant insertion (y/n/s) for vowel-ending stems
- [ ] `@inflect` accepts Phrase parameters, not only term references (Turkish `{@inflect:acc $target}`)
- [ ] `@particle` strips trailing HTML markup to find last visible character (Korean/Japanese)
- [ ] `@particle:and` (와/과), `:copula` (이다/다), `:dir` (으로/로) contexts for Korean
- [ ] `@particle` handles ASCII digit endings via Korean pronunciation lookup table
- [ ] `@al` correctly reads `:sun`/`:moon` tags inherited via `:from` (Arabic)
- [ ] `@le`/`@de`/`@au` read `:vowel` tag for French elision (l', d', à l')
- [ ] `@le:other`/`@de:other`/`@au:other` produce French plural forms (les/des/aux)
- [ ] `:from` propagates `:vowel` tag through compound phrases (French elision chain)

### 9.8 RLF Framework Changes Required

These changes to the RLF framework (`rlf` crate) were identified across two rounds of i18n review (11 languages total). See `docs/plans/i18n-review-*.md` for full analysis. Changes are ordered by priority.

#### 9.8.1 Portuguese `@para` Transform — HIGH

Add preposition "a" + article contraction (ao/à/aos/às). Needed for dynamic "to X" phrases like "devolva à mão" (return to the hand). ~30 lines following the exact pattern of existing `@de` and `@em` transforms. See `i18n-review-pt-br.md` Section 14.1.

#### 9.8.2 Portuguese `@por` Transform — MEDIUM

Add preposition "por" + article contraction (pelo/pela/pelos/pelas). Needed for "by/for each" patterns. ~30 lines. See `i18n-review-pt-br.md` Section 14.2.

#### 9.8.3 Portuguese `@um` Plural Context — MEDIUM

Enhance `@um` to accept `:other` context for plural indefinite articles (uns/umas). Currently only produces singular (um/uma). ~10 lines. See `i18n-review-pt-br.md` Section 14.3.

#### 9.8.4 CJK `@count:word` Modifier — MEDIUM

Add a `:word` context modifier to `@count` that substitutes number words instead of digits for small numbers (1-10). Falls back to digits for larger numbers. Each CJK language has its own mapping: Chinese (一/两/三, note 两 not 二 for counting), Japanese (一/二/三, always 二), Korean (한/두/세 for native-system counters like `:jang`/`:myeong`, digits for Sino-Korean counters like 점/턴). ~30-40 lines per language. See `i18n-review-zh.md` Rec 1, `i18n-review-ja.md` Rec 1, `i18n-review-ko.md` KO-2.

#### 9.8.5 German `@ein` Empty Plural — LOW

Define `@ein` behavior for plural context: return empty string. German has no plural indefinite article. Currently undefined behavior. ~5 lines. See `i18n-review-de.md` Recommendation 2.

#### 9.8.6 Spanish `@del`/`@al` Contractions — LOW

Add preposition+article contraction transforms: "de"+"el"="del", "a"+"el"="al". Only applies to masculine singular definite article. Low priority because most card text uses possessives ("tu") not articles for locations. ~20 lines each. See `i18n-review-es.md` RFC-ES-3/RFC-ES-7.

#### 9.8.8 Turkish Buffer Consonant Insertion in `@inflect` — HIGH

Turkish `@inflect` must insert buffer consonants (y/n/s) when suffixes are appended to vowel-ending stems: "y" before case suffixes (elma+acc→elmayı), "s" before 3sg possessive (araba+poss3sg→arabası), "n" before case suffixes after possessive endings (arabası+acc→arabasını). Without this, ~50% of Turkish noun inflections produce incorrect output. ~40 lines. See `i18n-review-tr.md` BLOCK-TR-1.

#### 9.8.9 Korean `@particle` HTML Markup Stripping — HIGH

Korean `@particle` must strip trailing HTML markup tags (`</color>`, `</b>`) to find the last visible character before determining consonant/vowel ending for particle selection (을/를, 이/가, 은/는). Analogous to `@cap` skipping leading markup. Without this, every particle attached to a colored keyword is incorrect. ~10 lines. See `i18n-review-ko.md` KO-CRIT-1.

#### 9.8.10 Korean Additional Particle Contexts — MEDIUM

Add `:and` (와/과), `:copula` (이다/다), `:dir` (으로/로) contexts to Korean `@particle`. Needed for conjunctive, copular, and directional patterns in card text. ~15 lines following the existing `:subj`/`:obj`/`:topic` pattern. See `i18n-review-ko.md` KO-1.

#### 9.8.11 Japanese `:tai` Counter Tag — LOW

Add `:tai` (体) to the Japanese counter tag set. Standard counter for game characters/creatures in Japanese TCGs. Without it, translators must hardcode 体 in every character-counting phrase. ~2 lines. See `i18n-review-ja.md` J1.

#### 9.8.12 French `@un:other` Plural — LOW

Define `@un:other` → "des" for French (gender-neutral plural indefinite). Same pattern as Portuguese `@um:other` (9.8.3). ~5 lines. See `i18n-review-fr.md` RFC-FR-4.

#### 9.8.13 Changes NOT Recommended

The following were evaluated and rejected across both i18n review rounds:
- **Classifiers as first-class concept** (Chinese) — tags + `@count` already handle this
- **`@sep` for separable verbs** (German) — phrase templates handle this perfectly
- **`@part` for participle agreement** (all gendered languages) — `:match` is simpler and more flexible
- **`@kein` negative article** (German) — too few use cases, inline works
- **Verb conjugation transforms** (Portuguese, Spanish) — only ~15 fixed verbs in the game
- **Implicit `:from` on compound phrases** — explicit is better, avoid ambiguity with multi-parameter phrases
- **Language-specific `:from` behavior** — `:from` is correctly language-neutral by design
- **Declension context on `@der`/`@ein`** (German) — pragmatic alternative works for Dreamtides domain
- **Turkish-specific article transform** — Turkish has no articles; "bir" is a numeral, not an article
- **Automatic plural suppression after numerals** (Turkish) — translators handle via `:match`
- **`@ne` negation transform** (French) — ne...pas wraps verb, handled by phrase templates
- **Honorific-level transforms** (Japanese, Korean) — card text uses single speech level chosen at translation time
- **Full-width punctuation transforms** (Chinese, Japanese) — RLF phrase redefinition handles this
- **`@particle` for Japanese** — Japanese particles are phonology-independent, can be written directly in templates

### 9.9 Cross-Language Design Conventions

These conventions were identified by the i18n review and MUST be followed during Phase 2:

1. **Effect phrases MUST NOT include trailing periods.** The period is added by `period_suffix` at the assembly level. This prevents double punctuation when effects are joined by `then_joiner`. (German agent: separable verb prefix must come before the period/connector.)

2. **Each phrase MUST control its own capitalization via `@cap`.** The Rust code MUST NOT apply `capitalize_first_letter()` to already-rendered strings. The ability serializer concatenates pre-capitalized pieces. (German agent: V2 word order requires verb-initial effects; Chinese agent: `@cap` is a no-op on CJK.)

3. **Keyword terms MUST have separate imperative and participial forms.** `dissolve` (imperative, for effects) and `dissolved` (participle, for triggers) are distinct terms. Spanish/Portuguese need different verb forms; Russian/German need different gender-agreeing participles. (Spanish agent: BLOCK-ES-2.)

4. **Structural connectors MUST be named RLF phrases.** All punctuation, separators, and joiners used by `ability_serializer.rs` go through named phrases: `then_joiner`, `and_joiner`, `period_suffix`, `cost_effect_separator`, `once_per_turn_prefix`, `until_end_of_turn_prefix`, etc. (Chinese agent: full-width punctuation; Russian agent: em-dash separator convention.)

5. **`text_number` is English-specific.** Translators for gendered languages (Spanish, Portuguese, German) should inline number words in their phrase templates using `:match`, not call `text_number`. Only "1" varies by gender in most languages.

6. **All structural connectors in `serialize_effect_with_context()` MUST be RLF phrases.** This includes "you may " prefix, " to " connector (for `ListWithOptions`), modal bullet/newline formatting, and "it"/"them" pronouns in compound effects. Languages that restructure modality (Turkish verb suffix, Japanese potential form) or need gendered pronouns (Russian/Spanish "it") require these to be translatable.

7. **"this card"/"this character" MUST be RLF terms.** Currently hardcoded ~10 times in static_ability_serializer.rs. These need gender-aware translations (Russian: "этот персонаж" (masc) / "эта карта" (fem)).

---

### Task 10: Add `:from` to Compound Phrases and Audit Keywords

**Files:** `strings.rs`
**Risk:** LOW — additive changes only, no behavioral impact on English.
**Prerequisite:** Task 2 (predicate phrases must exist)

This task ensures the English source phrases support proper tag propagation for localization. Note: grammatical tags like `:anim`/`:inan`, `:masc`/`:fem`, and classifiers are defined by each translation file on its own terms — the English source only needs `:a`/`:an`. The key English-side requirement is that compound phrases use `:from` so that whatever tags the active locale defines will propagate through composition.

#### Step 1: Add `:from($entity)` to compound predicate phrases

Ensure tag propagation through the composition chain:

```rust
// Simple :from templates — variants auto-propagate (see Section 2.7)
allied($entity) = :from($entity) "allied {$entity}";
enemy_modified($entity) = :from($entity) "enemy {$entity}";
allied_subtype($t) = :from($t) "allied {$t}";
enemy_subtype($t) = :from($t) "enemy {$t}";
```

Without `:from`, downstream phrases that do `:match(allied_result)` for gender/animacy would fail because tags are lost in the composition. When running in Russian, `allied(ally)` inherits `:masc :anim` from the Russian `ally` definition. When running in English, it inherits `:an` from the English definition. Note: `:from` with simple templates auto-generates variant forms — `allied_subtype(warrior)` produces `{one: "allied Warrior", other: "allied Warriors"}` automatically.

#### Step 2: Verify keyword terms have separate imperative/participial forms

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

This task implements the RLF framework changes identified by both rounds of i18n review (Section 9.8).

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

#### Step 7: Add Turkish buffer consonant insertion to `@inflect` (HIGH)

Implement y/n/s buffer consonant insertion for Turkish vowel-ending stems. ~40 lines. See 9.8.8.

#### Step 8: Add Korean `@particle` HTML markup stripping (HIGH)

Strip trailing HTML tags in `@particle` to find last visible character. ~10 lines. See 9.8.9.

#### Step 9: Add Korean particle contexts `:and`/`:copula`/`:dir` (MEDIUM)

Add 3 new contexts to Korean `@particle` following existing pattern. ~15 lines. See 9.8.10.

#### Step 10: Add Japanese `:tai` counter tag (LOW)

Add `:tai` (体) to Japanese counter tag set. ~2 lines. See 9.8.11.

#### Step 11: Add French `@un:other` plural (LOW)

`@un:other` → "des". Same pattern as Portuguese `@um:other`. ~5 lines. See 9.8.12.

#### Step 12: Update APPENDIX_STDLIB.md documentation

- Document `@count:word` modifier for CJK (Chinese/Japanese/Korean number mappings)
- Document `@para`, `@por` in Portuguese transform table
- Document `@um:other` and `@un:other` plural behavior
- Document `@ein` empty-string plural behavior
- Document `@del`/`@al` in Spanish transform table
- Document Turkish suffix ordering: plural → possessive → case
- Document Turkish buffer consonant rules
- Document Korean `@particle` digit-based lookup table and HTML stripping behavior
- Add note that `text_number` is English-specific; gendered languages inline number words

#### Step 13: Add verification tests

Add test cases for:
- `:from` propagates all tags unconditionally from the active locale's definition
- `:from` preserves multi-dimensional variants (4 cases × 2 numbers for German)
- `@count:word` produces correct number words per CJK language (Chinese 两, Japanese 二, Korean 한/두/세)
- `@ein:acc.other` returns empty string
- `@um:other` and `@un:other` produce plural forms
- All new preposition+article transforms produce correct contractions
- Turkish `@inflect` buffer consonant insertion (elma+acc→elmayı, araba+poss3sg→arabası)
- Korean `@particle` strips HTML and selects correct particle form
- Korean `@particle` handles digit endings (2→vowel, 3→consonant)
- `@cap` is no-op on Arabic script and locale-sensitive for Turkish I/ı

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
Task 3: Convert to Dual-Path Rendered Comparison + Golden File
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
Task 8: Remove eval_str from production (keep as test-only)
    │
    ▼
Task 9: Remove Capitalization Helpers
    │
    ▼
Task 10: Add :from to Compound Phrases + Audit Keywords  ◄── i18n prep
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
| i18n review (Japanese) | `docs/plans/i18n-review-ja.md` | — |
| i18n review (Arabic) | `docs/plans/i18n-review-ar.md` | — |
| i18n review (Turkish) | `docs/plans/i18n-review-tr.md` | — |
| i18n review (Korean) | `docs/plans/i18n-review-ko.md` | — |
| i18n review (French) | `docs/plans/i18n-review-fr.md` | — |
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
| JA | "カードを一枚引く。" | "カードを3枚引く。" | Counter 枚, SOV, no articles |
| AR | "اسحب بطاقة." | "اسحب 3 بطاقات." | All 6 CLDR categories, broken plurals |
| TR | "bir kart çek." | "3 kart çek." | SOV, no plural after numerals |
| KO | "카드 한 장을 뽑는다." | "카드 3장을 뽑는다." | Native numbers, particles, SOV |
| FR | "Piochez une carte." | "Piochez 3 cartes." | Gender on article, elision |

### "Dissolve an enemy Ancient." across all languages

| Language | Output | Key Features |
|----------|--------|--------------|
| EN | "Dissolve an enemy Ancient." | `@a` reads `:an` tag |
| ZH | "消解一个敌方远古。" | Classifier 个, no articles |
| RU | "Растворите вражеского Древнего." | Accusative on BOTH adjective and noun (masc.anim → acc=gen) |
| ES | "Disuelve a un Antiguo enemigo." | Personal "a", reversed adjective order |
| PT-BR | "Dissolva um Ancião inimigo." | Reversed adjective order |
| DE | "Löse einen feindlichen Uralten auf." | Separable verb, accusative article, adjective declension |
| JA | "敵のエンシェントを消滅させる。" | SOV, particle を marks object |
| AR | "حَلّ عتيقاً معادياً." | Masc. imperative, accusative ending |
| TR | "Bir düşman Kadimi erit." | `@inflect:acc` on target, SOV |
| KO | "적 고대인을 해체한다." | `@particle:obj` selects 을, SOV |
| FR | "Dissolvez l'Ancien ennemi." | `@le` reads `:vowel`, elision l' |

These case studies confirm the architecture handles all 11 languages. The critical enabler is Task 2 (predicates returning `Phrase` with gender/case metadata).

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
