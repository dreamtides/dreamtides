use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use display_data::object_position::Position;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn return_void_card_to_hand_basic() {
    let mut s = TestBattle::builder().connect();
    let void_card = s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);

    let cards_in_browser = s.user_client.cards.browser_cards();

    assert!(cards_in_browser.len() >= 1, "Should have at least 1 card in browser");
    assert!(cards_in_browser.contains(&void_card), "Void card should be in browser");

    let void_card_view = s.user_client.cards.get(&void_card);
    assert!(
        void_card_view.view.revealed.as_ref().unwrap().outline_color.is_some(),
        "Void card should be selectable"
    );

    s.select_target(DisplayPlayer::User, &void_card);

    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert!(s.user_client.cards.user_hand().contains(&void_card), "Void card should be in hand");
    assert!(
        !s.user_client.cards.user_void().contains(&void_card),
        "Void card should not be in void"
    );
}

#[test]
fn return_void_card_to_hand_click_to_select() {
    let mut s = TestBattle::builder().connect();
    let void_card = s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);

    let void_card_view = s.user_client.cards.get(&void_card);
    assert!(
        void_card_view.view.revealed.as_ref().unwrap().actions.on_click.is_some(),
        "Void card should have click action"
    );

    s.select_target(DisplayPlayer::User, &void_card);

    let void_card_view_after = s.user_client.cards.get(&void_card);
    let outline_color = void_card_view_after.view.revealed.as_ref().unwrap().outline_color;
    assert!(outline_color.is_some(), "Selected void card should have outline");
}

#[test]
fn return_void_card_to_hand_click_twice_to_unselect() {
    let mut s = TestBattle::builder().connect();
    let void_card = s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);

    s.select_target(DisplayPlayer::User, &void_card);
    assert!(
        s.user_client.interface().primary_action_button.is_some(),
        "Submit button should be available after selection"
    );

    s.select_target(DisplayPlayer::User, &void_card);
    assert!(
        s.user_client.interface().primary_action_button.is_none(),
        "Submit button should not be available after unselection"
    );
}

#[test]
fn return_void_card_to_hand_select_different_card() {
    let mut s = TestBattle::builder().connect();
    let void_card_1 = s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);
    let void_card_2 = s.add_to_void(DisplayPlayer::User, CardName::TestDissolve);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);

    s.select_target(DisplayPlayer::User, &void_card_1);
    s.select_target(DisplayPlayer::User, &void_card_2);

    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert!(
        s.user_client.cards.user_hand().contains(&void_card_2),
        "Second selected card should be in hand"
    );
    assert!(
        !s.user_client.cards.user_void().contains(&void_card_2),
        "Second selected card should not be in void"
    );
    assert!(
        s.user_client.cards.user_void().contains(&void_card_1),
        "First card should still be in void"
    );
}

#[test]
fn return_void_card_to_hand_no_submit_without_selection() {
    let mut s = TestBattle::builder().connect();
    s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);

    assert!(
        s.user_client.interface().primary_action_button.is_none(),
        "Submit button should not be available without selection"
    );
}

#[test]
fn return_void_card_to_hand_with_multiple_void_cards() {
    let mut s = TestBattle::builder().connect();
    let void_card_1 = s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);
    let void_card_2 = s.add_to_void(DisplayPlayer::User, CardName::TestDissolve);
    let void_card_3 = s.add_to_void(DisplayPlayer::User, CardName::TestDrawOne);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);

    let cards_in_browser = s.user_client.cards.browser_cards();

    assert_eq!(cards_in_browser.len(), 3, "Should have 3 cards in browser");
    assert!(cards_in_browser.contains(&void_card_1), "First void card should be in browser");
    assert!(cards_in_browser.contains(&void_card_2), "Second void card should be in browser");
    assert!(cards_in_browser.contains(&void_card_3), "Third void card should be in browser");

    s.select_target(DisplayPlayer::User, &void_card_2);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert!(
        s.user_client.cards.user_hand().contains(&void_card_2),
        "Selected card should be in hand"
    );
    assert!(
        s.user_client.cards.user_void().contains(&void_card_1),
        "Unselected card should remain in void"
    );
    assert!(
        s.user_client.cards.user_void().contains(&void_card_3),
        "Unselected card should remain in void"
    );
}

#[test]
fn return_void_card_to_hand_with_empty_void() {
    let mut s = TestBattle::builder().connect();

    let hand_card = s.add_to_hand(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);

    assert!(
        !s.user_client
            .cards
            .get(&hand_card)
            .view
            .revealed
            .as_ref()
            .unwrap()
            .outline_color
            .is_some(),
        "Card should not be playable with empty void"
    );
    assert!(
        s.user_client
            .cards
            .get(&hand_card)
            .view
            .revealed
            .as_ref()
            .unwrap()
            .actions
            .can_play
            .is_none(),
        "Card should not have play action with empty void"
    );
}

#[test]
fn return_void_card_to_hand_browser_shows_correct_position() {
    let mut s = TestBattle::builder().connect();
    let void_card = s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);

    let card_in_browser = s.user_client.cards.get(&void_card);
    assert!(
        matches!(card_in_browser.view.position.position, Position::Browser),
        "Card should be positioned in browser"
    );
}

#[test]
fn return_void_card_to_hand_browser_closes_after_submit() {
    let mut s = TestBattle::builder().connect();
    let void_card = s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);

    s.select_target(DisplayPlayer::User, &void_card);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    let cards_in_browser = s.user_client.cards.browser_cards();

    assert_eq!(cards_in_browser.len(), 0, "No cards should be in browser position after submit");
}
