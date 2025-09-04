use ability_data::effect::ModelEffectChoiceIndex;
use battle_state::actions::battle_actions::{
    BattleAction, CardOrderSelectionTarget, DeckCardSelectedOrder,
};
use battle_state::battle::card_id::{
    ActivatedAbilityId, CharacterId, HandCardId, StackCardId, VoidCardId,
};
use battle_state::battle_cards::card_set::CardSet;
use battle_state::prompt_types::prompt_data::SelectDeckCardOrderPrompt;
use bit_set::BitSet;
use core_data::numerics::Energy;
use fastrand;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LegalActions {
    NoActionsGameOver,
    NoActionsOpponentPrompt,
    NoActionsOpponentPriority,
    NoActionsInCurrentPhase,
    Standard {
        actions: StandardLegalActions,
    },
    SelectCharacterPrompt {
        valid: CardSet<CharacterId>,
    },
    SelectStackCardPrompt {
        valid: CardSet<StackCardId>,
    },
    SelectVoidCardPrompt {
        valid: CardSet<VoidCardId>,
        current: CardSet<VoidCardId>,
        maximum_selection: usize,
    },
    SelectHandCardPrompt {
        valid: CardSet<HandCardId>,
        current: CardSet<HandCardId>,
        target_count: usize,
    },
    SelectPromptChoicePrompt {
        choice_count: usize,
    },
    SelectEnergyValuePrompt {
        minimum: Energy,
        maximum: Energy,
    },
    SelectDeckCardOrder {
        current: SelectDeckCardOrderPrompt,
    },
    ModalEffectPrompt {
        valid_choices: BitSet<usize>,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StandardLegalActions {
    pub primary: PrimaryLegalAction,
    pub play_card_from_hand: CardSet<HandCardId>,
    pub play_card_from_void: CardSet<VoidCardId>,
    pub activate_abilities: Vec<ActivatedAbilityId>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PrimaryLegalAction {
    PassPriority,
    EndTurn,
    StartNextTurn,
}

pub enum ForPlayer {
    Agent,
    Human,
}

impl LegalActions {
    pub fn is_prompt(&self) -> bool {
        matches!(
            self,
            LegalActions::SelectCharacterPrompt { .. }
                | LegalActions::SelectStackCardPrompt { .. }
                | LegalActions::SelectVoidCardPrompt { .. }
                | LegalActions::SelectHandCardPrompt { .. }
                | LegalActions::SelectPromptChoicePrompt { .. }
                | LegalActions::SelectEnergyValuePrompt { .. }
                | LegalActions::SelectDeckCardOrder { .. }
                | LegalActions::ModalEffectPrompt { .. }
        )
    }

    pub fn contains(&self, action: BattleAction, for_player: ForPlayer) -> bool {
        match action {
            BattleAction::Debug(..) => true,
            BattleAction::PlayCardFromHand(hand_card_id) => {
                if let LegalActions::Standard { actions } = self {
                    actions.play_card_from_hand.contains(hand_card_id)
                } else {
                    false
                }
            }
            BattleAction::PlayCardFromVoid(void_card_id) => {
                if let LegalActions::Standard { actions } = self {
                    actions.play_card_from_void.contains(void_card_id)
                } else {
                    false
                }
            }
            BattleAction::ActivateAbility(activated_ability_id) => {
                if let LegalActions::Standard { actions } = self {
                    actions.activate_abilities.contains(&activated_ability_id)
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
            BattleAction::SelectVoidCardTarget(void_card_id) => {
                if let LegalActions::SelectVoidCardPrompt { valid, current, maximum_selection } =
                    self
                {
                    match for_player {
                        // Agent cannot remove cards from the selected set
                        ForPlayer::Agent => {
                            !current.contains(void_card_id)
                                && valid.contains(void_card_id)
                                && current.len() < *maximum_selection
                        }

                        // In single card mode, always allow toggling
                        ForPlayer::Human if *maximum_selection == 1 => valid.contains(void_card_id),

                        // In multi-card mode, cannot select another card when
                        // at max selection
                        ForPlayer::Human => {
                            valid.contains(void_card_id)
                                && (current.len() < *maximum_selection
                                    || current.contains(void_card_id))
                        }
                    }
                } else {
                    false
                }
            }
            BattleAction::SubmitVoidCardTargets => {
                if let LegalActions::SelectVoidCardPrompt { current, maximum_selection, .. } = self
                {
                    !current.is_empty() && current.len() <= *maximum_selection
                } else {
                    false
                }
            }
            BattleAction::SelectHandCardTarget(hand_card_id) => {
                if let LegalActions::SelectHandCardPrompt { valid, current, target_count } = self {
                    match for_player {
                        ForPlayer::Agent => {
                            !current.contains(hand_card_id)
                                && valid.contains(hand_card_id)
                                && current.len() < *target_count
                        }

                        ForPlayer::Human if *target_count == 1 => valid.contains(hand_card_id),

                        ForPlayer::Human => {
                            valid.contains(hand_card_id)
                                && (current.len() < *target_count || current.contains(hand_card_id))
                        }
                    }
                } else {
                    false
                }
            }
            BattleAction::SubmitHandCardTargets => {
                if let LegalActions::SelectHandCardPrompt { current, target_count, .. } = self {
                    current.len() == *target_count
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
            BattleAction::SelectOrderForDeckCard(order) => {
                if let LegalActions::SelectDeckCardOrder { current } = self {
                    match for_player {
                        ForPlayer::Agent => {
                            self.contains_deck_card_order_for_agent(current, &order)
                        }
                        ForPlayer::Human => {
                            self.contains_deck_card_order_for_human(current, &order)
                        }
                    }
                } else {
                    false
                }
            }
            BattleAction::SubmitDeckCardOrder => {
                matches!(self, LegalActions::SelectDeckCardOrder { .. })
            }
            BattleAction::SubmitMulligan => todo!("Implement this"),
            BattleAction::SelectModalEffectChoice(modal_choice_index) => {
                if let LegalActions::ModalEffectPrompt { valid_choices } = self {
                    valid_choices.contains(modal_choice_index.value())
                } else {
                    false
                }
            }
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
            LegalActions::SelectVoidCardPrompt { .. } => false,
            LegalActions::SelectHandCardPrompt { .. } => false,
            LegalActions::SelectPromptChoicePrompt { choice_count } => *choice_count == 0,
            LegalActions::SelectEnergyValuePrompt { minimum, maximum } => maximum < minimum,
            LegalActions::SelectDeckCardOrder { .. } => false,
            LegalActions::ModalEffectPrompt { valid_choices } => valid_choices.is_empty(),
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
                let play_void_cards_count = actions.play_card_from_void.len();
                let ability_count = actions.activate_abilities.len();
                primary_count + play_cards_count + play_void_cards_count + ability_count
            }

            LegalActions::SelectCharacterPrompt { valid } => valid.len(),
            LegalActions::SelectStackCardPrompt { valid } => valid.len(),
            LegalActions::SelectVoidCardPrompt { valid, current, maximum_selection } => {
                if current.len() == *maximum_selection || current.len() == valid.len() {
                    1 // SubmitVoidCardTargets, only for AI when at max selection
                } else {
                    valid.len() - current.len()
                }
            }
            LegalActions::SelectHandCardPrompt { valid, current, target_count } => {
                if current.len() == *target_count || current.len() == valid.len() {
                    1 // SubmitHandCardTargets, only for AI when at max selection
                } else {
                    valid.len() - current.len()
                }
            }
            LegalActions::SelectPromptChoicePrompt { choice_count } => *choice_count,
            LegalActions::SelectEnergyValuePrompt { minimum, maximum } => {
                if maximum >= minimum {
                    (maximum.0 - minimum.0 + 1) as usize
                } else {
                    0
                }
            }
            LegalActions::SelectDeckCardOrder { current } => {
                let has_next_card =
                    current.initial.iter().any(|card_id| !current.moved.contains(*card_id));
                if has_next_card {
                    1 + // SubmitDeckCardOrder action
                    (current.deck.len() + 1) + // Deck positions: 0..=deck.len() (can place at end)
                    1 // Void position 0 (void is unordered)
                } else {
                    1 // Only SubmitDeckCardOrder when no cards left to move
                }
            }
            LegalActions::ModalEffectPrompt { valid_choices } => valid_choices.len(),
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
                            standard_actions.play_card_from_hand.iter().find(|&card_id| {
                                !actions.contains(&BattleAction::PlayCardFromHand(card_id))
                            })
                        {
                            Some(BattleAction::PlayCardFromHand(card_id))
                        } else if let Some(card_id) =
                            standard_actions.play_card_from_void.iter().find(|&card_id| {
                                !actions.contains(&BattleAction::PlayCardFromVoid(card_id))
                            })
                        {
                            Some(BattleAction::PlayCardFromVoid(card_id))
                        } else {
                            standard_actions
                                .activate_abilities
                                .iter()
                                .find(|&&ability| {
                                    !actions.contains(&BattleAction::ActivateAbility(ability))
                                })
                                .map(|&ability| BattleAction::ActivateAbility(ability))
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

            LegalActions::SelectVoidCardPrompt { valid, current, maximum_selection } => {
                if current.len() == *maximum_selection || current.len() == valid.len() {
                    // For now only submit for AI when at max selection
                    if actions.contains(&BattleAction::SubmitVoidCardTargets) {
                        None
                    } else {
                        Some(BattleAction::SubmitVoidCardTargets)
                    }
                } else {
                    valid
                        .iter()
                        .find(|id| {
                            !actions.contains(&BattleAction::SelectVoidCardTarget(*id))
                                && !current.contains(*id)
                        })
                        .map(BattleAction::SelectVoidCardTarget)
                }
            }

            LegalActions::SelectHandCardPrompt { valid, current, target_count } => {
                if current.len() == *target_count || current.len() == valid.len() {
                    if actions.contains(&BattleAction::SubmitHandCardTargets) {
                        None
                    } else {
                        Some(BattleAction::SubmitHandCardTargets)
                    }
                } else {
                    valid
                        .iter()
                        .find(|id| {
                            !actions.contains(&BattleAction::SelectHandCardTarget(*id))
                                && !current.contains(*id)
                        })
                        .map(BattleAction::SelectHandCardTarget)
                }
            }

            LegalActions::SelectPromptChoicePrompt { choice_count } => (0..*choice_count)
                .find(|&i| !actions.contains(&BattleAction::SelectPromptChoice(i)))
                .map(BattleAction::SelectPromptChoice),

            LegalActions::SelectEnergyValuePrompt { minimum, maximum } => (minimum.0..=maximum.0)
                .find(|&e| !actions.contains(&BattleAction::SelectEnergyAdditionalCost(Energy(e))))
                .map(|e| BattleAction::SelectEnergyAdditionalCost(Energy(e))),

            LegalActions::SelectDeckCardOrder { current } => {
                if !actions.contains(&BattleAction::SubmitDeckCardOrder) {
                    return Some(BattleAction::SubmitDeckCardOrder);
                }

                if let Some(next_card) =
                    current.initial.iter().find(|card_id| !current.moved.contains(**card_id))
                {
                    for position in 0..=current.deck.len() {
                        let action = BattleAction::SelectOrderForDeckCard(DeckCardSelectedOrder {
                            target: CardOrderSelectionTarget::Deck(position),
                            card_id: *next_card,
                        });
                        if !actions.contains(&action) {
                            return Some(action);
                        }
                    }

                    let action = BattleAction::SelectOrderForDeckCard(DeckCardSelectedOrder {
                        target: CardOrderSelectionTarget::Void,
                        card_id: *next_card,
                    });
                    if !actions.contains(&action) {
                        return Some(action);
                    }
                }

                None
            }

            LegalActions::ModalEffectPrompt { valid_choices } => valid_choices
                .iter()
                .find(|&i| {
                    !actions
                        .contains(&BattleAction::SelectModalEffectChoice(ModelEffectChoiceIndex(i)))
                })
                .map(|i| BattleAction::SelectModalEffectChoice(ModelEffectChoiceIndex(i))),
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

                for card_id in actions.play_card_from_void.iter() {
                    result.push(BattleAction::PlayCardFromVoid(card_id));
                }

                for ability in actions.activate_abilities.iter() {
                    result.push(BattleAction::ActivateAbility(*ability));
                }

                result
            }

            LegalActions::SelectCharacterPrompt { valid } => {
                valid.iter().map(BattleAction::SelectCharacterTarget).collect::<Vec<_>>()
            }

            LegalActions::SelectStackCardPrompt { valid } => {
                valid.iter().map(BattleAction::SelectStackCardTarget).collect::<Vec<_>>()
            }

            LegalActions::SelectVoidCardPrompt { valid, current, maximum_selection } => {
                if current.len() == *maximum_selection || current.len() == valid.len() {
                    // For now only submit for AI when at max selection or when
                    // all options are selected
                    vec![BattleAction::SubmitVoidCardTargets]
                } else {
                    valid
                        .iter()
                        .filter(|&id| !current.contains(id))
                        .map(BattleAction::SelectVoidCardTarget)
                        .collect::<Vec<_>>()
                }
            }

            LegalActions::SelectHandCardPrompt { valid, current, target_count } => {
                if current.len() == *target_count || current.len() == valid.len() {
                    vec![BattleAction::SubmitHandCardTargets]
                } else {
                    valid
                        .iter()
                        .filter(|&id| !current.contains(id))
                        .map(BattleAction::SelectHandCardTarget)
                        .collect::<Vec<_>>()
                }
            }

            LegalActions::SelectPromptChoicePrompt { choice_count } => {
                (0..*choice_count).map(BattleAction::SelectPromptChoice).collect::<Vec<_>>()
            }

            LegalActions::SelectEnergyValuePrompt { minimum, maximum } => (minimum.0..=maximum.0)
                .map(|e| BattleAction::SelectEnergyAdditionalCost(Energy(e)))
                .collect::<Vec<_>>(),

            LegalActions::SelectDeckCardOrder { current } => {
                let mut result = vec![BattleAction::SubmitDeckCardOrder];

                if let Some(next_card) =
                    current.initial.iter().find(|card_id| !current.moved.contains(**card_id))
                {
                    for position in 0..=current.deck.len() {
                        result.push(BattleAction::SelectOrderForDeckCard(DeckCardSelectedOrder {
                            target: CardOrderSelectionTarget::Deck(position),
                            card_id: *next_card,
                        }));
                    }

                    result.push(BattleAction::SelectOrderForDeckCard(DeckCardSelectedOrder {
                        target: CardOrderSelectionTarget::Void,
                        card_id: *next_card,
                    }));
                }

                result
            }

            LegalActions::ModalEffectPrompt { valid_choices } => valid_choices
                .iter()
                .map(|i| BattleAction::SelectModalEffectChoice(ModelEffectChoiceIndex(i)))
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
                            .get_at_index(remaining_index)
                            .map(BattleAction::PlayCardFromHand)
                    } else {
                        let void_index = remaining_index - actions.play_card_from_hand.len();
                        if void_index < actions.play_card_from_void.len() {
                            actions
                                .play_card_from_void
                                .get_at_index(void_index)
                                .map(BattleAction::PlayCardFromVoid)
                        } else {
                            let ability_index = void_index - actions.play_card_from_void.len();
                            actions
                                .activate_abilities
                                .get(ability_index)
                                .map(|&ability| BattleAction::ActivateAbility(ability))
                        }
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

            LegalActions::SelectVoidCardPrompt { .. } => {
                let all_actions = self.all();
                if all_actions.is_empty() {
                    None
                } else {
                    let index = fastrand::usize(..all_actions.len());
                    Some(all_actions[index])
                }
            }

            LegalActions::SelectHandCardPrompt { .. } => {
                let all_actions = self.all();
                if all_actions.is_empty() {
                    None
                } else {
                    let index = fastrand::usize(..all_actions.len());
                    Some(all_actions[index])
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

            LegalActions::SelectDeckCardOrder { .. } => {
                let all_actions = self.all();
                if all_actions.is_empty() {
                    None
                } else {
                    let index = fastrand::usize(..all_actions.len());
                    Some(all_actions[index])
                }
            }

            LegalActions::ModalEffectPrompt { .. } => {
                let all_actions = self.all();
                if all_actions.is_empty() {
                    None
                } else {
                    let index = fastrand::usize(..all_actions.len());
                    Some(all_actions[index])
                }
            }
        }
    }

    fn contains_deck_card_order_for_agent(
        &self,
        current: &SelectDeckCardOrderPrompt,
        order: &DeckCardSelectedOrder,
    ) -> bool {
        if let Some(next_card) =
            current.initial.iter().find(|card_id| !current.moved.contains(**card_id))
        {
            if order.card_id != *next_card {
                return false;
            }

            match order.target {
                CardOrderSelectionTarget::Deck(position) => position <= current.deck.len(),
                CardOrderSelectionTarget::Void => true,
            }
        } else {
            false
        }
    }

    fn contains_deck_card_order_for_human(
        &self,
        current: &SelectDeckCardOrderPrompt,
        order: &DeckCardSelectedOrder,
    ) -> bool {
        if !current.initial.contains(&order.card_id) {
            return false;
        }

        match order.target {
            CardOrderSelectionTarget::Deck(position) => position <= current.deck.len(),
            CardOrderSelectionTarget::Void => true,
        }
    }
}
