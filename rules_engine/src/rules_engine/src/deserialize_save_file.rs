use battle_mutations::actions::apply_battle_action;
use battle_state::battle::battle_state::BattleState;
use core_data::identifiers::QuestId;
use core_data::types::PlayerName;
use database::save_file::SaveFile;
use game_creation::new_battle;
use tracing::{info, subscriber};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

/// Returns a deserialized [BattleState] for the battle in this save
/// file, if one is present.
pub fn battle(file: &SaveFile) -> Option<(BattleState, QuestId)> {
    get_battle_impl(file, None)
}

/// Returns a deserialized [BattleState] for an 'undo' operation on the battle
/// in this save file, if  one is present.
///
/// We advance the battle state to one which is immediately before the named
/// player's last battle action.
pub fn undo(file: &SaveFile, player: PlayerName) -> Option<(BattleState, QuestId)> {
    get_battle_impl(file, Some(player))
}

fn get_battle_impl(file: &SaveFile, undo: Option<PlayerName>) -> Option<(BattleState, QuestId)> {
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

                    apply_battle_action::execute(
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
