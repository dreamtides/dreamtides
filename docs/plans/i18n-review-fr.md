# French (Français) — i18n Review of Serializer RLF Migration Plan

**Reviewer:** fr-agent
**Date:** 2026-02-08
**Scope:** Phase 2 migration plan (`serializer_rlf_migration.md`) reviewed for French localization readiness.

---

## 1. Executive Summary

French is well-supported by the RLF architecture. The framework already provides `@le`, `@un`, `@de`, `@au`, and `@liaison` transforms. Gender agreement works via `:match` on `:masc`/`:fem` tags. The single biggest French-specific concern is **elision** — the mandatory contraction of articles and prepositions before vowel sounds (le+ennemi=l'ennemi, de+énergie=d'énergie). RLF handles this via the `:vowel` tag on terms, which `@le` and `@de` already read. This tag-based approach is correct and sufficient for Dreamtides.

**Verdict:** The migration plan is **APPROVED** for French with minor recommendations. No blocking issues. Two low-priority RLF framework additions recommended.

---

## 2. French Grammatical Features Summary

| Feature | Complexity | RLF Mechanism |
|---------|------------|---------------|
| Gender (masc/fem) | Two genders | `:masc`/`:fem` tags, `:match` |
| Definite articles (le/la/l'/les) | Elision before vowels | `@le` reads `:masc`/`:fem`/`:vowel` |
| Indefinite articles (un/une) | Gender agreement only | `@un` reads `:masc`/`:fem` |
| Partitive articles (du/de la/de l'/des) | Contraction+elision | `@de` reads `:masc`/`:fem`/`:vowel` |
| Preposition à + article (au/à la/à l'/aux) | Contraction+elision | `@au` reads `:masc`/`:fem`/`:vowel` |
| Adjective agreement | Masc/fem + plural | `:match` or inline variants |
| Adjective placement | Usually post-nominal | Phrase templates |
| Past participle agreement | Gender + number | `:match` on entity tags |
| Ne...pas negation | Two-part | Phrase templates |
| Liaison (prevocalic adj forms) | Limited set | `@liaison` transform |
| Elision (l', d', qu', n', etc.) | Mandatory before vowels | `:vowel` tag system |
| Plural (one/other) | Simple | CLDR `one`/`other` |

---

## 3. Elision Analysis — The Core French Challenge

### 3.1 What Is Elision?

Elision is the mandatory replacement of a final vowel with an apostrophe before a word starting with a vowel sound. It is **not optional** — it is a spelling error to write "le ennemi" instead of "l'ennemi".

Affected forms:
| Base form | Elided form | Example |
|-----------|-------------|---------|
| le | l' | l'ennemi (the enemy) |
| la | l' | l'épée (the sword) |
| de | d' | d'énergie (of energy) |
| que | qu' | qu'il (that he) |
| ne | n' | n'a pas (does not have) |
| je | j' | j'ai (I have) |
| se | s' | s'en (of it) |
| ce | c' | c'est (it is) |

For Dreamtides card text, the most relevant are: **le→l'**, **la→l'**, **de→d'** (including partitive "du" → "de l'" before vowels), and **à** contractions.

### 3.2 How RLF Handles Elision

RLF uses a **tag-based** approach: terms starting with a vowel sound carry the `:vowel` tag, and transforms like `@le` read this tag to decide whether to elide.

From APPENDIX_STDLIB.md:
```
@le reads :masc, :fem, :vowel → produces le/la/l'/les
@de reads :masc, :fem, :vowel → produces du/de la/de l'/des
@au reads :masc, :fem, :vowel → produces au/à la/à l'/aux
```

Example from the French section:
```
enemy = :masc :vowel "ennemi";
the_enemy = "{@le enemy}";    // → "l'ennemi" (elision)
```

### 3.3 Why Tag-Based Elision Works for Dreamtides

The tag-based approach works because:

1. **All card game nouns are known at definition time.** Every entity (ally, enemy, card, event, subtype names) is defined as an RLF term. The translator annotates each with `:vowel` if it starts with a vowel sound.

2. **`:from` propagates `:vowel`.** When `subtype(ancient)` inherits tags via `:from`, it gets `:vowel` if the French translation of "ancient" starts with a vowel. So `{@le subtype($s)}` correctly produces "l'" or "le" depending on the subtype.

3. **No truly dynamic text.** Card text doesn't interpolate free-form user input. All inserted fragments are RLF phrases with known tags.

4. **English `@a` uses the same pattern.** The English `@a` reads `:a`/`:an` tags rather than inspecting phonology. French `@le` reads `:vowel` in the same way. The architecture is consistent.

### 3.4 Elision with Dynamic Targets — The Critical Test

The critical question: when a phrase like `dissolve_target($target)` receives a dynamic `Phrase` parameter, does `@le` correctly read the `:vowel` tag from the parameter?

**Yes.** RLF transforms read tags from their operand. When `$target` holds a `Phrase` with `:vowel` (e.g., from the French definition of "ally" = `:masc :vowel "allié"`), then `{@le $target}` produces "l'allié". When `$target` does NOT have `:vowel` (e.g., "carte"), it produces "la carte".

This is exactly how English `{@a $target}` works — it reads `:a`/`:an` from the `Phrase` parameter. No new mechanism needed.

### 3.5 Elision Edge Case: Compound Phrases

When a compound phrase wraps an entity, `:from` MUST be used to propagate `:vowel`:

```
// CORRECT: :from propagates :vowel
allied($entity) = :from($entity) "allié {$entity:one}";
// Result: @le applied to allied(ennemi) → "l'allié ennemi" (if ennemi has :vowel)

// WRONG: no :from, :vowel lost
allied($entity) = "allié {$entity:one}";
// Result: @le can't see :vowel → always "le allié ennemi" (SPELLING ERROR)
```

**This is already addressed by Task 10** (Section 9.6 of the migration plan), which requires `:from($entity)` on all compound predicate phrases. This is CRITICAL for French — without it, elision breaks on every compound modifier.

### 3.6 Elision Verdict

**No framework changes needed for elision.** The existing tag-based system (`@le` + `:vowel` + `:from` propagation) handles all French elision requirements. The translator must annotate each term with `:vowel` when it starts with a vowel sound. This is the same pattern as English `:a`/`:an` annotations — familiar and proven.

---

## 4. Contractions (de+article, à+article)

French has mandatory contractions of prepositions with definite articles:

| Preposition + Article | Result | RLF Transform |
|----------------------|--------|---------------|
| de + le | du | `@de` |
| de + la | de la | `@de` |
| de + l' | de l' | `@de` (with `:vowel`) |
| de + les | des | `@de` (with `:other` context) |
| à + le | au | `@au` |
| à + la | à la | `@au` |
| à + l' | à l' | `@au` (with `:vowel`) |
| à + les | aux | `@au` (with `:other` context) |

These are already documented in APPENDIX_STDLIB.md. The `@de` and `@au` transforms read `:masc`/`:fem`/`:vowel` tags and handle all cases.

**Example card text translations:**

```
// fr.rlf
void = :masc "vide";
hand = :fem "main";
deck = :masc "deck";

// "from your void" → "de votre vide" (no contraction with possessives)
// "from the void" → "du vide" → {@de void}
// "to the hand" → "à la main" → {@au hand}
```

**Important note:** French contracts with the *definite article* (le/la/les), not with possessives (votre/ton) or other determiners. Many card text patterns use "your" ("votre"), which does NOT trigger contraction. The translator chooses the appropriate construction.

**Verdict:** Fully supported by existing transforms. No changes needed.

---

## 5. Gender Agreement

### 5.1 Articles

French has two genders affecting articles:
- Masculine: le/un → Handled by `:masc` tag
- Feminine: la/une → Handled by `:fem` tag

### 5.2 Adjective Agreement

French adjectives agree in gender and number:
- "un allié dissous" (masc sg) → "une carte dissoute" (fem sg)
- "des alliés dissous" (masc pl) → "des cartes dissoutes" (fem pl)

Handled by `:match` on the entity's gender tag:

```
// fr.rlf
dissolved_adj($entity) = :match($entity) {
    masc: { one: "dissous", other: "dissous" },
    *fem: { one: "dissoute", other: "dissoutes" },
};
```

### 5.3 Past Participle Agreement

Same pattern as adjectives. "dissolvé/dissoute" agrees with the subject/object:

```
// fr.rlf
when_dissolved_trigger($target) = :match($target) {
    masc: "quand {$target} est dissous, ",
    *fem: "quand {$target} est dissoute, ",
};
```

**Rust code implication:** The predicate serializer MUST return `Phrase` values with gender tags (from the French translation). This is exactly what Task 2 delivers — predicates return `Phrase`, and the French translation defines `ally = :masc :vowel "allié"`, `card = :fem "carte"`, etc.

**Verdict:** Fully supported. No changes needed beyond what Tasks 2 and 10 already provide.

---

## 6. Partitive Articles

French uses partitive articles for uncountable quantities:
- "de l'énergie" (some energy) — uses `@de` + `:fem :vowel` on "énergie"
- "du courage" (some courage) — uses `@de` + `:masc`
- "de la force" (some strength) — uses `@de` + `:fem`

For Dreamtides, the main partitive usage is for **energy** ("de l'énergie"). Since energy is always displayed with a symbol (e.g., "2[energy]"), the partitive article mainly appears in phrases like "gain d'énergie" (energy gain).

```
// fr.rlf
energy_concept = :fem :vowel "énergie";
gain_energy_effect($e) = "gagnez {@de energy_concept}{energy($e)}.";
// → "gagnez de l'énergie 2●."
```

**Verdict:** Handled by existing `@de` transform. No changes needed.

---

## 7. Negation (Ne...pas)

French negation wraps the verb: "ne dissolvez pas" (don't dissolve). This is a phrase-level concern:

```
// fr.rlf
cannot_be_dissolved = "ne peut pas être dissous";
```

For Dreamtides card text, negation appears in:
- "Cannot be dissolved" → static ability text
- "Does not trigger" → edge case text
- Help text for prevent: "ne peut pas être ciblé"

These are handled entirely in translation file phrases. The two-part negation is simply written into the template. No RLF mechanism needed.

**Verdict:** Handled by phrase templates. No changes needed.

---

## 8. Adjective Placement

French adjectives usually follow the noun ("carte ennemie"), but a small set precedes it ("bonne carte", "petit allié"). This mirrors Spanish (also post-nominal default).

For Dreamtides:
- "enemy card" → "carte ennemie" (post-nominal)
- "allied character" → "personnage allié" (post-nominal)
- Subtypes like "Ancient" → placement chosen by translator

Handled entirely by phrase templates. The translator writes the correct word order.

**Verdict:** Handled by phrase templates. No changes needed.

---

## 9. Liaison (Prevocalic Adjective Forms)

French has a small set of adjectives with special prevocalic forms:
- ce/cet: "ce livre" but "cet ami"
- beau/bel: "beau livre" but "bel ami"
- nouveau/nouvel: "nouveau livre" but "nouvel ami"
- vieux/vieil: "vieux livre" but "vieil ami"

RLF provides the `@liaison` transform for this:

```
// fr.rlf
ce = { standard: "ce", vowel: "cet" };
this_thing($thing) = "{@liaison ce $thing} {$thing}";
// ami (has :vowel) → "cet ami"
// livre (no :vowel) → "ce livre"
```

For Dreamtides, this is unlikely to be needed frequently (card text rarely uses demonstratives like "this/that" with variable targets). But the mechanism exists if needed.

**Verdict:** Fully supported. No changes needed.

---

## 10. Phrase-by-Phrase French Translation Feasibility

### 10.1 Triggers

| English | French | Mechanism |
|---------|--------|-----------|
| "when you play {target}," | "quand vous jouez {target}," | Direct template |
| "when {target} is dissolved," | "quand {target} est dissous(e)," | `:match` for gender |
| "when you materialize {target}," | "quand vous matérialisez {target}," | Direct template |
| "at the end of your turn," | "à la fin de votre tour," | Direct template |

All work with current architecture.

### 10.2 Effects

| English | French | Mechanism |
|---------|--------|-----------|
| "draw {cards($c)}." | "piochez {cards($c)}." | Redefine `cards` for French |
| "Dissolve {target}." | "Dissolvez {@le $target}." | `@le` with gender+elision |
| "gain {energy($e)}." | "gagnez {energy($e)}." | Direct template |
| "Banish {target}." | "Bannissez {@le $target}." | `@le` with gender+elision |

All work with current architecture.

### 10.3 Costs

| English | French | Mechanism |
|---------|--------|-----------|
| "discard {cards($d)}" | "défaussez {cards($d)}" | Redefine `cards` for French |
| "{energy($e)}" | "{energy($e)}" | Same (symbols are universal) |
| "abandon {target}" | "abandonnez {target}" | Direct template |

All work with current architecture.

### 10.4 Static Abilities

| English | French | Mechanism |
|---------|--------|-----------|
| "Cannot be dissolved" | "Ne peut pas être dissous(e)" | `:match` for gender |
| "Gains +1 spark" | "Gagne +1 étincelle" | Direct template |

All work with current architecture.

---

## 11. Review of Cross-Language Conventions (Section 9.9)

### 11.1 Convention 1: No trailing periods in effect phrases

**French impact:** Supported. French uses the same period convention. Full-width period is not an issue (unlike Chinese).

### 11.2 Convention 2: Each phrase controls its own capitalization via @cap

**French impact:** Fully compatible. French capitalizes sentence-initial words just like English. `@cap` works correctly on French text (accented characters like "é" → "É").

### 11.3 Convention 3: Separate imperative and participial keyword forms

**French impact:** CRITICAL. French verbs have distinct imperative and participial forms:
- Dissolve (imperative): "Dissolvez" (2nd person plural imperative)
- Dissolved (participle): "dissous" (masc) / "dissoute" (fem)
- Materialize (imperative): "Matérialisez"
- Materialized (participle): "matérialisé(e)"

Furthermore, past participles in French agree in gender, requiring `:match`:

```
// fr.rlf
dissolved = :match($target) {
    masc: "<color=#AA00FF>dissous</color>",
    *fem: "<color=#AA00FF>dissoute</color>",
};
```

This is already addressed by Convention 3. The English source having separate `dissolve` and `dissolved` terms is sufficient — the French translation redefines them with gender agreement.

### 11.4 Convention 4: Structural connectors as named phrases

**French impact:** Important. French uses:
- then_joiner: ", puis " (not ", then ")
- and_joiner: " et " (not " and ")
- cost_effect_separator: " : " (same)
- period_suffix: "." (same)

All handled by overriding the structural phrases in the French translation file.

### 11.5 Convention 5: text_number is English-specific

**French impact:** Correct. French has no gender agreement on most number words (unlike Portuguese/Spanish "um/uma"). "Un/une" (one) does vary by gender, but this is handled via `:match` in the French translation.

---

## 12. Review of Proposed RLF Framework Changes (Section 9.8)

### 12.1 Portuguese @para and @por

**French relevance:** None. French uses `@de` and `@au` which already exist.

### 12.2 Chinese @count:word

**French relevance:** None.

### 12.3 German @ein empty plural

**French relevance:** None. French `@un` is always singular (no plural indefinite article in standard card text).

### 12.4 Spanish @del/@al

**French relevance:** Low. French has analogous contractions (de+le=du, à+le=au) but these are already handled by `@de` and `@au`.

---

## 13. RLF Framework Changes — French-Specific Recommendations

### 13.1 RFC-FR-1: `@de` Bare Preposition (Without Article) — LOW PRIORITY

**Current state:** `@de` always produces a contraction with the definite article (du/de la/de l'/des).

**Need:** Some card text uses "de" as a bare preposition without a definite article, and it still needs elision: "de l'énergie" (of energy, partitive) vs "du vide" (of the void, contracted). The `@de` transform handles the contracted forms, but the bare "de" + elision case ("d'") is also needed.

**Proposed solution:** This can be handled by defining a helper phrase in the translation file:

```
// fr.rlf
de_bare($x) = :match($x) {
    vowel: "d'{$x}",
    *other: "de {$x}",
};
```

No framework change needed. The translator writes this helper.

**Status:** No action required.

### 13.2 RFC-FR-2: Verify `@le`/`@de`/`@au` with `:other` Context for Plurals — VERIFICATION

**Current state:** APPENDIX_STDLIB.md shows `@le` producing "les" for plurals, but the context mechanism for plural (`@le:other`) is not explicitly documented for French the way it is for Spanish (`@el:other`).

**Need:** Verify that `@le:other`, `@de:other`, `@au:other` correctly produce:
- `@le:other` → "les"
- `@de:other` → "des"
- `@au:other` → "aux"

**Proposed action:** Add test cases to verify plural forms for all three French article transforms.

**Status:** VERIFICATION — add to Section 9.7 checklist.

### 13.3 RFC-FR-3: `@ne` Negation Transform — NOT RECOMMENDED

French "ne...pas" negation wraps the verb, which cannot be expressed as a single transform on a noun phrase. It's inherently a phrase-level pattern. Template text handles it perfectly:

```
// fr.rlf
cannot_be_dissolved($target) = :match($target) {
    masc: "ne peut pas être dissous",
    *fem: "ne peut pas être dissoute",
};
```

**Status:** Not recommended. Phrase templates are sufficient.

### 13.4 RFC-FR-4: `@un` Plural Behavior — LOW PRIORITY

French has no standard plural indefinite article in the traditional sense ("des" is technically the plural of "un/une" but is actually the plural partitive/indefinite). Some card text may need "des" for "some/any":
- "des alliés" (some allies)

**Proposed:** Define `@un:other` → "des" for both genders (French plural indefinite is gender-neutral).

**Comparison:** Portuguese already requested `@um:other` for plural indefinite (Section 9.8.3). French has the same need. If `@um:other` is implemented for Portuguese, `@un:other` → "des" for French follows the same pattern.

**Status:** LOW priority. Workaround: use inline "des" in templates.

---

## 14. Additions to Section 9.7 Verification Checklist

Add the following French-specific verification items:

- [ ] `@le` reads `:vowel` tag and produces "l'" (elision)
- [ ] `@le:other` produces "les" (plural definite)
- [ ] `@de` reads `:vowel` tag and produces "de l'" (partitive elision)
- [ ] `@de:other` produces "des" (plural)
- [ ] `@au` reads `:vowel` tag and produces "à l'" (elision)
- [ ] `@au:other` produces "aux" (plural)
- [ ] `@un` reads `:masc`/`:fem` and produces "un"/"une"
- [ ] `@liaison` correctly selects prevocalic variant from first arg based on `:vowel` of second arg
- [ ] `:from` propagates `:vowel` tag through compound phrases
- [ ] `@cap` correctly capitalizes accented characters (é→É, à→À)
- [ ] `:match` on `:masc`/`:fem` works for past participle agreement

---

## 15. Example: Full French Translation of "Dissolve an enemy Ancient."

### English source (strings.rs):
```
ancient = :an { one: "Ancient", other: "Ancients" };
subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";
dissolve_target_effect($target) = "{@cap dissolve} {@a $target}.";
```

### French translation (fr.rlf):
```
// Subtypes carry :vowel where applicable
ancient = :masc { one: "Ancien", other: "Anciens" };
// Note: "Ancien" starts with a vowel → needs :vowel
ancient = :masc :vowel { one: "Ancien", other: "Anciens" };

// subtype inherits tags via :from (including :vowel)
subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";

// Dissolve keyword (imperative)
dissolve = "<color=#AA00FF>dissolvez</color>";

// Effect phrase — uses @le which reads :masc/:fem/:vowel
dissolve_target_effect($target) = "{@cap dissolve} {@le $target}.";
```

### Resolution chain:
1. `$target` = `enemy_subtype(ancient)` → French Phrase with `:masc :vowel` (via `:from`)
2. `{@le $target}` → reads `:vowel` → produces "l'<color=#2E7D32><b>Ancien</b></color> ennemi"
3. `{@cap dissolve}` → "Dissolvez"
4. Final: "Dissolvez l'<color=#2E7D32><b>Ancien</b></color> ennemi."

This demonstrates that **elision works correctly through the composition chain** — `:from` propagates `:vowel` from the inner subtype through compound phrases, and `@le` reads it at the point of use.

---

## 16. Comparison: French Elision vs English @a

| Aspect | English @a | French @le |
|--------|-----------|------------|
| Tag source | `:a` or `:an` on term | `:vowel` on term |
| Decision | "a" vs "an" | "le/la" vs "l'" |
| Propagation | via `:from` | via `:from` |
| Dynamic targets | reads tag from Phrase param | reads tag from Phrase param |
| Fallback | error if no tag | no elision (safe default) |

The mechanisms are identical. French elision is NOT harder than English article selection — it's the same architectural pattern with different tags.

---

## 17. Potential Issues and Mitigations

### 17.1 HTML Tags and Elision

When `@le` is applied to a phrase starting with `<color=...>` (like keyword formatting), the transform must inspect the first **visible** character after HTML tags, not the literal first character.

**Mitigation:** `@cap` already skips HTML tags (documented in APPENDIX_STDLIB.md). Verify that `@le` uses the same HTML-skipping logic for `:vowel` inspection. If `@le` relies solely on the `:vowel` tag (not text inspection), this is a non-issue — the tag is pre-annotated.

**Verdict:** Non-issue. `@le` reads the `:vowel` **tag**, not the rendered text. Tags are pre-annotated by the translator.

### 17.2 Compound Modifiers and Elision

"allied Ancient" in French might be "Ancien allié" (adjective order flipped). The French translator has full control over word order in the phrase template:

```
// French word order: adjective placement is translator's choice
allied_subtype($t) = :from($t) :match($t) {
    vowel: "allié {$t:one}",     // only if :vowel matters for the compound
    *other: "allié {$t:one}",    // default
};
```

In practice, elision at the compound modifier level is rare in Dreamtides card text. The more common case is elision at the article level (`@le`), which works as described in Section 3.

### 17.3 Missing `:vowel` Tag — Silent Failure

If a translator forgets the `:vowel` tag on a vowel-initial term (e.g., defining `ennemi = :masc "ennemi"` without `:vowel`), `@le` will produce "le ennemi" instead of "l'ennemi". This is a **spelling error** but not a crash.

**Mitigation:** This mirrors the English `:a`/`:an` situation. A translation validator tool could flag missing `:vowel` tags by inspecting the first character of each term's default variant. This is a tooling concern, not an architecture concern.

---

## 18. Summary of Findings

### No Blocking Issues

The French language is fully supported by the current RLF architecture and migration plan.

### Key Architectural Dependencies (Already Planned)

1. **Task 2 (Predicate → Phrase):** French requires `Phrase` values with tags for gender agreement and elision. Already planned.
2. **Task 10 (`:from` on compound phrases):** French requires `:vowel` tag propagation through compound phrases. Already planned.
3. **Convention 3 (imperative/participial forms):** French requires separate verb forms with gender-agreeing participles. Already addressed.

### Verification Items to Add (Section 9.7)

See Section 14 above — 11 verification items for French-specific RLF behavior.

### Low-Priority RLF Changes

1. **RFC-FR-4:** `@un:other` → "des" (plural indefinite). Same pattern as Portuguese `@um:other`. LOW priority.

### No Framework Changes Required

Unlike Portuguese (which needs `@para`, `@por`) and Spanish (which needs `@del`, `@al`), French requires **no new transforms**. The existing `@le`, `@un`, `@de`, `@au`, and `@liaison` transforms cover all French grammatical needs for Dreamtides card text.

---

## 19. Recommendations

1. **Proceed with the migration plan as-is.** No French-specific blocking issues.
2. **Prioritize Task 10** (`:from` on compound phrases) — this is critical for French elision to work through the composition chain.
3. **Add French verification items** to Section 9.7 checklist.
4. **Consider `@un:other` → "des"** when implementing Portuguese `@um:other` — same pattern, minimal effort.
5. **When writing French translations:** Always annotate vowel-initial terms with `:vowel`. Create a translation validation tool to catch missing `:vowel` tags.
