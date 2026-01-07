# Parser Data Types Reference

This document provides a complete reference for all ability data types that
parsers produce. These types live in `rules_engine/src/ability_data/src/`.

---

## Ability (Top-Level Type)

The `Ability` enum is the top-level type returned by the parser:

```rust
pub enum Ability {
    Event(EventAbility),      // Immediate effect when event card is played
    Static(StaticAbility),    // Permanent rule modification
    Activated(ActivatedAbility), // "cost: effect" on characters
    Triggered(TriggeredAbility), // "When/Whenever/At..." abilities
    Named(NamedAbility),      // Keyword abilities like Reclaim
}
```

**EventAbility**: Used for event cards.

```rust
pub struct EventAbility {
    pub additional_cost: Option<Cost>,  // Paid when playing, not resolving
    pub effect: Effect,
}
```

---

## Effect

Effects represent game state mutations:

```rust
pub enum Effect {
    Effect(StandardEffect),           // Simple effect
    WithOptions(EffectWithOptions),   // Effect with conditions/costs
    List(Vec<EffectWithOptions>),     // Multiple sequential effects
    Modal(Vec<ModalEffectChoice>),    // "Choose one" effects
}

pub struct EffectWithOptions {
    pub effect: StandardEffect,
    pub optional: bool,               // "You may..." effects
    pub trigger_cost: Option<Cost>,   // "Pay X to..." on resolution
    pub condition: Option<Condition>, // "If X, then..." effects
}

pub struct ModalEffectChoice {
    pub energy_cost: Energy,
    pub effect: Effect,
}
```

---

## StandardEffect (All Effect Variants)

`StandardEffect` contains all possible game effects.

### Card Movement Effects

```rust
DrawCards { count: u32 }
DrawCardsForEach { count: u32, for_each: QuantityExpression }
DrawMatchingCard { predicate: CardPredicate }
DiscardCards { count: u32 }
DiscardCardFromEnemyHand { predicate: CardPredicate }
DiscardCardFromEnemyHandThenTheyDraw { predicate: CardPredicate }
MaterializeCharacter { target: Predicate }
MaterializeCharacterAtEndOfTurn { target: Predicate }
MaterializeCharacterFromVoid { target: CardPredicate }
MaterializeRandomFromDeck { count: u32, predicate: CardPredicate }
MaterializeSilentCopy { target: Predicate, count: u32, quantity: QuantityExpression }
DissolveCharacter { target: Predicate }
DissolveCharactersCount { target: Predicate, count: CollectionExpression }
DissolveCharactersQuantity { target: Predicate, quantity: QuantityExpression }
BanishCharacter { target: Predicate }
BanishCollection { target: Predicate, count: CollectionExpression }
BanishCharacterUntilLeavesPlay { target: Predicate, until_leaves: Predicate }
BanishUntilNextMain { target: Predicate }
BanishWhenLeavesPlay { target: Predicate }
BanishCardsFromEnemyVoid { count: u32 }
BanishEnemyVoid
ReturnToHand { target: Predicate }
ReturnFromYourVoidToHand { target: Predicate }
ReturnFromYourVoidToPlay { target: Predicate }
ReturnUpToCountFromYourVoidToHand { target: Predicate, count: u32 }
PutCardsFromYourDeckIntoVoid { count: u32 }
PutCardsFromVoidOnTopOfDeck { count: u32, matching: CardPredicate }
PutOnTopOfEnemyDeck { target: Predicate }
Copy { target: Predicate }
CopyNextPlayed { matching: Predicate, times: Option<u32> }
```

### Resource Effects

```rust
GainEnergy { gains: Energy }
GainEnergyForEach { gains: Energy, for_each: Predicate }
GainPoints { gains: Points }
GainPointsForEach { gain: Points, for_count: QuantityExpression }
LosePoints { loses: Points }
EnemyGainsPoints { count: u32 }
EnemyGainsPointsEqualToItsSpark
EnemyLosesPoints { count: u32 }
MultiplyYourEnergy { multiplier: u32 }
GainTwiceThatMuchEnergyInstead
```

### Spark Effects

```rust
GainsSpark { target: Predicate, gains: Spark }
GainsSparkForQuantity { target: Predicate, gains: Spark, for_quantity: QuantityExpression }
GainsSparkUntilYourNextMainForEach { target: Predicate, gains: Spark, for_each: Predicate }
Kindle { amount: Spark }
SparkBecomes { collection: CollectionExpression, matching: CardPredicate, spark: Spark }
EachMatchingGainsSparkForEach { each: CardPredicate, gains: Spark, for_each: CardPredicate }
EachMatchingGainsSparkUntilNextMain { each: CardPredicate, gains: Spark }
```

### Game Effects

```rust
Foresee { count: u32 }
Discover { predicate: CardPredicate }
DiscoverAndThenMaterialize { predicate: CardPredicate }
Counterspell { target: Predicate }
CounterspellUnlessPaysCost { target: Predicate, cost: Cost }
GainControl { target: Predicate }
DisableActivatedAbilitiesWhileInPlay { target: Predicate }
TakeExtraTurn
YouWinTheGame
ShuffleHandAndDeckAndDraw { count: u32 }
```

### Reclaim Effects

```rust
GainsReclaimUntilEndOfTurn { target: Predicate, cost: Option<Energy> }
CardsInVoidGainReclaimThisTurn { count: CollectionExpression, predicate: CardPredicate }
```

### Other Effects

```rust
CreateTriggerUntilEndOfTurn { trigger: Box<TriggeredAbility> }
TriggerJudgmentAbility { matching: Predicate, collection: CollectionExpression }
AbandonAtEndOfTurn { target: Predicate }
AbandonAndGainEnergyForSpark { target: Predicate, energy_per_spark: Energy }
EachPlayerAbandonsCharacters { matching: CardPredicate, count: u32 }
EachPlayerDiscardCards { count: u32 }
GainsAegisThisTurn { target: Predicate }
PreventDissolveThisTurn { target: Predicate }
PayCost { cost: Cost }
OpponentPaysCost { cost: Cost }
SpendAllEnergyDissolveEnemy
SpendAllEnergyDrawAndDiscard
ThenMaterializeIt
NoEffect
```

---

## TriggeredAbility

Triggered abilities fire when specific game events occur:

```rust
pub struct TriggeredAbility {
    pub trigger: TriggerEvent,
    pub effect: Effect,
    pub options: Option<TriggeredAbilityOptions>,
}

pub struct TriggeredAbilityOptions {
    pub once_per_turn: bool,      // "Once per turn, when..."
    pub until_end_of_turn: bool,  // Created triggers that expire
}
```

---

## TriggerEvent

Specifies what game events cause a triggered ability to fire:

```rust
pub enum TriggerEvent {
    Keywords(Vec<TriggerKeyword>),    // {Materialized}, {Judgment}, etc.
    Abandon(Predicate),               // "When you abandon..."
    Banished(Predicate),              // "When X is banished..."
    Discard(Predicate),               // "When you discard..."
    Dissolved(Predicate),             // "When X is dissolved..."
    Materialize(Predicate),           // "When you materialize..."
    MaterializeNthThisTurn(Predicate, u32), // "When you materialize Nth..."
    Play(Predicate),                  // "When you play..."
    PlayFromHand(Predicate),          // "When you play from hand..."
    PlayDuringTurn(Predicate, PlayerTurn), // "When you play N in a turn..."
    EndOfYourTurn,                    // "At the end of your turn..."
    GainEnergy,                       // "When you gain energy..."
    DrawAllCardsInCopyOfDeck,         // "When you have no cards in deck..."
}

pub enum TriggerKeyword {
    Materialized,  // Triggers when this card enters play
    Judgment,      // Triggers during judgment phase
    Dissolved,     // Triggers when this card is destroyed
}

pub enum PlayerTurn {
    YourTurn,
    EnemyTurn,
}
```

---

## ActivatedAbility

Activated abilities allow paying costs for effects:

```rust
pub struct ActivatedAbility {
    pub costs: Vec<Cost>,
    pub effect: Effect,
    pub options: Option<ActivatedAbilityOptions>,
}

pub struct ActivatedAbilityOptions {
    pub is_fast: bool,   // Can respond to enemy actions
    pub is_multi: bool,  // Can activate multiple times per turn
}
```

---

## Cost

Costs represent what must be paid:

```rust
pub enum Cost {
    Energy(Energy),
    AbandonCharactersCount { target: Predicate, count: CollectionExpression },
    AbandonDreamscapes(u32),
    DiscardCards(CardPredicate, u32),
    DiscardHand,
    BanishCardsFromYourVoid(u32),
    BanishCardsFromEnemyVoid(u32),
    BanishAllCardsFromYourVoid,
    BanishFromHand(Predicate),
    Choice(Vec<Cost>),  // Alternative costs separated by "or"
    ReturnToHand { target: Predicate, count: CollectionExpression },
    SpendOneOrMoreEnergy,
    CostList(Vec<Cost>),  // Multiple costs combined
}
```

---

## StaticAbility

Static abilities modify game rules while in play:

```rust
pub enum StaticAbility {
    StaticAbility(StandardStaticAbility),
    WithOptions(StaticAbilityWithOptions),
}

pub enum StandardStaticAbility {
    YourCardsCostReduction { matching: CardPredicate, reduction: Energy },
    YourCardsCostIncrease { matching: CardPredicate, increase: Energy },
    EnemyCardsCostIncrease { matching: CardPredicate, increase: Energy },
    SparkBonusYourCharacters { matching: CardPredicate, added_spark: Spark },
    SparkBonusOtherCharacters { matching: CardPredicate, added_spark: Spark },
    SparkEqualToPredicateCount { predicate: Predicate },
    DisableEnemyMaterializedAbilities,
    HasAllCharacterTypes,
    CharactersInHandHaveFast,
    CardsInYourVoidHaveReclaim { matching: CardPredicate },
    OncePerTurnPlayFromVoid { matching: CardPredicate },
    PlayFromVoid(PlayFromVoid),
    PlayOnlyFromVoid,
    PlayForAlternateCost(AlternateCost),
    JudgmentTriggersWhenMaterialized { predicate: Predicate },
    CostReductionForEach { reduction: Energy, quantity: QuantityExpression },
    YouMayLookAtTopCardOfYourDeck,
    YouMayPlayFromTopOfDeck { matching: CardPredicate },
}
```

---

## Predicate

Predicates specify which game objects are targeted:

```rust
pub enum Predicate {
    This,                    // "this character"
    It,                      // Previously referenced card
    Them,                    // Previously referenced cards (plural)
    That,                    // Card that triggered the ability
    Enemy(CardPredicate),    // "an enemy" / "enemy characters"
    Another(CardPredicate),  // "another character you control"
    Your(CardPredicate),     // "a character you control"
    Any(CardPredicate),      // "a character" (any controller)
    AnyOther(CardPredicate), // "another character" (any controller)
    YourVoid(CardPredicate), // "a card in your void"
    EnemyVoid(CardPredicate),// "a card in the enemy void"
}
```

---

## CardPredicate

CardPredicates filter cards by type and attributes:

```rust
pub enum CardPredicate {
    Card,                          // Any card
    Character,                     // Any character
    Event,                         // Any event
    CharacterType(CardSubtype),    // "a Warrior"
    NotCharacterType(CardSubtype), // "a non-Warrior"
    CharacterWithSpark(Spark, Operator<Spark>),
    CardWithCost { target: Box<CardPredicate>, cost_operator: Operator<Energy>, cost: Energy },
    CharacterWithCostComparedToControlled { target: Box<CardPredicate>, cost_operator: Operator<Energy>, count_matching: Box<CardPredicate> },
    CharacterWithCostComparedToAbandoned { target: Box<CardPredicate>, cost_operator: Operator<Energy> },
    CharacterWithSparkComparedToAbandoned { target: Box<CardPredicate>, spark_operator: Operator<Spark> },
    CharacterWithSparkComparedToAbandonedCountThisTurn { target: Box<CardPredicate>, spark_operator: Operator<Spark> },
    CharacterWithMaterializedAbility,
    CharacterWithMultiActivatedAbility,
    Fast { target: Box<CardPredicate> },  // "a fast card"
}

pub enum Operator<T> {
    LowerBy(T),  // "X lower"
    OrLess,      // "X or less"
    Exactly,     // "exactly X"
    OrMore,      // "X or more"
    HigherBy(T), // "X higher"
}
```

---

## CollectionExpression

Collection expressions describe variable quantities of targets:

```rust
pub enum CollectionExpression {
    All,          // "all characters"
    EachOther,    // "each other character"
    AnyNumberOf,  // "any number of characters"
    AllButOne,    // "all but one character"
    UpTo(u32),    // "up to N characters"
    Exactly(u32), // "exactly N characters"
    OrMore(u32),  // "N or more characters"
}
```

---

## QuantityExpression

Quantity expressions describe counts based on game state:

```rust
pub enum QuantityExpression {
    Matching(Predicate),               // Count of matching cards
    AbandonedThisTurn(CardPredicate),  // Cards abandoned this turn
    AbandonedThisWay(CardPredicate),   // Cards abandoned by this effect
    CardsDrawnThisTurn(CardPredicate),
    DiscardedThisTurn(CardPredicate),
    DissolvedThisTurn(CardPredicate),
    PlayedThisTurn(CardPredicate),
    ReturnedToHandThisWay(CardPredicate), // Cards returned to hand by this effect
    ForEachEnergySpentOnThisCard,
}
```

---

## Condition

Conditions are boolean predicates for conditional effects:

```rust
pub enum Condition {
    CardsDiscardedThisTurn { count: u32 },
    CardsDrawnThisTurn { count: u32 },
    CardsInVoidCount { count: u32 },
    DissolvedThisTurn { predicate: Predicate },
    PredicateCount { count: u32, predicate: Predicate },
    ThisCharacterIsInYourVoid,
}
```

---

## NamedAbility

Named abilities are keywords that expand to full ability text:

```rust
pub enum NamedAbility {
    Reclaim(Option<Energy>),  // Play from void, banish when leaves play
}
```

---

## Core Numeric Types

These types from `core_data::numerics` represent game values:

```rust
pub struct Energy(pub u32);  // Energy cost/gain
pub struct Points(pub u32);  // Victory points
pub struct Spark(pub i32);   // Character power (can be negative)
```
