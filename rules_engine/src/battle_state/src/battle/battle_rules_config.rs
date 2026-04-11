use core_data::numerics::Points;
use serde::{Deserialize, Serialize};

/// Maximum number of slots per rank, used for fixed-size array allocation.
pub const MAX_ROW_SIZE: usize = 8;

/// Balance compensation mode for second player advantage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BalanceMode {
    /// No compensation beyond the existing turn-0 draw skip.
    #[default]
    None,
    /// P2 draws 6 opening cards instead of 5.
    ExtraCard,
    /// P2 starts with +1 produced and current energy.
    BonusEnergy,
    /// P2 starts with +1 produced and current energy, but skips the turn-1
    /// draw.
    BonusEnergyNoDraw,
    /// P2 starts with 3 victory points.
    BonusPoints,
    /// P2's characters skip summoning sickness on their first turn.
    NoSickness,
    /// P2 receives a 0-cost fast "Balance Coin" event (gain 1 energy).
    Coin,
}

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

    /// Balance compensation for the second player.
    #[serde(default)]
    pub balance_mode: BalanceMode,
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
