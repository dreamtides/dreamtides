use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use display_data::command::ArrowStyle;
use display_data::object_position::Position;
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session_prelude::*;

#[test]
fn return_void_card_to_hand_basic() {
    let mut s = TestBattle::builder().connect();
    let void_card = s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);

    let cards_in_browser = s.user_client.cards.browser_cards();

    assert!(!cards_in_browser.is_empty(), "Should have at least 1 card in browser");
    assert!(cards_in_browser.contains(&void_card), "Void card should be in browser");

    let void_card_view = s.user_client.cards.get(&void_card);
    assert!(
        void_card_view.view.revealed.as_ref().unwrap().outline_color.is_some(),
        "Void card should be selectable"
    );

    s.click_card(DisplayPlayer::User, &void_card);

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

    let cards_in_browser = s.user_client.cards.browser_cards();

    assert_eq!(cards_in_browser.len(), 3, "Should have 3 cards in browser");
    assert!(cards_in_browser.contains(&void_card_1), "First void card should be in browser");
    assert!(cards_in_browser.contains(&void_card_2), "Second void card should be in browser");
    assert!(cards_in_browser.contains(&void_card_3), "Third void card should be in browser");

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
        s.user_client.cards.get(&hand_card).view.revealed.as_ref().unwrap().outline_color.is_none(),
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

    s.click_card(DisplayPlayer::User, &void_card);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    let cards_in_browser = s.user_client.cards.browser_cards();

    assert_eq!(cards_in_browser.len(), 0, "No cards should be in browser position after submit");
}

#[test]
fn void_card_shows_above_void_position_when_targeted() {
    let mut s = TestBattle::builder().enemy(TestPlayer::builder().energy(99).build()).connect();
    let void_card = s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);

    // Give the enemy a fast card so they can respond and keep the effect on the
    // stack
    s.add_to_hand(DisplayPlayer::Enemy, CardName::TestDrawOne);

    // Play the return void card effect, which puts it on the stack
    let return_card = s.add_to_hand(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);
    s.play_card_from_hand(DisplayPlayer::User, &return_card);

    // During target selection, void cards appear in browser
    let cards_in_browser = s.user_client.cards.browser_cards();
    assert!(
        cards_in_browser.contains(&void_card),
        "Void card should be in browser during selection"
    );

    // Select the void card as a target and submit
    s.click_card(DisplayPlayer::User, &void_card);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    // Now the effect is on the stack with targets set, and the enemy can respond
    // The void card should be in AboveVoid position while being targeted
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::AboveVoid(DisplayPlayer::User)).len(),
        1,
        "Void card should be in AboveVoid position while targeted on stack"
    );

    let above_void_cards =
        s.user_client.cards.cards_at_position(&Position::AboveVoid(DisplayPlayer::User));
    assert!(
        above_void_cards.contains(&void_card),
        "The correct void card should be in AboveVoid position"
    );

    // Verify there's a card on the stack with targets
    assert!(
        s.user_client.cards.stack_cards().len() > 0,
        "There should be a card on the stack with targets set"
    );

    // Verify the void card is no longer in normal void position
    assert_eq!(
        s.user_client.cards.cards_at_position(&Position::InVoid(DisplayPlayer::User)).len(),
        0,
        "No cards should be in normal void position while targeted"
    );
}

#[test]
fn void_card_targeting_creates_green_arrows() {
    let mut s = TestBattle::builder().enemy(TestPlayer::builder().energy(99).build()).connect();
    let void_card = s.add_to_void(DisplayPlayer::User, CardName::TestVanillaCharacter);

    // Give the enemy a fast card so they can respond and keep the effect on the
    // stack
    s.add_to_hand(DisplayPlayer::Enemy, CardName::TestDrawOne);

    // Play the return void card effect, which puts it on the stack
    let return_card = s.add_to_hand(DisplayPlayer::User, CardName::TestReturnVoidCardToHand);
    s.play_card_from_hand(DisplayPlayer::User, &return_card);

    // Select the void card as a target and submit
    s.click_card(DisplayPlayer::User, &void_card);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    // Check that there are arrows displayed
    assert!(!s.user_client.arrows.is_empty(), "There should be arrows displayed");

    // Verify that there's at least one green arrow (pointing to void card)
    let green_arrows = s
        .user_client
        .arrows
        .iter()
        .filter(|arrow| matches!(arrow.color, ArrowStyle::Green))
        .count();

    assert!(green_arrows > 0, "There should be at least one green arrow pointing to the void card");
}
