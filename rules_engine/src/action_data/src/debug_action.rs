use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::game_action::GameAction;

/// Private actions for developer use
#[derive(Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum DebugAction {
    ApplyTestScenarioAction,
    DrawCard,
    TriggerUserJudgment,
    TriggerEnemyJudgment,
    PerformSomeAction,
}

impl From<DebugAction> for GameAction {
    fn from(action: DebugAction) -> Self {
        GameAction::DebugAction(action)
    }
}
