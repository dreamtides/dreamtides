use core_data::numerics::Energy;
use display_data::battle_view::DisplayPlayer;
use tabula_generated::test_card;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn gain_energy_increases_user_energy() {
    let mut s = TestBattle::builder().connect();
    let starting_energy = s.user_client.me.energy();
    let id = s.create_and_play(DisplayPlayer::User, test_card::TEST_GAIN_ENERGY);
    let cost = s.user_client.cards.get_cost(&id);
    assert_eq!(s.user_client.me.energy(), starting_energy - cost + Energy(1));
}

#[test]
fn gain_energy_accumulates_across_multiple_plays() {
    let mut s = TestBattle::builder().connect();
    let starting_energy = s.user_client.me.energy();
    let id1 = s.create_and_play(DisplayPlayer::User, test_card::TEST_GAIN_ENERGY);
    let cost1 = s.user_client.cards.get_cost(&id1);
    let id2 = s.create_and_play(DisplayPlayer::User, test_card::TEST_GAIN_ENERGY);
    let cost2 = s.user_client.cards.get_cost(&id2);
    assert_eq!(s.user_client.me.energy(), starting_energy - cost1 - cost2 + Energy(2));
}
