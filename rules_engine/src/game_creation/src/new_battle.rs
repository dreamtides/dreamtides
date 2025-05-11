use ai_data::game_ai::GameAI;
use battle_state::battle::animation_data::AnimationData;
use battle_state::battle::battle_history::BattleHistory;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_player::battle_player_state::PlayerType;
use battle_state::battle_trace::battle_tracing::BattleTracing;
use core_data::identifiers::{BattleId, UserId};
use rand::RngCore;

use crate::new_test_battle;

/// Creates a new battle and starts it.
pub fn create_and_start(user_id: UserId, battle_id: BattleId) -> BattleState {
    create_and_start_with_options(
        battle_id,
        rand::rng().next_u64(),
        PlayerType::User(user_id),
        PlayerType::Agent(GameAI::Uct1MaxIterations(1_000)),
    )
}

/// Creates a new battle and starts it using a given seed and [PlayerType]
/// specification.
pub fn create_and_start_with_options(
    battle_id: BattleId,
    seed: u64,
    player_one: PlayerType,
    player_two: PlayerType,
) -> BattleState {
    let mut battle = new_test_battle::create_and_start(battle_id, seed, player_one, player_two);
    battle.animations = Some(AnimationData::default());
    battle.tracing = Some(BattleTracing::default());
    battle.history = Some(BattleHistory::default());
    battle
}
