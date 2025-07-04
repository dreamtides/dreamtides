use action_data::battle_display_action::{BattleDisplayAction, CardBrowserType};
use battle_state::actions::debug_battle_action::DebugBattleAction;
use core_data::identifiers::CardName;
use core_data::types::PlayerName;
use display_data::battle_view::DisplayPlayer;
use display_data::object_position::Position;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn browse_user_void_moves_cards_to_browser() {
    let mut s = TestBattle::builder().connect();

    s.perform_user_action(DebugBattleAction::AddCardToVoid {
        player: PlayerName::One,
        card: CardName::TestVanillaCharacter,
    });
    s.perform_user_action(DebugBattleAction::AddCardToVoid {
        player: PlayerName::One,
        card: CardName::TestDissolve,
    });

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

    s.perform_user_action(DebugBattleAction::AddCardToVoid {
        player: PlayerName::Two,
        card: CardName::TestVanillaCharacter,
    });
    s.perform_user_action(DebugBattleAction::AddCardToVoid {
        player: PlayerName::Two,
        card: CardName::TestDissolve,
    });

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

    s.perform_user_action(DebugBattleAction::AddCardToVoid {
        player: PlayerName::One,
        card: CardName::TestVanillaCharacter,
    });
    s.perform_user_action(DebugBattleAction::AddCardToVoid {
        player: PlayerName::One,
        card: CardName::TestDissolve,
    });

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

    assert!(!s.user_client.cards.user_deck().is_empty(), "user deck should have cards");
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

    assert!(!s.user_client.cards.enemy_deck().is_empty(), "enemy deck should have cards");
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

    s.perform_user_action(DebugBattleAction::AddCardToVoid {
        player: PlayerName::One,
        card: CardName::TestVanillaCharacter,
    });

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

#[test]
fn toggle_stack_visibility_hides_and_shows_cards() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspell);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    let enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_character),
        "enemy character should be on stack"
    );
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::OnScreenStorage).len(),
        0,
        "on screen storage should be empty initially"
    );

    s.perform_user_action(BattleDisplayAction::ToggleStackVisibility);

    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "stack should appear empty when hidden");
    assert!(
        s.user_client
            .cards
            .cards_at_position(&Position::OnScreenStorage)
            .contains(&enemy_character),
        "enemy character should be in on screen storage when stack is hidden"
    );

    s.perform_user_action(BattleDisplayAction::ToggleStackVisibility);

    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_character),
        "enemy character should be back on stack when visibility toggled again"
    );
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::OnScreenStorage).len(),
        0,
        "on screen storage should be empty after showing stack again"
    );
}

#[test]
fn toggle_stack_visibility_before_passing_priority() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspell);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    let enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    assert!(s.user_client.me.can_act(), "user should be able to act");
    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_character),
        "enemy character should be on stack"
    );

    s.perform_user_action(BattleDisplayAction::ToggleStackVisibility);

    assert!(s.user_client.me.can_act(), "user should still be able to act after hiding stack");
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "stack should appear empty when hidden");
    assert!(
        s.user_client
            .cards
            .cards_at_position(&Position::OnScreenStorage)
            .contains(&enemy_character),
        "enemy character should be in on screen storage"
    );

    s.perform_user_action(BattleDisplayAction::ToggleStackVisibility);

    assert!(s.user_client.me.can_act(), "user should still be able to act after showing stack");
    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_character),
        "enemy character should be back on stack"
    );
}

#[test]
fn toggle_stack_visibility_during_complex_targeting() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspell);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let user_counterspell1 = s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspell);
    let enemy_counterspell1 = s.add_to_hand(DisplayPlayer::Enemy, CardName::TestCounterspell);

    let enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_character),
        "enemy character should be on stack"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "one card on stack");
    assert!(s.user_client.me.can_act(), "user can act");

    s.play_card_from_hand(DisplayPlayer::User, &user_counterspell1);
    assert_eq!(s.user_client.cards.stack_cards().len(), 2, "two cards on stack");

    s.play_card_from_hand(DisplayPlayer::Enemy, &enemy_counterspell1);
    assert_eq!(s.user_client.cards.stack_cards().len(), 3, "three cards on stack");
    assert!(s.user_client.me.can_act(), "user can act again");

    s.perform_user_action(BattleDisplayAction::ToggleStackVisibility);

    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "stack should appear empty when hidden");
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::OnScreenStorage).len(),
        3,
        "all three cards should be in on screen storage"
    );
    assert!(s.user_client.me.can_act(), "user should still be able to act with stack hidden");

    s.perform_user_action(BattleDisplayAction::ToggleStackVisibility);

    assert_eq!(s.user_client.cards.stack_cards().len(), 3, "all cards should be back on stack");
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::OnScreenStorage).len(),
        0,
        "on screen storage should be empty after showing stack"
    );
    assert!(s.user_client.me.can_act(), "user should still be able to act with stack visible");
}

#[test]
fn other_game_actions_unhide_stack() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspell);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    let enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_character),
        "enemy character should be on stack"
    );
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::OnScreenStorage).len(),
        0,
        "on screen storage should be empty initially"
    );

    s.perform_user_action(BattleDisplayAction::ToggleStackVisibility);

    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "stack should appear empty when hidden");
    assert!(
        s.user_client
            .cards
            .cards_at_position(&Position::OnScreenStorage)
            .contains(&enemy_character),
        "enemy character should be in on screen storage when stack is hidden"
    );

    s.perform_user_action(BattleDisplayAction::BrowseCards(CardBrowserType::UserVoid));

    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_character),
        "enemy character should be back on stack after performing other action"
    );
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::OnScreenStorage).len(),
        0,
        "on screen storage should be empty after performing other action"
    );
}
