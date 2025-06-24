use action_data::battle_display_action::{BattleDisplayAction, CardBrowserType};
use battle_state::actions::debug_battle_action::DebugBattleAction;
use core_data::identifiers::CardName;
use core_data::types::PlayerName;
use display_data::object_position::Position;
use test_utils::battle::test_battle::TestBattle;

#[test]
fn browse_user_void_moves_cards_to_browser() {
    let mut s = TestBattle::builder().connect();

    s.perform_user_action(DebugBattleAction::AddCardToVoid(
        PlayerName::One,
        CardName::MinstrelOfFallingLight,
    ));
    s.perform_user_action(DebugBattleAction::AddCardToVoid(PlayerName::One, CardName::Immolate));

    assert_eq!(s.user_client.cards.user_void().len(), 2, "user void should have 2 cards");
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::Browser).len(),
        0,
        "browser should be empty"
    );

    s.perform_user_action(BattleDisplayAction::BrowseCards(CardBrowserType::UserVoid));

    assert_eq!(s.user_client.cards.user_void().len(), 0, "user void should be empty");
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::Browser).len(),
        2,
        "browser should have 2 cards"
    );
}

#[test]
fn browse_enemy_void_moves_cards_to_browser() {
    let mut s = TestBattle::builder().connect();

    s.perform_user_action(DebugBattleAction::AddCardToVoid(
        PlayerName::Two,
        CardName::MinstrelOfFallingLight,
    ));
    s.perform_user_action(DebugBattleAction::AddCardToVoid(PlayerName::Two, CardName::Immolate));

    assert_eq!(s.user_client.cards.enemy_void().len(), 2, "enemy void should have 2 cards");
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::Browser).len(),
        0,
        "browser should be empty"
    );

    s.perform_user_action(BattleDisplayAction::BrowseCards(CardBrowserType::EnemyVoid));

    assert_eq!(s.user_client.cards.enemy_void().len(), 0, "enemy void should be empty");
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::Browser).len(),
        2,
        "browser should have 2 cards"
    );
}

#[test]
fn close_card_browser_returns_cards_to_void() {
    let mut s = TestBattle::builder().connect();

    s.perform_user_action(DebugBattleAction::AddCardToVoid(
        PlayerName::One,
        CardName::MinstrelOfFallingLight,
    ));
    s.perform_user_action(DebugBattleAction::AddCardToVoid(PlayerName::One, CardName::Immolate));

    s.perform_user_action(BattleDisplayAction::BrowseCards(CardBrowserType::UserVoid));

    assert_eq!(
        s.user_client.cards.user_void().len(),
        0,
        "user void should be empty after browsing"
    );
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::Browser).len(),
        2,
        "browser should have 2 cards"
    );

    s.perform_user_action(BattleDisplayAction::CloseCardBrowser);

    assert_eq!(
        s.user_client.cards.user_void().len(),
        2,
        "user void should have 2 cards after closing"
    );
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::Browser).len(),
        0,
        "browser should be empty after closing"
    );
}

#[test]
fn browse_user_deck_moves_cards_to_browser() {
    let mut s = TestBattle::builder().connect();

    assert!(s.user_client.cards.user_deck().len() > 0, "user deck should have cards");
    let original_deck_count = s.user_client.cards.user_deck().len();
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::Browser).len(),
        0,
        "browser should be empty"
    );

    s.perform_user_action(BattleDisplayAction::BrowseCards(CardBrowserType::UserDeck));

    assert_eq!(s.user_client.cards.user_deck().len(), 0, "user deck should be empty");
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::Browser).len(),
        original_deck_count,
        "browser should have all deck cards"
    );
}

#[test]
fn browse_enemy_deck_moves_cards_to_browser() {
    let mut s = TestBattle::builder().connect();

    assert!(s.user_client.cards.enemy_deck().len() > 0, "enemy deck should have cards");
    let original_deck_count = s.user_client.cards.enemy_deck().len();
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::Browser).len(),
        0,
        "browser should be empty"
    );

    s.perform_user_action(BattleDisplayAction::BrowseCards(CardBrowserType::EnemyDeck));

    assert_eq!(s.user_client.cards.enemy_deck().len(), 0, "enemy deck should be empty");
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::Browser).len(),
        original_deck_count,
        "browser should have all deck cards"
    );
}

#[test]
fn close_browser_returns_deck_cards() {
    let mut s = TestBattle::builder().connect();

    let original_deck_count = s.user_client.cards.user_deck().len();

    s.perform_user_action(BattleDisplayAction::BrowseCards(CardBrowserType::UserDeck));

    assert_eq!(
        s.user_client.cards.user_deck().len(),
        0,
        "user deck should be empty after browsing"
    );
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::Browser).len(),
        original_deck_count,
        "browser should have all cards"
    );

    s.perform_user_action(BattleDisplayAction::CloseCardBrowser);

    assert_eq!(
        s.user_client.cards.user_deck().len(),
        original_deck_count,
        "user deck should have all cards after closing"
    );
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::Browser).len(),
        0,
        "browser should be empty after closing"
    );
}

#[test]
fn browse_multiple_sources_sequentially() {
    let mut s = TestBattle::builder().connect();

    s.perform_user_action(DebugBattleAction::AddCardToVoid(
        PlayerName::One,
        CardName::MinstrelOfFallingLight,
    ));

    s.perform_user_action(BattleDisplayAction::BrowseCards(CardBrowserType::UserVoid));

    assert_eq!(
        s.user_client.cards.user_void().len(),
        0,
        "user void should be empty after browsing"
    );
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::Browser).len(),
        1,
        "browser should have 1 card from void"
    );

    s.perform_user_action(BattleDisplayAction::CloseCardBrowser);

    assert_eq!(
        s.user_client.cards.user_void().len(),
        1,
        "user void should have 1 card after closing"
    );
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::Browser).len(),
        0,
        "browser should be empty after closing"
    );

    let original_deck_count = s.user_client.cards.user_deck().len();

    s.perform_user_action(BattleDisplayAction::BrowseCards(CardBrowserType::UserDeck));

    assert_eq!(
        s.user_client.cards.user_deck().len(),
        0,
        "user deck should be empty after browsing"
    );
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::Browser).len(),
        original_deck_count,
        "browser should have all deck cards"
    );

    s.perform_user_action(BattleDisplayAction::CloseCardBrowser);

    assert_eq!(
        s.user_client.cards.user_deck().len(),
        original_deck_count,
        "user deck should have all cards after closing"
    );
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::Browser).len(),
        0,
        "browser should be empty after closing"
    );
}
