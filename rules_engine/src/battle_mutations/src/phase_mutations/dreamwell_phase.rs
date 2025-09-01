use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;

use crate::card_mutations::dreamwell;
use crate::effects::apply_effect;
use crate::player_mutations::energy;

/// Runs a dreamwell activation for the indicated player
pub fn activate(battle: &mut BattleState, player: PlayerName, source: EffectSource) {
    battle.phase = BattleTurnPhase::Dreamwell;
    let card = dreamwell::draw(battle);
    let new_produced_energy = battle.players.player(player).produced_energy + card.produced_energy;
    energy::set_produced(battle, player, source, new_produced_energy);
    energy::set(battle, player, source, new_produced_energy);
    apply_effect::execute_event_abilities(
        battle,
        |_| EffectSource::Dreamwell { controller: player },
        &card.effects,
        None,
        None,
    );
}
