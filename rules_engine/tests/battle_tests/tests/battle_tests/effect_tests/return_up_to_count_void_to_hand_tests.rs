use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn return_up_to_two_event_cards_basic() {
    let mut s = TestBattle::builder().connect();
    let event_card_1 = s.add_to_void(DisplayPlayer::User, CardName::TestDrawOne);
    let event_card_2 = s.add_to_void(DisplayPlayer::User, CardName::TestDissolve);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnOneOrTwoVoidEventCardsToHand);

    let cards_in_browser = s.user_client.cards.browser_cards();
    assert_eq!(cards_in_browser.len(), 2, "Should have 2 event cards in browser");
    assert!(cards_in_browser.contains(&event_card_1), "First event card should be in browser");
    assert!(cards_in_browser.contains(&event_card_2), "Second event card should be in browser");

    s.select_target(DisplayPlayer::User, &event_card_1);
    s.select_target(DisplayPlayer::User, &event_card_2);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert!(
        s.user_client.cards.user_hand().contains(&event_card_1),
        "First event card should be in hand"
    );
    assert!(
        s.user_client.cards.user_hand().contains(&event_card_2),
        "Second event card should be in hand"
    );
    assert!(
        !s.user_client.cards.user_void().contains(&event_card_1),
        "First event card should not be in void"
    );
    assert!(
        !s.user_client.cards.user_void().contains(&event_card_2),
        "Second event card should not be in void"
    );
}

#[test]
fn return_up_to_two_event_cards_select_only_one() {
    let mut s = TestBattle::builder().connect();
    let event_card_1 = s.add_to_void(DisplayPlayer::User, CardName::TestDrawOne);
    let event_card_2 = s.add_to_void(DisplayPlayer::User, CardName::TestDissolve);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnOneOrTwoVoidEventCardsToHand);

    s.select_target(DisplayPlayer::User, &event_card_1);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert!(
        s.user_client.cards.user_hand().contains(&event_card_1),
        "Selected event card should be in hand"
    );
    assert!(
        !s.user_client.cards.user_void().contains(&event_card_1),
        "Selected event card should not be in void"
    );
    assert!(
        s.user_client.cards.user_void().contains(&event_card_2),
        "Unselected event card should still be in void"
    );
}

#[test]
fn return_up_to_two_event_cards_with_empty_void() {
    let mut s = TestBattle::builder().connect();

    let hand_card =
        s.add_to_hand(DisplayPlayer::User, CardName::TestReturnOneOrTwoVoidEventCardsToHand);

    assert!(
        s.user_client.cards.get_revealed(&hand_card).actions.can_play.is_none(),
        "Card should not be playable with empty void"
    );
}

#[test]
fn return_up_to_two_event_cards_with_only_character_cards() {
    let mut s = TestBattle::builder().connect();
    s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);

    let hand_card =
        s.add_to_hand(DisplayPlayer::User, CardName::TestReturnOneOrTwoVoidEventCardsToHand);

    assert!(
        s.user_client.cards.get_revealed(&hand_card).actions.can_play.is_none(),
        "Card should not be playable with only character cards in void"
    );
}

#[test]
fn return_up_to_two_event_cards_with_three_event_cards() {
    let mut s = TestBattle::builder().connect();
    let event_card_1 = s.add_to_void(DisplayPlayer::User, CardName::TestDrawOne);
    let event_card_2 = s.add_to_void(DisplayPlayer::User, CardName::TestDissolve);
    let event_card_3 = s.add_to_void(DisplayPlayer::User, CardName::TestCounterspell);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnOneOrTwoVoidEventCardsToHand);

    let cards_in_browser = s.user_client.cards.browser_cards();
    assert_eq!(cards_in_browser.len(), 3, "Should have 3 event cards in browser");
    assert!(cards_in_browser.contains(&event_card_1), "First event card should be in browser");
    assert!(cards_in_browser.contains(&event_card_2), "Second event card should be in browser");
    assert!(cards_in_browser.contains(&event_card_3), "Third event card should be in browser");

    s.select_target(DisplayPlayer::User, &event_card_1);
    s.select_target(DisplayPlayer::User, &event_card_2);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert!(
        s.user_client.cards.user_hand().contains(&event_card_1),
        "First event card should be in hand"
    );
    assert!(
        s.user_client.cards.user_hand().contains(&event_card_2),
        "Second event card should be in hand"
    );
    assert!(
        s.user_client.cards.user_void().contains(&event_card_3),
        "Third event card should remain in void"
    );
}

#[test]
fn return_up_to_two_event_cards_can_deselect() {
    let mut s = TestBattle::builder().connect();
    let event_card_1 = s.add_to_void(DisplayPlayer::User, CardName::TestDrawOne);
    let event_card_2 = s.add_to_void(DisplayPlayer::User, CardName::TestDissolve);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnOneOrTwoVoidEventCardsToHand);

    s.select_target(DisplayPlayer::User, &event_card_1);
    s.select_target(DisplayPlayer::User, &event_card_2);
    s.select_target(DisplayPlayer::User, &event_card_1);

    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert!(
        !s.user_client.cards.user_hand().contains(&event_card_1),
        "First event card should not be in hand"
    );
    assert!(
        s.user_client.cards.user_hand().contains(&event_card_2),
        "Second event card should be in hand"
    );
    assert!(
        s.user_client.cards.user_void().contains(&event_card_1),
        "First event card should remain in void"
    );
}

#[test]
fn return_up_to_two_event_cards_browser_closes_after_submit() {
    let mut s = TestBattle::builder().connect();
    let event_card = s.add_to_void(DisplayPlayer::User, CardName::TestDrawOne);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnOneOrTwoVoidEventCardsToHand);

    s.select_target(DisplayPlayer::User, &event_card);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    let cards_in_browser = s.user_client.cards.browser_cards();
    assert_eq!(cards_in_browser.len(), 0, "No cards should be in browser position after submit");
}
