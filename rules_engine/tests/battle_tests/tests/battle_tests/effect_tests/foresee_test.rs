use battle_state::actions::battle_actions::{
    CardOrderSelectionTarget, CardOrderSelectionTargetDiscriminants,
};
use battle_state::actions::debug_battle_action::DebugBattleAction;
use core_data::identifiers::CardName;
use core_data::types::PlayerName;
use display_data::battle_view::DisplayPlayer;
use display_data::object_position::Position;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn foresee_one_keep_card_on_deck() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::User, CardName::TestDrawOne);
    s.add_to_hand(DisplayPlayer::User, CardName::TestVanillaCharacter);
    s.add_to_hand(DisplayPlayer::User, CardName::TestDissolve);

    s.create_and_play(DisplayPlayer::User, CardName::TestForeseeOne);

    assert!(
        s.user_client.interface().card_order_selector.is_some(),
        "Card order selector should be displayed"
    );

    let cards_in_browser: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| matches!(card.view.position.position, Position::CardOrderSelector(_)))
        .collect();

    assert_eq!(cards_in_browser.len(), 1, "Should have exactly 1 card in browser");

    let card_id = cards_in_browser[0].id.clone();

    s.select_card_order(DisplayPlayer::User, &card_id, CardOrderSelectionTarget::Deck(0));
    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert!(
        s.user_client.interface().card_order_selector.is_none(),
        "Card order selector should be hidden after submission"
    );

    s.create_and_play(DisplayPlayer::User, CardName::TestDrawOne);

    assert!(
        s.user_client.cards.user_hand().contains(&card_id),
        "The card we placed on deck should now be in hand after drawing"
    );
}

#[test]
fn foresee_one_put_card_in_void() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::User, CardName::TestDrawOne);
    s.add_to_hand(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestForeseeOne);

    let cards_in_browser: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| matches!(card.view.position.position, Position::CardOrderSelector(_)))
        .collect();

    assert_eq!(cards_in_browser.len(), 1, "Should have exactly 1 card in browser");

    let card_id = cards_in_browser[0].id.clone();

    s.select_card_order(DisplayPlayer::User, &card_id, CardOrderSelectionTarget::Void);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert!(
        s.user_client.cards.user_void().contains(&card_id),
        "The card we placed in void should now be in the void"
    );

    s.create_and_play(DisplayPlayer::User, CardName::TestDrawOne);

    assert!(
        !s.user_client.cards.user_hand().contains(&card_id),
        "The voided card should not be in hand after drawing"
    );
}

#[test]
fn foresee_two_keep_both_cards_same_order() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::User, CardName::TestDrawOne);
    s.add_to_hand(DisplayPlayer::User, CardName::TestVanillaCharacter);
    s.add_to_hand(DisplayPlayer::User, CardName::TestDissolve);

    s.create_and_play(DisplayPlayer::User, CardName::TestForeseeTwo);

    let cards_in_browser: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| matches!(card.view.position.position, Position::CardOrderSelector(_)))
        .collect();

    assert_eq!(cards_in_browser.len(), 2, "Should have exactly 2 cards in browser");

    let first_card_id = cards_in_browser[0].id.clone();
    let second_card_id = cards_in_browser[1].id.clone();

    s.select_card_order(DisplayPlayer::User, &first_card_id, CardOrderSelectionTarget::Deck(0));
    s.select_card_order(DisplayPlayer::User, &second_card_id, CardOrderSelectionTarget::Deck(1));
    s.click_primary_button(DisplayPlayer::User, "Submit");

    s.create_and_play(DisplayPlayer::User, CardName::TestDrawOne);

    assert!(
        !s.user_client.cards.user_hand().contains(&first_card_id),
        "The first card at position 0 should still be on deck, not in hand"
    );

    assert!(
        s.user_client.cards.user_hand().contains(&second_card_id),
        "The second card at position 1 should be in hand after drawing"
    );
}

#[test]
fn foresee_two_reverse_card_order() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::User, CardName::TestDrawOne);
    s.add_to_hand(DisplayPlayer::User, CardName::TestVanillaCharacter);
    s.add_to_hand(DisplayPlayer::User, CardName::TestDissolve);

    s.create_and_play(DisplayPlayer::User, CardName::TestForeseeTwo);

    let cards_in_browser: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| matches!(card.view.position.position, Position::CardOrderSelector(_)))
        .collect();

    assert_eq!(cards_in_browser.len(), 2, "Should have exactly 2 cards in browser");

    let first_card_id = cards_in_browser[0].id.clone();
    let second_card_id = cards_in_browser[1].id.clone();

    s.select_card_order(DisplayPlayer::User, &second_card_id, CardOrderSelectionTarget::Deck(0));
    s.select_card_order(DisplayPlayer::User, &first_card_id, CardOrderSelectionTarget::Deck(1));
    s.click_primary_button(DisplayPlayer::User, "Submit");

    s.create_and_play(DisplayPlayer::User, CardName::TestDrawOne);

    assert!(
        !s.user_client.cards.user_hand().contains(&second_card_id),
        "The second card at position 0 should still be on deck, not in hand"
    );

    assert!(
        s.user_client.cards.user_hand().contains(&first_card_id),
        "The first card at position 1 should be in hand after drawing"
    );
}

#[test]
fn foresee_two_put_both_cards_in_void() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::User, CardName::TestDrawOne);
    s.add_to_hand(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestForeseeTwo);

    let cards_in_browser: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| matches!(card.view.position.position, Position::CardOrderSelector(_)))
        .collect();

    assert_eq!(cards_in_browser.len(), 2, "Should have exactly 2 cards in browser");

    let first_card_id = cards_in_browser[0].id.clone();
    let second_card_id = cards_in_browser[1].id.clone();

    s.select_card_order(DisplayPlayer::User, &first_card_id, CardOrderSelectionTarget::Void);
    s.select_card_order(DisplayPlayer::User, &second_card_id, CardOrderSelectionTarget::Void);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert!(
        s.user_client.cards.user_void().contains(&first_card_id),
        "The first card we placed in void should be in the void"
    );

    assert!(
        s.user_client.cards.user_void().contains(&second_card_id),
        "The second card we placed in void should be in the void"
    );

    s.create_and_play(DisplayPlayer::User, CardName::TestDrawOne);

    assert!(
        !s.user_client.cards.user_hand().contains(&first_card_id),
        "The first voided card should not be in hand after drawing"
    );

    assert!(
        !s.user_client.cards.user_hand().contains(&second_card_id),
        "The second voided card should not be in hand after drawing"
    );
}

#[test]
fn foresee_two_mixed_one_deck_one_void() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::User, CardName::TestDrawOne);
    s.add_to_hand(DisplayPlayer::User, CardName::TestVanillaCharacter);
    s.add_to_hand(DisplayPlayer::User, CardName::TestDissolve);

    s.create_and_play(DisplayPlayer::User, CardName::TestForeseeTwo);

    let cards_in_browser: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| matches!(card.view.position.position, Position::CardOrderSelector(_)))
        .collect();

    assert_eq!(cards_in_browser.len(), 2, "Should have exactly 2 cards in browser");

    let first_card_id = cards_in_browser[0].id.clone();
    let second_card_id = cards_in_browser[1].id.clone();

    s.select_card_order(DisplayPlayer::User, &first_card_id, CardOrderSelectionTarget::Deck(0));
    s.select_card_order(DisplayPlayer::User, &second_card_id, CardOrderSelectionTarget::Void);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert!(
        s.user_client.cards.user_void().contains(&second_card_id),
        "The second card we placed in void should be in the void"
    );

    s.create_and_play(DisplayPlayer::User, CardName::TestDrawOne);

    assert!(
        s.user_client.cards.user_hand().contains(&first_card_id),
        "The first card we kept on deck should be in hand after drawing"
    );

    assert!(
        !s.user_client.cards.user_hand().contains(&second_card_id),
        "The voided card should not be in hand after drawing"
    );
}

#[test]
fn foresee_cards_appear_in_browser_position() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::User, CardName::TestDrawOne);

    s.create_and_play(DisplayPlayer::User, CardName::TestForeseeTwo);

    let cards_in_deck_selector: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| {
            matches!(
                card.view.position.position,
                Position::CardOrderSelector(CardOrderSelectionTargetDiscriminants::Deck)
            )
        })
        .collect();

    let cards_in_void_selector: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| {
            matches!(
                card.view.position.position,
                Position::CardOrderSelector(CardOrderSelectionTargetDiscriminants::Void)
            )
        })
        .collect();

    assert_eq!(cards_in_deck_selector.len(), 2, "Should have 2 cards in deck selector initially");

    assert_eq!(cards_in_void_selector.len(), 0, "Should have 0 cards in void selector initially");

    let first_card_id = cards_in_deck_selector[0].id.clone();

    s.select_card_order(DisplayPlayer::User, &first_card_id, CardOrderSelectionTarget::Void);

    let cards_in_deck_selector_after: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| {
            matches!(
                card.view.position.position,
                Position::CardOrderSelector(CardOrderSelectionTargetDiscriminants::Deck)
            )
        })
        .collect();

    let cards_in_void_selector_after: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| {
            matches!(
                card.view.position.position,
                Position::CardOrderSelector(CardOrderSelectionTargetDiscriminants::Void)
            )
        })
        .collect();

    assert_eq!(
        cards_in_deck_selector_after.len(),
        1,
        "Should have 1 card in deck selector after moving one to void"
    );

    assert_eq!(
        cards_in_void_selector_after.len(),
        1,
        "Should have 1 card in void selector after moving one to void"
    );

    assert!(
        cards_in_void_selector_after.iter().any(|card| card.id == first_card_id),
        "The moved card should now be in the void selector"
    );

    assert!(
        !cards_in_deck_selector_after.iter().any(|card| card.id == first_card_id),
        "The moved card should no longer be in the deck selector"
    );
}

#[test]
fn foresee_cards_have_can_select_order_action() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::User, CardName::TestDrawOne);

    s.create_and_play(DisplayPlayer::User, CardName::TestForeseeOne);

    let cards_in_browser: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| matches!(card.view.position.position, Position::CardOrderSelector(_)))
        .collect();

    assert_eq!(cards_in_browser.len(), 1, "Should have exactly 1 card in browser");

    let card_in_browser = &cards_in_browser[0];
    let revealed_card = card_in_browser.view.revealed.as_ref().expect("Card should be revealed");

    assert!(
        revealed_card.actions.can_select_order.is_some(),
        "Card in browser should have can_select_order action"
    );
}

#[test]
fn foresee_with_empty_deck() {
    let mut s = TestBattle::builder().connect();

    s.perform_user_action(DebugBattleAction::SetCardsRemainingInDeck {
        player: PlayerName::One,
        cards: 0,
    });

    assert_eq!(s.user_client.cards.user_deck().len(), 0, "User deck should be empty");

    s.create_and_play(DisplayPlayer::User, CardName::TestForeseeOne);

    assert!(
        s.user_client.interface().card_order_selector.is_some(),
        "Card order selector should be displayed even when deck is empty"
    );

    let cards_in_browser: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| matches!(card.view.position.position, Position::CardOrderSelector(_)))
        .collect();

    assert_eq!(
        cards_in_browser.len(),
        1,
        "Should have 1 card in browser even when deck starts empty"
    );

    let card_id = cards_in_browser[0].id.clone();

    s.select_card_order(DisplayPlayer::User, &card_id, CardOrderSelectionTarget::Void);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert!(
        s.user_client.interface().card_order_selector.is_none(),
        "Card order selector should be hidden after submission"
    );

    assert!(
        s.user_client.cards.user_void().contains(&card_id),
        "The card should be in the void after being placed there"
    );
}

#[test]
fn foresee_two_with_only_one_card_in_deck() {
    let mut s = TestBattle::builder().connect();

    s.perform_user_action(DebugBattleAction::SetCardsRemainingInDeck {
        player: PlayerName::One,
        cards: 1,
    });

    assert_eq!(s.user_client.cards.user_deck().len(), 1, "User deck should have 1 card");

    s.create_and_play(DisplayPlayer::User, CardName::TestForeseeTwo);

    let cards_in_browser: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| matches!(card.view.position.position, Position::CardOrderSelector(_)))
        .collect();

    assert_eq!(
        cards_in_browser.len(),
        2,
        "Should have exactly 2 cards in browser after adding cards to deck"
    );

    let first_card_id = cards_in_browser[0].id.clone();
    let second_card_id = cards_in_browser[1].id.clone();

    s.select_card_order(DisplayPlayer::User, &first_card_id, CardOrderSelectionTarget::Deck(0));
    s.select_card_order(DisplayPlayer::User, &second_card_id, CardOrderSelectionTarget::Deck(1));
    s.click_primary_button(DisplayPlayer::User, "Submit");

    s.create_and_play(DisplayPlayer::User, CardName::TestDrawOne);

    assert!(
        !s.user_client.cards.user_hand().contains(&first_card_id),
        "The first card at position 0 should still be on deck, not in hand"
    );

    assert!(
        s.user_client.cards.user_hand().contains(&second_card_id),
        "The second card at position 1 should be in hand after drawing"
    );
}
