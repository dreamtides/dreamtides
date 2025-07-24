use core_data::numerics::Points;

/// Global configuration for the rules of a battle.
#[derive(Clone, Debug)]
pub struct BattleRulesConfig {
    /// The number of points required to win the battle.
    pub points_to_win: Points,
}
