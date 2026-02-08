# German (Deutsch) i18n Review of Serializer RLF Migration Plan

**Reviewer**: German language specialist
**Date**: 2026-02-08
**Scope**: Phase 2 migration plan at `docs/plans/serializer_rlf_migration.md`

---

## Executive Summary

The migration plan is **well-designed for German**. RLF's `@der`/`@ein` transforms with case context, `:from` metadata inheritance, and multi-dimensional variants provide the tools needed for German's three genders, four cases, separable verbs, and adjective declension. Most German challenges are solvable entirely in translation files with zero Rust code changes.

**Critical issues found: 1**
**Significant issues found: 2**
**Minor concerns: 4**

---

## Issue 1 (CRITICAL): Top-Level String Concatenation Breaks German Verb-Second Word Order

### The Problem

German main clauses require the finite verb in **second position** (V2 rule). When a subordinate clause (trigger) comes first, it occupies position 1, and the verb of the main clause (effect) must come immediately after:

- English: "When you materialize an ally, draw 3 cards."
  - trigger: "When you materialize an ally, " + effect: "draw 3 cards."
- German: "Wenn du einen Verbündeten materialisierst, **ziehe** 3 Karten."
  - The verb "ziehe" (draw) is in second position after the subordinate clause

This works fine because the effect phrase can define itself starting with the verb. **However**, when there are prefixes like "Once per turn" or "Until end of turn", the situation becomes more complex:

- English: "Once per turn, when you materialize an ally, draw 3 cards."
- German: "Einmal pro Zug, wenn du einen Verbündeten materialisierst, ziehe 3 Karten."

This still works because "Einmal pro Zug" is a parenthetical that doesn't count for V2.

**Where it truly breaks** is the `ability_serializer.rs` pattern at lines 29-61 for triggered abilities:

```rust
// Current ability_serializer.rs lines 36-44:
if has_until_end_of_turn {
    result.push_str("Until end of turn, ");
}
if has_once_per_turn {
    result.push_str("Once per turn, ");
}
result.push_str(if has_prefix { &trigger } else { &capitalized_trigger });
```

The Rust code decides the **ordering** of prefix + trigger + effect via string concatenation. For German, this ordering is actually acceptable for triggered abilities because the prefixes are parenthetical. But the **capitalization logic** in Rust (`capitalize_first_letter`) is language-dependent:

- English capitalizes "When" at the start, or the trigger keyword
- German needs to capitalize the first word of whatever comes first, which varies

### Severity

**CRITICAL** but **conditionally mitigated**. The plan already proposes replacing `capitalize_first_letter` with `@cap` in RLF phrases (Task 9). If each assembled piece is an RLF phrase that handles its own capitalization internally, the concatenation order doesn't affect German. The V2 rule is satisfied because each German effect phrase naturally starts with the verb.

### Recommendation

The plan at Section 2.3 says "for languages that need to reorder trigger vs effect, the translation can restructure at the phrase level below." This is correct — but only if **each phrase controls its own capitalization**. Verify that after Task 7, no Rust code applies `@cap` or capitalization to already-rendered strings. The Rust code must concatenate pre-capitalized pieces, not capitalize after assembly. The current plan's Task 7 Step 2 mentions using `@cap` in phrase templates — this is the correct approach. Mark this as a hard requirement, not just a nice-to-have.

---

## Issue 2 (SIGNIFICANT): Adjective Declension in Predicate Composition

### The Problem

German adjectives decline based on three factors:
1. **Gender** of the noun (masc/fem/neut)
2. **Case** (nom/acc/dat/gen)
3. **Article type** preceding the adjective (strong/weak/mixed declension)
   - After definite article → weak: "der feindlich**e** Charakter"
   - After indefinite article → mixed: "ein feindlich**er** Charakter"
   - Without article → strong: "feindlich**er** Charakter"

The migration plan's predicate composition pattern (Section 2.5) returns `Phrase` objects from predicates, and consuming phrases use `{@a $target}` or `{@ein:acc $target}` to add articles. But the **adjective form inside the Phrase must change depending on which article is applied at the call site**.

Consider "enemy Ancient" → German:
- With `@ein:acc`: "einen feindlich**en** Uralten" (mixed declension, acc masc)
- With `@der:acc`: "den feindlich**en** Uralten" (weak declension, acc masc — same here, but differs in nom)
- With `@der:nom`: "der feindlich**e** Uralte" (weak declension, nom masc)
- Without article: "feindlich**er** Uralter" (strong declension, nom masc)

The predicate phrase `enemy_subtype($t)` would need to return different adjective forms based on the article context at the call site.

### Impact on Rust Code

This is solvable in RLF without Rust changes, but requires careful phrase design in the German translation file:

```
// de.rlf
// Approach: Use multi-dimensional variants for article-context + case
enemy_subtype($t) = :from($t) {
    def.nom: "feindliche {$t:nom}",      // weak after definite
    def.acc: "feindlichen {$t:acc}",
    indef.nom.masc: "feindlicher {$t:nom}",  // mixed after indefinite
    indef.nom.fem: "feindliche {$t:nom}",
    indef.acc.masc: "feindlichen {$t:acc}",
    strong.nom.masc: "feindlicher {$t:nom}",  // strong without article
    ...
};
```

### Severity

**SIGNIFICANT** — but solvable without Rust changes. The key requirement is that RLF's `:from` supports multi-dimensional variants on composed phrases, and that `@der`/`@ein` can communicate the article context to the noun phrase they modify.

### Recommendation

Add to the RLF Feature Verification Checklist (Section 9.7):
- [ ] `@der`/`@ein` transforms communicate declension context (weak/mixed/strong) to the phrase they modify, OR the German translation uses separate phrase variants for each article context
- [ ] `:from` correctly propagates multi-dimensional variants when the source phrase itself has multi-dimensional variants (e.g., `enemy_subtype` with case × declension-type, applied to `ancient` with case × number)

---

## Issue 3 (SIGNIFICANT): Separable Verb Prefix Placement

### The Problem

German separable verbs split in main clauses: "auflösen" (dissolve) → "Löse ... **auf**." The prefix goes to the end of the clause. This is handled in Section 9.5 of the plan:

```
// de.rlf
dissolve_target($target) = "Löse {@ein:acc $target} auf.";
```

This works for simple single-effect phrases. But what about **compound effects** joined by structural phrases?

- English: "dissolve an enemy, then draw 3 cards."
- German: "Löse einen Feind **auf**, dann ziehe 3 Karten."

The separable prefix "auf" must come **before** the "then" connector, not at the end of the full sentence. The ability serializer (Task 7) joins effects with `then_joiner` (", then ") via string concatenation:

```rust
// format!("{effect1}{then_joiner}{effect2}")
```

This works IF each effect phrase is self-contained with its own punctuation and separable verb prefix. Let's check:
- Effect 1: "Löse einen Feind auf" (needs "auf" before the period/comma)
- Then joiner: ", dann "
- Effect 2: "ziehe 3 Karten."

If the German `dissolve_target` phrase is defined as `"Löse {@ein:acc $target} auf."` (with period), then the concatenation becomes: "Löse einen Feind auf., dann ziehe 3 Karten." — double punctuation.

### Severity

**SIGNIFICANT** — This is a punctuation/structural issue, not a fundamental architectural one. The English code already handles this: effects are defined without trailing periods, and the period is added by the structural phrase at the assembly level. Looking at the current code, `dissolve_target_effect` in the plan (Section 5.1) includes the period: `"Dissolve {@a $target}."`. This means the plan needs to define effect phrases **both with and without** trailing punctuation, or have the structural joiner handle period removal.

### Recommendation

Define effect phrases **without trailing periods** for composability. Add the period via the structural phrase or the ability serializer:

```
// de.rlf
dissolve_target_effect($target) = "Löse {@ein:acc $target} auf";  // no period
// When used standalone, the ability serializer adds period_suffix
// When used in a list, then_joiner connects without double punctuation
```

This is consistent with what the plan already does for English (the `period_suffix` phrase exists). The plan should explicitly document this convention: effect phrases should NOT include trailing periods.

---

## Issue 4 (MINOR): Noun Capitalization

### The Problem

ALL German nouns are capitalized: "Karte" (card), "Charakter" (character), "Energie" (energy). The plan mentions `@cap` for sentence-initial capitalization, but German needs inherently capitalized nouns throughout.

### Impact

**None on Rust code.** German translation files simply define nouns with capital letters:

```
// de.rlf
karte = :fem { nom: "Karte", acc: "Karte", ... };
charakter = :masc { nom: "Charakter", acc: "Charakter", ... };
```

The `@cap` transform still works correctly for sentence-initial capitalization of non-nouns. No Rust changes needed.

---

## Issue 5 (MINOR): Gender Tags on English Source Terms

### The Problem

The plan (Section 9.6) says gender tags (`:masc`/`:fem`/`:neut`) come from translation files, not the English source. This is correct — English doesn't have grammatical gender. But `:from` propagates tags from the source term.

For German translations, when `subtype($s) = :from($s) "..."` is used, the German `subtype` inherits tags from the German `ancient` term (which carries `:masc`). This works correctly because `:from` evaluates against the current locale's definitions.

### Verification Needed

Confirm that `:from` in a translation file reads the **translation file's** version of the source term, not the English source. If `ancient` in English has `:an` tag, and `ancient` in German has `:masc :an` tags, then `subtype(ancient)` in the German context should inherit `:masc :an` from the German definition.

Per the RLF design doc: "All evaluation (source and translations) goes through the interpreter" and translations override source definitions. This should work correctly.

---

## Issue 6 (MINOR): `@ein` Has No Plural Form

### The Problem

German has no plural indefinite article. `@ein` is singular-only (documented in APPENDIX_STDLIB.md). For plural indefinite contexts, German uses no article (bare noun with strong adjective declension):

- Singular: "einen feindlichen Charakter" (with `@ein:acc`)
- Plural: "feindliche Charaktere" (no article, strong declension)

The English pattern `cards($n) = :match($n) { 1: "a card", *other: "{$n} cards" }` uses `@a` for singular. The German equivalent cannot use `@ein` for plural:

```
// de.rlf
karten($n) = :match($n) {
    1: "{@ein:acc karte}",
    *other: "{$n} {karte:acc:other}",  // no article for plural
};
```

### Impact

**None on Rust code.** This is handled naturally by `:match` in the German translation file. The Rust code passes the count; the German `:match` branch for `*other` simply omits the article.

---

## Issue 7 (MINOR): Verb-Final Order in Subordinate Clauses

### The Problem

German subordinate clauses (introduced by "wenn", "dass", "weil") push the verb to the **end**:

- "when you play a card" → "wenn du eine Karte **spielst**" (verb last)

Trigger phrases like `when_you_play_trigger($target)` define the entire subordinate clause in the translation:

```
// de.rlf
when_you_play_trigger($target) = "wenn du {@ein:acc $target} spielst, ";
```

This works perfectly because each trigger phrase controls its own word order. The Rust code just passes the target Phrase.

### Impact

**None on Rust code.** Fully handled in translation files.

---

## Positive Findings

### Things the Plan Gets Right for German

1. **`:from` for subtype composition** — Correctly preserves gender/case metadata through `subtype($s)`, essential for "einen feindlichen **Uralten**" where the subtype needs case declension.

2. **`@der`/`@ein` transforms with case context** — The design `{@ein:acc $target}` maps perfectly to German's article+case system. All 16 article forms (4 cases × {def/indef} × 3 genders) are covered.

3. **Predicate → Phrase return type** — Returning `Phrase` with tags/variants rather than `String` is essential. German needs the gender tag to select the correct article form, and case variants for declension. Pre-rendered strings would make German impossible.

4. **Separable verbs in translation files** — Section 9.5 correctly shows "Löse ... auf" handled entirely in the `.rlf` file. This works for single-effect phrases.

5. **`:match` for plurals** — German's simple `one`/`other` plural system maps cleanly to `:match($n)`.

6. **Animacy tags (`:anim`/`:inan`)** — While less critical for German than for Russian/Spanish, these don't cause problems and are forward-compatible.

7. **Multi-dimensional variants** — German nouns need case × number variants (8 combinations for 4 cases × 2 numbers). RLF's dot notation (`nom.one`, `acc.other`) handles this elegantly.

---

## German Translation Case Study: "Dissolve an enemy Ancient."

English: "Dissolve an enemy Ancient."
German: "Löse einen feindlichen Uralten auf."

### Breakdown

| Component | English | German | RLF Feature |
|-----------|---------|--------|-------------|
| Verb stem | "Dissolve" | "Löse" | Phrase template |
| Article | "an" | "einen" | `@ein:acc` reads `:masc` tag |
| Adjective | "enemy" | "feindlichen" | Variant selection by case+article context |
| Noun | "Ancient" | "Uralten" | `{$t:acc}` selects accusative form |
| Verb prefix | (none) | "auf" | Phrase template suffix |
| Period | "." | "." | Phrase template |

### Required RLF definitions

```
// de.rlf
uralter = :masc {
    nom: "Uralter", acc: "Uralten",
    nom.other: "Uralte", acc.other: "Uralte",
    dat: "Uraltem", gen: "Uralten",
};

subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";

dissolve_target_effect($target) = "Löse {@ein:acc $target} auf";
// With target = subtype(uralter):
// → "Löse einen <b>Uralten</b> auf"
// (But "feindlichen" adjective is missing — see Issue 2)
```

For the full "enemy Ancient" case, the predicate phrase must include the adjective:

```
// de.rlf
enemy_subtype($t) = :from($t) :match($t) {
    masc: "feindlichen {$t:acc}",      // acc masc after indef article
    fem: "feindliche {$t:acc}",
    *neut: "feindliches {$t:acc}",
};
// Applied: enemy_subtype(uralter) → "feindlichen Uralten"

dissolve_target_effect($target) = "Löse {@ein:acc $target} auf";
// With target = enemy_subtype(uralter):
// → "Löse einen feindlichen Uralten auf"
```

This works, though the adjective declension is hardcoded for the accusative+indefinite context. A more flexible approach would use variant dimensions, but that adds complexity that may not be needed if the Rust code always uses `@ein:acc` in this context.

---

## Summary of Recommendations

| # | Severity | Issue | Action Required |
|---|----------|-------|-----------------|
| 1 | CRITICAL | Capitalization applied in Rust after concatenation | Verify Task 7 removes ALL Rust-side capitalization; each phrase self-capitalizes |
| 2 | SIGNIFICANT | Adjective declension depends on article context | Add RLF feature verification for article-context communication |
| 3 | SIGNIFICANT | Separable verb + compound effects + punctuation | Document convention: effect phrases have no trailing period |
| 4 | MINOR | Noun capitalization | No action — handled in translation files |
| 5 | MINOR | Gender tag propagation via `:from` | Verify `:from` uses locale-specific definitions |
| 6 | MINOR | No plural indefinite article | No action — handled by `:match` |
| 7 | MINOR | Verb-final in subordinate clauses | No action — handled in translation files |

### Overall Assessment

The plan is **well-suited for German localization**. The critical issue (#1) is already addressed by the plan's design (Task 9 removes capitalization helpers), but should be explicitly marked as a hard requirement. The significant issues (#2, #3) are solvable in translation files without Rust code changes, but need documentation and RLF feature verification.

**No Rust code changes are required beyond what the plan already proposes** to support German. This confirms the plan achieves its goal of language-neutral Rust code.

---

## Appendix: RLF Framework Change Recommendations for German

Since RLF is an in-house framework, we can modify it to better support German. Below are concrete recommendations, ordered by impact.

### Recommendation 1: `@der`/`@ein` Should Set a Declension Context Tag on Their Argument

**Problem**: German adjective declension has three paradigms (strong/weak/mixed) determined by the preceding article. When `@ein:acc` is applied to a noun phrase containing an adjective, the adjective needs the mixed declension. When `@der:acc` is applied, it needs weak declension. When no article is applied, it needs strong declension. Currently, `@der`/`@ein` just prepend the article text — they don't communicate anything to the phrase they modify.

**Proposed change**: `@der` and `@ein` should add a **transient tag** to the Phrase they transform before rendering it. This tag tells the phrase what declension context applies:

```
// How it works internally:
// 1. @ein:acc reads :masc tag from $target → selects "einen"
// 2. @ein:acc adds transient tag :mixed to $target
// 3. $target is rendered — its adjective variants can read :mixed

// How translators use it:
// de.rlf
feindlich = {
    // weak declension (after @der)
    weak.nom.masc: "feindliche",   weak.nom.fem: "feindliche",   weak.nom.neut: "feindliche",
    weak.acc.masc: "feindlichen",  weak.acc.fem: "feindliche",   weak.acc.neut: "feindliche",
    // mixed declension (after @ein)
    mixed.nom.masc: "feindlicher", mixed.nom.fem: "feindliche",  mixed.nom.neut: "feindliches",
    mixed.acc.masc: "feindlichen", mixed.acc.fem: "feindliche",  mixed.acc.neut: "feindliches",
    // strong declension (no article)
    *strong.nom.masc: "feindlicher", strong.nom.fem: "feindliche", strong.nom.neut: "feindliches",
    strong.acc.masc: "feindlichen",  strong.acc.fem: "feindliche", strong.acc.neut: "feindliches",
};

enemy_subtype($t) = :from($t) "{feindlich:$DECL:$CASE:$t} {$t}";
```

**However**, this is complex and I'm not sure it's worth the framework change. A simpler alternative exists — see "Pragmatic Alternative" below.

**Pragmatic alternative — no framework change needed**: In Dreamtides, each predicate noun phrase is always used in a specific, predictable article context. The Rust code calls `dissolve_target_effect($target)` where the phrase template always applies `@ein:acc`. The German translator knows this context and can hardcode the adjective form:

```
// de.rlf — simple approach, no framework changes
enemy_subtype($t) = :from($t) :match($t) {
    // These are always used with @ein:acc in dissolve/banish/etc. contexts
    masc: "feindlichen {$t:acc}",
    fem: "feindliche {$t:acc}",
    *neut: "feindliches {$t:acc}",
};
```

If a predicate is used in multiple article contexts (rare in card games), the translator defines separate phrases:

```
// de.rlf
enemy_subtype_indef_acc($t) = :from($t) :match($t) {
    masc: "feindlichen {$t:acc}",
    fem: "feindliche {$t:acc}",
    *neut: "feindliches {$t:acc}",
};
enemy_subtype_def_nom($t) = :from($t) :match($t) {
    masc: "feindliche {$t:nom}",
    fem: "feindliche {$t:nom}",
    *neut: "feindliche {$t:nom}",
};
```

**Verdict: DO NOT add declension context to `@der`/`@ein`.** The pragmatic alternative works for the Dreamtides domain where predicate contexts are predictable. Adding transient tags is complex machinery for a niche use case. If a future game needs fully general German adjective declension, revisit then.

---

### Recommendation 2: `@ein` Should Return Empty String for Plural Context

**Problem**: `@ein` is documented as "singular-only" because German has no plural indefinite article. But the current behavior when `@ein:acc.other` is called is undefined — it might error, return "ein", or return empty.

**Proposed behavior**: `@ein` with an `.other` (plural) context should return the **empty string**. This allows translators to write:

```
// de.rlf
karten($n) = :match($n) {
    1: "{@ein:acc karte}",
    *other: "{$n} {karte:acc:other}",
};
```

But more importantly, it allows a single template to work for both singular and plural without `:match`:

```
// de.rlf — hypothetical if both singular and plural go through same template
draw_target($n, $t) = "{@ein:acc:$n $t:acc:$n}";
// n=1 → "eine Karte" (@ein:acc.one + karte:acc:one)
// n=3 → "Karten"     (@ein:acc.other = "" + karte:acc:other)
```

This is a minor convenience. In practice, the `:match` approach is clearer. But the framework should define the behavior rather than leaving it unspecified.

**Proposed syntax**: No new syntax needed. `@ein` with `.other` context returns `""`. Document this behavior explicitly.

**Verdict: YES, implement this.** Small change, prevents runtime errors, documented behavior is better than undefined behavior.

---

### Recommendation 3: NO New `@sep` Transform for Separable Verbs

**Problem**: German separable verbs split: "auflösen" → "Löse ... auf". Should there be a `@sep` transform?

**Answer: No.** Separable verbs are handled perfectly by phrase templates:

```
// de.rlf
dissolve_target_effect($target) = "Löse {@ein:acc $target} auf";
```

The translator places the prefix ("Löse") and suffix ("auf") freely in the template. No framework support needed. A `@sep` transform would need to:
1. Know where the verb splits (different for each verb: "auf|lösen", "ein|setzen", "ab|schaffen")
2. Know where to place the intervening text
3. Handle subordinate clause remerging ("...aufzulösen")

This is inherently phrase-level, not transform-level. The translator has full control with templates. An automated `@sep` would be brittle and less flexible.

**Verdict: DO NOT add `@sep`.** Phrase templates are the right abstraction.

---

### Recommendation 4: Add `:case` Shorthand for German Noun Variant Tables

**Problem**: German nouns need 4 cases × 2 numbers = 8 forms. Many nouns share forms across cases. Writing all 8 is verbose:

```
// Current verbose notation:
karte = :fem {
    nom: "Karte", acc: "Karte", dat: "Karte", gen: "Karte",
    nom.other: "Karten", acc.other: "Karten", dat.other: "Karten", gen.other: "Karten",
};
```

RLF already supports multi-key shorthand (`nom, acc, dat, gen: "Karte"`), so this is already somewhat compact:

```
// Already possible:
karte = :fem {
    nom, acc, dat, gen: "Karte",
    nom.other, acc.other, dat.other, gen.other: "Karten",
};
```

**However**, masculine nouns with different accusative forms and dative/genitive are more complex:

```
charakter = :masc {
    nom: "Charakter",
    acc, dat, gen: "Charakter",  // actually all same for this word
    nom.other: "Charaktere",
    acc.other, dat.other: "Charaktere",
    gen.other: "Charaktere",
};

// A noun with actual case differences:
junge = :masc {
    nom: "Junge",
    acc, dat, gen: "Jungen",     // weak masculine declension
    nom.other, acc.other, dat.other, gen.other: "Jungen",
};
```

**Proposed feature — wildcard-only variant (already in RLF!)**: The design doc says wildcard fallbacks work: `nom: "Karte"` matches `nom.*`. This means the "Karte" example can be written:

```
karte = :fem {
    nom: "Karte",         // fallback for nom.one AND nom.other (if no override)
    nom.other: "Karten",  // override for nom.other
    acc: "Karte",
    acc.other: "Karten",
    dat: "Karte",
    dat.other: "Karten",
    gen: "Karte",
    gen.other: "Karten",
};
```

But with multi-key shorthand, the compact form already works well:

```
karte = :fem {
    nom, acc, dat, gen: "Karte",
    nom.other, acc.other, dat.other, gen.other: "Karten",
};
```

**Verdict: NO new feature needed.** RLF's existing multi-key shorthand + wildcard fallbacks are sufficient. German nouns typically have at most 2-3 distinct forms across 8 slots. The existing notation is compact enough. A special `:case` shorthand would add syntax complexity for marginal gain.

---

### Recommendation 5: `@cap` Should Be Aware of Leading Markup — Already Is

**Problem**: German nouns are inherently capitalized ("Karte"). If `@cap` is applied to an already-capitalized German noun, it should be a no-op. If `@lower` is applied somewhere in the pipeline, it could incorrectly lowercase German nouns.

**Current behavior** (from APPENDIX_STDLIB.md): "`@cap` skips leading HTML-like markup tags (e.g., `<b>`, `<color=#AA00FF>`) to find the first visible character to capitalize." This is correct behavior — if the first visible character is already uppercase, `@cap` is a no-op.

**Concern about `@lower`**: If any RLF phrase or Rust code applies `@lower` to German text, nouns would be incorrectly lowercased. Verify that `@lower` is never used in English source phrases that German inherits. Currently, the English source doesn't use `@lower` at all — good.

**Verdict: NO framework change needed.** `@cap` is already correct. Document the guideline: never use `@lower` on content that might contain German nouns. This is a translation authoring guideline, not a framework constraint.

---

### Recommendation 6: Add `@kein` Transform for German Negative Article

**Problem**: German uses "kein" (no/not a) which declines like "ein": "keinen Charakter" (acc masc), "keine Karte" (acc fem), "kein Ereignis" (acc neut). Card text might need negative contexts: "if you have no cards" → "wenn du keine Karten hast".

**Proposed behavior**: `@kein` works identically to `@ein` but uses the negative article paradigm:

| Case | Masc | Fem | Neut | Plural |
|------|------|-----|------|--------|
| nom | kein | keine | kein | keine |
| acc | keinen | keine | kein | keine |
| dat | keinem | keiner | keinem | keinen |
| gen | keines | keiner | keines | keiner |

Syntax: `{@kein:acc $target}` → "keinen Charakter"

**However**, "kein" contexts are rare in card games. Most negation is phrased as "not a" which in German becomes "kein" anyway, but can also be expressed as "nicht ein" in certain constructions. The translator can inline "kein" forms in the phrase template without a transform:

```
// de.rlf — without @kein transform
no_cards = "keine Karten";
if_no_cards = "wenn du keine Karten hast";
```

**Verdict: DO NOT add `@kein` now.** It's a clean parallel to `@ein`, but the use cases in Dreamtides are few enough to handle inline. If a pattern emerges where many phrases need `@kein`, add it then.

---

### Recommendation 7: `text_number` Should Support Gender/Case Variants in German

**Problem** (raised by es-agent): English `text_number($n)` maps 1→"one", 2→"two", etc. German needs gendered number words for 1: "ein" (masc nom), "eine" (fem nom), "einen" (masc acc). Higher numbers don't vary by gender.

**Proposed change — not a framework change, but a translation pattern**:

```
// de.rlf
text_number = {
    // Override only the gendered form; 2-5 are invariant
    nom.masc: "ein", nom.fem: "eine", nom.neut: "ein",
    acc.masc: "einen", acc.fem: "eine", acc.neut: "ein",
    dat.masc: "einem", dat.fem: "einer", dat.neut: "einem",
};

text_number($n) = :match($n) {
    1: "{text_number}",  // returns the term with case/gender variants
    2: "zwei",
    3: "drei",
    4: "vier",
    5: "fünf",
    *other: "{$n}",
};
```

**The issue**: The consuming phrase needs to select both the number AND the case/gender: `{text_number($n):acc:$entity}`. But `text_number` is a phrase call, not a term, so the variant selection chains would need to work on the `:match` result.

**Actually, the simpler approach**: Since only "1" varies by gender in German, and "1" is usually expressed via `@ein` or `:match($n) { 1: ... }` anyway, the gendered number issue is already handled by the existing article transforms and `:match` branches. For `n_figments($n, $f)`:

```
// de.rlf
n_figments($n, $f) = :match($n) {
    1: "{@ein:acc figment($f)}",
    *other: "{text_number($n)} {figments_plural($f)}",
};
// n=1 → "ein Celestial-Figment" (or "einen Celestial-Figment" for acc masc)
// n=2 → "zwei Celestial-Figmente"
```

**Verdict: NO framework change needed.** German's gendered numbers only matter for "1", which is always handled by `@ein` or `:match` branches in practice. The `text_number` phrase in translation files simply returns invariant German words for 2+.

---

### Recommendation 8: `:from` Multi-Dimensional Propagation — Verify, Don't Change

**Problem**: When `:from($s)` is used and `$s` has multi-dimensional variants (e.g., `nom.one`, `acc.other`), the result should have the same multi-dimensional structure. This is critical for German where composed phrases like `enemy_subtype($t)` need case × number × gender dimensions inherited from `$t`.

**Current documented behavior**: "`:from($param)` causes a phrase to inherit tags and variants from a parameter" and "evaluates the template once per variant." This implies it iterates all variants, preserving the full multi-dimensional key structure.

**Verification needed**: Does `:from($s)` correctly handle:
1. `$s` with `nom.one: "X", nom.other: "Y", acc.one: "Z"` → result has `nom.one`, `nom.other`, `acc.one` variants?
2. Wildcard fallbacks in `$s` (e.g., `nom: "X"` as fallback) → propagated to result?
3. Tags from `$s` (`:masc`, `:fem`) → propagated to result?

The Russian translation appendix shows `:from` with case variants working correctly. Trust but verify for German's 4×2 case×number grid.

**Verdict: NO framework change.** Just verify the implementation handles multi-dimensional variants correctly. Add a test case: `:from($s)` where `$s` has `nom.one`, `nom.other`, `acc.one`, `acc.other` variants. Confirm all four propagate.

---

### Summary of RLF Framework Recommendations

| # | Recommendation | Change Type | Verdict |
|---|---------------|-------------|---------|
| 1 | Declension context on `@der`/`@ein` | New feature | **DO NOT ADD** — pragmatic alternative (hardcoded adjective forms per context) works for Dreamtides |
| 2 | `@ein` returns empty for plural | Behavior spec | **YES** — small change, prevents errors |
| 3 | `@sep` transform for separable verbs | New feature | **DO NOT ADD** — phrase templates handle this perfectly |
| 4 | `:case` shorthand for noun tables | New syntax | **DO NOT ADD** — existing multi-key shorthand is sufficient |
| 5 | `@cap` language awareness | Behavior check | **NO CHANGE** — already correct, document `@lower` guideline |
| 6 | `@kein` negative article transform | New feature | **DO NOT ADD** now — too few use cases, inline works |
| 7 | `text_number` gender/case support | Translation pattern | **NO CHANGE** — handled by `@ein` and `:match` branches |
| 8 | `:from` multi-dimensional propagation | Verification | **NO CHANGE** — verify implementation, add test cases |

**Net result: Only one small framework change recommended** — defining `@ein` behavior for plural context as returning empty string. Everything else is handled by existing RLF features or can be solved through translation authoring patterns.
