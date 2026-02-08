# Turkish (Turkce) i18n Review of Serializer RLF Migration Plan

**Reviewer**: Turkish language specialist
**Date**: 2026-02-08
**Scope**: Phase 2 migration plan at `docs/plans/serializer_rlf_migration.md`

---

## Executive Summary

The migration plan is **well-designed for Turkish**, largely thanks to the `@inflect` transform already present in the RLF standard library, which handles agglutinative suffix chains with vowel harmony. Turkish's six cases, possessive suffixes, plural markers, and vowel harmony are all expressible through `@inflect` with context chains like `@inflect:acc.poss3sg.pl`. The absence of grammatical gender in Turkish simplifies matters considerably -- `:match` on `:masc`/`:fem`/`:neut` is simply irrelevant and never triggers.

However, Turkish's agglutinative morphology creates **structural challenges** that the current plan does not fully address. A single Turkish word can encode what English expresses with 3-4 separate words (prepositions, articles, possessives), meaning phrase templates must compose differently. The `@inflect` transform handles individual noun suffixation, but several patterns in the migration plan assume isolating-language structure (separate articles, separate possessives, separate prepositions) that Turkish collapses into suffix chains.

**Critical issues found: 1**
**Significant issues found: 3**
**Minor concerns: 3**

---

## 1. Issue 1 (CRITICAL): Buffer Consonant Insertion in `@inflect`

### The Problem

When Turkish suffixes beginning with a vowel are added to a stem that ends in a vowel, a **buffer consonant** (typically "y", sometimes "n" or "s") must be inserted. The current `@inflect` documentation in APPENDIX_STDLIB.md shows suffix concatenation with vowel harmony but does not mention buffer consonant logic.

Examples:

| Stem | Suffix | Wrong | Correct | Buffer |
|------|--------|-------|---------|--------|
| elma (apple) | -ı (acc) | elmaı | elmayı | y |
| araba (car) | -ın (gen) | arabaın | arabanın | n |
| elma (apple) | -ım (poss1sg) | elmaım | elmam | contraction |
| su (water) | -u (acc) | suu | suyu | y |

Buffer consonant rules:
1. **"y" buffer** before vowel-initial case suffixes (-ı/-i/-u/-ü for acc, -a/-e for dat) when stem ends in vowel
2. **"n" buffer** before case suffixes when a possessive suffix (which ends in a vowel) precedes: "arabası" (his car) + acc → "arabasını" (not "arabasıı")
3. **"s" buffer** before 3rd-person possessive suffix (-ı) when stem ends in vowel: "araba" + -ı → "arabası" (not "arabaı")

If `@inflect` does not handle buffer consonant insertion, **every Turkish noun phrase with a vowel-ending stem will produce incorrect output** when case or possessive suffixes are applied. This is a large fraction of Turkish nouns.

### Severity

**CRITICAL**. Buffer consonants are not optional or rare -- they apply to every vowel-ending stem (a very large class of Turkish words including many game terms). Without buffer consonant handling, `@inflect` will produce ungrammatical Turkish for roughly half of all noun inflections.

### Recommendation

**BLOCK-TR-1**: The `@inflect` implementation for Turkish **must** include buffer consonant insertion logic. Specifically:
1. Inspect the final character of the current stem (after previous suffixes) before appending a new suffix
2. If stem ends in vowel and suffix starts with vowel, insert the appropriate buffer consonant:
   - "y" for case suffixes (accusative, dative, etc.)
   - "n" for case suffixes after possessive suffixes
   - "s" for 3rd-person possessive suffix on vowel-ending stems
3. Add this to Task 11 as a required sub-step for the Turkish `@inflect` implementation

Add this to Section 9.8 as a new item:

> **9.8.8 Turkish Buffer Consonant Insertion in `@inflect` -- HIGH**
>
> Turkish `@inflect` must handle buffer consonant insertion (y/n/s) when suffixes are appended to vowel-ending stems. Without this, accusative, dative, genitive, and possessive forms of vowel-ending nouns will be incorrect. ~40 lines in the Turkish suffix chain logic. Affects roughly 50% of Turkish nouns.

---

## 2. Issue 2 (SIGNIFICANT): Accusative Suffix as Definite Marker

### The Problem

Turkish has no articles (definite or indefinite) in the European sense. Instead, definiteness is expressed through the **accusative case suffix**:

- "Bir kart cek" = "Draw a card" (indefinite -- "bir" = one/a, no accusative suffix)
- "Karti cek" = "Draw the card" / "Draw that card" (definite -- accusative suffix -ı)
- "Kart cek" = "Draw card(s)" (generic, unmarked)

The English source uses `@a` for indefinite articles and direct reference for bare nouns. Turkish needs a different mechanism: the accusative suffix marks definiteness on the **object** noun, while subjects remain unmarked.

Consider the phrase `dissolve_target($target) = "{@cap dissolve} {@a $target}."`:
- English: "Dissolve an enemy." (`@a` adds "an")
- Turkish: "Bir dusmani erit." (indefinite enemy, accusative) or "Dusmani erit." (definite enemy, accusative)

The Turkish translator needs:
1. `@inflect:acc` on the target to mark it as a direct object
2. Optionally "bir" before it for indefiniteness
3. No suffix at all if the object is generic/indefinite non-specific

The `@a` transform is English-only and irrelevant in Turkish. But the **semantic distinction** (definite vs. indefinite vs. generic) that `@a` encodes in English must be expressible in Turkish through case suffix presence/absence and the word "bir".

### Severity

**SIGNIFICANT**. Turkish translators can work around this by defining separate phrase variants or using `:match` to select between definite/indefinite forms. However, the migration plan should acknowledge that "article" selection in Turkish is actually case suffix selection, and the phrase architecture must allow the Turkish translation file to apply `@inflect:acc` to `$target` parameters.

### Recommendation

**RFC-TR-1**: Verify that `@inflect` can be applied to `$target` parameters (Phrase values), not just term references. The pattern `{@inflect:acc $target}` must work where `$target` is a Phrase passed from the predicate serializer. This is the Turkish equivalent of `{@a $target}` -- it adds the accusative suffix to mark a definite direct object.

Add a note to Section 9.6 (Tag System Design):

> Turkish uses accusative case suffix (-ı/-i/-u/-ü) where European languages use definite articles. Turkish translators will use `@inflect:acc` on `$target` parameters in place of `@a`. The `@inflect` transform must accept Phrase parameters, not only term references.

---

## 3. Issue 3 (SIGNIFICANT): Possessive Suffix Chains Replace English Possessive Constructions

### The Problem

English uses separate words for possession: "your card", "your hand", "the opponent's void", "its spark", "that character's cost". Turkish expresses all of these as suffix chains on the possessed noun:

| English | Turkish | Analysis |
|---------|---------|----------|
| your card | kartın | kart + 2sg possessive |
| your hand | elin | el + 2sg possessive |
| your void | boslugun | bosluk + 2sg possessive |
| the opponent's void | rakibin boslugu | rakip + gen + bosluk + 3sg poss |
| its spark | kivılcımı | kivılcım + 3sg poss |
| that character's cost | o karakterin maliyeti | karakter + gen + maliyet + 3sg poss |

The migration plan has many phrases with English possessive constructions:
- `discard_your_hand_cost = "discard your hand"`
- `banish_your_void_cost = "Banish your void"`
- `gain_energy_equal_to_that_cost_effect = "gain energy equal to that character's cost."`

Turkish translators must replace "your X" with `{@inflect:poss2sg X}` and "X's Y" with `{@inflect:gen X} {@inflect:poss3sg Y}`. This works if `@inflect` is flexible enough, but the suffix chains can get long:

"from your cards" = "kartlarından" = kart + pl + poss2sg + abl (stem + plural + your + from)

This requires `@inflect:abl.poss2sg.pl` -- a 3-suffix chain. The APPENDIX_STDLIB.md already shows this pattern works. But verify the Turkish `@inflect` implementation supports **all orderings** of these suffixes, since suffix order in Turkish is rigid:

**stem + plural + possessive + case** (always this order)

### Severity

**SIGNIFICANT**. If `@inflect` respects the correct suffix ordering and supports all possessive+case combinations, this works. But if suffix application order is left-to-right as specified in the context string (as stated in APPENDIX_STDLIB.md: "Suffixes are applied left-to-right as specified in the context"), translators must always write them in the correct Turkish morphological order. This is a potential source of translator error.

### Recommendation

**RFC-TR-2**: Document the required suffix ordering for Turkish in APPENDIX_STDLIB.md:

> **Turkish suffix ordering (mandatory):** plural → possessive → case. The context string `@inflect:abl.poss2sg.pl` must be written as `@inflect:pl.poss2sg.abl` because suffixes are applied left-to-right and Turkish requires plural before possessive before case. Example: kart + pl + poss2sg + abl = "kartlarından" (from your cards).

Consider adding a validation step that warns if Turkish suffix contexts are specified in an incorrect morphological order, or alternatively, document clearly that the order in the context string **is** the application order and translators must get it right.

---

## 4. Issue 4 (SIGNIFICANT): SOV Word Order and Verb-Final Position

### The Problem

Turkish is a strictly SOV (Subject-Object-Verb) language. The verb always comes at the end:

| English (SVO) | Turkish (SOV) |
|---------------|---------------|
| Draw 3 cards. | 3 kart cek. |
| Dissolve an enemy. | Bir dusmani erit. |
| Gain 3 energy. | 3 enerji kazan. |
| Discard a card, then draw a card. | Bir kart at, sonra bir kart cek. |

The migration plan (Section 2.3) correctly notes that "for languages that need to reorder trigger vs effect, the translation can restructure at the phrase level below." Each effect phrase in the Turkish translation file would define its own SOV word order:

```
// English source
draw_cards_effect($c) = "draw {cards($c)}.";

// Turkish translation
draw_cards_effect($c) = "{cards($c)} cek.";
```

This works for simple effects. However, the **top-level string concatenation** in `ability_serializer.rs` (Section 2.3, Level 2) assembles trigger + effect by concatenation:

```rust
format!("{trigger_text}{effect_text}")
```

For Turkish triggered abilities, the structure is:

- English: "When you play a card, draw 3 cards." (trigger, effect)
- Turkish: "Bir kart oynadiginda, 3 kart cek." (trigger, effect)

The order happens to be the same (subordinate clause + main clause), so concatenation works. **However**, for conditional abilities the structure differs:

- English: "If a character dissolved this turn, draw a card."
- Turkish: "Bu tur bir karakter eritmisse, bir kart cek."

This still works because Turkish subordinate clauses precede main clauses. The plan's concatenation approach is **acceptable for Turkish** because Turkish subordinate clauses (triggers, conditions) naturally precede the main clause (effect), matching the English concatenation order.

### Severity

**SIGNIFICANT but mitigated**. The concatenation order (prefix + trigger/condition + effect) happens to match Turkish clause order. The only risk is if future ability structures require post-verbal elements in Turkish, which is unlikely for card game text.

### Recommendation

No Rust code changes needed. Document that Turkish clause order (subordinate + main) matches the concatenation order used by `ability_serializer.rs`. Turkish translators should define each phrase with correct internal SOV order, and the top-level concatenation will produce correct Turkish.

---

## 5. Issue 5 (MINOR): No Gender -- `:match` on Gender Tags

### The Problem

Turkish has absolutely no grammatical gender. There is no distinction between "he", "she", and "it" -- Turkish uses "o" for all three. The `:match($target)` pattern used by Russian, Spanish, Portuguese, and German for gender agreement is completely irrelevant in Turkish:

```
// ru.rlf -- gender-matching (:match)
when_dissolved_trigger($target) = :match($target) {
    masc: "когда {$target:nom} растворён, ",
    fem: "когда {$target:nom} растворена, ",
    *neut: "когда {$target:nom} растворено, ",
};

// tr.rlf -- no gender matching needed
when_dissolved_trigger($target) = "{@inflect:nom $target} eritildiginde, ";
```

### Severity

**MINOR**. No action needed. Turkish translations simply don't use `:match` on gender tags. Turkish Phrase values will have no `:masc`/`:fem`/`:neut` tags, which is correct -- the system handles this gracefully because `*` default branches catch unmatched tags.

### Verification

Confirm that when a Turkish Phrase (no gender tags) is passed to a gendered-language phrase (e.g., during locale switching or testing), the `*` default branch is always selected. This should already work by design.

---

## 6. Issue 6 (MINOR): Vowel Harmony Beyond `@inflect`

### The Problem

The `@inflect` transform handles vowel harmony for suffix chains. But Turkish vowel harmony also affects:

1. **Loanword behavior**: Some loanwords don't follow standard vowel harmony (e.g., "saat" has back vowel "a" but takes front-vowel suffixes in some dialects)
2. **Compound nouns**: "hangiisi" vs "hangisi" -- buffer letter behavior in compounds
3. **Proper nouns and game terms**: Dreamtides-specific terms like card subtypes will be transliterated or translated. If transliterated (e.g., "Ancient" stays as-is or becomes "Antik"), the Turkish translator must assign the correct `:front`/`:back` tag based on the final vowel of the Turkish word.

### Severity

**MINOR**. The `:front`/`:back` tag system correctly puts vowel harmony classification in the translator's hands. Loanwords and game-specific terms can be tagged correctly by the translator. No framework changes needed.

### Recommendation

Document in the Turkish section of APPENDIX_STDLIB.md that translators must assign `:front`/`:back` tags based on the **last vowel** of the Turkish word, even for loanwords where the tag might seem counterintuitive. For example, "karakter" (character) has final vowel "e" (front), so it gets `:front` despite having back vowels earlier in the word.

---

## 7. Issue 7 (MINOR): `@cap` and Turkish Dotted/Dotless I

### The Problem

APPENDIX_STDLIB.md already notes that `@cap` is locale-sensitive and mentions the Turkish I specifically: "Turkish 'istanbul' uppercases to 'ISTANBUL' with a dotted capital I". Turkish has two distinct letters:

- "i" (dotted lowercase) ↔ "I" (dotted uppercase, U+0130)
- "ı" (dotless lowercase) ↔ "I" (dotless uppercase, standard ASCII I)

The `@cap` transform must use Turkish-specific case mapping when the locale is Turkish:
- `@cap` on "istanbul" must produce "Istanbul" (dotted capital I), not "Istanbul" (dotless)
- `@cap` on "ısık" must produce "Isık" (dotless capital I), not "Isık" (dotted)

### Severity

**MINOR** if `@cap` already uses ICU/Unicode locale-sensitive case mapping. **SIGNIFICANT** if it uses simple ASCII uppercasing.

### Recommendation

Add to Section 9.7 (RLF Feature Verification Checklist):

> - [ ] `@cap` uses locale-sensitive Unicode case mapping (Turkish dotted/dotless I distinction)

This is likely already handled if `@cap` delegates to Rust's `to_uppercase()` with locale awareness, but verify. If using simple ASCII case mapping, this is a bug that must be fixed for Turkish.

---

## 8. Turkish Case Study: "Draw 3 cards."

To validate the architecture, here is "Draw N cards" in Turkish:

```
// tr.rlf
kart = :back { one: "kart", other: "kart" };  // no plural change in bare form

cards_tr($n) = :match($n) {
    1: "bir kart",
    *other: "{$n} kart",
};

draw_cards_effect($c) = "{cards_tr($c)} cek.";
```

| n | English | Turkish | Notes |
|---|---------|---------|-------|
| 1 | "draw a card." | "bir kart cek." | SOV, "bir" = indefinite |
| 3 | "draw 3 cards." | "3 kart cek." | No plural suffix on object in Turkish |

Note: Turkish does NOT pluralize nouns after numerals. "3 kart" (3 card), not "3 kartlar" (3 cards). This is a key difference from European languages. The `:match` on `$n` handles this correctly by using the bare stem "kart" after a numeral.

---

## 9. Turkish Case Study: "Dissolve an enemy Ancient."

```
// tr.rlf
ancient = :back { one: "Kadim", other: "Kadimler" };
enemy_modifier = "dusman";
dissolve_tr = "<color=#AA00FF>erit</color>";

subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";

dissolve_target($target) = "bir {@inflect:acc $target} {dissolve_tr}.";
// with enemy_ancient passed as $target:
// → "bir dusman Kadimi erit."
```

| Language | Output | Key Features |
|----------|--------|--------------|
| EN | "Dissolve an enemy Ancient." | `@a` article, SVO |
| TR | "Bir dusman Kadimi erit." | accusative suffix on target, SOV, no article |

The accusative suffix "-i" on "Kadim" marks it as a definite direct object. "Bir" (one/a) makes it indefinite. This correctly uses `@inflect:acc` in place of the English `@a`.

---

## 10. Turkish Case Study: Possessive Chains

"gain energy equal to that character's cost":

```
// English
gain_energy_equal_to_that_cost_effect = "gain energy_symbol equal to that character's cost.";

// Turkish
gain_energy_equal_to_that_cost_effect =
    "o {@inflect:gen karakter} {@inflect:poss3sg maliyet} kadar {energy_symbol} kazan.";
// → "o karakterin maliyeti kadar [energy] kazan."
// (that character's cost as-much-as energy gain)
```

This demonstrates the genitive+possessive suffix chain ("karakterin maliyeti" = character-GEN cost-POSS3SG) working through `@inflect`.

---

## 11. RLF Framework Changes Required for Turkish

### 11.1 Buffer Consonant Insertion in `@inflect` -- HIGH (BLOCK-TR-1)

**Priority**: HIGH -- without this, ~50% of Turkish noun inflections are incorrect.

The Turkish `@inflect` implementation must handle buffer consonant insertion between vowel-ending stems and vowel-initial suffixes:

| Context | Buffer | Example |
|---------|--------|---------|
| Vowel stem + acc/dat/gen/loc/abl | y | elma + acc → elmayı |
| Vowel stem + 3sg possessive | s | araba + poss3sg → arabası |
| Possessive-ending + case | n | arabası + acc → arabasını |

Implementation: ~40 lines in the Turkish suffix chain handler. Before appending each suffix, check if stem ends in vowel and suffix starts with vowel, then insert the appropriate buffer consonant based on the suffix type.

### 11.2 `@inflect` Accepts Phrase Parameters -- MEDIUM (RFC-TR-1)

Verify that `@inflect` works on `$target` parameters (Phrase values from predicate serializer), not just static term references. Turkish translators need `{@inflect:acc $target}` where `$target` is a Phrase. This is the Turkish equivalent of `{@a $target}` in English.

### 11.3 Suffix Order Validation or Documentation -- LOW (RFC-TR-2)

Either:
- (a) Add validation in `@inflect` that warns when Turkish suffixes are in incorrect morphological order (plural must precede possessive, possessive must precede case), or
- (b) Document the required ordering prominently in APPENDIX_STDLIB.md so translators don't make ordering mistakes.

Option (b) is simpler and sufficient for a small translation team.

### 11.4 `@cap` Locale-Sensitive Case Mapping -- LOW

Verify `@cap` produces correct Turkish dotted/dotless I behavior. If Rust's `to_uppercase()` is locale-unaware, add Turkish-specific handling.

### 11.5 NOT Recommended for Turkish

The following were evaluated and rejected:

- **Turkish-specific article transform**: Turkish has no articles. The word "bir" (one/a) is a numeral, not an article. Translators can include "bir" directly in phrase templates.
- **Automatic plural suppression after numerals**: Translators handle this in `:match` by using the bare stem after numbers. No framework feature needed.
- **Suffix-aware `@count`**: Turkish doesn't use classifiers/measure words. Numbers are simply placed before the noun. `@count` is not needed.
- **Honorific/politeness transforms**: Turkish has formal/informal "you" (siz/sen), but card game text consistently uses 2sg informal, so no switching needed.

---

## 12. Cross-Language Observations

### 12.1 Turkish + Japanese Parallel (SOV Order)

Both Turkish and Japanese are SOV languages. The migration plan's concatenation approach (trigger + effect) works for both because subordinate clauses precede main clauses in both languages. The Japanese agent should verify the same conclusion.

### 12.2 Turkish + Finnish/Hungarian Parallel (Agglutinative)

All three languages share the `@inflect` transform. The buffer consonant issue (BLOCK-TR-1) is Turkish-specific -- Finnish and Hungarian have different morphophonological rules. Verify that the `@inflect` infrastructure supports language-specific morphophonological adjustments (buffer consonants for Turkish, consonant gradation for Finnish, etc.) within the shared suffix-chain framework.

### 12.3 Turkish + Arabic Parallel (No Gender vs. Rich Gender)

Turkish has no gender; Arabic has rich gender agreement. Both work correctly under the plan because gender tags are language-specific: Arabic defines `:masc`/`:fem` on its terms, Turkish defines `:front`/`:back`. The tag system's language-independence is a strength.

### 12.4 Convention 5 Alignment

Section 9.9 Convention 5 says "text_number is English-specific." This is correct for Turkish too. Turkish number words don't inflect for gender, so plain numerals or "bir" (one) work directly in phrase templates. Turkish translators should not call `text_number`.

---

## 13. Verdict

The migration plan is **sound for Turkish** with one critical fix required:

| ID | Severity | Summary | Action |
|----|----------|---------|--------|
| BLOCK-TR-1 | CRITICAL | Buffer consonant insertion in `@inflect` | Add to Task 11, Section 9.8.8 |
| RFC-TR-1 | SIGNIFICANT | `@inflect` must accept Phrase parameters | Verify in Task 11 |
| RFC-TR-2 | SIGNIFICANT | Document Turkish suffix ordering | Add to APPENDIX_STDLIB.md |
| RFC-TR-3 | SIGNIFICANT | SOV word order -- verify concatenation compatibility | Document, no code change |
| MINOR-TR-1 | MINOR | No gender -- `:match` defaults work correctly | Verify `*` defaults |
| MINOR-TR-2 | MINOR | Vowel harmony for loanwords -- translator responsibility | Document |
| MINOR-TR-3 | MINOR | `@cap` locale-sensitive I handling | Verify implementation |

The `@inflect` transform is the cornerstone of Turkish support. With buffer consonant handling (BLOCK-TR-1) and Phrase parameter support (RFC-TR-1), the architecture can express all Turkish card text patterns. The SOV word order, accusative-as-definiteness, possessive suffix chains, and vowel harmony are all well-served by the existing RLF primitives.
