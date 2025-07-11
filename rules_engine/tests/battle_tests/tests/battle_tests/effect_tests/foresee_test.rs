use battle_state::actions::battle_actions::CardOrderSelectionTargetDiscriminants;
use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use display_data::object_position::Position;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn foresee_creates_card_order_browser() {
    let mut s = TestBattle::builder().connect();

    // Add the foresee card to hand and play it
    s.create_and_play(DisplayPlayer::User, CardName::TestForeseeOne);

    // The foresee effect should create a card order selector with the top card
    let cards_in_selector = s.user_client.cards.cards_at_position(&Position::CardOrderSelector(
        CardOrderSelectionTargetDiscriminants::Deck,
    ));
    assert_eq!(
        cards_in_selector.len(),
        1,
        "Should have exactly one card in the card order selector"
    );

    // Check that the submit button is available
    assert!(
        s.user_client.interface.primary_action_button_contains("Submit"),
        "Should have Submit button"
    );
}
