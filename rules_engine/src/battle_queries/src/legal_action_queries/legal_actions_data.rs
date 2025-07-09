use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::card_id::{CharacterId, HandCardId, StackCardId};
use battle_state::battle_cards::card_set::CardSet;
use core_data::identifiers::AbilityNumber;
use core_data::numerics::Energy;
use fastrand;

#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StandardLegalActions {
    pub primary: PrimaryLegalAction,
    pub play_card_from_hand: Vec<HandCardId>,
    pub activate_abilities: Vec<ActivatableAbility>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ActivatableAbility {
    pub character_id: CharacterId,
    pub ability_number: AbilityNumber,
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
                    actions.play_card_from_hand.contains(&hand_card_id)
                } else {
                    false
                }
            }
            BattleAction::ActivateAbility { character_id, ability_number } => {
                if let LegalActions::Standard { actions } = self {
                    actions.activate_abilities.iter().any(|ability| {
                        ability.character_id == character_id
                            && ability.ability_number == ability_number
                    })
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
            BattleAction::SelectEnergyAdditionalCost(energy) => {
                if let LegalActions::SelectEnergyValuePrompt { minimum, maximum } = self {
                    energy >= *minimum && energy <= *maximum
                } else {
                    false
                }
            }
            BattleAction::SelectCardOrder(..) => false,
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
                let ability_count = actions.activate_abilities.len();
                primary_count + play_cards_count + ability_count
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

    /// Returns a legal [BattleAction] from this action set which is *not*
    /// present in `actions`, if any.
    pub fn find_missing(&self, actions: &[BattleAction]) -> Option<BattleAction> {
        match self {
            LegalActions::NoActionsGameOver
            | LegalActions::NoActionsOpponentPrompt
            | LegalActions::NoActionsOpponentPriority
            | LegalActions::NoActionsInCurrentPhase => None,

            LegalActions::Standard { actions: standard_actions } => {
                match standard_actions.primary {
                    PrimaryLegalAction::PassPriority
                        if !actions.contains(&BattleAction::PassPriority) =>
                    {
                        Some(BattleAction::PassPriority)
                    }
                    PrimaryLegalAction::EndTurn if !actions.contains(&BattleAction::EndTurn) => {
                        Some(BattleAction::EndTurn)
                    }
                    PrimaryLegalAction::StartNextTurn
                        if !actions.contains(&BattleAction::StartNextTurn) =>
                    {
                        Some(BattleAction::StartNextTurn)
                    }
                    _ => {
                        if let Some(card_id) =
                            standard_actions.play_card_from_hand.iter().find(|&&card_id| {
                                !actions.contains(&BattleAction::PlayCardFromHand(card_id))
                            })
                        {
                            Some(BattleAction::PlayCardFromHand(*card_id))
                        } else {
                            standard_actions
                                .activate_abilities
                                .iter()
                                .find(|ability| {
                                    !actions.contains(&BattleAction::ActivateAbility {
                                        character_id: ability.character_id,
                                        ability_number: ability.ability_number,
                                    })
                                })
                                .map(|ability| BattleAction::ActivateAbility {
                                    character_id: ability.character_id,
                                    ability_number: ability.ability_number,
                                })
                        }
                    }
                }
            }

            LegalActions::SelectCharacterPrompt { valid } => valid
                .iter()
                .find(|id| !actions.contains(&BattleAction::SelectCharacterTarget(*id)))
                .map(BattleAction::SelectCharacterTarget),

            LegalActions::SelectStackCardPrompt { valid } => valid
                .iter()
                .find(|id| !actions.contains(&BattleAction::SelectStackCardTarget(*id)))
                .map(BattleAction::SelectStackCardTarget),

            LegalActions::SelectPromptChoicePrompt { choice_count } => (0..*choice_count)
                .find(|&i| !actions.contains(&BattleAction::SelectPromptChoice(i)))
                .map(BattleAction::SelectPromptChoice),

            LegalActions::SelectEnergyValuePrompt { minimum, maximum } => (minimum.0..=maximum.0)
                .find(|&e| !actions.contains(&BattleAction::SelectEnergyAdditionalCost(Energy(e))))
                .map(|e| BattleAction::SelectEnergyAdditionalCost(Energy(e))),
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
                    result.push(BattleAction::PlayCardFromHand(*card_id));
                }

                for ability in actions.activate_abilities.iter() {
                    result.push(BattleAction::ActivateAbility {
                        character_id: ability.character_id,
                        ability_number: ability.ability_number,
                    });
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

    /// Returns a random action from the legal actions.
    ///
    /// Returns `None` if there are no legal actions.
    pub fn random_action(&self) -> Option<BattleAction> {
        match self {
            LegalActions::NoActionsGameOver
            | LegalActions::NoActionsOpponentPrompt
            | LegalActions::NoActionsOpponentPriority
            | LegalActions::NoActionsInCurrentPhase => None,

            LegalActions::Standard { actions } => {
                let total_actions = self.len();
                if total_actions == 0 {
                    return None;
                }

                let index = fastrand::usize(..total_actions);

                if index == 0 {
                    Some(match actions.primary {
                        PrimaryLegalAction::PassPriority => BattleAction::PassPriority,
                        PrimaryLegalAction::EndTurn => BattleAction::EndTurn,
                        PrimaryLegalAction::StartNextTurn => BattleAction::StartNextTurn,
                    })
                } else {
                    let remaining_index = index - 1;
                    if remaining_index < actions.play_card_from_hand.len() {
                        actions
                            .play_card_from_hand
                            .get(remaining_index)
                            .map(|card_id| BattleAction::PlayCardFromHand(*card_id))
                    } else {
                        let ability_index = remaining_index - actions.play_card_from_hand.len();
                        actions.activate_abilities.get(ability_index).map(|ability| {
                            BattleAction::ActivateAbility {
                                character_id: ability.character_id,
                                ability_number: ability.ability_number,
                            }
                        })
                    }
                }
            }

            LegalActions::SelectCharacterPrompt { valid } => {
                if valid.is_empty() {
                    None
                } else {
                    let index = fastrand::usize(..valid.len());
                    valid.iter().nth(index).map(BattleAction::SelectCharacterTarget)
                }
            }

            LegalActions::SelectStackCardPrompt { valid } => {
                if valid.is_empty() {
                    None
                } else {
                    let index = fastrand::usize(..valid.len());
                    valid.iter().nth(index).map(BattleAction::SelectStackCardTarget)
                }
            }

            LegalActions::SelectPromptChoicePrompt { choice_count } => {
                if *choice_count == 0 {
                    None
                } else {
                    Some(BattleAction::SelectPromptChoice(fastrand::usize(..*choice_count)))
                }
            }

            LegalActions::SelectEnergyValuePrompt { minimum, maximum } => {
                if maximum >= minimum {
                    Some(BattleAction::SelectEnergyAdditionalCost(Energy(fastrand::u32(
                        minimum.0..=maximum.0,
                    ))))
                } else {
                    None
                }
            }
        }
    }
}
