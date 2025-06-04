use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardId;
use display_data::object_position::{Position, StackType};

use crate::display_actions::apply_battle_display_action;

/// Returns an alternate position for a card based on display logic, e.g.
/// showing it in a browser.
pub fn position(battle: &BattleState, card_id: CardId, position: Position) -> Position {
    if let Some(prompt) = &battle.prompt
        && let Some(source_card_id) = prompt.source.card_id()
        && source_card_id == card_id
    {
        return Position::OnStack(StackType::Default);
    }

    if let Some(browser_source) = apply_battle_display_action::current_browser_source()
        && position == browser_source
    {
        return Position::Browser;
    }

    position
}
