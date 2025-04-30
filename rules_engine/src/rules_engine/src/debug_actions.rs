use action_data::debug_action::DebugAction;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_mutations::zone_mutations::deck;
use core_data::identifiers::{BattleId, UserId};
use core_data::types::PlayerName;
use game_creation::new_battle;
use uuid::Uuid;

pub fn execute(
    battle: &mut BattleData,
    initiated_by: UserId,
    player: PlayerName,
    action: DebugAction,
) {
    let source = EffectSource::Game { controller: player };
    match action {
        DebugAction::ApplyTestScenarioAction => {}
        DebugAction::DrawCard => {
            deck::draw_card(battle, source, player);
        }
        DebugAction::RestartBattle => {
            *battle = new_battle::create_and_start(initiated_by, BattleId(Uuid::new_v4()));
        }
    }
}
