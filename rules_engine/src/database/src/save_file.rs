use core_data::identifiers::UserId;
use serde::{Deserialize, Serialize};

use crate::quest_save_file::QuestSaveFile;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FileFormatVersion {
    V1,
}

/// Represents the entirety of a user's game state.
///
/// Terminology Note : If someone has multiple save files, we think of these as
/// separate "users", even if they are actually the same human.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveFile {
    pub id: UserId,
    pub version: FileFormatVersion,
    pub quest: Option<QuestSaveFile>,
}
