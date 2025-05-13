use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::card_id::{CharacterId, HandCardId, StackCardId};
use battle_state::battle_cards::card_set::CardSet;
use core_data::numerics::Energy;

#[derive(Debug, Clone)]
pub enum LegalActions {
    NoActionsGameOver,
    NoActionsOpponentPrompt,
    NoActionsOpponentPriority,
    NoActionsInCurrentPhase,
    Standard { actions: StandardLegalActions },
    SelectCharacterPrompt { valid: CardSet<CharacterId> },
    SelectStackCardPrompt { valid: CardSet<StackCardId> },
    SelectPromptChoicePrompt { choice_count: usize },
    SelectEnergyValuePrompt { minimum: Energy, maximum: Energy },
}

#[derive(Debug, Clone)]
pub struct StandardLegalActions {
    pub primary: PrimaryLegalAction,
    pub play_card_from_hand: CardSet<HandCardId>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PrimaryLegalAction {
    PassPriority,
    EndTurn,
    StartNextTurn,
}

impl LegalActions {
    pub fn contains(&self, action: BattleAction) -> bool {
        match action {
            BattleAction::Debug(..) => true,
            BattleAction::PlayCardFromHand(hand_card_id) => {
                if let LegalActions::Standard { actions } = self {
                    actions.play_card_from_hand.contains(hand_card_id)
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
                    valid.contains(character_id)
                } else {
                    false
                }
            }
            BattleAction::SelectStackCardTarget(stack_card_id) => {
                if let LegalActions::SelectStackCardPrompt { valid } = self {
                    valid.contains(stack_card_id)
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

    pub fn is_empty(&self) -> bool {
        match self {
            LegalActions::NoActionsGameOver
            | LegalActions::NoActionsOpponentPrompt
            | LegalActions::NoActionsOpponentPriority
            | LegalActions::NoActionsInCurrentPhase => true,
            LegalActions::Standard { .. } => false,
            LegalActions::SelectCharacterPrompt { valid } => valid.is_empty(),
            LegalActions::SelectStackCardPrompt { valid } => valid.is_empty(),
            LegalActions::SelectPromptChoicePrompt { choice_count } => *choice_count == 0,
            LegalActions::SelectEnergyValuePrompt { minimum, maximum } => maximum < minimum,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            LegalActions::NoActionsGameOver
            | LegalActions::NoActionsOpponentPrompt
            | LegalActions::NoActionsOpponentPriority
            | LegalActions::NoActionsInCurrentPhase => 0,

            LegalActions::Standard { actions } => {
                let primary_count = 1;
                let play_cards_count = actions.play_card_from_hand.len();
                primary_count + play_cards_count
            }

            LegalActions::SelectCharacterPrompt { valid } => valid.len(),
            LegalActions::SelectStackCardPrompt { valid } => valid.len(),
            LegalActions::SelectPromptChoicePrompt { choice_count } => *choice_count,
            LegalActions::SelectEnergyValuePrompt { minimum, maximum } => {
                if maximum >= minimum {
                    (maximum.0 - minimum.0 + 1) as usize
                } else {
                    0
                }
            }
        }
    }

    pub fn all(&self) -> Vec<BattleAction> {
        match self {
            LegalActions::NoActionsGameOver
            | LegalActions::NoActionsOpponentPrompt
            | LegalActions::NoActionsOpponentPriority
            | LegalActions::NoActionsInCurrentPhase => vec![],

            LegalActions::Standard { actions } => {
                let mut result = vec![];

                match actions.primary {
                    PrimaryLegalAction::PassPriority => result.push(BattleAction::PassPriority),
                    PrimaryLegalAction::EndTurn => result.push(BattleAction::EndTurn),
                    PrimaryLegalAction::StartNextTurn => result.push(BattleAction::StartNextTurn),
                }

                for card_id in actions.play_card_from_hand.iter() {
                    result.push(BattleAction::PlayCardFromHand(card_id));
                }

                result
            }

            LegalActions::SelectCharacterPrompt { valid } => {
                valid.iter().map(BattleAction::SelectCharacterTarget).collect::<Vec<_>>()
            }

            LegalActions::SelectStackCardPrompt { valid } => {
                valid.iter().map(BattleAction::SelectStackCardTarget).collect::<Vec<_>>()
            }

            LegalActions::SelectPromptChoicePrompt { choice_count } => {
                (0..*choice_count).map(BattleAction::SelectPromptChoice).collect::<Vec<_>>()
            }

            LegalActions::SelectEnergyValuePrompt { minimum, maximum } => (minimum.0..=maximum.0)
                .map(|e| BattleAction::SelectEnergyAdditionalCost(Energy(e)))
                .collect::<Vec<_>>(),
        }
    }
}
