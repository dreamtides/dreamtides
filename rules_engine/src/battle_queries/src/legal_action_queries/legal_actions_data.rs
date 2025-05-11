use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::card_id::CardIdType;
use bit_set::BitSet;
use core_data::numerics::Energy;

#[derive(Debug, Clone)]
pub enum LegalActions {
    NoActionsGameOver,
    NoActionsOpponentPrompt,
    NoActionsOpponentPriority,
    NoActionsInCurrentPhase,
    Standard { actions: StandardLegalActions },
    SelectCharacterPrompt { valid: BitSet<usize> },
    SelectStackCardPrompt { valid: BitSet<usize> },
    SelectPromptChoicePrompt { choice_count: usize },
    SelectEnergyValuePrompt { minimum: Energy, maximum: Energy },
}

#[derive(Debug, Clone)]
pub struct StandardLegalActions {
    pub primary: PrimaryLegalAction,
    pub play_card_from_hand: BitSet<usize>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PrimaryLegalAction {
    PassPriority,
    EndTurn,
    StartNextTurn,
}

impl LegalActions {
    pub fn is_legal(&self, action: BattleAction) -> bool {
        match action {
            BattleAction::Debug(..) => true,
            BattleAction::PlayCardFromHand(hand_card_id) => {
                if let LegalActions::Standard { actions } = self {
                    actions.play_card_from_hand.contains(hand_card_id.card_id().0)
                } else {
                    false
                }
            }
            BattleAction::PassPriority => {
                if let LegalActions::Standard { actions } = self {
                    actions.primary == PrimaryLegalAction::PassPriority
                } else {
                    false
                }
            }
            BattleAction::EndTurn => {
                if let LegalActions::Standard { actions } = self {
                    actions.primary == PrimaryLegalAction::EndTurn
                } else {
                    false
                }
            }
            BattleAction::StartNextTurn => {
                if let LegalActions::Standard { actions } = self {
                    actions.primary == PrimaryLegalAction::StartNextTurn
                } else {
                    false
                }
            }
            BattleAction::SelectCharacterTarget(character_id) => {
                if let LegalActions::SelectCharacterPrompt { valid } = self {
                    valid.contains(character_id.card_id().0)
                } else {
                    false
                }
            }
            BattleAction::SelectStackCardTarget(stack_card_id) => {
                if let LegalActions::SelectStackCardPrompt { valid } = self {
                    valid.contains(stack_card_id.card_id().0)
                } else {
                    false
                }
            }
            BattleAction::SelectPromptChoice(index) => {
                if let LegalActions::SelectPromptChoicePrompt { choice_count } = self {
                    index < *choice_count
                } else {
                    false
                }
            }
            BattleAction::SelectEnergyAdditionalCost(energy)
            | BattleAction::SetSelectedEnergyAdditionalCost(energy) => {
                if let LegalActions::SelectEnergyValuePrompt { minimum, maximum } = self {
                    energy >= *minimum && energy <= *maximum
                } else {
                    false
                }
            }
            BattleAction::SelectCardOrder(..) => false,
            BattleAction::BrowseCards(..) => todo!("Implement this"),
            BattleAction::CloseCardBrowser => true,
            BattleAction::ToggleOrderSelectorVisibility => true,
            BattleAction::SubmitMulligan => todo!("Implement this"),
        }
    }
}
