# I18n Specialist Review: Serializer RLF Migration Plan

**Reviewer:** I18n Specialist
**Date:** 2026-02-07
**Verdict:** Phase 1 is structurally sound for English-only extraction but will create significant rework for multilingual support. Several design assumptions bake in English word order and prevent grammatical agreement required by 4 of the 6 target languages.

---

## Methodology

I selected 5 real serializer outputs that stress different i18n failure modes, then worked through what each must look like in all 6 target languages. I used TCG localization conventions from Magic: The Gathering and similar games as reference points for domain terminology.

---

## Terminology Reference

Dreamtides-specific terms and their likely translations (following MTG conventions for card game terminology):

| English | Chinese (ZH) | Russian (RU) | Spanish (ES) | Portuguese-BR (PT) | German (DE) |
|---------|-------------|-------------|-------------|-------------------|-------------|
| dissolve | 消融 | растворить | disolver | dissolver | auflösen |
| materialize | 具现 | материализовать | materializar | materializar | materialisieren |
| banish | 放逐 | изгнать | desterrar | banir | verbannen |
| reclaim | 回收 | вернуть | reclamar | reclamar | zurückfordern |
| foresee | 预见 | предвидеть | prever | prever | vorhersehen |
| kindle | 燃魂 | разжечь | avivar | avivar | entfachen |
| prevent | 阻止 | предотвратить | prevenir | prevenir | verhindern |
| ally | 友军 | союзник (m) | aliado (m) | aliado (m) | Verbündeter (m) |
| enemy | 敌军 | враг (m) | enemigo (m) | inimigo (m) | Feind (m) |
| character | 角色 | персонаж (m) | personaje (m) | personagem (m/f) | Charakter (m) |
| event | 事件 | событие (n) | evento (m) | evento (m) | Ereignis (n) |
| card | 牌 | карта (f) | carta (f) | carta (f) | Karte (f) |
| spark | 灵火 | искра (f) | chispa (f) | centelha (f) | Funke (m) |
| void | 虚境 | пустота (f) | vacío (m) | vazio (m) | Leere (f) |
| energy | 能量 | энергия (f) | energía (f) | energia (f) | Energie (f) |

Note: Russian nouns have grammatical gender (m/f/n) AND case declension (6 cases). German has gender (m/f/n) AND case (4 cases). Spanish/Portuguese have gender (m/f). Chinese has none of these but uses classifiers.

---

## Example 1: `dissolve_target` — "dissolve an enemy."

**Source:** `effect_serializer.rs:225-228`
```rust
StandardEffect::DissolveCharacter { target } => {
    format!("{dissolve} {}.", predicate_serializer::serialize_predicate(target, bindings))
}
```

**Plan's proposed phrase:**
```
dissolve_target($target) = "{{dissolve}} {$target}.";
```

### Required output per language

Suppose `$target` = Predicate::Enemy(CardPredicate::Character), so in English this is "an enemy":

| Language | Required Text | Notes |
|----------|--------------|-------|
| EN | dissolve an enemy. | Works as-is |
| ZH | 消融一个敌军。 | Classifier "个" required before "敌军". Word order: verb + classifier + noun |
| RU | Растворить врага. | "враг" (nominative) → "врага" (accusative). The verb's object MUST be accusative case. |
| ES | Disolver a un enemigo. | Spanish requires personal "a" before animate objects: "disolver **a** un enemigo" |
| PT | Dissolver um inimigo. | Article agrees with gender of target noun |
| DE | Einen Feind auflösen. | Accusative case: "ein Feind" → "**einen** Feind". Verb goes to END in infinitive constructions. |

### Analysis

**The `$target` parameter is a pre-rendered string.** When the plan calls `dissolve_target(target_text)`, `target_text` is already "an enemy" — a fully baked English string. This is catastrophic for:

1. **Russian:** The predicate serializer returns "враг" (nominative). But dissolve's object needs accusative "врага". The dissolve_target phrase has NO way to inflect the target because it received a `String`, not a `Phrase` with case variants.

2. **German:** "ein Feind" (nominative) must become "einen Feind" (accusative). Same problem — the article form depends on the grammatical role in the sentence, which is determined by the *outer* phrase (dissolve), not the *inner* phrase (predicate).

3. **Spanish:** "un enemigo" must become "a un enemigo" (personal "a" before animate direct objects). The dissolve phrase needs to know the target is animate to insert "a".

4. **Chinese:** "一个敌军" requires a classifier "个" between the number and the noun. The classifier depends on the *type* of noun. If `$target` is pre-rendered, no classifier can be inserted.

**Verdict:** The `$target`-as-String design **fundamentally cannot work** for Russian, German, Spanish, or Chinese. Phase 2 (Phrase-based composition) is not optional — it is **mandatory** for any language with case marking, article declension, or classifiers.

---

## Example 2: `banish_cards_from_void` — "{Banish} 3 cards from your void"

**Source:** `cost_serializer.rs:43-44`
```rust
Cost::BanishCardsFromYourVoid(count) => {
    bindings.insert("c".to_string(), VariableValue::Integer(*count));
    "{Banish} {cards($c)} from your void".to_string()
}
```

**Plan's proposed phrase:**
```
banish_cards_from_void($c) = "{{Banish}} {{cards($c)}} from your void";
```

### Required output per language

With count = 3:

| Language | Required Text |
|----------|--------------|
| EN | Banish 3 cards from your void |
| ZH | 从你的虚境中放逐三张牌 | "from your void" comes FIRST. Classifier 张 for cards. |
| RU | Изгнать 3 карты из вашей пустоты | "карты" (accusative plural). "из вашей пустоты" (genitive for "from your void") |
| ES | Desterrar 3 cartas de tu vacío | Straightforward with "de tu vacío" |
| PT | Banir 3 cartas do seu vazio | "do" = contraction of "de" + "o" |
| DE | 3 Karten aus deiner Leere verbannen | Object-verb order. "deiner Leere" (dative for "from your void") |

### Analysis

**The escaped braces approach `{{Banish}} {{cards($c)}}` creates an untranslatable pattern.** Here's why:

The phrase output is: `{Banish} {cards($c)} from your void` — literal template text. A translator seeing this in a `.rlf` file would need to rearrange these escaped directive fragments. But `{{Banish}}` and `{{cards($c)}}` are opaque tokens to the translator — they don't know what they expand to, and they cannot:

1. **Reorder:** Chinese needs `从你的虚境中{{Banish}}{{cards($c)}}` — the prepositional phrase must move to the front.
2. **Inflect the noun inside `{{cards($c)}}`:** Russian needs accusative "карты" but `cards()` only knows about English plurals.
3. **Contract prepositions:** Portuguese "de o" → "do" depends on the gender of the following noun (vacío/vazio is masculine → "do").

**The double-brace escaping is the core design problem.** By deferring evaluation of `{Banish}` and `{cards($c)}` to a second pass (via `eval_str`), the plan creates a two-layer template system where:
- Layer 1 (RLF phrase): arranges escaped directives in English word order
- Layer 2 (eval_str): evaluates the directives

A translator can rearrange Layer 1 tokens, but they **cannot modify Layer 2 directives.** The `{cards($c)}` directive is English-only — it produces "a card" / "3 cards". For Russian, you'd need `{карты($c)}` which doesn't exist. For Chinese, you'd need `{张牌($c)}`.

**This means Phase 1's escaped-brace phrases are English-only by construction.** They cannot be translated. When Phase 2 replaces them with real RLF phrase composition (`{banish} {cards($c)}` with actual RLF resolution), every single phrase with `{{...}}` will need to be rewritten. This is the definition of throwaway work.

---

## Example 3: `when_dissolved` — "when this character is dissolved, draw a card."

**Source:** `trigger_serializer.rs:63-67`
```rust
TriggerEvent::Dissolved(predicate) => {
    format!("when {} is {{dissolved}}, ",
        predicate_serializer::serialize_predicate(predicate, bindings))
}
```

**Plan's proposed phrase:**
```
when_dissolved($target) = "when {$target} is {{dissolved}}, ";
```

### Required output per language

With target = Predicate::This ("this character"):

| Language | Required Text |
|----------|--------------|
| EN | when this character is dissolved, |
| ZH | 当此角色被消融时， | Topic-comment: "当...时" frame wraps the whole clause. No passive marker needed or different one (被). |
| RU | когда этот персонаж растворён, | Past passive participle "растворён" must agree with subject gender. "персонаж" is masculine → "растворён". If subject were "карта" (f), it would be "растворена". |
| ES | cuando este personaje es disuelto, | Participle "disuelto" agrees with masculine subject. For feminine: "disuelta". |
| PT | quando este personagem é dissolvido, | Same gender agreement: "dissolvido" (m) / "dissolvida" (f). |
| DE | wenn dieser Charakter aufgelöst wird, | "dieser" must agree with gender/case. Verb "aufgelöst wird" at the end (subordinate clause). |

### Analysis

**Passive voice constructions require gender agreement on the participle.** "is dissolved" in English is gender-neutral, but in Russian/Spanish/Portuguese/German, the participle must agree with the subject:

- RU: растворён (m) / растворена (f) / растворено (n)
- ES: disuelto (m) / disuelta (f)
- PT: dissolvido (m) / dissolvida (f)
- DE: aufgelöst (invariant, but article "dieser/diese/dieses" must agree)

Since `$target` is a pre-rendered string, the phrase has **no way to know the gender of the target** to select the correct participle form. This is not a problem `eval_str` can solve — it's a structural issue with how the predicate and trigger are composed.

**RLF's `:match` on tags could solve this** — if `$target` were a `Phrase` with gender tags (`:masc`, `:fem`, `:neut`), then:
```
// Russian translation
when_dissolved($target) = :match($target) {
    masc: "когда {$target:nom} растворён, ",
    fem: "когда {$target:nom} растворена, ",
    *neut: "когда {$target:nom} растворено, ",
};
```

But this requires Phase 2 (passing Phrase, not String). With Phase 1's string approach, this is impossible.

---

## Example 4: Complex composition — "you may pay 3● to dissolve an enemy with cost 2● or less."

**Source:** `effect_serializer.rs:730-758` (serialize_effect_with_context for Effect::WithOptions)

This combines: optional ("you may") + trigger cost ("pay 3●") + action ("dissolve") + complex predicate ("an enemy with cost 2● or less").

### Required output per language

| Language | Required Text |
|----------|--------------|
| EN | you may pay 3● to dissolve an enemy with cost 2● or less. |
| ZH | 你可以支付3●来消融一个费用为2●或更少的敌军。 | "费用为2●或更少的" is a relative clause BEFORE "敌军". "来" connects cost to action. |
| RU | вы можете заплатить 3● чтобы растворить врага со стоимостью 2● или менее. | "врага" (accusative). "со стоимостью" (instrumental case for "with cost"). |
| ES | puedes pagar 3● para disolver a un enemigo con coste de 2● o menos. | "a un enemigo" (personal a). "con coste de" (prepositional phrase). |
| PT | você pode pagar 3● para dissolver um inimigo com custo de 2● ou menos. | |
| DE | du darfst 3● bezahlen, um einen Feind mit Kosten von 2● oder weniger aufzulösen. | "einen Feind" (accusative). "aufzulösen" (zu-infinitive at end). Comma before "um...zu". |

### Analysis

This example compounds all previous problems PLUS adds structural composition issues:

1. **The "you may X to Y" frame** has language-specific connectors:
   - EN: "you may [cost] to [action]"
   - ZH: "你可以[cost]来[action]" (来 connects purpose)
   - RU: "вы можете [cost] чтобы [action]" (чтобы + infinitive)
   - DE: "[cost], um [action]" (um...zu construction, verb at end)

2. **The plan's structural phrases** (`you_may_prefix`, `cost_to_connector($cost)`) lock in English word order:
   ```
   you_may_prefix = "you may ";
   cost_to_connector($cost) = "{$cost} to ";
   ```
   These assume: `[optional] [cost] to [action]` order. German requires: `[optional] [cost], um [action]zu[verb]`. Chinese requires: `[optional] [cost]来[action]`. The "to" connector is built into a phrase that receives pre-rendered strings.

3. **The predicate "an enemy with cost 2● or less"** is built by recursive composition in `serialize_card_predicate`. The plan proposes:
   ```
   with_cost($target, $op) = "{$target} with cost {{energy($e)}}{$op}";
   ```
   But in Chinese, the cost constraint goes BEFORE the noun: "费用为2●或更少的敌军". This is a fundamental word-order reversal that a translator cannot achieve by rearranging `{$target}` and the rest of the string — because `$target` already contains "enemy" and the qualifier must wrap around it as a prenominal modifier.

**This demonstrates that English's post-nominal modification ("enemy WITH cost...") vs. Chinese/German/Japanese prenominal modification ("cost-2-or-less enemy") makes the plan's phrase structure untranslatable in its current form.**

---

## Example 5: Collection with void targeting — "up to 3 characters in your void gain reclaim equal to their cost."

**Source:** `effect_serializer.rs:1164-1171`
```rust
CollectionExpression::UpTo(n) => format!(
    "up to {} {} in your void gain {}{}{}.",
    n,
    predicate_serializer::serialize_card_predicate_plural(predicate, bindings),
    reclaim_directive_plural, reclaim_suffix_plural, this_turn_suffix
)
```

### Required output per language

| Language | Required Text |
|----------|--------------|
| EN | up to 3 characters in your void gain reclaim equal to their cost. |
| ZH | 你虚境中至多三个角色获得回收，其费用等于其法力费用。 | "你虚境中" (location) comes first. 个 classifier. "至多" = "up to". |
| RU | до 3 персонажей в вашей пустоте получают возврат, равный их стоимости. | "персонажей" (genitive plural after numerals). "в вашей пустоте" (prepositional case). "равный" agrees with "возврат" (m). |
| ES | hasta 3 personajes en tu vacío obtienen reclamar igual a su coste. | |
| PT | até 3 personagens no seu vazio ganham reclamar igual ao custo deles. | "ao" = contraction "a" + "o". "deles" = possessive for "their". |
| DE | bis zu 3 Charaktere in deiner Leere erhalten Zurückfordern in Höhe ihrer Kosten. | "deiner Leere" (dative). "ihrer Kosten" (genitive possessive). |

### Analysis

This is one of the most complex serializer outputs, combining:
- Collection quantifier ("up to 3")
- Plural predicate ("characters")
- Location qualifier ("in your void")
- Keyword ability ("reclaim")
- Relative clause ("equal to their cost")

**Problems with the plan's approach:**

1. **"their" pronoun agreement:** "equal to their cost" — in Russian, the possessive "их" is invariant, but "равный" (equal) must agree with "возврат" (reclaim, masculine). In German, "ihrer" must agree with the noun it modifies ("Kosten" is plural). The plan has no mechanism for this.

2. **Numeral + noun agreement:** Russian requires different noun forms after different numbers: "1 персонаж" (nominative singular), "2-4 персонажа" (genitive singular), "5+ персонажей" (genitive plural). RLF's CLDR plural rules handle this IF the noun is an RLF term with proper variants — but the plan passes pre-rendered plural strings, not Phrase values.

3. **The plan splits this into multiple concatenated strings:** Looking at the serializer code, this is built from `"up to {} {} in your void gain {}{}{}"` with 5 separate interpolations. The plan would turn each piece into phrases, but the overall sentence structure must be a single phrase for a translator to rearrange word order.

---

## Systematic Findings

### Finding 1: Phase 1 escaped-brace phrases are English-only and will be rewritten

Every phrase using `{{directive}}` syntax creates a template-within-a-template that:
- Cannot be translated (the inner directives are English-specific)
- Cannot be reordered meaningfully (the directive tokens are opaque)
- Will be replaced in Phase 2 when directives become real RLF calls

**Count of affected phrases in the plan:** ~80% of all proposed phrases contain `{{...}}` escapes. This means ~80% of Phase 1 work is throwaway.

### Finding 2: `$target` as pre-rendered String prevents agreement in 4/6 languages

| Agreement type | Languages affected | RLF feature needed |
|---------------|-------------------|-------------------|
| Case declension | Russian, German | Pass `Phrase` with case variants, select via `:acc`, `:gen`, etc. |
| Gender agreement on participles | Russian, Spanish, Portuguese, German | Pass `Phrase` with gender tags, use `:match` |
| Personal "a" (animate objects) | Spanish | Pass `Phrase` with `:anim` tag, use `@a` transform variant |
| Classifier insertion | Chinese | Pass `Phrase` with measure word tags, use `@count` |
| Article declension | German | Pass `Phrase` with gender tags, use `@der`/`@ein` with case context |
| Possessive contraction | Portuguese | Know gender of following noun for "de+o→do", "de+a→da" |

**None of these can be solved while `$target` is a `String`.** Phase 2 is mandatory.

### Finding 3: Word order assumptions baked into phrase structure

The plan's phrases assume English SVO word order with post-nominal modification:

| Pattern | English | Counter-example |
|---------|---------|-----------------|
| `verb $target` | "dissolve an enemy" | DE: "einen Feind auflösen" (object-verb) |
| `$target with cost X` | "enemy with cost 3●" | ZH: "费用为3●的敌军" (pre-nominal) |
| `verb $n $target from location` | "banish 3 cards from void" | ZH: "从虚境放逐三张牌" (location-verb-object) |
| `$target gains $keyword` | "it gains reclaim" | DE: "es erhält Zurückfordern" (works, but "it" must agree) |
| `when $target is $participle` | "when it is dissolved" | RU: passive participle agrees with gender |

**A translator cannot fix word order by rearranging `{$target}` inside a phrase** when the target is a pre-rendered string, because:
- They may need to split the target (put qualifier before and noun after)
- They may need to add case markers or prepositions adjacent to the target
- The target itself may need different forms in different positions

### Finding 4: Specific RLF features that MUST exist before migration makes sense

Based on the RLF DESIGN.md, the following features **already exist** and are sufficient for Phase 2:

| Feature | Status | Needed for |
|---------|--------|-----------|
| `:from($param)` tag inheritance | Exists | Propagating gender/animacy from predicate terms |
| `:match($param)` tag branching | Exists | Gender-based participle selection |
| Multi-dimensional variants | Exists | Case × number (e.g., `nom.one`, `acc.many`) |
| `@der`/`@ein` German articles | Exists | German article declension |
| `@el`/`@un` Spanish articles | Exists | Spanish article agreement |
| `@count` classifier transform | Exists | Chinese measure words |
| `@a` English article | Exists | English a/an selection |

**Missing features / unclear:**

| Feature | Status | Needed for |
|---------|--------|-----------|
| Spanish personal "a" transform | Not mentioned in DESIGN.md | "disolver **a** un enemigo" |
| Portuguese contractions (de+o→do) | Not mentioned | "do seu vazio" |
| Phrase-level case propagation | Unclear | Telling an inner phrase "you are in accusative context" |
| Wildcard fallback for multi-dim variants | Exists | Reducing translation burden for languages without all distinctions |

The critical missing piece is **not an RLF feature** — it's the **serializer architecture**. The serializer must pass `Phrase` values (not `String`) so RLF's existing features can work. This is Phase 2 of the plan.

### Finding 5: Phase 1 is NOT throw-away IF scoped correctly

Phase 1 could be valuable if it is limited to:

1. **Phrases that are self-contained English text without `$target` parameters:**
   - `discard_your_hand = "discard your hand";` — This is a complete sentence fragment. Translators can replace it entirely.
   - `take_extra_turn = "take an extra turn after this one.";` — Complete, no parameters.
   - `at_end_of_your_turn = "at the end of your turn, ";` — Complete temporal phrase.

2. **Phrases whose parameters are only numbers:**
   - `draw_cards_effect($c) = "draw {{cards($c)}}.";` — The `$c` is a number. **BUT** the `{{cards($c)}}` is an escaped directive, which makes this still English-only (see Finding 1).

   Actually, if `cards` were defined as a real RLF phrase (which it is!), then the correct Phase 1 approach would be:
   ```
   draw_cards_effect($c) = "draw {cards($c)}.";
   ```
   Without the double braces. This would produce evaluated text ("draw a card." / "draw 3 cards."), which breaks the current round-trip test design but would be translatable.

3. **Naming and semantic cataloging:** The most durable value of Phase 1 is giving every serializer output a **named semantic identity** (e.g., `dissolve_target`, `when_dissolved`). Even if the phrase bodies are rewritten for Phase 2, the names and parameter signatures persist.

### Finding 6: The round-trip constraint forces the escaped-brace design

The plan correctly identifies that the parser expects `{Banish}` as input, not `<color=#AA00FF>Banish</color>`. This round-trip requirement is what forces the `{{Banish}}` escaped-brace approach.

**This is the fundamental design tension:** The serializer must produce template text for the parser, but RLF naturally produces evaluated text. The plan resolves this by making RLF output template text (via escaping), but this makes the RLF phrases untranslatable.

**The real solution** (which the plan defers to Phase 2) is to decouple the parser round-trip path from the display path:
- The serializer produces `Phrase` values for display (translatable)
- A separate "template serializer" or the existing code produces template text for round-trip testing
- Round-trip tests compare at the AST level, not the string level

Until this decoupling happens, the escaped-brace approach is a workaround that provides English-only value.

---

## Recommendations

### Recommendation 1: Reframe Phase 1 as "semantic cataloging" not "localization"

Phase 1 should be honest about its scope: it extracts English strings into named phrases for code organization and semantic clarity, **not** for translation readiness. This is still valuable — it creates the phrase namespace that Phase 2 will fill with real multilingual content.

### Recommendation 2: Prioritize Phase 2 for any phrase taking `$target`

Any phrase that receives a predicate (`$target`) as a string is structurally untranslatable. The Phase 1 version of these phrases will be fully rewritten in Phase 2. Consider:
- Doing Phase 1 only for self-contained phrases (no `$target`)
- Jumping directly to Phase 2 for target-bearing phrases

### Recommendation 3: Decouple round-trip testing from display serialization

The round-trip constraint is the single biggest obstacle to correct i18n. Consider:
1. Round-trip tests compare at the **AST level** (parse → serialize → compare AST, not string)
2. OR create a separate `to_template_string()` method for round-trip that is not used for display
3. OR accept that round-trip tests only validate English and add separate per-language display tests

### Recommendation 4: Define term gender/case variants NOW

Even during Phase 1, the RLF term definitions in `strings.rs` should be enriched with gender tags and case variants. This is zero-risk work that enables Phase 2:

```rust
// Current (English only):
ally = :an { one: "ally", other: "allies" };

// Enriched (ready for Russian translation file to reference):
ally = :an :anim { one: "ally", other: "allies" };
```

The `:anim` tag doesn't affect English output but signals to translators that this term needs animate-object handling. Similarly, translation files can define:
```
// ru.rlf
ally = :masc :anim {
    nom.one: "союзник", nom.few: "союзника", nom.many: "союзников",
    acc.one: "союзника", acc.few: "союзников", acc.many: "союзников",
    gen.one: "союзника", gen.few: "союзников", gen.many: "союзников",
    // ... etc
};
```

### Recommendation 5: Avoid `{{escaped_directive}}` — use real RLF calls or plain strings

Every `{{...}}` escape creates a phrase that looks translatable but isn't. Instead:
- For keywords like `{Banish}`: create a RLF term (`banish_keyword`) and use `{Banish_keyword}` directly. Yes, this produces evaluated text, which breaks round-trip tests — but that's the correct forcing function to fix the test architecture.
- For counted phrases like `{cards($c)}`: use the existing `cards` RLF phrase directly. The RLF phrase IS the single source of truth for "how to express N cards."

### Recommendation 6: Plan for Chinese classifier complexity

Chinese is uniquely challenging because:
1. Every noun requires a classifier (measure word) before it: 一**张**牌, 一**个**角色
2. The classifier depends on the noun, not the number
3. RLF's `@count` transform handles this, but only if terms have classifier tags

The plan should ensure all card game terms get classifier tags in their Chinese definitions:
```
// zh.rlf
card = :zhang { one: "牌", other: "牌" };  // 张 classifier
character = :ge { one: "角色", other: "角色" };  // 个 classifier
```

---

## Summary Verdict

| Aspect | Rating | Comment |
|--------|--------|---------|
| English extraction value | Good | Named phrases improve code organization |
| Multilingual readiness | Poor | Pre-rendered strings and escaped braces prevent translation |
| Phase 1 → Phase 2 continuity | Low | ~80% of phrase bodies will be rewritten |
| RLF feature coverage | Good | RLF already has the features needed for Phase 2 |
| Architectural risk | Medium | Round-trip test coupling is the key constraint |

**Bottom line:** Phase 1 is useful for English code organization but should not be marketed as "localization-ready." For actual multilingual support, Phase 2 (Phrase-based composition) and round-trip test decoupling are prerequisites, not nice-to-haves. The plan should acknowledge this explicitly and consider whether the Phase 1 → Phase 2 rewrite cost is acceptable, or whether jumping to Phase 2 for target-bearing phrases would be more efficient.
