use core_data::identifiers::BattleId;

use crate::player_data::PlayerData;

/// Contains data types for a "battle", a single instance of playing a match
/// against an enemy.
#[derive(Clone, Debug)]
pub struct BattleData {
    pub id: BattleId,
    pub user: PlayerData,
    pub enemy: PlayerData,
}
