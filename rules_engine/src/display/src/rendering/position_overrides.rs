use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::ForPlayer;
use battle_state::actions::battle_actions::{BattleAction, CardOrderSelectionTargetDiscriminants};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, DeckCardId, VoidCardId};
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::types::PlayerName;
use display_data::battle_view::DisplayPlayer;
use display_data::object_position::{ObjectPosition, Position, StackType};

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
        && prompt.source.card_id() == Some(card_id)
    {
        Position::OnStack(positions::current_stack_type(builder, battle))
    } else {
        base_object_position.position
    };
    let position = for_hidden_stack(builder, position);
    let position = for_stack_during_prompt(battle, position);
    let position = for_playable_void_cards(builder, battle, card_id, position);
    let object_position = for_top_of_deck(battle, card_id, ObjectPosition {
        position,
        sorting_key: base_object_position.sorting_key,
    });
    let object_position = for_card_order_browser(battle, card_id, object_position);
    let position = for_browser(builder, object_position.position);
    ObjectPosition { position, sorting_key: object_position.sorting_key }
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

/// Returns the position for a card if the stack is being displayed during a
/// prompt.
fn for_stack_during_prompt(battle: &BattleState, position: Position) -> Position {
    // Minimize the stack during the "select deck card order" prompt since it's
    // visually distracting.
    if let Some(prompt) = &battle.prompt
        && let PromptType::SelectDeckCardOrder { .. } = &prompt.prompt_type
        && matches!(position, Position::OnStack(_))
    {
        Position::OnStack(StackType::TargetingBothBattlefields)
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

/// Returns the position for a void card if it can be played, moving it to hand
/// position.
fn for_playable_void_cards(
    builder: &ResponseBuilder,
    battle: &BattleState,
    card_id: CardId,
    position: Position,
) -> Position {
    if matches!(position, Position::InVoid(DisplayPlayer::User)) {
        let void_card_id = VoidCardId(card_id);
        let legal_actions = legal_actions::compute(battle, builder.display_for_player());
        if legal_actions.contains(BattleAction::PlayCardFromVoid(void_card_id), ForPlayer::Human) {
            return Position::InHand(DisplayPlayer::User);
        }
    }
    position
}

/// Returns the position and sorting key for a card if it is on top of deck.
fn for_top_of_deck(
    battle: &BattleState,
    card_id: CardId,
    base_object_position: ObjectPosition,
) -> ObjectPosition {
    if matches!(base_object_position.position, Position::InDeck(_)) {
        let deck_card_id = DeckCardId(card_id);
        for player in [PlayerName::One, PlayerName::Two] {
            let top_of_deck_list = battle.cards.top_of_deck(player);
            if let Some(position) = top_of_deck_list.iter().position(|&id| id == deck_card_id) {
                let next_display_id = battle.cards.next_object_id_for_display().0 as u32;
                return ObjectPosition {
                    position: base_object_position.position,
                    sorting_key: next_display_id + position as u32,
                };
            }
        }
    }

    base_object_position
}

/// Returns the position for a card in the card order browser, if it is being
/// ordered.
fn for_card_order_browser(
    battle: &BattleState,
    card_id: CardId,
    base_object_position: ObjectPosition,
) -> ObjectPosition {
    if let Some(prompt) = &battle.prompt
        && let PromptType::SelectDeckCardOrder { prompt: deck_prompt } = &prompt.prompt_type
    {
        let deck_card_id = DeckCardId(card_id);
        if deck_prompt.initial.contains(&deck_card_id) {
            if deck_prompt.void.contains(deck_card_id) {
                return ObjectPosition {
                    position: Position::CardOrderSelector(
                        CardOrderSelectionTargetDiscriminants::Void,
                    ),
                    sorting_key: base_object_position.sorting_key,
                };
            } else if let Some(position_in_deck) =
                deck_prompt.deck.iter().position(|&id| id == deck_card_id)
            {
                return ObjectPosition {
                    position: Position::CardOrderSelector(
                        CardOrderSelectionTargetDiscriminants::Deck,
                    ),
                    sorting_key: position_in_deck as u32,
                };
            } else {
                return ObjectPosition {
                    position: Position::CardOrderSelector(
                        CardOrderSelectionTargetDiscriminants::Deck,
                    ),
                    sorting_key: base_object_position.sorting_key,
                };
            }
        }
    }
    base_object_position
}
