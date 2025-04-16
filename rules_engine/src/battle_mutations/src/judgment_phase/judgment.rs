use battle_data::battle::battle_data::BattleData;
use battle_data::battle_animations::battle_animation::BattleAnimation;
use core_data::effect_source::EffectSource;
use core_data::numerics::Points;
use core_data::types::PlayerName;

use crate::player_mutations::points;

/// Runs a Judgment phase for the indicated player, comparing their total spark
/// to their opponent's and assigning points.
pub fn run(battle: &mut BattleData, player: PlayerName, source: EffectSource) {
    let spark = battle_queries::player_queries::spark_total::query(battle, player, source);
    let opponent_spark =
        battle_queries::player_queries::spark_total::query(battle, player.opponent(), source);

    if spark > opponent_spark {
        let points = Points((spark - opponent_spark).0);
        battle.push_animation(|| BattleAnimation::Judgment { player, new_score: Some(points) });
        points::gain(battle, player, source, points);
    } else {
        battle.push_animation(|| BattleAnimation::Judgment { player, new_score: None });
    }
}
