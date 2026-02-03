# Appendix: Russian Translation Walkthrough

This appendix provides a comprehensive example of translating `predicate_serializer.rs`
to Russian using RLT. It demonstrates how to keep language-specific text in RLT files
while maintaining language-agnostic Rust logic.

## Overview

The original `predicate_serializer.rs` contains ~800 lines of Rust code that produces
English text for card predicates. The goal is to:

1. Extract all English text into `en.rlt.rs`
2. Create a Russian translation in `ru.rlt.rs`
3. Refactor `predicate_serializer.rs` to call RLT functions instead of returning
   hardcoded strings

---

## Part 1: Analysis of the Original Code

### Text Categories

The serializer produces several categories of text:

**1. Pronouns/References:**
```rust
"this character"    // Predicate::This
"that character"    // Predicate::That
"them"              // Predicate::Them
"it"                // Predicate::It
```

**2. Ownership Qualifiers:**
```rust
"ally"              // Your character
"your card"         // Your card
"your event"        // Your event
"enemy"             // Enemy character
"allied {subtype}"  // Your character of type
"enemy {subtype}"   // Enemy character of type
```

**3. Card Type Bases:**
```rust
"card", "cards"
"character", "characters"
"event", "events"
```

**4. Complex Predicates:**
```rust
"a character with spark {s} or less"
"with cost less than the number of allied {counting}"
"event which could {dissolve} {target}"
```

**5. Location Qualifiers:**
```rust
"in your void"
"in the opponent's void"
```

### Russian Grammatical Requirements

Russian requires:
- **Six grammatical cases**: nominative, accusative, genitive, dative, instrumental,
  prepositional
- **Three genders**: masculine, feminine, neuter
- **Complex plural forms**: one (1, 21, 31...), few (2-4, 22-24...), many (5-20, 25-30...)
- **Animacy distinction**: affects accusative case for masculine nouns

For this serializer, we primarily need:
- Nominative (subject): "карта есть" (the card is)
- Accusative (direct object): "возьмите карту" (take the card)
- Genitive (possession/counting): "нет карт" (no cards), "5 карт" (5 cards)

---

## Part 2: The English RLT File

First, we extract all English text into RLT phrases:

```rust
// en.rlt.rs
rlt! {
    // =========================================================================
    // Basic Card Types
    // =========================================================================

    card = {
        one: "card",
        other: "cards",
    } :a;

    character = {
        one: "character",
        other: "characters",
    } :a;

    event = {
        one: "event",
        other: "events",
    } :an;

    // =========================================================================
    // Pronouns and References
    // =========================================================================

    this_character = "this character";
    that_character = "that character";
    these_characters = "these characters";
    those_characters = "those characters";
    them = "them";
    it = "it";

    // =========================================================================
    // Ownership: Your/Allied
    // =========================================================================

    ally = {
        one: "ally",
        other: "allies",
    } :an;

    your_card = {
        one: "your card",
        other: "your cards",
    };

    your_event = {
        one: "your event",
        other: "your events",
    };

    allied(thing) = "allied {thing}";

    // =========================================================================
    // Ownership: Enemy
    // =========================================================================

    enemy = {
        one: "enemy",
        other: "enemies",
    } :an;

    enemy_card = {
        one: "enemy card",
        other: "enemy cards",
    } :an;

    enemy_event = {
        one: "enemy event",
        other: "enemy events",
    } :an;

    // =========================================================================
    // Articles
    // =========================================================================

    // The @a transform handles "a" vs "an" based on the :a/:an tags
    with_article(thing) = "{@a thing}";

    // =========================================================================
    // Subtype Predicates
    // =========================================================================

    // Singular with article: "a {subtype}"
    a_subtype(subtype) = "{@a subtype}";

    // Plural: "{plural-subtype}"
    plural_subtype(subtype) = "{subtype:other}";

    // Allied subtype (singular/plural)
    allied_subtype(subtype) = "allied {subtype}";
    allied_subtype_plural(subtype) = "allied {subtype:other}";

    // Enemy subtype
    enemy_subtype(subtype) = "enemy {subtype}";
    enemy_subtype_plural(subtype) = "enemy {subtype:other}";

    // =========================================================================
    // Negation
    // =========================================================================

    not_a_subtype(subtype) = "a character that is not {@a subtype}";
    ally_not_subtype(subtype) = "ally that is not {@a subtype}";
    non_subtype_enemy(subtype) = "non-{subtype} enemy";

    characters_not_subtype_plural(subtype) = "characters that are not {subtype:other}";
    allies_not_subtype_plural(subtype) = "allies that are not {subtype:other}";

    // =========================================================================
    // Spark Constraints
    // =========================================================================

    character_with_spark(spark, op) = "a character with spark {spark}{op}";
    characters_with_spark(spark, op) = "characters with spark {spark}{op}";
    ally_with_spark(spark, op) = "ally with spark {spark}{op}";
    allies_with_spark(spark, op) = "allies with spark {spark}{op}";
    enemy_with_spark(spark, op) = "enemy with spark {spark}{op}";

    // =========================================================================
    // Cost Constraints
    // =========================================================================

    with_cost(base, cost, op) = "{base} with cost {cost}{op}";
    enemy_with_cost(cost, op) = "enemy with cost {cost}{op}";

    // =========================================================================
    // Ability Constraints
    // =========================================================================

    character_with_materialized = "a character with a {materialized} ability";
    characters_with_materialized = "characters with {materialized} abilities";
    ally_with_materialized = "ally with a {materialized} ability";
    allies_with_materialized = "allies with {materialized} abilities";
    enemy_with_materialized = "enemy with a {materialized} ability";

    character_with_activated = "a character with an activated ability";
    characters_with_activated = "characters with activated abilities";
    ally_with_activated = "ally with an activated ability";
    allies_with_activated = "allies with activated abilities";
    enemy_with_activated = "enemy with an activated ability";

    // =========================================================================
    // Complex Comparisons
    // =========================================================================

    with_spark_less_than_energy(base) =
        "{base} with spark less than the amount of {energy_symbol} paid";

    with_cost_less_than_allied(base, counting) =
        "{base} with cost less than the number of allied {counting:other}";

    with_cost_less_than_abandoned(base) =
        "{base} with cost less than the abandoned ally's cost";

    with_spark_less_than_abandoned(base) =
        "{base} with spark less than the abandoned ally's spark";

    with_spark_less_than_abandoned_enemy(base) =
        "{base} with spark less than that ally's spark";

    with_spark_less_than_abandoned_count(base) =
        "{base} with spark less than the number of allies abandoned this turn";

    with_cost_less_than_void(base) =
        "{base} with cost less than the number of cards in your void";

    // =========================================================================
    // Could Dissolve
    // =========================================================================

    event_could_dissolve(target) = "an event which could {dissolve} {target}";
    your_event_could_dissolve(target) = "your event which could {dissolve} {target}";
    events_could_dissolve(target) = "events which could {dissolve} {target}";
    your_events_could_dissolve(target) = "your events which could {dissolve} {target}";
    event_could_dissolve_base(target) = "event which could {dissolve} {target}";

    // =========================================================================
    // Fast Modifier
    // =========================================================================

    fast(base) = "a {fast} {base}";
    fast_plural(base) = "fast {base}";

    // =========================================================================
    // Void Location
    // =========================================================================

    in_your_void(things) = "{things} in your void";
    in_opponent_void(things) = "{things} in the opponent's void";

    // =========================================================================
    // Other Modifiers
    // =========================================================================

    another(thing) = "another {thing}";
    other(things) = "other {things}";

    // =========================================================================
    // For Each Patterns
    // =========================================================================

    for_each_prefix(thing) = "each {thing}";

    // Special cases for "for each"
    allied_character_each = "allied character";
    ally_each = "ally";
    enemy_each = "enemy";
    character_each = "character";
    card_each = "card";
    event_each = "event";
    other_character_each = "other character";
    card_in_void_each = "card in your void";
    character_in_void_each = "character in your void";
    event_in_void_each = "event in your void";
    card_in_opponent_void_each = "card in the opponent's void";
    character_in_opponent_void_each = "character in the opponent's void";
    event_in_opponent_void_each = "event in the opponent's void";

    // =========================================================================
    // Keywords (with formatting markup)
    // =========================================================================

    dissolve = "<k>dissolve</k>";
    materialized = "<k>materialized</k>";
    fast = "<k>fast</k>";
    energy_symbol = "<e>●</e>";
}
```

---

## Part 3: The Russian RLT File

Now we translate to Russian, handling case, number, and gender:

```rust
// ru.rlt.rs
rlt! {
    // =========================================================================
    // Basic Card Types
    //
    // Russian nouns decline for case and number. We need:
    // - nom (nominative): subject - "карта есть"
    // - acc (accusative): direct object - "возьмите карту"
    // - gen (genitive): possession, counting, "of" - "нет карт"
    //
    // Number categories: one (1, 21), few (2-4, 22-24), many (5-20, 0)
    // =========================================================================

    card = {
        // Nominative: "the card is..."
        nom.one: "карта",
        nom.few: "карты",
        nom.many: "карт",
        // Accusative: "take a card..."
        acc.one: "карту",
        acc.few: "карты",
        acc.many: "карт",
        // Genitive: "5 cards", "no cards"
        gen.one: "карты",
        gen.few: "карт",
        gen.many: "карт",
    } :fem :inan;

    character = {
        nom.one: "персонаж",
        nom.few: "персонажа",
        nom.many: "персонажей",
        acc.one: "персонажа",      // Animate masculine: acc = gen
        acc.few: "персонажей",
        acc.many: "персонажей",
        gen.one: "персонажа",
        gen.few: "персонажей",
        gen.many: "персонажей",
    } :masc :anim;

    event = {
        nom.one: "событие",
        nom.few: "события",
        nom.many: "событий",
        acc.one: "событие",        // Neuter: acc = nom
        acc.few: "события",
        acc.many: "событий",
        gen.one: "события",
        gen.few: "событий",
        gen.many: "событий",
    } :neut :inan;

    // =========================================================================
    // Pronouns and References
    // =========================================================================

    this_character = "этот персонаж";
    that_character = "тот персонаж";
    these_characters = "эти персонажи";
    those_characters = "те персонажи";
    them = "их";
    it = "его";

    // =========================================================================
    // Ownership: Your/Allied
    //
    // "союзник" (ally) is masculine animate
    // =========================================================================

    ally = {
        nom.one: "союзник",
        nom.few: "союзника",
        nom.many: "союзников",
        acc.one: "союзника",
        acc.few: "союзников",
        acc.many: "союзников",
        gen.one: "союзника",
        gen.few: "союзников",
        gen.many: "союзников",
    } :masc :anim;

    your_card = {
        nom.one: "ваша карта",
        nom.few: "ваши карты",
        nom.many: "ваших карт",
        acc.one: "вашу карту",
        acc.few: "ваши карты",
        acc.many: "ваших карт",
        gen.one: "вашей карты",
        gen.few: "ваших карт",
        gen.many: "ваших карт",
    };

    your_event = {
        nom.one: "ваше событие",
        nom.few: "ваши события",
        nom.many: "ваших событий",
        acc.one: "ваше событие",
        acc.few: "ваши события",
        acc.many: "ваших событий",
        gen.one: "вашего события",
        gen.few: "ваших событий",
        gen.many: "ваших событий",
    };

    // "союзный" (allied) must agree with the noun it modifies
    allied(thing) = "союзный {thing}";  // Simplified; see gender agreement below

    // =========================================================================
    // Ownership: Enemy
    //
    // "враг" (enemy) is masculine animate
    // =========================================================================

    enemy = {
        nom.one: "враг",
        nom.few: "врага",
        nom.many: "врагов",
        acc.one: "врага",
        acc.few: "врагов",
        acc.many: "врагов",
        gen.one: "врага",
        gen.few: "врагов",
        gen.many: "врагов",
    } :masc :anim;

    enemy_card = {
        nom.one: "вражеская карта",
        nom.few: "вражеские карты",
        nom.many: "вражеских карт",
        acc.one: "вражескую карту",
        acc.few: "вражеские карты",
        acc.many: "вражеских карт",
        gen.one: "вражеской карты",
        gen.few: "вражеских карт",
        gen.many: "вражеских карт",
    };

    enemy_event = {
        nom.one: "вражеское событие",
        nom.few: "вражеские события",
        nom.many: "вражеских событий",
        acc.one: "вражеское событие",
        acc.few: "вражеские события",
        acc.many: "вражеских событий",
        gen.one: "вражеского события",
        gen.few: "вражеских событий",
        gen.many: "вражеских событий",
    };

    // =========================================================================
    // Articles
    //
    // Russian has no articles! These phrases become identity functions.
    // =========================================================================

    with_article(thing) = "{thing}";

    // =========================================================================
    // Subtype Predicates
    //
    // Subtypes are passed as parameters with their own case variants.
    // =========================================================================

    a_subtype(subtype) = "{subtype:acc.one}";
    plural_subtype(subtype) = "{subtype:gen.many}";

    allied_subtype(subtype) = "союзный {subtype:nom.one}";
    allied_subtype_plural(subtype) = "союзных {subtype:gen.many}";

    enemy_subtype(subtype) = "вражеский {subtype:nom.one}";
    enemy_subtype_plural(subtype) = "вражеских {subtype:gen.many}";

    // =========================================================================
    // Negation
    // =========================================================================

    not_a_subtype(subtype) = "персонаж, который не является {subtype:ins.one}";
    ally_not_subtype(subtype) = "союзник, который не является {subtype:ins.one}";
    non_subtype_enemy(subtype) = "враг, не являющийся {subtype:ins.one}";

    characters_not_subtype_plural(subtype) =
        "персонажи, которые не являются {subtype:ins.many}";
    allies_not_subtype_plural(subtype) =
        "союзники, которые не являются {subtype:ins.many}";

    // =========================================================================
    // Spark Constraints
    // =========================================================================

    character_with_spark(spark, op) = "персонаж с искрой {spark}{op}";
    characters_with_spark(spark, op) = "персонажи с искрой {spark}{op}";
    ally_with_spark(spark, op) = "союзник с искрой {spark}{op}";
    allies_with_spark(spark, op) = "союзники с искрой {spark}{op}";
    enemy_with_spark(spark, op) = "враг с искрой {spark}{op}";

    // =========================================================================
    // Cost Constraints
    // =========================================================================

    with_cost(base, cost, op) = "{base} со стоимостью {cost}{op}";
    enemy_with_cost(cost, op) = "враг со стоимостью {cost}{op}";

    // =========================================================================
    // Ability Constraints
    // =========================================================================

    character_with_materialized = "персонаж со способностью {materialized}";
    characters_with_materialized = "персонажи со способностями {materialized}";
    ally_with_materialized = "союзник со способностью {materialized}";
    allies_with_materialized = "союзники со способностями {materialized}";
    enemy_with_materialized = "враг со способностью {materialized}";

    character_with_activated = "персонаж с активируемой способностью";
    characters_with_activated = "персонажи с активируемыми способностями";
    ally_with_activated = "союзник с активируемой способностью";
    allies_with_activated = "союзники с активируемыми способностями";
    enemy_with_activated = "враг с активируемой способностью";

    // =========================================================================
    // Complex Comparisons
    //
    // Russian word order is more flexible. We use structures that sound natural.
    // =========================================================================

    with_spark_less_than_energy(base) =
        "{base} с искрой меньше количества уплаченной {energy_symbol}";

    with_cost_less_than_allied(base, counting) =
        "{base} со стоимостью меньше количества союзных {counting:gen.many}";

    with_cost_less_than_abandoned(base) =
        "{base} со стоимостью меньше стоимости покинутого союзника";

    with_spark_less_than_abandoned(base) =
        "{base} с искрой меньше искры покинутого союзника";

    with_spark_less_than_abandoned_enemy(base) =
        "{base} с искрой меньше искры того союзника";

    with_spark_less_than_abandoned_count(base) =
        "{base} с искрой меньше количества союзников, покинутых в этом ходу";

    with_cost_less_than_void(base) =
        "{base} со стоимостью меньше количества карт в вашей бездне";

    // =========================================================================
    // Could Dissolve
    // =========================================================================

    event_could_dissolve(target) = "событие, способное {dissolve} {target}";
    your_event_could_dissolve(target) = "ваше событие, способное {dissolve} {target}";
    events_could_dissolve(target) = "события, способные {dissolve} {target}";
    your_events_could_dissolve(target) = "ваши события, способные {dissolve} {target}";
    event_could_dissolve_base(target) = "событие, способное {dissolve} {target}";

    // =========================================================================
    // Fast Modifier
    // =========================================================================

    fast(base) = "{fast} {base}";
    fast_plural(base) = "{fast} {base}";

    // =========================================================================
    // Void Location
    //
    // "бездна" (void) is feminine
    // "в вашей бездне" = in your void (prepositional case)
    // =========================================================================

    in_your_void(things) = "{things} в вашей бездне";
    in_opponent_void(things) = "{things} в бездне противника";

    // =========================================================================
    // Other Modifiers
    // =========================================================================

    another(thing) = "другой {thing}";
    other(things) = "другие {things}";

    // =========================================================================
    // For Each Patterns
    // =========================================================================

    for_each_prefix(thing) = "каждый {thing}";

    allied_character_each = "союзный персонаж";
    ally_each = "союзник";
    enemy_each = "враг";
    character_each = "персонаж";
    card_each = "карта";
    event_each = "событие";
    other_character_each = "другой персонаж";
    card_in_void_each = "карта в вашей бездне";
    character_in_void_each = "персонаж в вашей бездне";
    event_in_void_each = "событие в вашей бездне";
    card_in_opponent_void_each = "карта в бездне противника";
    character_in_opponent_void_each = "персонаж в бездне противника";
    event_in_opponent_void_each = "событие в бездне противника";

    // =========================================================================
    // Keywords
    // =========================================================================

    dissolve = "<k>растворить</k>";
    materialized = "<k>материализации</k>";
    fast = "<k>быстрый</k>";
    energy_symbol = "<e>●</e>";
}
```

---

## Part 4: The Refactored predicate_serializer.rs

The refactored serializer uses RLT phrases instead of hardcoded strings. The key
principle is: **Rust handles logic, RLT handles text**.

```rust
// predicate_serializer.rs

use ability_data::predicate::{CardPredicate, Predicate};
use ability_data::variable_value::VariableValue;

use crate::localization::{Language, rlt};
use crate::variables::parser_bindings::VariableBindings;
use crate::variables::parser_substitutions;

/// Serialize a predicate to localized text.
pub fn serialize_predicate(
    predicate: &Predicate,
    bindings: &mut VariableBindings,
    lang: Language,
) -> String {
    match predicate {
        Predicate::This => rlt::this_character(lang),
        Predicate::That => rlt::that_character(lang),
        Predicate::Them => rlt::them(lang),
        Predicate::It => rlt::it(lang),
        Predicate::Your(card_predicate) => {
            serialize_your_predicate_with_article(card_predicate, bindings, lang)
        }
        Predicate::Another(card_predicate) => {
            let base = serialize_your_predicate_base(card_predicate, bindings, lang);
            rlt::with_article(lang, base)
        }
        Predicate::Any(card_predicate) => {
            serialize_card_predicate(card_predicate, bindings, lang)
        }
        Predicate::Enemy(card_predicate) => {
            let base = serialize_enemy_predicate_base(card_predicate, bindings, lang);
            rlt::with_article(lang, base)
        }
        Predicate::YourVoid(card_predicate) => {
            let things = serialize_card_predicate_plural(card_predicate, bindings, lang);
            rlt::in_your_void(lang, things)
        }
        Predicate::EnemyVoid(card_predicate) => {
            let things = serialize_card_predicate_plural(card_predicate, bindings, lang);
            rlt::in_opponent_void(lang, things)
        }
        Predicate::AnyOther(card_predicate) => {
            let base = serialize_card_predicate_base(card_predicate, bindings, lang);
            rlt::another(lang, base)
        }
    }
}

/// Serialize a predicate in plural form.
pub fn serialize_predicate_plural(
    predicate: &Predicate,
    bindings: &mut VariableBindings,
    lang: Language,
) -> String {
    match predicate {
        Predicate::Another(card_predicate) | Predicate::Your(card_predicate) => {
            serialize_your_predicate_plural(card_predicate, bindings, lang)
        }
        Predicate::Any(card_predicate) => {
            serialize_card_predicate_plural(card_predicate, bindings, lang)
        }
        Predicate::Enemy(card_predicate) => {
            serialize_enemy_predicate_plural(card_predicate, bindings, lang)
        }
        Predicate::YourVoid(card_predicate) => {
            let things = serialize_card_predicate_plural(card_predicate, bindings, lang);
            rlt::in_your_void(lang, things)
        }
        Predicate::EnemyVoid(card_predicate) => {
            let things = serialize_card_predicate_plural(card_predicate, bindings, lang);
            rlt::in_opponent_void(lang, things)
        }
        Predicate::This => rlt::these_characters(lang),
        Predicate::That => rlt::those_characters(lang),
        Predicate::Them | Predicate::It => rlt::them(lang),
        Predicate::AnyOther(card_predicate) => {
            let base = serialize_card_predicate_plural(card_predicate, bindings, lang);
            rlt::other(lang, base)
        }
    }
}

/// Serialize the base text of a card predicate (no article).
fn serialize_card_predicate_base(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
    lang: Language,
) -> String {
    match card_predicate {
        CardPredicate::Card => rlt::card(lang),
        CardPredicate::Character => rlt::character(lang),
        CardPredicate::Event => rlt::event(lang),
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::subtype_placeholder(lang)  // "{subtype}" - resolved at render time
        }
        _ => serialize_card_predicate(card_predicate, bindings, lang),
    }
}

/// Serialize a card predicate with article.
fn serialize_card_predicate(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
    lang: Language,
) -> String {
    match card_predicate {
        CardPredicate::Card | CardPredicate::Character | CardPredicate::Event => {
            let base = serialize_card_predicate_base(card_predicate, bindings, lang);
            rlt::with_article(lang, base)
        }
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::a_subtype(lang, "{subtype}")
        }
        CardPredicate::Fast { target } => {
            let base = serialize_fast_target(target, bindings, lang);
            rlt::fast(lang, base)
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bind_cost(bindings, cost.0);
            let base = serialize_card_predicate(target, bindings, lang);
            let op = serialize_operator(cost_operator, lang);
            rlt::with_cost(lang, base, "{e}", op)
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            rlt::character_with_materialized(lang)
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            rlt::character_with_activated(lang)
        }
        CardPredicate::CharacterWithSparkComparedToEnergySpent { target, .. } => {
            let base = serialize_card_predicate(target, bindings, lang);
            rlt::with_spark_less_than_energy(lang, base)
        }
        CardPredicate::CouldDissolve { target } => {
            let target_text = serialize_predicate(target, bindings, lang);
            rlt::event_could_dissolve(lang, target_text)
        }
        CardPredicate::NotCharacterType(subtype) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::not_a_subtype(lang, "{subtype}")
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            let op = serialize_operator(operator, lang);
            rlt::character_with_spark(lang, "{s}", op)
        }
        CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
            let base = serialize_card_predicate(target, bindings, lang);
            let counting = serialize_card_predicate_plural(count_matching, bindings, lang);
            rlt::with_cost_less_than_allied(lang, base, counting)
        }
        CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
            let base = serialize_card_predicate(target, bindings, lang);
            rlt::with_cost_less_than_abandoned(lang, base)
        }
        CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
            let base = serialize_card_predicate(target, bindings, lang);
            rlt::with_spark_less_than_abandoned(lang, base)
        }
        CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
            let base = serialize_card_predicate(target, bindings, lang);
            rlt::with_spark_less_than_abandoned_count(lang, base)
        }
        CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
            let base = serialize_card_predicate(target, bindings, lang);
            rlt::with_cost_less_than_void(lang, base)
        }
    }
}

/// Serialize a card predicate in plural form.
fn serialize_card_predicate_plural(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
    lang: Language,
) -> String {
    match card_predicate {
        CardPredicate::Card => rlt::card_plural(lang),
        CardPredicate::Character => rlt::character_plural(lang),
        CardPredicate::Event => rlt::event_plural(lang),
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::plural_subtype(lang, "{subtype}")
        }
        CardPredicate::NotCharacterType(subtype) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::characters_not_subtype_plural(lang, "{subtype}")
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            let op = serialize_operator(operator, lang);
            rlt::characters_with_spark(lang, "{s}", op)
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            rlt::characters_with_materialized(lang)
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            rlt::characters_with_activated(lang)
        }
        CardPredicate::Fast { target } => {
            let base = serialize_card_predicate_plural(target, bindings, lang);
            rlt::fast_plural(lang, base)
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bind_cost(bindings, cost.0);
            let base = serialize_card_predicate_plural(target, bindings, lang);
            let op = serialize_operator(cost_operator, lang);
            rlt::with_cost(lang, base, "{e}", op)
        }
        CardPredicate::CouldDissolve { target } => {
            let target_text = predicate_base_text(target, bindings, lang);
            rlt::events_could_dissolve(lang, target_text)
        }
        // ... remaining cases follow the same pattern
        _ => serialize_card_predicate(card_predicate, bindings, lang),
    }
}

/// Serialize "your" predicate (ally/your card/your event).
fn serialize_your_predicate_base(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
    lang: Language,
) -> String {
    match card_predicate {
        CardPredicate::Character => rlt::ally(lang),
        CardPredicate::Card => rlt::your_card(lang),
        CardPredicate::Event => rlt::your_event(lang),
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::allied_subtype(lang, "{subtype}")
        }
        CardPredicate::NotCharacterType(subtype) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::ally_not_subtype(lang, "{subtype}")
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            let op = serialize_operator(operator, lang);
            rlt::ally_with_spark(lang, "{s}", op)
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            rlt::ally_with_materialized(lang)
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            rlt::ally_with_activated(lang)
        }
        CardPredicate::Fast { target } => {
            let base = serialize_fast_target(target, bindings, lang);
            rlt::fast(lang, base)
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bind_cost(bindings, cost.0);
            let base = serialize_your_predicate_base(target, bindings, lang);
            let op = serialize_operator(cost_operator, lang);
            rlt::with_cost(lang, base, "{e}", op)
        }
        CardPredicate::CouldDissolve { target } => {
            let target_text = predicate_base_text(target, bindings, lang);
            rlt::your_event_could_dissolve(lang, target_text)
        }
        _ => {
            // Fallback: use generic card predicate
            serialize_card_predicate(card_predicate, bindings, lang)
        }
    }
}

fn serialize_your_predicate_with_article(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
    lang: Language,
) -> String {
    if is_generic_card_type(card_predicate) {
        serialize_card_predicate(card_predicate, bindings, lang)
    } else if let CardPredicate::CharacterType(subtype) = card_predicate {
        bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
        rlt::a_subtype(lang, "{subtype}")
    } else {
        serialize_your_predicate_base(card_predicate, bindings, lang)
    }
}

/// Serialize "your" predicate in plural form.
fn serialize_your_predicate_plural(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
    lang: Language,
) -> String {
    match card_predicate {
        CardPredicate::Character => rlt::ally_plural(lang),
        CardPredicate::Card => rlt::your_card_plural(lang),
        CardPredicate::Event => rlt::your_event_plural(lang),
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::allied_subtype_plural(lang, "{subtype}")
        }
        CardPredicate::NotCharacterType(subtype) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::allies_not_subtype_plural(lang, "{subtype}")
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            let op = serialize_operator(operator, lang);
            rlt::allies_with_spark(lang, "{s}", op)
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            rlt::allies_with_materialized(lang)
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            rlt::allies_with_activated(lang)
        }
        CardPredicate::Fast { target } => {
            let base = serialize_card_predicate_plural(target, bindings, lang);
            rlt::fast_plural(lang, base)
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bind_cost(bindings, cost.0);
            let base = serialize_your_predicate_plural(target, bindings, lang);
            let op = serialize_operator(cost_operator, lang);
            rlt::with_cost(lang, base, "{e}", op)
        }
        CardPredicate::CouldDissolve { target } => {
            let target_text = predicate_base_text(target, bindings, lang);
            rlt::your_events_could_dissolve(lang, target_text)
        }
        _ => serialize_card_predicate_plural(card_predicate, bindings, lang),
    }
}

/// Serialize "enemy" predicate base.
fn serialize_enemy_predicate_base(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
    lang: Language,
) -> String {
    match card_predicate {
        CardPredicate::Character => rlt::enemy(lang),
        CardPredicate::Card => rlt::enemy_card(lang),
        CardPredicate::Event => rlt::enemy_event(lang),
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::enemy_subtype(lang, "{subtype}")
        }
        CardPredicate::NotCharacterType(subtype) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::non_subtype_enemy(lang, "{subtype}")
        }
        CardPredicate::CharacterWithSpark(spark, operator) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            let op = serialize_operator(operator, lang);
            rlt::enemy_with_spark(lang, "{s}", op)
        }
        CardPredicate::CharacterWithMaterializedAbility => {
            rlt::enemy_with_materialized(lang)
        }
        CardPredicate::CharacterWithMultiActivatedAbility => {
            rlt::enemy_with_activated(lang)
        }
        CardPredicate::CardWithCost { cost_operator, cost, .. } => {
            bind_cost(bindings, cost.0);
            let op = serialize_operator(cost_operator, lang);
            rlt::enemy_with_cost(lang, "{e}", op)
        }
        CardPredicate::Fast { target } => {
            let base = serialize_fast_target(target, bindings, lang);
            rlt::fast(lang, base)
        }
        CardPredicate::CouldDissolve { target } => {
            let target_text = serialize_predicate(target, bindings, lang);
            rlt::event_could_dissolve_base(lang, target_text)
        }
        _ => {
            // Complex predicates: delegate to generic handling
            let base = serialize_enemy_predicate_base_inner(card_predicate, bindings, lang);
            base
        }
    }
}

/// Serialize "enemy" predicate in plural form.
fn serialize_enemy_predicate_plural(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
    lang: Language,
) -> String {
    match card_predicate {
        CardPredicate::Character => rlt::enemy_plural(lang),
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::enemy_subtype_plural(lang, "{subtype}")
        }
        _ => {
            // Fallback: "enemy" + generic plural
            let base = serialize_card_predicate_plural(card_predicate, bindings, lang);
            rlt::enemy_prefix(lang, base)
        }
    }
}

/// Serialize fast target (no article).
fn serialize_fast_target(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
    lang: Language,
) -> String {
    match card_predicate {
        CardPredicate::Card => rlt::card(lang),
        CardPredicate::Character => rlt::character(lang),
        CardPredicate::Event => rlt::event(lang),
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::subtype_placeholder(lang)
        }
        CardPredicate::Fast { target } => {
            let base = serialize_fast_target(target, bindings, lang);
            rlt::fast_modifier(lang, base)
        }
        _ => serialize_card_predicate_base(card_predicate, bindings, lang),
    }
}

/// Serialize predicate for "for each" contexts.
pub fn serialize_for_each_predicate(
    predicate: &Predicate,
    bindings: &mut VariableBindings,
    lang: Language,
) -> String {
    // Many "for each" cases have special phrasings
    match predicate {
        Predicate::Another(CardPredicate::Character) => rlt::allied_character_each(lang),
        Predicate::Your(CardPredicate::Character) => rlt::ally_each(lang),
        Predicate::Enemy(CardPredicate::Character) => rlt::enemy_each(lang),
        Predicate::Any(CardPredicate::Character) => rlt::character_each(lang),
        Predicate::Any(CardPredicate::Card) => rlt::card_each(lang),
        Predicate::Any(CardPredicate::Event) => rlt::event_each(lang),
        Predicate::AnyOther(CardPredicate::Character) => rlt::other_character_each(lang),
        Predicate::YourVoid(CardPredicate::Card) => rlt::card_in_void_each(lang),
        Predicate::YourVoid(CardPredicate::Character) => rlt::character_in_void_each(lang),
        Predicate::YourVoid(CardPredicate::Event) => rlt::event_in_void_each(lang),
        Predicate::EnemyVoid(CardPredicate::Card) => rlt::card_in_opponent_void_each(lang),
        Predicate::EnemyVoid(CardPredicate::Character) => {
            rlt::character_in_opponent_void_each(lang)
        }
        Predicate::EnemyVoid(CardPredicate::Event) => rlt::event_in_opponent_void_each(lang),
        Predicate::This => rlt::this_character(lang),
        Predicate::That | Predicate::It => rlt::that_character(lang),
        Predicate::Them => rlt::character_each(lang),

        // Subtype handling
        Predicate::Another(CardPredicate::CharacterType(subtype))
        | Predicate::Your(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::allied_subtype(lang, "{subtype}")
        }
        Predicate::Enemy(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::enemy_subtype(lang, "{subtype}")
        }
        Predicate::Any(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::subtype_placeholder(lang)
        }
        Predicate::AnyOther(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::other_subtype(lang, "{subtype}")
        }

        // Spark constraints
        Predicate::Another(CardPredicate::CharacterWithSpark(spark, operator))
        | Predicate::Your(CardPredicate::CharacterWithSpark(spark, operator)) => {
            bindings.insert("s".to_string(), VariableValue::Integer(spark.0));
            let op = serialize_operator(operator, lang);
            rlt::ally_with_spark(lang, "{s}", op)
        }

        // Void subtypes
        Predicate::YourVoid(CardPredicate::CharacterType(subtype)) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::subtype_in_void(lang, "{subtype}")
        }

        // Generic fallback
        _ => {
            let base = predicate_base_text(predicate, bindings, lang);
            rlt::for_each_prefix(lang, base)
        }
    }
}

/// Get the base text of a predicate (used in various contexts).
pub fn predicate_base_text(
    predicate: &Predicate,
    bindings: &mut VariableBindings,
    lang: Language,
) -> String {
    match predicate {
        Predicate::This => rlt::this_character(lang),
        Predicate::That => rlt::that_character(lang),
        Predicate::Them => rlt::them(lang),
        Predicate::It => rlt::it(lang),
        Predicate::Your(card_predicate) => {
            if is_generic_card_type(card_predicate) {
                serialize_card_predicate_base(card_predicate, bindings, lang)
            } else {
                serialize_your_predicate_base(card_predicate, bindings, lang)
            }
        }
        Predicate::Another(card_predicate) => {
            serialize_your_predicate_base(card_predicate, bindings, lang)
        }
        Predicate::Any(card_predicate) => {
            serialize_card_predicate_base(card_predicate, bindings, lang)
        }
        Predicate::Enemy(card_predicate) => {
            // Complex: depends on context
            serialize_enemy_base_for_context(card_predicate, bindings, lang)
        }
        Predicate::YourVoid(card_predicate) => {
            let base = serialize_card_predicate_base(card_predicate, bindings, lang);
            rlt::in_your_void(lang, base)
        }
        Predicate::EnemyVoid(card_predicate) => {
            let base = serialize_card_predicate_base(card_predicate, bindings, lang);
            rlt::in_opponent_void(lang, base)
        }
        Predicate::AnyOther(card_predicate) => {
            let base = serialize_card_predicate_base(card_predicate, bindings, lang);
            rlt::another(lang, base)
        }
    }
}

// =========================================================================
// Helper Functions
// =========================================================================

fn is_generic_card_type(card_predicate: &CardPredicate) -> bool {
    matches!(
        card_predicate,
        CardPredicate::Card | CardPredicate::Character | CardPredicate::Event
    )
}

fn bind_cost(bindings: &mut VariableBindings, cost: i32) {
    if let Some(var_name) = parser_substitutions::directive_to_integer_variable("e") {
        bindings.insert(var_name.to_string(), VariableValue::Integer(cost));
    }
}

fn serialize_operator(operator: &CostOperator, lang: Language) -> String {
    match operator {
        CostOperator::LessOrEqual => rlt::or_less(lang),
        CostOperator::GreaterOrEqual => rlt::or_more(lang),
        CostOperator::Equal => String::new(),
    }
}

fn serialize_enemy_base_for_context(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
    lang: Language,
) -> String {
    match card_predicate {
        _ if is_generic_card_type(card_predicate) => {
            serialize_card_predicate_base(card_predicate, bindings, lang)
        }
        CardPredicate::CouldDissolve { target } => {
            let target_text = serialize_predicate(target, bindings, lang);
            rlt::event_could_dissolve_base(lang, target_text)
        }
        CardPredicate::CardWithCost { .. } => {
            serialize_card_predicate_without_article(card_predicate, bindings, lang)
        }
        _ => serialize_enemy_predicate_base(card_predicate, bindings, lang),
    }
}

fn serialize_card_predicate_without_article(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
    lang: Language,
) -> String {
    match card_predicate {
        CardPredicate::Card | CardPredicate::Character | CardPredicate::Event => {
            serialize_card_predicate_base(card_predicate, bindings, lang)
        }
        CardPredicate::CharacterType(subtype) => {
            bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
            rlt::subtype_placeholder(lang)
        }
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bind_cost(bindings, cost.0);
            let base = serialize_card_predicate_without_article(target, bindings, lang);
            let op = serialize_operator(cost_operator, lang);
            rlt::with_cost(lang, base, "{e}", op)
        }
        _ => serialize_card_predicate(card_predicate, bindings, lang),
    }
}

/// Serialize only the cost constraint (for contexts where base type is implied).
pub fn serialize_cost_constraint_only(
    card_predicate: &CardPredicate,
    bindings: &mut VariableBindings,
    lang: Language,
) -> String {
    match card_predicate {
        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            bind_cost(bindings, cost.0);
            let op = serialize_operator(cost_operator, lang);
            if is_generic_card_type(target) {
                rlt::cost_constraint_only(lang, "{e}", op)
            } else {
                let base = serialize_card_predicate_without_article(target, bindings, lang);
                rlt::with_cost(lang, base, "{e}", op)
            }
        }
        _ => serialize_card_predicate_without_article(card_predicate, bindings, lang),
    }
}
```

---

## Part 5: Key Design Decisions

### 1. Language Parameter Threading

Every serialization function takes a `Language` parameter. This allows the same Rust
logic to produce text in any supported language:

```rust
let en_text = serialize_predicate(&pred, &mut bindings, Language::En);
let ru_text = serialize_predicate(&pred, &mut bindings, Language::Ru);
```

### 2. RLT Phrases for All Text

No hardcoded English strings remain in the Rust code. Every piece of text comes from
an RLT phrase:

```rust
// Before (hardcoded)
"this character".to_string()

// After (RLT)
rlt::this_character(lang)
```

### 3. Variable Bindings Pass Through

Subtype and numeric variables are still bound in Rust code, but their text
representation is determined by RLT:

```rust
bindings.insert("subtype".to_string(), VariableValue::Subtype(*subtype));
rlt::allied_subtype(lang, "{subtype}")  // RLT handles the text
```

### 4. Complex Phrases in RLT

Complex phrases with multiple parameters are defined in RLT:

```rust
// en.rlt.rs
with_cost_less_than_allied(base, counting) =
    "{base} with cost less than the number of allied {counting:other}";

// ru.rlt.rs
with_cost_less_than_allied(base, counting) =
    "{base} со стоимостью меньше количества союзных {counting:gen.many}";
```

The Rust code just calls these phrases with the appropriate parameters.

### 5. Case Selection in Russian

Russian case selection happens through RLT's variant selection:

```rust
// When we need genitive plural (for counting)
rlt::allied_subtype_plural(lang, "{subtype}")
// → "союзных {subtype:gen.many}" in Russian
// → "allied {subtype:other}" in English
```

### 6. Article Handling

English needs articles; Russian doesn't. The `with_article` phrase handles this:

```rust
// en.rlt.rs
with_article(thing) = "{@a thing}";  // Adds "a" or "an"

// ru.rlt.rs
with_article(thing) = "{thing}";     // No article needed
```

---

## Part 6: Testing the Translation

### Example: "Draw 3 cards"

```rust
// English
rlt::draw(Language::En, 3)  // → "Draw 3 cards."

// Russian
rlt::draw(Language::Ru, 3)  // → "Возьмите 3 карты."
```

### Example: "an ally with spark 2 or less"

```rust
let pred = Predicate::Your(CardPredicate::CharacterWithSpark(Spark(2), CostOperator::LessOrEqual));

serialize_predicate(&pred, &mut bindings, Language::En)
// → "an ally with spark 2 or less"

serialize_predicate(&pred, &mut bindings, Language::Ru)
// → "союзник с искрой 2 или меньше"
```

### Example: "characters with cost less than the number of allied characters"

```rust
let pred = Predicate::Any(CardPredicate::CharacterWithCostComparedToControlled {
    target: Box::new(CardPredicate::Character),
    count_matching: Box::new(CardPredicate::Character),
    operator: CostOperator::LessThan,
});

serialize_predicate(&pred, &mut bindings, Language::En)
// → "a character with cost less than the number of allied characters"

serialize_predicate(&pred, &mut bindings, Language::Ru)
// → "персонаж со стоимостью меньше количества союзных персонажей"
```

---

## Summary

| Aspect | Before | After |
|--------|--------|-------|
| Text location | Hardcoded in Rust | RLT files |
| Language support | English only | Any language |
| Article handling | In Rust logic | RLT transforms |
| Plural forms | English rules | CLDR rules per language |
| Case declension | N/A | RLT variant selection |
| Rust code role | Text + logic | Logic only |

The refactored approach keeps all language-specific text in RLT files, making
translation straightforward: translators work with `.rlt.rs` files and never need
to touch Rust code.
