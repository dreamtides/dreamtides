use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::prompt_types::prompt_data::PromptType;
use bit_set::BitSet;
use core_data::types::PlayerName;

use crate::legal_action_queries::can_play_cards::{self, FastOnly};
use crate::legal_action_queries::legal_actions_data::{
    LegalActions, PrimaryLegalAction, StandardLegalActions,
};
use crate::legal_action_queries::{can_activate_abilities, legal_modal_effect_choices};

pub fn compute(battle: &BattleState, player: PlayerName) -> LegalActions {
    if matches!(battle.status, BattleStatus::GameOver { .. }) {
        return LegalActions::NoActionsGameOver;
    }

    // If there's an active prompt, the only legal actions are those
    // corresponding to the prompt
    if let Some(prompt_data) = battle.prompts.front() {
        if prompt_data.player != player {
            return LegalActions::NoActionsOpponentPrompt;
        }

        return match &prompt_data.prompt_type {
            PromptType::ChooseCharacter { valid, .. } => {
                LegalActions::SelectCharacterPrompt { valid: valid.clone() }
            }
            PromptType::ChooseStackCard { valid, .. } => {
                LegalActions::SelectStackCardPrompt { valid: valid.clone() }
            }
            PromptType::ChooseVoidCard(prompt) => LegalActions::SelectVoidCardPrompt {
                valid: prompt.valid.clone(),
                current: prompt.selected.clone(),
                maximum_selection: prompt.maximum_selection as usize,
            },
            PromptType::Choose { choices } => {
                LegalActions::SelectPromptChoicePrompt { choice_count: choices.len() }
            }
            PromptType::ChooseEnergyValue { minimum, maximum } => {
                LegalActions::SelectEnergyValuePrompt { minimum: *minimum, maximum: *maximum }
            }
            PromptType::ModalEffect(prompt) => {
                let source = prompt_data.source;
                let mut valid_choices = BitSet::<usize>::default();
                for (i, choice) in prompt.choices.iter().enumerate() {
                    if legal_modal_effect_choices::is_legal_choice(battle, source, player, choice) {
                        valid_choices.insert(i);
                    }
                }
                LegalActions::ModalEffectPrompt { valid_choices }
            }
            PromptType::SelectDeckCardOrder { prompt } => {
                LegalActions::SelectDeckCardOrder { current: prompt.clone() }
            }
        };
    }

    if let Some(priority) = battle.stack_priority {
        if priority == player {
            LegalActions::Standard {
                actions: standard_legal_actions(
                    battle,
                    player,
                    PrimaryLegalAction::PassPriority,
                    FastOnly::Yes,
                ),
            }
        } else {
            LegalActions::NoActionsOpponentPriority
        }
    } else if battle.turn.active_player == player && battle.phase == BattleTurnPhase::Main {
        LegalActions::Standard {
            actions: standard_legal_actions(
                battle,
                player,
                PrimaryLegalAction::EndTurn,
                FastOnly::No,
            ),
        }
    } else if battle.turn.active_player != player && battle.phase == BattleTurnPhase::Ending {
        LegalActions::Standard {
            actions: standard_legal_actions(
                battle,
                player,
                PrimaryLegalAction::StartNextTurn,
                FastOnly::Yes,
            ),
        }
    } else {
        LegalActions::NoActionsInCurrentPhase
    }
}

/// Returns the player who is next to act.
///
/// Returns None if the game is over.
pub fn next_to_act(battle: &BattleState) -> Option<PlayerName> {
    if matches!(battle.status, BattleStatus::GameOver { .. }) {
        return None;
    }

    if let Some(prompt_data) = &battle.prompts.front() {
        return Some(prompt_data.player);
    }

    if let Some(priority) = battle.stack_priority {
        Some(priority)
    } else if battle.phase != BattleTurnPhase::Ending {
        Some(battle.turn.active_player)
    } else {
        Some(battle.turn.active_player.opponent())
    }
}

fn standard_legal_actions(
    battle: &BattleState,
    player: PlayerName,
    primary: PrimaryLegalAction,
    fast_only: FastOnly,
) -> StandardLegalActions {
    StandardLegalActions {
        primary,
        play_card_from_hand: can_play_cards::from_hand(battle, player, fast_only),
        play_card_from_void: if battle
            .ability_state
            .has_play_from_void_ability
            .player(player)
            .is_empty()
        {
            vec![]
        } else {
            can_play_cards::from_void(battle, player, fast_only)
        },
        activate_abilities: if battle.activated_abilities.player(player).characters.is_empty() {
            vec![]
        } else {
            can_activate_abilities::for_player(battle, player, fast_only)
        },
    }
}
