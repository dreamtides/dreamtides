use ability_data::cost::Cost;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;

use crate::player_mutations::energy;

/// Causes the controller of this [EffectSource] to pay the indicated [Cost].
pub fn apply(battle: &mut BattleData, source: EffectSource, cost: Cost) {
    match cost {
        Cost::Energy(energy) => {
            energy::spend(battle, source.controller(), source, energy);
        }
        _ => todo!("Implement {:?}", cost),
    }
}
