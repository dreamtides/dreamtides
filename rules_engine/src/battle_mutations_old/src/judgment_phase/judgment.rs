use battle_data_old::battle::battle_turn_step::BattleTurnStep;
use battle_data_old::battle::effect_source::EffectSource;
use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle_animations::battle_animation::BattleAnimation;
use battle_queries_old::player_queries::spark_total;
use core_data::numerics::Points;
use core_data::types::PlayerName;

use crate::player_mutations::points;

/// Runs a Judgment phase for the indicated player, comparing their total spark
/// to their opponent's and assigning points.
pub fn run(battle: &mut BattleData, player: PlayerName, source: EffectSource) {
    battle.step = BattleTurnStep::Judgment;
    let spark = spark_total::query(battle, player);
    let opponent_spark = spark_total::query(battle, player.opponent());

    if spark > opponent_spark {
        let points = Points((spark - opponent_spark).0);
        battle.push_animation(|| BattleAnimation::Judgment { player, new_score: Some(points) });
        points::gain(battle, player, source, points);
    } else {
        battle.push_animation(|| BattleAnimation::Judgment { player, new_score: None });
    }
}
