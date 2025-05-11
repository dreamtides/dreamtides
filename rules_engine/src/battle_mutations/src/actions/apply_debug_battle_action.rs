use battle_state::actions::battle_actions::DebugBattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;
use tracing_macros::battle_trace;

use crate::card_mutations::deck;

pub fn execute(battle: &mut BattleState, player: PlayerName, action: DebugBattleAction) {
    battle_trace!("Executing debug action", battle, player, action);
    let source = EffectSource::Game { controller: player };
    match action {
        DebugBattleAction::DrawCard(player_name) => {
            deck::draw_card(battle, source, player_name);
        }
        DebugBattleAction::SetEnergy(player_name, energy) => {
            battle.players.player_mut(player_name).current_energy = energy;
        }
    }
}
