use action_data::battle_display_action::{BattleDisplayAction, CardBrowserType};
use battle_state::actions::battle_actions::BattleAction;
use battle_state::actions::debug_battle_action::DebugBattleAction;
use core_data::types::PlayerName;
use display_data::battle_view::DisplayPlayer;
use display_data::object_position::Position;
use tabula_ids::test_card;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn browse_user_void_moves_cards_to_browser() {
    let mut s = TestBattle::builder().connect();

    s.perform_user_action(DebugBattleAction::AddCardToVoid {
        player: PlayerName::One,
        card: test_card::TEST_VANILLA_CHARACTER,
    });
    s.perform_user_action(DebugBattleAction::AddCardToVoid {
        player: PlayerName::One,
        card: test_card::TEST_DISSOLVE,
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
        card: test_card::TEST_VANILLA_CHARACTER,
    });
    s.perform_user_action(DebugBattleAction::AddCardToVoid {
        player: PlayerName::Two,
        card: test_card::TEST_DISSOLVE,
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
        card: test_card::TEST_VANILLA_CHARACTER,
    });
    s.perform_user_action(DebugBattleAction::AddCardToVoid {
        player: PlayerName::One,
        card: test_card::TEST_DISSOLVE,
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
        card: test_card::TEST_VANILLA_CHARACTER,
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
    s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    let enemy_character =
        s.create_and_play(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

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
    s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    let enemy_character =
        s.create_and_play(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

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
    s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let user_counterspell1 = s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL);
    let enemy_counterspell1 = s.add_to_hand(DisplayPlayer::Enemy, test_card::TEST_COUNTERSPELL);

    let enemy_character =
        s.create_and_play(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

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
    s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    let enemy_character =
        s.create_and_play(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

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

#[test]
fn toggle_stack_visibility_hides_and_shows_activated_ability_token() {
    let mut s = TestBattle::builder().connect();

    let character_id =
        s.add_to_battlefield(DisplayPlayer::User, test_card::TEST_ACTIVATED_ABILITY_DRAW_CARD);
    let _enemy_fast_card = s.add_to_hand(DisplayPlayer::Enemy, test_card::TEST_DRAW_ONE);

    s.activate_ability(DisplayPlayer::User, &character_id, 0);

    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "activated ability should be on stack");
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::OnScreenStorage).len(),
        0,
        "on screen storage empty before hiding"
    );

    s.perform_user_action(BattleDisplayAction::ToggleStackVisibility);

    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "stack hidden should appear empty");
    let storage_cards = s.user_client.cards.cards_at_position(&Position::OnScreenStorage);
    assert_eq!(storage_cards.len(), 1, "ability token moved to on screen storage");
    let stored = storage_cards.get(0).unwrap();
    assert!(
        stored.view.revealed.as_ref().unwrap().name.contains("Ability"),
        "stored card is ability token"
    );

    s.perform_user_action(BattleDisplayAction::ToggleStackVisibility);

    assert_eq!(
        s.user_client.cards.stack_cards().len(),
        1,
        "ability token back on stack after showing"
    );

    s.perform_enemy_action(BattleAction::PassPriority);
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "stack empty after resolution");
}
