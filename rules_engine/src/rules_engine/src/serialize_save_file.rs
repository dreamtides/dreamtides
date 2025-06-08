use battle_state::battle::battle_state::BattleState;
use battle_state::battle_player::player_map::PlayerMap;
use core_data::identifiers::{QuestId, UserId};
use database::battle_save_file::BattleSaveFile;
use database::quest_save_file::QuestSaveFile;
use database::save_file::{SaveFile, SaveFileV1};
use tracing_macros::panic_with;

/// Serializes a [BattleState] to a [SaveFile] for a given [UserId] and
/// [QuestId].
pub fn battle(user_id: UserId, quest_id: QuestId, battle: &BattleState) -> SaveFile {
    let Some(history) = battle.action_history.as_ref() else {
        panic_with!("Expected battle with history for serialization", battle);
    };
    SaveFile::V1(SaveFileV1 {
        id: user_id,
        quest: Some(QuestSaveFile {
            id: quest_id,
            battle: Some(BattleSaveFile {
                id: battle.id,
                seed: battle.seed,
                player_types: PlayerMap {
                    one: battle.players.one.player_type.clone(),
                    two: battle.players.two.player_type.clone(),
                },
                actions: history.actions.clone(),
            }),
        }),
    })
}
