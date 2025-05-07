use ai_data::game_ai::GameAI;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_animations::animation_data::AnimationData;
use battle_data::battle_player::player_data::PlayerType;
use core_data::identifiers::{BattleId, UserId};
use rand::RngCore;

use crate::new_test_battle;

/// Creates a new battle and starts it.
pub fn create_and_start(user_id: UserId, battle_id: BattleId) -> BattleData {
    create_and_start_with_options(
        battle_id,
        rand::rng().next_u64(),
        PlayerType::User(user_id),
        PlayerType::Agent(GameAI::Uct1MaxIterations(50_000)),
    )
}

/// Creates a new battle and starts it using a given seed and [PlayerType]
/// specification.
pub fn create_and_start_with_options(
    battle_id: BattleId,
    seed: u64,
    player_one: PlayerType,
    player_two: PlayerType,
) -> BattleData {
    let mut battle = new_test_battle::create_and_start(battle_id, seed, player_one, player_two);
    battle.animations = Some(AnimationData::default());
    battle
}
