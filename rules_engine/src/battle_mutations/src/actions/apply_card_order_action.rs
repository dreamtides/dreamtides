use battle_queries::{battle_trace, panic_with};
use battle_state::actions::battle_actions::{CardOrderSelectionTarget, DeckCardSelectedOrder};
use battle_state::battle::battle_state::BattleState;
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::types::PlayerName;

use crate::card_mutations::move_card;

pub fn execute_select_order_for_deck_card(
    battle: &mut BattleState,
    player: PlayerName,
    order: DeckCardSelectedOrder,
) {
    battle_trace!("Selecting order for deck card", battle, order);

    let Some(prompt) = &mut battle.prompt else {
        panic_with!("No prompt found", battle, player);
    };

    if prompt.player != player {
        panic_with!("Prompt player mismatch", battle, player);
    }

    let PromptType::SelectDeckCardOrder { prompt: deck_order_prompt } = &mut prompt.prompt_type
    else {
        panic_with!("Prompt type mismatch", battle, player);
    };

    let card_id = order.card_id;

    deck_order_prompt.moved.insert(card_id);
    deck_order_prompt.deck.retain(|&id| id != card_id);
    deck_order_prompt.void.remove(card_id);

    match order.target {
        CardOrderSelectionTarget::Deck(position) => {
            let insert_pos = position.min(deck_order_prompt.deck.len());
            deck_order_prompt.deck.insert(insert_pos, card_id);
        }
        CardOrderSelectionTarget::Void => {
            deck_order_prompt.void.insert(card_id);
        }
    }
}

pub fn execute_submit_deck_card_order(battle: &mut BattleState, player: PlayerName) {
    battle_trace!("Submitting deck card order", battle, player);

    let Some(prompt) = battle.prompt.take() else {
        panic_with!("No prompt found", battle, player);
    };

    if prompt.player != player {
        panic_with!("Prompt player mismatch", battle, player);
    }

    let PromptType::SelectDeckCardOrder { prompt: deck_order_prompt } = prompt.prompt_type else {
        panic_with!("Prompt type mismatch", battle, player);
    };

    for card_id in &deck_order_prompt.void {
        move_card::from_deck_to_void(battle, prompt.source, player, card_id);
    }

    let top_of_deck = battle.cards.top_of_deck_mut(player);
    top_of_deck.retain(|&card_id| !deck_order_prompt.deck.contains(&card_id));
    for card_id in &deck_order_prompt.deck {
        top_of_deck.push(*card_id);
    }
}
