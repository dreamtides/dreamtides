use battle_state::battle::battle_state::BattleState;
use core_data::identifiers::QuestId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestSaveFile {
    pub id: QuestId,
    pub battle: Option<BattleState>,
}
