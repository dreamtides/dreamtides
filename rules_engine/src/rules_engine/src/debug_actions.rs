use action_data::debug_action::DebugAction;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_mutations::zone_mutations::deck;
use core_data::types::PlayerName;

pub fn execute(battle: &mut BattleData, player: PlayerName, action: DebugAction) {
    let source = EffectSource::Game { controller: player };
    match action {
        DebugAction::ApplyTestScenarioAction => {}
        DebugAction::DrawCard => {
            deck::draw_card(battle, source, player);
        }
    }
}
