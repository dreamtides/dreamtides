use action_data::battle_action::BattleAction;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_status::BattleStatus;
use battle_data::prompt_types::prompt_data::Prompt;
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
            return match &prompt_data.prompt {
                Prompt::ChooseCharacter { valid } => {
                    valid.iter().map(|&id| BattleAction::SelectCharacter(id)).collect()
                }
                Prompt::ChooseStackCard { valid } => {
                    valid.iter().map(|&id| BattleAction::SelectStackCard(id)).collect()
                }
                Prompt::Choose { choices } => choices
                    .iter()
                    .enumerate()
                    .map(|(i, _)| BattleAction::SelectPromptChoice(i))
                    .collect(),
                Prompt::ChooseNumber { minimum, maximum, .. } => {
                    (*minimum..=*maximum).map(|i| BattleAction::SelectNumber(i)).collect()
                }
            };
        } else {
            return vec![];
        }
    }

    let is_active_player = battle.turn.active_player == player;
    let mut actions = Vec::new();

    let stack = battle.cards.stack();
    let has_stack_cards = !stack.is_empty();

    if has_stack_cards {
        if let Some(top_card_id) = stack.last() {
            if let Some(top_card) = battle.cards.card(*top_card_id) {
                if top_card.controller() != player {
                    // If the player doesn't control the top card, they can resolve the stack
                    actions.push(BattleAction::ResolveStack);
                }
            }
        }
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

    if is_active_player && !has_stack_cards {
        actions.push(BattleAction::EndTurn);
    }

    actions
}

/// Returns the player who can currently take game actions in the provided
/// [BattleData] state, or None if the battle has ended.
pub fn next_to_act(battle: &BattleData) -> Option<PlayerName> {
    if matches!(battle.status, BattleStatus::GameOver { .. }) {
        return None;
    }

    // If there's an active prompt, the prompted player is next to act
    if let Some(prompt_data) = &battle.prompt {
        return Some(prompt_data.player);
    }

    if let Some(top_card_id) = battle.cards.stack().last() {
        if let Some(top_card) = battle.cards.card(*top_card_id) {
            let controller = top_card.controller();
            // The opponent of the controller of the top card on the stack gets to act
            return Some(controller.opponent());
        }
    }

    Some(battle.turn.active_player)
}
