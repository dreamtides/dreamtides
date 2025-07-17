use core_data::numerics::Energy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

use crate::actions::debug_battle_action::DebugBattleAction;
use crate::battle::card_id::{
    AbilityId, ActivatedAbilityId, CharacterId, DeckCardId, HandCardId, StackCardId, VoidCardId,
};

/// An action that can be performed in a battle
#[derive(
    Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum BattleAction {
    /// Developer action
    Debug(DebugBattleAction),
    /// Play a card in the user's hand using the standard play action.
    PlayCardFromHand(HandCardId),
    /// Play a card in the user's void using the given ability.
    PlayCardFromVoid(VoidCardId, AbilityId),
    /// Activate a character's ability.
    ActivateAbility(ActivatedAbilityId),
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
    /// Select a card in a void as a target
    SelectVoidCardTarget(VoidCardId),
    /// Select a choice at a given index position in response to a prompt.
    SelectPromptChoice(usize),
    /// Pick an amount of energy to pay as an additional cost to play a card.
    SelectEnergyAdditionalCost(Energy),
    /// Sets the position of a card in a card order selector.
    SelectOrderForDeckCard(DeckCardSelectedOrder),
    /// Submit the selected deck card order configuration in the current
    /// ordering prompt.
    SubmitDeckCardOrder,
    /// Toggle the visibility of the card order selector
    ToggleOrderSelectorVisibility,
    /// Confirm the selected cards to mulligan
    SubmitMulligan,
}

#[derive(
    Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct DeckCardSelectedOrder {
    pub card_id: DeckCardId,
    pub target: CardOrderSelectionTarget,
}

#[derive(
    Debug,
    Copy,
    Clone,
    Serialize,
    Eq,
    PartialEq,
    Hash,
    PartialOrd,
    Ord,
    Deserialize,
    JsonSchema,
    EnumDiscriminants,
)]
#[serde(rename_all = "camelCase")]
#[strum_discriminants(derive(Hash, Serialize, Deserialize, Ord, PartialOrd, JsonSchema))]
pub enum CardOrderSelectionTarget {
    Deck(usize),
    Void,
}

impl BattleAction {
    /// Format a battle action as a short name for display.
    ///
    /// For example, "Play Card From Hand(HandCardId(20))" becomes "PCFH20"
    pub fn battle_action_string(&self) -> String {
        match self {
            BattleAction::Debug(..) => "DEBUG".to_string(),
            BattleAction::PlayCardFromHand(hand_card_id) => format!("PCFH{:?}", hand_card_id.0.0),
            BattleAction::PlayCardFromVoid(void_card_id, ability_id) => {
                format!("PCFV{:?}_{:?}", void_card_id.0.0, ability_id.ability_number.0)
            }
            BattleAction::ActivateAbility(activated_ability_id) => {
                format!(
                    "AA{:?}_{:?}",
                    activated_ability_id.character_id.0.0, activated_ability_id.ability_number.0
                )
            }
            BattleAction::PassPriority => "PP".to_string(),
            BattleAction::EndTurn => "ET".to_string(),
            BattleAction::StartNextTurn => "SNT".to_string(),
            BattleAction::SelectCharacterTarget(character_id) => {
                format!("SCT{:?}", character_id.0.0)
            }
            BattleAction::SelectStackCardTarget(stack_card_id) => {
                format!("SSCT{:?}", stack_card_id.0.0)
            }
            BattleAction::SelectVoidCardTarget(void_card_id) => {
                format!("SVC{:?}", void_card_id.0.0)
            }
            BattleAction::SelectPromptChoice(index) => format!("SPC{index:?}"),
            BattleAction::SelectEnergyAdditionalCost(energy) => format!("SEAC{}", energy.0),
            BattleAction::SelectOrderForDeckCard(order) => {
                let target = match order.target {
                    CardOrderSelectionTarget::Deck(position) => format!("D{position}"),
                    CardOrderSelectionTarget::Void => "V".to_string(),
                };
                format!("SCO{}{:?}", target, order.card_id.0.0)
            }
            BattleAction::SubmitDeckCardOrder => "SDCO".to_string(),
            BattleAction::ToggleOrderSelectorVisibility => "TOSV".to_string(),
            BattleAction::SubmitMulligan => "SM".to_string(),
        }
    }
}
