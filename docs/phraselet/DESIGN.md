# Phraselet: A Domain-Specific Language for Localization

**Version:** 0.1 Draft
**Status:** Design Document

## Executive Summary

Phraselet is a localization DSL embedded in Rust that prioritizes translator readability while maintaining strong type safety. Unlike language-agnostic systems, Phraselet provides first-class syntax for common grammatical patterns in the world's most widely-spoken languages.

**Primary Goal:** Human readability for translators with minimal programming knowledge.
**Secondary Goal:** Strong typing with compile-time error detection.

## Design Principles

### 1. Asymmetric Localization

A simple English message can expand to a complex Russian translation without affecting other languages. Each language file contains exactly the complexity that language requires—no more, no less.

### 2. Source Language as Contract

The English (or designated source) `.phr.rs` file defines the API contract: message IDs, parameter names, parameter types, and enums. Other languages must provide translations for all messages but can structure their text freely.

### 3. Language-Aware, Not Language-Blind

Phraselet includes built-in syntax for common patterns:
- **English:** Article selection (a/an), regular pluralization
- **Russian:** Case declension, gender agreement, Slavic plural rules
- **Chinese:** Measure words, no plural/article overhead
- **Spanish/Portuguese:** Gender agreement, contractions
- **German:** Case + gender + number agreement
- **Japanese/Korean:** Counters, particles

### 4. Valid Rust, Readable Text

Phraselet files (`.phr.rs`) are valid Rust source files processed by macros. However, the macro syntax is designed to look like natural text with minimal punctuation.

---

## File Structure

```
src/
  localization/
    mod.rs              # Module declarations
    en.phr.rs           # English (source language)
    zh_cn.phr.rs        # Simplified Chinese
    ru.phr.rs           # Russian
    es.phr.rs           # Spanish
    pt_br.phr.rs        # Portuguese (Brazil)
    de.phr.rs           # German
    fr.phr.rs           # French
    ja.phr.rs           # Japanese
    ko.phr.rs           # Korean
    pl.phr.rs           # Polish
```

---

## Core Syntax

### Basic Messages

The simplest form is a message with no parameters:

```rust
phraselet! {
    // Message ID followed by equals sign and quoted text
    confirm_button = "Confirm"
    cancel_button = "Cancel"

    // Multi-line strings use triple quotes
    welcome_message = """
        Welcome to Dreamtides!
        Prepare for battle.
        """
}
```

### Messages with Parameters

Parameters are declared in parentheses with types:

```rust
phraselet! {
    // Integer parameter
    draw_cards(count: Int) = "Draw {count} cards."

    // String parameter
    player_turn(name: String) = "{name}'s turn."

    // Multiple parameters
    damage(source: String, amount: Int, target: String) =
        "{source} deals {amount} damage to {target}."
}
```

### Interpolation

Variables are referenced with `{name}` syntax:

```rust
phraselet! {
    gain_spark(target: Predicate, amount: Int) =
        "{target} gains +{amount} spark."
}
```

---

## Nouns and Inflection

### Defining Nouns (English)

Nouns have singular and plural forms:

```rust
phraselet! {
    // Simple noun: singular / plural
    noun card = "card" / "cards"
    noun character = "character" / "characters"
    noun event = "event" / "events"

    // Irregular plural
    noun ally = "ally" / "allies"
    noun enemy = "enemy" / "enemies"
}
```

### Article Selection (English)

English requires choosing "a" or "an" based on the following sound:

```rust
phraselet! {
    // Automatic: 'a' before consonants, 'an' before vowels
    noun card = "card" / "cards"        // "a card"
    noun event = "event" / "events"     // "an event"

    // Override when pronunciation differs from spelling
    noun uniform = a "uniform" / "uniforms"   // Forces "a"
    noun hour = an "hour" / "hours"           // Forces "an"
}
```

### Using Nouns in Messages

```rust
phraselet! {
    // Reference with article
    draw_a_card = "Draw {card.a}."           // "Draw a card."
    play_an_event = "Play {event.a}."        // "Play an event."

    // Reference plural form
    all_cards = "All {card.plural}."         // "All cards."

    // Count-sensitive (automatic singular/plural)
    draw_n(count: Int) = "Draw {count} {card.count(count)}."
    // count=1: "Draw 1 card."
    // count=3: "Draw 3 cards."
}
```

---

## Pluralization

### English Pluralization

English uses two forms: singular (1) and plural (everything else, including 0):

```rust
phraselet! {
    cards_remaining(count: Int) = match count {
        1 => "1 card remaining."
        _ => "{count} cards remaining."
    }
}
```

### Russian Pluralization

Russian has complex rules based on the final digits:

```rust
phraselet! {
    // Russian uses CLDR categories: one, few, many
    карт_осталось(count: Int) = match count {
        one  => "{count} карта осталась."    // 1, 21, 31...
        few  => "{count} карты осталось."    // 2-4, 22-24...
        many => "{count} карт осталось."     // 0, 5-20, 25-30...
    }
}
```

### Chinese (No Pluralization)

Chinese doesn't distinguish singular/plural:

```rust
phraselet! {
    // No match needed - same text for all counts
    剩余卡牌(count: Int) = "剩余{count}张牌。"
}
```

---

## Grammatical Gender

### Spanish Example

```rust
phraselet! {
    // Define nouns with gender
    noun carta(fem) = "carta" / "cartas"
    noun personaje(masc) = "personaje" / "personajes"

    // Adjectives agree with noun gender
    destroyed(target: Noun) = "{target} {está.gender(target)} {destruido.gender(target)}."
    // carta: "La carta está destruida."
    // personaje: "El personaje está destruido."
}
```

### Russian Example (Gender + Case)

```rust
phraselet! {
    // Define noun with full declension
    noun карта(fem) = {
        nom.sg: "карта",    nom.pl: "карты",
        gen.sg: "карты",    gen.pl: "карт",
        dat.sg: "карте",    dat.pl: "картам",
        acc.sg: "карту",    acc.pl: "карты",
        ins.sg: "картой",   ins.pl: "картами",
        pre.sg: "карте",    pre.pl: "картах",
    }

    // Use case markers in interpolation
    draw_card = "Возьмите {карта.acc.sg}."  // "Возьмите карту."
    with_cards(count: Int) = "с {count} {карта.ins.count(count)}."
    // count=1: "с 1 картой."
    // count=3: "с 3 картами."
}
```

---

## Measure Words (Chinese, Japanese, Korean)

### Chinese

```rust
phraselet! {
    // Define measure words for noun categories
    measure 张 for [card, event]     // flat objects
    measure 个 for [character, ally] // generic
    measure 点 for [spark, energy]   // points/small amounts

    // Use in messages - measure word inserted automatically
    draw_cards(count: Int) = "抽{count:card}。"
    // Expands to: "抽3张牌。" (draws measure word 张 from card's category)

    gain_spark(amount: Int) = "获得{amount:spark}火花。"
    // Expands to: "获得5点火花。"
}
```

### Japanese

```rust
phraselet! {
    measure 枚 for [card]      // flat objects
    measure 体 for [character] // bodies/characters
    measure 点 for [points]    // points

    draw_cards(count: Int) = "{count:card}を引く。"
    // Expands to: "3枚を引く。"
}
```

---

## Ownership and Context Transforms

Game text often refers to "your" things vs "enemy" things differently:

### English Transforms

```rust
phraselet! {
    // Define how ownership changes nouns
    transform your {
        character => "ally" / "allies"
        card => "your card" / "your cards"
        event => "your event" / "your events"
    }

    transform enemy {
        character => "enemy" / "enemies"
        card => "enemy card" / "enemy cards"
    }

    // Use in messages
    dissolve_ally = "Dissolve {your.character.a}."    // "Dissolve an ally."
    dissolve_enemy = "Dissolve {enemy.character.a}."  // "Dissolve an enemy."
}
```

### Chinese Transforms

```rust
phraselet! {
    transform 友方 {
        角色 => "友方角色"
    }

    transform 敌方 {
        角色 => "敌方角色"
    }

    消散友方 = "消散一个{友方.角色}。"  // "消散一个友方角色。"
}
```

---

## Predicates (Complex Noun Phrases)

Card games often have complex targeting expressions:

```rust
phraselet! {
    // Simple predicate
    predicate any_character = {character}

    // Predicate with constraint
    predicate character_with_spark(s: Int, op: Operator) =
        "{character} with spark {s}{op}"

    // Ownership + constraint
    predicate ally_with_cost(cost: Int, op: Operator) =
        "{your.character} with cost {cost}{op}"

    // Nested predicate
    predicate card_with_cost(target: Predicate, cost: Int, op: Operator) =
        "{target} with cost {cost}{op}"

    // Usage
    dissolve_target(target: Predicate) = "Dissolve {target.a}."
    // With ally_with_cost(3, OrLess): "Dissolve an ally with cost 3 or less."
}
```

---

## Keywords and Directives

Game keywords get special formatting:

```rust
phraselet! {
    // Define keywords with formatting
    keyword dissolve = "<purple>dissolve</purple>"
    keyword Dissolve = "<purple>Dissolve</purple>"  // Capitalized variant
    keyword foresee(n: Int) = "<purple>Foresee</purple> {n}"
    keyword kindle(k: Int) = "<purple>Kindle</purple> {k}"

    // Use in messages - keywords auto-format
    effect_dissolve(target: Predicate) = "{Dissolve} {target.a}."
    effect_foresee(n: Int) = "{foresee(n)}."
}
```

---

## Conditional Text

### Simple Conditionals

```rust
phraselet! {
    gains_reclaim(target: Predicate, this_turn: Bool) =
        "{target} gains reclaim{if this_turn: this turn}."
    // this_turn=true:  "It gains reclaim this turn."
    // this_turn=false: "It gains reclaim."
}
```

### Match Expressions

```rust
phraselet! {
    collection_text(expr: Collection) = match expr {
        exactly(1) => "{target.a}"
        exactly(n) => "{n} {target.plural}"
        up_to(n)   => "up to {n} {target.plural}"
        all        => "all {target.plural}"
        any_number => "any number of {target.plural}"
    }
}
```

---

## Enums

### Defining Enums

```rust
phraselet! {
    enum CardType {
        Character = "character"
        Event = "event"
    }

    enum Operator {
        OrLess = " or less"
        OrMore = " or more"
        Exactly = ""
    }

    // Usage
    card_type_label(card_type: CardType) = "{card_type} card"
}
```

### Localized Enum Values

```rust
// en.phr.rs
phraselet! {
    enum CardType {
        Character = "character"
        Event = "event"
    }
}

// de.phr.rs
phraselet! {
    enum CardType {
        Character = "Charakter"
        Event = "Ereignis"
    }
}
```

---

## Message Composition

Messages can reference other messages:

```rust
phraselet! {
    // Base messages
    draw_cards(count: Int) = "draw {count} {card.count(count)}"
    gain_spark(amount: Int) = "gain +{amount} spark"

    // Composed message
    draw_and_gain(cards: Int, spark: Int) =
        "{draw_cards(cards)}, then {gain_spark(spark)}."
    // "draw 3 cards, then gain +2 spark."
}
```

---

## Generated Rust API

Phraselet generates strongly-typed Rust functions:

```rust
// Generated from en.phr.rs

pub mod en {
    /// Draw {count} cards.
    pub fn draw_cards(count: i64) -> Message { ... }

    /// {target} gains +{amount} spark.
    pub fn gains_spark(target: impl Into<Predicate>, amount: i64) -> Message { ... }

    /// Dissolve {target}.
    pub fn dissolve(target: impl Into<Predicate>) -> Message { ... }
}

// Language-agnostic API
pub mod messages {
    pub fn draw_cards(lang: Language, count: i64) -> Message { ... }
}

// Usage
let msg = en::draw_cards(3);           // "Draw 3 cards."
let msg = de::draw_cards(3);           // "Ziehe 3 Karten."
let msg = messages::draw_cards(Language::En, 3);
```

---

## Error Messages

### Missing Translation

```
error[P001]: Missing translation for message 'dissolve_ally'
  --> src/localization/de.phr.rs
   |
   = note: message defined in src/localization/en.phr.rs:45
   = help: add to de.phr.rs:
   |
   |   dissolve_ally = "..."  // Dissolve an ally.
```

### Wrong Parameter Type

```
error[P002]: Type mismatch in message 'draw_cards'
  --> src/localization/de.phr.rs:12:25
   |
12 |     draw_cards(count: String) = "Ziehe {count} Karten."
   |                       ^^^^^^ expected `Int`, found `String`
   |
   = note: parameter type defined in src/localization/en.phr.rs:8
```

### Missing Parameter

```
error[P003]: Parameter 'amount' not used in translation
  --> src/localization/de.phr.rs:15:5
   |
15 |     damage_dealt(source: String, amount: Int) = "{source} verursacht Schaden."
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: parameter 'amount' defined but not used in message text
   = help: use {amount} in the translation or mark as unused with _amount
```

### Unknown Interpolation

```
error[P004]: Unknown variable '{damage}' in message
  --> src/localization/de.phr.rs:18:42
   |
18 |     attack_message = "{source} fügt {damage} Schaden zu."
   |                                      ^^^^^^ not defined
   |
   = note: defined parameters: source, amount
   = help: did you mean {amount}?
```

---

## Complete Example: Card Effect

### English (Source)

```rust
// en.phr.rs
phraselet! {
    language: "en-US"

    // Nouns
    noun card = "card" / "cards"
    noun character = "character" / "characters"
    noun ally = "ally" / "allies"
    noun enemy = "enemy" / "enemies"
    noun event = "event" / "events"

    // Transforms
    transform your {
        character => "ally" / "allies"
    }
    transform enemy {
        character => "enemy" / "enemies"
    }

    // Keywords
    keyword dissolve = "<k>dissolve</k>"
    keyword Dissolve = "<k>Dissolve</k>"
    keyword foresee(n: Int) = "<k>Foresee</k> {n}"

    // Effects
    dissolve_target(target: Predicate) = "{Dissolve} {target.a}."

    foresee_effect(n: Int) = "{foresee(n)}."

    draw_and_dissolve(cards: Int, target: Predicate) =
        "Draw {cards} {card.count(cards)}, then {dissolve} {target.a}."

    conditional_draw(count: Int, condition: Condition) =
        "{condition} draw {count} {card.count(count)}."
}
```

### Russian

```rust
// ru.phr.rs
phraselet! {
    language: "ru-RU"

    // Nouns with declension
    noun карта(fem) = {
        nom: "карта" / "карты",
        gen: "карты" / "карт",
        acc: "карту" / "карты",
        // ... full declension table
    }

    noun союзник(masc.animate) = {
        nom: "союзник" / "союзники",
        gen: "союзника" / "союзников",
        acc: "союзника" / "союзников",  // animate = genitive
        // ...
    }

    // Effects (must match English message IDs)
    dissolve_target(target: Predicate) =
        "{Dissolve} {target.acc}."

    foresee_effect(n: Int) = "{foresee(n)}."

    draw_and_dissolve(cards: Int, target: Predicate) = match cards {
        one  => "Возьмите {cards} {карта.acc.sg}, затем {dissolve} {target.acc}."
        few  => "Возьмите {cards} {карта.acc.pl}, затем {dissolve} {target.acc}."
        many => "Возьмите {cards} {карта.gen.pl}, затем {dissolve} {target.acc}."
    }
}
```

### Simplified Chinese

```rust
// zh_cn.phr.rs
phraselet! {
    language: "zh-CN"

    // Measure words
    measure 张 for [card, event]
    measure 个 for [character, ally, enemy]
    measure 点 for [spark, energy]

    // No plural forms needed
    noun 牌 = "牌"
    noun 角色 = "角色"
    noun 友方角色 = "友方角色"
    noun 敌方角色 = "敌方角色"

    // Keywords
    keyword 消散 = "<k>消散</k>"
    keyword 预见(n: Int) = "<k>预见</k>{n}"

    // Effects
    dissolve_target(target: Predicate) = "{消散}{target}。"

    foresee_effect(n: Int) = "{预见(n)}。"

    // No plural branching needed
    draw_and_dissolve(cards: Int, target: Predicate) =
        "抽{cards:card}，然后{消散}{target}。"
}
```

---

## Language Feature Matrix

| Feature | EN | ZH | RU | ES | PT | DE | FR | JA | KO | PL |
|---------|----|----|----|----|----|----|----|----|----|----|
| Articles (a/an/the) | ✓ | - | - | ✓ | ✓ | ✓ | ✓ | - | - | - |
| Plural forms | 2 | 1 | 3 | 2 | 2 | 2 | 2 | 1 | 1 | 3 |
| Grammatical gender | - | - | 3 | 2 | 2 | 3 | 2 | - | - | 3 |
| Case system | - | - | 6 | - | - | 4 | - | - | - | 7 |
| Measure words | - | ✓ | - | - | - | - | - | ✓ | ✓ | - |
| Verb agreement | - | - | ✓ | ✓ | ✓ | ✓ | ✓ | - | - | ✓ |
| Contractions | - | - | - | - | ✓ | - | ✓ | - | - | - |
| Particles | - | - | - | - | - | - | - | ✓ | ✓ | - |

---

## Appendices

See the following appendix documents for detailed language-specific guidance:

- [Appendix A: English Localization Guide](./appendix-a-english.md)
- [Appendix B: Russian Localization Guide](./appendix-b-russian.md)
- [Appendix C: Chinese Localization Guide](./appendix-c-chinese.md)
- [Appendix D: Spanish/Portuguese Localization Guide](./appendix-d-romance.md)
- [Appendix E: German Localization Guide](./appendix-e-german.md)
- [Appendix F: CJK Languages Guide](./appendix-f-cjk.md)

---

## Implementation Notes

This design document focuses on the DSL syntax. Implementation details including:

- Macro expansion strategy
- CLDR plural rule integration
- Build-time validation
- Runtime message resolution

...are covered in the separate [Implementation Specification](./IMPLEMENTATION.md).
