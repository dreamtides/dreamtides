use core_data::card_types::CharacterType;
use core_data::numerics::{Energy, Spark};
use serde::{Deserialize, Serialize};

/// Specifies which game object is being affected by a card.
///
/// This is used for both targeting constraints as well as describing the
/// implicit target of an effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Predicate {
    /// Predicate which matches the owning card, e.g. "Whenever you draw a
    /// card, this character gains +1 spark".
    This,

    /// Card referenced by this effect. This is typically used for applying
    /// multiple effects to the same target, e.g. "banish a character you
    /// control, then materialize it." or when referencing a card triggering
    /// itself, like "when you discard this card, materialize it."
    It,

    /// All cards referenced by this effect, as in "Banish any number of cards,
    /// then materialize them".
    Them,

    /// Card which triggered an event. Used for applying effects to the
    /// triggering card, e.g. "Whenever you materialize a spirit animal, that
    /// character gains +1 spark."
    That,

    /// Cards controlled by the enemy matching a given predicate.
    Enemy(CardPredicate),

    /// Another card controlled by the owner matching a predicate.
    Another(CardPredicate),

    /// Any card controlled by the owner matching a predicate.
    Your(CardPredicate),

    /// Any card matching a predicate.
    Any(CardPredicate),

    /// Any other card matching a predicate, including enemy cards.
    AnyOther(CardPredicate),

    /// Any card in your void matching a predicate.
    YourVoid(CardPredicate),

    /// Any card in the enemy void matching a predicate.
    EnemyVoid(CardPredicate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CardPredicate {
    Card,
    Character,
    Event,
    CardOnStack,
    CharacterType(CharacterType),
    NotCharacterType(CharacterType),
    CharacterWithSpark(Spark, Operator<Spark>),
    CardWithCost {
        target: Box<CardPredicate>,
        cost_operator: Operator<Energy>,
        cost: Energy,
    },
    CharacterWithCostComparedToControlled {
        target: Box<CardPredicate>,
        cost_operator: Operator<Energy>,
        count_matching: Box<CardPredicate>,
    },
    CharacterWithCostComparedToAbandoned {
        target: Box<CardPredicate>,
        cost_operator: Operator<Energy>,
    },
    CharacterWithSparkComparedToAbandoned {
        target: Box<CardPredicate>,
        spark_operator: Operator<Spark>,
    },
    CharacterWithSparkComparedToAbandonedCountThisTurn {
        target: Box<CardPredicate>,
        spark_operator: Operator<Spark>,
    },
    CharacterWithMaterializedAbility,
    Fast {
        target: Box<CardPredicate>,
    },
    CharacterWithMultiActivatedAbility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operator<T> {
    LowerBy(T),
    OrLess,
    Exactly,
    OrMore,
    HigherBy(T),
}
