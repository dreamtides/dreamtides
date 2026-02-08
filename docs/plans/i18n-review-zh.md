# Simplified Chinese (简体中文) i18n Review of Serializer RLF Migration Plan

**Reviewer:** zh-agent (Simplified Chinese localization specialist)
**Date:** 2026-02-08
**Status:** PASS with recommendations

---

## Executive Summary

The migration plan is **well-designed for Chinese support**. The RLF framework's `@count` transform with classifier tags (`:zhang`, `:ge`, etc.), `:match` for exact numeric branching, and per-phrase word order control address the core needs of Simplified Chinese. No Rust code changes are required to add Chinese translations, which is the stated goal.

There are **3 medium-priority issues** and **4 minor recommendations** that would make the Chinese translation experience smoother. None are blockers.

---

## Analysis by Concern Area

### 1. Classifiers / Measure Words (量词)

**Verdict: SUPPORTED, with one gap to monitor**

Chinese requires measure words between numbers and nouns: 一**张**牌 (one-*zhāng*-card), 一**个**角色 (one-*gè*-character). The RLF stdlib defines the `@count` transform reading tags like `:zhang`, `:ge`, `:ming`, etc. This is exactly what's needed.

**How it works in practice:**
```
// zh_cn.rlf
pai = :zhang "牌";
cards($n) = :match($n) {
    1: "{@count($n) pai}",
    *other: "{@count($n) pai}",
};
// n=1 → "1张牌", n=3 → "3张牌"
```

**Gap — `@count` with quantity 1:** In Chinese, "draw a card" should be "抽一张牌" (with 一, the word "one"), not "抽1张牌" (with the digit 1). The `@count` transform inserts the numeric parameter directly. For n=1, the Chinese translation file can use `:match` to branch on exact 1 and use the word "一" instead:

```
cards($n) = :match($n) {
    1: "一张牌",
    *other: "{$n}张牌",
};
```

This works. The plan's section 9.4 shows exactly this pattern. **No issue.**

**Gap — classifier tags on predicate Phrase values:** The plan says predicates return `Phrase` objects. Chinese needs classifier tags on these Phrases. Currently, English source terms use `:a`/`:an` tags. The Chinese translation file defines *its own* terms with classifier tags. Since `:from` propagates tags from the parameter, the Chinese `subtype($s)` would inherit `:zhang`/`:ge` from the Chinese subtype definition.

**RECOMMENDATION M1:** Verify that when a translation file redefines a term (e.g., `card` in zh_cn.rlf has `:zhang` instead of `:a`), the `:from` inheritance propagates the *translation's* tags, not the source language's tags. This should work by design (the interpreter evaluates in the target language), but it's worth a test case.

---

### 2. No Plural / No Gender

**Verdict: FULLY SUPPORTED**

Chinese has no plural inflection and no grammatical gender. Variant blocks with `one`/`other` can simply be ignored — Chinese defines a single `other` variant (or no variants at all):

```
// zh_cn.rlf
card = :zhang "牌";                    // No variants needed
ally = :ge { other: "友方角色" };       // Single variant is fine
```

`:match` on `:masc`/`:fem` tags? Chinese just uses `*` default:
```
when_dissolved_trigger($target) = "当{$target}被消解时，";
// No gender branching needed — works fine with *default
```

The RLF requirement that `:match` blocks have a `*` default ensures Chinese can always use the fallback. **No issues.**

---

### 3. Word Order (语序)

**Verdict: SUPPORTED, with one structural concern**

Chinese often places modifiers before the modified word, and uses Topic-Comment structure. Key patterns:

| English | Chinese | Structure |
|---------|---------|-----------|
| "dissolve an enemy" | "消解一个敌方角色" | keyword + classifier + target |
| "draw 3 cards" | "抽3张牌" | verb + count+classifier+noun |
| "from your void" | "从你的虚空中" | prepositional phrase before verb |
| "banish from void" | "从虚空放逐" | PP-before-verb reordering |

Each phrase in the translation file controls its own word order. The phrase `dissolve_target($target)` in English is "Dissolve {@a $target}." while in Chinese it becomes "消解{@count(1) $target}。" — the RLF template handles reordering within each phrase. **This works.**

**CONCERN M2 — Top-level string concatenation for ability assembly (Section 2.3, Level 2):**

The plan uses string concatenation at the ability serializer level:
```rust
format!("{trigger_text}{effect_text}")
```

For Chinese, many triggered abilities need *different structural ordering*. English puts the trigger first: "When you play a card, draw a card." Chinese can follow this order ("当你打出一张牌时，抽一张牌。") and it reads naturally. However, some Chinese card game translations put the effect first for emphasis, or insert different connectors.

The plan acknowledges this (Section 2.3): "for languages that need to reorder trigger vs effect, the translation can restructure at the phrase level below." This means each trigger phrase must include its own trailing comma/separator and each effect phrase is self-contained. **This is acceptable** because:

1. Chinese card game text typically follows trigger-then-effect order (same as English)
2. The structural connectors (`then_joiner`, `and_joiner`, `period_suffix`) are all separate RLF phrases that Chinese can redefine
3. Truly different structural patterns (e.g., activated abilities with "cost: effect") use dedicated structural phrases

However, the **cost-separator ordering** in activated abilities deserves attention:

```rust
// Current Rust code in ability_serializer.rs (line 88-93):
result.push_str(&costs);
result.push_str(", once per turn");
result.push_str(": ");
result.push_str(&effect);
```

Chinese activated abilities should read: "费用：效果" (cost：effect) — same ordering, but the colon is full-width. The `cost_effect_separator` phrase handles this (Chinese would define it as "："). The `once_per_turn_suffix` would be "，每回合一次". **This works as-is.**

**RECOMMENDATION M2:** Ensure all structural connectors used in `ability_serializer.rs` go through named RLF phrases. Currently (line 36-37, 39-40), `"Until end of turn, "` and `"Once per turn, "` are hardcoded strings. The plan's Task 7 addresses this, but verify that the hardcoded strings at lines 36-40 are captured. Currently `until_end_of_turn_prefix` and `once_per_turn_prefix` exist in strings.rs, so this should be fine after Task 7.

---

### 4. Sentence-Final Punctuation

**Verdict: FULLY SUPPORTED**

Chinese uses full-width punctuation: 。(period), ，(comma), ：(colon), ！(exclamation). These all live in RLF phrase definitions:

```
// zh_cn.rlf
draw_cards_effect($c) = "抽{cards($c)}。";      // Full-width period
then_joiner = "，然后";                           // Full-width comma
cost_effect_separator = "：";                     // Full-width colon
period_suffix = "。";                             // Full-width period
```

The `period_suffix` phrase is used for terminal punctuation. Each effect phrase already embeds its own period. **No issues.**

---

### 5. Number Words (数字词)

**Verdict: SUPPORTED via `:match`**

Chinese uses number words in certain contexts: 一张牌 (one-classifier-card) not "1张牌" for quantity 1. The `:match` with exact numeric key handles this perfectly:

```
cards($n) = :match($n) {
    1: "一张牌",
    *other: "{$n}张牌",
};
```

The English `text_number` phrase (lines 214-221 of strings.rs) converts 1→"one", 2→"two", etc. Chinese can define its own `text_number`:

```
text_number($n) = :match($n) {
    1: "一",
    2: "两",       // NOTE: 两 not 二 for counting
    3: "三",
    4: "四",
    5: "五",
    *other: "{$n}",
};
```

**NOTE:** Chinese uses 两 (liǎng) for "two" when counting objects, not 二 (èr). This is a translation concern, not a Rust code concern. The architecture supports it. **No issues.**

---

### 6. String Concatenation at Top Level

**Verdict: ACCEPTABLE, not ideal but workable**

Section 2.3 explains why the ability serializer uses string concatenation instead of a single top-level RLF phrase. The concerns for Chinese:

- **Trigger + Effect ordering**: Chinese card games use the same trigger-before-effect ordering as English. ✓
- **Cost + Effect separator**: The "：" (full-width colon) is an RLF phrase. ✓
- **Keyword trigger + capitalized effect**: Chinese doesn't capitalize, but `@cap` on Chinese text is a no-op (Chinese characters have no case). ✓
- **Multiple effects with "then"/"and"**: Chinese uses "，然后" and "和", which are RLF phrases. ✓

**CONCERN M3 — `capitalize_first_letter` interactions:** The plan removes `capitalize_first_letter` (Task 9) and replaces it with `@cap` in RLF phrases. For Chinese, `@cap` is harmless (no-op on CJK characters). However, if any Rust code currently applies capitalization logic *before* passing text to RLF, it could mangle CJK text. Looking at `ability_serializer.rs`, `capitalize_first_letter` is called on serialized output (lines 43, 48, 64, 81, 93, 103, 145). After migration, these become `@cap` in phrase templates. Since `@cap` skips non-alphabetic characters, **this is safe for Chinese.** No issue after migration.

---

### 7. Keyword Formatting

**Verdict: FULLY SUPPORTED**

Keywords like "Dissolve" → "消解", "Banish" → "放逐" are defined as RLF terms with color formatting:

```
// zh_cn.rlf
dissolve = "<color=#AA00FF>消解</color>";
banish = "<color=#AA00FF>放逐</color>";
materialize = "<color=#AA00FF>实体化</color>";
prevent = "<color=#AA00FF>防止</color>";
reclaim = "<color=#AA00FF>回收</color>";
kindle($k) = "<color=#AA00FF>点燃</color> {$k}";
foresee($n) = "<color=#AA00FF>预见</color> {$n}";
```

The `{@cap dissolve}` pattern produces "消解" in Chinese (no-op cap) with the color markup. The color wrapping is in the RLF definition, so the Rust code doesn't need to know about it. **No issues.**

---

### 8. Predicate Phrases

**Verdict: SUPPORTED, with recommendation**

The plan has predicates return `Phrase` objects. Chinese predicates need classifier tags instead of article tags:

```
// zh_cn.rlf
card = :zhang "牌";
character = :ge "角色";
event = :ge "事件";
ally = :ge { other: "友方角色" };
enemy_character = :ge "敌方角色";
```

Chinese doesn't need `:a`/`:an` tags — it uses classifier tags instead. The `@count` transform reads these. The consuming phrase in Chinese:

```
dissolve_target_effect($target) = "消解{@count(1) $target}。";
// With enemy_character → "消解1个敌方角色。"
// Or with :match for n=1:
dissolve_target_effect($target) = "消解一{$target}。";
// Wait — this doesn't include the classifier...
```

**CONCERN — Classifier insertion in consuming phrases:** The English pattern `{@a $target}` prepends "a"/"an". The Chinese equivalent needs `{@count(1) $target}` to prepend "一张"/"一个". But `@count` requires the *count* as a dynamic parameter. For phrases where the count is always 1 (like "dissolve an enemy"), this works:

```
dissolve_target_effect($target) = "消解{@count(1) $target}。";
```

But `@count` reads the classifier tag from `$target`. If `$target` is a `Phrase` with `:ge` tag, then `@count(1)` produces "1个" and the result is "消解1个敌方角色。". For n=1, we'd want "一个" not "1个". This can be handled with `:match` in a wrapper:

```
one_of($target) = :match($target) {
    ge: "一个{$target}",
    zhang: "一张{$target}",
    *ge: "一个{$target}",
};
dissolve_target_effect($target) = "消解{one_of($target)}。";
```

Or, if `@count` supports number word substitution for n=1 in the Chinese locale (i.e., `@count(1)` produces "一张" not "1张"), then it's simpler. **RECOMMENDATION M4:** Verify or document whether `@count` with n=1 produces "一张" or "1张" in the Chinese locale. If it produces "1张", add a note that Chinese translators may need a `:match`-based wrapper for single-quantity expressions.

---

## Issue Summary

### Medium Priority

| ID | Issue | Section | Impact | Recommendation |
|----|-------|---------|--------|----------------|
| M1 | `:from` tag propagation with translation-file tags | 1 | Could break classifier inheritance | Add test: Chinese term with `:zhang`, passed through `:from`, verify tag propagated |
| M2 | Hardcoded structural strings in ability_serializer.rs | 3 | Would prevent Chinese punctuation | Task 7 addresses this — verify all hardcoded strings are captured |
| M3 | `@count(1)` output for Chinese (一 vs 1) | 8 | Affects naturalness of Chinese text | Document `@count` behavior for n=1; provide `:match` pattern if needed |

### Minor / Informational

| ID | Note | Section |
|----|------|---------|
| I1 | Chinese uses 两 not 二 for counting — translation concern only | 5 |
| I2 | Full-width punctuation (。，：) works naturally via RLF phrase redefinition | 4 |
| I3 | `@cap` is a no-op on CJK — safe, no special handling needed | 6 |
| I4 | Chinese card game text follows trigger-before-effect order — no reordering needed at ability level | 3 |

---

## Detailed Test Cases for Chinese

### "Draw 3 cards." → "抽3张牌。"

```
// zh_cn.rlf
pai = :zhang "牌";
cards($n) = :match($n) {
    1: "一张牌",
    *other: "{$n}张牌",
};
draw_cards_effect($c) = "抽{cards($c)}。";
// draw_cards_effect(1) → "抽一张牌。" ✓
// draw_cards_effect(3) → "抽3张牌。" ✓
```

### "Dissolve an enemy Ancient." → "消解一个敌方远古。"

```
// zh_cn.rlf
ancient = :ge "远古";
subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";
enemy_subtype($t) = :from($t) "敌方{$t}";

dissolve_target_effect($target) = "消解一个{$target}。";
// With enemy Ancient phrase → "消解一个敌方<b>远古</b>。" ✓
```

Note: The classifier "个" is hardcoded here as "一个" because dissolve always targets one entity. For variable counts, use `@count`.

### "When you play a card, draw a card." → "当你打出一张牌时，抽一张牌。"

```
// zh_cn.rlf
when_you_play_trigger($target) = "当你打出{$target}时，";
// Concatenated with draw_cards_effect(1):
// "当你打出一张牌时，" + "抽一张牌。"
// → "当你打出一张牌时，抽一张牌。" ✓
```

### Activated ability: "Discard a card: Draw 2 cards."

```
// zh_cn.rlf
discard_cards_cost($d) = "弃{cards($d)}";
cost_effect_separator = "：";
draw_cards_effect($c) = "抽{cards($c)}。";
// "弃一张牌" + "：" + "抽2张牌。"
// → "弃一张牌：抽2张牌。" ✓
```

---

## Conclusion

The migration plan is **sound for Simplified Chinese**. The key enablers are:

1. **`@count` transform with classifier tags** — handles measure words
2. **`:match` with exact numeric keys** — handles 一/两/三 number words
3. **Per-phrase word order control** — Chinese templates define their own structure
4. **`:from` tag propagation** — Chinese classifier tags flow through composition
5. **`@cap` no-op on CJK** — capitalization logic doesn't break Chinese text
6. **Structural phrases as RLF terms** — punctuation and connectors are localizable

The Rust code after Phase 2 will be fully language-neutral for Chinese. No Rust code changes will be needed to add Chinese support — only Chinese `.rlf` translation files.

---

## RLF Framework Change Recommendations for Chinese

*Added 2026-02-08 in response to team lead follow-up. Since RLF is an internal framework, we can modify it to improve CJK support.*

### Updated Status of Original Concerns

After investigating the RLF implementation in detail:

- **M1 (`:from` tag propagation) — RESOLVED, no change needed.** The interpreter maintains per-language `PhraseRegistry` instances. When evaluating in Chinese, `:from($s)` reads tags from the Chinese definition of the term, not the English source. This is confirmed in `evaluator.rs` lines 212-320: tags are cloned directly from the parameter's `Phrase` object, which was constructed in the current language's registry. The original concern was unfounded.

- **M3 (`@count(1)` output) — CONFIRMED as a real gap.** The `@count` transform in `transforms.rs` lines 1501-1507 uses `context_to_count()` which returns the raw `i64` value. For n=1, `@count(1)` produces "1张牌", not "一张牌". This is the primary gap for natural Chinese text.

### Recommendation 1: Add `@count` number word mode for CJK locales

**Problem:** `@count(1)` in Chinese produces "1张牌" but natural Chinese uses "一张牌" (number word + classifier + noun). Translators currently must bypass `@count` entirely and use `:match` wrappers for every phrase that can have n=1, which is verbose and error-prone.

**Proposed change:** Add a `:word` context modifier to `@count` that substitutes CJK number words for small numbers (1-10).

```
// Current behavior (unchanged):
draw($n) = "抽{@count($n) pai}";           // n=1 → "抽1张牌"

// New: :word modifier uses CJK number words
draw($n) = "抽{@count:word($n) pai}";      // n=1 → "抽一张牌", n=3 → "抽三张牌"
                                             // n=15 → "抽15张牌" (falls back to digits)
```

**Implementation:** In the Chinese `@count` transform, when the `:word` context is present, map small integers to Chinese number words before concatenation:

| n | Word |
|---|------|
| 1 | 一 |
| 2 | 两 (liǎng, the counting form) |
| 3 | 三 |
| 4 | 四 |
| 5 | 五 |
| 6 | 六 |
| 7 | 七 |
| 8 | 八 |
| 9 | 九 |
| 10 | 十 |
| >10 | digit form |

This mapping uses 两 (not 二) for 2 — the correct counting form in Chinese. Japanese and Korean `@count` would get their own `:word` mappings (Japanese uses 一/二/三, Korean uses native numerals 하나/둘/셋 for small counts with certain counters).

**Why not just use `:match`?** Translators would need to write this pattern for *every* phrase that takes a count parameter:

```
// Without @count:word — every phrase needs this boilerplate:
cards($n) = :match($n) { 1: "一张牌", 2: "两张牌", *other: "{$n}张牌" };
draw_cards_effect($c) = "抽{cards($c)}。";
discard_cards_effect($d) = "弃{cards($d)}。";
// ... duplicated for every count-taking phrase
```

With `@count:word`, the base `cards` phrase handles it once:
```
cards($n) = "{@count:word($n) pai}";
draw_cards_effect($c) = "抽{cards($c)}。";
discard_cards_effect($d) = "弃{cards($d)}。";
```

**Scope:** Modify `chinese_count_transform` in `transforms.rs` to accept a `:word` context. Add equivalent for Japanese (`一枚`/`二枚`/`三枚`) and Korean (depends on counter: native vs Sino-Korean). ~30 lines per language.

**Priority: MEDIUM.** Without this, Chinese translations work but require verbose `:match` boilerplate. Not a blocker.

---

### Recommendation 2: Do NOT make classifiers a first-class concept

**Question from team lead:** Should classifiers be a first-class concept in RLF (like gender is)?

**Answer: No.** Classifiers are already well-served by the tag + `@count` transform pattern. Making them first-class would mean:

- A dedicated `classifier: zhang` syntax (like `:masc` for gender)
- Automatic classifier insertion in certain contexts
- Special variant selection rules based on classifiers

This is unnecessary because:

1. **Classifiers don't drive agreement.** Unlike gender (which affects adjectives, articles, participles across an entire sentence), classifiers only appear in one place: between the number and the noun. There's no "classifier agreement" on other words.

2. **The tag system already handles it.** `:zhang` is a tag. `@count` reads it. `:from` propagates it. This is exactly how `:masc`/`:fem` work for gendered languages — gender isn't "first-class" either, it's just a tag that transforms read.

3. **Classifier selection is noun-intrinsic.** Each noun has exactly one classifier (牌 is always 张, 角色 is always 个). This maps perfectly to a tag on the term definition. No dynamic selection is needed.

**Keep classifiers as tags + `@count`.** The current design is correct.

---

### Recommendation 3: No changes needed for `:from` tag propagation

**Question from team lead:** Should `:from` work differently for classifier-based languages?

**Answer: No.** The current implementation is already correct for Chinese. Confirmed by reading `evaluator.rs`:

1. When evaluating in Chinese, `:from($s)` reads `$s` from the Chinese registry
2. If Chinese defines `ancient = :ge "远古"`, then `subtype(ancient)` inherits `:ge`
3. A consuming phrase can then use `{@count(1) subtype($s)}` to produce "1个远古"

The tag propagation is language-scoped by design — each language's registry is independent. No change needed.

---

### Recommendation 4: No changes needed for variant selection in no-plural languages

**Question from team lead:** Should variant selection work differently for languages without plural/gender?

**Answer: No.** Chinese CLDR always returns "other" for any number. This means:

- `{card:$n}` always resolves to the "other" variant regardless of n
- Chinese terms can be defined as plain strings (`card = :zhang "牌"`) — no variants needed
- If a term has no variants, `{card:$n}` returns the term's text directly
- `:match` with exact numeric keys (`1: "一张牌"`) works independently of CLDR

The current system is clean for Chinese. Defining unnecessary `one`/`other` variants would be harmless but wasteful. Chinese translators simply don't use variant blocks for pluralization.

---

### Recommendation 5: Add `@yi` convenience transform (OPTIONAL, LOW PRIORITY)

**Problem:** The most common Chinese pattern for single-target phrases is "一 + classifier + noun" (e.g., "一张牌", "一个角色"). Currently this requires either:

- `{@count:word(1) $target}` (with Recommendation 1) — verbose for a constant
- Hardcoding "一个" in the template — loses the classifier tag

**Proposed:** A `@yi` ("one") transform for Chinese that produces "一 + classifier":

```
// zh_cn.rlf
dissolve_target($target) = "消解{@yi $target}。";
// With :ge tagged target → "消解一个敌方角色。"
// With :zhang tagged target → "消解一张牌。"
```

`@yi` reads the classifier tag from the phrase and prepends "一" + the classifier character. It's syntactic sugar for `@count:word(1)` but much more readable for the extremely common "one of X" pattern.

**Implementation:** ~15 lines in the Chinese transform module. Reads the same classifier tags as `@count`.

**Priority: LOW.** This is a convenience — `@count:word(1)` or `:match` wrappers work fine. Only implement if Chinese translators find the boilerplate burdensome in practice.

---

### Recommendation 6: Add stdlib `@count` documentation note about `:match` fallback pattern

**Problem:** The `@count` transform always outputs digits. Translators who want number words for small counts (especially n=1 and n=2 in Chinese) may not immediately know the workaround.

**Proposed:** Add a documentation section to APPENDIX_STDLIB.md under the Chinese section showing the canonical `:match` fallback pattern:

```
// Recommended pattern for natural Chinese number words:
cards($n) = :match($n) {
    1: "一张牌",
    2: "两张牌",
    *other: "{@count($n) pai}",
};

// Or with @count:word (if Recommendation 1 is implemented):
cards($n) = "{@count:word($n) pai}";
```

This costs nothing and prevents translator confusion. **Priority: HIGH** (documentation-only change).

---

### Summary of RLF Framework Recommendations

| # | Change | Type | Priority | Effort |
|---|--------|------|----------|--------|
| 1 | `@count:word` modifier for CJK number words | Feature | MEDIUM | ~30 lines/language |
| 2 | Do NOT make classifiers first-class | Decision | N/A | 0 |
| 3 | No changes to `:from` propagation | Decision | N/A | 0 |
| 4 | No changes to variant selection | Decision | N/A | 0 |
| 5 | `@yi` convenience transform | Feature | LOW | ~15 lines |
| 6 | Document `:match` fallback pattern | Docs | HIGH | ~10 lines of docs |

**Net recommendation:** Implement #1 (`@count:word`) and #6 (documentation). Skip #5 unless translators request it. The other items confirm the current design is correct — no changes needed.
