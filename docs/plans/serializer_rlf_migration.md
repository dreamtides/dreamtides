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
Predicate serializers return `Phrase` values that carry `:a`/`:an` tags and `one`/`other` variants. These are passed directly to consuming phrases (the `rlf!` macro generates `impl Into<Value>` parameters, so `Phrase` auto-converts — no `Value::Phrase()` wrapper needed — verified in `rlf-macros/src/codegen.rs:88`). The consuming phrase can use `{@a $target}` to add the correct article, `{$target:other}` to select the plural form, or `:match($target)` for gender agreement.

```rust
// Predicate returns Phrase with :an tag and one/other variants
let target = predicate_serializer::serialize_predicate(pred);
// Phrase passes directly via impl Into<Value> — no wrapping needed:
strings::dissolve_target(target).to_string()
```

Level 1 applies **inside** each `serialize_standard_effect()` arm. Each arm uses Phrase parameters from the predicate serializer to call RLF phrases. The arm itself returns `String` (via `.to_string()`), not `Phrase`, because its output feeds into Level 2 assembly.

**Level 2 — String concatenation (structural assembly):**
Both the ability serializer (top-level) and `serialize_effect_with_context()` (intermediate-level) assemble final text from already-rendered String pieces. This applies to:
- Ability serializer: trigger + effect, costs + separator + effect
- `serialize_effect_with_context()`: sub-effect joining with "then"/"and", "you may" prefixes, period trimming

All structural connectors (": ", ", then ", "you may", etc.) MUST be named RLF phrases. Each concatenated piece is itself an RLF phrase that translation files can reorder internally.

**Why not a single top-level phrase for each ability structure?**
Ability structures are highly variable (optional costs, optional once-per-turn, keyword vs non-keyword triggers, multiple cost slots, 4+ branches in Effect::List, etc.). Expressing every combination as a single RLF phrase would require dozens of conditional phrases. String concatenation at the top level is simpler, and for languages that need to reorder trigger vs effect, the translation can restructure at the phrase level below.

**Level 2 i18n limitation — acknowledged and accepted:**
SOV languages (Turkish, Japanese, Korean) and VSO patterns (Arabic) may need different ordering of trigger + effect at Level 2. The current approach handles this because: (a) each *phrase* internally defines its own word order (e.g., Turkish `dissolve_target` puts the verb last), (b) trigger-effect ordering is consistent within each language (Japanese always puts the trigger clause first with a conditional particle), and (c) the few structural patterns at Level 2 (trigger+effect, cost:effect) can be overridden by making the top-level assembly phrases themselves translatable. If a language needs radically different trigger-effect ordering, add a `triggered_ability($trigger, $effect)` phrase that the ability serializer calls instead of concatenation.

### 2.4 FormattedText → Phrase Mapping

`FormattedText` currently provides five operations. Each maps directly to an RLF feature:

| FormattedText method | RLF equivalent | Example |
|---------------------|----------------|---------|
| `.with_article()` → `"a card"`, `"an ally"` | `{@a $phrase}` reads `:a`/`:an` tag | `ally = :an { one: "ally", other: "allies" };` |
| `.without_article()` → `"card"`, `"ally"` | Direct `{$phrase}` reference | `{$target}` |
| `.plural()` → `"cards"`, `"allies"` | Variant selection `{$phrase:other}` | `{$target:other}` |
| `.capitalized()` → `"Card"`, `"Ally"` | `{@cap $phrase}` transform | `{@cap $target}` |
| `.capitalized_with_article()` → `"A card"`, `"An ally"` | `{@cap @a $phrase}` | `{@cap @a $target}` |

**Production usage:** Only 3 of the 5 methods are used in production: `.with_article()` (3 sites), `.without_article()` (16 sites in predicate_serializer + 8 in effect_serializer's `card_predicate_base_text` calls), `.plural()` (5 sites). `.capitalized()` and `.capitalized_with_article()` are only called from within `text_formatting.rs` itself.

**Additional `FormattedText` concern:** `new_non_vowel()` (used in `enemy_predicate_formatted` for `NotCharacterType` — "non-{subtype($t)} enemy") forces `starts_with_vowel_sound = false`. In RLF, this maps to using `:a` tag instead of `:an`. But the phrase contains a template `{subtype($t)}` whose vowel status depends on the subtype. The RLF approach is better: define the compound phrase with `:from($t)` and let the subtype's own `:a`/`:an` tag propagate.

### 2.5 Predicate System Redesign

Currently, `serialize_predicate()` returns strings like `"a character"`, `"an enemy"`, `"{@a subtype($t)}"` — baking in the article and ownership context. In Phase 2, predicates return `Phrase` objects and the consuming phrase decides presentation.

The key insight: predicates no longer decide whether to include an article — they return a bare `Phrase` with metadata, and the consuming phrase template uses `{@a $target}` or `{$target}` or `{$target:other}` to control presentation. This is essential for localization because different languages need different article/case forms at different call sites.

**Predicate function count (verified):** 12 functions total: 8 public (`serialize_predicate`, `serialize_predicate_plural`, `predicate_base_text`, `serialize_card_predicate`, `serialize_card_predicate_without_article`, `serialize_card_predicate_plural`, `serialize_cost_constraint_only`, `serialize_for_each_predicate`, `serialize_fast_target`, `serialize_your_predicate`, `serialize_enemy_predicate`) + 2 private (`your_predicate_formatted`, `enemy_predicate_formatted`) + 2 private plural helpers + 1 utility (`is_generic_card_type`).

**Call site count (verified):** 120 `predicate_serializer::` calls across 5 consumer files (effect_serializer: 75, trigger_serializer: 13, condition_serializer: 13, cost_serializer: 11, static_ability_serializer: 8).

### 2.6 Keyword Capitalization Strategy

Currently, `capitalize_first_letter()` handles two patterns:
1. Regular text: uppercase first character
2. Keywords in braces: `"{kindle($k)} ..."` → `"{Kindle($k)} ..."` with title-case logic

In Phase 2, keywords are always rendered (no more template `{keyword}` syntax). All keyword phrases in `strings.rs` are defined with their display formatting. Capitalization is handled by:
- RLF `@cap` transform for sentence-initial capitalization
- Keyword phrases are defined lowercase by convention; `@cap` is applied at the call site

**`lowercase_leading_keyword` usage (verified):** 7 call sites, ALL in `serialize_effect_with_context()` for the "you may" / trigger cost lowering pattern. After migration, these become `@lower` on the first term reference in the phrase, or the phrase is designed to produce lowercase output by default.

**`capitalize_first_letter` usage (verified):** 16 real call sites (10 in ability_serializer, 4 in effect_serializer, 2 in static_ability_serializer). All become `@cap` in phrase templates.

### 2.7 `:from` Constraints

**`:from` + `:match` IS supported.** Verified in `rlf/crates/rlf/src/interpreter/evaluator.rs` — the function `eval_from_with_match()` explicitly handles this combination. `:from` determines the inherited tag/variant structure; `:match` branches within each variant's evaluation.

**`:from` + variant body (without `:match`) is DISALLOWED.** The RLF evaluator rejects `:from($param)` combined with a plain variant block `{ one: ..., other: ... }`. Use `:from($param) :match(...)` or `:from($param)` with a simple template.

**Correct patterns:**
```rust
// Simple :from template — variants auto-propagate
allied_subtype($t) = :from($t) "allied {$t}";

// :from + :match — variants from :from, branching from :match
count_subtype($n, $s) = :from($s) :match($n) {
    1: "союзный {subtype($s)}",
    *other: "{$n} союзных {subtype($s):gen:many}",
};
```

**`:from` replaces definition tags, does NOT merge.** When a definition has both its own tags AND `:from`, only the inherited tags survive. Don't combine `:from` with explicit tags on the same definition.

---

## 3. Current State Inventory

### 3.1 Serializer Files

| File | Lines | Migration Status (Post Phase 1.5) |
|------|-------|----|
| `ability_serializer.rs` | 177 | Not migrated. Uses `strings::fast_prefix()` only. Hardcoded structural text. |
| `cost_serializer.rs` | 119 | Fully migrated to `strings::` phrases (Category B with `{{ }}`). |
| `trigger_serializer.rs` | 108 | Fully migrated (keyword arms stay as `format!` by design). |
| `condition_serializer.rs` | 99 | Fully migrated to `strings::` phrases. |
| `effect_serializer.rs` | 1139 | Mixed state: ~25 arms use `strings::` phrases, ~54 use `format!()`, some use raw RLF templates. |
| `predicate_serializer.rs` | 803 | Not migrated. 12 functions return hardcoded `String`. 120 call sites across consumers. |
| `static_ability_serializer.rs` | 222 | Not migrated. Zero `strings::` usage. 23 `StandardStaticAbility` variants. |
| `text_formatting.rs` | 78 | Not migrated. `FormattedText` to be replaced by `Phrase`. |
| `serializer_utils.rs` | 87 | `serialize_operator` migrated. `capitalize_first_letter`/`lowercase_leading_keyword` to be deleted. |

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

No type contains `f32`/`f64`, `HashMap`, or other problematic types — all are simple `derive` additions.

### 3.5 VariableBindings Threading (Verified)

**42 VariableBindings references** across 7 serializer files. Every serializer function takes `bindings: &mut VariableBindings`. This is NOT just a parameter count — many functions do both `bindings.insert()` AND pass `bindings` to sub-calls. During migration, `bindings.insert()` calls are replaced by passing values directly to phrase functions, and the `bindings` parameter is removed from all function signatures.

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

**text_formatting coupling (additional):** `text_formatting::card_predicate_base_text()` is called from `predicate_serializer` (many sites) AND directly from `effect_serializer` (8 sites in `serialize_for_count_expression`) AND from `static_ability_serializer` (1 site). When `text_formatting` is deleted, ALL these callers must be updated simultaneously.

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
3. Dual-path rendered comparison: `eval_str(input, vars) == serializer_rendered_output`

**Phase B (Tasks 4-7): Transition era.** Serializer output gradually becomes rendered text (with HTML, Unicode). Text-equality tests break as output format changes. AST round-trip also breaks because the parser cannot re-parse rendered text. **The dual-path rendered comparison is the primary safety net.**

**Phase C (Task 8+): Rendered text era.** Serializer produces final rendered text directly. `eval_str` is preserved as a **test-only helper**. The dual-path comparison continues working: both paths independently convert template input to rendered output.

**IMPORTANT: Re-parsing rendered text is not feasible.** The parser expects template text with `{Dissolve}`, `{cards($c)}` syntax. It cannot handle rendered text like `<color=#AA00FF>Dissolve</color>`.

### 5.2 Dual-Path Test Oracle — Critical Details

The dual-path comparison requires careful setup:

```
Path A: template_text + vars → parse() → AST → serialize() → .to_string() → rendered string
Path B: template_text + vars → eval_str(template_text, parse_bindings(vars)) → rendered string
Assert: Path A == Path B
```

**Path B requires the ORIGINAL template text and VariableBindings.** After the serializer stops producing template text, Path B must use the test *input* (which is still in template format) as its source. The test helper must:
1. Parse the input template text + vars to get the AST
2. Serialize the AST to get rendered text (Path A)
3. Evaluate the *original* input template text through `eval_str` with the *original* vars (Path B)
4. Compare

**This means test inputs must remain in template format.** The TOML card data (`cards.toml`) is in template format with `{...}` directives and variable bindings, so it naturally serves as the Path B input. Unit tests also use template-format inputs. No test infrastructure change is needed for the input format.

**Edge case — predicates that embed templates:** Some predicate arms return template strings like `"{@a subtype($t)}"`. After migration, these return `Phrase` values. The Path A output will differ from the Path B output if the RLF phrase definition produces different text than the template string. Each such difference must be investigated during migration.

### 5.3 Golden-File Regression Detection

Before starting the migration, generate a golden file of `(card_name, ability_index, rendered_text)` for every card in the game. After each task, regenerate and diff against the golden file. Expected changes are annotated; unexpected changes are flagged.

### 5.4 Test Coverage

- `cards_toml_round_trip_tests` — serializes every card in the game
- `dreamwell_toml_round_trip_tests` — same for dreamwell cards
- Unit round-trip tests in 6 files — individual ability patterns
- `card_effect_parser_tests.rs` — insta snapshot tests for parser AST

---

## 6. Task Breakdown

### Task 1: AST-Level Round-Trip Tests + Golden File Baseline

**Files:** `ability_data` types, `test_helpers.rs`, new test file
**Risk:** HIGH — this is the single point of failure for all subsequent work.

#### Step 1: Add PartialEq to the Ability AST

Add `PartialEq, Eq` to all ~29 types listed in Section 3.4. These are all simple `derive` additions.

#### Step 2: Add `assert_ast_round_trip` helper

Add to `test_helpers.rs` a function that parses, serializes, re-parses, and asserts AST equality.

#### Step 3: Add parallel AST round-trip tests

For every existing `assert_round_trip(text, vars)` call, add a corresponding `assert_ast_round_trip(text, vars)`. Run both in the same test function.

#### Step 4: Add AST-level comparison to TOML bulk tests

In `cards_toml_round_trip_tests.rs` and `dreamwell_toml_round_trip_tests.rs`, add AST comparison alongside the existing text comparison.

#### Step 5: Generate golden-file baseline

Generate a golden file of `(card_name, ability_index, rendered_text)` for every card. Store it in the test fixtures directory. This provides a regression detection baseline for all subsequent tasks.

#### Step 6: Validate and commit

---

### Task 2: Predicate Serializer + FormattedText → Phrase

**Files:** `predicate_serializer.rs`, `text_formatting.rs`, `strings.rs`
**Risk:** HIGH — 803 lines, 12 functions, 120+ call sites across 5 consumer files.
**Prerequisite:** Task 1

This is the hardest single step. It changes the predicate serializer from returning `String` to returning `Phrase`, replacing `FormattedText` with RLF metadata.

#### Step 1: Add predicate noun phrases to strings.rs

Define RLF terms for all composite predicate noun phrases that `FormattedText` currently constructs. These carry `:a`/`:an` tags and `one`/`other` variants for article and plural selection:

```rust
// Base entity terms (already partially defined)
ally = :an { one: "ally", other: "allies" };
enemy = :an { one: "enemy", other: "enemies" };
your_card = :a { one: "your card", other: "your cards" };
your_event = :an { one: "your event", other: "your events" };
enemy_card = :an { one: "enemy card", other: "enemy cards" };

// Compound phrases use :from with simple templates
allied_subtype($t) = :from($t) "allied {$t}";
enemy_subtype($t) = :from($t) "enemy {$t}";
```

**`serialize_for_each_predicate` has 28 match arms** that produce specialized text for "for each X" contexts. Each needs its own phrase since the grammar differs from the standard predicate forms (no articles, different phrasing). This is a significant phrase count — budget for ~30 new phrases just for this function.

**`serialize_for_count_expression` has 15 arms** using `text_formatting::card_predicate_base_text()` directly. These must be migrated simultaneously with `text_formatting` deletion. Each arm needs a phrase for participial constructions like "ally abandoned this turn" — these require gender agreement in Russian/German, making them high-value localization points.

#### Step 2: Refactor predicate_serializer return types

Change all public functions to return `Phrase` instead of `String`. The private `your_predicate_formatted()` and `enemy_predicate_formatted()` currently return `FormattedText` — change them to return `Phrase` directly.

**Function-by-function plan:**
- `serialize_predicate()` → `Phrase` (11 arms)
- `serialize_predicate_plural()` → `Phrase` (11 arms)
- `predicate_base_text()` → `Phrase` (11 arms)
- `serialize_card_predicate()` → `Phrase` (17 arms)
- `serialize_card_predicate_without_article()` → `Phrase` (6 arms)
- `serialize_card_predicate_plural()` → `Phrase` (17 arms)
- `serialize_cost_constraint_only()` → `Phrase` (3 arms)
- `serialize_for_each_predicate()` → `Phrase` (28 arms)
- `serialize_fast_target()` → `Phrase` (17 arms)
- `serialize_your_predicate()` / `serialize_enemy_predicate()` → `Phrase` (delegating)

#### Step 3: Update all 120+ call sites

Each call site currently does one of:
1. Passes predicate `String` to `format!()` — change to `p.to_string()` for now, or create a new phrase
2. Passes predicate `String` to a `strings::` phrase — change to pass `Phrase` directly
3. Calls `.with_article()` / `.without_article()` / `.plural()` — use `@a`, direct ref, or `:other`

**Coupling warning:** Many effect_serializer arms that consume predicate results ALSO have `{{ }}` escapes. The predicate→Phrase change and the `{{ }}`→`{ }` change (Task 4) are tightly coupled for these arms. The `.to_string()` bridge (pattern #1 above) provides a valid transitional approach, but expect some arms to be touched twice.

#### Step 4: Delete text_formatting.rs

Remove the `FormattedText` struct and `card_predicate_base_text()` function. Update `mod.rs` to remove the module declaration. **Must also update the 8 `card_predicate_base_text()` calls in `serialize_for_count_expression()` and the 1 call in `static_ability_serializer`.**

#### Step 5: Validate and commit

---

### Task 3: Convert to Dual-Path Rendered Output Comparison

**Files:** `test_helpers.rs`, all round-trip test files, TOML round-trip tests
**Risk:** MEDIUM — replacing the primary test safety net.
**Prerequisite:** Task 2

#### Step 1: Add dual-path rendered comparison helper

```rust
pub fn assert_rendered_match(input_text: &str, vars: &str) {
    // Path A: parse → serialize → rendered string
    let parsed = parse_ability(input_text, vars);
    let serialized = ability_serializer::serialize_ability(&parsed);
    // Path B: evaluate template text directly through RLF (uses original input + vars)
    let bindings = VariableBindings::parse(vars).unwrap();
    let rendered = rlf_helper::eval_str(input_text, &bindings);
    assert_eq!(serialized.text, rendered,  // .text is now rendered after migration
        "Rendered mismatch for input: {input_text}");
}
```

**Note:** During the transition, `serialized.text` may still contain some template syntax for arms not yet migrated. The helper must tolerate this — compare only arms that have been fully migrated, or use a flag to distinguish.

#### Step 2: Replace text-equality assertions with dual-path comparison

In each test function, replace `assert_round_trip(text, vars)` with `assert_rendered_match(text, vars)`. Keep AST round-trip tests from Task 1 alongside.

#### Step 3: Validate golden-file against dual-path

Verify that the golden-file baseline from Task 1 Step 5 matches the dual-path output. Any differences indicate a migration bug.

#### Step 4: Validate and commit

---

### Task 4: Remove `{{ }}` Escapes and VariableBindings from Leaf Serializers

**Files:** `strings.rs`, `cost_serializer.rs`, `trigger_serializer.rs`, `condition_serializer.rs`, `serializer_utils.rs`, simple `effect_serializer.rs` arms
**Risk:** LOW — straightforward mechanical changes.
**Prerequisite:** Task 3

#### Step 1: Rewrite Category B phrases in strings.rs

For every phrase containing `{{ }}`, remove the escape braces so RLF evaluates directly:
```rust
// Before: draw_cards_effect($c) = "draw {{cards($c)}}.";
// After:  draw_cards_effect($c) = "draw {cards($c)}.";
```

All 49 Category B phrases are rewritten.

#### Step 2: Pass real values to phrase functions

Replace `bindings.insert(...); strings::phrase(0).to_string()` with `strings::phrase(real_value).to_string()`.

#### Step 3: Update phrases that take $target as Phrase

After Task 2, predicates return `Phrase`. Pass directly — `impl Into<Value>` handles the conversion.

#### Step 4: Validate and commit

---

### Task 5: Migrate Remaining Effect Serializer Arms

**Files:** `strings.rs`, `effect_serializer.rs`
**Risk:** HIGH — large file (1139 lines), many arms with varied complexity.
**Prerequisite:** Task 4

#### Step 1: Add phrases for remaining StandardEffect arms

Define RLF phrases for all ~54 unmigrated arms. Key groups:

**Predicate-consuming arms (~35):** `DissolveCharacter`, `BanishCharacter`, `GainControl`, `MaterializeCharacter`, `Discover`, `ReturnToHand`, `Counterspell`, etc. Each passes `Phrase` from predicate serializer.

**For-each pattern arms (~8):** `DrawCardsForEach`, `GainEnergyForEach`, `GainPointsForEach`, etc. The for-each predicate MUST be passed as `Phrase` so translations can restructure word order.

**Collection arms (~15 including sub-arms):** `DissolveCharactersCount` (5 sub-arms), `BanishCollection` (5 sub-arms), `MaterializeCollection` (4 sub-arms), `MaterializeSilentCopy` (5 sub-arms). These have nested `CollectionExpression` matches.

**Compound pattern arms:** `BanishThenMaterialize` uses "it"/"them" pronouns that need gender agreement. Define separate phrases for singular vs plural targets so translations can select gendered pronouns via `:match`.

#### Step 2: Migrate `serialize_for_count_expression`

15 arms returning quantity descriptions. Each needs its own RLF phrase for participial agreement: "ally abandoned this turn" requires gender-agreeing participle in Russian/German. **These depend on `text_formatting::card_predicate_base_text()` being replaced** (done in Task 2 Step 4).

#### Step 3: Migrate helper functions

- `serialize_gains_reclaim` (34 lines) + `serialize_void_gains_reclaim` (82 lines) — most complex compound pattern. 8 `CollectionExpression` branches × optional cost × optional "this turn" suffix × predicate. Needs ~8+ separate RLF phrases.
- `serialize_allied_card_predicate` / `serialize_allied_card_predicate_plural` (2 arms each)

#### Step 4: Migrate `serialize_effect_with_context` structural logic

**This is the structural heart of the serializer.** Key patterns to extract as RLF phrases:

| Pattern | Current code | RLF phrase needed |
|---------|-------------|-------------------|
| "you may " prefix | hardcoded string | `you_may_prefix` |
| " to " connector | hardcoded in format! | `to_connector` |
| ", then " joiner | hardcoded string | `then_joiner` (may already exist) |
| " and " joiner | hardcoded string | `and_joiner` |
| ". " sentence joiner | hardcoded | `sentence_joiner` |
| Modal "{choose_one}" | template string | already exists |
| Modal "{bullet} " | template string | already exists |
| `trim_end_matches('.')` | String operation | **See note** |

**Period trimming concern:** `trim_end_matches('.')` is used in 6 places to strip trailing periods before joining with ", then ". After migration, the period is produced by RLF phrases. Two approaches: (a) effect phrases omit trailing periods (added by `period_suffix` at assembly level), or (b) keep trim_end_matches on rendered Strings. Approach (a) is better for i18n (different languages use different sentence-ending punctuation). **Convention: effect phrases MUST NOT include trailing periods. Punctuation is added at assembly level via `period_suffix` / `full_width_period_suffix` phrases.**

#### Step 5: Validate and commit

---

### Task 6: Migrate Static Ability Serializer (Atomic with Task 5)

**Files:** `strings.rs`, `static_ability_serializer.rs`
**Risk:** MEDIUM — circular dependency with effect_serializer.
**Prerequisite:** Task 4. **Must be coordinated with Task 5** due to circular dependency.

#### Step 1: Add static ability phrases to strings.rs

Define phrases for all 23 `StandardStaticAbility` variants. Specific concerns:
- `PlayForAlternateCost` and `PlayFromVoid` have complex branching (card_type context, optional additional_cost, optional if_you_do effect) — 3-4 phrase patterns each
- ~10 occurrences of hardcoded "this card"/"this character" should become RLF terms (`this_card`, `this_character`) for gender-aware translations

#### Step 2: Migrate `serialize_standard_static_ability`

Replace all `format!()` arms with `strings::` phrase calls. Handle condition placement logic in `serialize_static_ability`: conditions can be prepended or appended depending on type. This logic stays in Rust code.

#### Step 3: Validate and commit

---

### Task 7: Migrate Ability Serializer

**Files:** `ability_serializer.rs`, `strings.rs`
**Risk:** LOW — top-level orchestrator, straightforward once everything below returns Phrase.
**Prerequisite:** Tasks 5 and 6

#### Step 1: Add ability structure phrases

Add remaining structural phrases. Most already exist from Phase 1.5. Ensure these exist:
- `once_per_turn_prefix`, `until_end_of_turn_prefix`, `fast_prefix`, `cost_effect_separator`
- `modal_choice_prefix` for bullet/newline structure
- `you_may_prefix`, `to_connector` for `ListWithOptions`

#### Step 2: Refactor ability_serializer.rs

Replace `capitalize_first_letter()` calls with `@cap` in phrase templates. Each ability pattern (`Triggered`, `Event`, `Activated`, `Named`, `Static`) gets its own assembly logic using structural phrases.

**`serialize_ability_effect()` and `serialize_modal_choices()`** — these also need migration (3 functions total in ability_serializer.rs that produce `SerializedAbility`).

#### Step 3: Validate and commit

---

### Task 8: Remove eval_str and VariableBindings from Serializer

**Files:** `ability_serializer.rs`, `rlf_helper.rs`, `card_rendering.rs`, `dreamwell_card_rendering.rs`, `modal_effect_prompt_rendering.rs`
**Risk:** MEDIUM — atomic switchover of 4 display call sites + 42 function signature changes.
**Prerequisite:** Task 7

#### Step 1: Change SerializedAbility

Remove the `variables` field. The `text` field now contains final rendered text.

#### Step 2: Remove VariableBindings from serializer functions

Remove the `bindings: &mut VariableBindings` parameter from all 42 serializer function signatures. Remove `VariableBindings::new()` creation.

#### Step 3: Update display layer call sites

All 4 call sites change from `eval_str(serialized.text, serialized.variables)` to just `serialized.text`.

#### Step 4: Preserve eval_str as test-only helper

Move `rlf_helper::eval_str()` and `build_params()` to a test-only module. They serve as the Path B oracle for the dual-path rendered output comparison. The `subtype_phrase()` and `figment_phrase()` helper functions must remain available to both production and test code.

**CRITICAL:** `VariableBindings` and `VariableValue` CANNOT be deleted. They are used by the parser (`parser_substitutions.rs`) and throughout test infrastructure. Only the serializer's dependency is removed.

#### Step 5: Validate and commit

---

### Task 9: Remove Capitalization Helpers

**Files:** `serializer_utils.rs`
**Risk:** LOW — straightforward deletion.
**Prerequisite:** Task 8

#### Step 1: Verify no remaining callers

Grep for `capitalize_first_letter` and `lowercase_leading_keyword`. After Tasks 7-8, there should be zero callers.

#### Step 2: Delete the functions

Remove `capitalize_first_letter`, `capitalize_string`, `title_case_keyword`, `is_capitalizable_keyword`, and `lowercase_leading_keyword`.

#### Step 3: Validate and commit

---

### Task 10: Add `:from` to Compound Phrases and Audit Keywords

**Files:** `strings.rs`
**Risk:** LOW — additive changes only, no behavioral impact on English.
**Prerequisite:** Task 2

Ensures English source phrases support proper tag propagation for localization.

#### Step 1: Add `:from($entity)` to compound predicate phrases

```rust
allied($entity) = :from($entity) "allied {$entity}";
enemy_modified($entity) = :from($entity) "enemy {$entity}";
```

Without `:from`, downstream `:match` for gender/animacy would fail because tags are lost. When running in Russian, `allied(ally)` inherits `:masc :anim` from the Russian `ally` definition. When running in English, it inherits `:an` from the English definition.

#### Step 2: Verify keyword terms have separate imperative/participial forms

Audit: `dissolve`/`dissolved`, `banish`/`banished`, `materialize`/`materialized`, `reclaim`/`reclaimed`, `prevent`/`prevented`, `discover`/`discovered`. Each needs distinct terms because Spanish/Portuguese need different verb forms; Russian/German need gender-agreeing participles.

#### Step 3: Validate and commit

---

### Task 11: RLF Framework Changes for Localization

**Files:** RLF crate (`~/rlf/`) — `transforms.rs`, stdlib definitions
**Risk:** LOW — additive changes following established patterns.
**Prerequisite:** None (can be done in parallel with Phase 2 tasks)

Implements RLF framework changes identified by i18n review (Section 9.8). Ordered by priority:

| Change | Priority | Lines | Description |
|--------|----------|-------|-------------|
| Turkish buffer consonant `@inflect` | HIGH | ~40 | y/n/s insertion for vowel-ending stems |
| Korean `@particle` HTML stripping | HIGH | ~10 | Strip trailing `</color>` etc. |
| Portuguese `@para` | HIGH | ~30 | "a" + article contraction (ao/à/aos/às) |
| CJK `@count:word` | MEDIUM | ~30/lang | Number words instead of digits for 1-10 |
| Portuguese `@por` | MEDIUM | ~30 | "por" + article contraction |
| Portuguese `@um:other` | MEDIUM | ~10 | Plural indefinite articles |
| Korean particle contexts | MEDIUM | ~15 | `:and`/`:copula`/`:dir` |
| German `@ein` empty plural | LOW | ~5 | Return empty string for plural |
| Spanish `@del`/`@al` | LOW | ~40 | Preposition+article contractions |
| Japanese `:tai` counter | LOW | ~2 | Counter for characters |
| French `@un:other` | LOW | ~5 | Plural indefinite "des" |

---

## 7. Design Decisions

### 7.1 Testing: Dual-Path Rendered Output Comparison

See Section 5.1-5.2 for the full strategy. Core principle:

```
Path A: template_text → parse() → AST → serialize() → rendered string
Path B: template_text → eval_str() (test-only)       → rendered string
Assert: Path A == Path B
```

Both paths start from the same template-format input. `eval_str` is preserved as a test-only function. The golden-file snapshot catches cases where both paths produce the same wrong output.

### 7.2 Phrase Parameter Types — VERIFIED

The `rlf!` macro generates `impl Into<Value>` parameters (verified in `rlf-macros/src/codegen.rs:88`). `Phrase` implements `From<Phrase> for Value`. Therefore `Phrase` values can be passed directly to any phrase function.

### 7.3 register_source_phrases() — VERIFIED

With `global-locale`, registration is automatic via `__RLF_REGISTER.call_once(...)`. No explicit registration needed after Phase 2 removes `eval_str()`.

### 7.4 `:from` + `:match` Combination — VERIFIED

The RLF evaluator has a dedicated `eval_from_with_match()` function (verified in `rlf/crates/rlf/src/interpreter/evaluator.rs`). `:from` determines inherited structure, `:match` branches within each variant's evaluation. This is critical for Russian/German compound phrases.

### 7.5 Effect Phrase Period Convention

Effect phrases MUST NOT include trailing periods. Punctuation is language-specific (`.` for Western, `。` for CJK). The assembly level adds punctuation via translatable `period_suffix` phrases. This also prevents double punctuation when effects are joined by `then_joiner`.

### 7.6 `serialize_effect_with_context` Branching Complexity

This function has **4 distinct branches** for `Effect::List` plus separate logic for `Effect::WithOptions`, `Effect::ListWithOptions`, and `Effect::Modal`. Key structural patterns:

1. All optional + all have trigger cost → "you may [cost] to [effect1] and [effect2]."
2. Not optional + all have trigger cost → "[cost] to [effect1] and [effect2]."
3. All optional + no trigger cost → "you may [effect1], then [effect2]."
4. Default (mandatory, mixed) → "[effect1], then [effect2]." (triggered) or "[Effect1]. [Effect2]." (event)

Each of these patterns needs its own set of structural phrases. The Rust branching logic stays in Rust, but ALL connectors ("you may", "to", "and", "then", period) must be RLF phrases.

---

## 8. Risk Assessment

| Task | Risk | Key Concerns |
|------|------|-------------|
| 1. AST round-trip + golden file | HIGH | ~29 types need PartialEq; golden file generation infrastructure |
| 2. Predicate → Phrase | HIGH | 803 lines, 12 functions, 120+ call sites; text_formatting deletion couples to effect_serializer |
| 3. Dual-path test conversion | MEDIUM | Must preserve eval_str as test oracle; transition period with mixed template/rendered output |
| 4. Remove `{{ }}` escapes | LOW | Mechanical; 49 phrases to update |
| 5. Remaining effect arms | HIGH | 54 arms; `serialize_effect_with_context` has 4 complex branches; period convention change |
| 6. Static ability serializer | MEDIUM | 23 variants; `PlayForAlternateCost`/`PlayFromVoid` are complex; circular dep |
| 7. Ability serializer | LOW | 3 functions; straightforward once dependencies are done |
| 8. Remove eval_str | MEDIUM | Atomic switchover of 4 display sites + 42 signature changes; must preserve VariableBindings for parser |
| 9. Remove capitalization | LOW | 23 call sites total |
| 10. `:from` + keyword audit | LOW | Additive changes only |
| 11. RLF framework changes | LOW | Independent of Dreamtides code |

---

## 9. Multilingual Design Considerations

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

### 9.3 Personal "a" (Spanish)

"dissolve an enemy" → "disolver **a** un enemigo". Handled by `:match` on `:anim` tag.

### 9.4 Chinese Classifiers and Word Order

Different classifiers per noun (张 for cards, 个 for characters). Different word order handled entirely in translation files.

### 9.5 German Separable Verbs

"auflösen" splits: "Löse ... auf". Handled by phrase templates.

### 9.6 Tag System Design

All grammatical tags are **language-specific** and live in **translation files**, not in the English source. The English source only carries `:a`/`:an` tags (English grammar). All other grammatical metadata (gender, animacy, classifiers, case) is defined by each translation file on its own terms.

| Tag | Defined in | Purpose |
|-----|-----------|---------|
| `:a` / `:an` | English source | English indefinite article |
| `:anim` / `:inan` | Translation files (es, ru) | Animacy |
| `:masc` / `:fem` / `:neut` | Translation files (es, ru, de, pt, fr, ar) | Gender |
| `:zhang` / `:ge` etc. | Chinese translation | Classifier tags |
| `:mai` / `:tai` etc. | Japanese translation | Counter tags |
| `:sun` / `:moon` | Arabic translation | Definite article assimilation |
| `:front` / `:back` | Turkish translation | Vowel harmony |
| `:vowel` | French translation | Elision trigger |

### 9.7 RLF Feature Verification Checklist

Before writing translation files, verify these RLF features:
- [ ] `@count` transform for CJK classifiers
- [ ] `@count:word` modifier for CJK number words
- [ ] `@der`/`@ein` for German articles + case
- [ ] `@ein` empty string for plural context
- [ ] `@el`/`@un` for Spanish articles
- [ ] `@del`/`@al` for Spanish contractions
- [ ] `@o`/`@um` for Portuguese articles
- [ ] `@um:other` plural indefinite articles
- [ ] `@para` Portuguese "a" + article contraction
- [ ] `@por` Portuguese "por" + article contraction
- [ ] `@le`/`@un` for French articles
- [ ] `@le:other`/`@un:other` French plurals
- [ ] Multi-parameter `:match`
- [ ] `:from` with multi-dimensional variant propagation
- [ ] `:from` + `:match` combination (VERIFIED — `eval_from_with_match`)
- [ ] `@cap` is no-op on CJK and Arabic script
- [ ] `@cap` skips leading HTML markup tags
- [ ] `@cap` locale-sensitive (Turkish I/ı)
- [ ] `@inflect` Turkish buffer consonant insertion
- [ ] `@inflect` accepts Phrase parameters
- [ ] `@particle` strips trailing HTML markup (Korean)
- [ ] `@particle` additional contexts `:and`/`:copula`/`:dir` (Korean)
- [ ] `@particle` digit ending lookup table (Korean)
- [ ] `@al` reads `:sun`/`:moon` tags (Arabic)
- [ ] `:from` propagates `:vowel` tag (French)

### 9.8 Cross-Language Design Conventions

1. **Effect phrases MUST NOT include trailing periods.** Punctuation added at assembly level.
2. **Each phrase controls its own capitalization via `@cap`.** Rust code MUST NOT apply `capitalize_first_letter()` to rendered strings.
3. **Keyword terms MUST have separate imperative and participial forms.** `dissolve` vs `dissolved`.
4. **Structural connectors MUST be named RLF phrases.** All punctuation, separators, joiners.
5. **`text_number` is English-specific.** Gendered languages inline number words via `:match`.
6. **All structural connectors in `serialize_effect_with_context()` MUST be RLF phrases.** Including "you may", "to", modal formatting, pronouns.
7. **"this card"/"this character" MUST be RLF terms.** For gender-aware translations.

---

## 10. Migration Ordering Summary

```
Task 1: AST Round-Trip Tests + Golden File Baseline
    │
    ▼
Task 2: Predicate Serializer → Phrase  ◄── HARDEST STEP
    │
    ▼
Task 3: Convert to Dual-Path Rendered Comparison
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

Tasks 1-9 are sequential — each depends on the previous. Task 10 depends on Task 2. Task 11 is independent.

---

## Appendix A: File Reference

| File | Path | Lines |
|------|------|-------|
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
| RLF evaluator | `~/rlf/crates/rlf/src/interpreter/evaluator.rs` | — |
| RLF codegen | `~/rlf/crates/rlf-macros/src/codegen.rs` | — |

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
| RU | "Растворите вражеского Древнего." | Acc on BOTH adjective and noun (masc.anim) |
| ES | "Disuelve a un Antiguo enemigo." | Personal "a", reversed adjective order |
| PT-BR | "Dissolva um Ancião inimigo." | Reversed adjective order |
| DE | "Löse einen feindlichen Uralten auf." | Separable verb, acc article, adj declension |
| JA | "敵のエンシェントを消滅させる。" | SOV, particle を marks object |
| AR | "حَلّ عتيقاً معادياً." | Masc. imperative, accusative ending |
| TR | "Bir düşman Kadimi erit." | `@inflect:acc` on target, SOV |
| KO | "적 고대인을 해체한다." | `@particle:obj` selects 을, SOV |
| FR | "Dissolvez l'Ancien ennemi." | `@le` reads `:vowel`, elision l' |

### "Banish an enemy, then materialize it." (compound effect with pronoun)

| Language | Output | Key Features |
|----------|--------|--------------|
| EN | "Banish an enemy, then materialize it." | "it" is fixed |
| RU | "Изгоните врага, затем материализуйте его." | "его" (masc acc) from enemy's `:masc` |
| ES | "Destierra a un enemigo, luego materialízalo." | "-lo" (masc) clitic attached to verb |
| DE | "Verbanne einen Feind, dann materialisiere ihn." | "ihn" (masc acc) |
| FR | "Bannissez un ennemi, puis matérialisez-le." | "le" (masc) |

This case study demonstrates why compound effects with pronouns need the antecedent's gender tags. The pronoun phrase uses `:match($target)` to select the correct form.

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
