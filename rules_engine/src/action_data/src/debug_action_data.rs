use ai_data::game_ai::GameAI;
use battle_state::actions::debug_battle_action::DebugBattleAction;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::game_action_data::GameAction;

/// Private actions for developer use
#[derive(Debug, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum DebugAction {
    ApplyTestScenarioAction,
    RestartBattle,
    SetOpponentAgent(GameAI),
    ApplyActionList(Vec<DebugBattleAction>),
}

impl From<DebugAction> for GameAction {
    fn from(action: DebugAction) -> Self {
        GameAction::DebugAction(action)
    }
}
