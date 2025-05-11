use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::types::PlayerName;

use crate::legal_action_queries::can_play_cards::{self, FastOnly};
use crate::legal_action_queries::legal_actions_data::{
    LegalActions, PrimaryLegalAction, StandardLegalActions,
};

#[derive(Debug, Clone)]
pub struct LegalActionOptions {
    pub for_human_player: bool,
}

pub fn compute(
    battle: &BattleState,
    player: PlayerName,
    _options: LegalActionOptions,
) -> LegalActions {
    if matches!(battle.status, BattleStatus::GameOver { .. }) {
        return LegalActions::NoActionsGameOver;
    }

    // If there's an active prompt, the only legal actions are those
    // corresponding to the prompt
    if let Some(prompt_data) = &battle.prompt {
        if prompt_data.player != player {
            return LegalActions::NoActionsOpponentPrompt;
        }

        return match &prompt_data.prompt_type {
            PromptType::ChooseCharacter { valid } => {
                LegalActions::SelectCharacterPrompt { valid: valid.clone() }
            }
            PromptType::ChooseStackCard { valid } => {
                LegalActions::SelectStackCardPrompt { valid: valid.clone() }
            }
            PromptType::Choose { choices } => {
                LegalActions::SelectPromptChoicePrompt { choice_count: choices.len() }
            }
            PromptType::ChooseEnergyValue { minimum, maximum, .. } => {
                LegalActions::SelectEnergyValuePrompt { minimum: *minimum, maximum: *maximum }
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

fn standard_legal_actions(
    battle: &BattleState,
    player: PlayerName,
    primary: PrimaryLegalAction,
    fast_only: FastOnly,
) -> StandardLegalActions {
    StandardLegalActions {
        primary,
        play_card_from_hand: can_play_cards::from_hand(battle, player, fast_only),
    }
}
