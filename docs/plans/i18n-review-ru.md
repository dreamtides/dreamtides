# Russian (Русский) i18n Review of Serializer RLF Migration Plan

**Reviewer:** Russian language specialist
**Date:** 2026-02-08
**Scope:** Review migration plan for Russian localizability — 3 genders, 6 cases, complex plurals, animacy, participle agreement

---

## Executive Summary

The migration plan is **well-designed for Russian** and demonstrates strong awareness of Slavic linguistic requirements. The RLF framework's multi-dimensional variant system (`nom.one`, `acc.many`, etc.), `:match` branching, `:from` inheritance, and tag-based selection are sufficient to express all Russian grammatical patterns. The existing `APPENDIX_RUSSIAN_TRANSLATION.md` proves this with concrete working examples.

**Verdict: No Rust code changes needed for Russian support**, provided the plan's key architectural decisions (predicates returning `Phrase`, not `String`) are implemented as specified.

However, I identify **5 concrete issues** where the current plan could create friction or require workarounds. None are blockers, but addressing them would make Russian translation significantly smoother.

---

## Issue 1: Missing Animacy Tags on English Source Terms (MEDIUM)

### Problem

Russian accusative case depends on animacy: for masculine animate nouns, accusative = genitive; for masculine inanimate nouns, accusative = nominative. The plan acknowledges this (Section 9.6 Tag System Design) and states `:anim`/`:inan` tags should be on English source terms.

**Current state in `strings.rs`:** Character subtypes (agent, ancient, warrior, etc.) carry only `:a`/`:an` tags — **no `:anim`/`:inan` tags**. Base terms `card`, `ally` also lack animacy tags.

```rust
// Current (strings.rs)
agent = :an{ one: "Agent", other: "Agents" };
card = :a{ one: "card", other: "cards" };
ally = :an{ one: "ally", other: "allies" };
```

The Russian translation appendix shows these terms **with** animacy tags:
```
// ru.rlf (appendix example)
character = :masc :anim { ... };
card = :fem :inan { ... };
```

But the `:anim`/`:inan` distinction must originate from the **source language** definition, not just the translation. The `:match($target)` mechanism for Spanish personal "a" (Section 9.3) depends on reading `:anim`/`:inan` tags that travel with the `Phrase` value.

### Impact

If animacy tags are only defined in `ru.rlf` but not in the English source, then:
- `:match($target)` in `ru.rlf` won't find `:anim`/`:inan` tags when the `Phrase` was created from the English source definition
- Spanish `es.rlf` also needs these tags for personal "a"
- The runtime `Phrase` object carries tags from whichever language definition created it

### Recommendation

**Task 2 (Predicate Serializer → Phrase) must add `:anim`/`:inan` tags to ALL English source terms** that represent game entities:

```rust
// Required additions in strings.rs
card = :a :inan { one: "card", other: "cards" };
ally = :an :anim { one: "ally", other: "allies" };
agent = :an :anim { one: "Agent", other: "Agents" };
ancient = :an :anim { one: "Ancient", other: "Ancients" };
// ... all character subtypes need :anim
```

The plan's Section 9.6 already specifies this, but the current `strings.rs` code (Phase 1.5 output) has **not implemented it yet**. This should be an explicit step in Task 2, Step 1.

**Severity: MEDIUM** — Without this, Russian and Spanish translations would need workarounds or Rust code changes.

---

## Issue 2: Top-Level String Concatenation Prevents Russian Word Reordering (MEDIUM)

### Problem

Section 2.3 (Phrase Composition Strategy, Level 2) states the ability serializer assembles final text via string concatenation:

```rust
let trigger_text = strings::when_you_play_trigger(target_phrase).to_string();
let effect_text = strings::draw_cards_effect(count).to_string();
format!("{trigger_text}{effect_text}")
```

This locks in `[trigger][effect]` word order. Russian generally preserves SVO order for card game imperatives, so this is **usually acceptable**. However, there are specific patterns where Russian requires different structural ordering:

**Pattern 1: Condition placement.** English: "If X, do Y." Russian can be "Сделайте Y, если X." (Do Y, if X.) — the condition may come after the effect for stylistic reasons. The plan's static ability serializer (Task 6, Step 2) acknowledges condition placement flexibility, but the ability serializer's top-level concatenation in Task 7 does not.

**Pattern 2: "Once per turn" placement.** English: "Once per turn, [cost]: [effect]". Russian: "[cost]: [effect], один раз за ход" — the limitation may come after the effect.

**Pattern 3: Activated ability structure.** English: "[Cost]: [Effect]" (colon separator). Russian: "[Cost] — [Effect]" (em-dash separator convention in Russian card games).

### Impact

All three patterns currently use structural phrases (`once_per_turn_prefix`, `cost_effect_separator`) from `strings.rs`, which is good. However, the ability serializer (Task 7) uses **Rust-controlled** concatenation order between these structural pieces. Russian cannot reorder trigger vs. effect, or cost vs. effect, because the Rust code dictates the assembly order.

### Recommendation

The plan's current approach (Level 2 string concatenation at the top) is **pragmatically acceptable** because:
1. Most card games use consistent trigger→effect ordering across languages
2. The structural connectors (`: `, `", then "`) are already RLF phrases
3. Creating a single top-level RLF phrase for each ability structure would be over-engineered

**However**, the plan should explicitly note that the `cost_effect_separator` and `once_per_turn_prefix`/`once_per_turn_suffix` phrases exist as both prefix AND suffix variants. The current `strings.rs` already has both `once_per_turn_prefix` and `once_per_turn_suffix`, which is good. Ensure the Rust code in Task 7 uses the appropriate variant based on the locale's preference. This could be a simple RLF phrase like:

```rust
// English
once_per_turn_position = "prefix";  // or use a tag-based approach
// Russian
once_per_turn_position = "suffix";
```

Or more practically, wrap the entire activated ability structure in a single phrase that receives the cost and effect as `Phrase` parameters, allowing the translation to control ordering:

```rust
activated_ability($cost, $effect) = "{$cost}: {$effect}";
// Russian: activated_ability($cost, $effect) = "{$cost} — {$effect}";
```

**Severity: MEDIUM** — Addressable but requires awareness during Task 7. The current plan's approach is workable if structural phrases cover all needed variants.

---

## Issue 3: Gender Tags Not Propagated Through `:from` for Compound Phrases (LOW)

### Problem

The plan describes `:from($s)` on `subtype($s)` to inherit tags/variants from the source term. This works perfectly for single-term inheritance. But for **compound phrases** like `allied($entity)` or `enemy_modified($entity)`, the English source definitions are:

```rust
allied($entity) = "allied {$entity:one}";
enemy_modified($entity) = "enemy {$entity:one}";
```

These **do not use `:from($entity)`**, so they produce a plain `Phrase` with no tags. When a downstream phrase does `:match($target)` to check gender/animacy, and `$target` is the result of `allied(character)`, the gender tags from `character` are **lost**.

The Russian translation appendix solves this by defining `allied()` with explicit tag-based selection:
```
// ru.rlf
allied($entity) = "{allied_adj:$entity} {$entity:nom:one}";
```

This uses `{allied_adj:$entity}` which reads tags from `$entity` directly. But the **output Phrase** from `allied()` still has no tags itself — downstream consumers can't do `:match(allied_result)` for gender.

### Impact

If a consuming phrase needs to agree with the gender of an "allied Ancient" (e.g., "Dissolve an allied Ancient" → Russian needs participle agreement), it cannot read the gender from the compound phrase result.

### Recommendation

Add `:from($entity)` to compound phrases that wrap entity references:

```rust
// English
allied($entity) = :from($entity) "allied {$entity:one}";
enemy_modified($entity) = :from($entity) "enemy {$entity:one}";
```

This ensures the compound phrase inherits the entity's tags (including `:anim`/`:inan`, and in translations, `:masc`/`:fem`/`:neut`) and variants. The Russian appendix example already does this implicitly via direct selection, but **explicit `:from` is safer** for downstream composition.

**Severity: LOW** — Most consuming patterns select from the entity directly rather than the compound phrase, but `:from` propagation would be defensive best practice.

---

## Issue 4: Participle Agreement for "when X is dissolved/banished" (LOW)

### Problem

Russian past passive participles agree with the subject's gender:
- "когда персонаж растворён" (masc: растворён)
- "когда карта растворена" (fem: растворена)
- "когда событие растворено" (neut: растворено)

The plan's Section 9.2 shows this handled via `:match($target)`:
```
when_dissolved_trigger($target) = :match($target) {
    masc: "когда {$target:nom} растворён, ",
    fem: "когда {$target:nom} растворена, ",
    *neut: "когда {$target:nom} растворено, ",
};
```

This works **if and only if** `$target` carries gender tags. For English source definitions, gender tags are not applicable. The question is: does the `Phrase` carry gender tags when evaluated in Russian locale?

### Analysis

The flow is:
1. Rust calls `strings::ally()` → returns `Phrase` from whichever locale is active
2. If locale is Russian, `ally` is defined as `:masc :anim { nom.one: "союзник", ... }`
3. The `Phrase` carries `:masc` and `:anim` tags
4. Rust passes this `Phrase` to `strings::when_dissolved_trigger(target)`
5. Russian `:match($target)` finds `:masc` → selects correct participle

This **works correctly** because the `Phrase` is evaluated in the target locale, not the source locale. The tags come from the Russian definition.

### Remaining concern

The one edge case is if the Rust code ever creates a `Phrase` from a string literal or number (not from an RLF term lookup). In that case, there would be no gender tags. The plan correctly avoids this by requiring predicates to return `Phrase` values from named terms.

**Severity: LOW** — Works as designed. No action needed, but worth documenting as a "don't break this invariant" note.

---

## Issue 5: CLDR Plural Categories for Russian Need `few` Variant (LOW)

### Problem

Russian CLDR plural rules use four categories: `one`, `few`, `many`, `other`. The current English `strings.rs` only uses `one` and `other` variants (English only has these two). When RLF evaluates Russian `card:$n`, it will need `few` and `many` variants.

The question is: will the variant lookup for `{card:$n}` in a Russian translation file correctly resolve when the Russian definition has `nom.few` but the English source only defines `one`/`other`?

### Analysis

This is **not an issue** because:
1. Translation files define their own variant structure independently
2. Russian `ru.rlf` defines `card` with all needed case×number dimensions
3. The `Phrase` is evaluated from the Russian definition, not the English one
4. There's no cross-language variant inheritance

The English source's `one`/`other` variants are irrelevant when the Russian locale is active — the Russian definition fully replaces it.

**Severity: LOW** — No issue. Documented for completeness.

---

## Issue 6: Structural Connector Phrases and Russian Sentence Boundaries (INFO)

### Observation

The plan defines structural connectors as separate phrases:
```rust
then_joiner = ", then ";
and_joiner = " and ";
period_suffix = ".";
cost_effect_separator = ": ";
```

In Russian:
- `then_joiner` → `", затем "` or `", потом "`
- `and_joiner` → `" и "`
- `period_suffix` → `"."`
- `cost_effect_separator` → `" — "` (em-dash, Russian card game convention)

These translate cleanly. The fact that they're independent phrases means Russian can adjust punctuation and spacing without Rust changes. **This is well-designed.**

---

## Verification Checklist (Russian-Specific)

| Requirement | Plan Status | Notes |
|---|---|---|
| 6 grammatical cases (nom, acc, gen, dat, ins, prep) | ✅ Supported | Multi-dimensional variants with wildcard fallbacks |
| 3 genders (masc, fem, neut) | ✅ Supported | Tag-based `:match` selection |
| Complex plurals (one, few, many, other) | ✅ Supported | CLDR-compatible variant keys |
| Animacy (anim/inan) | ⚠️ Partially | Tags specified in plan but NOT yet in English source terms (Issue 1) |
| Participle agreement | ✅ Supported | Via `:match($target)` on gender tags |
| Word order flexibility | ⚠️ Limited at top level | Structural connectors are RLF phrases; ability-level order is Rust-controlled (Issue 2) |
| Adjective-noun agreement | ✅ Supported | Tag-based selection on adjective term (e.g., `allied_adj:$entity`) |
| Top-level concatenation | ⚠️ Acceptable | Issue 2 — pragmatically workable, not ideal |
| `:from` tag propagation | ⚠️ Partial | Compound phrases should add `:from` (Issue 3) |

---

## Recommendations Summary

| Priority | Issue | Action | Migration Task |
|---|---|---|---|
| **MEDIUM** | Issue 1: Missing animacy tags | Add `:anim`/`:inan` to all entity terms in `strings.rs` | Task 2, Step 1 |
| **MEDIUM** | Issue 2: Top-level word order | Wrap activated/triggered ability structures in RLF phrases that receive cost+effect as parameters, OR document the ordering constraint | Task 7 |
| **LOW** | Issue 3: `:from` on compound phrases | Add `:from($entity)` to `allied()`, `enemy_modified()`, etc. | Task 2, Step 1 |
| **LOW** | Issue 4: Participle agreement | No action needed — works as designed | — |
| **LOW** | Issue 5: CLDR plural categories | No action needed — translations define independently | — |
| **INFO** | Issue 6: Structural connectors | Well-designed, translates cleanly | — |

---

## Conclusion

The RLF framework and migration plan are **fundamentally sound for Russian localization**. The two MEDIUM-priority issues (animacy tags and top-level word order) are both addressable within the existing plan structure without architectural changes. Russian's complex morphology (6 cases × 4 plural forms × 3 genders = potentially 72 noun forms) is well-served by RLF's multi-dimensional variant system with wildcard fallbacks.

The critical architectural decision — **predicates returning `Phrase` with metadata tags rather than pre-rendered `String`** — is exactly what Russian needs. This single change (Task 2) unlocks all downstream Russian grammatical patterns through variant selection and tag-based matching.

---

## Appendix: RLF Framework Change Recommendations for Russian

Since RLF is a project-internal framework, we can modify it. Below are concrete recommendations from the Russian language perspective, addressing the team lead's specific questions.

### Recommendation 1: Paradigm Table Shorthand (HIGH VALUE)

**Problem:** Russian nouns require up to 6 cases × 4 plural forms = 24 variant entries per term. Writing these out is verbose and error-prone:

```
card = :fem :inan {
    nom.one: "карта", nom.few: "карты", nom.many: "карт",
    acc.one: "карту", acc.few: "карты", acc.many: "карт",
    gen.one: "карты", gen.few: "карт", gen.many: "карт",
    dat.one: "карте", dat.few: "картам", dat.many: "картам",
    ins.one: "картой", ins.few: "картами", ins.many: "картами",
    prep.one: "карте", prep.few: "картах", prep.many: "картах",
};
```

**Proposal:** Add a `@paradigm` directive that declares a structured declension grid. The syntax uses `|`-separated values in case-row order:

```
card = :fem :inan @paradigm(ru_noun) {
    //       one      | few      | many
    nom:    "карта"   | "карты"  | "карт",
    acc:    "карту"   | "карты"  | "карт",
    gen:    "карты"   | "карт"   | "карт",
    dat:    "карте"   | "картам" | "картам",
    ins:    "картой"  | "картами"| "картами",
    prep:   "карте"   | "картах" | "картах",
};
```

This expands internally to the same multi-dimensional variants. The `@paradigm(ru_noun)` directive specifies which column headers apply (`one`, `few`, `many` for Russian nouns; `nom`, `acc`, `dat`, `gen` for German nouns). Paradigm types are language-specific and defined in the RLF stdlib.

**Alternative (simpler):** Don't add new syntax. Instead, provide a linting/validation tool that checks `.rlf` files for completeness against a declared paradigm. The verbose syntax works, it's just tedious. A validator that says "card is missing `dat.few` variant" would catch 80% of the errors without new syntax.

**Recommendation: Start with the validator tool.** The verbose syntax is functional and established. A `@paradigm` shorthand could come later as a quality-of-life improvement if translators request it.

### Recommendation 2: No Russian-Specific Transforms Needed (NO CHANGE)

**Question:** Should there be Russian-specific transforms like `@v` for "в"/"во" preposition selection?

**Answer: No.** Russian doesn't need language-specific transforms. Unlike German (`@der`/`@ein` article inflection), Spanish (`@el`/`@un`), or Chinese (`@count` classifiers), Russian's complexity is entirely in **noun/adjective declension**, which is handled by variant selection (`:case:$n`).

The "в"/"во" alternation before consonant clusters (e.g., "во вторник" vs "в среду") is rare in card game text. If needed, it can be handled by `:match` on a tag:

```
// ru.rlf
in_zone($zone) = :match($zone) {
    cluster: "во {$zone:prep}",
    *other: "в {$zone:prep}",
};
```

The APPENDIX_STDLIB.md Summary Table already correctly lists Russian as needing no special transforms — only variant selection. This is a good design.

### Recommendation 3: `:from` Should Propagate ALL Tags (CONFIRM CURRENT BEHAVIOR)

**Question:** Should `:from` automatically propagate all tags, or should there be explicit tag forwarding?

**Answer:** `:from` should propagate ALL tags from the source parameter. This is what the APPENDIX_RUNTIME_INTERPRETER.md already specifies (Section "Metadata Inheritance Evaluation", step 2: "Clone the source phrase's tags for inheritance").

**Why this matters for Russian:** When `subtype($s) = :from($s) "<b>{$s}</b>"` inherits from `ancient` (tagged `:an` in English, `:masc :anim` in Russian), the result must carry ALL of those tags so that:
- `:match(subtype_result)` can branch on `:masc` for adjective agreement
- `:match(subtype_result)` can branch on `:anim` for accusative case selection
- `@a` can read `:an` for English article selection

If tags were selectively forwarded, translators would lose the ability to compose phrases freely. **No change needed — confirm the current "propagate all" behavior is correct and document it as a guarantee.**

### Recommendation 4: Compound Phrases Should NOT Get Implicit `:from` (NO CHANGE)

**Question:** Should compound phrases automatically inherit tags from their parameters?

**Answer: No.** Implicit `:from` would be surprising and create ambiguity for multi-parameter phrases. Consider:

```
with_cost_less_than_allied($base, $counting) = "...";
```

Which parameter should provide the implicit tags? `$base` or `$counting`? There's no correct default.

**Instead:** Require explicit `:from($param)` where tag inheritance is needed. This is already the design. The issue I flagged (Issue 3) is that some compound phrases like `allied($entity)` **should** have `:from($entity)` but currently don't. This is a content issue (add `:from` to specific phrases), not a framework issue.

### Recommendation 5: `:match` Already Supports Compound Conditions (CONFIRM)

**Question:** Should `:match` support compound conditions (e.g., matching on both gender AND animacy)?

**Analysis:** `:match` already supports multi-parameter matching via dot notation:

```
n_allied($n, $entity) = :match($n, $entity) {
    1.masc: "союзный {$entity:nom:one}",
    1.fem: "союзная {$entity:nom:one}",
    *other.*neut: "{$n} союзных {$entity:gen:many}",
};
```

For matching on **two tags from the same parameter** (e.g., both `:masc` and `:anim` on one entity), the current mechanism uses the first matching tag. This means if a Phrase is tagged `:masc :anim`, `:match($entity)` with branches `masc:` and `anim:` will match `masc` first (tag order matters).

**For Russian, this is sufficient.** The cases where you need to branch on both gender AND animacy simultaneously are:
- Masculine animate: accusative = genitive
- Masculine inanimate: accusative = nominative
- Feminine/neuter: accusative follows its own pattern regardless of animacy

This can be expressed by nesting or by using specific combined tags:

**Option A: Nested match (current RLF)**
```
// Russian accusative selection
acc_form($entity) = :match($entity) {
    masc: "{acc_masc($entity)}",
    fem: "{$entity:acc}",
    *neut: "{$entity:acc}",
};
acc_masc($entity) = :match($entity) {
    anim: "{$entity:gen}",   // animate masc: acc = gen
    *inan: "{$entity:nom}",  // inanimate masc: acc = nom
};
```

**Option B: Combined tag on source terms**
Add combined tags like `:masc_anim`, `:masc_inan` to source terms (as Polish does in APPENDIX_STDLIB.md). Then `:match` can branch directly:
```
acc_form($entity) = :match($entity) {
    masc_anim: "{$entity:gen}",
    masc_inan: "{$entity:nom}",
    fem: "{$entity:acc}",
    *neut: "{$entity:acc}",
};
```

**Recommendation:** Option B is cleaner. Add compound animacy-gender tags to the English source terms alongside separate gender and animacy tags:

```rust
// English source (strings.rs)
agent = :an :anim :masc_anim { one: "Agent", other: "Agents" };
card = :a :inan :fem_inan { one: "card", other: "cards" };
```

The `:masc_anim` tag is redundant with `:masc` + `:anim` but provides a single matchable key. This avoids any need to change the `:match` mechanism.

**However:** If RLF were to add a "match on multiple tags" syntax, it would be cleaner:

```
// Hypothetical: match on tag intersection
acc_form($entity) = :match($entity) {
    masc & anim: "{$entity:gen}",
    masc & inan: "{$entity:nom}",
    *other: "{$entity:acc}",
};
```

**Verdict: No framework change needed now.** Option B (compound tags) works with current RLF. A `&` operator in `:match` branches would be a nice quality-of-life improvement but is not blocking.

### Recommendation 6: CLDR Plurals Are Already First-Class (NO CHANGE)

**Question:** Should the CLDR plural system be a first-class feature?

**Answer:** It already is. The RLF evaluator uses ICU4X `icu_plurals` to map numbers to CLDR categories (`one`, `few`, `many`, `other`). Parameterized selection `{card:$n}` automatically uses CLDR mapping. `:match($n)` tries exact number first, then CLDR category.

Russian's 4-category system (one/few/many/other) works correctly because ICU4X implements the CLDR rules for Russian:
- `one`: 1, 21, 31, 41, ... (but not 11)
- `few`: 2-4, 22-24, 32-34, ... (but not 12-14)
- `many`: 0, 5-20, 25-30, 35-40, ... (includes 11-14)
- `other`: fractional numbers

No change needed.

### Recommendation 7: Case Selection Composition Already Works (CONFIRM)

**Question:** Should `{$x:acc}` compose with gender/number?

**Answer:** It already does via chained selectors. `{$x:acc:$n}` first selects `acc` (static case selector), then `:$n` (parameterized number selector via CLDR). The wildcard fallback system resolves this correctly:

```
card = :fem :inan {
    acc.one: "карту",
    acc.few: "карты",
    acc.many: "карт",
    acc: "карту",      // fallback for acc.* (defaults to singular)
};

// {card:acc:$n} where n=3 →
//   try "acc.few" → hit → "карты" ✓
// {card:acc:$n} where n=1 →
//   try "acc.one" → hit → "карту" ✓
// {card:acc} →
//   try "acc" → hit → "карту" (fallback) ✓
```

This works perfectly. No change needed.

---

### Summary of RLF Framework Recommendations

| # | Recommendation | Change Type | Priority |
|---|---|---|---|
| 1 | Paradigm table validator tool | New tooling (not syntax) | MEDIUM — quality-of-life for translators |
| 2 | No Russian-specific transforms | No change | N/A — confirmed correct |
| 3 | `:from` propagates all tags | Confirm + document | LOW — already works, needs explicit guarantee |
| 4 | No implicit `:from` on compound phrases | No change | N/A — explicit is better |
| 5 | Compound animacy-gender tags in source terms | Content change, not framework | MEDIUM — pragmatic workaround |
| 5b | `&` operator in `:match` branches | Future framework feature | LOW — nice-to-have, not blocking |
| 6 | CLDR plurals already first-class | No change | N/A — confirmed correct |
| 7 | Case + number selection composition | No change | N/A — already works via chaining |

**Bottom line:** RLF's current feature set is sufficient for Russian. The framework needs **zero mandatory changes**. The only concrete recommendation is a translation validation tool (Recommendation 1) and compound tags on source terms (Recommendation 5). Everything else works as-designed.
