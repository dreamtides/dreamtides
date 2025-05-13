use assert_with::expect;
use battle_data_old::battle::old_battle_data::BattleData;
use core_data::identifiers::{QuestId, UserId};
use database_old::battle_save_file::{BattleSaveFile, PlayerMap};
use database_old::quest_save_file::QuestSaveFile;
use database_old::save_file::{SaveFile, SaveFileV1};

/// Serializes a [BattleData] to a [SaveFile] for a given [UserId] and
/// [QuestId].
pub fn battle(user_id: UserId, quest_id: QuestId, battle: &BattleData) -> SaveFile {
    let history = expect!(battle.history.as_ref(), battle, || {
        "Expected battle with history for serialization"
    });
    SaveFile::V1(SaveFileV1 {
        id: user_id,
        quest: Some(QuestSaveFile {
            id: quest_id,
            battle: Some(BattleSaveFile {
                id: battle.id,
                seed: battle.seed,
                player_types: PlayerMap {
                    one: battle.player_one.player_type.clone(),
                    two: battle.player_two.player_type.clone(),
                },
                actions: history.actions.clone(),
            }),
        }),
    })
}
