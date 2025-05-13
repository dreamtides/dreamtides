use action_data_old::debug_action_data::DebugAction;
use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle_player::player_data::PlayerType;
use core_data::identifiers::{BattleId, UserId};
use core_data::types::PlayerName;
use game_creation_old::new_battle;
use uuid::Uuid;

pub fn execute(
    battle: &mut BattleData,
    initiated_by: UserId,
    user_player: PlayerName,
    action: DebugAction,
) {
    match action {
        DebugAction::ApplyTestScenarioAction => {}
        DebugAction::RestartBattle => {
            *battle = new_battle::create_and_start(initiated_by, BattleId(Uuid::new_v4()));
        }
        DebugAction::SetOpponentAgent(ai) => {
            battle.player_mut(user_player.opponent()).player_type = PlayerType::Agent(ai);
        }
    }
}
