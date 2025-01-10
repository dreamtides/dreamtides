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
    YourCardsCostIncrease {
        matching: CardPredicate,
        reduction: Energy,
    },
    YourCardsCostReduction {
        matching: CardPredicate,
        reduction: Energy,
    },
    DisableEnemyMaterializedAbilities,
    EnemyCardsCostIncrease {
        matching: CardPredicate,
        increase: Energy,
    },
    OncePerTurnPlayFromVoid {
        matching: CardPredicate,
    },
    OtherCharactersSparkBonus {
        matching: CardPredicate,
        added_spark: Spark,
    },
    HasAllCharacterTypes,
    PlayFromVoidForCost {
        energy_cost: Energy,
        additional_cost: Cost,
    },
    PlayFromVoidWithConditionAndCost {
        condition: Condition,
        energy_cost: Energy,
        additional_cost: Cost,
    },
    PlayForAlternateCost(AlternateCost),
    Reclaim {
        cost: Option<Cost>,
    },
    SparkEqualToPredicateCount {
        predicate: Predicate,
    },
    CharactersInHandHaveFast,
    JudgmentTriggersWhenMaterialized {
        predicate: Predicate,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternateCost {
    pub condition: Option<Condition>,
    pub energy_cost: Energy,
    pub additional_cost: Cost,
    pub if_you_do: Option<Effect>,
}
