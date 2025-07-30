use core_data::identifiers::QuestId;
use serde::{Deserialize, Serialize};

use crate::battle_save_file::BattleSaveFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestSaveFile {
    pub id: QuestId,
    pub battle: Option<BattleSaveFile>,
}
