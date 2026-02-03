# Appendix: Localizing Cost Serializer for Russian

This appendix analyzes localizing `cost_serializer.rs` to Russian using Phraselet, focusing on case agreement challenges.

## The Core Challenge: Case Governance

In English, cost phrases are straightforward:
- "abandon an ally"
- "return a character to hand"
- "{Banish} a card from your void"

In Russian, verbs **govern** the case of their objects:
- "пожертвовать союзника" (abandon ally-ACC)
- "вернуть персонажа в руку" (return character-ACC to hand)
- "изгнать карту из вашей пустоты" (banish card-ACC from your-GEN void-GEN)

The predicate must appear in the **accusative case** after these verbs. Prepositions like "из" (from) require **genitive case** for their objects.

---

## Design Principle: Case as a Serialization Parameter

The current Rust code:

```rust
format!("abandon {}", predicate_serializer::serialize_predicate(target, bindings))
```

For Russian, the serializer must know which case to use. This is **Rust logic**, not Phraselet text:

```rust
ru::abandon(predicate_serializer::serialize_predicate(target, Case::Acc, bindings))
```

The predicate serializer becomes case-aware and calls appropriate Phraselet functions.

---

## English Phraselet File

```rust
// en.phr.rs
phraselet! {
    //==========================================================================
    // VERBS / ACTIONS
    //==========================================================================

    abandon(target) = "abandon {target}";
    abandon_any_number(target) = "abandon any number of {target}";
    abandon_count(count) = "abandon {count}";

    discard(count) = "discard {count}";
    discard_hand = "discard your hand";

    return_to_hand(target) = "return {target} to hand";
    return_n_to_hand(n, target) = "return {n} {target} to hand";
    return_all_to_hand(target) = "return all {target} to hand";
    return_all_but_one_to_hand(target) = "return all but one {target} to hand";
    return_any_number_to_hand(target) = "return any number of {target} to hand";
    return_up_to_to_hand(n, target) = "return up to {n} {target} to hand";
    return_each_other_to_hand(target) = "return each other {target} to hand";
    return_n_or_more_to_hand(n, target) = "return {n} or more {target} to hand";

    //==========================================================================
    // BANISH PATTERNS
    //==========================================================================

    banish_another_from_void = "{banish} another card in your void";
    banish_n_from_void(n) = "{banish} {n} from your void";
    banish_n_from_enemy_void(n) = "{banish} {n} from the opponent's void";
    banish_void_with_min(n) = "{banish} your void with {n} or more cards";
    banish_from_hand(target) = "{banish} {target} from hand";
    banish_your_void = "{banish} your void";

    //==========================================================================
    // ENERGY / RESOURCES
    //==========================================================================

    energy_cost(e) = "{e}";
    lose_max_energy(n) = "lose {n}";
    pay_one_or_more_energy = "pay 1 or more {energy_symbol}";
    pay(cost) = "pay {cost}";

    //==========================================================================
    // COMBINATORS
    //==========================================================================

    choice_separator = " or ";
    list_separator = " and ";

    //==========================================================================
    // KEYWORDS
    //==========================================================================

    banish = "{Banish}";
    energy_symbol = "{energy-symbol}";
}
```

---

## Russian Phraselet File

```rust
// ru.phr.rs
phraselet! {
    //==========================================================================
    // VERBS / ACTIONS
    // Russian verbs govern accusative case for direct objects
    //==========================================================================

    // "пожертвовать" (sacrifice/abandon) + accusative
    abandon(target) = "пожертвовать {target}";
    abandon_any_number(target) = "пожертвовать любое количество {target}";
    abandon_count(count) = "пожертвовать {count}";

    // "сбросить" (discard) + accusative
    discard(count) = "сбросить {count}";
    discard_hand = "сбросить руку";

    // "вернуть в руку" (return to hand) + accusative
    return_to_hand(target) = "вернуть {target} в руку";
    return_n_to_hand(n, target) = "вернуть {n} {target} в руку";
    return_all_to_hand(target) = "вернуть всех {target} в руку";
    return_all_but_one_to_hand(target) = "вернуть всех {target} кроме одного в руку";
    return_any_number_to_hand(target) = "вернуть любое количество {target} в руку";
    return_up_to_to_hand(n, target) = "вернуть до {n} {target} в руку";
    return_each_other_to_hand(target) = "вернуть каждого другого {target} в руку";
    return_n_or_more_to_hand(n, target) = "вернуть {n} или больше {target} в руку";

    //==========================================================================
    // BANISH PATTERNS
    // "изгнать" (banish) + accusative; "из" (from) + genitive
    //==========================================================================

    banish_another_from_void = "{banish} ещё одну карту из вашей пустоты";
    banish_n_from_void(n) = "{banish} {n} из вашей пустоты";
    banish_n_from_enemy_void(n) = "{banish} {n} из пустоты противника";
    banish_void_with_min(n) = "{banish} вашу пустоту с {n} или больше картами";
    banish_from_hand(target) = "{banish} {target} из руки";
    banish_your_void = "{banish} вашу пустоту";

    //==========================================================================
    // ENERGY / RESOURCES
    //==========================================================================

    energy_cost(e) = "{e}";
    lose_max_energy(n) = "потерять {n}";
    pay_one_or_more_energy = "заплатить 1 или больше {energy_symbol}";
    pay(cost) = "заплатить {cost}";

    //==========================================================================
    // COMBINATORS
    //==========================================================================

    choice_separator = " или ";
    list_separator = " и ";

    //==========================================================================
    // KEYWORDS
    //==========================================================================

    banish = "{Изгнание}";
    energy_symbol = "{символ-энергии}";
}
```

---

## Design Issue Discovered: Case-Aware Predicate Serialization

The `target` parameter in phrases like `abandon(target)` receives pre-rendered text. For Russian, this text must already be in the correct case.

**Solution:** The predicate serializer accepts a grammatical context parameter:

```rust
// predicate_serializer.rs (language-agnostic interface)

pub enum GrammaticalCase {
    Nominative,  // subject
    Accusative,  // direct object
    Genitive,    // possession, "of", after "из"
    Dative,      // indirect object
    // etc.
}

pub fn serialize_predicate(
    pred: &Predicate,
    case: GrammaticalCase,
    bindings: &mut VariableBindings,
) -> String {
    // Dispatch to language-specific serializer
}
```

For English, the case parameter is ignored (English doesn't inflect for case). For Russian, it selects the appropriate variant.

---

## Russian Predicate Phrases with Cases

```rust
// ru.phr.rs (predicate section)
phraselet! {
    //==========================================================================
    // BASE TYPES WITH CASE + NUMBER
    //==========================================================================

    card: {
        nom.one = "карта",
        nom.few = "карты",
        nom.many = "карт",
        acc.one = "карту",
        acc.few = "карты",
        acc.many = "карт",
        gen.one = "карты",
        gen.few = "карт",
        gen.many = "карт",
    };

    character: {
        nom.one = "персонаж",
        nom.few = "персонажа",
        nom.many = "персонажей",
        acc.one = "персонажа",
        acc.few = "персонажей",
        acc.many = "персонажей",
        gen.one = "персонажа",
        gen.few = "персонажей",
        gen.many = "персонажей",
    };

    ally: {
        nom.one = "союзник",
        nom.few = "союзника",
        nom.many = "союзников",
        acc.one = "союзника",
        acc.few = "союзников",
        acc.many = "союзников",
        gen.one = "союзника",
        gen.few = "союзников",
        gen.many = "союзников",
    };

    enemy: {
        nom.one = "враг",
        nom.few = "врага",
        nom.many = "врагов",
        acc.one = "врага",
        acc.few = "врагов",
        acc.many = "врагов",
        gen.one = "врага",
        gen.few = "врагов",
        gen.many = "врагов",
    };
}
```

---

## Rust Serializer Using Phraselet

```rust
// cost_serializer.rs (refactored for localization)

use phraselet::{Language, ru, en};

pub fn serialize_cost(cost: &Cost, lang: Language, bindings: &mut VariableBindings) -> String {
    match lang {
        Language::En => serialize_cost_en(cost, bindings),
        Language::Ru => serialize_cost_ru(cost, bindings),
        // ...
    }
}

fn serialize_cost_ru(cost: &Cost, bindings: &mut VariableBindings) -> String {
    match cost {
        Cost::AbandonCharactersCount { target, count } => match count {
            CollectionExpression::AnyNumberOf => {
                // "любое количество" requires genitive plural
                ru::abandon_any_number(
                    predicate_serializer::serialize_predicate_ru(
                        target,
                        Case::Gen,
                        Number::Many,
                        bindings
                    )
                )
            }
            CollectionExpression::Exactly(1) => {
                // Direct object requires accusative singular
                ru::abandon(
                    predicate_serializer::serialize_predicate_ru(
                        target,
                        Case::Acc,
                        Number::One,
                        bindings
                    )
                )
            }
            CollectionExpression::Exactly(n) => {
                // Number determines plural category
                let num_cat = russian_plural_category(*n);
                bindings.insert("count-allies", VariableValue::Integer(*n));
                ru::abandon_count("{count-allies}")
            }
            // ...
        },

        Cost::ReturnToHand { target, count } => match count {
            CollectionExpression::Exactly(1) => {
                ru::return_to_hand(
                    predicate_serializer::serialize_predicate_ru(
                        target,
                        Case::Acc,
                        Number::One,
                        bindings
                    )
                )
            }
            CollectionExpression::All => {
                // "всех" (all) requires genitive plural
                ru::return_all_to_hand(
                    predicate_serializer::serialize_predicate_ru(
                        target,
                        Case::Gen,
                        Number::Many,
                        bindings
                    )
                )
            }
            // ...
        },

        Cost::BanishFromHand(predicate) => {
            // Accusative for the banished thing
            ru::banish_from_hand(
                predicate_serializer::serialize_predicate_ru(
                    predicate,
                    Case::Acc,
                    Number::One,
                    bindings
                )
            )
        },

        // ...
    }
}
```

---

## Design Issue Discovered: Quantifier Case Requirements

Different quantifiers require different cases in Russian:

| Quantifier | Case Required | Example |
|------------|---------------|---------|
| 1 | Nominative or Accusative | "одна карта" / "одну карту" |
| 2-4 | Genitive singular | "две карты" |
| 5+ | Genitive plural | "пять карт" |
| "any number of" | Genitive plural | "любое количество карт" |
| "all" | Genitive plural | "всех союзников" |
| "up to N" | Genitive | "до трёх карт" |

**Implication:** The Rust code must know which quantifier it's using and pass the appropriate case to the predicate serializer. This is **logic**, not text, so it belongs in Rust.

---

## Design Issue Discovered: Compound Predicates

What about "ally with spark 3 or more"? In Russian:

- Nominative: "союзник с искрой 3 или больше"
- Accusative: "союзника с искрой 3 или больше"
- Genitive: "союзника с искрой 3 или больше"

Only the head noun inflects; the modifier phrase ("with spark...") stays the same.

**Solution:** The predicate serializer composes the phrase, inflecting only the head:

```rust
// ru.phr.rs
phraselet! {
    with_spark(base, s, op) = "{base} с искрой {s}{op}";
}

// predicate_serializer.rs
fn serialize_character_with_spark_ru(
    spark: i32,
    op: Operator,
    case: Case,
    number: Number,
) -> String {
    // Get the head noun in the right case/number
    let base = match (case, number) {
        (Case::Acc, Number::One) => ru::character_acc_one(),
        (Case::Gen, Number::Many) => ru::character_gen_many(),
        // ...
    };

    ru::with_spark(base, spark, operator_phrase_ru(op))
}
```

This works because Russian modifier phrases typically don't need case agreement with their head (unlike adjectives).

---

## Updated Design Insight: Case is a Rust Concern

The Phraselet design is validated: **case selection is logic, not text**.

1. **Phraselet provides** the inflected forms as variants
2. **Rust decides** which form to use based on grammatical context
3. **Composition** happens in Rust, calling Phraselet for atomic pieces

This separation is clean because:
- Translators define the text forms (they know their language's inflections)
- Programmers define the selection logic (they know which grammatical context applies)

---

## Example: Full Cost Serialization Flow

English input: `Cost::ReturnToHand { target: Predicate::Your(CardPredicate::Character), count: CollectionExpression::All }`

**English output:** "return all your characters to hand"

**Russian output:** "вернуть всех ваших персонажей в руку"

Flow:
1. Rust sees `CollectionExpression::All` → calls `ru::return_all_to_hand(target)`
2. Rust knows "all" requires genitive plural → calls predicate serializer with `(Case::Gen, Number::Many)`
3. Predicate serializer sees `Your(Character)` → needs "ваших персонажей" (your-GEN.PL characters-GEN.PL)
4. Calls `ru::your_character_gen_many()` → "ваших персонажей"
5. Final: `ru::return_all_to_hand("ваших персонажей")` → "вернуть всех ваших персонажей в руку"

---

## Summary of Design Validations

1. **Case as variants works well.** Multi-dimensional variants (`acc.one`, `gen.many`) handle Russian's case+number matrix.

2. **Logic in Rust is correct.** Case selection depends on grammatical context (verb, quantifier, preposition), which is code logic.

3. **Compound phrases compose correctly.** Only head nouns inflect; modifiers are appended unchanged.

4. **Language-specific serializers are needed.** The cost serializer needs per-language implementations that understand grammatical requirements.

## Potential Design Addition: Case Hints in Phrases

One possible enhancement: allow phrases to **document** what case their parameters expect, even though enforcement is in Rust:

```rust
// Hypothetical syntax - for documentation only
abandon(target: acc) = "пожертвовать {target}";
```

This would:
- Help translators understand what form `target` will have
- Enable tooling to validate that callers provide the right case
- Not change runtime behavior (still just string interpolation)

**Decision:** This is a nice-to-have for tooling, not essential for the core design.
