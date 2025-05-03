use actions::battle_actions;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_animations::animation_data::AnimationData;
use database::save_file::SaveFile;
use game_creation::new_battle;

/// Returns a deserialized [BattleData] for the battle in this save
/// file, if  one is present.
pub fn battle(file: &SaveFile) -> Option<BattleData> {
    match file {
        SaveFile::V1(v1) => {
            let file = v1.quest.as_ref()?.battle.as_ref()?;
            let mut battle = new_battle::create_and_start_with_options(
                file.id,
                file.seed,
                file.player_types.one.clone(),
                file.player_types.two.clone(),
            );
            battle.animations = Some(AnimationData::default());

            for history_action in &file.actions {
                battle_actions::execute(&mut battle, history_action.player, history_action.action);
            }

            Some(battle)
        }
    }
}
