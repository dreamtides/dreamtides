use core_data::identifiers::CardName;
use core_data::numerics::Energy;
use display_data::battle_view::DisplayPlayer;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn hand_size_limit_exceeded_gains_energy() {
    let mut s = TestBattle::builder().connect();

    let initial_energy = s.user_client.me.energy();

    for _ in 0..9 {
        s.add_to_hand(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    }

    assert_eq!(s.user_client.cards.user_hand().len(), 9, "user should have 9 cards in hand");

    let draw_id = s.add_to_hand(DisplayPlayer::User, CardName::Dreamscatter);
    let draw_cost = s.user_client.cards.get_cost(&draw_id);

    s.play_card_from_hand(DisplayPlayer::User, &draw_id);
    s.click_increment_button(DisplayPlayer::User);
    s.click_increment_button(DisplayPlayer::User);
    s.click_primary_button(DisplayPlayer::User, "Spend");

    assert_eq!(
        s.user_client.me.energy(),
        initial_energy - draw_cost - Energy(1),
        "User should have spent dreamscatter cost and 2 energy, then gained 1 energy from hand size limit"
    );
    assert_eq!(
        s.user_client.cards.user_hand().len(),
        10,
        "User should have drawn 1 card due to hand size limit"
    );
}

#[test]
fn character_limit_exceeded_abandons_character() {
    let mut s = TestBattle::builder().connect();
    let initial_void = s.user_client.cards.user_void().len();
    // Add 8  to the battlefield
    for _ in 0..8 {
        s.add_to_battlefield(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    }
    assert_eq!(
        s.user_client.cards.user_battlefield().len(),
        8,
        "User should have 8 characters on battlefield"
    );
    let char_id = s.add_to_hand(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    s.play_card_from_hand(DisplayPlayer::User, &char_id);

    assert_eq!(
        s.user_client.cards.user_battlefield().len(),
        8,
        "User should still have 8 characters on battlefield"
    );
    assert_eq!(
        s.user_client.cards.user_void().len(),
        initial_void + 1,
        "User void should have increased by 1"
    );
}
