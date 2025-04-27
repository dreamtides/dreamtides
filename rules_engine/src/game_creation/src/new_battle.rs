use ai_data::game_ai::GameAI;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_animations::animation_data::AnimationData;
use battle_data::battle_player::player_data::PlayerType;
use core_data::identifiers::{BattleId, UserId};

use crate::new_test_battle;

/// Creates a new battle and starts it.
pub fn create_and_start(user_id: UserId, battle_id: BattleId) -> BattleData {
    let mut battle = new_test_battle::create_and_start(
        battle_id,
        PlayerType::User(user_id),
        PlayerType::Agent(GameAI::Uct1MaxIterations(5000)),
    );
    battle.animations = Some(AnimationData::default());
    battle
}
