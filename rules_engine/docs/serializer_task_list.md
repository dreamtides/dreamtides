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

### Task 7: Implement Missing serialize_enemy_predicate Variants

**Location:** `predicate_serializer.rs` lines 61-96

**Variants to add:**
```rust
CardPredicate::CharacterType(_) => "enemy {subtype}".to_string(),
CardPredicate::NotCharacterType(_) => "enemy that is not {a-subtype}".to_string(),
CardPredicate::CharacterWithMaterializedAbility => "enemy with a {materialized} ability".to_string(),
CardPredicate::CharacterWithMultiActivatedAbility => "enemy with an activated ability".to_string(),
CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
    format!("{} with cost less than the abandoned ally's cost", serialize_enemy_predicate(target))
}
CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
    format!(
        "{} with spark less than the number of allies abandoned this turn",
        serialize_enemy_predicate(target)
    )
}
CardPredicate::Fast { target } => format!("fast {}", serialize_enemy_predicate(target)),
```

---

### Task 8: Implement Missing serialize_card_predicate Variants

**Location:** `predicate_serializer.rs` line 145

**Variants to add:**
```rust
CardPredicate::NotCharacterType(_) => "a character that is not {a-subtype}".to_string(),
CardPredicate::CharacterWithSpark(_, operator) => {
    format!("a character with spark {{s}} {}", serialize_operator(operator))
}
CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
    format!(
        "{} with cost less than the number of allied {}",
        serialize_card_predicate(target),
        serialize_card_predicate_plural(count_matching)
    )
}
CardPredicate::CharacterWithCostComparedToAbandoned { target, .. } => {
    format!("{} with cost less than the abandoned ally's cost", serialize_card_predicate(target))
}
CardPredicate::CharacterWithSparkComparedToAbandoned { target, .. } => {
    format!("{} with spark less than the abandoned ally's spark", serialize_card_predicate(target))
}
CardPredicate::CharacterWithSparkComparedToAbandonedCountThisTurn { target, .. } => {
    format!(
        "{} with spark less than the number of allies abandoned this turn",
        serialize_card_predicate(target)
    )
}
CardPredicate::CharacterWithCostComparedToVoidCount { target, .. } => {
    format!("{} with cost less than the number of cards in your void", serialize_card_predicate(target))
}
```

Also add these to `serialize_card_predicate_plural` (line 169) and `serialize_fast_target` (line 190).

---

### Task 9: Implement Missing serialize_for_each_predicate Variants

**Location:** `predicate_serializer.rs` lines 194-201

**Variants to add:**
```rust
Predicate::Your(CardPredicate::Character) => "ally you control".to_string(),
Predicate::Your(CardPredicate::CharacterType(_)) => "allied {subtype} you control".to_string(),
Predicate::Enemy(CardPredicate::Character) => "enemy".to_string(),
Predicate::Any(CardPredicate::Character) => "character".to_string(),
Predicate::Any(CardPredicate::Card) => "card".to_string(),
Predicate::YourVoid(CardPredicate::Card) => "card in your void".to_string(),
Predicate::This => "this character".to_string(),
Predicate::It => "that character".to_string(),
```

---

## Part 3: Effect Serialization Features

### Task 10: Implement Collection-Based Effect Variants

**Location:** `effect_serializer.rs`

**DissolveCharactersCount (lines 140-151):**
```rust
StandardEffect::DissolveCharactersCount { target, count } => match count {
    CollectionExpression::All => format!("{{Dissolve}} all {}.", serialize_predicate_plural(target)),
    CollectionExpression::Exactly(n) => format!("{{Dissolve}} {} {}.", n, serialize_predicate_plural(target)),
    CollectionExpression::UpTo(n) => format!("{{Dissolve}} up to {} {}.", n, serialize_predicate_plural(target)),
    CollectionExpression::AnyNumberOf => format!("{{Dissolve}} any number of {}.", serialize_predicate_plural(target)),
    _ => format!("{{Dissolve}} {}.", serialize_predicate(target)),
},
```

**BanishCollection (lines 155-160):**
```rust
StandardEffect::BanishCollection { target, count } => match count {
    CollectionExpression::AnyNumberOf => format!("{{Banish}} any number of {}.", serialize_predicate_plural(target)),
    CollectionExpression::All => format!("{{Banish}} all {}.", serialize_predicate_plural(target)),
    CollectionExpression::Exactly(n) => format!("{{Banish}} {} {}.", n, serialize_predicate_plural(target)),
    CollectionExpression::UpTo(n) => format!("{{Banish}} up to {} {}.", n, serialize_predicate_plural(target)),
    _ => format!("{{Banish}} {}.", serialize_predicate(target)),
},
```

**MaterializeCollection (lines 246-254):**
```rust
StandardEffect::MaterializeCollection { target, count } => match (target, count) {
    (Predicate::Them, CollectionExpression::All) => "{Materialize} them.".to_string(),
    (_, CollectionExpression::All) => format!("{{Materialize}} all {}.", serialize_predicate_plural(target)),
    (_, CollectionExpression::AnyNumberOf) => format!("{{Materialize}} any number of {}.", serialize_predicate_plural(target)),
    (_, CollectionExpression::UpTo(n)) => format!("{{Materialize}} up to {} {}.", n, serialize_predicate_plural(target)),
    _ => format!("{{Materialize}} {}.", serialize_predicate(target)),
},
```

---

### Task 11: Implement Materialize Copy and Figment Effects

**Location:** `effect_serializer.rs`

**MaterializeSilentCopy (lines 187-192):**
```rust
StandardEffect::MaterializeSilentCopy { target, count, quantity } => {
    match (count, quantity) {
        (1, QuantityExpression::Matching(_)) => {
            format!("{{Materialize}} a copy of {}.", serialize_predicate(target))
        }
        (n, QuantityExpression::Matching(_)) if *n > 1 => {
            format!("{{Materialize}} {} copies of {}.", n, serialize_predicate(target))
        }
        (_, quantity_expr) => {
            format!(
                "{{Materialize}} copies of {} equal to the number of {}.",
                serialize_predicate(target),
                serialize_for_count_expression(quantity_expr)
            )
        }
    }
}
```

**MaterializeFigmentsQuantity (lines 201-209):**
```rust
StandardEffect::MaterializeFigmentsQuantity { figment, count, quantity } => {
    let figment_text = if *count == 1 { "{a-figment}" } else { "{n-figments}" };
    match quantity {
        QuantityExpression::PlayedThisTurn(_) => {
            format!("{{Materialize}} {} for each card you have played this turn.", figment_text)
        }
        QuantityExpression::Matching(predicate) => {
            format!("{{Materialize}} {} for each {}.", figment_text, serialize_for_each_predicate(predicate))
        }
        _ => format!("{{Materialize}} {} for each {}.", figment_text, serialize_for_count_expression(quantity)),
    }
}
```

---

### Task 12: Implement TriggerJudgmentAbility and PutCardsFromVoid Effects

**Location:** `effect_serializer.rs`

**TriggerJudgmentAbility (lines 280-290):**
```rust
StandardEffect::TriggerJudgmentAbility { matching, collection } => match collection {
    CollectionExpression::All => {
        format!("trigger the {{Judgment}} ability of each {}.", predicate_base_text(matching).without_article())
    }
    CollectionExpression::Exactly(1) => {
        format!("trigger the {{Judgment}} ability of {}.", serialize_predicate(matching))
    }
    CollectionExpression::Exactly(n) => {
        format!("trigger the {{Judgment}} ability of {} {}.", n, serialize_predicate_plural(matching))
    }
    _ => format!("trigger the {{Judgment}} ability of {}.", serialize_predicate(matching)),
},
```

**PutCardsFromVoidOnTopOfDeck (line 121):**
```rust
StandardEffect::PutCardsFromVoidOnTopOfDeck { matching, count } => {
    if *count == 1 {
        format!("put {} from your void on top of your deck.", serialize_card_predicate(matching))
    } else {
        format!("put {{up-to-{}-cards}} {} from your void on top of your deck.", count, serialize_card_predicate_plural(matching))
    }
}
```

**GainEnergyEqualToCost (lines 54-59):**
```rust
StandardEffect::GainEnergyEqualToCost { target } => match target {
    Predicate::It | Predicate::That => "gain {energy-symbol} equal to that character's cost.".to_string(),
    Predicate::This => "gain {energy-symbol} equal to this character's cost.".to_string(),
    _ => format!("gain {{energy-symbol}} equal to {}'s cost.", serialize_predicate(target)),
},
```

---

### Task 13: Implement Abandon Effects

**Location:** `effect_serializer.rs` line 297 (catch-all)

**Implementation:**
```rust
StandardEffect::AbandonAndGainEnergyForSpark { target, .. } => {
    format!("abandon {} and gain {{e}} for each spark it had.", serialize_predicate(target))
}
StandardEffect::AbandonAtEndOfTurn { target } => {
    format!("abandon {} at end of turn.", serialize_predicate(target))
}
```

---

### Task 14: Implement Banish Effects

**Location:** `effect_serializer.rs` line 297 (catch-all)

**Implementation:**
```rust
StandardEffect::BanishWhenLeavesPlay { target } => {
    format!("{{Banish}} {} when it leaves play.", serialize_predicate(target))
}
```

---

### Task 15: Implement Dissolve and Prevent Effects

**Location:** `effect_serializer.rs` line 297 (catch-all)

**Implementation:**
```rust
StandardEffect::DissolveCharactersQuantity { target, quantity } => {
    format!(
        "{{Dissolve}} {} equal to the number of {}.",
        serialize_predicate_plural(target),
        serialize_for_count_expression(quantity)
    )
}
StandardEffect::PreventDissolveThisTurn { target } => {
    format!("{} cannot be {{dissolved}} this turn.", serialize_predicate(target))
}
```

---

### Task 16: Implement Enemy and Points Effects

**Location:** `effect_serializer.rs` line 297 (catch-all)

**Implementation:**
```rust
StandardEffect::EnemyGainsPointsEqualToItsSpark => {
    "the opponent gains points equal to its spark.".to_string()
}
StandardEffect::EnemyLosesPoints { .. } => "the opponent loses {points}.".to_string(),
```

---

### Task 17: Implement Spark Gain Effects

**Location:** `effect_serializer.rs` line 297 (catch-all)

**Implementation:**
```rust
StandardEffect::GainsAegisThisTurn { target } => {
    format!("{} gains {{Aegis}} this turn.", serialize_predicate(target))
}
StandardEffect::GainsSparkUntilYourNextMainForEach { target, for_each, .. } => {
    format!(
        "{} gains +{{s}} spark until your next main phase for each {}.",
        serialize_predicate(target),
        serialize_for_each_predicate(for_each)
    )
}
```

---

### Task 18: Implement Energy and Materialize Effects

**Location:** `effect_serializer.rs` line 297 (catch-all)

**Implementation:**
```rust
StandardEffect::GainTwiceThatMuchEnergyInstead => {
    "gain twice that much {energy-symbol} instead.".to_string()
}
StandardEffect::MaterializeCharacterFromVoid { target } => {
    format!("{{Materialize}} {} from your void.", serialize_card_predicate(target))
}
StandardEffect::ThenMaterializeIt => "then {Materialize} it.".to_string(),
```

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
