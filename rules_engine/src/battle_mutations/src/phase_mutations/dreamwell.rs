use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::core::effect_source::EffectSource;
use core_data::numerics::{Energy, TurnId};
use core_data::types::PlayerName;

use crate::player_mutations::energy;

/// Runs a dreamwell activation for the indicated player
pub fn activate(battle: &mut BattleState, player: PlayerName, source: EffectSource) {
    battle.phase = BattleTurnPhase::Dreamwell;
    let new_produced_energy = battle.players.player(player).produced_energy
        + if battle.turn.turn_id == TurnId(1) || battle.turn.turn_id == TurnId(2) {
            Energy(2)
        } else {
            Energy(1)
        };
    energy::set_produced(battle, player, source, new_produced_energy);
    energy::set(battle, player, source, new_produced_energy);
}
