use core_data::identifiers::CardName;
use core_data::numerics::Spark;
use display_data::battle_view::DisplayPlayer;
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session_prelude::*;

use crate::battle_tests::basic_tests::test_helpers;

#[test]
fn triggered_ability_gain_spark_when_materialize_another_character() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let trigger_character_id = s.create_and_play(
        DisplayPlayer::User,
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter,
    );

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one character on battlefield");

    let initial_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(initial_spark, Some(Spark(5)), "trigger character has base spark");

    s.create_and_play(DisplayPlayer::User, CardName::TestVanillaCharacter);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 2, "two characters on battlefield");

    let final_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(final_spark, Some(Spark(6)), "trigger character gained +1 spark");

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn triggered_ability_gain_spark_multiple_materializations() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let trigger_character_id = s.create_and_play(
        DisplayPlayer::User,
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter,
    );

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one character on battlefield");

    let initial_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(initial_spark, Some(Spark(5)), "trigger character has base spark");

    s.create_and_play(DisplayPlayer::User, CardName::TestVanillaCharacter);

    let first_trigger_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(
        first_trigger_spark,
        Some(Spark(6)),
        "trigger character gained +1 spark after first materialization"
    );

    s.create_and_play(DisplayPlayer::User, CardName::TestVanillaCharacter);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 3, "three characters on battlefield");

    let final_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(
        final_spark,
        Some(Spark(7)),
        "trigger character gained +1 spark again after second materialization"
    );

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn triggered_ability_does_not_trigger_on_self_materialization() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let trigger_character_id = s.create_and_play(
        DisplayPlayer::User,
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter,
    );

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one character on battlefield");

    let final_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(
        final_spark,
        Some(Spark(5)),
        "trigger character does not gain spark from its own materialization"
    );

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn triggered_ability_does_not_trigger_on_enemy_materialization() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    let trigger_character_id = s.create_and_play(
        DisplayPlayer::User,
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter,
    );

    assert_eq!(
        s.user_client.cards.user_battlefield().len(),
        1,
        "one user character on battlefield"
    );

    let initial_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(initial_spark, Some(Spark(5)), "trigger character has base spark");

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        1,
        "one enemy character on battlefield"
    );

    let final_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(
        final_spark,
        Some(Spark(5)),
        "trigger character does not gain spark from enemy materialization"
    );

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn triggered_ability_multiple_trigger_characters() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let first_trigger_character_id = s.create_and_play(
        DisplayPlayer::User,
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter,
    );

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one character on battlefield");

    let first_initial_spark = s.user_client.cards.get_revealed(&first_trigger_character_id).spark;
    assert_eq!(first_initial_spark, Some(Spark(5)), "first trigger character has base spark");

    let second_trigger_character_id = s.create_and_play(
        DisplayPlayer::User,
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter,
    );

    assert_eq!(s.user_client.cards.user_battlefield().len(), 2, "two characters on battlefield");

    let first_after_second = s.user_client.cards.get_revealed(&first_trigger_character_id).spark;
    let second_initial_spark = s.user_client.cards.get_revealed(&second_trigger_character_id).spark;
    assert_eq!(
        first_after_second,
        Some(Spark(6)),
        "first trigger character gained +1 spark when second was materialized"
    );
    assert_eq!(second_initial_spark, Some(Spark(5)), "second trigger character has base spark");

    s.create_and_play(DisplayPlayer::User, CardName::TestVanillaCharacter);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 3, "three characters on battlefield");

    let first_final_spark = s.user_client.cards.get_revealed(&first_trigger_character_id).spark;
    let second_final_spark = s.user_client.cards.get_revealed(&second_trigger_character_id).spark;
    assert_eq!(
        first_final_spark,
        Some(Spark(7)),
        "first trigger character gained another +1 spark when third character was materialized"
    );
    assert_eq!(
        second_final_spark,
        Some(Spark(6)),
        "second trigger character gained +1 spark when third character was materialized"
    );

    test_helpers::assert_clients_identical(&s);
}
