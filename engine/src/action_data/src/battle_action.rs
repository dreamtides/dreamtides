use core_data::identifiers::CardId;
use serde::{Deserialize, Serialize};

use crate::user_action::UserAction;

/// An action that can be performed in a battle
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BattleAction {
    PlayCard(CardId),
}

impl From<BattleAction> for UserAction {
    fn from(action: BattleAction) -> Self {
        UserAction::BattleAction(action)
    }
}
