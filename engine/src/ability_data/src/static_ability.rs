use core_data::numerics::Energy;
use serde::{Deserialize, Serialize};

use crate::cost::Cost;
use crate::predicate::CardPredicate;

/// A static ability represents something which modifies the rules of the game,
/// either for this specific card or globally. Static abilities do not 'happen',
/// they're just something that is always true.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StaticAbility {
    EnemyAddedCostToPlay(CardPredicate, Energy),
    OncePerTurnPlayFromVoid(CardPredicate),
    PlayFromVoidForCost { energy_cost: Energy, additional_cost: Cost },
}
