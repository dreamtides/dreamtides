use serde::{Deserialize, Serialize};

use crate::battle_action::BattleAction;

/// All possible user interface actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UserAction {
    BattleAction(BattleAction),
}
