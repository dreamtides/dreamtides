use core_data::numerics::Energy;
use core_data::types::PlayerName;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::battle::card_id::{CardId, CharacterId, HandCardId, StackCardId};

/// An action that can be performed in a battle
#[derive(
    Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum BattleAction {
    /// Developer action
    Debug(DebugBattleAction),
    /// Play a card in the user's hand.
    PlayCardFromHand(HandCardId),
    /// Pass on taking actions in response to a card being played by the
    /// opponent, thus causing the stack to be resolved.
    PassPriority,
    /// End your turn
    EndTurn,
    /// Start your next turn after the opponent takes the `EndTurn` action.
    StartNextTurn,
    /// Select a character as a target
    SelectCharacterTarget(CharacterId),
    /// Select a card on the stack as a target
    SelectStackCardTarget(StackCardId),
    /// Select a choice at a given index position in response to a prompt.
    SelectPromptChoice(usize),
    /// Pick an amount of energy to pay as an additional cost to play a card.
    SelectEnergyAdditionalCost(Energy),
    /// Sets the selected amount of energy to pay as an additional cost to play
    /// a card.
    SetSelectedEnergyAdditionalCost(Energy),
    /// Sets the position of a card in a card order selector.
    SelectCardOrder(SelectCardOrder),
    /// Show cards in a zone
    BrowseCards(CardBrowserType),
    /// Close the card browser
    CloseCardBrowser,
    /// Toggle the visibility of the card order selector
    ToggleOrderSelectorVisibility,
    /// Confirm the selected cards to mulligan
    SubmitMulligan,
}

#[derive(
    Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum DebugBattleAction {
    /// Draw a card
    DrawCard(PlayerName),
    /// Set the energy of the player
    SetEnergy(PlayerName, Energy),
}

#[derive(
    Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, JsonSchema,
)]
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

#[derive(
    Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum CardBrowserType {
    UserDeck,
    EnemyDeck,
    UserVoid,
    EnemyVoid,
    UserStatus,
    EnemyStatus,
}
