use action_data::battle_display_action::{BattleDisplayAction, CardBrowserType};
use core_data::numerics::Energy;
use display_data::battle_view::DisplayPlayer;
use display_data::object_position::Position;

use crate::display_actions::display_state;

/// Modifies the display state of a battle.
pub fn execute(action: BattleDisplayAction) {
    match action {
        BattleDisplayAction::BrowseCards(card_browser_type) => browse_cards(card_browser_type),
        BattleDisplayAction::CloseCardBrowser => close_card_browser(),
        BattleDisplayAction::SetSelectedEnergyAdditionalCost(energy) => {
            set_selected_energy_additional_cost(energy);
        }
    }
}

fn browse_cards(card_browser: CardBrowserType) {
    let source_position = match card_browser {
        CardBrowserType::UserDeck => Position::InDeck(DisplayPlayer::User),
        CardBrowserType::EnemyDeck => Position::InDeck(DisplayPlayer::Enemy),
        CardBrowserType::UserVoid => Position::InVoid(DisplayPlayer::User),
        CardBrowserType::EnemyVoid => Position::InVoid(DisplayPlayer::Enemy),
        CardBrowserType::UserStatus => Position::InPlayerStatus(DisplayPlayer::User),
        CardBrowserType::EnemyStatus => Position::InPlayerStatus(DisplayPlayer::Enemy),
    };

    display_state::set_card_browser_source(Some(source_position));
}

fn close_card_browser() {
    display_state::set_card_browser_source(None);
}

fn set_selected_energy_additional_cost(energy: Energy) {
    display_state::set_selected_energy_additional_cost(Some(energy));
}

/// Returns whether a card browser is currently active and what source position
/// it's browsing.
pub fn current_browser_source() -> Option<Position> {
    display_state::get_card_browser_source()
}
