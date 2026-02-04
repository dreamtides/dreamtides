# Appendix: Spanish Translation Walkthrough

This appendix provides a comprehensive example of translating `cost_serializer.rs`
to Spanish using RLT. It demonstrates how to extract all localization concerns
into RLT files while keeping the serializer code language-agnostic.

## Overview

The original `cost_serializer.rs` contains ~150 lines of Rust code that produces
English text for card costs. The goal is to:

1. Extract all English text into `en.rlt.rs` using `rlt_source!`
2. Create a Spanish translation in `es.rlt.rs` using `rlt_lang!`
3. Refactor `cost_serializer.rs` to be language-agnostic, delegating all
   grammatical decisions to RLT

---

## Part 1: Analysis of the Original Code

### Cost Types

The serializer handles these cost types:

```rust
Cost::AbandonCharactersCount { target, count }  // "abandon 3 allies"
Cost::DiscardCards { count, .. }                // "discard 2"
Cost::DiscardHand                               // "discard your hand"
Cost::Energy(energy)                            // "{e}"
Cost::LoseMaximumEnergy(amount)                 // "lose {maximum-energy}"
Cost::BanishCardsFromYourVoid(count)            // "{Banish} 3 from your void"
Cost::BanishCardsFromEnemyVoid(count)           // "{Banish} 3 from the opponent's void"
Cost::BanishAllCardsFromYourVoidWithMinCount(n) // "{Banish} your void with 5 or more cards"
Cost::BanishFromHand(predicate)                 // "{Banish} a card from hand"
Cost::Choice(costs)                             // cost1 or cost2
Cost::ReturnToHand { target, count }            // "return 2 allies to hand"
Cost::SpendOneOrMoreEnergy                      // "pay 1 or more {energy-symbol}"
Cost::BanishAllCardsFromYourVoid                // "{Banish} your void"
Cost::CostList(costs)                           // cost1 and cost2
```

### Problematic Patterns in the Original

The original code mixes text generation with logic:

```rust
// Problem 1: Hardcoded English text
Cost::DiscardHand => "discard your hand".to_string(),

// Problem 2: Pre-rendering predicates as strings
CollectionExpression::AnyNumberOf => {
    format!(
        "abandon any number of {}",
        predicate_serializer::serialize_predicate_plural(target, bindings)
    )
}

// Problem 3: Variable binding for numbers (loses context)
if let Some(var_name) = parser_substitutions::directive_to_integer_variable("discards") {
    bindings.insert(var_name.to_string(), VariableValue::Integer(*count));
}
"{discards}".to_string()
```

### Spanish Grammatical Requirements

Spanish requires:
- **Gender agreement**: Articles and adjectives must agree with noun gender
- **Definite articles**: el (masc), la (fem), los (masc pl), las (fem pl)
- **Indefinite articles**: un (masc), una (fem), unos (masc pl), unas (fem pl)
- **Number agreement**: Singular/plural for nouns, verbs, adjectives, articles

For this serializer:
- "carta" (card) is feminine → "una carta", "las cartas"
- "personaje" (character) is masculine → "un personaje", "los personajes"
- "aliado" (ally) is masculine → "un aliado", "los aliados"
- "mano" (hand) is feminine → "tu mano"
- "vacío" (void) is masculine → "tu vacío", "el vacío del oponente"

---

## Part 2: Key Design Principle

### Pass PhraseRef, Not String

The critical insight: **Rust should pass `PhraseRef` values to RLT phrases, not
pre-rendered strings.** This allows RLT to select the appropriate grammatical form.

**Wrong approach** (current code):
```rust
// Rust pre-renders predicate, losing grammatical information
let target_text = serialize_predicate_plural(target, bindings);
// target_text = "allies" (String) — no gender, no case variants!

format!("abandon any number of {}", target_text)
// Cannot produce correct Spanish: "abandona cualquier cantidad de aliados"
// because we don't know "aliados" is masculine for agreement
```

**Correct approach**:
```rust
// Rust passes PhraseRef with full grammatical information
let target = lang.ally();  // PhraseRef with :masc tag and all forms

lang.abandon_any_number(target)
// Spanish template can now use gender tag for agreement
```

### Let RLT Handle All Grammatical Decisions

The serializer should identify *what* to say (semantic meaning), not *how* to
say it (grammatical form). Examples:

| Semantic Intent | Rust Calls | RLT Decides |
|-----------------|------------|-------------|
| "one ally" | `lang.abandon_one(target)` | Article, gender agreement |
| "3 cards" | `lang.abandon_n(3, target)` | Number agreement, word order |
| "any number of allies" | `lang.abandon_any_number(target)` | Quantifier, agreement |
| "your hand" | `lang.your_hand()` | Possessive form |

---

## Part 3: The English RLT File

English is simpler—no gender, simple plurals:

```rust
// en.rlt.rs
rlt_source! {
    // =========================================================================
    // Basic Types
    // =========================================================================

    card = :a { one: "card", other: "cards" };
    character = :a { one: "character", other: "characters" };
    ally = :an { one: "ally", other: "allies" };

    // =========================================================================
    // Keyword Formatting
    // =========================================================================

    banish = "<k>Banish</k>";
    energy_symbol = "<e>●</e>";

    // =========================================================================
    // Locations
    // =========================================================================

    your_void = "your void";
    opponent_void = "the opponent's void";
    your_hand = "your hand";
    hand = "hand";

    // =========================================================================
    // Abandon Costs
    // =========================================================================

    abandon_any_number(target) = "abandon any number of {target:other}";
    abandon_one(target) = "abandon {@a target}";
    abandon_n(n, target) = "abandon {n} {target:n}";

    // =========================================================================
    // Discard Costs
    // =========================================================================

    discard_n(n) = "discard {n}";
    discard_your_hand = "discard your hand";

    // =========================================================================
    // Energy Costs
    // =========================================================================

    energy_cost(n) = "{n}";
    lose_maximum_energy(n) = "lose {n}";
    pay_one_or_more_energy = "pay 1 or more {energy_symbol}";

    // =========================================================================
    // Banish Costs
    // =========================================================================

    banish_one_from_void = "{banish} another card in your void";
    banish_n_from_your_void(n) = "{banish} {n} from your void";
    banish_n_from_opponent_void(n) = "{banish} {n} from the opponent's void";
    banish_your_void = "{banish} your void";
    banish_void_with_min(n) = "{banish} your void with {n} or more cards";
    banish_from_hand(target) = "{banish} {target} from hand";

    // =========================================================================
    // Return to Hand Costs
    // =========================================================================

    return_one(target) = "return {@a target} to hand";
    return_n(n, target) = "return {n} {target:n} to hand";
    return_all_but_one(target) = "return all but one {target:one} to hand";
    return_all(target) = "return all {target:other} to hand";
    return_any_number(target) = "return any number of {target:other} to hand";
    return_up_to(n, target) = "return up to {n} {target:n} to hand";
    return_each_other(target) = "return each other {target:one} to hand";
    return_n_or_more(n, target) = "return {n} or more {target:other} to hand";

    // =========================================================================
    // Connectors
    // =========================================================================

    cost_or = " or ";
    cost_and = " and ";
}
```

---

## Part 4: The Spanish RLT File

Spanish uses the same phrase names but different templates with gender agreement:

```rust
// es.rlt.rs
rlt_lang!(Es) {
    // =========================================================================
    // Basic Types
    //
    // Spanish nouns have gender. Tags enable article transforms and agreement.
    // Plural forms: one (1), other (2+)
    // =========================================================================

    card = :fem {
        one: "carta",
        other: "cartas",
    };

    character = :masc {
        one: "personaje",
        other: "personajes",
    };

    ally = :masc {
        one: "aliado",
        other: "aliados",
    };

    // =========================================================================
    // Keyword Formatting
    // =========================================================================

    banish = "<k>Destierra</k>";
    energy_symbol = "<e>●</e>";

    // =========================================================================
    // Locations
    //
    // "vacío" is masculine, "mano" is feminine
    // =========================================================================

    your_void = "tu vacío";
    opponent_void = "el vacío del oponente";
    your_hand = "tu mano";
    hand = "mano";

    // =========================================================================
    // Abandon Costs
    //
    // Spanish uses gender-agreeing quantifiers
    // "cualquier cantidad de" works for both genders
    // =========================================================================

    abandon_any_number(target) = "abandona cualquier cantidad de {target:other}";
    abandon_one(target) = "abandona {@un target}";
    abandon_n(n, target) = "abandona {n} {target:n}";

    // =========================================================================
    // Discard Costs
    // =========================================================================

    discard_n(n) = "descarta {n}";
    discard_your_hand = "descarta tu mano";

    // =========================================================================
    // Energy Costs
    // =========================================================================

    energy_cost(n) = "{n}";
    lose_maximum_energy(n) = "pierde {n}";
    pay_one_or_more_energy = "paga 1 o más {energy_symbol}";

    // =========================================================================
    // Banish Costs
    //
    // Gender agreement with "carta" (fem) and "vacío" (masc)
    // =========================================================================

    banish_one_from_void = "{banish} otra carta de tu vacío";
    banish_n_from_your_void(n) = "{banish} {n} de tu vacío";
    banish_n_from_opponent_void(n) = "{banish} {n} del vacío del oponente";
    banish_your_void = "{banish} tu vacío";
    banish_void_with_min(n) = "{banish} tu vacío con {n} o más cartas";
    banish_from_hand(target) = "{banish} {target} de la mano";

    // =========================================================================
    // Return to Hand Costs
    //
    // "devolver" (return) conjugates; agreement with target gender
    // =========================================================================

    return_one(target) = "devuelve {@un target} a la mano";
    return_n(n, target) = "devuelve {n} {target:n} a la mano";
    return_all_but_one(target) = "devuelve todos menos {@un target} a la mano";
    return_all(target) = "devuelve {@el:other target} a la mano";
    return_any_number(target) = "devuelve cualquier cantidad de {target:other} a la mano";
    return_up_to(n, target) = "devuelve hasta {n} {target:n} a la mano";
    return_each_other(target) = "devuelve cada otro {target:one} a la mano";
    return_n_or_more(n, target) = "devuelve {n} o más {target:other} a la mano";

    // =========================================================================
    // Connectors
    // =========================================================================

    cost_or = " o ";
    cost_and = " y ";
}
```

---

## Part 5: Refactored cost_serializer.rs

The refactored serializer passes `PhraseRef` values and delegates all grammatical
decisions to RLT:

```rust
// cost_serializer.rs

use ability_data::collection_expression::CollectionExpression;
use ability_data::cost::Cost;
use crate::localization::{RltLang, PhraseRef};

/// Serialize a cost to localized text.
pub fn serialize_cost(cost: &Cost, lang: &impl RltLang) -> String {
    match cost {
        Cost::AbandonCharactersCount { target, count } => {
            serialize_abandon(target, count, lang)
        }

        Cost::DiscardCards { count, .. } => {
            lang.discard_n(*count).to_string()
        }

        Cost::DiscardHand => {
            lang.discard_your_hand().to_string()
        }

        Cost::Energy(energy) => {
            lang.energy_cost(energy.0).to_string()
        }

        Cost::LoseMaximumEnergy(amount) => {
            lang.lose_maximum_energy(*amount).to_string()
        }

        Cost::BanishCardsFromYourVoid(count) => {
            if *count == 1 {
                lang.banish_one_from_void().to_string()
            } else {
                lang.banish_n_from_your_void(*count).to_string()
            }
        }

        Cost::BanishCardsFromEnemyVoid(count) => {
            lang.banish_n_from_opponent_void(*count).to_string()
        }

        Cost::BanishAllCardsFromYourVoidWithMinCount(min_count) => {
            lang.banish_void_with_min(*min_count).to_string()
        }

        Cost::BanishFromHand(predicate) => {
            let target = predicate_to_phrase(predicate, lang);
            lang.banish_from_hand(target).to_string()
        }

        Cost::Choice(costs) => {
            costs
                .iter()
                .map(|c| serialize_cost(c, lang))
                .collect::<Vec<_>>()
                .join(&lang.cost_or().to_string())
        }

        Cost::ReturnToHand { target, count } => {
            serialize_return_to_hand(target, count, lang)
        }

        Cost::SpendOneOrMoreEnergy => {
            lang.pay_one_or_more_energy().to_string()
        }

        Cost::BanishAllCardsFromYourVoid => {
            lang.banish_your_void().to_string()
        }

        Cost::CostList(costs) => {
            costs
                .iter()
                .map(|c| serialize_cost(c, lang))
                .collect::<Vec<_>>()
                .join(&lang.cost_and().to_string())
        }
    }
}

/// Convert a predicate to a PhraseRef for use in cost phrases.
/// This preserves variant information for RLT selection.
fn predicate_to_phrase(predicate: &Predicate, lang: &impl RltLang) -> PhraseRef {
    match predicate {
        Predicate::Your(CardPredicate::Character) => lang.ally(),
        Predicate::Enemy(CardPredicate::Character) => lang.character(),
        Predicate::Any(CardPredicate::Card) => lang.card(),
        Predicate::Any(CardPredicate::Character) => lang.character(),
        // ... other predicate types
        _ => lang.card(),
    }
}

/// Serialize an abandon cost with collection expression.
fn serialize_abandon(
    target: &Predicate,
    count: &CollectionExpression,
    lang: &impl RltLang,
) -> String {
    let target_phrase = predicate_to_phrase(target, lang);

    match count {
        CollectionExpression::AnyNumberOf => {
            lang.abandon_any_number(target_phrase).to_string()
        }
        CollectionExpression::Exactly(1) => {
            lang.abandon_one(target_phrase).to_string()
        }
        CollectionExpression::Exactly(n) => {
            lang.abandon_n(*n, target_phrase).to_string()
        }
        _ => lang.abandon_n(1, target_phrase).to_string(),
    }
}

/// Serialize a return-to-hand cost with collection expression.
fn serialize_return_to_hand(
    target: &Predicate,
    count: &CollectionExpression,
    lang: &impl RltLang,
) -> String {
    let target_phrase = predicate_to_phrase(target, lang);

    match count {
        CollectionExpression::Exactly(1) => {
            lang.return_one(target_phrase).to_string()
        }
        CollectionExpression::Exactly(n) => {
            lang.return_n(*n, target_phrase).to_string()
        }
        CollectionExpression::AllButOne => {
            lang.return_all_but_one(target_phrase).to_string()
        }
        CollectionExpression::All => {
            lang.return_all(target_phrase).to_string()
        }
        CollectionExpression::AnyNumberOf => {
            lang.return_any_number(target_phrase).to_string()
        }
        CollectionExpression::UpTo(n) => {
            lang.return_up_to(*n, target_phrase).to_string()
        }
        CollectionExpression::EachOther => {
            lang.return_each_other(target_phrase).to_string()
        }
        CollectionExpression::OrMore(n) => {
            lang.return_n_or_more(*n, target_phrase).to_string()
        }
    }
}

/// Serialize a cost used as a trigger cost (may need different phrasing).
pub fn serialize_trigger_cost(cost: &Cost, lang: &impl RltLang) -> String {
    match cost {
        Cost::Energy(_) => format!("pay {}", serialize_cost(cost, lang)),
        _ => serialize_cost(cost, lang),
    }
}
```

---

## Part 6: How Spanish Gender Agreement Works

### The @un Transform

Spanish uses the `@un` transform to add indefinite articles with gender agreement:

```rust
// es.rlt.rs
rlt_lang!(Es) {
    card = :fem { one: "carta", other: "cartas" };
    ally = :masc { one: "aliado", other: "aliados" };

    abandon_one(target) = "abandona {@un target}";
}
```

The `@un` transform reads the `:fem` or `:masc` tag from the target phrase:

| Input | Tag | Transform Result |
|-------|-----|------------------|
| `card` | `:fem` | "una carta" |
| `ally` | `:masc` | "un aliado" |

### The @el Transform with Number

For definite articles with plural agreement:

```rust
return_all(target) = "devuelve {@el:other target} a la mano";
```

The `@el:other` syntax selects the plural definite article:

| Input | Tag | Transform Result |
|-------|-----|------------------|
| `card` (other) | `:fem` | "las cartas" |
| `ally` (other) | `:masc` | "los aliados" |

### Direct Number Selection

For phrases with explicit counts, selection handles agreement:

```rust
abandon_n(n, target) = "abandona {n} {target:n}";
```

| n | target | Result |
|---|--------|--------|
| 1 | ally | "abandona 1 aliado" |
| 3 | ally | "abandona 3 aliados" |
| 1 | card | "abandona 1 carta" |
| 5 | card | "abandona 5 cartas" |

---

## Part 7: Complete Example Traces

### Example 1: "abandon any number of allies"

**Rust code:**
```rust
let target = Predicate::Your(CardPredicate::Character);
let count = CollectionExpression::AnyNumberOf;
serialize_cost(&Cost::AbandonCharactersCount { target, count }, &lang)
```

**English flow:**
```
predicate_to_phrase(target) → lang.ally()
    → PhraseRef { text: "ally", variants: [("one", "ally"), ("other", "allies")], tags: ["an"] }

lang.abandon_any_number(target_phrase)
    Template: "abandon any number of {target:other}"
    Selection: target:other → "allies"
    Result: "abandon any number of allies"
```

**Spanish flow:**
```
predicate_to_phrase(target) → lang.ally()
    → PhraseRef { text: "aliado", variants: [("one", "aliado"), ("other", "aliados")], tags: ["masc"] }

lang.abandon_any_number(target_phrase)
    Template: "abandona cualquier cantidad de {target:other}"
    Selection: target:other → "aliados"
    Result: "abandona cualquier cantidad de aliados"
```

### Example 2: "return a card to hand"

**Rust code:**
```rust
let target = Predicate::Any(CardPredicate::Card);
let count = CollectionExpression::Exactly(1);
serialize_cost(&Cost::ReturnToHand { target, count }, &lang)
```

**English flow:**
```
predicate_to_phrase(target) → lang.card()
    → PhraseRef { tags: ["a"], ... }

lang.return_one(target_phrase)
    Template: "return {@a target} to hand"
    Transform: @a reads :a tag → "a card"
    Result: "return a card to hand"
```

**Spanish flow:**
```
predicate_to_phrase(target) → lang.card()
    → PhraseRef { tags: ["fem"], ... }

lang.return_one(target_phrase)
    Template: "devuelve {@un target} a la mano"
    Transform: @un reads :fem tag → "una carta"
    Result: "devuelve una carta a la mano"
```

### Example 3: "discard 3 or banish 2 from your void"

**Rust code:**
```rust
let cost = Cost::Choice(vec![
    Cost::DiscardCards { count: 3, .. },
    Cost::BanishCardsFromYourVoid(2),
]);
serialize_cost(&cost, &lang)
```

**English flow:**
```
serialize_cost(DiscardCards) → lang.discard_n(3) → "discard 3"
serialize_cost(Banish) → lang.banish_n_from_your_void(2) → "{banish} 2 from your void"
Join with cost_or → " or "
Result: "discard 3 or <k>Banish</k> 2 from your void"
```

**Spanish flow:**
```
serialize_cost(DiscardCards) → lang.discard_n(3) → "descarta 3"
serialize_cost(Banish) → lang.banish_n_from_your_void(2) → "{banish} 2 de tu vacío"
Join with cost_or → " o "
Result: "descarta 3 o <k>Destierra</k> 2 de tu vacío"
```

---

## Part 8: Benefits of This Approach

### What the Serializer Does NOT Need to Know

The refactored serializer is completely language-agnostic. It does NOT:

- Know that Spanish has gender
- Know that "carta" is feminine
- Know that "aliado" is masculine
- Know which article form to use
- Know how to pluralize words
- Handle any grammatical agreement

All of these concerns are handled in RLT files.

### Comparison: Old vs New

**Old approach (hardcoded English):**
```rust
CollectionExpression::Exactly(1) => {
    format!("abandon {}", predicate_serializer::serialize_predicate(target, bindings))
}
```

Problems:
- English text hardcoded in Rust
- Predicate pre-rendered as String, losing gender information
- Would need entirely different code paths for Spanish

**New approach (RLT):**
```rust
CollectionExpression::Exactly(1) => {
    lang.abandon_one(predicate_to_phrase(target, lang)).to_string()
}
```

Benefits:
- No English text in Rust
- PhraseRef preserves gender tag
- Same Rust code works for all languages
- Spanish translator controls all grammatical decisions

### For Translators

1. **Full control over articles**: Use `@un`/`@el` transforms
2. **Full control over gender**: Tags on phrases enable automatic agreement
3. **Full control over word order**: Rearrange phrase templates freely
4. **No Rust knowledge required**: All work happens in RLT files

### For Developers

1. **Simpler serializer**: No linguistic logic, just semantic decisions
2. **Single code path**: Same Rust for all languages
3. **Type-safe**: PhraseRef carries metadata; selection validates at compile time
4. **Testable**: Can test serializer output for any language

---

## Summary

The key insight: **keep grammatical decisions in RLT, semantic decisions in Rust.**

| Responsibility | Where |
|----------------|-------|
| "What cost type is this?" | Rust |
| "What predicate should I reference?" | Rust |
| "What article does this noun need?" | RLT |
| "What is the plural form?" | RLT |
| "How does gender agreement work?" | RLT |
| "What word order sounds natural?" | RLT |

The cost serializer becomes a simple mapping from cost types to RLT phrase calls.
All the linguistic complexity—gender, articles, pluralization, word order—lives
in the RLT files where translators can control it directly.

This approach scales to any language: add a new `.rlt.rs` file with appropriate
tags and phrase templates, and the same Rust serializer produces grammatically
correct output automatically.
