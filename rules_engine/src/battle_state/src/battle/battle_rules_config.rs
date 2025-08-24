use core_data::numerics::Points;
use serde::{Deserialize, Serialize};

/// Global configuration for the rules of a battle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BattleRulesConfig {
    /// The number of points required to win the battle.
    pub points_to_win: Points,
}
