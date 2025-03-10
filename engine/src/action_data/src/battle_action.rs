use core_data::identifiers::CardId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::user_action::UserAction;

/// An action that can be performed in a battle
#[derive(Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum BattleAction {
    PlayCard(CardId),
    /// Set a card as a target of the card currently being played.
    SelectTarget(CardId),
    /// Show cards in a zone
    BrowseCards(CardBrowserType),
}

impl From<BattleAction> for UserAction {
    fn from(action: BattleAction) -> Self {
        UserAction::BattleAction(action)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CardBrowserType {
    UserDeck,
    EnemyDeck,
    UserVoid,
    EnemyVoid,
    UserStatus,
    EnemyStatus,
}
