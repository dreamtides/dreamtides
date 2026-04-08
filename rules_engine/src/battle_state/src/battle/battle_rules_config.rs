use core_data::numerics::Points;
use serde::{Deserialize, Serialize};

/// Maximum number of slots per rank, used for fixed-size array allocation.
pub const MAX_ROW_SIZE: usize = 8;

/// Global configuration for the rules of a battle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BattleRulesConfig {
    /// The number of points required to win the battle.
    pub points_to_win: Points,

    /// Number of front-row slots per player.
    #[serde(default = "default_front_row_size")]
    pub front_row_size: usize,

    /// Number of back-row slots per player.
    #[serde(default = "default_back_row_size")]
    pub back_row_size: usize,
}

fn default_front_row_size() -> usize {
    4
}

fn default_back_row_size() -> usize {
    5
}

impl BattleRulesConfig {
    /// Maximum characters allowed on the battlefield.
    pub fn character_limit(&self) -> usize {
        self.front_row_size + self.back_row_size
    }
}
