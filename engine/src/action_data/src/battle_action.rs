use core_data::identifiers::CardId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::user_action::UserAction;

/// An action that can be performed in a battle
#[derive(Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum BattleAction {
    PlayCard(CardId),
    /// Select a card by ID in resopnse to some prompt, e.g. as a target of a
    /// card being played.
    SelectCard(CardId),
    /// Sets the position of a card in a card order selector.
    SelectCardOrder(SelectCardOrder),
    /// Show cards in a zone
    BrowseCards(CardBrowserType),
    /// Close the card browser
    CloseCardBrowser,
    /// Toggle the visibility of the card order selector
    ToggleOrderSelectorVisibility,
    /// End the turn
    EndTurn,
    /// Confirm the selected cards to mulligan
    SubmitMulligan,
}

impl From<BattleAction> for UserAction {
    fn from(action: BattleAction) -> Self {
        UserAction::BattleAction(action)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SelectCardOrder {
    pub target: CardOrderSelectionTarget,
    pub card_id: CardId,
    pub position: usize,
}

#[derive(
    Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum CardOrderSelectionTarget {
    Deck,
    Void,
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
