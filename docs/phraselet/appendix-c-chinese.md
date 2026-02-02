# Appendix C: Simplified Chinese Localization Guide

## Overview

Simplified Chinese has a simpler grammar than many European languages, making localization easier in some ways but requiring different patterns:

- **No plural forms** - nouns don't change
- **No articles** - no equivalent to "a/an/the"
- **No grammatical gender** - no agreement needed
- **Measure words required** - numbers need classifiers
- **Different word order** - often topic-comment structure

## Measure Words (量词)

### The Measure Word System

In Chinese, you cannot directly attach a number to a noun. You must use a measure word (classifier) between them:

```
English: 3 cards
Chinese: 3张牌 (3 + 张 + 牌)
         number + measure + noun
```

### Common Measure Words for Card Games

| Measure | Pinyin | Used for | Example |
|---------|--------|----------|---------|
| 张 | zhāng | flat objects (cards) | 3张牌 (3 cards) |
| 个 | gè | generic (people, things) | 2个角色 (2 characters) |
| 点 | diǎn | points, small amounts | 5点火花 (5 spark) |
| 次 | cì | occurrences, times | 2次 (twice) |
| 倍 | bèi | multiples | 2倍 (double) |
| 回合 | huíhé | turns, rounds | 这个回合 (this turn) |

### Defining Measure Words in Phraselet

```rust
phraselet! {
    // Associate measure words with nouns
    measure 张 for [牌, 事件牌]      // flat objects
    measure 个 for [角色, 友方角色, 敌方角色, 复制品]  // generic
    measure 点 for [火花, 能量, 分数]  // points

    // Nouns (no singular/plural distinction)
    noun 牌 = "牌"
    noun 角色 = "角色"
    noun 火花 = "火花"
    noun 能量 = "能量"
}
```

### Using Measure Words

```rust
phraselet! {
    // {count:noun} syntax automatically inserts the measure word
    抽牌(count: Int) = "抽{count:牌}。"
    // count=1: "抽1张牌。"
    // count=3: "抽3张牌。"

    获得火花(amount: Int) = "获得{amount:火花}。"
    // amount=5: "获得5点火花。"

    具现化角色(count: Int) = "具现化{count:角色}。"
    // count=2: "具现化2个角色。"
}
```

### The "一个" Pattern

When English uses "a/an", Chinese uses "一" (one) + measure word:

```rust
phraselet! {
    // "an ally" → "一个友方角色"
    消散友方 = "消散一个{友方角色}。"

    // "a card" → "一张牌"
    抽一张牌 = "抽一张{牌}。"

    // With shorthand
    消散(target: Predicate) = "消散一{target.measure}{target}。"
}
```

---

## Ownership Prefixes

Instead of possessive pronouns, Chinese uses prefixes:

| English | Chinese | Literal |
|---------|---------|---------|
| your character | 友方角色 | friendly-side character |
| enemy character | 敌方角色 | enemy-side character |
| your card | 你的牌 | your card |
| opponent's void | 对手的虚空 | opponent's void |

### Defining Ownership Transforms

```rust
phraselet! {
    // Ownership prefixes
    transform 友方 {
        角色 => "友方角色"
        事件 => "友方事件"
    }

    transform 敌方 {
        角色 => "敌方角色"
        事件 => "敌方事件"
    }

    // Usage
    消散友方角色 = "消散一个{友方.角色}。"  // "消散一个友方角色。"
    消散敌方角色 = "消散一个{敌方.角色}。"  // "消散一个敌方角色。"
}
```

---

## No Pluralization Needed

Chinese nouns don't change form for plural. The same word is used for one or many:

```rust
phraselet! {
    // No match statement needed for singular/plural
    抽牌(count: Int) = "抽{count:牌}。"
    // Works for all counts: "抽1张牌。", "抽3张牌。", "抽10张牌。"

    // Compare to English which needs:
    // match count { 1 => "Draw 1 card." _ => "Draw {count} cards." }
}
```

---

## Keywords

```rust
phraselet! {
    // Game keywords with Chinese translations
    keyword 消散 = "<k>消散</k>"
    keyword 放逐 = "<k>放逐</k>"
    keyword 具现化 = "<k>具现化</k>"
    keyword 回收 = "<k>回收</k>"
    keyword 阻止 = "<k>阻止</k>"

    // Keywords with parameters
    // Note: In Chinese, the number often comes AFTER the keyword
    keyword 预见(n: Int) = "<k>预见</k>{n}"
    keyword 点燃(k: Int) = "<k>点燃</k>{k}"

    // Reclaim with cost
    keyword 回收花费(cost: Int) = "<k>回收</k> <e>{cost}</e>"
}
```

---

## Word Order Differences

Chinese often places modifiers before the noun they modify, and may use different sentence structures:

### Relative Clauses

```rust
phraselet! {
    // English: "a character with spark 3 or more"
    // Chinese: "火花3以上的角色" (spark 3 or more DE character)

    // The modifier comes BEFORE the noun, connected by 的 (de)
    带火花的角色(s: Int, op: Operator) =
        "火花{s}{op}的{角色}"

    // Usage: "消散一个火花3以上的角色。"
}
```

### Comparison with English

| English | Chinese | Structure |
|---------|---------|-----------|
| a character with spark 3 | 火花3的角色 | modifier + 的 + noun |
| ally in your void | 你虚空中的友方角色 | location + 中的 + noun |
| card that costs 2 | 费用2的牌 | property + 的 + noun |

```rust
phraselet! {
    // "an ally with cost 2 or less"
    费用内的友方(cost: Int, op: Operator) =
        "费用{cost}{op}的友方角色"

    // "a card in your void"
    虚空中的牌 = "你虚空中的牌"

    // "the opponent's void"
    对手虚空 = "对手的虚空"
}
```

---

## Operators

```rust
phraselet! {
    enum Operator {
        OrLess = "以下"      // or less
        OrMore = "以上"      // or more
        Exactly = ""         // exactly (often implied)
    }

    // Usage
    火花要求(s: Int, op: Operator) = "火花{s}{op}"
    // s=3, OrMore: "火花3以上"
    // s=2, OrLess: "火花2以下"
}
```

---

## Conditional Text

```rust
phraselet! {
    // Boolean conditionals work the same
    获得回收(target: Predicate, this_turn: Bool) =
        "{target}获得{回收}{if this_turn:，直到回合结束}。"

    // this_turn=true:  "它获得回收，直到回合结束。"
    // this_turn=false: "它获得回收。"
}
```

---

## Collection Expressions

```rust
phraselet! {
    消散目标(target: Predicate, collection: Collection) = match collection {
        exactly(1)    => "消散一{target.measure}{target}。"
        exactly(n)    => "消散{n}{target.measure}{target}。"
        up_to(n)      => "消散至多{n}{target.measure}{target}。"
        all           => "消散所有{target}。"
        any_number    => "消散任意数量的{target}。"
        all_but_one   => "消散除一个以外的所有{target}。"
    }
}
```

---

## Complete Examples

### Draw Cards

```rust
phraselet! {
    // Simple draw
    抽牌(count: Int) = "抽{count:牌}。"
    // "抽3张牌。"

    // Draw with condition
    条件抽牌(count: Int, condition: Condition) =
        "{condition}，抽{count:牌}。"
    // "如果你控制3个以上的友方角色，抽2张牌。"
}
```

### Gain Spark

```rust
phraselet! {
    // Character gains spark
    获得火花(target: Predicate, amount: Int) =
        "{target}获得+{amount:火花}。"
    // "此角色获得+3点火花。"

    // Each ally gains spark
    每个友方获得火花(amount: Int) =
        "每个友方角色获得+{amount:火花}。"
}
```

### Dissolve Effect

```rust
phraselet! {
    消散效果(target: Predicate) = "{消散}一个{target}。"
    // "消散一个敌方角色。"

    消散带条件(target: Predicate, s: Int, op: Operator) =
        "{消散}一个火花{s}{op}的{target}。"
    // "消散一个火花3以上的敌方角色。"
}
```

### Reclaim

```rust
phraselet! {
    // Gain reclaim
    获得回收(target: Predicate) =
        "{target}获得{回收}，其费用等于其原本费用。"

    // Reclaim with cost
    获得回收费用(target: Predicate, cost: Int) =
        "{target}获得{回收花费(cost)}。"
}
```

### Card in Void

```rust
phraselet! {
    // Cards in void
    你虚空中的牌 = "你虚空中的牌"
    对手虚空中的牌 = "对手虚空中的牌"

    // Return from void
    从虚空返回(target: Predicate) =
        "将一{target.measure}{target}从你的虚空移回你的手牌。"
    // "将一张牌从你的虚空移回你的手牌。"
}
```

### Triggered Abilities

```rust
phraselet! {
    // Trigger prefixes
    具现化触发 = "▸ <b>具现化:</b>"
    消散触发 = "▸ <b>消散:</b>"
    裁决触发 = "▸ <b>裁决:</b>"

    // Full triggered ability
    具现化效果(effect: String) = "{具现化触发} {effect}"
}
```

---

## Numbers

### Arabic vs Chinese Numerals

Chinese can use either Arabic numerals (1, 2, 3) or Chinese characters (一, 二, 三):

```rust
phraselet! {
    // For game mechanics, Arabic numerals are typically used
    抽牌(count: Int) = "抽{count}张牌。"  // "抽3张牌。"

    // For formal/literary text, Chinese characters might be used
    文字数字(n: Int) = match n {
        1 => "一"
        2 => "二"
        3 => "三"
        4 => "四"
        5 => "五"
        // ...
        _ => "{n}"  // Fall back to Arabic for larger numbers
    }
}
```

### Multipliers

```rust
phraselet! {
    倍数(n: Int) = match n {
        2 => "双倍"     // double
        3 => "三倍"     // triple
        4 => "四倍"     // quadruple
        _ => "{n}倍"    // N times
    }
}
```

---

## Simplifications vs English

| Feature | English | Chinese |
|---------|---------|---------|
| Articles | a/an/the | None |
| Plurals | card/cards | 牌 (same) |
| Gender | he/she/it | 它 (same for objects) |
| Cases | subjective/objective | None |
| Verb conjugation | gains/gained | 获得 (same) |

This means Chinese localization files are often **shorter and simpler** than English ones, with fewer match statements and special cases.

---

## Common Mistakes to Avoid

### Don't Forget Measure Words

```rust
// WRONG - missing measure word
抽牌(count: Int) = "抽{count}牌。"

// CORRECT - includes measure word
抽牌(count: Int) = "抽{count}张牌。"
// or
抽牌(count: Int) = "抽{count:牌}。"  // auto-inserted
```

### Word Order for Modifiers

```rust
// WRONG - English word order
带火花的角色(s: Int) = "角色带火花{s}"

// CORRECT - Chinese word order (modifier + 的 + noun)
带火花的角色(s: Int) = "火花{s}的角色"
```

### Location Expressions

```rust
// WRONG - missing 中 for "in"
虚空的牌 = "你虚空的牌"

// CORRECT - 中 indicates "in/inside"
虚空中的牌 = "你虚空中的牌"
```
