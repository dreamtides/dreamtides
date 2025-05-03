use actions::battle_actions;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_animations::animation_data::AnimationData;
use core_data::identifiers::QuestId;
use database::save_file::SaveFile;
use game_creation::new_battle;
use tracing::{info, subscriber};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

/// Returns a deserialized [BattleData] for the battle in this save
/// file, if  one is present.
pub fn battle(file: &SaveFile) -> Option<(BattleData, QuestId)> {
    match file {
        SaveFile::V1(v1) => {
            let quest_id = v1.quest.as_ref()?.id;
            let file = v1.quest.as_ref()?.battle.as_ref()?;
            let mut battle = new_battle::create_and_start_with_options(
                file.id,
                file.seed,
                file.player_types.one.clone(),
                file.player_types.two.clone(),
            );
            battle.animations = Some(AnimationData::default());

            info!("Replaying battle history to construct state");
            let filter = EnvFilter::new("warn");
            let forest_subscriber =
                tracing_subscriber::registry().with(logging::create_forest_layer(filter));
            subscriber::with_default(forest_subscriber, || {
                for history_action in &file.actions {
                    battle_actions::execute(
                        &mut battle,
                        history_action.player,
                        history_action.action,
                    );
                }
            });

            Some((battle, quest_id))
        }
    }
}
