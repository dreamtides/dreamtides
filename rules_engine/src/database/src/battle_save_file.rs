use battle_state::battle::battle_history::BattleHistoryAction;
use battle_state::battle_player::battle_player_state::CreateBattlePlayer;
use battle_state::battle_player::player_map::PlayerMap;
use core_data::identifiers::BattleId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleSaveFile {
    pub id: BattleId,
    pub seed: u64,
    pub player_types: PlayerMap<CreateBattlePlayer>,
    pub actions: Vec<BattleHistoryAction>,
}
