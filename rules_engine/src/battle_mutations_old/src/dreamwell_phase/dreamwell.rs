use battle_data_old::battle::battle_turn_step::BattleTurnStep;
use battle_data_old::battle::effect_source::EffectSource;
use battle_data_old::battle::old_battle_data::BattleData;
use core_data::numerics::{Energy, TurnId};
use core_data::types::PlayerName;

use crate::player_mutations::energy;

/// Runs a dreamwell activation for the indicated player
pub fn activate(battle: &mut BattleData, player: PlayerName, source: EffectSource) {
    battle.step = BattleTurnStep::Dreamwell;
    let new_produced_energy = battle.player(player).produced_energy
        + if battle.turn.turn_id == TurnId(1) || battle.turn.turn_id == TurnId(2) {
            Energy(2)
        } else {
            Energy(1)
        };
    energy::set_produced(battle, player, source, new_produced_energy);
    energy::set(battle, player, source, new_produced_energy);
}
