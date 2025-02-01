use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::battle_action::BattleAction;
use crate::debug_action::DebugAction;

/// All possible user interface actions
#[derive(Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum UserAction {
    DebugAction(DebugAction),
    BattleAction(BattleAction),
}
