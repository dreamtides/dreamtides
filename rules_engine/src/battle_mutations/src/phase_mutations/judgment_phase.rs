use battle_queries::battle_trace;
use battle_state::battle::battle_state::BattleState;
use battle_state::core::effect_source::EffectSource;
use battle_state::core::should_animate::ShouldAnimate;
use core_data::numerics::Points;
use core_data::types::PlayerName;

use crate::character_mutations::dissolve;
use crate::player_mutations::points;

/// Resolves one column of front-rank combat for the active player during the
/// Judgment phase.
///
/// Returns true if all 8 positions have been processed.
pub fn run(battle: &mut BattleState, player: PlayerName, source: EffectSource) -> bool {
    let position = battle.turn.judgment_position;
    battle_trace!("Judgment phase resolving position", battle, position, player);

    let opponent = player.opponent();
    let attacker_id = battle.cards.battlefield(player).front[position as usize];
    let defender_id = battle.cards.battlefield(opponent).front[position as usize];

    match (attacker_id, defender_id) {
        (Some(attacker), Some(defender)) => {
            let attacker_spark = battle.cards.spark(player, attacker).unwrap_or_default();
            let defender_spark = battle.cards.spark(opponent, defender).unwrap_or_default();
            if attacker_spark > defender_spark {
                dissolve::execute(battle, source, defender);
            } else if defender_spark > attacker_spark {
                dissolve::execute(battle, source, attacker);
            } else {
                dissolve::execute(battle, source, defender);
                dissolve::execute(battle, source, attacker);
            }
        }
        (Some(attacker), None) => {
            let spark = battle.cards.spark(player, attacker).unwrap_or_default();
            points::gain(battle, player, source, Points(spark.0), ShouldAnimate::Yes);
        }
        _ => {}
    }

    if position >= 7 {
        true
    } else {
        battle.turn.judgment_position = position + 1;
        false
    }
}
