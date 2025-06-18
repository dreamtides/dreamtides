use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardId;
use display_data::object_position::Position;

use crate::core::response_builder::ResponseBuilder;
use crate::display_actions::apply_battle_display_action;
use crate::rendering::positions;

/// Returns an alternate position for a card based on display logic, e.g.
/// showing it in a browser.
pub fn position(
    builder: &ResponseBuilder,
    battle: &BattleState,
    card_id: CardId,
    position: Position,
) -> Position {
    if let Some(prompt) = &battle.prompt
        && let Some(source_card_id) = prompt.source.card_id()
        && source_card_id == card_id
    {
        return Position::OnStack(positions::current_stack_type(builder, battle));
    }

    for_browser(position)
}

/// Returns the position for a card in the browser, if it is the current
/// browser.
pub fn for_browser(position: Position) -> Position {
    if let Some(browser_source) = apply_battle_display_action::current_browser_source()
        && position == browser_source
    {
        Position::Browser
    } else {
        position
    }
}
