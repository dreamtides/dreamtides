use actions::battle_actions;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_animations::animation_data::AnimationData;
use core_data::identifiers::QuestId;
use core_data::types::PlayerName;
use database::save_file::SaveFile;
use game_creation::new_battle;
use tracing::{info, subscriber};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

/// Returns a deserialized [BattleData] for the battle in this save
/// file, if  one is present.
pub fn battle(file: &SaveFile) -> Option<(BattleData, QuestId)> {
    get_battle_impl(file, None)
}

/// Returns a deserialized [BattleData] for an 'undo' operation on the battle in
/// this save file, if  one is present.
///
/// We advance the battle state to one which is immediately before the named
/// player's last battle action.
pub fn undo(file: &SaveFile, player: PlayerName) -> Option<(BattleData, QuestId)> {
    get_battle_impl(file, Some(player))
}

fn get_battle_impl(file: &SaveFile, undo: Option<PlayerName>) -> Option<(BattleData, QuestId)> {
    match file {
        SaveFile::V1(v1) => {
            info!("Replaying battle history to construct state");
            let filter = EnvFilter::new("warn");
            let forest_subscriber =
                tracing_subscriber::registry().with(logging::create_forest_layer(filter));

            subscriber::with_default(forest_subscriber, || {
                let quest_id = v1.quest.as_ref()?.id;
                let file = v1.quest.as_ref()?.battle.as_ref()?;
                let mut battle = new_battle::create_and_start_with_options(
                    file.id,
                    file.seed,
                    file.player_types.one.clone(),
                    file.player_types.two.clone(),
                );
                battle.animations = Some(AnimationData::default());

                // Find the last action by the undo player if specified
                let replay_limit = match undo {
                    Some(player) => file
                        .actions
                        .iter()
                        .enumerate()
                        .filter(|(_, action)| action.player == player)
                        .map(|(index, _)| index)
                        .last(),
                    None => None,
                };

                for (index, history_action) in file.actions.iter().enumerate() {
                    // Skip the last action by the undo player
                    if replay_limit == Some(index) {
                        break;
                    }

                    battle_actions::execute(
                        &mut battle,
                        history_action.player,
                        history_action.action,
                    );
                }

                Some((battle, quest_id))
            })
        }
    }
}
