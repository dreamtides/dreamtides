use battle_state::actions::battle_actions::BattleAction;
use battle_state::actions::debug_battle_action::DebugBattleAction;
use core_data::types::PlayerName;
use display_data::battle_view::DisplayPlayer;
use tabula_generated::test_card;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn discard_one_card_from_hand() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    s.create_and_play(DisplayPlayer::User, test_card::TEST_DISCARD);

    assert_eq!(s.user_client.cards.user_hand().len(), 0);
    assert_eq!(s.user_client.cards.user_void().len(), 2);
}

#[test]
fn discard_when_multiple_cards_in_hand() {
    let mut s = TestBattle::builder().connect();
    let card1 = s.add_to_hand(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    s.add_to_hand(DisplayPlayer::User, test_card::TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER);
    s.create_and_play(DisplayPlayer::User, test_card::TEST_DISCARD);

    s.click_card(DisplayPlayer::User, &card1);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert_eq!(s.user_client.cards.user_hand().len(), 1);
    assert_eq!(s.user_client.cards.user_void().len(), 2);
}

#[test]
fn discard_when_no_cards_in_hand() {
    let mut s = TestBattle::builder().connect();

    s.perform_user_action(BattleAction::Debug(DebugBattleAction::MoveHandToDeck {
        player: PlayerName::One,
    }));

    s.create_and_play(DisplayPlayer::User, test_card::TEST_DISCARD);

    assert_eq!(s.user_client.cards.user_hand().len(), 0);
    assert_eq!(s.user_client.cards.user_void().len(), 1);
}

#[test]
fn discard_two_cards_basic() {
    let mut s = TestBattle::builder().connect();
    let card1 = s.add_to_hand(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    let card2 = s.add_to_hand(
        DisplayPlayer::User,
        test_card::TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER,
    );
    let card3 = s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL_CHARACTER);

    s.create_and_play(DisplayPlayer::User, test_card::TEST_DISCARD_TWO);

    let cards_in_hand = s.user_client.cards.user_hand();
    assert!(!cards_in_hand.is_empty(), "Should have cards in hand for selection");
    assert!(cards_in_hand.contains(&card1), "Card1 should be selectable");
    assert!(cards_in_hand.contains(&card2), "Card2 should be selectable");
    assert!(cards_in_hand.contains(&card3), "Card3 should be selectable");

    s.click_card(DisplayPlayer::User, &card1);
    s.click_card(DisplayPlayer::User, &card2);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert_eq!(s.user_client.cards.user_hand().len(), 1, "Should have 1 card remaining in hand");
    assert_eq!(
        s.user_client.cards.user_void().len(),
        3,
        "Should have 3 cards in void (2 discarded + 1 played)"
    );
    assert!(s.user_client.cards.user_hand().contains(&card3), "Card3 should remain in hand");
}

#[test]
fn discard_two_cards_click_to_select_and_unselect() {
    let mut s = TestBattle::builder().connect();
    let card1 = s.add_to_hand(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    let card2 = s.add_to_hand(
        DisplayPlayer::User,
        test_card::TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER,
    );
    s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL_CHARACTER);

    s.create_and_play(DisplayPlayer::User, test_card::TEST_DISCARD_TWO);

    s.click_card(DisplayPlayer::User, &card1);
    assert!(
        s.user_client.interface().primary_action_button.is_none(),
        "Submit button should not be available with only 1 of 2 cards selected"
    );

    s.click_card(DisplayPlayer::User, &card2);
    assert!(
        s.user_client.interface().primary_action_button.is_some(),
        "Submit button should be available with 2 cards selected"
    );

    s.click_card(DisplayPlayer::User, &card1);
    assert!(
        s.user_client.interface().primary_action_button.is_none(),
        "Submit button should not be available after unselecting one card"
    );
}

#[test]
fn discard_two_cards_select_different_combination() {
    let mut s = TestBattle::builder().connect();
    let card1 = s.add_to_hand(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    let card2 = s.add_to_hand(
        DisplayPlayer::User,
        test_card::TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER,
    );
    let card3 = s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL_CHARACTER);

    s.create_and_play(DisplayPlayer::User, test_card::TEST_DISCARD_TWO);

    s.click_card(DisplayPlayer::User, &card1);
    s.click_card(DisplayPlayer::User, &card2);
    s.click_card(DisplayPlayer::User, &card1);
    s.click_card(DisplayPlayer::User, &card3);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert_eq!(s.user_client.cards.user_hand().len(), 1, "Should have 1 card remaining in hand");
    assert_eq!(s.user_client.cards.user_void().len(), 3, "Should have 3 cards in void");
    assert!(s.user_client.cards.user_hand().contains(&card1), "Card1 should remain in hand");
    assert!(!s.user_client.cards.user_hand().contains(&card2), "Card2 should be discarded");
    assert!(!s.user_client.cards.user_hand().contains(&card3), "Card3 should be discarded");
}

#[test]
fn discard_two_cards_with_exactly_two_in_hand() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    s.add_to_hand(DisplayPlayer::User, test_card::TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER);

    s.create_and_play(DisplayPlayer::User, test_card::TEST_DISCARD_TWO);

    assert_eq!(s.user_client.cards.user_hand().len(), 0, "Should have 0 cards in hand");
    assert_eq!(
        s.user_client.cards.user_void().len(),
        3,
        "Should have 3 cards in void (2 discarded + 1 played)"
    );
}

#[test]
fn discard_two_cards_with_only_one_in_hand() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);

    s.create_and_play(DisplayPlayer::User, test_card::TEST_DISCARD_TWO);

    assert_eq!(s.user_client.cards.user_hand().len(), 0, "Should have 0 cards in hand");
    assert_eq!(
        s.user_client.cards.user_void().len(),
        2,
        "Should have 2 cards in void (1 discarded + 1 played)"
    );
}
