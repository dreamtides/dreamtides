# Korean (한국어) i18n Review of Serializer RLF Migration Plan

**Reviewer:** ko-agent (Korean localization specialist)
**Date:** 2026-02-08
**Status:** PASS with critical recommendation for `@particle` phonology implementation

---

## Executive Summary

The migration plan is **well-designed for Korean support**. Korean shares structural similarities with Japanese and Chinese — no gender, no articles, counters/classifiers — but has one unique and **critical challenge: phonology-dependent postpositional particles**. The choice between particle forms (은/는, 이/가, 을/를) depends on whether the preceding syllable ends in a consonant (받침, batchim) or a vowel. This is a runtime phonological decision that cannot be resolved via tags because the final sound depends on the dynamically rendered text.

The RLF stdlib already defines `@particle` for Korean with `:subj`, `:obj`, and `:topic` contexts (APPENDIX_STDLIB.md lines 957-981). This is the correct approach. However, several design details need verification and a few additional particle contexts are required for full Dreamtides card text coverage.

**Critical issues found: 1** (particle after HTML markup)
**Significant issues found: 2** (native vs Sino-Korean number systems; additional particle contexts)
**Minor concerns: 3**

---

## Analysis by Concern Area

### 1. Postpositional Particles (조사) — The Core Challenge

**Verdict: SUPPORTED via `@particle`, with gaps to address**

Korean particles attach directly after nouns and vary based on the final sound of the preceding word:

| Particle function | After vowel-final | After consonant-final | Example (vowel) | Example (consonant) |
|---|---|---|---|---|
| Subject (이/가) | 가 | 이 | 사과가 (apple-SUBJ) | 책이 (book-SUBJ) |
| Object (을/를) | 를 | 을 | 사과를 (apple-OBJ) | 책을 (book-OBJ) |
| Topic (은/는) | 는 | 은 | 사과는 (apple-TOP) | 책은 (book-TOP) |
| And/with (와/과) | 와 | 과 | 사과와 (apple-AND) | 책과 (book-AND) |
| Copula (이다) | 다 | 이다 | 사과다 (is apple) | 책이다 (is book) |
| Direction (으로/로) | 로 | 으로 | 사과로 (toward apple) | 책으로 (toward book) |

The `@particle` transform in the RLF stdlib correctly uses runtime phonological inspection of the final Unicode grapheme cluster, not tags. This is essential because the same noun may have different rendered forms depending on context (e.g., a subtype name vs a colored keyword).

**Gap 1 — Only 3 contexts defined:** The stdlib defines `:subj` (이/가), `:obj` (을/를), and `:topic` (은/는). Dreamtides card text will also need:

- `:and` — 와/과 (conjunctive, "X and Y": "카드와 캐릭터")
- `:copula` — 이다/다 ("X is Y": "캐릭터이다" vs "사과다")
- `:dir` — (으)로 (directional/instrumental, "to your hand": "손으로")

**RECOMMENDATION KO-1 (SIGNIFICANT):** Add `:and`, `:copula`, and `:dir` particle contexts to the Korean `@particle` transform. ~15 lines following the existing pattern. These are needed for:
- "X and Y" patterns (`and_joiner`): "카드**와** 에너지" (card and energy)
- "becomes X" patterns: "2**가** 된다" (becomes 2)
- "to hand/to void" patterns: "손**으로** 돌아간다" (returns to hand)

**Gap 2 — Particle after HTML markup tags:** If a word is wrapped in formatting like `<color=#AA00FF>소환</color>` (materialize), the particle must attach after the closing tag: `<color=#AA00FF>소환</color>을`. The `@particle` transform inspects the **final grapheme cluster** of the rendered text — but does it skip HTML tags to find the last *visible* character?

Consider: `{@particle:obj materialize}` where `materialize` renders as `<color=#AA00FF>소환</color>`. The last visible character is 환 (ends in ㄴ, consonant), so the particle should be 을. But the raw string ends with `>`, not 환.

**ISSUE KO-CRIT-1 (CRITICAL):** The `@particle` transform MUST skip trailing HTML-like markup tags to inspect the last visible character's phonological shape. This mirrors the existing `@cap` behavior (which skips *leading* markup to find the first visible character). Without this, every Korean particle attached to a formatted keyword or colored text will be incorrect.

This is likely already needed for Japanese too (the `@particle` transform is shared), though Japanese particles are phonology-independent so the impact there is just incorrect spacing.

---

### 2. Counters (수사/수량사) — Native vs Sino-Korean Numbers

**Verdict: SUPPORTED for basic cases, but number system selection is complex**

Korean has TWO number systems, and which system is used depends on the counter:

| Counter | System | Example (n=3) | Usage |
|---|---|---|---|
| 장 (flat things, cards) | Native Korean | 세 장 | 카드 세 장 |
| 개 (general items) | Native Korean | 세 개 | 세 개 |
| 명 (people) | Native Korean | 세 명 | 세 명 |
| 마리 (animals) | Native Korean | 세 마리 | 세 마리 |
| 턴 (turns) | Sino-Korean | 3턴 | 3턴 동안 |
| 점 (points) | Sino-Korean | 3점 | 3점 |
| 원 (currency/energy) | Sino-Korean | 3원 | 3원 |

Native Korean numbers 1-5: 하나→한, 둘→두, 셋→세, 넷→네, 다섯. After 5, native numbers become cumbersome and digits are common. The counting form (한/두/세/네) differs from the standalone form (하나/둘/셋/넷).

**Comparison with Chinese:** The Chinese review (Section 1) notes that `@count` inserts `$n` directly as a digit, and recommends `@count:word` for CJK number words (Section 9.8.4 of the migration plan). Korean needs this too, but with an important difference: Korean needs to select between native and Sino-Korean number words based on the counter type.

**How it could work with `@count:word`:**

```
// ko.rlf
card = :jang "카드";
point = "점";

// @count:word with :jang tag → native Korean numbers
draw($n) = "{@count:word($n) card}를 뽑는다";
// n=1 → "카드 한 장을 뽑는다" (native: 한)
// n=3 → "카드 세 장을 뽑는다" (native: 세)

// Sino-Korean counters just use digits
gain_points($p) = "{$p}점을 얻는다";
// p=3 → "3점을 얻는다"
```

**RECOMMENDATION KO-2 (SIGNIFICANT):** The `@count:word` modifier proposed in Section 9.8.4 should support **Korean native counting numbers** (한/두/세/네/다섯/여섯/일곱/여덟/아홉/열) for counters tagged with native-system tags (`:jang`, `:gae`, `:myeong`, `:mari`). For Sino-Korean counters, digits are standard and `@count` without `:word` suffices. Implementation: ~40 lines in the Korean `@count` transform, checking whether the counter tag belongs to the native or Sino-Korean set. Falls back to digits for numbers > 10.

Note: The Chinese `@count:word` uses 一/两/三 uniformly. Korean is more complex because number system selection varies by counter. The implementation should be per-language in the `@count` transform, not a shared CJK behavior.

---

### 3. No Gender, No Articles, No Plural Inflection

**Verdict: FULLY SUPPORTED — No issues**

Korean has none of these features. This dramatically simplifies translation:

- `:a`/`:an` tags on English terms are irrelevant in Korean context
- `:masc`/`:fem`/`:neut` tags from other languages' definitions are irrelevant
- `{$target:other}` for plural can be handled with a `*` default or simply using the bare form
- `:match` blocks with gender branches just use the `*` default

```
// ko.rlf
card = :jang "카드";
ally = :myeong "아군";
enemy = :myeong "적";

// No gender branching needed
when_dissolved_trigger($target) = "{$target}이(가) 해체되었을 때, ";
// Same form regardless of what $target is
```

The `:match` requirement for a `*` default ensures Korean can always use the fallback. **No issues.**

---

### 4. Word Order (SOV / Topic-Comment)

**Verdict: SUPPORTED — Translation files control word order**

Korean is strictly SOV (Subject-Object-Verb), with the verb always final. English is SVO. This means every effect phrase must be completely restructured:

| English (SVO) | Korean (SOV) |
|---|---|
| "Draw 3 cards." | "카드 3장을 뽑는다." (card 3-counter-OBJ draw) |
| "Dissolve an enemy." | "적 1명을 해체한다." (enemy 1-counter-OBJ dissolve) |
| "Gain 2 energy." | "에너지 2를 얻는다." (energy 2-OBJ gain) |

Each phrase defines its own word order in the translation file. The Rust code passes the same parameters regardless of language. This is the correct design.

**Minor concern — top-level concatenation order:** Section 2.3 states the ability serializer concatenates trigger + effect. In Korean, the trigger (a subordinate clause) naturally precedes the main clause, same as English:

- "아군을 소환할 때, 카드 3장을 뽑는다." (when you materialize an ally, draw 3 cards)

This ordering works. Korean subordinate clauses use clause-final verb forms (할 때, when-doing) that naturally precede the main verb. **No issue with concatenation order for Korean.**

---

### 5. Honorific Level (경어체)

**Verdict: NOT A DYNAMIC CONCERN — Translation-time decision**

Korean has multiple speech levels. Card game text typically uses one of:

- **해라체 (plain/assertive):** "뽑는다" (draws), "해체한다" (dissolves) — authoritative, like rulebook commands
- **합쇼체 (formal polite):** "뽑으십시오" (please draw), "해체하십시오" (please dissolve) — very formal
- **해요체 (informal polite):** "뽑으세요" (please draw), "해체하세요" (please dissolve) — conversational

Most Korean card games use 해라체 for card effects and 합쇼체 for UI/prompts. This is a translation-time decision, not a dynamic one. The translator picks verb endings consistently across all phrases.

**Minor concern:** Prompt phrases (e.g., `prompt_choose_character`, `prompt_select_from_void`) should use a polite form (합쇼체 or 해요체), while card effect text uses the plain form (해라체). This distinction is entirely in the translation file. **No Rust code impact.**

---

### 6. Particles with Dynamic Targets — The Detailed Challenge

**Verdict: REQUIRES CAREFUL `@particle` IMPLEMENTATION**

The hardest Korean-specific problem: when a phrase template uses a dynamic parameter with a particle, the particle form depends on the *rendered text* of that parameter.

```
// ko.rlf
dissolve_target($target) = "{$target}{@particle:obj $target} 해체한다.";
```

If `$target` renders to "고대인" (Ancient, ends in ㄴ consonant): "고대인**을** 해체한다."
If `$target` renders to "아군" (ally, ends in ㄴ consonant): "아군**을** 해체한다."
If `$target` renders to "사과" (apple, ends in vowel): "사과**를** 해체한다."

The `@particle` transform receives the `$target` Phrase as its reference and must:
1. Render the Phrase to text
2. Strip any trailing HTML markup tags
3. Find the last visible Hangul syllable
4. Decompose it to check for a final consonant (받침)
5. Select the correct particle allomorph

**Hangul decomposition logic:** A Hangul syllable block (U+AC00-U+D7A3) encodes (initial, medial, final) consonants. If `(codepoint - 0xAC00) % 28 == 0`, there is no final consonant (받침) → vowel-final. Otherwise → consonant-final. This is a simple arithmetic check, ~10 lines.

**What about non-Hangul endings?** If the rendered text ends in a Latin character (e.g., from a proper name or game term left untranslated), Korean convention is to treat it as consonant-final. If it ends in a digit, the digit's Korean pronunciation determines the particle:
- 0 (공/영): consonant ㅇ → consonant-final
- 1 (일): consonant ㄹ → consonant-final
- 2 (이): vowel → vowel-final
- 3 (삼): consonant ㅁ → consonant-final
- 4 (사): vowel → vowel-final
- 5 (오): vowel → vowel-final
- 6 (육): consonant ㄱ → consonant-final
- 7 (칠): consonant ㄹ → consonant-final
- 8 (팔): consonant ㄹ → consonant-final
- 9 (구): vowel → vowel-final

**RECOMMENDATION KO-3 (MINOR):** The `@particle` transform should handle three cases for Korean:
1. **Hangul syllable:** decompose and check for 받침 (most common case)
2. **ASCII digit:** use lookup table for Korean pronunciation
3. **Other (Latin letter, symbol):** default to consonant-final (은/이/을)

This is ~30 lines of logic in the Korean `@particle` implementation.

---

### 7. `@cap` Transform on Korean Text

**Verdict: CORRECT — `@cap` is a no-op on CJK**

Korean (like Chinese and Japanese) does not have upper/lowercase distinction. Section 9.7 of the migration plan correctly notes: "`@cap` is a no-op on CJK characters." Section 9.9 convention #2 states each phrase controls its own capitalization via `@cap`. For Korean phrases, `@cap` simply does nothing, which is correct.

**Minor concern:** Verify `@cap` correctly identifies Hangul characters (U+AC00-U+D7A3, U+1100-U+11FF, U+3130-U+318F) as having no case distinction and skips them. The Rust `char::is_uppercase()` / `char::to_uppercase()` methods handle this correctly for Unicode, but a test case would be prudent.

---

### 8. Structural Connectors

**Verdict: SUPPORTED — Korean uses different connectors**

Korean structural connectors differ significantly from English:

| English phrase | Korean equivalent | Notes |
|---|---|---|
| `then_joiner` = ", then " | ", 그 후 " or connective verb form | Korean prefers verb chaining (고, 서) |
| `and_joiner` = " and " | ", 그리고 " or 고 connective | Often attached to verb, not standalone |
| `period_suffix` = "." | "." | Same |
| `cost_effect_separator` = ": " | ": " | Same |
| `once_per_turn_prefix` = "Once per turn, " | "턴당 한 번, " | |
| `until_end_of_turn_prefix` = "Until end of turn, " | "턴 종료까지, " | |
| `you_may_prefix` = "you may " | suffix pattern preferred | See below |

**Important note on "you may":** English uses a prefix ("you may draw a card"). Korean typically uses a suffix or modal verb ending: "카드를 뽑을 수 있다" (card-OBJ draw-can). The `you_may_prefix` phrase in Korean would need to wrap the entire effect, not just prepend text. Since each phrase controls its own text, the Korean translator can restructure `you_may_prefix` as needed:

```
// ko.rlf
you_may_prefix = "";  // Empty — handled by modifying the effect verb form
// OR: effect phrases have a "may" variant that changes the verb ending
```

Alternatively, if the Rust code concatenates `you_may_prefix + effect`, the Korean `you_may_prefix` could be empty, and the effect phrase could use a conditional verb ending. This is slightly awkward but workable within the existing architecture.

**RECOMMENDATION KO-4 (MINOR):** Consider whether "you may" effects should be a separate phrase that wraps the effect (e.g., `optional_effect($effect) = "{$effect:may}"`) rather than a prefix. This would let Korean use a suffix pattern naturally. However, this would require Rust code changes and affects all languages. The prefix approach with an empty Korean prefix is an acceptable workaround.

---

### 9. Predicate Phrases and `:from` Tag Propagation

**Verdict: SUPPORTED — `:from` propagates counter tags**

Korean needs counter tags (`:jang`, `:gae`, `:myeong`) to propagate through compound phrases. The `:from` mechanism handles this:

```
// ko.rlf
ally = :myeong "아군";
subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";
// subtype(ally) inherits :myeong from Korean ally definition
```

A downstream phrase can then use `@count` with the propagated tag:

```
count_allies($n) = :match($n) {
    1: "{ally} 한 명",
    *other: "{ally} {$n}명",
};
```

The migration plan's Section 9.6 correctly states that each language defines its own tags on its own terms. Korean defines counter tags; these propagate via `:from`. **No issues.**

---

### 10. `@count` Spacing Convention

**Verdict: MINOR FORMATTING CONCERN**

Korean `@count` output format varies by convention:
- "3장" (no space) — most common for card games
- "3 장" (with space) — sometimes seen in formal text
- "세 장" (native number + space) — when using word-form numbers

The `@count` transform implementation should **not** insert a space between the digit and the counter for Korean (unlike some other languages). The stdlib example shows "3장카드" in the Korean section, which is correct for the counter but missing a space before the noun. The conventional format for card text would be either:
- "카드 3장" (noun first, then count — more natural Korean)
- "3장의 카드" (count + possessive particle + noun — also valid)

Since each phrase controls its own template, the translator decides placement:

```
// ko.rlf
draw($n) = "카드 {$n}장을 뽑는다.";  // noun-first order
// OR
draw($n) = "{@count($n) card}을 뽑는다.";  // using @count
```

**RECOMMENDATION KO-5 (MINOR):** Verify that the Korean `@count` transform produces "{n}{counter}" without space (e.g., "3장") as this is the dominant convention in Korean game text. The noun placement relative to the count is controlled by the phrase template.

---

### 11. Full-Width Punctuation

**Verdict: NOT NEEDED — Korean uses ASCII punctuation**

Unlike Chinese (which uses full-width period 。 and comma ，), Korean uses standard ASCII period (.) and comma (,) in modern text. The `period_suffix` phrase does not need a Korean override for punctuation character. **No issues.**

---

### 12. Keyword Display

**Verdict: SUPPORTED — Keywords are named phrases**

Korean would display keywords using their Korean translations inside color tags:

```
// ko.rlf
dissolve = "<color=#AA00FF>해체</color>";
banish = "<color=#AA00FF>추방</color>";
materialize = "<color=#AA00FF>소환</color>";
reclaim = "<color=#AA00FF>회수</color>";
prevent = "<color=#AA00FF>방해</color>";
kindle($k) = "<color=#AA00FF>점화</color> {$k}";
foresee($n) = "<color=#AA00FF>예견</color> {$n}";
```

Each keyword is a separate RLF term. Korean translations simply replace the English text inside the same color tags. **No issues.**

**Important interaction with particles:** When a keyword is followed by a particle, the particle must come after the closing `</color>` tag. This reinforces KO-CRIT-1 — the `@particle` transform must look *inside* markup to find the last visible character.

---

## 13. Case Study: "Draw 3 cards." in Korean

Demonstrating the full translation pipeline:

```
// ko.rlf — Translation file
card = :jang "카드";

cards($n) = :match($n) {
    1: "카드 한 장",
    *other: "카드 {$n}장",
};

draw_cards_effect($c) = "{cards($c)}을 뽑는다.";
// c=1 → "카드 한 장을 뽑는다." (draw one card)
// c=3 → "카드 3장을 뽑는다." (draw 3 cards)
```

Note: The particle 을 is hardcoded here because "장" always ends in consonant ㅇ. When the preceding text is always the same counter, the translator can hardcode the particle. `@particle` is only needed when the preceding text varies dynamically.

---

## 14. Case Study: "Dissolve an enemy Ancient." in Korean

```
// ko.rlf
ancient = :myeong "고대인";
enemy_subtype($t) = :from($t) "적 {$t}";
dissolve = "<color=#AA00FF>해체</color>";

dissolve_target($target) = "{$target}{@particle:obj $target} {@cap dissolve}한다.";
// enemy_subtype(ancient) → "적 고대인"
// dissolve_target(enemy_subtype(ancient)) → "적 고대인을 해체한다."
// (enemy Ancient-OBJ dissolve)
// "인" ends in ㄴ (consonant) → 을
```

---

## 15. RLF Framework Changes Required for Korean

### 15.1 `@particle` Must Strip HTML Markup (CRITICAL — KO-CRIT-1)

**Priority: HIGH**
**Estimated effort:** ~10 lines

The `@particle` transform must skip trailing HTML markup tags when inspecting the final character. Pattern: strip `</...>` tags from the end to find the last visible character, then apply Hangul decomposition or digit lookup.

This is analogous to `@cap` skipping *leading* HTML tags. Both transforms need to "see through" markup to the actual rendered text.

**Test cases:**
- `{@particle:obj dissolve}` where `dissolve` = `<color=#AA00FF>해체</color>` → "를" (체 ends in no 받침 → vowel)
- Wait: 체 = U+CCB4. (0xCCB4 - 0xAC00) = 0x20B4 = 8372. 8372 % 28 = 8372 - 299*28 = 8372 - 8372 = 0. No 받침 → vowel-final → "를". But "해체**를**" — that's correct.
- `{@particle:obj banish}` where `banish` = `<color=#AA00FF>추방</color>` → "을" (방 = U+BC29, (0xBC29 - 0xAC00) = 0x1029 = 4137, 4137 % 28 = 4137 - 147*28 = 4137 - 4116 = 21 ≠ 0 → has 받침 → consonant-final → "을"). "추방**을**" — correct.

### 15.2 Additional Particle Contexts (SIGNIFICANT — KO-1)

**Priority: MEDIUM**
**Estimated effort:** ~15 lines

Add to the Korean `@particle` transform:

| Context | Vowel-final | Consonant-final | Usage |
|---|---|---|---|
| `:and` | 와 | 과 | Conjunctive ("X and Y") |
| `:copula` | 다 | 이다 | Copula ("is X") |
| `:dir` | 로 | 으로 | Direction/instrument ("to/with X") |

### 15.3 Native Korean Number Words in `@count:word` (SIGNIFICANT — KO-2)

**Priority: MEDIUM**
**Estimated effort:** ~40 lines

The `@count:word` modifier (Section 9.8.4) must handle Korean's dual number system. For counters using native Korean numbers (`:jang`, `:gae`, `:myeong`, `:mari`), produce counting forms:

| n | Native counting form | Sino-Korean form |
|---|---|---|
| 1 | 한 | 일 |
| 2 | 두 | 이 |
| 3 | 세 | 삼 |
| 4 | 네 | 사 |
| 5 | 다섯 | 오 |
| 6 | 여섯 | 육 |
| 7 | 일곱 | 칠 |
| 8 | 여덟 | 팔 |
| 9 | 아홉 | 구 |
| 10 | 열 | 십 |

For n > 10, both systems fall back to digits.

Implementation: the Korean `@count` transform checks whether the counter tag belongs to the native set. If so and `:word` is specified, use native counting forms. Otherwise use digits.

Note: In practice, Dreamtides card text mostly uses small numbers (1-5) and displays them as digits on cards. Korean translators may opt to use digits universally and reserve word forms for the "a card" (한 장) singular case via `:match`. The `@count:word` modifier provides the option but isn't strictly required for an MVP Korean translation.

### 15.4 Digit-Based Particle Selection (MINOR — KO-3)

**Priority: LOW**
**Estimated effort:** ~10 lines

If the last visible character is an ASCII digit, determine consonant/vowel ending based on Korean pronunciation of that digit. Lookup table:

```
0 → consonant (공/영, ends in ㅇ)
1 → consonant (일, ends in ㄹ)
2 → vowel (이, ends in vowel)
3 → consonant (삼, ends in ㅁ)
4 → vowel (사, ends in vowel)
5 → vowel (오, ends in vowel)
6 → consonant (육, ends in ㄱ)
7 → consonant (칠, ends in ㄹ)
8 → consonant (팔, ends in ㄹ)
9 → vowel (구, ends in vowel)
```

This matters for patterns like "3점을 얻는다" vs "2점를 얻는다" — wait, those would likely be hardcoded particles since 점 always follows. But for dynamic patterns like "에너지 {$e}를/을 얻는다", the digit's pronunciation matters.

---

## 16. Verification Checklist for Korean

Before writing Korean translation files, verify:

- [ ] `@particle` strips trailing HTML markup to find last visible character (KO-CRIT-1)
- [ ] `@particle:subj` produces 가/이 correctly for Hangul syllables
- [ ] `@particle:obj` produces 를/을 correctly for Hangul syllables
- [ ] `@particle:topic` produces 는/은 correctly for Hangul syllables
- [ ] `@particle:and` produces 와/과 (KO-1)
- [ ] `@particle:copula` produces 다/이다 (KO-1)
- [ ] `@particle:dir` produces 로/으로 (KO-1)
- [ ] `@particle` handles ASCII digits correctly (KO-3)
- [ ] `@particle` defaults to consonant-final for non-Hangul, non-digit characters
- [ ] `@cap` is a no-op on Hangul characters (U+AC00-U+D7A3)
- [ ] `@count` produces "{n}{counter}" without space for Korean
- [ ] `@count:word` produces native Korean counting forms for appropriate counters (KO-2)
- [ ] `:from` propagates Korean counter tags (`:jang`, `:myeong`, etc.) through composition chain

---

## 17. Changes NOT Recommended

The following were evaluated and rejected:

- **Honorific-level transforms:** Card text uses a single speech level chosen at translation time. No dynamic switching needed.
- **Full-width punctuation:** Korean uses ASCII punctuation. No override needed.
- **Topic/subject distinction transform:** The choice between topic (은/는) and subject (이/가) particles is a semantic distinction made by the translator in each phrase template. It cannot be automated.
- **Plural markers:** Korean has optional plural markers (들) but they are rarely used in card game text. Translators add them manually in templates where needed.
- **Word spacing (띄어쓰기) transforms:** Korean word spacing is handled by the translator in phrase templates. No automated spacing logic needed.

---

## 18. Summary of Recommendations

| ID | Priority | Description | Effort | Section |
|---|---|---|---|---|
| KO-CRIT-1 | HIGH | `@particle` must strip trailing HTML markup | ~10 lines | 15.1 |
| KO-1 | MEDIUM | Add `:and`, `:copula`, `:dir` particle contexts | ~15 lines | 15.2 |
| KO-2 | MEDIUM | `@count:word` native Korean number support | ~40 lines | 15.3 |
| KO-3 | LOW | Digit-based particle selection in `@particle` | ~10 lines | 15.4 |
| KO-4 | LOW | Consider wrapping approach for "you may" prefix | Design only | 8 |
| KO-5 | LOW | Verify `@count` no-space convention for Korean | Testing only | 10 |

**Total estimated RLF framework changes for Korean: ~75 lines**

---

## 19. Cross-Language Notes

Korean shares characteristics with several other languages in this review:

- **Japanese:** Both use `@count` with counters and `@particle`. Korean particles are phonology-dependent (harder); Japanese particles are fixed (easier). Both need the HTML-stripping fix for `@particle` (KO-CRIT-1).
- **Chinese:** Both use `@count` with classifiers. Chinese is simpler (single number system); Korean has native/Sino-Korean duality. Both benefit from `@count:word` (Section 9.8.4).
- **Turkish:** Both are agglutinative to a degree (Korean less so than Turkish). Korean particles are postpositional but are separate words, not suffixed. No need for `@inflect`.
- **Arabic:** Both have complex morphological rules but of entirely different types. No shared concerns.
- **French:** Both have phonology-dependent morphology (French liaison, Korean particles). The `@particle` approach for Korean is analogous to `@liaison` for French — runtime text inspection rather than static tags.
