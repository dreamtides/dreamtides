use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::quest_save_file::QuestSaveFile;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FileFormatVersion {
    V1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveFileId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveFile {
    pub id: SaveFileId,
    pub version: FileFormatVersion,
    pub quest: Option<QuestSaveFile>,
}
