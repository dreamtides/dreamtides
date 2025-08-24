use battle_state::battle::battle_state::BattleState;
use core_data::identifiers::QuestId;
use serde::{Deserialize, Serialize};

use crate::battle_save_file::BattleSaveFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestSaveFileV2 {
    pub id: QuestId,
    pub battle: Option<BattleState>,
    pub replay: Option<BattleSaveFile>,
}
