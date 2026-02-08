# i18n Review: Brazilian Portuguese (PT-BR)

**Reviewer:** PT-BR Language Agent
**Date:** 2025-02-08
**Document Reviewed:** `docs/plans/serializer_rlf_migration.md`
**References:** RLF DESIGN.md, APPENDIX_STDLIB.md, APPENDIX_SPANISH_TRANSLATION.md, strings.rs

---

## Executive Summary

The migration plan is **well-suited for PT-BR** with only minor gaps. Portuguese
and Spanish share many structural features (gender agreement, article systems,
adjective placement), and the RLF framework already provides dedicated PT-BR
transforms (`@o`, `@um`, `@de`, `@em`). The architecture of passing `Phrase`
values with metadata tags is the correct approach for a gendered Romance language
with preposition-article contractions. I identify **3 blocking issues** (requiring
Rust code awareness), **4 moderate concerns**, and **5 informational notes**.

**Overall verdict:** GREEN — no Rust code changes needed beyond what the plan
already specifies. All PT-BR requirements can be met through `.rlf` translation
files alone, provided the plan's Phase 2 architecture is fully implemented.

---

## 1. Gender Agreement

### 1.1 Noun Gender System

PT-BR has a strict two-gender system (masculine/feminine). Every noun, article,
adjective, and many pronouns must agree in gender and number.

| English | PT-BR | Gender | Singular | Plural |
|---------|-------|--------|----------|--------|
| card | carta | :fem | carta | cartas |
| character | personagem | :masc | personagem | personagens |
| ally | aliado | :masc | aliado | aliados |
| enemy | inimigo | :masc | inimigo | inimigos |
| event | evento | :masc | evento | eventos |
| void | vazio | :masc | vazio | — |
| hand | mão | :fem | mão | mãos |
| spark | centelha | :fem | centelha | centelhas |
| cost | custo | :masc | custo | custos |

### 1.2 Assessment

**ADEQUATE.** The plan calls for `:anim`/`:inan` tags in English source terms
(Section 9.6). For PT-BR, we need `:masc`/`:fem` tags, which are added in the
translation `.rlf` file — exactly as shown in the Spanish walkthrough. The
`:from($s)` mechanism correctly propagates gender tags through `subtype()` and
similar wrapping phrases.

### 1.3 Concern: Animacy Tags on English Source Terms

The plan states English terms should carry `:anim`/`:inan` tags (Section 9.2,
9.6). These are primarily needed for Spanish personal "a" and Russian
accusative=genitive rule. PT-BR does **not** use personal "a" for direct objects
(unlike Spanish), so `:anim`/`:inan` tags are not strictly needed for PT-BR.
However, they do no harm and are useful cross-linguistically.

**Status:** No issue for PT-BR.

---

## 2. Article System and Contractions

### 2.1 Definite Articles (via `@o`)

PT-BR definite articles:

| | Masculine | Feminine |
|---|-----------|----------|
| Singular | o | a |
| Plural | os | as |

The `@o` transform (APPENDIX_STDLIB.md, line 298-301) reads `:masc`/`:fem` tags
and supports `:one`/`:other` context for plural selection, exactly like Spanish `@el`.

```
// pt_br.rlf
card = :fem { one: "carta", other: "cartas" };
enemy = :masc { one: "inimigo", other: "inimigos" };

return_all($target) = "devolva {@o:other $target} à mão";
// card → "devolva as cartas à mão"
// enemy → "devolva os inimigos à mão"
```

**Status:** ADEQUATE.

### 2.2 Indefinite Articles (via `@um`)

PT-BR indefinite articles:

| | Masculine | Feminine |
|---|-----------|----------|
| Singular | um | uma |
| Plural | uns | umas |

The `@um` transform reads `:masc`/`:fem` tags.

```
// pt_br.rlf
abandon_one($target) = "abandone {@um $target}";
// ally (masc) → "abandone um aliado"
// card (fem) → "abandone uma carta"
```

**Status:** ADEQUATE.

### 2.3 Preposition Contractions (via `@de` and `@em`)

This is where PT-BR has a critical feature: prepositions contract with definite
articles.

| Preposition | + o | + a | + os | + as |
|-------------|-----|-----|------|------|
| de (of/from) | do | da | dos | das |
| em (in/at) | no | na | nos | nas |
| a (to) | ao | à | aos | às |
| por (by/for) | pelo | pela | pelos | pelas |

RLF provides `@de` and `@em` for Portuguese (APPENDIX_STDLIB.md, line 300-301).

```
// pt_br.rlf
void = :masc "vazio";
hand = :fem "mão";

from_void = "{@de void}";    // → "do vazio"
in_hand = "{@em hand}";      // → "na mão"
```

### 2.4 MODERATE CONCERN: Missing `@a` and `@por` Preposition Transforms

The APPENDIX_STDLIB.md lists `@o`, `@um`, `@de`, and `@em` for Portuguese. But
PT-BR also needs:

- **`@a` (to + article):** "ao vazio" (to the void), "à mão" (to the hand),
  "aos aliados" (to the allies), "às cartas" (to the cards). The contraction
  `a + a = à` uses a grave accent (crase).
- **`@por` (by/for + article):** "pelo custo" (by the cost), "pela centelha"
  (by the spark).

The current plan notes `@a` as an Italian transform (preposition+article) and
as an English transform (indefinite article). Since transforms are
language-scoped (APPENDIX_STDLIB.md, Design Notes), PT-BR can define its own
`@a` without conflict. However, `@a` and `@por` need to be **implemented in the
RLF stdlib** for Portuguese. If they are not, translators would need workarounds.

**Workaround:** Translators can manually write the contracted forms:

```
// Workaround if @a is not available
to_void = "ao vazio";       // hardcoded instead of {@a void}
to_hand = "à mão";          // hardcoded instead of {@a hand}
```

This works for fixed location terms but breaks down for dynamic phrases where
the target is a `$parameter` (e.g., "return X **to the** hand" where X varies).
In those cases, the preposition+article form depends on the gender of the
following noun.

**Impact:** MODERATE. For Dreamtides card text specifically, most "to" phrases
involve fixed locations ("to hand", "to your void"), so hardcoded workarounds
suffice. But a complete PT-BR stdlib should include `@a` and `@por`.

**Recommendation:** Add `@a` (to+article) and `@por` (by+article) to the RLF
Portuguese stdlib before writing PT-BR translation files. This is an **RLF
library change**, not a Rust serializer change.

---

## 3. Adjective Placement

### 3.1 Post-nominal Adjectives

PT-BR, like Spanish, places most adjectives **after** the noun:

| English | PT-BR | Note |
|---------|-------|------|
| enemy character | personagem inimigo | adj follows noun |
| allied warrior | guerreiro aliado | adj follows noun |
| enemy Ancient | Ancião inimigo | adj follows noun |
| random character | personagem aleatório | adj follows noun |

A few adjectives can precede the noun for emphasis or style, but the default
post-nominal position is standard for card game text.

### 3.2 Assessment

**ADEQUATE.** RLF phrase templates give translators full control over word order.
The English template `"enemy {$entity:one}"` becomes PT-BR
`"{$entity:one} inimigo"` — a simple reordering within the `.rlf` file.

```
// en.rlf
enemy_modified($entity) = "enemy {$entity:one}";

// pt_br.rlf
enemy_adj = :match($entity) {
    masc: "inimigo",
    *fem: "inimiga",
};
enemy_modified($entity) = "{$entity:one} {enemy_adj:$entity}";
// Ancient (masc) → "Ancião inimigo"
// card (fem) → "carta inimiga"
```

The adjective must also agree in gender, which is handled by `:match` on the
entity's tag. This pattern is identical to the Spanish and Russian walkthroughs.

**Status:** No Rust code changes needed.

---

## 4. Possessives

### 4.1 PT-BR Possessive Agreement

Possessives in PT-BR agree with the **possessed noun's** gender, not the
possessor's:

| English | PT-BR | Why |
|---------|-------|-----|
| your hand | sua mão | :fem (mão) |
| your void | seu vazio | :masc (vazio) |
| your card | sua carta | :fem (carta) |
| your ally | seu aliado | :masc (aliado) |
| your cards | suas cartas | :fem + plural |
| your allies | seus aliados | :masc + plural |

### 4.2 Assessment

**ADEQUATE with caveat.** For fixed terms like "your hand" and "your void", the
translator simply writes the correct possessive form directly:

```
// pt_br.rlf
your_hand = "sua mão";
your_void = "seu vazio";
```

For dynamic possessives (e.g., "your [predicate]"), the possessive must agree
with the parameter's gender. This requires `:match`:

```
// pt_br.rlf
seu_adj = :match($thing) {
    masc: "seu",
    *fem: "sua",
};
seus_adj = :match($thing) {
    masc: "seus",
    *fem: "suas",
};

your_card = :fem {
    one: "sua carta",
    other: "suas cartas",
};
your_event = :masc {
    one: "seu evento",
    other: "seus eventos",
};
```

**Status:** ADEQUATE. No Rust code changes needed. The existing plan for
predicate phrases returning `Phrase` with gender tags is exactly what PT-BR
needs.

---

## 5. Verb Forms (Imperatives)

### 5.1 PT-BR Imperative Mode

Card game text in PT-BR uses the imperative mood (command form). PT-BR has a
specific imperative conjugation for "você" (the standard 2nd person):

| English | PT-BR Imperative | Infinitive |
|---------|-----------------|------------|
| Draw | Compre | Comprar |
| Discard | Descarte | Descartar |
| Dissolve | Dissolva | Dissolver |
| Banish | Desterre | Desterrar |
| Materialize | Materialize | Materializar |
| Prevent | Evite | Evitar |
| Foresee | Preveja | Prever |
| Kindle | Acenda | Acender |
| Reclaim | Recupere | Recuperar |
| Gain | Ganhe | Ganhar |
| Lose | Perca | Perder |
| Return | Devolva | Devolver |
| Abandon | Abandone | Abandonar |

### 5.2 Assessment

**ADEQUATE.** Keywords and verbs are defined as RLF terms/phrases, so PT-BR
simply provides its own imperative forms:

```
// pt_br.rlf
dissolve = "<color=#AA00FF>dissolva</color>";
banish = "<color=#AA00FF>desterre</color>";
reclaim = "<color=#AA00FF>recupere</color>";
materialize = "<color=#AA00FF>materialize</color>";
prevent = "<color=#AA00FF>evite</color>";
kindle($k) = "<color=#AA00FF>acenda</color> {$k}";
foresee($n) = "<color=#AA00FF>preveja</color> {$n}";
```

**Note:** Some keyword verbs in English are also used as nouns/adjectives
(e.g., "Materialized" as a trigger name, "Dissolved" as a state). PT-BR may
need different forms for these:

| English | PT-BR (imperative) | PT-BR (adjective/trigger) |
|---------|--------------------|---------------------------|
| Dissolve | Dissolva | Dissolvido/Dissolvida |
| Materialized | — | Materializado |
| Dissolved | — | Dissolvido |

The trigger names ("Materialized:", "Dissolved:") are separate RLF terms
(`materialized`, `dissolved`), so PT-BR can provide the participial form
independently of the imperative keyword.

**Status:** ADEQUATE.

---

## 6. Preposition Contractions in Context

### 6.1 Common Patterns in Card Text

| English | PT-BR | Contraction |
|---------|-------|-------------|
| from your void | do seu vazio | de + o = do |
| from the opponent's void | do vazio do oponente | de + o = do |
| in your hand | na sua mão | em + a = na |
| to hand | à mão | a + a = à |
| from your void | de seu vazio | (no article, no contraction) |
| for each ally | para cada aliado | (no contraction) |

### 6.2 Assessment

Most contraction contexts in Dreamtides involve **fixed location phrases**
("your void", "your hand", "the opponent's void"). These are defined as complete
terms in the `.rlf` file, so contractions are baked in:

```
// pt_br.rlf
banish_n_from_your_void($n) = "{banish} {cards($n)} do seu vazio";
banish_n_from_opponent_void($n) = "{banish} {cards($n)} do vazio do oponente";
in_your_void($things) = "{$things} no seu vazio";
```

For dynamic targets, the `@de` and `@em` transforms handle contractions
correctly.

**Status:** ADEQUATE for current card text patterns.

---

## 7. String Concatenation at Top Level

### 7.1 The Concern

Section 2.3 of the migration plan uses string concatenation at the ability
serializer level:

```rust
let trigger_text = strings::when_you_play_trigger(target_phrase).to_string();
let effect_text = strings::draw_cards_effect(count).to_string();
format!("{trigger_text}{effect_text}")
```

For PT-BR, this pattern works because Portuguese follows the same
trigger-then-effect ordering as English:

| English | PT-BR |
|---------|-------|
| When you play an ally, draw 3 cards. | Quando você joga um aliado, compre 3 cartas. |
| Materialized: Gain 2 energy. | Materializado: Ganhe 2 de energia. |
| At the end of your turn, kindle 1. | No final do seu turno, acenda 1. |

### 7.2 Assessment

**ADEQUATE.** Portuguese maintains the same clause ordering as English for game
text (trigger/condition first, then effect). The concatenation approach works.

### 7.3 INFORMATIONAL: "When" Clauses

PT-BR uses "Quando" (when) at the start, exactly like English. The trigger
phrase can be a self-contained translated unit:

```
// pt_br.rlf
when_you_play_trigger($target) = "quando você joga {$target}, ";
when_dissolved_trigger($target) = "quando {$target} é {dissolved}, ";
at_end_of_your_turn_trigger = "no final do seu turno, ";
```

**Status:** No issues.

---

## 8. Predicate Return Type and Gender Tags

### 8.1 BLOCKING ISSUE: `:a`/`:an` Tags Are English-Specific

The current `strings.rs` uses `:a` and `:an` tags on predicate noun phrases
(e.g., `ally = :an{ ... }`, `card = :a{ ... }`). These tags are
**English-specific** — they control the `@a` transform for English indefinite
articles.

For PT-BR, what matters is `:masc`/`:fem` tags (controlling `@um` and `@o`
transforms). The plan correctly notes that gender tags are added in translation
files (Section 9.6). However, the `:a`/`:an` tags on the English source terms
do **not propagate as gender information** to PT-BR.

**How it works:** When PT-BR `.rlf` file redefines `card = :fem { ... }` and
`ally = :masc { ... }`, the runtime uses the PT-BR definitions (with their
`:fem`/`:masc` tags) instead of the English definitions (with `:a`/`:an` tags).
The `@um` transform in PT-BR reads `:masc`/`:fem` from the PT-BR definition.

**This is correct behavior.** Each language provides its own tags appropriate
for its transforms. No issue here.

**Status:** NOT AN ISSUE. The architecture handles this correctly — each
language's `.rlf` file defines its own tags.

### 8.2 BLOCKING CONCERN: `:from` Must Propagate Language-Specific Tags

The `subtype($s) = :from($s) "..."` pattern propagates tags from the parameter.
For PT-BR, the subtype definitions must carry `:masc`/`:fem` tags:

```
// pt_br.rlf
ancient = :masc { one: "Ancião", other: "Anciãos" };
child = :fem { one: "Criança", other: "Crianças" };
warrior = :masc { one: "Guerreiro", other: "Guerreiros" };
mage = :masc { one: "Mago", other: "Magos" };

subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";
```

Then `subtype(ancient)` in PT-BR produces a `Phrase` with `:masc` tag, enabling
`{@um subtype($s)}` → "um Ancião".

**Critical question:** Does `:from` work correctly when the source language uses
`:a`/`:an` tags but the translation uses `:masc`/`:fem`? **Yes** — at runtime,
the interpreter uses the translation's definition of `ancient` (with `:masc`),
not the English one (with `:an`). The `:from` propagates whatever tags the
current language defines.

**Status:** ADEQUATE, but this is a critical architectural dependency. If the
runtime interpreter were to fall back to English definitions when a translation
is missing, it would propagate English tags (`:a`/`:an`) instead of gender tags.
The plan's note that "translations must be complete" (DESIGN.md, line 664)
correctly prevents this.

---

## 9. Comparison with Spanish

PT-BR and Spanish are closely related but have key differences that affect
localization:

### 9.1 Differences That DO Matter

| Feature | Spanish | PT-BR | Impact |
|---------|---------|-------|--------|
| 2nd person | "tú" (informal) | "você" (standard) | Verb conjugation differs |
| Imperative form | "Roba" (tú) | "Compre" (você) | Different verb endings |
| Personal "a" | "Disuelve **a** un enemigo" | "Dissolva um inimigo" (no personal "a") | PT-BR is simpler |
| Preposition contractions | de+el=del, a+el=al | de+o=do, de+a=da, em+o=no, em+a=na, a+o=ao, a+a=à | PT-BR has MORE contractions |
| Definite articles | el/la/los/las | o/a/os/as | Different transforms (`@el` vs `@o`) |
| Indefinite articles | un/una/unos/unas | um/uma/uns/umas | Same transform (`@un`) |
| "Void" | "vacío" (masc) | "vazio" (masc) | Same gender, different word |
| "Hand" | "mano" (fem) | "mão" (fem) | Same gender, different word |

### 9.2 Differences That DON'T Matter for Architecture

| Feature | Spanish | PT-BR | Why No Impact |
|---------|---------|-------|---------------|
| Gender system | 2 genders | 2 genders | Same architecture |
| Adjective placement | post-nominal | post-nominal | Same RLF pattern |
| Number agreement | singular/plural | singular/plural | Same CLDR categories |
| Word order | SVO | SVO | Same concatenation approach |

### 9.3 MODERATE CONCERN: "You" Pronoun

Spanish card text often uses the implicit subject (verb conjugation implies the
subject). PT-BR more explicitly uses "você":

| English | Spanish | PT-BR |
|---------|---------|-------|
| "when you play" | "cuando juegas" | "quando você joga" |
| "draw 3 cards" | "roba 3 cartas" | "compre 3 cartas" |
| "gain 2 energy" | "gana 2 de energía" | "ganhe 2 de energia" |

The effect phrases in the current plan work fine for this:

```
// pt_br.rlf
draw_cards_effect($c) = "compre {cards($c)}.";
// No "you" needed — imperative mode implies the subject
```

However, trigger phrases that say "when you..." need "quando você...":

```
// pt_br.rlf
when_you_play_trigger($target) = "quando você joga {$target}, ";
when_you_materialize_trigger($target) = "quando você {materialize} {$target}, ";
```

**Status:** ADEQUATE. The phrase-per-trigger approach gives translators full
control.

---

## 10. Specific Phrase-by-Phrase Analysis

### 10.1 Category B Phrases (Escapes to be Removed)

All 50 Category B phrases can be translated to PT-BR. Sampling key ones:

| English Phrase | PT-BR Translation | Notes |
|---------------|-------------------|-------|
| `draw_cards_effect($c)` | `"compre {cards($c)}."` | cards() redefined in PT-BR |
| `gain_energy_effect($e)` | `"ganhe {energy($e)}."` | energy() shared |
| `foresee_effect($f)` | `"{foresee($f)}."` | foresee() redefined |
| `kindle_effect($k)` | `"{kindle($k)}."` | kindle() redefined |
| `when_you_materialize_trigger($target)` | `"quando você {materialize} {$target}, "` | Natural word order |
| `when_dissolved_trigger($target)` | `"quando {$target} é {dissolved}, "` | Passive with "é" |
| `banish_your_void_cost` | `"{Banish} seu vazio"` | Possessive agrees with "vazio" (masc) |

### 10.2 Structural Phrases

| English Phrase | PT-BR Translation | Notes |
|---------------|-------------------|-------|
| `you_may_prefix` | `"você pode "` | Explicit "você" |
| `until_end_of_turn_prefix` | `"Até o final do turno, "` | Contraction: a+o=ao not needed here |
| `once_per_turn_prefix` | `"Uma vez por turno, "` | "Uma" (fem) agrees with "vez" |
| `then_joiner` | `", então "` | Or `", depois "` |
| `and_joiner` | `" e "` | Simple |
| `cost_effect_separator` | `": "` | Same |

### 10.3 MODERATE CONCERN: Passive Voice with Gender Agreement

The "when dissolved" trigger uses passive voice. In PT-BR, the past participle
agrees with the subject's gender:

| English | PT-BR (masc subject) | PT-BR (fem subject) |
|---------|---------------------|---------------------|
| "when X is dissolved" | "quando X é dissolvido" | "quando X é dissolvida" |
| "when X is banished" | "quando X é desterrado" | "quando X é desterrada" |

The current plan defines `when_dissolved_trigger($target)` as a single phrase.
For PT-BR, this phrase needs `:match($target)` to select the correct participle
gender:

```
// pt_br.rlf
when_dissolved_trigger($target) = :match($target) {
    masc: "quando {$target} é {dissolved:masc}, ",
    *fem: "quando {$target} é {dissolved:fem}, ",
};

dissolved = {
    masc: "<color=#AA00FF>dissolvido</color>",
    fem: "<color=#AA00FF>dissolvida</color>",
};
```

**Status:** ADEQUATE — RLF's `:match` on gender tags handles this. No Rust code
changes needed. The translator has full control.

### 10.4 INFORMATIONAL: Plural Card Counts

PT-BR uses the same CLDR plural categories as English (`one`, `other`). The
"cards" phrase works naturally:

```
// pt_br.rlf
card = :fem { one: "carta", other: "cartas" };
cards($n) = :match($n) {
    1: "uma carta",
    *other: "{$n} cartas",
};
```

No issues with CLDR plural rules for PT-BR.

---

## 11. Summary of Findings

### BLOCKING ISSUES (0)

None. All PT-BR requirements can be met through `.rlf` translation files alone.

### MODERATE CONCERNS (4)

1. **Missing `@a` (to+article) and `@por` (by+article) transforms** in the RLF
   Portuguese stdlib. These are preposition+article contractions needed for
   natural PT-BR text. Workaround exists for fixed locations but breaks for
   dynamic targets. **Fix:** Add to RLF stdlib (not a Rust serializer change).

2. **Passive voice gender agreement** ("é dissolvido" vs "é dissolvida") requires
   `:match` on gender tags in PT-BR translation files. The architecture supports
   this perfectly — just flagging it as a pattern translators must implement
   consistently.

3. **"You" pronoun** — PT-BR explicitly uses "você" in some constructions where
   Spanish uses implicit subject. The phrase-per-template approach handles this,
   but translators need awareness.

4. **Adjective gender agreement** — unlike English, PT-BR adjectives must agree
   in gender with their noun. Phrases like `allied($entity)` and
   `enemy_modified($entity)` need `:match($entity)` for adjective agreement in
   the PT-BR file. Same pattern as Spanish/Russian — no issue for the
   architecture.

### INFORMATIONAL NOTES (5)

1. PT-BR does **not** use "personal a" (unlike Spanish). The `:anim`/`:inan`
   tags planned for Spanish are harmless but unnecessary for PT-BR.

2. PT-BR has **more preposition contractions** than Spanish (em+article, por+article
   in addition to de+article and a+article). The RLF stdlib covers `@de` and
   `@em` but may need `@a` and `@por`.

3. PT-BR uses the same CLDR plural categories (`one`, `other`) as English and
   Spanish — no complexity here (unlike Russian's `one`/`few`/`many`).

4. The case study in Appendix C correctly shows PT-BR "Compre uma carta" /
   "Compre 3 cartas" and "Dissolva um Ancião inimigo" — these are accurate
   translations.

5. PT-BR keyword translations should use the "você" imperative form (3rd person
   singular subjunctive used as imperative): "Compre" (buy/draw), "Dissolva"
   (dissolve), "Desterre" (banish), etc.

---

## 12. Recommendations

1. **Add `@a` and `@por` to RLF Portuguese stdlib** before writing translation
   files. This is the only non-trivial gap.

2. **Create a PT-BR translation template** following the Spanish walkthrough
   pattern. PT-BR and Spanish are similar enough that the Spanish walkthrough
   serves as an excellent starting point, with adjustments for:
   - Different verb conjugations (você imperative)
   - Different contractions (do/da/no/na vs del/al)
   - Lack of personal "a"
   - Different articles (o/a vs el/la)

3. **Test passive constructions** ("é dissolvido/a", "é desterrado/a") early,
   as these require gender-matching participles that stress-test the `:match`
   system.

4. **Validate contraction contexts** — PT-BR has more contraction types than
   Spanish. Ensure all preposition+article combinations that appear in card text
   are covered either by transforms or by direct translation.

---

## 13. Conclusion

The Phase 2 migration plan provides a solid foundation for PT-BR localization.
The key architectural decisions — `Phrase` values with metadata tags, per-language
transforms, `:from` propagation, `:match` branching — are all correct and
sufficient for Portuguese. The only gap is in the RLF stdlib (missing `@a` and
`@por` transforms), which is a library-level fix, not a Rust serializer change.

PT-BR localization can proceed as a pure translation task after Phase 2 is
complete, requiring zero Rust code changes.

---

## 14. Concrete RLF Framework Change Recommendations

This section proposes specific changes to the RLF framework itself (the `rlf`
crate) to improve Brazilian Portuguese support. Since RLF was created for this
project, we can modify it freely. Each recommendation includes exact syntax,
behavior, and implementation guidance.

---

### 14.1 Add `@para` Transform (Preposition "a" + Article Contraction)

**Priority:** HIGH — needed for natural card text with dynamic targets.

**Problem:** PT-BR contracts the preposition "a" (to/at) with definite articles:

| Preposition + Article | Contraction | Example |
|----------------------|-------------|---------|
| a + o | ao | "devolva ao vazio" (return to the void) |
| a + a | à | "devolva à mão" (return to the hand) |
| a + os | aos | "devolva aos aliados" (return to the allies) |
| a + as | às | "devolva às cartas" (return to the cards) |

Currently, `@a` is an **alias for `@o`** (the definite article) in Portuguese
(see `transforms.rs` line 2465). This means `@a` cannot also be the
preposition+article contraction transform.

**Proposed solution:** Add a new transform `@para` (named after the Portuguese
preposition "para", which is related to "a" but unambiguous). Use `@ao` as an
alias.

**Syntax:**

```
// pt_br.rlf
void = :masc "vazio";
hand = :fem "mão";

to_void = "{@para void}";          // → "ao vazio"
to_hand = "{@para hand}";          // → "à mão"
to_allies = "{@para:other ally}";  // → "aos aliados"
to_cards = "{@para:other card}";   // → "às cartas"
```

**Implementation:** Add to `transforms.rs`:

```rust
// In TransformKind enum:
PortuguesePara,

// Lookup table:
fn portuguese_a_contraction(gender: RomanceGender, plural: RomancePlural) -> &'static str {
    match (gender, plural) {
        (RomanceGender::Masculine, RomancePlural::One) => "ao",
        (RomanceGender::Masculine, RomancePlural::Other) => "aos",
        (RomanceGender::Feminine, RomancePlural::One) => "à",
        (RomanceGender::Feminine, RomancePlural::Other) => "às",
    }
}

// Transform function (follows exact pattern of portuguese_de_transform):
fn portuguese_para_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = resolve_text_with_context(value, context);
    let gender = parse_romance_gender(value, "para")?;
    let plural = parse_romance_plural(context);
    let contracted = portuguese_a_contraction(gender, plural);
    Ok(format!("{} {}", contracted, text))
}

// Registration:
("pt", "para") => Some(TransformKind::PortuguesePara),

// Alias:
("ao", "pt") => "para",
```

**Why not reuse `@a`?** The `@a` alias already maps to `@o` (definite article)
for Portuguese (line 2465). Overloading it for preposition+article would create
ambiguity. The name `@para` is idiomatic Portuguese and unambiguous.

**Why needed for Dreamtides?** The "return to hand" pattern appears in ~8 cost
phrases. With dynamic targets:

```
// pt_br.rlf
return_one($target) = "devolva {@um $target} {@para hand}";
// ally → "devolva um aliado à mão"
// card → "devolva uma carta à mão"
```

Without `@para`, the translator must hardcode "à mão" — which works for the
fixed "hand" location but wouldn't scale to dynamic destinations.

**APPENDIX_STDLIB.md update:** Add `@para` to the Portuguese transforms table:

```
| `@para` | `@ao` | `:masc`, `:fem` | "a" + article (ao/à/aos/às) |
```

---

### 14.2 Add `@por` Transform (Preposition "por" + Article Contraction)

**Priority:** MEDIUM — useful for "by/for each" patterns.

**Problem:** PT-BR contracts "por" (by/for) with definite articles:

| Preposition + Article | Contraction | Example |
|----------------------|-------------|---------|
| por + o | pelo | "pelo custo" (by the cost) |
| por + a | pela | "pela centelha" (by the spark) |
| por + os | pelos | "pelos aliados" (by the allies) |
| por + as | pelas | "pelas cartas" (by the cards) |

**Proposed syntax:**

```
// pt_br.rlf
cost_term = :masc "custo";

by_cost = "{@por cost_term}";              // → "pelo custo"
by_cards = "{@por:other card}";            // → "pelas cartas"
```

**Implementation:** Same pattern as `@de` and `@em`:

```rust
// In TransformKind enum:
PortuguesePor,

// Lookup table:
fn portuguese_por_contraction(gender: RomanceGender, plural: RomancePlural) -> &'static str {
    match (gender, plural) {
        (RomanceGender::Masculine, RomancePlural::One) => "pelo",
        (RomanceGender::Masculine, RomancePlural::Other) => "pelos",
        (RomanceGender::Feminine, RomancePlural::One) => "pela",
        (RomanceGender::Feminine, RomancePlural::Other) => "pelas",
    }
}

// Transform function:
fn portuguese_por_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = resolve_text_with_context(value, context);
    let gender = parse_romance_gender(value, "por")?;
    let plural = parse_romance_plural(context);
    let contracted = portuguese_por_contraction(gender, plural);
    Ok(format!("{} {}", contracted, text))
}

// Registration:
("pt", "por") => Some(TransformKind::PortuguesePor),
```

**Dreamtides usage:** Less frequent than `@para`, but needed for:
- "gain energy equal to that character's cost" → "ganhe energia igual {@por cost_term} daquele personagem"
- Some conditional phrases comparing "by the number of..."

**Practical assessment:** For Dreamtides specifically, `@por` contractions can
usually be hardcoded since they appear with fixed nouns. But including it makes
the stdlib complete and future-proof.

---

### 14.3 Enhance `@um` to Support Plural Context

**Priority:** MEDIUM — needed for plural indefinite articles.

**Problem:** The current `@um` implementation (`portuguese_um_transform` in
`transforms.rs` line 749-754) does **not** accept a context parameter. It always
produces singular articles (um/uma). But PT-BR has plural indefinite articles:

| | Masculine | Feminine |
|---|-----------|----------|
| Singular | um | uma |
| Plural | uns | umas |

Spanish `@un` supports `@un:other` for plural forms (unos/unas). Portuguese
`@um` should too.

**Current behavior:**
```
{@um card}         // → "uma carta" ✓
{@um:other card}   // → ERROR (context not supported)
```

**Proposed behavior:**
```
{@um card}         // → "uma carta" (unchanged)
{@um:other card}   // → "umas cartas"
{@um:other ally}   // → "uns aliados"
```

**Implementation change:** Modify `portuguese_um_transform` to accept context:

```rust
/// Portuguese indefinite article lookup table with plural support.
fn portuguese_indefinite_article(gender: RomanceGender, plural: RomancePlural) -> &'static str {
    match (gender, plural) {
        (RomanceGender::Masculine, RomancePlural::One) => "um",
        (RomanceGender::Masculine, RomancePlural::Other) => "uns",
        (RomanceGender::Feminine, RomancePlural::One) => "uma",
        (RomanceGender::Feminine, RomancePlural::Other) => "umas",
    }
}

fn portuguese_um_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = resolve_text_with_context(value, context);
    let gender = parse_romance_gender(value, "um")?;
    let plural = parse_romance_plural(context);
    let article = portuguese_indefinite_article(gender, plural);
    Ok(format!("{} {}", article, text))
}
```

Also update the dispatch in `apply_transform` to pass context:

```rust
TransformKind::PortugueseUm => portuguese_um_transform(value, context),
// Was: portuguese_um_transform(value)
```

**Dreamtides usage:** Relatively rare — PT-BR plural indefinites ("uns/umas")
are less common in card game text than singular articles. But completeness
matters: the Spanish `@un` already supports `:other`, so Portuguese should match.

**Does this hurt Spanish?** No. `@un` for Spanish is a separate transform
(`SpanishUn`) with its own implementation. Portuguese `@um` is `PortugueseUm`.
They are independent.

---

### 14.4 Do NOT Add a "você" Conjugation Pattern

**Priority:** NOT RECOMMENDED.

**Rationale:** A verb conjugation transform was suggested (question 6). After
analysis, this is unnecessary and would be over-engineering:

1. **PT-BR card text uses a small, fixed set of verbs.** Dreamtides has ~15
   keyword verbs (draw, dissolve, banish, materialize, etc.). Each is already
   defined as its own RLF term with the correct imperative form.

2. **There are no productive verb patterns.** Unlike Russian case declension
   (which applies uniformly to all nouns), Portuguese verb conjugation has
   three conjugation classes (-ar, -er, -ir) with irregular forms. A transform
   would need a full conjugation table, which is massive overkill.

3. **Verbs don't appear as dynamic parameters.** The Rust serializer never
   passes a verb as a `$parameter` to a phrase. Verbs are always known at
   phrase-definition time.

4. **The imperative form is always the same.** Card text uses exactly one verb
   form (the "você" imperative), so there's no need to select among forms.

**Conclusion:** Verb forms are best handled as individual RLF terms, not via a
conjugation transform. This is already the approach in `strings.rs` (separate
`dissolve`, `banish`, `reclaim` terms).

---

### 14.5 Do NOT Change `:from` Behavior for PT-BR vs Spanish

**Priority:** NOT RECOMMENDED.

**Rationale:** Question 4 asked whether `:from` should propagate tags
differently per language. It should NOT:

1. **`:from` already works correctly.** It propagates whatever tags the current
   language's definition has. When running in PT-BR, `subtype(ancient)` reads
   PT-BR's `ancient = :masc { ... }` and propagates `:masc`. When running in
   Spanish, it reads Spanish's `ancient = :masc { ... }` and propagates `:masc`.
   When running in English, it reads `ancient = :an { ... }` and propagates
   `:an`.

2. **Language-specific behavior comes from the definitions, not the mechanism.**
   The power of `:from` is that it's a generic propagation tool. Making it
   language-aware would couple it to specific grammars.

3. **The "complete translations" guarantee (DESIGN.md line 664) ensures
   correctness.** If a PT-BR file defines `ancient` with `:masc`, `:from` gets
   `:masc`. If the translation is missing, it's a `PhraseNotFound` error — it
   never silently falls back to English's `:an` tag.

**Conclusion:** `:from` is language-neutral by design and should stay that way.

---

### 14.6 Do NOT Add Adjective Agreement Transforms

**Priority:** NOT RECOMMENDED.

**Rationale:** Question 7 asked about adjective agreement in composed phrases.
PT-BR adjectives must agree in gender and number with their noun:

| | Masculine | Feminine |
|---|-----------|----------|
| Singular | aliado | aliada |
| Plural | aliados | aliadas |

| | Masculine | Feminine |
|---|-----------|----------|
| Singular | inimigo | inimiga |
| Plural | inimigos | inimigas |

However, this is already handled perfectly by **`:match` on gender tags**:

```
// pt_br.rlf
allied_adj = :match($entity) {
    masc: "aliado",
    *fem: "aliada",
};
allied($entity) = "{$entity:one} {allied_adj:$entity}";
// Ancient (masc) → "Ancião aliado"
// card (fem) → "carta aliada"
```

A transform like `@agree` would add framework complexity for something `:match`
already handles. The Russian walkthrough uses the same `:match` pattern for
adjective agreement (e.g., `allied_adj`, `enemy_adj`, `another_adj`,
`each_adj`) and it works cleanly.

**Conclusion:** `:match` is the correct tool for adjective agreement. No new
transform needed.

---

### 14.7 Add `@de` Bare Preposition Mode (Without Article)

**Priority:** LOW — nice-to-have for completeness.

**Problem:** The current `@de` always contracts with an article:
`{@de void}` → "do vazio" (de + o). But sometimes PT-BR needs "de" without
contraction, when there's no article:

| With article | Without article |
|--------------|-----------------|
| "do vazio" (from THE void) | "de seu vazio" (from YOUR void) |
| "da mão" (from THE hand) | "de minha mão" (from MY hand) |

When a possessive precedes the noun, there's no definite article, so no
contraction occurs. Currently, translators handle this by writing "de" directly
in the template string:

```
// pt_br.rlf
banish_from_your_void($n) = "{banish} {cards($n)} de seu vazio";
// "de" is a literal string, not a transform
```

**Assessment:** This is fine for Dreamtides. The cases where "de" appears
without contraction are always with possessives, which are static in the
template. No transform change needed.

**If desired later:** A context like `@de:bare` could emit just "de " without
contraction. But this adds complexity for a pattern that's adequately handled by
literal text.

**Conclusion:** NOT RECOMMENDED for now. Revisit if dynamic possessive+noun
combinations become common.

---

### 14.8 Summary of RLF Framework Changes

| Change | Priority | Effort | Benefit |
|--------|----------|--------|---------|
| Add `@para` (a+article contraction) | HIGH | Small (~30 lines) | Enables dynamic "to X" phrases |
| Add `@por` (por+article contraction) | MEDIUM | Small (~30 lines) | Completes contraction set |
| Enhance `@um` with plural context | MEDIUM | Tiny (~10 lines) | Enables "uns/umas" plural articles |
| Add `@de:bare` mode | LOW | Small (~15 lines) | Marginal: "de" without contraction |
| Add verb conjugation transform | NOT REC. | Large | Over-engineering for 15 fixed verbs |
| Change `:from` per language | NOT REC. | — | Already works correctly |
| Add adjective agreement transform | NOT REC. | — | `:match` handles this perfectly |

**Total implementation cost:** ~70 lines of Rust in `transforms.rs` for the
HIGH and MEDIUM items, following the exact patterns already established by
`@de` and `@em`. Plus ~5 lines of registration in the transform dispatch table
and alias table. Minimal risk since the pattern is well-established.

**Impact on other languages:** Zero. All changes are Portuguese-only transforms
registered under the `"pt"` language code. Spanish, German, Russian, Chinese
transforms are completely independent.

---

### 14.9 Complete PT-BR Transform Table (Post-Changes)

After implementing recommendations 14.1–14.3, the Portuguese stdlib would be:

| Transform | Aliases | Reads | Context | Effect |
|-----------|---------|-------|---------|--------|
| `@o` | `@a` | `:masc`, `:fem` | `:one`/`:other` | Definite article (o/a/os/as) |
| `@um` | `@uma` | `:masc`, `:fem` | `:one`/`:other` | Indefinite article (um/uma/uns/umas) |
| `@de` | - | `:masc`, `:fem` | `:one`/`:other` | "de" + article (do/da/dos/das) |
| `@em` | - | `:masc`, `:fem` | `:one`/`:other` | "em" + article (no/na/nos/nas) |
| `@para` | `@ao` | `:masc`, `:fem` | `:one`/`:other` | "a" + article (ao/à/aos/às) |
| `@por` | - | `:masc`, `:fem` | `:one`/`:other` | "por" + article (pelo/pela/pelos/pelas) |

This gives PT-BR translators the complete set of preposition+article contraction
tools needed for natural card game text, matching the coverage that French gets
with `@de`/`@au` and Italian gets with `@di`/`@a`.
