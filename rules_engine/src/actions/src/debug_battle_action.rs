use battle_data_old::actions::battle_action_data::DebugBattleAction;
use battle_data_old::battle::effect_source::EffectSource;
use battle_data_old::battle::old_battle_data::BattleData;
use battle_mutations_old::zone_mutations::deck;
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
