# Arabic (العربية) i18n Review of Serializer RLF Migration Plan

**Reviewer:** ar-agent (Arabic localization specialist — Modern Standard Arabic / فصحى)
**Date:** 2026-02-08
**Status:** PASS with recommendations

---

## Executive Summary

The migration plan is **well-designed for Arabic support**. The RLF framework's 6-category CLDR plural system (`zero`, `one`, `two`, `few`, `many`, `other`), multi-dimensional variant blocks, `:match` branching, `:from` tag inheritance, and the `@al` definite article transform provide a solid foundation for Arabic localization. Arabic has among the richest morphological systems of any target language, and the framework can accommodate it.

**Verdict: No Rust code changes needed for Arabic support**, provided the plan's key architectural decisions (predicates returning `Phrase`, not `String`) are implemented as specified and the `:from` tag propagation guarantee holds.

I identify **6 concrete issues**, ranging from medium to informational severity. None are blockers for Phase 2, but two deserve attention during implementation to avoid Arabic translator friction.

---

## Analysis by Concern Area

### 1. CLDR Plural Categories — Dual Number and Complex Plurals

**Verdict: FULLY SUPPORTED**

Arabic has the most complex CLDR plural system of any language, requiring **all 6 categories**:

| CLDR Category | Arabic Range | Example ("card" = بطاقة) |
|---------------|-------------|--------------------------|
| `zero` | 0 | لا بطاقات (no cards) |
| `one` | 1 | بطاقة واحدة (one card) |
| `two` | 2 | بطاقتان (two cards — dual) |
| `few` | 3-10 | بطاقات (cards, sound fem. plural) |
| `many` | 11-99 | بطاقة (card — singular form with plural meaning!) |
| `other` | 100+ | بطاقة (card — same as `many`) |

The RLF framework already documents Arabic with all 6 plural categories in APPENDIX_STDLIB.md. The `card` term example in the stdlib shows:

```
card = :fem :moon {
    one: "بطاقة",
    two: "بطاقتان",
    few: "بطاقات",
    many: "بطاقة",
    other: "بطاقات",
};
```

**Key nuance — numbers 11-99 use the singular noun form:** "11 بطاقة" (eleven card), not "11 بطاقات" (eleven cards). This is correctly handled by CLDR mapping 11-99 to `many`, where the variant provides the singular form. **No issue.**

**Dual number (مثنى):** Arabic has a grammatical dual for exactly 2 items. "بطاقتان" (two-cards) is a single morphological form, not "2 بطاقات". The `two` CLDR category handles this perfectly via `:match` exact-2 branching:

```
// ar.rlf
cards($n) = :match($n) {
    0: "لا بطاقات",
    1: "بطاقة واحدة",
    2: "بطاقتان",
    *other: "{$n} {card:$n}",
};
```

**No issues. Arabic plurals are fully expressible.**

---

### 2. RTL Text Direction and HTML Markup (Issue AR-1, MEDIUM)

**Problem:**

Arabic is written right-to-left (RTL). The codebase embeds LTR HTML-like markup tags (`<color=#AA00FF>`, `<b>`, `</b>`, `<u>`) within text strings. When Arabic text contains these tags, the bidirectional text algorithm (Unicode BiDi) can produce incorrect visual ordering.

Example — current English keyword rendering:
```
dissolve = "<color=#AA00FF>dissolve</color>";
```

Arabic equivalent:
```
dissolve = "<color=#AA00FF>حَلّ</color>";
```

The HTML tags are LTR (Latin characters). In an RTL context, the BiDi algorithm treats `<color=#AA00FF>` as a "neutral" or LTR run, which can cause the tag to visually appear in the wrong position or break the flow.

**Specific scenarios of concern:**

1. **Mixed Arabic + numbers:** "اسحب 3 بطاقات" (Draw 3 cards). The digit "3" is treated as European number (weak LTR). Combined with color markup around it, the visual order may scramble.

2. **Colored keywords inside Arabic sentences:** If a keyword like "حَلّ" (dissolve) is wrapped in `<color>` tags inside an Arabic sentence, the BiDi algorithm sees: RTL text → LTR markup → RTL text → LTR markup → RTL text. Each boundary is a potential reordering point.

3. **The `@cap` transform:** Section 9.9 states `@cap` should be a no-op on CJK characters. Arabic has uppercase/lowercase equivalents for... nothing. Arabic has no letter case at all. `@cap` should also be a no-op for Arabic script. The verification checklist (Section 9.7) mentions CJK but not Arabic.

**Impact:**

This is a **rendering layer concern**, not a framework concern. The RLF framework produces text strings; the game client's text renderer (Unity TextMeshPro or similar) handles BiDi layout. However, if the renderer doesn't properly handle BiDi with embedded markup, Arabic text will be garbled.

**Recommendation AR-1:**

1. Add `@cap` no-op verification for Arabic script to Section 9.7 checklist: "- [ ] `@cap` is a no-op on Arabic, Hebrew, and other scripts without letter case"
2. Document in the plan that Arabic localization requires the client's text rendering system to support Unicode BiDi algorithm (UAX #9) with markup passthrough. This is a client-layer requirement, not an RLF issue.
3. Consider wrapping Arabic text segments in Unicode BiDi control characters (U+200F RIGHT-TO-LEFT MARK, or explicit embedding U+202B/U+202C) if the renderer doesn't handle it natively. This can be done in the translation file.

**Severity: MEDIUM** — Not an RLF framework issue, but must be addressed at the rendering layer before Arabic ships.

---

### 3. Broken Plurals (جمع التكسير) (Issue AR-2, LOW)

**Problem:**

Arabic has two plural formation strategies:
- **Sound plurals (جمع سالم):** Regular suffixation. "مهندس" → "مهندسون" (engineer → engineers). Predictable.
- **Broken plurals (جمع التكسير):** Internal vowel pattern change. "كتاب" → "كتب" (book → books), "عدو" → "أعداء" (enemy → enemies), "لاعب" → "لاعبون" or "لُعّاب" (player → players). Unpredictable — each noun has its own broken plural pattern.

Most common Arabic nouns use broken plurals. These cannot be generated by any rule — they must be listed as explicit variant forms.

**Verdict: FULLY SUPPORTED by the variant system.**

RLF's variant blocks are designed exactly for this. Each Arabic term lists all its morphological forms explicitly:

```
// ar.rlf
enemy = :masc :sun {
    one: "عدو",
    two: "عدوان",
    few: "أعداء",     // broken plural
    many: "عدو",
    other: "أعداء",
};

ally = :masc :moon {
    one: "حليف",
    two: "حليفان",
    few: "حلفاء",     // broken plural
    many: "حليف",
    other: "حلفاء",
};
```

Since Arabic plurals are inherently irregular and listed per-noun anyway, the variant system's explicit enumeration is actually the *ideal* approach. No rule-based plural transform would work for Arabic.

**No action needed.** The existing design is correct.

---

### 4. Definiteness — The `@al` Transform and Construct State (Issue AR-3, MEDIUM)

**Problem:**

Arabic definiteness is marked by the prefix "ال" (al-), not a separate word. The APPENDIX_STDLIB.md documents the `@al` transform with `:sun`/`:moon` tags for phonological assimilation:

- **Moon letters (حروف قمرية):** "ال" stays as-is: البطاقة (al-biṭāqa, "the card")
- **Sun letters (حروف شمسية):** "ال" assimilates: الشمس → اشّمس (ash-shams, "the sun")

This is correctly designed. However, two additional concerns arise:

#### 4a. Construct State (الإضافة — Idafa)

Arabic possessive/genitive constructions use the **construct state**, where the first noun loses its definite article and takes a special form:

- "بطاقة العدو" (biṭāqat al-ʿaduw) = "the card of the enemy" = "the enemy's card"
- NOT "البطاقة العدو" — the first noun must be indefinite in construct

The construct state changes the noun form (adding tā' marbūṭa pronunciation, changing case endings). In the game context, phrases like "enemy card" or "your card" use construct-like patterns.

**Impact:** The RLF variant system handles this by defining construct-state variants:

```
// ar.rlf
card = :fem :moon {
    one: "بطاقة",
    def: "البطاقة",        // definite: the card
    construct: "بطاقة",    // construct state: card of...
    two: "بطاقتان",
    few: "بطاقات",
    many: "بطاقة",
    other: "بطاقات",
};

// "enemy card" as construct:
enemy_card = "{card:construct} {@al enemy}";
// → "بطاقة العدو" (card-of the-enemy)
```

This works within the existing variant system.

#### 4b. `@al` Interaction with `:from`

When a compound phrase uses `:from($s)` to inherit tags from a subtype, the `@al` transform must read `:sun`/`:moon` from the inherited tags:

```
subtype($s) = :from($s) "<b>{$s}</b>";
// If $s carries :sun, then @al on the result should assimilate
```

**Recommendation AR-3:** Verify that `:from` propagates `:sun`/`:moon` tags correctly so `{@al subtype($s)}` produces the correct assimilation. Add this to the verification checklist (Section 9.7):
"- [ ] `@al` correctly reads `:sun`/`:moon` tags inherited via `:from`"

**Severity: MEDIUM** — Construct state is expressible with variants. `@al` + `:from` interaction needs verification.

---

### 5. Gender Agreement on Everything (Issue AR-4, LOW)

**Problem:**

Arabic requires gender agreement on:
- **Verbs:** "اسحب" (draw, masc. imperative) vs "اسحبي" (draw, fem. imperative)
- **Adjectives:** "عدو قوي" (strong enemy, masc.) vs "عدوة قوية" (strong enemy, fem.)
- **Numbers 3-10:** Take the **opposite gender** of the counted noun! "ثلاث بطاقات" (three-fem cards-fem) but "ثلاثة لاعبين" (three-masc players-masc). This is the infamous Arabic number-noun agreement reversal.
- **Demonstratives, relative pronouns, etc.**

**Card game context:** Card text typically uses masculine imperative for commands ("اسحب" = draw, "حَلّ" = dissolve). This is the conventional register for instructions and rules text. Gender agreement on verbs is therefore fixed at masculine imperative in most cases.

**The number-noun reversal (3-10):** This is the trickiest Arabic grammar rule. For numbers 3-10, the number word takes the OPPOSITE gender of the noun:

| Count | "cards" (بطاقة, fem.) | "players" (لاعب, masc.) |
|-------|----------------------|------------------------|
| 3 | ثلاث بطاقات | ثلاثة لاعبين |
| 4 | أربع بطاقات | أربعة لاعبين |
| 10 | عشر بطاقات | عشرة لاعبين |

This is handleable via multi-parameter `:match` on both count and gender:

```
// ar.rlf
n_cards($n, $entity) = :match($n, $entity) {
    1.*fem: "{$entity:one} واحدة",
    1.*masc: "{$entity:one} واحد",
    2.*: "{$entity:two}",
    few.fem: "{reverse_number_fem($n)} {$entity:few}",
    few.masc: "{reverse_number_masc($n)} {$entity:few}",
    *many.*: "{$n} {$entity:many}",
};
```

Where `reverse_number_fem` and `reverse_number_masc` provide the reversed-gender number words. This is verbose but correct.

**Recommendation AR-4:** The number-noun reversal requires that number phrases receive BOTH the count AND the entity's gender tag. Multi-parameter `:match($n, $entity)` handles this. No framework changes needed, but Arabic translators should be warned about this pattern in documentation.

**Severity: LOW** — Fully expressible in `:match`. Just requires careful translation authoring.

---

### 6. `@al` Transform — Sun/Moon Letter Assimilation Details (Issue AR-5, LOW)

**Problem:**

The `@al` transform must correctly handle sun letter assimilation. The 14 sun letters (ت ث د ذ ر ز س ش ص ض ط ظ ل ن) cause the "ل" in "ال" to assimilate:

- Moon: ال + بطاقة → البطاقة (al-biṭāqa)
- Sun: ال + شمس → الشّمس (ash-shams) — but in unvocalized text, it's written as الشمس with optional shaddah (ّ)

In modern unvocalized Arabic (which card games would use), the written form is always "ال" + noun regardless of sun/moon — the assimilation is phonetic only. The visual text is the same: "ال" prefix.

**However:** If the game includes vocalized/diacritized text (with tashkeel), sun letter assimilation adds a shaddah (ّ) to the initial letter. For unvocalized text (standard for games), `@al` simply prepends "ال".

**Recommendation AR-5:** If `@al` is implemented for unvocalized text (which is standard), it can simply prepend "ال" unconditionally. The `:sun`/`:moon` distinction is only needed for:
1. Vocalized text (adding shaddah)
2. Transliteration output

For a card game, unvocalized text is standard. The `:sun`/`:moon` tags can be retained for completeness but the transform can be simplified.

**Severity: LOW** — The existing design is overly precise for a card game context but harmless.

---

### 7. Structural Connectors and Punctuation (Issue AR-6, INFO)

**Verdict: SUPPORTED by named connector phrases.**

Arabic punctuation differs from English:
- Period: same (.) or Arabic period (۔) — typically "." is used in modern text
- Comma: "،" (Arabic comma, U+060C) not ","
- Semicolon: "؛" (Arabic semicolon, U+061B)
- Question mark: "؟" (Arabic question mark, U+061F)
- Colon: same (:) but may appear on the opposite side in RTL

Section 9.9 Convention 4 requires structural connectors to be named RLF phrases. This is exactly right for Arabic — the Arabic translation file overrides:

```
// ar.rlf
then_joiner = "، ثم ";        // Arabic comma + "then"
and_joiner = " و";             // "and" (waw)
period_suffix = ".";            // Standard period is fine
cost_effect_separator = ": ";   // Same
```

**No issues. Fully supported.**

---

### 8. Case System — Nominative/Accusative/Genitive (Issue AR-7, INFO)

Arabic has 3 grammatical cases marked by short vowel diacritics (i'rab):
- Nominative (مرفوع): -u / -un
- Accusative (منصوب): -a / -an
- Genitive (مجرور): -i / -in

In **modern unvocalized Arabic** (standard for all published media including card games), these diacritics are omitted. The word form is identical across all three cases for most nouns.

**Verdict: Not an issue for card game localization.** Case marking is purely phonetic and invisible in written text. No case variants needed in translation files.

Exceptions (dual and sound masculine plural endings ARE visible):
- Nominative dual: بطاقتان (-ān)
- Accusative/genitive dual: بطاقتين (-ayn)

These are handled by the variant system:
```
card = :fem :moon {
    two: "بطاقتان",           // nominative dual
    two.acc: "بطاقتين",      // accusative dual (if needed)
};
```

Most card text won't need case-distinguished duals, but the framework supports it if a translator needs it.

---

## RLF Framework Changes for Arabic

### RFC-AR-1: `@cap` No-Op for Arabic Script (LOW)

The Section 9.7 verification checklist mentions `@cap` is a no-op for CJK characters but doesn't mention Arabic. Arabic script has no uppercase/lowercase distinction. `@cap` must be a no-op when the first visible character is Arabic (Unicode block U+0600-U+06FF, U+0750-U+077F, U+FB50-U+FDFF, U+FE70-U+FEFF).

**Implementation:** `@cap` already skips HTML markup to find the first visible character. It should also check if that character's Unicode script property is Arabic (or any other script without case) and return unchanged.

**Effort:** ~5 lines in the `@cap` implementation.

### RFC-AR-2: `@al` Simplified Unvocalized Mode (INFO — No Change Needed)

The `@al` transform as designed in APPENDIX_STDLIB.md reads `:sun`/`:moon` tags. For unvocalized Arabic, this distinction is unnecessary — `@al` just prepends "ال". However, the current design is harmless and provides future-proofing for vocalized text. **No change recommended.**

### RFC-AR-3: RTL BiDi Control Characters (LOW — Documentation Only)

Add a note to the migration plan or RLF documentation that Arabic (and Hebrew) translation files may need to include Unicode BiDi control characters:
- U+200F (RIGHT-TO-LEFT MARK) — after digits or markup to restore RTL context
- U+2067/U+2069 (RIGHT-TO-LEFT ISOLATE / POP DIRECTIONAL ISOLATE) — around embedded LTR runs

This is a translator-level concern, not a framework change. But documenting the recommendation prevents confusion.

---

## Cross-Reference with Existing Plan Sections

| Plan Section | Arabic Impact | Notes |
|-------------|--------------|-------|
| 9.1 Case Declension | LOW | Arabic cases invisible in unvocalized text |
| 9.2 Gender Agreement | MEDIUM | Arabic needs gender tags for number reversal |
| 9.3 Personal "a" | N/A | Arabic has no equivalent construction |
| 9.4 Chinese Classifiers | N/A | Arabic doesn't use classifiers |
| 9.5 Separable Verbs | N/A | Arabic verbs don't separate |
| 9.6 Tag System Design | GOOD | `:sun`/`:moon` already designed |
| 9.7 Verification Checklist | NEEDS UPDATE | Add `@cap` Arabic no-op check |
| 9.8 Framework Changes | NO ADDITIONS | No new transforms needed |
| 9.9 Cross-Language Conventions | GOOD | Connector phrases handle Arabic punctuation |

---

## Case Studies — Arabic Translations

### "Draw 3 cards." (اسحب 3 بطاقات)

| n | English | Arabic | Key Features |
|---|---------|--------|--------------|
| 0 | "Draw no cards." | "لا تسحب أي بطاقة." | Zero form, negation |
| 1 | "Draw a card." | "اسحب بطاقة." | Indefinite singular (no article) |
| 2 | "Draw 2 cards." | "اسحب بطاقتين." | Dual accusative form |
| 3 | "Draw 3 cards." | "اسحب 3 بطاقات." | Sound feminine plural (3-10) |
| 11 | "Draw 11 cards." | "اسحب 11 بطاقة." | Singular form after 11+ (tamyiz) |

RLF implementation:
```
// ar.rlf
card = :fem :moon {
    one: "بطاقة",
    two: "بطاقتين",     // accusative dual
    few: "بطاقات",
    many: "بطاقة",       // tamyiz singular for 11-99
    other: "بطاقات",
};

cards($n) = :match($n) {
    0: "لا تسحب أي بطاقة",
    1: "بطاقة",
    2: "{card:two}",
    *other: "{$n} {card:$n}",
};

draw_cards_effect($c) = "اسحب {cards($c)}.";
```

### "Dissolve an enemy Ancient." (حَلّ عتيقاً معادياً)

| Language | Output | Key Features |
|----------|--------|--------------|
| EN | "Dissolve an enemy Ancient." | `@a` reads `:an` |
| AR | "حَلّ عتيقاً معادياً." | Masc. imperative verb, accusative noun ending |

```
// ar.rlf
dissolve = "<color=#AA00FF>حَلّ</color>";
ancient = :masc :moon {
    one: "عتيق",
    acc: "عتيقاً",
};
enemy_modified($entity) = :from($entity) :match($entity) {
    masc: "معادياً",
    *fem: "معادية",
};
dissolve_target($target) = "{dissolve} {$target:acc} {enemy_modified($target)}.";
```

---

## Summary of Recommendations

| ID | Severity | Description | Plan Section |
|----|----------|-------------|-------------|
| AR-1 | MEDIUM | RTL + HTML markup BiDi handling — client rendering requirement | New: rendering layer |
| AR-2 | LOW | Broken plurals — no action, variant system handles perfectly | N/A |
| AR-3 | MEDIUM | Verify `@al` + `:from` tag propagation for `:sun`/`:moon` | 9.7 checklist |
| AR-4 | LOW | Number-noun gender reversal — verbose but expressible in `:match` | Documentation |
| AR-5 | LOW | `@al` sun/moon distinction optional for unvocalized text | N/A |
| AR-6 | INFO | Arabic punctuation — fully handled by named connector phrases | 9.9 Convention 4 |
| AR-7 | INFO | Case system invisible in modern Arabic — not a concern | N/A |
| RFC-AR-1 | LOW | `@cap` no-op for Arabic script — add to verification checklist | 9.7 |
| RFC-AR-2 | INFO | `@al` unvocalized simplification — no change needed | N/A |
| RFC-AR-3 | LOW | Document BiDi control character recommendations for translators | Documentation |

---

## Final Verdict

**PASS.** The migration plan's architecture is sound for Arabic. The combination of:
- 6-category CLDR plural system (covering Arabic's dual and complex plural breaks)
- Multi-dimensional variant blocks (for broken plurals, case-distinguished duals)
- `:match` with multi-parameter support (for number-noun gender reversal)
- `@al` transform with `:sun`/`:moon` tags (for definite article)
- Named structural connector phrases (for Arabic punctuation)
- `:from` tag inheritance (for compound phrase composition)

...provides everything Arabic needs. The two medium-priority items (RTL rendering and `@al`+`:from` verification) should be addressed, but neither requires Rust code changes to the serializer. Arabic translations can be written using only `.rlf` translation files, which is the stated goal.
