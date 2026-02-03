# Appendix: Localizing Predicate Serialization

This appendix works through localizing `predicate_serializer.rs` patterns using Phraselet, identifying design issues and proposing solutions.

## Overview of Existing Patterns

The serializer handles:
1. **Base card types**: card, character, event
2. **Ownership transforms**: your character → "ally", enemy character → "enemy"
3. **Pronouns**: this, that, it, them
4. **Subtypes**: Warrior, Mage, etc. (dynamic)
5. **Constraints**: "with spark 3 or more", "with cost less than..."
6. **Zones**: "in your void", "in the opponent's void"
7. **Multiple forms**: singular, plural, with/without article

## Design Principle: Logic in Rust, Text in Phraselet

The serializer has complex branching logic:

```rust
match predicate {
    Predicate::Your(card_predicate) => {
        if is_generic_card_type(card_predicate) {
            serialize_card_predicate(card_predicate, bindings)
        } else if let CardPredicate::CharacterType(subtype) = card_predicate {
            // ...
        }
    }
    // ...
}
```

This logic stays in Rust. Phraselet provides the atomic text pieces that Rust composes.

---

## English Phraselet File

```rust
// en.phr.rs
phraselet! {
    //==========================================================================
    // BASE CARD TYPES
    //==========================================================================

    card: {
        one = "card",
        other = "cards",
    };

    character: {
        one = "character",
        other = "characters",
    };

    event: {
        one = "event",
        other = "events",
    };

    //==========================================================================
    // OWNERSHIP VARIANTS
    //==========================================================================
    // "Your character" becomes "ally", "enemy character" becomes "enemy"

    ally: {
        one = "ally",
        other = "allies",
    };

    your_card: {
        one = "your card",
        other = "your cards",
    };

    your_event: {
        one = "your event",
        other = "your events",
    };

    enemy: {
        one = "enemy",
        other = "enemies",
    };

    enemy_card: {
        one = "enemy card",
        other = "enemy cards",
    };

    enemy_event: {
        one = "enemy event",
        other = "enemy events",
    };

    //==========================================================================
    // PRONOUNS
    //==========================================================================

    this_character: {
        one = "this character",
        other = "these characters",
    };

    that_character: {
        one = "that character",
        other = "those characters",
    };

    it: {
        one = "it",
        other = "them",
    };

    them = "them";

    //==========================================================================
    // OPERATORS
    //==========================================================================

    or_less = " or less";
    or_more = " or more";
    exactly = "";

    //==========================================================================
    // CONSTRAINT PHRASES
    //==========================================================================

    with_spark(base, s, op) = "{base} with spark {s}{op}";
    with_cost(base, cost, op) = "{base} with cost {cost}{op}";

    with_cost_less_than_allies(base, counting) =
        "{base} with cost less than the number of allied {counting:other}";

    with_cost_less_than_abandoned(base) =
        "{base} with cost less than the abandoned ally's cost";

    with_spark_less_than_abandoned(base) =
        "{base} with spark less than the abandoned ally's spark";

    with_spark_less_than_abandoned_count(base) =
        "{base} with spark less than the number of allies abandoned this turn";

    with_cost_less_than_void_count(base) =
        "{base} with cost less than the number of cards in your void";

    with_spark_less_than_energy_paid(base) =
        "{base} with spark less than the amount of {energy_symbol} paid";

    //==========================================================================
    // SPECIAL PREDICATES
    //==========================================================================

    with_materialized_ability: {
        one = "with a {materialized} ability",
        other = "with {materialized} abilities",
    };

    with_activated_ability: {
        one = "with an activated ability",
        other = "with activated abilities",
    };

    fast_prefix = "{fast}";

    not_subtype(subtype) = "that is not {@a subtype}";
    not_subtype_plural(subtype) = "that are not {subtype:other}";

    could_dissolve(target) = "which could {dissolve} {target}";

    //==========================================================================
    // ZONE MODIFIERS
    //==========================================================================

    in_your_void(base) = "{base} in your void";
    in_opponent_void(base) = "{base} in the opponent's void";

    //==========================================================================
    // SUBTYPE FORMATTING
    //==========================================================================
    // Subtypes are passed as strings. The @a transform handles articles.

    allied_subtype(subtype) = "allied {subtype}";
    allied_subtype_plural(subtype) = "allied {subtype:other}";

    enemy_subtype(subtype) = "enemy {subtype}";
    enemy_subtype_plural(subtype) = "enemy {subtype:other}";

    other_subtype(subtype) = "other {subtype}";

    //==========================================================================
    // FOR-EACH CONTEXTS
    //==========================================================================

    allied_character = "allied character";
    other_character = "other character";
    card_in_your_void = "card in your void";
    character_in_your_void = "character in your void";
    event_in_your_void = "event in your void";
    card_in_opponent_void = "card in the opponent's void";
    character_in_opponent_void = "character in the opponent's void";
    event_in_opponent_void = "event in the opponent's void";
    subtype_in_your_void(subtype) = "{subtype} in your void";

    //==========================================================================
    // KEYWORDS (formatted)
    //==========================================================================

    dissolve = "<k>dissolve</k>";
    materialized = "<k>materialized</k>";
    fast = "<k>fast</k>";
    energy_symbol = "<e>●</e>";

    //==========================================================================
    // MISCELLANEOUS
    //==========================================================================

    another(base) = "another {base}";
}
```

---

## Rust Serializer Using Phraselet

```rust
// predicate_serializer.rs (refactored)

use phraselet::en;  // or use phraselet::Language for dynamic selection

/// Returns the phrase to use for a card predicate, with article.
pub fn serialize_card_predicate(pred: &CardPredicate) -> String {
    match pred {
        CardPredicate::Card => format!("{}", en::a(en::card())),
        CardPredicate::Character => format!("{}", en::a(en::character())),
        CardPredicate::Event => format!("{}", en::an(en::event())),

        CardPredicate::CharacterType(subtype) => {
            en::a(subtype.name())  // @a applied to subtype string
        }

        CardPredicate::CharacterWithSpark(spark, op) => {
            en::with_spark(
                en::a(en::character()),
                spark.0,
                operator_phrase(op),
            )
        }

        CardPredicate::CardWithCost { target, cost_operator, cost } => {
            en::with_cost(
                serialize_card_predicate(target),
                cost.0,
                operator_phrase(cost_operator),
            )
        }

        // ... etc
    }
}

/// Returns plural form of a card predicate.
pub fn serialize_card_predicate_plural(pred: &CardPredicate) -> String {
    match pred {
        CardPredicate::Card => en::card_plural(),
        CardPredicate::Character => en::character_plural(),
        CardPredicate::Event => en::event_plural(),

        CardPredicate::CharacterType(subtype) => {
            subtype.plural_name()
        }

        // ... etc
    }
}

fn operator_phrase(op: &Operator) -> &'static str {
    match op {
        Operator::OrLess => en::or_less(),
        Operator::OrMore => en::or_more(),
        Operator::Exactly => en::exactly(),
    }
}
```

---

## Design Issues Discovered

### Issue 1: Selecting Variants of Phrase Parameters

When a phrase takes another phrase as a parameter, we need to select its variants:

```rust
with_cost_less_than_allies(base, counting) =
    "{base} with cost less than the number of allied {counting:other}";
```

Here `{counting:other}` means "use the 'other' (plural) variant of whatever phrase `counting` refers to."

**Resolution:** This works with current design. Selection on a phrase parameter uses that phrase's variants.

### Issue 2: The `@a` Transform on Dynamic Strings

Subtypes are runtime strings like "Warrior" or "Ancient". We need:

```rust
not_subtype(subtype) = "that is not {@a subtype}";
// subtype="Warrior" → "that is not a Warrior"
// subtype="Ancient" → "that is not an Ancient"
```

**Resolution:** `@a` inspects the rendered text at runtime and prepends "a" or "an". This works for any displayable value, not just phrase references.

### Issue 3: Phrases Need Both Singular and Plural Variants Accessible

The generated API needs to expose both forms:

```rust
// Current design generates:
pub fn card() -> &'static str;  // Returns... which form?

// Need:
pub fn card() -> &'static str;        // Singular (default)
pub fn card_plural() -> &'static str; // Plural
```

**Resolution:** Add to design: for phrases with variants, generate `_plural()`, `_one()`, etc. accessors. The base function returns the default (first) variant.

### Issue 4: Composing Formatted Keywords

Keywords like `{dissolve}` need to be interpolated with their formatting:

```rust
could_dissolve(target) = "which could {dissolve} {target}";
```

This works - `dissolve` is a phrase that returns `<k>dissolve</k>`, and interpolation includes it.

### Issue 5: The `@a` Transform with Adjectives

"a fast character" vs "an allied character" - the article depends on the first word after it.

```rust
// This works:
fast_character = "{@a fast} character";  // → "a fast character"

// But what about:
some_phrase(adj, noun) = "{@a adj} {noun}";
// adj="fast" → "a fast {noun}"
// adj="allied" → "an allied {noun}"
```

**Resolution:** `@a` applies to what follows it. `{@a adj}` prepends article to `adj`'s rendered text.

---

## Updated Design: Variant Accessors

For phrases with variants, Phraselet generates accessor functions:

```rust
// Phraselet definition:
card: {
    one = "card",
    other = "cards",
};

// Generated Rust:
pub fn card() -> &'static str { "card" }        // Default (first variant)
pub fn card_one() -> &'static str { "card" }
pub fn card_other() -> &'static str { "cards" }

// Or with an enum:
pub enum CardForm { One, Other }
pub fn card(form: CardForm) -> &'static str;
```

For simplicity, the `_variant()` suffix approach is clearest.

---

## Russian Example

```rust
// ru.phr.rs
phraselet! {
    // Cases: nom (nominative), acc (accusative), gen (genitive), etc.
    // Numbers: one, few, many

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
        // ...
    };

    ally: {
        nom.one = "союзник",
        nom.few = "союзника",
        nom.many = "союзников",
        acc.one = "союзника",  // animate masculine uses genitive
        acc.few = "союзников",
        acc.many = "союзников",
        // ...
    };

    enemy: {
        nom.one = "враг",
        nom.few = "врага",
        nom.many = "врагов",
        acc.one = "врага",
        acc.few = "врагов",
        acc.many = "врагов",
        // ...
    };

    // Operators
    or_less = " или меньше";
    or_more = " или больше";
    exactly = "";

    // Constraints - note case usage
    with_spark(base, s, op) = "{base} с искрой {s}{op}";
    with_cost(base, cost, op) = "{base} со стоимостью {cost}{op}";

    with_cost_less_than_allies(base, counting) =
        "{base} со стоимостью меньше количества союзных {counting:gen.other}";

    // Zones use prepositional case
    in_your_void(base) = "{base} в вашей пустоте";
    in_opponent_void(base) = "{base} в пустоте противника";
}
```

---

## Chinese Example

```rust
// zh_cn.phr.rs
phraselet! {
    // No pluralization, no articles, but measure words

    card = "牌";
    character = "角色";
    event = "事件";

    // Ownership as prefix
    ally = "友方角色";
    your_card = "你的牌";
    enemy = "敌方角色";
    enemy_card = "敌方牌";

    // Pronouns
    this_character = "此角色";
    that_character = "该角色";
    it = "它";
    them = "它们";

    // Operators
    or_less = "以下";
    or_more = "以上";
    exactly = "";

    // Constraints - word order differs
    with_spark(base, s, op) = "火花{s}{op}的{base}";
    with_cost(base, cost, op) = "费用{cost}{op}的{base}";

    with_cost_less_than_allies(base, counting) =
        "费用低于友方{counting}数量的{base}";

    // Zones
    in_your_void(base) = "你虚空中的{base}";
    in_opponent_void(base) = "对手虚空中的{base}";

    // Keywords
    dissolve = "<k>消散</k>";
    materialized = "<k>具现化</k>";
    fast = "<k>快速</k>";

    // Measure word phrases
    张(n, thing) = "{n}张{thing}";
    个(n, thing) = "{n}个{thing}";
}
```

---

## Summary of Design Refinements

1. **Variant accessors**: Phrases with variants generate `_variant()` functions (e.g., `card_other()`).

2. **`@a` works on any text**: Not just phrase references, but also string parameters. Inspects rendered text at runtime.

3. **Selection on phrase parameters**: `{param:variant}` selects a variant from the phrase that `param` refers to.

4. **Logic stays in Rust**: The complex branching logic (which phrase to use when) stays in Rust code. Phraselet provides atomic text pieces.

5. **Keywords are just phrases**: No special keyword syntax - define a phrase that returns formatted text.
