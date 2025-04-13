use battle_data::battle::battle_data::BattleData;
use battle_data::battle_animations::battle_animation::BattleAnimation;
use core_data::numerics::Energy;
use core_data::source::Source;
use core_data::types::PlayerName;

use crate::player_mutations::energy;

/// Runs a dreamwell activation for the indicated player
pub fn activate(battle: &mut BattleData, player: PlayerName, source: Source) {
    let new_produced_energy = battle.player(player).produced_energy + Energy(1);
    battle.push_animation(|| BattleAnimation::DreamwellActivation {
        player,
        dreamwell_card_id: None,
        new_energy: new_produced_energy,
        new_produced_energy,
    });
    energy::set_produced(battle, player, source, new_produced_energy);
    energy::set(battle, player, source, new_produced_energy);
}
