use core_data::numerics::Points;
use display_data::battle_view::DisplayPlayer;
use display_data::command::GameMessageType;
use tabula_generated::test_card;
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session_prelude::*;

#[test]
fn gain_points_increases_user_score() {
    let mut s = TestBattle::builder().connect();
    let starting_points = s.user_client.me.score();
    s.create_and_play(DisplayPlayer::User, test_card::TEST_GAIN_POINTS);
    assert_eq!(s.user_client.me.score(), starting_points + Points(2));
}

#[test]
fn gain_points_accumulates_across_multiple_plays() {
    let mut s = TestBattle::builder().connect();
    let starting_points = s.user_client.me.score();
    s.create_and_play(DisplayPlayer::User, test_card::TEST_GAIN_POINTS);
    s.create_and_play(DisplayPlayer::User, test_card::TEST_GAIN_POINTS);
    assert_eq!(s.user_client.me.score(), starting_points + Points(4));
}

#[test]
fn gain_points_triggers_victory_on_threshold() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().points(24).build()).connect();
    s.create_and_play(DisplayPlayer::User, test_card::TEST_GAIN_POINTS);
    assert_eq!(s.user_client.me.score(), Points(26));
    assert_eq!(s.user_client.last_game_message, Some(GameMessageType::Victory));
}
