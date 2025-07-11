use battle_state::actions::battle_actions::CardOrderSelectionTargetDiscriminants;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, DeckCardId};
use battle_state::prompt_types::prompt_data::PromptType;
use display_data::object_position::{ObjectPosition, Position};

use crate::core::response_builder::ResponseBuilder;
use crate::display_actions::{apply_battle_display_action, display_state};
use crate::rendering::positions;

/// Returns an alternate object position for a card based on display logic, e.g.
/// showing it in a browser.
pub fn object_position(
    builder: &ResponseBuilder,
    battle: &BattleState,
    card_id: CardId,
    base_object_position: ObjectPosition,
) -> ObjectPosition {
    let position = if let Some(prompt) = &battle.prompt
        && let Some(source_card_id) = prompt.source.card_id()
        && source_card_id == card_id
    {
        Position::OnStack(positions::current_stack_type(builder, battle))
    } else {
        base_object_position.position
    };
    let position = for_hidden_stack(builder, position);
    let (position, sorting_key, sorting_sub_key) =
        for_card_order_browser(battle, card_id, position, base_object_position.sorting_key);
    let position = for_browser(builder, position);

    ObjectPosition { position, sorting_key, sorting_sub_key }
}

/// Returns the position for a card in the browser, if it is the current
/// browser.
pub fn for_browser(builder: &ResponseBuilder, position: Position) -> Position {
    if let Some(browser_source) = apply_battle_display_action::current_browser_source(builder)
        && position == browser_source
    {
        Position::Browser
    } else {
        position
    }
}

/// Returns the position for a card if the stack is hidden.
fn for_hidden_stack(builder: &ResponseBuilder, position: Position) -> Position {
    if display_state::is_stack_hidden(builder) && matches!(position, Position::OnStack(_)) {
        Position::OnScreenStorage
    } else {
        position
    }
}

/// Returns the position for a card in the card order browser, if it is being
/// ordered.
fn for_card_order_browser(
    battle: &BattleState,
    card_id: CardId,
    position: Position,
    base_sorting_key: u32,
) -> (Position, u32, u32) {
    if let Some(prompt) = &battle.prompt
        && let PromptType::SelectDeckCardOrder { prompt: deck_prompt } = &prompt.prompt_type
    {
        let deck_card_id = DeckCardId(card_id);
        if deck_prompt.initial.contains(&deck_card_id) {
            if deck_prompt.void.contains(deck_card_id) {
                return (
                    Position::CardOrderSelector(CardOrderSelectionTargetDiscriminants::Void),
                    base_sorting_key,
                    0,
                );
            } else if let Some(position_in_deck) =
                deck_prompt.deck.iter().position(|&id| id == deck_card_id)
            {
                return (
                    Position::CardOrderSelector(CardOrderSelectionTargetDiscriminants::Deck),
                    position_in_deck as u32,
                    0,
                );
            } else {
                return (
                    Position::CardOrderSelector(CardOrderSelectionTargetDiscriminants::Deck),
                    base_sorting_key,
                    0,
                );
            }
        }
    }
    (position, base_sorting_key, 0)
}
