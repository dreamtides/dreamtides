use battle_queries::battle_player_queries::player_properties;
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::core::effect_source::EffectSource;
use core_data::numerics::Points;
use core_data::types::PlayerName;

use crate::player_mutations::points;

/// Runs a Judgment phase for the indicated player, comparing their total spark
/// to their opponent's and assigning points.
pub fn run(battle: &mut BattleState, player: PlayerName, source: EffectSource) {
    battle.phase = BattleTurnPhase::Judgment;
    let spark = player_properties::spark_total(battle, player);
    let opponent_spark = player_properties::spark_total(battle, player.opponent());

    if spark > opponent_spark {
        let points = Points((spark - opponent_spark).0);
        battle.push_animation(|| BattleAnimation::Judgment { player, new_score: Some(points) });
        points::gain(battle, player, source, points);
    } else {
        battle.push_animation(|| BattleAnimation::Judgment { player, new_score: None });
    }
}
