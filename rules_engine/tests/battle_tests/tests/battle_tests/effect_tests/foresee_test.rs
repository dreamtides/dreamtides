use battle_state::actions::battle_actions::CardOrderSelectionTargetDiscriminants;
use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use display_data::object_position::Position;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn foresee_creates_card_order_browser() {
    let mut s = TestBattle::builder().connect();

    s.create_and_play(DisplayPlayer::User, CardName::TestForeseeOne);

    let cards_in_selector = s.user_client.cards.cards_at_position(&Position::CardOrderSelector(
        CardOrderSelectionTargetDiscriminants::Deck,
    ));
    assert_eq!(
        cards_in_selector.len(),
        1,
        "Should have exactly one card in the card order selector"
    );

    assert!(
        s.user_client.interface.primary_action_button_contains("Submit"),
        "Should have Submit button"
    );
}

#[test]
fn foresee_interface_elements_work_correctly() {
    let mut s = TestBattle::builder().connect();

    s.create_and_play(DisplayPlayer::User, CardName::TestForeseeOne);

    assert!(
        s.user_client.interface.primary_action_button_contains("Submit"),
        "Should have Submit button"
    );

    assert!(
        s.user_client.interface.screen_overlay_contains("Select card order"),
        "Should show card order prompt"
    );
}

#[test]
fn foresee_card_sorting_keys_reflect_order() {
    let mut s = TestBattle::builder().connect();

    s.create_and_play(DisplayPlayer::User, CardName::TestForeseeOne);

    let cards_in_selector = s.user_client.cards.cards_at_position(&Position::CardOrderSelector(
        CardOrderSelectionTargetDiscriminants::Deck,
    ));
    assert_eq!(cards_in_selector.len(), 1, "Should have exactly one card");

    let card = cards_in_selector.get(0).expect("Should have a card");
    assert_eq!(
        card.view.position.sorting_key, 0,
        "First card in deck order should have sorting_key 0"
    );
}
