use battle_state::actions::battle_actions::BattleAction;
use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use display_data::object_position::Position;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn return_void_card_to_hand_basic() {
    let mut s = TestBattle::builder().connect();
    let void_card = s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);

    let initial_hand_size = s.user_client.cards.user_hand().len();
    let initial_void_size = s.user_client.cards.user_void().len();

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);

    assert!(!s.user_client.interface().browser.is_none(), "Browser should be displayed");

    let cards_in_browser: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| matches!(card.view.position.position, Position::InBrowser))
        .collect();

    assert!(cards_in_browser.len() >= 1, "Should have at least 1 card in browser");
    assert!(
        cards_in_browser.iter().any(|card| card.id == void_card),
        "Void card should be in browser"
    );

    let void_card_view = s.user_client.cards.get(&void_card);
    assert!(
        void_card_view.view.revealed.as_ref().unwrap().outline_color.is_some(),
        "Void card should be selectable"
    );

    s.click_card(DisplayPlayer::User, &void_card);

    assert!(
        s.user_client.interface().primary_action_button.is_some(),
        "Submit button should be available"
    );
    assert_eq!(s.user_client.interface().primary_action_button.as_ref().unwrap().label, "Submit");

    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert!(
        s.user_client.interface().browser.is_none(),
        "Browser should be closed after submission"
    );
    assert!(s.user_client.cards.user_hand().contains(&void_card), "Void card should be in hand");
    assert!(
        !s.user_client.cards.user_void().contains(&void_card),
        "Void card should not be in void"
    );
    assert_eq!(
        s.user_client.cards.user_hand().len(),
        initial_hand_size + 1,
        "Hand size should increase by 1"
    );
    assert_eq!(
        s.user_client.cards.user_void().len(),
        initial_void_size - 1,
        "Void size should decrease by 1"
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

    s.click_card(DisplayPlayer::User, &void_card);

    let void_card_view_after = s.user_client.cards.get(&void_card);
    let outline_color = void_card_view_after.view.revealed.as_ref().unwrap().outline_color;
    assert!(outline_color.is_some(), "Selected void card should have outline");
}

#[test]
fn return_void_card_to_hand_click_twice_to_unselect() {
    let mut s = TestBattle::builder().connect();
    let void_card = s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);

    s.click_card(DisplayPlayer::User, &void_card);
    assert!(
        s.user_client.interface().primary_action_button.is_some(),
        "Submit button should be available after selection"
    );

    s.click_card(DisplayPlayer::User, &void_card);
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

    s.click_card(DisplayPlayer::User, &void_card_1);
    s.click_card(DisplayPlayer::User, &void_card_2);

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

    let cards_in_browser: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| matches!(card.view.position.position, Position::InBrowser))
        .collect();

    assert_eq!(cards_in_browser.len(), 3, "Should have 3 cards in browser");
    assert!(
        cards_in_browser.iter().any(|card| card.id == void_card_1),
        "First void card should be in browser"
    );
    assert!(
        cards_in_browser.iter().any(|card| card.id == void_card_2),
        "Second void card should be in browser"
    );
    assert!(
        cards_in_browser.iter().any(|card| card.id == void_card_3),
        "Third void card should be in browser"
    );

    s.click_card(DisplayPlayer::User, &void_card_2);
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
fn return_void_card_to_hand_card_returns_to_void_when_played() {
    let mut s = TestBattle::builder().connect();
    let void_card = s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);
    s.click_card(DisplayPlayer::User, &void_card);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert!(
        s.user_client.cards.user_hand().contains(&void_card),
        "Card should be in hand after return"
    );

    let initial_void_size = s.user_client.cards.user_void().len();
    s.create_and_play(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);

    assert_eq!(
        s.user_client.cards.user_void().len(),
        initial_void_size + 1,
        "Return card should be in void after playing"
    );
}

#[test]
fn return_void_card_to_hand_browser_shows_correct_position() {
    let mut s = TestBattle::builder().connect();
    let void_card = s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);

    let card_in_browser = s.user_client.cards.get(&void_card);
    assert!(
        matches!(card_in_browser.view.position.position, Position::InBrowser),
        "Card should be positioned in browser"
    );
}

#[test]
fn return_void_card_to_hand_browser_closes_after_submit() {
    let mut s = TestBattle::builder().connect();
    let void_card = s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);

    assert!(s.user_client.interface().browser.is_some(), "Browser should be open");

    s.click_card(DisplayPlayer::User, &void_card);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert!(s.user_client.interface().browser.is_none(), "Browser should be closed after submit");

    let cards_in_browser: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| matches!(card.view.position.position, Position::InBrowser))
        .collect();

    assert_eq!(cards_in_browser.len(), 0, "No cards should be in browser position after submit");
}
