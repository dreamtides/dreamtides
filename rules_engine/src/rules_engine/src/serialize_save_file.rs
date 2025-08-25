use battle_state::battle::battle_state::BattleState;
use core_data::identifiers::{QuestId, UserId};
use database::quest_save_file::QuestSaveFile;
use database::save_file::{SaveFile, SaveFileV1};

/// Serializes a [BattleState] to a [SaveFile] for a given [UserId] and
/// [QuestId].
pub fn battle(user_id: UserId, quest_id: QuestId, battle: &BattleState) -> SaveFile {
    SaveFile::V1(Box::new(SaveFileV1 {
        id: user_id,
        quest: Some(QuestSaveFile { id: quest_id, battle: Some(battle.clone()) }),
    }))
}
