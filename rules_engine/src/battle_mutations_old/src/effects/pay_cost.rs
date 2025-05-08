use ability_data::cost::Cost;
use battle_data_old::battle::effect_source::EffectSource;
use battle_data_old::battle::old_battle_data::BattleData;
use core_data::types::PlayerName;
use logging::battle_trace;

use crate::player_mutations::energy;

/// Causes the [PlayerName] player to pay the indicated [Cost].
pub fn apply(battle: &mut BattleData, source: EffectSource, player: PlayerName, cost: Cost) {
    battle_trace!("Paying cost {:?}", battle, player, cost);
    match cost {
        Cost::Energy(energy) => {
            energy::spend(battle, player, source, energy);
        }
        _ => todo!("Implement {:?}", cost),
    }
}
