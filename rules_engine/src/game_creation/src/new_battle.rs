use ai_data::game_ai::GameAI;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_animations::animation_data::AnimationData;
use core_data::identifiers::BattleId;

use crate::new_test_battle;

/// Creates a new battle and starts it.
pub fn create_and_start(id: BattleId) -> BattleData {
    let mut battle =
        new_test_battle::create_and_start(id, None, Some(GameAI::FirstAvailableAction));
    battle.animations = Some(AnimationData::default());
    battle
}
