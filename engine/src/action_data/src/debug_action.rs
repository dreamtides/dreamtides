use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::user_action::UserAction;

/// Private actions for developer use
#[derive(Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum DebugAction {
    DrawCard,
    TriggerUserJudgment,
    TriggerEnemyJudgment,
}

impl From<DebugAction> for UserAction {
    fn from(action: DebugAction) -> Self {
        UserAction::DebugAction(action)
    }
}
