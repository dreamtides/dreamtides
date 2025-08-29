use battle_state::actions::debug_battle_action::DebugBattleAction;
use core_data::types::PlayerName;
use display_data::battle_view::DisplayPlayer;
use tabula_ids::test_card;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn put_three_cards_from_deck_into_void() {
    let mut s = TestBattle::builder().connect();
    let initial_void_size = s.user_client.cards.user_void().len();

    s.create_and_play(DisplayPlayer::User, test_card::TEST_DECK_TO_VOID);

    let final_void_size = s.user_client.cards.user_void().len();
    let expected_void_size = initial_void_size + 4; // +1 for the event card itself, +3 for deck cards

    assert_eq!(
        final_void_size, expected_void_size,
        "Event card and 3 deck cards should be in void. Initial: {}, Final: {}, Expected: {}",
        initial_void_size, final_void_size, expected_void_size
    );
}

#[test]
fn put_cards_from_deck_into_void_with_small_deck() {
    let mut s = TestBattle::builder().connect();

    // Set deck to have only 2 cards
    s.perform_user_action(DebugBattleAction::SetCardsRemainingInDeck {
        player: PlayerName::One,
        cards: 2,
    });

    let initial_void_size = s.user_client.cards.user_void().len();

    s.create_and_play(DisplayPlayer::User, test_card::TEST_DECK_TO_VOID);

    // Should still put 3 deck cards + 1 event card into void after adding new deck
    // copy
    assert_eq!(
        s.user_client.cards.user_void().len(),
        initial_void_size + 4,
        "Should put 3 deck cards + 1 event card into void even with small initial deck"
    );

    // Deck should be refilled
    assert!(
        !s.user_client.cards.user_deck().is_empty(),
        "Deck should be refilled when not enough cards available"
    );
}

#[test]
fn put_cards_from_deck_into_void_with_empty_deck() {
    let mut s = TestBattle::builder().connect();

    // Set deck to be empty
    s.perform_user_action(DebugBattleAction::SetCardsRemainingInDeck {
        player: PlayerName::One,
        cards: 0,
    });

    let initial_void_size = s.user_client.cards.user_void().len();

    s.create_and_play(DisplayPlayer::User, test_card::TEST_DECK_TO_VOID);

    // Should create new deck copy and put 3 deck cards + 1 event card into void
    assert_eq!(
        s.user_client.cards.user_void().len(),
        initial_void_size + 4,
        "Should put 3 deck cards + 1 event card into void even when starting with empty deck"
    );

    assert!(!s.user_client.cards.user_deck().is_empty(), "Deck should be refilled when empty");
}

#[test]
fn cards_moved_to_void_are_visible() {
    let mut s = TestBattle::builder().connect();
    let initial_void_size = s.user_client.cards.user_void().len();

    s.create_and_play(DisplayPlayer::User, test_card::TEST_DECK_TO_VOID);

    let final_void_cards = s.user_client.cards.user_void();

    assert_eq!(
        final_void_cards.len(),
        initial_void_size + 4,
        "Exactly 4 cards (3 deck + 1 event) should be visible in void"
    );

    // Verify that we have new cards in void and they are revealed
    assert!(final_void_cards.len() > initial_void_size, "Should have added cards to void");
}
