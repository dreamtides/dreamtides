use battle_data::actions::battle_action_data::DebugBattleAction;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_mutations::zone_mutations::deck;
use core_data::types::PlayerName;
use logging::battle_trace;

pub fn execute(battle: &mut BattleData, player: PlayerName, action: DebugBattleAction) {
    battle_trace!("Executing debug action", battle, player, action);
    let source = EffectSource::Game { controller: player };
    match action {
        DebugBattleAction::DrawCard(player_name) => {
            deck::draw_card(battle, source, player_name);
        }
        DebugBattleAction::SetEnergy(player_name, energy) => {
            battle.player_mut(player_name).current_energy = energy;
        }
    }
}
