use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BattleSaveFile {
    pub id: BattleId,
    pub seed: u64,
    pub starting_player: PlayerName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMap<T> {
    pub one: T,
    pub two: T,
}
