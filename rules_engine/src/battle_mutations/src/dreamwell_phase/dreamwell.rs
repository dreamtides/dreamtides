use battle_data::battle::battle_data::BattleData;
use core_data::effect_source::EffectSource;
use core_data::numerics::Energy;
use core_data::types::PlayerName;

use crate::player_mutations::energy;

/// Runs a dreamwell activation for the indicated player
pub fn activate(battle: &mut BattleData, player: PlayerName, source: EffectSource) {
    let new_produced_energy = battle.player(player).produced_energy + Energy(1);
    energy::set_produced(battle, player, source, new_produced_energy);
    energy::set(battle, player, source, new_produced_energy);
}
