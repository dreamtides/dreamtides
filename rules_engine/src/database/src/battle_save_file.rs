use battle_data::battle::battle_history::BattleHistoryAction;
use battle_data::battle_player::player_data::PlayerType;
use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BattleSaveFile {
    pub id: BattleId,
    pub seed: u64,
    pub starting_player: PlayerName,
    pub player_types: PlayerMap<PlayerType>,
    pub actions: Vec<BattleHistoryAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMap<T> {
    pub one: T,
    pub two: T,
}
