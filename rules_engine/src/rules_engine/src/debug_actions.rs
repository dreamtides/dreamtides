use action_data::debug_action_data::DebugAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_player::battle_player_state::PlayerType;
use core_data::identifiers::{BattleId, UserId};
use core_data::types::PlayerName;
use game_creation::new_battle;
use uuid::Uuid;

pub fn execute(
    battle: &mut BattleState,
    initiated_by: UserId,
    user_player: PlayerName,
    action: DebugAction,
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
    }
}
