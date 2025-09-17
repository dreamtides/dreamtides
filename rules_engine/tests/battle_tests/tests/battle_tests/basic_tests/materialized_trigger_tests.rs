use display_data::battle_view::DisplayPlayer;
use tabula_ids::test_card;
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session_prelude::*;

use crate::battle_tests::basic_tests::test_helpers;

#[test]
fn materialized_trigger_fires_when_character_enters_battlefield() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let initial_hand_size = s.user_client.cards.user_hand().len();

    let _materialized_character_id =
        s.create_and_play(DisplayPlayer::User, test_card::TEST_MATERIALIZED_DRAW);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one character on battlefield");
    assert_eq!(
        s.user_client.cards.user_hand().len(),
        initial_hand_size + 1,
        "should have drawn a card due to materialized trigger"
    );

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn materialized_trigger_multiple_characters() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let initial_hand_size = s.user_client.cards.user_hand().len();

    // Play first character with {materialized} trigger
    let _first_character_id =
        s.create_and_play(DisplayPlayer::User, test_card::TEST_MATERIALIZED_DRAW);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one character on battlefield");
    assert_eq!(
        s.user_client.cards.user_hand().len(),
        initial_hand_size + 1,
        "should have drawn a card from first character"
    );

    // Play second character with {materialized} trigger
    let _second_character_id =
        s.create_and_play(DisplayPlayer::User, test_card::TEST_MATERIALIZED_DRAW);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 2, "two characters on battlefield");
    assert_eq!(
        s.user_client.cards.user_hand().len(),
        initial_hand_size + 2,
        "should have drawn another card from second character"
    );

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn materialized_trigger_does_not_fire_for_other_characters() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let initial_hand_size = s.user_client.cards.user_hand().len();

    // Play a character with {materialized} trigger
    let _materialized_character_id =
        s.create_and_play(DisplayPlayer::User, test_card::TEST_MATERIALIZED_DRAW);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one character on battlefield");
    let hand_after_first = s.user_client.cards.user_hand().len();
    assert_eq!(
        hand_after_first,
        initial_hand_size + 1,
        "should have drawn a card from materialized trigger"
    );

    // Play a vanilla character - should NOT trigger the {materialized} ability
    // again
    let _vanilla_character_id =
        s.create_and_play(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 2, "two characters on battlefield");
    assert_eq!(
        s.user_client.cards.user_hand().len(),
        hand_after_first,
        "should NOT have drawn another card - vanilla character doesn't trigger materialized"
    );

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn materialized_trigger_does_not_fire_for_enemy_characters() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    let initial_hand_size = s.user_client.cards.user_hand().len();

    // Play user character with {materialized} trigger
    let _user_character_id =
        s.create_and_play(DisplayPlayer::User, test_card::TEST_MATERIALIZED_DRAW);

    assert_eq!(
        s.user_client.cards.user_battlefield().len(),
        1,
        "one user character on battlefield"
    );
    let hand_after_user = s.user_client.cards.user_hand().len();
    assert_eq!(
        hand_after_user,
        initial_hand_size + 1,
        "should have drawn a card from user's materialized trigger"
    );

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    // Enemy plays character with {materialized} trigger - should NOT affect user's
    // hand
    let _enemy_character_id =
        s.create_and_play(DisplayPlayer::Enemy, test_card::TEST_MATERIALIZED_DRAW);

    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        1,
        "one enemy character on battlefield"
    );
    assert_eq!(
        s.user_client.cards.user_hand().len(),
        hand_after_user,
        "user hand size should not change when enemy materializes character"
    );

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn materialized_trigger_works_with_other_triggered_abilities() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let initial_hand_size = s.user_client.cards.user_hand().len();

    // Play a character that triggers on materializing another character
    let trigger_character_id = s.create_and_play(
        DisplayPlayer::User,
        test_card::TEST_TRIGGER_GAIN_SPARK_WHEN_MATERIALIZE_ANOTHER_CHARACTER,
    );

    let initial_spark = s.user_client.cards.get_revealed(&trigger_character_id).numeric_spark();
    assert_eq!(
        initial_spark,
        Some(core_data::numerics::Spark(5)),
        "trigger character has base spark"
    );

    // Play a character with {materialized} trigger - should trigger both abilities
    let _materialized_character_id =
        s.create_and_play(DisplayPlayer::User, test_card::TEST_MATERIALIZED_DRAW);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 2, "two characters on battlefield");
    assert_eq!(
        s.user_client.cards.user_hand().len(),
        initial_hand_size + 1,
        "should have drawn a card from materialized trigger"
    );

    let final_spark = s.user_client.cards.get_revealed(&trigger_character_id).numeric_spark();
    assert_eq!(
        final_spark,
        Some(core_data::numerics::Spark(6)),
        "trigger character should have gained spark from materialization"
    );

    test_helpers::assert_clients_identical(&s);
}
