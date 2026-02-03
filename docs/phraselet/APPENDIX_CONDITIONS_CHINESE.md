# Appendix: Localizing Condition Serializer for Simplified Chinese

This appendix analyzes localizing `condition_serializer.rs` to Simplified Chinese using Phraselet, focusing on word order, measure words, and aspect markers.

## Chinese Grammatical Properties

Chinese differs fundamentally from English and Russian:

| Feature | English | Russian | Chinese |
|---------|---------|---------|---------|
| Pluralization | yes (card/cards) | yes (карта/карты/карт) | **no** |
| Grammatical case | no | yes (6 cases) | **no** |
| Grammatical gender | no | yes (3 genders) | **no** |
| Measure words | no | no | **yes** (张, 个, etc.) |
| Word order | SVO, fixed | SVO, flexible | SVO, **topic-prominent** |
| Tense marking | yes (has/had) | yes | **no** (uses aspect) |

This makes Chinese simpler in some ways (no inflection) but introduces **measure words** (classifiers) as a new concern.

---

## The Core Challenge: Measure Words

In English: "3 cards", "2 characters"

In Chinese, you cannot directly combine a number with a noun. A **measure word** (量词) must intervene:

| Noun | Measure Word | Example |
|------|--------------|---------|
| 牌 (card) | 张 (zhāng) | 3张牌 (3 MW cards) |
| 角色 (character) | 个/名 (gè/míng) | 2个角色 (2 MW characters) |
| 事件 (event) | 个 (gè) | 1个事件 (1 MW event) |

The measure word is determined by the **noun type**, so this is Rust logic that selects the appropriate Phraselet phrase.

---

## Word Order Differences

English places time expressions at the end; Chinese places them before the verb:

| English | Chinese |
|---------|---------|
| "if you have discarded a card **this turn**" | "如果你**本回合**弃置了一张牌" |
| "if you have drawn 3 cards **this turn**" | "如果你**本回合**抽了3张牌" |

The phrase templates simply have different structures—this is handled naturally by having separate templates per language.

---

## Aspect Markers vs. Tense

English uses tense ("have discarded" = perfect aspect). Chinese uses the aspect marker 了 (le):

- 弃置 (discard) → 弃置了 (discarded / have discarded)
- 抽 (draw) → 抽了 (drew / have drawn)

This is just part of the verb in the Chinese phrase template.

---

## English Phraselet File

```rust
// en.phr.rs
phraselet! {
    //==========================================================================
    // CONDITION PHRASES
    //==========================================================================

    // "with X allies that share a character type,"
    with_allies_sharing_type(count) = "with {count} that share a character type,";

    // "if you have discarded X this turn"
    if_discarded_this_turn(target) = "if you have discarded {target} this turn";

    // "if you have drawn N or more cards this turn"
    if_drawn_n_or_more(n) = "if you have drawn {n} or more cards this turn";

    // "while you have N or more cards in your void,"
    while_cards_in_void(n) = "while you have {n} or more cards in your void,";

    // "if a character dissolved this turn"
    if_character_dissolved = "if a character dissolved this turn";

    // "with an allied {subtype},"
    with_allied_subtype(subtype) = "with an allied {subtype},";

    // "while this card is in your void,"
    while_this_in_void = "while this card is in your void,";

    // "with {predicate},"
    with_predicate(predicate) = "with {predicate},";

    //==========================================================================
    // COUNTING PATTERNS
    //==========================================================================

    count_allies(n) = "{n} other allies";
    count_allied_subtype(n, subtype) = "{n} other allied {subtype:other}";
}
```

---

## Chinese Phraselet File

```rust
// zh_cn.phr.rs
phraselet! {
    //==========================================================================
    // CONDITION PHRASES
    // Note: Time expressions ("this turn") come BEFORE the verb in Chinese
    //==========================================================================

    // "拥有{count}个共享角色类型的友军时，"
    // (when possessing {count} allies sharing character type)
    with_allies_sharing_type(count) = "拥有{count}个共享角色类型的友军时，";

    // "如果你本回合弃置了{target}"
    // (if you this-turn discarded {target})
    if_discarded_this_turn(target) = "如果你本回合弃置了{target}";

    // "如果你本回合抽了{n}张以上的牌"
    // (if you this-turn drew {n}-MW-or-more cards)
    if_drawn_n_or_more(n) = "如果你本回合抽了{n}张以上的牌";

    // "当你的虚空中有{n}张以上的牌时，"
    // (when your void has {n}-MW-or-more cards)
    // Note: "当...时" is a circumfix (wraps the condition)
    while_cards_in_void(n) = "当你的虚空中有{n}张以上的牌时，";

    // "如果本回合有角色消散"
    // (if this-turn has character dissolved)
    if_character_dissolved = "如果本回合有角色消散";

    // "拥有一个友方{subtype}时，"
    // (when possessing one allied {subtype})
    with_allied_subtype(subtype) = "拥有一个友方{subtype}时，";

    // "当此牌在你的虚空中时，"
    // (when this card is in your void)
    while_this_in_void = "当此牌在你的虚空中时，";

    // "拥有{predicate}时，"
    with_predicate(predicate) = "拥有{predicate}时，";

    //==========================================================================
    // COUNTING PATTERNS WITH MEASURE WORDS
    //==========================================================================

    // "{n}个其他友军" ({n} MW other allies)
    count_allies(n) = "{n}个其他友军";

    // "{n}个其他友方{subtype}" ({n} MW other allied {subtype})
    count_allied_subtype(n, subtype) = "{n}个其他友方{subtype}";

    //==========================================================================
    // MEASURE WORD COMPOSITION
    // Used by Rust code to build counted noun phrases
    //==========================================================================

    // 张 (zhāng) - for flat objects: cards, tickets, paper
    mw_zhang(n, thing) = "{n}张{thing}";

    // 个 (gè) - general classifier: characters, events, abilities
    mw_ge(n, thing) = "{n}个{thing}";

    // 名 (míng) - for people/characters (more formal)
    mw_ming(n, thing) = "{n}名{thing}";

    //==========================================================================
    // BASE NOUNS (no inflection needed)
    //==========================================================================

    card = "牌";
    character = "角色";
    event = "事件";
    ally = "友军";
    enemy = "敌人";
}
```

---

## Design Insight: Measure Words are Rust Logic

The measure word selection depends on the **semantic category** of the noun:

```rust
// zh_cn serializer logic
fn serialize_counted_noun(noun_type: NounType, count: i32) -> String {
    match noun_type {
        NounType::Card => zh_cn::mw_zhang(count, zh_cn::card()),
        NounType::Character => zh_cn::mw_ge(count, zh_cn::character()),
        NounType::Event => zh_cn::mw_ge(count, zh_cn::event()),
        // ...
    }
}
```

This is the same pattern as Russian case selection: **Rust decides the grammatical context**, Phraselet provides the text.

---

## Design Insight: Circumfix Patterns

Chinese "while" conditions use a circumfix structure: 当...时 (when...time)

English: `"while X,"` → prefix only
Chinese: `"当X时，"` → circumfix (prefix + suffix)

This is handled naturally by the phrase template:

```rust
// English: prefix only
while_cards_in_void(n) = "while you have {n} or more cards in your void,";

// Chinese: the suffix is part of the template
while_cards_in_void(n) = "当你的虚空中有{n}张以上的牌时，";
```

No special Phraselet syntax needed—the circumfix is just part of the phrase text.

---

## Design Insight: No Variants Needed for Chinese

Unlike English (singular/plural) or Russian (case/number/gender), Chinese nouns don't inflect. The Phraselet file is simpler:

```rust
// English needs variants
card = {
    one: "card",
    other: "cards",
};

// Chinese doesn't
card = "牌";
```

However, Chinese does need **multiple phrase patterns** for different measure word contexts:

```rust
// Different patterns for counting different things
mw_zhang(n, thing) = "{n}张{thing}";  // for cards
mw_ge(n, thing) = "{n}个{thing}";      // for characters
```

---

## Full Serialization Example

**Condition:** `Condition::CardsInVoidCount { count: 5 }`

**English flow:**
1. Rust calls `en::while_cards_in_void(5)`
2. Result: "while you have 5 or more cards in your void,"

**Chinese flow:**
1. Rust calls `zh_cn::while_cards_in_void(5)`
2. Result: "当你的虚空中有5张以上的牌时，"

Note: The "5张" (5 MW) is embedded in the phrase template. If the count needed flexible noun types, the Rust code would compose it:

```rust
// More complex case with variable noun type
fn serialize_while_in_void(count: i32, noun: NounType) -> String {
    let counted = serialize_counted_noun(noun, count);
    zh_cn::while_n_in_void(counted)
}
```

With phrase:
```rust
while_n_in_void(counted) = "当你的虚空中有{counted}以上时，";
```

---

## Design Issue Discovered: "or more" Placement

In English, "or more" comes after the number: "5 or more cards"
In Chinese, "以上" comes after the measure word+noun: "5张以上的牌" or "5张牌以上"

If we need flexible composition:

```rust
// English
n_or_more(n, things) = "{n} or more {things}";

// Chinese - "以上" attaches to the counted phrase
n_or_more(counted) = "{counted}以上";
// Usage: n_or_more(mw_zhang(5, card())) → "5张牌以上"
```

Or embed it in the measure word phrase:

```rust
mw_zhang_or_more(n, thing) = "{n}张以上的{thing}";
// Usage: mw_zhang_or_more(5, card()) → "5张以上的牌"
```

**Resolution:** Both approaches work. The choice depends on how often "or more" appears and whether it's always paired with measure words. The current design supports either.

---

## Rust Serializer for Chinese

```rust
// condition_serializer.rs (Chinese implementation)

fn serialize_condition_zh_cn(
    condition: &Condition,
    bindings: &mut VariableBindings
) -> String {
    match condition {
        Condition::CardsInVoidCount { count } => {
            zh_cn::while_cards_in_void(*count)
        }

        Condition::CardsDrawnThisTurn { count } => {
            zh_cn::if_drawn_n_or_more(*count)
        }

        Condition::CardsDiscardedThisTurn { count: 1, predicate } => {
            let target = predicate_serializer::serialize_zh_cn(
                predicate,
                MeasureWord::Zhang,  // cards use 张
                bindings
            );
            zh_cn::if_discarded_this_turn(target)
        }

        Condition::DissolvedThisTurn { .. } => {
            zh_cn::if_character_dissolved()
        }

        Condition::ThisCardIsInYourVoid => {
            zh_cn::while_this_in_void()
        }

        Condition::PredicateCount { count: 1, predicate } => {
            if let Predicate::Another(CardPredicate::CharacterType(subtype)) = predicate {
                zh_cn::with_allied_subtype(subtype.chinese_name())
            } else {
                // ...
            }
        }

        Condition::PredicateCount { count, predicate } => {
            let pred_text = serialize_predicate_count_zh_cn(*count, predicate, bindings);
            zh_cn::with_predicate(pred_text)
        }

        // ...
    }
}

fn serialize_predicate_count_zh_cn(
    count: u32,
    predicate: &Predicate,
    bindings: &mut VariableBindings
) -> String {
    match predicate {
        Predicate::Another(CardPredicate::Character) => {
            zh_cn::count_allies(count)
        }
        Predicate::Another(CardPredicate::CharacterType(subtype)) => {
            zh_cn::count_allied_subtype(count, subtype.chinese_name())
        }
        // ...
    }
}
```

---

## Summary of Design Validations

1. **No variants needed for Chinese nouns.** Simpler Phraselet files since Chinese doesn't inflect.

2. **Measure words are Rust logic.** The semantic category of the noun determines the measure word, which is a code decision.

3. **Word order handled by templates.** Different phrase templates naturally accommodate Chinese word order (time before verb).

4. **Circumfix patterns work naturally.** Chinese "当...时" is just part of the phrase text, no special syntax needed.

5. **"or more" composition is flexible.** Can be embedded in templates or composed via helper phrases.

## Design Validation: Phraselet Scales Down

For languages with less grammatical complexity, Phraselet files are simpler:
- No variant blocks needed
- Fewer phrase templates (no singular/plural variants)
- Measure word selection is Rust logic, not Phraselet variants

The same design that handles Russian's 6 cases × 3 genders × 3 numbers also handles Chinese's simpler noun system—without forcing Chinese translators to deal with unused complexity.
