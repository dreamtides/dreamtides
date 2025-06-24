use core_data::identifiers::CardName;
use core_data::numerics::Energy;
use display_data::battle_view::DisplayPlayer;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn draw_card_for_each_energy_spent() {
    let mut s = TestBattle::builder().connect();
    let starting_energy = s.user_client.me.energy();
    let id = s.create_and_play(DisplayPlayer::User, CardName::Dreamscatter);
    let cost = s.user_client.cards.get_cost(&id);
    s.click_increment_button(DisplayPlayer::User);
    s.click_increment_button(DisplayPlayer::User);
    s.click_primary_button(DisplayPlayer::User, "Spend");
    assert_eq!(
        s.user_client.me.energy(),
        starting_energy - cost - Energy(3),
        "user should have spent 3 energy"
    );
    assert_eq!(s.user_client.cards.user_hand().len(), 3, "user should have drawn 3 cards");
}
