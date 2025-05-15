use ai_core::state_evaluator::StateEvaluator;
use battle_state::battle::battle_status::BattleStatus;
use core_data::types::PlayerName;

use crate::game_state_node_integration::AgentBattleState;

pub struct WinLossEvaluator;

impl StateEvaluator<AgentBattleState> for WinLossEvaluator {
    fn evaluate(&self, agent_state: &AgentBattleState, player: PlayerName) -> i32 {
        match agent_state.state.status {
            BattleStatus::GameOver { winner } if winner == Some(player) => 1,
            BattleStatus::GameOver { winner } if winner != Some(player) => -1,
            _ => 0,
        }
    }
}
