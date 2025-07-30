use core_data::identifiers::UserId;
use serde::{Deserialize, Serialize};

use crate::quest_save_file::QuestSaveFile;

/// Represents the entirety of a user's game state.
///
/// Terminology Note: If someone has multiple save files, we think of these as
/// separate "users", even if they are actually the same human.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SaveFile {
    V1(SaveFileV1),
}

impl SaveFile {
    pub fn id(&self) -> UserId {
        match self {
            SaveFile::V1(v1) => v1.id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveFileV1 {
    pub id: UserId,
    pub quest: Option<QuestSaveFile>,
}
