# i18n Review: Spanish (Espanol)

Reviewer: es-agent
Date: 2026-02-08

---

## Executive Summary

The Phase 2 migration plan is **well-designed for Spanish support**. The architecture of predicates returning `Phrase` objects with metadata tags, combined with RLF's `@el`/`@un` transforms and `:match` branching, addresses the vast majority of Spanish grammatical requirements. The APPENDIX_SPANISH_TRANSLATION.md walkthrough demonstrates that the cost serializer can be fully translated without any Rust code changes.

However, I identified **3 blocking issues** and **4 non-blocking concerns** that should be addressed to ensure Spanish localization works without future Rust code changes.

---

## Blocking Issues

### BLOCK-ES-1: Missing `:anim`/`:inan` Tags on English Source Terms

**Severity:** BLOCKING
**Affected:** All predicate noun terms in `strings.rs`

Spanish requires the "personal a" before animate direct objects:
- "Disuelve **a** un enemigo" (Dissolve an enemy) -- animate, needs "a"
- "Disuelve una carta" (Dissolve a card) -- inanimate, no "a"

The plan correctly identifies this in Section 9.3 and Section 9.6, stating that "English terms need `:anim`/`:inan` tags (already added in Phase 1.5 prep)." However, **inspecting the current `strings.rs`, no `:anim` or `:inan` tags exist on any term.** All character subtypes (agent, ancient, avatar, child, detective, etc.) only carry `:a` or `:an` tags. The terms `card`, `ally`, `figment`, etc. similarly lack animacy tags.

Without `:anim`/`:inan` tags on the English source terms, the Spanish translation file cannot use `:match($target)` to branch on animacy for the personal "a". This would require either:
- (a) Adding animacy tags later, which means modifying Rust source code (strings.rs), or
- (b) Creating separate Spanish-only terms for each animate/inanimate noun, duplicating the entire predicate system.

**Recommendation:** Add `:anim`/`:inan` tags to ALL noun terms in strings.rs during Phase 2 Task 2 (when predicate phrases are being defined). Specifically:
- `:anim` on: `ally`, all character subtypes (agent, ancient, avatar, child, etc.), `character` (when it appears as a term)
- `:inan` on: `card`, `event`, `figment` types, location-like terms

This is purely additive and won't break English (English doesn't read these tags), but it's critical for Spanish, Russian (accusative = genitive for animate masculine), and Portuguese.

### BLOCK-ES-2: Keyword Verbs Need Language-Specific Conjugation

**Severity:** BLOCKING
**Affected:** All keyword terms in `strings.rs` (dissolve, banish, materialize, reclaim, prevent, discover)

The current keyword definitions are:
```
dissolve = "<color=#AA00FF>dissolve</color>";
banish = "<color=#AA00FF>banish</color>";
materialize = "<color=#AA00FF>materialize</color>";
```

These are bare English infinitives wrapped in color markup. In Spanish, keywords on cards appear as **imperative verb forms** (2nd person singular command), which differ per verb:
- dissolve -> "disuelve" (irregular: o->ue stem change)
- banish -> "destierra" (irregular: e->ie stem change)
- materialize -> "materializa"
- reclaim -> "reclama"
- prevent -> "previene" (irregular: e->ie)
- discover -> "descubre" (irregular)

The problem: phrases like `dissolve_target_effect($target) = "{@cap dissolve} {@a $target}."` embed a reference to the `dissolve` term. In the Spanish translation file, the translator can redefine `dissolve = "<color=#AA00FF>disuelve</color>"`, which works for this one context. But if `dissolve` is also used in non-imperative contexts (e.g., "when X is dissolved" -> "cuando X es disuelto/disuelta"), the same term can't serve both purposes.

Currently, strings.rs has separate terms: `dissolve` (imperative), `dissolved` (past participle for triggers). This pattern is correct and should be **explicitly maintained** for all keywords. Verify that every keyword that appears in both imperative and participial/passive contexts has separate terms.

**Recommendation:** Audit all keyword terms to ensure that:
1. Imperative uses (effect text) and participial uses (trigger text) are separate terms
2. The `dissolved`/`banished` terms get `:match`-compatible structure so Spanish can branch on gender for past participles: "disuelto" (masc) vs "disuelta" (fem)
3. Document this pattern as a requirement for all future keyword additions

### BLOCK-ES-3: Past Participle Gender Agreement in Trigger Phrases

**Severity:** BLOCKING
**Affected:** `when_dissolved_trigger`, `when_banished_trigger`, and similar passive constructions

Consider:
```
when_dissolved_trigger($target) = "when {$target} is {{dissolved}}, ";
```

In Spanish, "dissolved" must agree with the gender of `$target`:
- "cuando un aliado es **disuelto**" (masculine)
- "cuando una carta es **disuelta**" (feminine)

The plan in Section 9.2 shows exactly how to handle this with `:match($target)`:
```
// ru.rlf
when_dissolved_trigger($target) = :match($target) {
    masc: "cuando {$target:nom} es disuelto, ",
    fem: "cuando {$target:nom} es disuelta, ",
};
```

This requires TWO things from the Rust/English side:
1. The `$target` Phrase must carry gender tags (`:masc`/`:fem`) -- these come from the translation file, not the English source. **This is fine** as long as the Spanish `.rlf` file redefines terms with gender tags.
2. The `dissolved` and `banished` terms in the Spanish translation must be able to vary by gender. Since they're separate terms referenced inside phrases, the Spanish translator can redefine them as variant blocks.

**However**, the current English source term `dissolved` is a simple string with no variant structure. If a consuming phrase uses `{dissolved:$target}` to try gender selection, this will fail at runtime because the English definition has no variants. The English source definition needs to at least have a default variant structure that won't break when Spanish adds gender variants.

**Recommendation:** Define `dissolved` and `banished` (and any other past participle terms) with variant blocks, even in English where they're identical:
```
dissolved = :inan { *default: "<color=#AA00FF>dissolved</color>" };
banished = :inan { *default: "<color=#AA00FF>banished</color>" };
```

Or alternatively, ensure that trigger phrases that use participles are structured so the Spanish translation can use `:match` at the phrase level (branching on `$target` tags) without needing to select variants from the participle term itself. The plan's approach in Section 9.2 (`:match` at the phrase level) is the correct solution -- as long as the consuming phrases accept `$target` as a `Phrase` parameter (which Phase 2 Task 2 guarantees).

**Revised assessment:** This is actually handled correctly by the plan IF the trigger phrases take `$target` as a Phrase parameter. The Spanish translation redefines the entire phrase with `:match($target)` branching. The English `dissolved` term doesn't need variants -- Spanish creates its own participial forms inline. **Downgrading from BLOCKING to CONCERN** -- the architecture supports this, but it should be documented as a pattern.

---

## Non-Blocking Concerns

### CONCERN-ES-1: Adjective Placement (Post-nominal in Spanish)

**Severity:** LOW
**Affected:** Predicate phrases with ownership qualifiers ("enemy card", "allied warrior")

English places adjectives before nouns: "enemy card", "allied warrior". Spanish places most adjectives after: "carta **enemiga**", "guerrero **aliado**".

The plan handles this correctly: each language defines its own phrase templates with whatever word order is natural. For example:
```
// en.rlf
enemy_card = :an :inan { one: "enemy card", other: "enemy cards" };

// es.rlf
enemy_card = :fem :inan { one: "carta enemiga", other: "cartas enemigas" };
```

The key insight is that "enemy" in Spanish becomes an adjective that must agree in gender AND number with the noun. The phrase template system handles this naturally since the entire noun phrase is defined as a unit.

**No action needed** -- this works as designed. Just noting it for documentation.

### CONCERN-ES-2: Top-Level String Concatenation and Trigger+Effect Assembly

**Severity:** MEDIUM
**Affected:** `ability_serializer.rs` (Section 2.3 of the plan)

The plan states that the ability serializer concatenates rendered pieces:
```rust
let trigger_text = strings::when_you_play_trigger(target_phrase).to_string();
let effect_text = strings::draw_cards_effect(count).to_string();
format!("{trigger_text}{effect_text}")
```

For Spanish, the trigger+effect concatenation generally works because Spanish card text follows a similar structure: "Cuando materializas a un aliado, roba 3 cartas." However, there are two sub-concerns:

1. **Comma placement**: Spanish uses commas in the same positions as English for triggers, so the current pattern of triggers ending with ", " works. No issue.

2. **Capitalization after triggers**: The ability serializer currently capitalizes the effect when there's no trigger. In Spanish, this is the same pattern. The `@cap` transform handles this correctly.

**Minor concern:** Some ability structures in Spanish might need the trigger to reference the effect's target for pronoun agreement. For example, "Cuando lo materializas, gana +2 de chispa" (When you materialize **it**, **it** gains +2 spark). If the trigger and effect are rendered independently, pronoun agreement across the boundary may not work. However, this is an edge case that can be handled by defining trigger phrases that include the relevant pronoun forms.

### CONCERN-ES-3: Possessives Agree with Possessed Noun, Not Possessor

**Severity:** LOW
**Affected:** Location phrases like "your hand", "your void", "your deck"

In English, "your" is invariant. In Spanish, possessives agree with the possessed noun's gender:
- "tu mano" (your hand, feminine)
- "tu vacío" (your void, masculine)
- "tu mazo" (your deck, masculine)

Since these are simple terms (not parameterized), the Spanish translation file can define them directly:
```
// es.rlf
your_hand = "tu mano";
your_void = "tu vacío";
```

**No issue** -- the architecture handles this naturally since these are standalone terms.

### CONCERN-ES-4: Number Words and Gender Agreement

**Severity:** LOW
**Affected:** `text_number` phrase, `copies` phrase, `n_figments` phrase

The `text_number` phrase converts numbers to words (1->"one", 2->"two", etc.). In Spanish, the numbers "one" ("un"/"una") and "twenty-one" ("veintiún"/"veintiuna") have gendered forms. For a card game context where numbers appear as "one", "two", "three", this matters primarily for `n=1`.

Currently, `text_number` takes only `$n` as a parameter. For Spanish, the translator would need to know the gender of the thing being counted to produce "un" vs "una". But since `text_number` is only called with a number parameter and no noun context, the Spanish translator cannot branch on gender.

**Recommendation:** For phrases that use `text_number` in contexts where gender matters (like `copies`, `n_figments`), the Spanish translation should inline the number word rather than calling `text_number`. This is a translator-level workaround and doesn't require Rust changes. However, consider adding a `text_number_gendered($n, $thing)` phrase for languages that need it, or document that `text_number` is English-specific and translators should inline numbers.

---

## Positive Findings

### The `:from` Mechanism Works Perfectly for Spanish

The `:from($s)` mechanism for subtypes is ideal for Spanish. When the Spanish `.rlf` file defines:
```
ancient = :masc { one: "Ancestral", other: "Ancestrales" };
subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";
```

The `subtype(ancient)` call inherits `:masc` tag and `one`/`other` variants, allowing `@un` and `@el` transforms to work correctly. This is demonstrated in the APPENDIX_SPANISH_TRANSLATION.md and works as designed.

### The `@un`/`@el` Transforms Cover Spanish Articles

The RLF stdlib (APPENDIX_STDLIB.md) defines:
- `@el` / `@la`: Definite articles (el/la/los/las) reading `:masc`/`:fem` with `:one`/`:other` context
- `@un` / `@una`: Indefinite articles (un/una/unos/unas) reading `:masc`/`:fem` with `:one`/`:other` context

These cover all Spanish article needs for card text.

### Predicate-as-Phrase Architecture Enables Full Spanish Support

The core Phase 2 decision -- predicates returning `Phrase` with metadata rather than pre-rendered `String` -- is exactly what Spanish needs. Without this, every consuming phrase would need to know the gender of every possible target. With this, the Phrase carries the gender tag and Spanish phrases use `:match` to branch.

### Section 9.3 Correctly Identifies the Personal "a" Pattern

The plan's example for Spanish personal "a":
```
dissolve_target($target) = :match($target) {
    anim: "{@cap dissolve} a {@un $target:acc}.",
    *inan: "{@cap dissolve} {@un $target:acc}.",
};
```

This is the correct approach. The only blocker is BLOCK-ES-1 (animacy tags must exist on source terms).

---

## Summary of Recommendations

| ID | Severity | Recommendation | Plan Task |
|----|----------|---------------|-----------|
| BLOCK-ES-1 | BLOCKING | Add `:anim`/`:inan` tags to all noun terms | Task 2 |
| BLOCK-ES-2 | BLOCKING | Ensure keyword terms have separate imperative vs participial forms | Task 2 |
| BLOCK-ES-3 | CONCERN | Document participle gender agreement pattern for trigger phrases | Task 2 |
| CONCERN-ES-1 | LOW | No action (architecture handles adjective placement) | -- |
| CONCERN-ES-2 | MEDIUM | Document trigger+effect concatenation behavior for translators | Task 7 |
| CONCERN-ES-3 | LOW | No action (possessives handled by standalone terms) | -- |
| CONCERN-ES-4 | LOW | Document `text_number` limitation; translators inline numbers | Task 4 |

**Bottom line:** The plan's architecture is sound for Spanish. The two blocking issues (animacy tags and keyword verb forms) are additive changes to `strings.rs` that should be made during Task 2. No structural redesign is needed.

---

## RLF Framework Change Recommendations for Spanish

Since RLF is a project-internal framework, we can modify it. Below are concrete recommendations for each proposed change, assessed against real Spanish card text patterns.

### RFC-ES-1: Should `:match` support `:anim`/`:inan` natively?

**Verdict: NO change needed -- already works.**

`:match` already supports tag-based branching. If a Phrase carries `:anim`, then `:match($target) { anim: "...", *inan: "..." }` works today. The issue isn't `:match` support; it's that the English source terms don't carry `:anim`/`:inan` tags yet. That's a `strings.rs` change, not an RLF framework change.

```
// This already works in RLF:
dissolve_target($target) = :match($target) {
    anim: "{@cap dissolve} a {@un $target}.",
    *inan: "{@cap dissolve} {@un $target}.",
};
```

No RLF change. Just add tags to English source terms.

### RFC-ES-2: Should `@un`/`@el` automatically insert personal "a"?

**Verdict: NO -- do NOT build this in.**

The personal "a" is syntactically separate from the article. It's a preposition, not part of the article system. Merging it into `@un`/`@el` would:
1. Make the transforms context-dependent (accusative vs nominative), adding complexity
2. Not generalize -- Portuguese doesn't have personal "a", so the same `@un` behavior would differ
3. Create confusing interactions: `@un` would sometimes produce "a un" (preposition + article) and sometimes just "un"

The correct pattern is explicit in the phrase template:

```
// es.rlf -- translator writes the "a" explicitly based on animacy
dissolve_target($target) = :match($target) {
    anim: "{@cap dissolve} a {@un $target}.",
    *inan: "{@cap dissolve} {@un $target}.",
};
```

This is readable, explicit, and gives translators full control. No framework change.

### RFC-ES-3: Should transforms communicate context to the phrase they modify?

**Verdict: PARTIALLY -- one small addition would help.**

The current context system (`@el:other`, `@un:one`, `@der:acc`) lets you pass static context to transforms. This handles article case/number selection well. However, there's no way for a transform to pass context *downstream* to the phrase it modifies.

**Concrete scenario where this matters:** When `@un` produces "un aliado", the downstream text knows the article was indefinite. But when building compound noun phrases like "un aliado enemigo" (an enemy ally), the adjective "enemigo" needs to agree with "aliado" in gender. This agreement already works via `:match` on the noun's gender tag -- the transform doesn't need to communicate anything.

**One case where downstream context would help:** The `@el` definite article in Spanish contracts with prepositions: "a" + "el" = "al", "de" + "el" = "del". Currently, a translator would need separate phrases for prepositional contexts:

```
// Current workaround -- separate phrases for contractions
from_target($t) = :match($t) { masc: "del {$t}", *fem: "de la {$t}" };
to_target($t) = :match($t) { masc: "al {$t}", *fem: "a la {$t}" };
```

**Proposed addition: `@de` and `@a` transforms for Spanish** (preposition+article contractions):

```
// New Spanish-specific transforms
@de($t)  reads :masc/:fem  → "del"/"de la"/"de los"/"de las"
@a($t)   reads :masc/:fem  → "al"/"a la"/"a los"/"a las"
```

Usage:
```
// es.rlf
from_void($t) = "{@cap banish} {@de $t}.";
// with :masc void → "Destierra del vacío."
// with :fem hand → "Destierra de la mano."
```

**This is a SMALL addition** (two new Spanish transforms) following the same pattern as Portuguese's existing `@de` and French's `@de`/`@au`. Worth adding to the stdlib.

### RFC-ES-4: Should past participle agreement be a built-in `@part` transform?

**Verdict: NO -- `:match` at the phrase level is simpler and more flexible.**

A hypothetical `@part` transform would:
```
// Hypothetical
when_dissolved_trigger($target) = "cuando {$target} es {@part dissolved $target}, ";
// @part reads :masc/:fem from $target, selects dissolved:masc or dissolved:fem
```

This is equivalent to what `:match` already does at the phrase level:
```
// Current approach -- equally readable, no new feature needed
when_dissolved_trigger($target) = :match($target) {
    masc: "cuando {$target} es disuelto, ",
    *fem: "cuando {$target} es disuelta, ",
};
```

The `:match` approach is:
- More explicit (translator sees exactly what each gender produces)
- More flexible (can restructure the entire sentence, not just the participle)
- Already implemented
- Works for Russian too (which needs case+gender+number, not just gender)

A `@part` transform would save a few lines per phrase but adds a new concept translators must learn, and it only works when the participle is the *only* thing that varies by gender. In practice, Spanish often varies more than just the participle (determiners, adjectives, pronouns may also change), so `:match` is the right tool.

**No framework change.**

### RFC-ES-5: Should `:from` propagate animacy tags?

**Verdict: YES -- `:from` should propagate ALL tags, including `:anim`/`:inan`.**

`:from($param)` currently inherits tags and variants from the parameter. The question is whether it propagates *all* tags or only certain ones.

**Concrete scenario:**
```
// en.rlf
ally = :an :anim { one: "ally", other: "allies" };
subtype($s) = :from($s) "<b>{$s}</b>";
```

If `subtype(ally)` inherits `:an` and `:anim`, then downstream phrases can both:
- Use `@a` to produce "an ally" (reads `:an`)
- Use `:match` to branch on animacy (reads `:anim`)

If `:from` only propagates article tags (`:a`/`:an`) but not animacy (`:anim`/`:inan`), then `subtype(ally)` loses animacy information, and Spanish can't do:
```
dissolve_target($target) = :match($target) {
    anim: "Disuelve a {@un $target}.",
    *inan: "Disuelve {@un $target}.",
};
```

**Recommendation: Verify and document that `:from` propagates ALL tags unconditionally.** If it currently filters or selects specific tags, change it to propagate everything. This is critical for the composition chain: `ally` → `subtype(ally)` → predicate phrase → consuming effect phrase. At each step, the animacy tag must survive.

**This may already work** -- the DESIGN.md says `:from` inherits "tags and variants" without mentioning filtering. But it should be explicitly tested and documented.

### RFC-ES-6: Should `text_number_gendered($n, $thing)` be a stdlib pattern?

**Verdict: YES -- add it as an OPTIONAL pattern, not a built-in transform.**

Spanish needs gendered number words only for n=1 ("un"/"una") and n=21 ("veintiún"/"veintiuna"). For a card game, only n=1 realistically matters.

**Proposed stdlib pattern** (not a new transform, just a recommended phrase pattern):

```
// es.rlf -- gendered number words
text_number($n, $thing) = :match($n, $thing) {
    1.masc: "un",
    1.*fem: "una",
    2.*: "dos",
    3.*: "tres",
    4.*: "cuatro",
    5.*: "cinco",
    *other.*: "{$n}",
};
```

This uses existing multi-parameter `:match` -- no new RLF feature needed. The English `text_number($n)` has one parameter; the Spanish version has two. This is fine because each language defines its own phrase implementations.

**However**, there's a problem: the English source defines `text_number($n)` with ONE parameter, and the Spanish translation defines `text_number($n, $thing)` with TWO parameters. RLF validates arity at compile time for source definitions. Does the runtime interpreter allow a translation to have a different parameter count than the source?

**If not, this is an RLF framework change needed:** Allow translation files to add parameters that the source definition doesn't have. The consuming phrase would need to pass the extra parameter:

```
// en.rlf (source)
copies($n) = :match($n) { 1: "a copy", *other: "{text_number($n)} copies" };

// es.rlf (translation) -- needs $n AND gender context
copies($n) = :match($n) { 1: "una copia", *other: "{text_number($n, copies)} copias" };
```

Wait -- this doesn't work either because `copies` is a phrase call, not a term. The Spanish translator would actually write:

```
// es.rlf
copia = :fem { one: "copia", other: "copias" };
copies($n) = :match($n) {
    1: "una copia",
    *other: "{$n} copias",
};
```

The translator just inlines the number word and doesn't call `text_number` at all. This is the simplest approach and requires no framework change.

**Revised verdict: No framework change needed.** Document that `text_number` is an English convenience and translators for gendered languages should inline number words in their phrase templates. The existing `:match` handles all cases.

### RFC-ES-7: New Spanish-specific transforms beyond `@el`/`@un`?

**Verdict: YES -- add `@de` and `@a` for preposition+article contractions.**

As discussed in RFC-ES-3, Spanish has two mandatory contractions:
- "de" + "el" = "del" (of the)
- "a" + "el" = "al" (to the)

These ONLY apply to the masculine singular definite article "el". All other combinations are not contracted: "de la", "de los", "de las", "a la", "a los", "a las".

**Proposed transforms:**

| Transform | Aliases | Reads | Context | Effect |
|-----------|---------|-------|---------|--------|
| `@de` | -- | `:masc`, `:fem` | `:one`/`:other` | "de" + article (del/de la/de los/de las) |
| `@a_prep` | -- | `:masc`, `:fem` | `:one`/`:other` | "a" + article (al/a la/a los/a las) |

Note: Can't use `@a` as the transform name because `@a` is already taken by English for indefinite articles. Use `@a_prep` or `@al` instead.

Actually, looking at the stdlib more carefully: the DESIGN.md says "Transform Names Are Language-Scoped" -- the same name can have different meanings in different languages. English's `@a` and Spanish's `@a` would be different transforms registered for different languages. So `@a` IS available for Spanish.

**However**, this creates confusion: Spanish's `@a` would mean "preposition 'a' + article contraction" while in the same project English's `@a` means "indefinite article." To avoid translator confusion, use a distinct name.

**Proposed:**

```
// Spanish preposition+article contractions
@del  reads :masc/:fem, context :one/:other
  masc.one: "del"    masc.other: "de los"
  fem.one: "de la"   fem.other: "de las"

@al   reads :masc/:fem, context :one/:other
  masc.one: "al"     masc.other: "a los"
  fem.one: "a la"    fem.other: "a las"
```

Usage:
```
// es.rlf
banish_from_void($location) = "{banish} {@del $location}.";
// masc void → "Destierra del vacío."

return_to_hand($target, $location) = "devuelve {$target} {@al $location}.";
// fem hand → "devuelve un aliado a la mano."
```

**Priority: LOW.** Card text rarely needs these contractions in parameterized contexts. Most locations ("tu vacío", "tu mano") use the possessive "tu" instead of a definite article, avoiding the contraction entirely. But "the opponent's void" might contract: "del vacío del oponente." Add these if time permits; translators can work around them with `:match` for now.

---

## Summary of RLF Framework Recommendations

| ID | Change | Priority | Effort |
|----|--------|----------|--------|
| RFC-ES-1 | No change (`:match` already supports animacy branching) | -- | -- |
| RFC-ES-2 | No change (personal "a" is explicit in phrase templates) | -- | -- |
| RFC-ES-3 | Add `@del`/`@al` Spanish preposition+article contractions | LOW | Small |
| RFC-ES-4 | No change (`:match` handles participle agreement) | -- | -- |
| RFC-ES-5 | Verify `:from` propagates ALL tags (including `:anim`/`:inan`) | HIGH | Verify only |
| RFC-ES-6 | No change (translators inline gendered number words) | -- | -- |
| RFC-ES-7 | Same as RFC-ES-3: `@del`/`@al` transforms | LOW | Small |

**Key takeaway:** RLF's existing primitives (`:match`, `:from`, `@un`/`@el`, tag-based branching) handle Spanish grammar well. The framework needs almost no changes. The critical action items are:

1. **Verify `:from` tag propagation** -- ensure all tags (including future `:anim`/`:inan`) propagate through the composition chain
2. **Add `:anim`/`:inan` tags to English source terms** -- this is a `strings.rs` change, not an RLF change
3. **Optionally add `@del`/`@al`** -- low priority, workarounds exist
