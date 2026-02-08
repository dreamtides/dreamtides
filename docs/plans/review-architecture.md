# Architecture Review: Serializer RLF Migration Plan

**Reviewer:** Architecture Review Agent
**Date:** 2026-02-07

---

## Executive Summary

The plan proposes migrating hardcoded English strings in 6 serializers to RLF phrase calls, with a Phase 1 "escaped braces" approach that preserves round-trip test compatibility, followed by a future Phase 2 where serializers return `Phrase` objects. After careful analysis, I recommend **skipping Phase 1 entirely and implementing Phase 2 directly**. Phase 1 creates a deceptive intermediate state that appears localization-ready but isn't, introduces a novel escape-sequence pattern that will confuse developers, and doubles the total migration effort. Phase 2 is not significantly harder to implement and delivers the real architectural benefits.

---

## 1. The Two-Layer Anti-Pattern: Escaped Braces are Actively Harmful

### The Problem

The plan's central mechanism is having RLF phrases output *template text* via escaped braces:

```rust
// Phase 1: strings.rs
banish_another_in_void = "{{Banish}} another card in your void";
// Evaluates to: "{Banish} another card in your void"
// Which is then later evaluated AGAIN by eval_str()
```

This creates a **double-evaluation pipeline**: RLF evaluates the phrase (producing template text), then `eval_str()` evaluates the template text again (producing rendered text). This is an anti-pattern for several reasons:

**1a. It's a string-generating-strings pattern.** The RLF phrase is not doing localization — it's constructing an intermediate representation that happens to look like RLF source code. This is the moral equivalent of generating SQL strings instead of using parameterized queries. You get all the fragility of string interpolation with none of the benefits of structured composition.

**1b. The `$parameter` inside `{{...}}` problem is a landmine.** The plan itself identified this issue and had to reason through RLF escape semantics to determine whether `{{count_allies($a)}}` would evaluate `$a` or not. This is exactly the kind of subtle, non-obvious behavior that causes bugs months later when someone doesn't understand the escaping rules. The plan concludes that `$a` is NOT evaluated inside `{{...}}` — but this means the RLF phrase cannot substitute its own parameter values into the template it generates. The phrase and the eval_str() system must agree on variable names, creating a hidden coupling.

**1c. It makes the "language-agnostic serializer" claim false.** Consider:

```rust
banish_from_hand($target) = "{{Banish}} {$target} from hand";
```

A Spanish translator would need to write something like `"{{Desterrar}} {$target} de la mano"`. But the `{$target}` value is a pre-rendered English string from `predicate_serializer`. The translator gets a phrase that interpolates English text. This isn't localization — it's English-structure-with-translatable-verbs.

**1d. The `FormattedText` / article logic stays in Rust.** Predicates still return `String` with English articles ("a card", "an ally"). No amount of RLF phrase wrapping changes this. The English grammar is still hardcoded in the predicate serializer, the trigger serializer's comma-space conventions, and the effect serializer's period-placement logic.

### Recommendation

**Skip Phase 1. Go directly to Phase 2 (Phrase-based composition).** The escaped-braces approach creates substantial migration work that will all be thrown away in Phase 2. Every phrase written with `{{Banish}}` will need to be rewritten as `{Banish}` (a real RLF reference) in Phase 2. Every caller that does `.to_string()` will need to change to return `Phrase`. The total effort is approximately:

- Phase 1 only: ~16 tasks × moderate effort each
- Phase 2 only: ~16 tasks × moderate-to-high effort each
- Phase 1 + Phase 2: ~32 tasks total (nearly double)

Phase 2 is harder per-task, but the total effort is much less than doing both phases.

---

## 2. Abstraction Boundaries: Where Should Language Logic Live?

### Current State

Language-specific logic currently lives in three places:

1. **Serializer code (Rust):** English sentence structure, articles ("a"/"an"), verb-object ordering, pluralization rules, comma placement, capitalization
2. **`text_formatting.rs`:** `FormattedText` struct with `with_article()`/`plural()` methods — pure English grammar
3. **`strings.rs` (RLF):** Keyword formatting, count-dependent plurals, subtype terms

### Plan's Claim vs. Reality

The plan claims "ALL language-specific logic in RLF" but this is aspirational, not what Phase 1 delivers. After Phase 1:

- `FormattedText` still exists and handles articles
- `capitalize_first_letter()` still handles English capitalization of keyword-initial strings
- `lowercase_leading_keyword()` still exists
- Sentence structure ("when you play X, effect" with trailing comma) remains in Rust
- Predicate pluralization ("card" → "cards", "ally" → "allies") remains in Rust

### Recommendation

The correct abstraction boundary is:

- **Rust serializers:** Map AST → semantic intent (what kind of effect, what kind of target)
- **RLF phrases:** All text production, including articles, pluralization, word order

This is only achievable with Phase 2, where predicates return `Phrase` objects that carry tags (`:a`, `:an`, `:masc`, etc.) and variants (one/other). The `@a` transform, `:from()` inheritance, and parameterized selection are specifically designed for this — it's the whole point of RLF.

---

## 3. Scalability of `strings.rs`

### Scale Analysis

The plan adds roughly:
- ~30 cost phrases (Task 1)
- ~20 trigger phrases (Task 2)
- ~10 condition phrases (Task 3)
- ~40 predicate phrases (Tasks 4-6)
- ~10 operator/utility phrases (Task 7)
- ~80 effect phrases (Tasks 8-12)
- ~20 structural phrases (Task 12)
- ~20 static ability phrases (Task 13)
- ~5 ability phrases (Task 14)

Total: **~235 new phrases** added to the existing ~100 definitions in `strings.rs`. The file would grow from ~460 lines to **~900+ lines**.

### Assessment

A single 900-line file is manageable for a Rust developer, but it's problematic for a *translator*. A translator working on Russian would need to navigate 300+ phrases with no semantic grouping beyond comments. More importantly:

- Many phrases are structurally identical (e.g., `dissolve_all`, `banish_all`, `materialize_all` all follow the pattern `"{{keyword}} all {$target}."`)
- Some phrase names are confusingly similar (`energy_cost_value` vs `energy_cost` vs `energy`)
- The escaped braces make phrases harder to read than they need to be

### Recommendation

**Organize phrases into separate files by semantic domain**, matching the serializer structure:

```
rules_engine/src/strings/src/
  strings.rs          # Core game symbols, keywords, UI text (existing)
  cost_strings.rs     # Cost phrases
  trigger_strings.rs  # Trigger phrases
  effect_strings.rs   # Effect phrases
  predicate_strings.rs # Predicate phrases
  static_strings.rs   # Static ability phrases
```

Each file would use its own `rlf!` block. RLF definitions in one block can reference definitions in another via `use` (check whether RLF supports cross-block references — if not, this is a feature to add before the migration). This matches the serializer structure 1:1 and gives translators natural units of work.

**However**: If going with Phase 2 (Phrase-based composition), the number of needed phrases may actually be *smaller*, because phrases can compose more naturally. Instead of 8 `dissolve_*` variants, you'd have one `dissolve($target)` that takes a `Phrase` parameter. The combinatorial explosion of `{action}_{quantity}_{target}` triples becomes `action(quantity_phrase, target_phrase)`.

---

## 4. Predicate Return Type: Start with Phrase from Day One

### The Plan's Approach

The plan has predicates return `String` throughout Phase 1, with a note to switch to `Phrase` in Phase 2. This means:

- Phase 1: `serialize_predicate() -> String` → passed as `$target` to RLF phrases → interpolated as opaque text
- Phase 2: `serialize_predicate() -> Phrase` → passed with tags/variants → enables gender agreement, case selection

### Cost Analysis

**Cost of returning String now and Phrase later:**
- Every caller of `serialize_predicate`, `serialize_predicate_plural`, `predicate_base_text`, `serialize_your_predicate`, `serialize_enemy_predicate`, `serialize_card_predicate`, etc. (30+ call sites across effect/cost/trigger/condition/static serializers) gets migrated twice
- All round-trip test expectations get updated twice
- All phrase signatures that accept `$target: String` become `$target: Phrase` — every phrase definition changes

**Cost of returning Phrase from day one:**
- Single migration per call site
- Need to decide what tags predicates carry (`:a` vs `:an` for English; `:masc`/`:fem` for gendered languages)
- The `to_string()` call moves to the top-level serializer instead of each intermediate function

### Recommendation

**Return `Phrase` from predicates from day one.** This is the single most impactful decision for reducing total migration effort. The `FormattedText` struct already tracks `starts_with_vowel_sound` — this maps directly to `:a`/`:an` tags. The `plural` field maps to `{ one: "...", other: "..." }` variants. `FormattedText` is already a poor man's `Phrase`; replacing it is a clean upgrade.

The migration path would be:
1. First: Migrate `predicate_serializer` to return `Phrase` objects (with tags and variants)
2. Then: Migrate cost/trigger/condition/effect/static serializers to accept `Phrase` parameters
3. Finally: Top-level `ability_serializer` calls `.to_string()` on the final composed `Phrase`

This front-loads the hardest work but eliminates the entire Phase 1 → Phase 2 transition.

---

## 5. Developer Experience: Adding a New Card Effect

### Current DX (status quo)

Developer adds a new `StandardEffect::BlinkCharacter { target }` variant:

1. Add the variant to `standard_effect.rs`
2. In `effect_serializer.rs`, add:
   ```rust
   StandardEffect::BlinkCharacter { target } => {
       format!("{{banish}} {}, then {{materialize}} it.",
           predicate_serializer::serialize_predicate(target, bindings))
   }
   ```
3. In the parser, add a pattern to recognize "banish X, then materialize it"
4. Add a round-trip test: `assert_round_trip("{Banish} an enemy, then {materialize} it.", "")`
5. Run `just review`

**Verdict:** Simple. One format string, copy-paste from similar effects.

### Post-Phase-1 DX (plan's approach)

Same developer, same effect:

1. Add the variant to `standard_effect.rs`
2. In `strings.rs`, add:
   ```rust
   blink_character($target) = "{{banish}} {$target}, then {{materialize}} it.";
   ```
3. In `effect_serializer.rs`, add:
   ```rust
   StandardEffect::BlinkCharacter { target } => {
       let target_text = predicate_serializer::serialize_predicate(target, bindings);
       strings::blink_character(target_text).to_string()
   }
   ```
4. In the parser, add the pattern
5. Add a round-trip test
6. Run `just review`

**Verdict:** Slightly more complex — developer must edit two files instead of one, learn the `{{...}}` escaping convention, and understand that `$target` is a pre-rendered String. The phrase definition is less readable than the format string it replaces due to escaped braces.

### Post-Phase-2 DX (recommended approach)

1. Add the variant to `standard_effect.rs`
2. In `strings.rs` (or `effect_strings.rs`), add:
   ```rust
   blink_character($target) = "{banish} {$target}, then {materialize} it.";
   ```
3. In `effect_serializer.rs`, add:
   ```rust
   StandardEffect::BlinkCharacter { target } => {
       strings::blink_character(predicate_serializer::serialize_predicate(target, bindings))
   }
   ```
   (where `serialize_predicate` returns `Phrase`)
4. In the parser, add the pattern
5. Add a round-trip test that tests parse→AST separately from AST→Phrase
6. Run `just review`

**Verdict:** Same number of files, but the phrase definition is cleaner (no escaped braces), and the RLF phrase actually *does something* — it composes `Phrase` objects with their tags/variants intact. A translator can write `blink_character($target) = "{desterrar} {$target:acc}, luego {materializar:inf}lo."` and get correct Spanish.

### Key Insight

The DX difference between Phase 1 and Phase 2 is small for the developer, but the *translator* DX is vastly better in Phase 2. Phase 1 gives translators phrases with opaque String parameters — they can translate the sentence frame but not inflect the interpolated values. Phase 2 gives translators full control.

---

## 6. Alternative Architectures

### 6a. Structured Intermediate Representation

Instead of serializers returning `String` or `Phrase`, they could return a tree of `TextNode` values:

```rust
enum TextNode {
    Keyword(KeywordKind),          // dissolve, banish, etc.
    Predicate(Predicate),          // a character, an ally, etc.
    Count(u32, TextNode),          // "3 cards"
    Action(ActionKind, TextNode),  // "dissolve an enemy"
    Sequence(Vec<TextNode>),       // concatenation
    Literal(String),               // fixed text
}
```

**Pros:** Maximum flexibility. Can render to any language without RLF.
**Cons:** Reinvents RLF. The `TextNode` tree is essentially a custom AST for a localization DSL. RLF already provides this — `Phrase` is the structured intermediate representation.

**Verdict:** Unnecessary given RLF exists. Would be the right approach if you were building from scratch without RLF.

### 6b. ICU MessageFormat

ICU MessageFormat is the industry standard for localization:

```
{count, plural, one {Draw a card} other {Draw {count} cards}}.
```

**Pros:** Industry standard, wide tooling support, translator familiarity.
**Cons:** No Rust macro integration (would need runtime parsing), no compile-time validation, less ergonomic than RLF's `rlf!` macro, doesn't support the tag/transform system that RLF provides for grammatical agreement.

**Verdict:** RLF is strictly better for this use case. RLF's `:match` is ICU plural rules; RLF's `:from` and `@transforms` go beyond what ICU offers. The compile-time validation alone is worth it.

### 6c. Mozilla Fluent

Fluent is the most modern alternative:

```ftl
draw-cards = { $count ->
    [one] Draw a card
   *[other] Draw { $count } cards
}
```

**Pros:** Designed for asymmetric localization (translators can restructure), good Rust bindings (`fluent-rs`), rich plural/gender support.
**Cons:** No compile-time validation in Rust (runtime parsing only), verbose syntax for simple cases, doesn't have RLF's tag inheritance model (`:from`), separate toolchain.

**Verdict:** Fluent is a strong alternative worth evaluating if RLF didn't exist. But RLF was specifically designed for this project's needs (compile-time validation, Phrase composition, tag-based grammatical agreement). Switching to Fluent would lose the `rlf!` macro benefits.

### 6d. gettext

```
msgid "Draw %d cards."
msgstr "Ziehe %d Karten."
```

**Pros:** Extremely well-known, vast tooling ecosystem.
**Cons:** Weak plural support (requires `ngettext`), no gender/case agreement, no structured composition, printf-style interpolation is fragile.

**Verdict:** Too primitive for a card game with complex grammatical requirements. Would need extensive wrapping to handle Russian cases, Chinese classifiers, etc.

### Overall Assessment

**RLF is the right choice.** It was designed for this exact use case and provides compile-time safety, structured composition, and rich grammatical features. The question isn't "should we use RLF?" but "how should we use it?" — and the answer is Phase 2 (Phrase-based composition), not Phase 1 (escaped-brace string wrapping).

---

## 7. The Round-Trip Problem

### Current Testing Model

```
assert_round_trip("{Dissolve} an enemy.", "");
// 1. Parse "{Dissolve} an enemy." → AST: StandardEffect::DissolveCharacter { target: Enemy(Character) }
// 2. Serialize AST → "{Dissolve} an enemy."
// 3. Assert: original text == serialized text
```

This tests **parse∘serialize = identity**, which is a strong property but has significant costs:

**7a. It couples serializer output to parser input format.** The serializer must produce text that the parser can re-parse. This means the serializer can't produce rendered text (with `<color>` tags) or even change whitespace without breaking tests.

**7b. It makes format changes expensive.** Changing from `{Dissolve}` to `{dissolve}` (lowercase) requires updating every test that contains the word "Dissolve". The plan's Phase 1 carefully preserves exact string equality, which is why it needs the escaped-braces hack.

**7c. It doesn't test what the player sees.** The round-trip test verifies internal template text, not the rendered output that appears on cards. A bug in `eval_str()` that breaks rendering would not be caught.

### Recommended Testing Architecture

Split into three test categories:

**1. Parse Tests (text → AST):**
```rust
fn test_parse_dissolve_enemy() {
    let ast = parse_ability("{Dissolve} an enemy.", "");
    assert_eq!(ast, Ability::Event(EventAbility {
        effect: Effect::Standard(StandardEffect::DissolveCharacter {
            target: Predicate::Enemy(CardPredicate::Character)
        })
    }));
}
```

**2. Serialize Tests (AST → Phrase):**
```rust
fn test_serialize_dissolve_enemy() {
    let phrase = serialize_ability(&Ability::Event(EventAbility {
        effect: Effect::Standard(StandardEffect::DissolveCharacter {
            target: Predicate::Enemy(CardPredicate::Character)
        })
    }));
    assert_eq!(phrase.to_string(), "<color=#AA00FF>Dissolve</color> an enemy.");
}
```

**3. Render Tests (end-to-end display):**
```rust
fn test_render_dissolve_enemy() {
    let rendered = render_card_text("{Dissolve} an enemy.", "");
    assert_eq!(rendered, "<color=#AA00FF>Dissolve</color> an enemy.");
}
```

**Benefits:**
- Parse tests catch parser regressions without depending on serializer changes
- Serialize tests verify actual rendered output (what players see)
- No coupling between parse and serialize formats
- Each test is independently updatable

**Cost:**
- More test boilerplate (constructing AST values)
- Need to update existing ~200+ round-trip tests to the new pattern
- Loss of the elegant one-liner `assert_round_trip` pattern

### Pragmatic Recommendation

**Keep round-trip tests for the parser migration, but plan their restructuring.** The round-trip tests are valuable during migration because they catch accidental output changes. But they should not be the *reason* for architectural compromises (like the escaped-braces pattern). After the migration:

1. Convert round-trip tests to parse-only tests (assert the AST, not the re-serialized text)
2. Add serialize tests that compare against rendered output
3. Keep the TOML round-trip tests (`cards_toml_round_trip_tests.rs`) as integration smoke tests

The TOML round-trip tests are the most valuable — they test every card in the game. But they can be adapted to compare `parse(text) == parse(serialize(parse(text)))` instead of `text == serialize(parse(text))`, which allows serializer output to change format without breaking the test.

---

## 8. Recommended Architecture: Phase 2 Direct Implementation

Based on the analysis above, here is the recommended approach:

### Step 1: Migrate Predicates to Return Phrase (Foundation)

This is the critical first step. Convert `FormattedText` to `Phrase`:

```rust
// predicate_serializer.rs - NEW
pub fn serialize_predicate(predicate: &Predicate, bindings: &mut VariableBindings) -> Phrase {
    match predicate {
        Predicate::Enemy(CardPredicate::Character) => strings::enemy(), // returns Phrase with :an tag
        Predicate::Your(CardPredicate::Character) => strings::ally(),   // returns Phrase with :an tag
        Predicate::Any(CardPredicate::Character) => strings::character(), // etc.
        // ...
    }
}
```

Add the necessary terms to `strings.rs`:
```rust
enemy = :an { one: "enemy", other: "enemies" };
character = :a { one: "character", other: "characters" };
// etc.
```

### Step 2: Migrate Serializers Bottom-Up

Convert each serializer to compose `Phrase` objects:

```rust
// effect_serializer.rs - NEW
StandardEffect::DissolveCharacter { target } => {
    let target_phrase = predicate_serializer::serialize_predicate(target, bindings);
    strings::dissolve_target(target_phrase)
}
```

```rust
// strings.rs
dissolve_target($target) = "{@cap dissolve} {@a $target}.";
```

This phrase is real localization: `{@a $target}` reads the `:a`/`:an` tag from the target Phrase and produces the correct article. A Russian translator writes `dissolve_target($target) = "Уничтожить {$target:acc}.";` and gets accusative case from the target's variant.

### Step 3: Update ability_serializer to Call .to_string() at the Top

```rust
pub fn serialize_ability(ability: &Ability) -> SerializedAbility {
    let phrase = match ability {
        Ability::Event(event) => serialize_effect_phrase(&event.effect, &mut variables),
        // ...
    };
    SerializedAbility { text: phrase.to_string(), variables }
}
```

### Step 4: Remove eval_str() from the Display Layer

Since the serializer now returns fully rendered text, `eval_str()` becomes unnecessary. The display layer simply uses the serializer output directly.

### Step 5: Restructure Tests

Convert round-trip tests to parse-and-serialize tests that compare against rendered output.

---

## 9. Risk Assessment

| Risk | Phase 1 | Phase 2 Direct | Mitigation |
|------|---------|----------------|------------|
| Large blast radius | Medium (string changes only) | High (return types change) | Migrate one serializer at a time, keep tests green |
| Intermediate broken state | Low (strings stay the same) | Medium (callers must adapt simultaneously) | Use a trait/enum adapter during transition |
| Developer confusion | High (escaped braces are unintuitive) | Low (standard RLF patterns) | Good documentation |
| Translation quality | Low (opaque String params) | High (Phrase params with tags) | This is the goal |
| Total effort | ~200 phrases × 2 migrations | ~150 phrases × 1 migration | Less total work |

---

## 10. Summary of Recommendations

1. **Skip Phase 1.** The escaped-braces approach is an anti-pattern that doubles migration effort.
2. **Go directly to Phase 2** (Phrase-based composition). Predicates return `Phrase` from day one.
3. **Split `strings.rs`** into domain-specific files matching the serializer structure.
4. **Restructure round-trip tests** to test parse→AST and AST→rendered-text separately.
5. **Migrate bottom-up:** predicates first, then costs/triggers/conditions, then effects, then static abilities, then the top-level ability serializer.
6. **Remove `eval_str()`** from the display layer once serializers produce final rendered text.
7. **Remove `FormattedText`** entirely — it's a poor man's `Phrase`.
8. **Remove `VariableBindings`** from the serializer return path (they become internal to predicate serialization, not exposed to callers).

The end state is a clean architecture where Rust code maps semantic intent to named RLF phrases, RLF handles all text production, and the display layer simply renders the result. This is achievable in a single migration phase with less total effort than the two-phase plan.
