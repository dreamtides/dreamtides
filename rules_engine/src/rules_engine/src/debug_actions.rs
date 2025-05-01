use action_data::debug_action_data::DebugAction;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_data::battle_player::player_data::PlayerType;
use battle_mutations::zone_mutations::deck;
use core_data::identifiers::{BattleId, UserId};
use core_data::types::PlayerName;
use game_creation::new_battle;
use uuid::Uuid;

pub fn execute(
    battle: &mut BattleData,
    initiated_by: UserId,
    user_player: PlayerName,
    action: DebugAction,
) {
    let source = EffectSource::Game { controller: user_player };
    match action {
        DebugAction::ApplyTestScenarioAction => {}
        DebugAction::DrawCard => {
            deck::draw_card(battle, source, user_player);
        }
        DebugAction::RestartBattle => {
            *battle = new_battle::create_and_start(initiated_by, BattleId(Uuid::new_v4()));
        }
        DebugAction::SetOpponentAgent(ai) => {
            battle.player_mut(user_player.opponent()).player_type = PlayerType::Agent(ai);
        }
    }
}
