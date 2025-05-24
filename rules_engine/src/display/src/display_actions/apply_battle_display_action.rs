use std::sync::{LazyLock, Mutex};

use action_data::battle_display_action::{BattleDisplayAction, CardBrowserType};
use display_data::battle_view::DisplayPlayer;
use display_data::object_position::Position;

static CARD_BROWSER_SOURCE: LazyLock<Mutex<Option<Position>>> = LazyLock::new(|| Mutex::new(None));

/// Modifies the display state of a battle.
pub fn execute(action: BattleDisplayAction) {
    match action {
        BattleDisplayAction::BrowseCards(card_browser_type) => browse_cards(card_browser_type),
        BattleDisplayAction::CloseCardBrowser => close_card_browser(),
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

    *CARD_BROWSER_SOURCE.lock().unwrap() = Some(source_position);
}

fn close_card_browser() {
    *CARD_BROWSER_SOURCE.lock().unwrap() = None;
}

/// Returns whether a card browser is currently active and what source position
/// it's browsing.
pub fn current_browser_source() -> Option<Position> {
    *CARD_BROWSER_SOURCE.lock().unwrap()
}
