use ai_core::state_evaluator::StateEvaluator;
use battle_data::battle::battle_status::BattleStatus;
use core_data::types::PlayerName;

use crate::state_node::AgentBattleState;

/// Evaluator which returns -1 for a loss, 1 for a win, and 0 otherwise
pub struct WinLossEvaluator;

impl StateEvaluator<AgentBattleState> for WinLossEvaluator {
    fn evaluate(&self, state: &AgentBattleState, player: PlayerName) -> i32 {
        match state.status {
            BattleStatus::GameOver { winner } if winner == player => 1,
            BattleStatus::GameOver { winner } if winner != player => -1,
            _ => 0,
        }
    }
}
