use core_data::numerics::{Energy, Spark};
use serde::{Deserialize, Serialize};

use crate::condition::Condition;
use crate::cost::Cost;
use crate::effect::Effect;
use crate::predicate::{CardPredicate, Predicate};

/// A static ability represents something which modifies the rules of the game,
/// either for this specific card or globally. Static abilities do not 'happen',
/// they're just something that is always true.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StaticAbility {
    StaticAbility(StandardStaticAbility),
    WithOptions(StaticAbilityWithOptions),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAbilityWithOptions {
    pub ability: StandardStaticAbility,

    /// Indicates an ability which occurs only if some condition is met,
    pub condition: Option<Condition>,
}

/// Basic static abilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StandardStaticAbility {
    CharactersInHandHaveFast,
    DisableEnemyMaterializedAbilities,
    EnemyCardsCostIncrease { matching: CardPredicate, increase: Energy },
    HasAllCharacterTypes,
    JudgmentTriggersWhenMaterialized { predicate: Predicate },
    OncePerTurnPlayFromVoid { matching: CardPredicate },
    PlayForAlternateCost(AlternateCost),
    PlayFromVoid(PlayFromVoid),
    PlayOnlyFromVoid,
    Reclaim { cost: Option<Cost> },
    SparkBonusYourCharacters { matching: CardPredicate, added_spark: Spark },
    SparkBonusOtherCharacters { matching: CardPredicate, added_spark: Spark },
    SparkEqualToPredicateCount { predicate: Predicate },
    YouMayLookAtTopCardOfYourDeck,
    YouMayPlayFromTopOfDeck { matching: CardPredicate },
    YourCardsCostIncrease { matching: CardPredicate, reduction: Energy },
    YourCardsCostReduction { matching: CardPredicate, reduction: Energy },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayFromVoid {
    pub energy_cost: Option<Energy>,
    pub additional_cost: Cost,
    pub if_you_do: Option<Effect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternateCost {
    pub energy_cost: Energy,
    pub additional_cost: Cost,
    pub if_you_do: Option<Effect>,
}
