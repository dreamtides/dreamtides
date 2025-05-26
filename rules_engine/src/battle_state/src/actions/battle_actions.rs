use core_data::identifiers::CardName;
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
    /// Sets the position of a card in a card order selector.
    SelectCardOrder(SelectCardOrder),
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
    /// Add a specific card to hand
    AddCardToHand(PlayerName, CardName),
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

impl BattleAction {
    /// Format a battle action as a short name for display.
    ///
    /// For example, "Play Card From Hand(HandCardId(20))" becomes "PCFH20"
    pub fn battle_action_string(&self) -> String {
        match self {
            BattleAction::Debug(..) => "DEBUG".to_string(),
            BattleAction::PlayCardFromHand(hand_card_id) => format!("PCFH{:?}", hand_card_id.0 .0),
            BattleAction::PassPriority => "PP".to_string(),
            BattleAction::EndTurn => "ET".to_string(),
            BattleAction::StartNextTurn => "SNT".to_string(),
            BattleAction::SelectCharacterTarget(character_id) => {
                format!("SCT{:?}", character_id.0 .0)
            }
            BattleAction::SelectStackCardTarget(stack_card_id) => {
                format!("SSCT{:?}", stack_card_id.0 .0)
            }
            BattleAction::SelectPromptChoice(index) => format!("SPC{:?}", index),
            BattleAction::SelectEnergyAdditionalCost(energy) => format!("SEAC{}", energy.0),
            BattleAction::SelectCardOrder(order) => {
                let target = match order.target {
                    CardOrderSelectionTarget::Deck => "D",
                    CardOrderSelectionTarget::Void => "V",
                };
                format!("SCO{}{}{}", target, order.card_id.0, order.position)
            }
            BattleAction::ToggleOrderSelectorVisibility => "TOSV".to_string(),
            BattleAction::SubmitMulligan => "SM".to_string(),
        }
    }
}
