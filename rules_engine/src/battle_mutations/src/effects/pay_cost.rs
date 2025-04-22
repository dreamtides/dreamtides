use ability_data::cost::Cost;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use core_data::types::PlayerName;

use crate::player_mutations::energy;

/// Causes the [PlayerName] player to pay the indicated [Cost].
pub fn apply(battle: &mut BattleData, source: EffectSource, player: PlayerName, cost: Cost) {
    match cost {
        Cost::Energy(energy) => {
            energy::spend(battle, player, source, energy);
        }
        _ => todo!("Implement {:?}", cost),
    }
}
