# Serializer Task List

This document outlines tasks for improving and expanding the ability serializer in
`src/parser_v2/src/serializer/`. The serializer converts `Ability` data structures into
human-readable rules text strings.

## Current Architecture

The serializer is organized into several modules:
- `ability_serializer.rs` - Top-level entry point for serializing abilities
- `effect_serializer.rs` - Serializes `StandardEffect` and `Effect` types
- `trigger_serializer.rs` - Serializes `TriggerEvent` types
- `cost_serializer.rs` - Serializes `Cost` types
- `static_ability_serializer.rs` - Serializes `StaticAbility` types
- `predicate_serializer.rs` - Serializes `Predicate` and `CardPredicate` types
- `serializer_utils.rs` - Shared utilities (`capitalize_first_letter`, `serialize_operator`)

---

## Part 3: Effect Serialization Features

---

### Task 19: Implement Cost and Utility Effects

**Location:** `effect_serializer.rs` line 297 (catch-all)

**Implementation:**
```rust
StandardEffect::NoEffect => "".to_string(),
StandardEffect::OpponentPaysCost { cost } => {
    format!("the opponent pays {}.", serialize_cost(cost))
}
StandardEffect::PayCost { cost } => format!("pay {}.", serialize_cost(cost)),
StandardEffect::SpendAllEnergyDissolveEnemy => {
    "spend all your {energy-symbol}. {Dissolve} an enemy with cost less than or equal to the amount spent.".to_string()
}
StandardEffect::SpendAllEnergyDrawAndDiscard => {
    "spend all your {energy-symbol}. Draw cards equal to the amount spent, then discard that many cards.".to_string()
}
```

---

## Part 4: Other Serializer Features

### Task 20: Implement Missing QuantityExpression Variants

**Location:** `effect_serializer.rs` lines 452-482

**In `serialize_predicate_count` (lines 452-458):**
```rust
Predicate::Another(CardPredicate::Character) => "{count-allies}".to_string(),
Predicate::Your(card_predicate) => {
    format!("{{count-your-{}}}", card_predicate_base_text(card_predicate).without_article())
}
Predicate::Enemy(CardPredicate::Character) => "{count-enemies}".to_string(),
```

**In `serialize_for_count_expression` (lines 461-482):**
```rust
QuantityExpression::AbandonedThisTurn(CardPredicate::CharacterType(_)) => {
    "allied {subtype} abandoned this turn".to_string()
}
QuantityExpression::AbandonedThisWay(CardPredicate::CharacterType(_)) => {
    "allied {subtype} abandoned this way".to_string()
}
QuantityExpression::ReturnedToHandThisWay(CardPredicate::CharacterType(_)) => {
    "allied {subtype} returned this way".to_string()
}
```

---

### Task 21: Implement CollectionExpression Cases in serialize_cards_in_void_gain_reclaim

**Location:** `effect_serializer.rs` lines 491-509

**Variants to add:**
```rust
CollectionExpression::UpTo(n) => {
    format!(
        "up to {} {} in your void gain {{reclaim}} equal to their cost this turn.",
        n,
        serialize_card_predicate_plural(predicate)
    )
}
CollectionExpression::AnyNumberOf => {
    format!(
        "any number of {} in your void gain {{reclaim}} equal to their cost this turn.",
        serialize_card_predicate_plural(predicate)
    )
}
```

---

### Task 22: Implement TriggerEvent::MaterializeNthThisTurn

**Location:** `trigger_serializer.rs` line 64

**Implementation:**

First, add an ordinal helper to `serializer_utils.rs`:
```rust
pub fn ordinal(n: u32) -> &'static str {
    match n {
        1 => "first",
        2 => "second",
        3 => "third",
        4 => "fourth",
        5 => "fifth",
        _ => "nth",
    }
}
```

Then implement the variant:
```rust
TriggerEvent::MaterializeNthThisTurn(predicate, n) => {
    format!(
        "when you {{materialize}} your {} {} this turn, ",
        ordinal(*n),
        predicate_base_text(predicate).without_article()
    )
}
```

---

### Task 23: Implement Missing Cost Variants

**Location:** `cost_serializer.rs` lines 55 and 59

**Implementation:**
```rust
Cost::AbandonDreamscapes(count) => {
    if *count == 1 {
        "abandon a dreamscape".to_string()
    } else {
        format!("abandon {} dreamscapes", count)
    }
}
Cost::BanishAllCardsFromYourVoid => "{Banish} your void".to_string(),
Cost::CostList(costs) => {
    costs.iter().map(serialize_cost).collect::<Vec<_>>().join(" and ")
}
```

For `ReturnToHand` (line 55), add:
```rust
CollectionExpression::UpTo(n) => {
    format!("return up to {} {} to hand", n, serialize_predicate_plural(target))
}
```

---

### Task 24: Implement Missing StaticAbility Variants

**Location:** `static_ability_serializer.rs` line 143

**Implementation:**
```rust
StandardStaticAbility::CostReductionForEach { quantity, .. } => {
    format!("this card costs {{e}} less for each {}.", serialize_for_count_expression(quantity))
}
StandardStaticAbility::SparkBonusYourCharacters { matching, .. } => {
    format!("your {} have +{{s}} spark.", serialize_card_predicate_plural(matching))
}
StandardStaticAbility::PlayFromVoid(play_from_void) => {
    let mut result = String::new();
    if let Some(cost) = &play_from_void.additional_cost {
        result.push_str(&format!("{}: ", capitalize_first_letter(&serialize_cost(cost))));
    }
    result.push_str("Play this card from your void for {e}");
    if play_from_void.if_you_do.is_some() {
        result.push_str(", then abandon it");
    }
    result.push('.');
    result
}
```

---

## Part 5: Testing

### Task 25: Add Round-Trip Serializer Tests

**Goal:** Create comprehensive tests that verify parsing and serialization produce
equivalent output.

**Implementation:**

Create `src/parser_v2/src/serializer/tests.rs`:

1. For each ability type, create test cases that:
   - Parse rules text into an `Ability`
   - Serialize the `Ability` back to text
   - Verify the text matches or is semantically equivalent

2. Test edge cases:
   - Articles ("a" vs "an") for vowel-starting words
   - Plural forms
   - Capitalization at sentence boundaries
   - Complex nested effects
   - Modal abilities with multiple choices

Example test structure:
```rust
#[test]
fn test_serialize_triggered_ability() {
    let text = "When you play a character, draw a card.";
    let ability = parse_ability(text).unwrap();
    let serialized = serialize_ability(&ability);
    assert_eq!(serialized, text);
}
```
