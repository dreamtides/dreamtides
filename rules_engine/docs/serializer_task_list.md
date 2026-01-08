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

## Part 4: Other Serializer Features

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
