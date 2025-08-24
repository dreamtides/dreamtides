use core_data::identifiers::UserId;
use serde::{Deserialize, Serialize};

use crate::quest_save_file::QuestSaveFileV2;

/// Represents the entirety of a user's game state.
///
/// Terminology Note: If someone has multiple save files, we think of these as
/// separate "users", even if they are actually the same human.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SaveFile {
    V2(Box<SaveFileV2>),
}

impl SaveFile {
    pub fn id(&self) -> UserId {
        match self {
            SaveFile::V2(v2) => v2.id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveFileV2 {
    pub id: UserId,
    pub quest: Option<QuestSaveFileV2>,
}
