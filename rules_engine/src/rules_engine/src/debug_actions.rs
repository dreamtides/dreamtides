use action_data::debug_action_data::DebugAction;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::{BattleState, RequestContext};
use battle_state::battle_player::battle_player_state::PlayerType;
use core_data::identifiers::{BattleId, UserId};
use core_data::types::PlayerName;
use game_creation::new_battle;
use state_provider::state_provider::StateProvider;
use uuid::Uuid;

use crate::handle_battle_action;

pub fn execute(
    provider: impl StateProvider + 'static,
    battle: &mut BattleState,
    initiated_by: UserId,
    user_player: PlayerName,
    action: DebugAction,
    context: &RequestContext,
    request_id: Option<Uuid>,
) {
    match action {
        DebugAction::ApplyTestScenarioAction => {}
        DebugAction::RestartBattle => {
            *battle = new_battle::create_and_start(
                initiated_by,
                BattleId(Uuid::new_v4()),
                battle.request_context.clone(),
            );
        }
        DebugAction::SetOpponentAgent(ai) => {
            battle.players.player_mut(user_player.opponent()).player_type = PlayerType::Agent(ai);
        }
        DebugAction::ApplyActionList(actions) => {
            for debug_action in actions {
                handle_battle_action::execute(
                    provider.clone(),
                    battle,
                    initiated_by,
                    user_player,
                    BattleAction::Debug(debug_action),
                    context,
                    request_id,
                );
            }
        }
    }
}
