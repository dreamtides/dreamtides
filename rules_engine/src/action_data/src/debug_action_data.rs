use ai_data::game_ai::GameAI;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::game_action_data::GameAction;

/// Private actions for developer use
#[derive(Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum DebugAction {
    ApplyTestScenarioAction,
    DrawCard,
    RestartBattle,
    SetOpponentAgent(GameAI),
}

impl From<DebugAction> for GameAction {
    fn from(action: DebugAction) -> Self {
        GameAction::DebugAction(action)
    }
}
