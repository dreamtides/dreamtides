use core_data::identifiers::CardName;
use core_data::numerics::Energy;
use display_data::battle_view::DisplayPlayer;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn negate_unless_pays_cost() {
    let mut s = TestBattle::builder().connect();
    let negate_id = s.add_to_hand(DisplayPlayer::User, CardName::RippleOfDefiance);
    s.add_to_battlefield(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let initial_enemy_energy = s.user_client.opponent.energy();

    let event_id = s.create_and_play(DisplayPlayer::Enemy, CardName::Immolate);
    let event_cost = s.user_client.cards.get_cost(&event_id);
    s.play_card_from_hand(DisplayPlayer::User, &negate_id);
    s.click_primary_button(DisplayPlayer::Enemy, "Spend");
    assert!(
        s.user_client.cards.stack_cards().is_empty(),
        "stack should be empty after cards resolve"
    );
    assert_eq!(
        s.user_client.cards.user_battlefield().len(),
        0,
        "character should be dissolved by Immolate"
    );
    assert_eq!(
        s.user_client.opponent.energy(),
        initial_enemy_energy - Energy(2) - event_cost,
        "enemy should have spent 2 more energy"
    );
}

#[test]
fn negate_unless_pays_cost_decline() {
    let mut s = TestBattle::builder().connect();
    let negate_id = s.add_to_hand(DisplayPlayer::User, CardName::RippleOfDefiance);
    s.add_to_battlefield(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let initial_enemy_energy = s.user_client.opponent.energy();

    let event_id = s.create_and_play(DisplayPlayer::Enemy, CardName::Immolate);
    let event_cost = s.user_client.cards.get_cost(&event_id);
    s.play_card_from_hand(DisplayPlayer::User, &negate_id);
    s.click_secondary_button(DisplayPlayer::Enemy, "Decline");
    assert!(
        s.user_client.cards.stack_cards().is_empty(),
        "stack should be empty after cards resolve"
    );
    assert_eq!(
        s.user_client.cards.user_battlefield().len(),
        1,
        "character should not be dissolved by Immolate"
    );
    assert_eq!(
        s.user_client.opponent.energy(),
        initial_enemy_energy - event_cost,
        "enemy should have only spent the original event cost"
    );
}
