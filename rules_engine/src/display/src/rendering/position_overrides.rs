use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardId;
use display_data::object_position::Position;

use crate::display_actions::apply_battle_display_action;

/// Returns an alternate position for a card based on display logic, e.g.
/// showing it in a browser.
pub fn position(_battle: &BattleState, _card_id: CardId, position: Position) -> Position {
    if let Some(browser_source) = apply_battle_display_action::current_browser_source() {
        if position == browser_source {
            return Position::Browser;
        }
    }
    position
}
