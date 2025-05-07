use battle_data::actions::battle_action_data::BattleAction;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_status::BattleStatus;
use battle_data::battle::battle_turn_step::BattleTurnStep;
use battle_data::prompt_types::prompt_data::PromptType;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use tracing::instrument;

use crate::legal_action_queries::can_play_card;

#[derive(Debug, Default, Clone, Copy)]
pub struct LegalActions {
    /// Include 'interface only' actions in the response which don't affect the
    /// game
    ///
    /// These are excluded from AI agent options in order to prevent infinite
    /// loops of game actions which do not progress the game state.
    pub for_human_player: bool,
}

/// List of all legal actions the named player can take in the
/// current battle state.
#[instrument(name = "legal_actions_compute", level = "trace", skip(battle))]
pub fn compute(
    battle: &BattleData,
    player: PlayerName,
    _options: LegalActions,
) -> Vec<BattleAction> {
    if matches!(battle.status, BattleStatus::GameOver { .. }) {
        return vec![];
    }

    // If there's an active prompt, the only legal actions are those
    // corresponding to the prompt
    if let Some(prompt_data) = &battle.prompt {
        if prompt_data.player == player {
            return match &prompt_data.prompt_type {
                PromptType::ChooseCharacter { valid } => {
                    valid.iter().map(|&id| BattleAction::SelectCharacterTarget(id)).collect()
                }
                PromptType::ChooseStackCard { valid } => {
                    valid.iter().map(|&id| BattleAction::SelectStackCardTarget(id)).collect()
                }
                PromptType::Choose { choices } => choices
                    .iter()
                    .enumerate()
                    .map(|(i, _)| BattleAction::SelectPromptChoice(i))
                    .collect(),
                PromptType::ChooseEnergyValue { minimum, maximum, .. } => (minimum.0..=maximum.0)
                    .map(|e| BattleAction::SelectEnergyAdditionalCost(Energy(e)))
                    .collect(),
            };
        } else {
            return vec![];
        }
    }

    if battle.priority != player {
        return vec![];
    }

    let mut actions = Vec::new();
    let has_stack_cards = !battle.cards.stack().is_empty();

    if has_stack_cards {
        actions.push(BattleAction::PassPriority);
    }

    actions.extend(
        battle
            .cards
            .hand(player)
            .iter()
            .filter(|&&id| can_play_card::from_hand(battle, id))
            .copied()
            .map(BattleAction::PlayCardFromHand),
    );

    if !has_stack_cards && battle.step != BattleTurnStep::Ending {
        actions.push(BattleAction::EndTurn);
    }

    if !has_stack_cards && battle.step == BattleTurnStep::Ending {
        actions.push(BattleAction::StartNextTurn);
    }

    actions
}

/// Returns the player who can currently take game actions in the provided
/// [BattleData] state, or None if the battle has ended.
pub fn next_to_act(battle: &BattleData) -> Option<PlayerName> {
    if matches!(battle.status, BattleStatus::GameOver { .. }) {
        return None;
    }

    if let Some(prompt_data) = &battle.prompt {
        return Some(prompt_data.player);
    }

    Some(battle.priority)
}
