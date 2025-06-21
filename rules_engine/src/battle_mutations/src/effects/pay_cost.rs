use ability_data::cost::Cost;
use battle_state::battle::battle_state::BattleState;
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;
use battle_queries::battle_trace;

use crate::player_mutations::energy;

/// Causes the [PlayerName] player to pay the indicated [Cost].
pub fn execute(battle: &mut BattleState, source: EffectSource, player: PlayerName, cost: &Cost) {
    battle_trace!("Paying cost {:?}", battle, player, cost);
    match cost {
        Cost::Energy(energy) => {
            energy::spend(battle, player, source, *energy);
        }
        _ => todo!("Implement {:?}", cost),
    }
}
