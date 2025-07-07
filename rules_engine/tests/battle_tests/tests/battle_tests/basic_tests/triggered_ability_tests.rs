use core_data::identifiers::CardName;
use core_data::numerics::Spark;
use display_data::battle_view::DisplayPlayer;
use display_data::card_view::{CardPrefab, CardView};
use display_data::command::{Command, GameObjectId};
use display_data::object_position::Position;
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

#[test]
fn triggered_ability_token_cards_appear_and_disappear_on_stack() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let trigger_character_id = s.create_and_play(
        DisplayPlayer::User,
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter,
    );

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one character on battlefield");
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "no cards on stack initially");

    let initial_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(initial_spark, Some(Spark(5)), "trigger character has base spark");

    s.create_and_play(DisplayPlayer::User, CardName::TestVanillaCharacter);

    let token_cards: Vec<&CardView> = s
        .find_all_commands(DisplayPlayer::User, |command| {
            if let Command::UpdateBattle(update_cmd) = command { Some(update_cmd) } else { None }
        })
        .iter()
        .flat_map(|update_cmd| &update_cmd.battle.cards)
        .filter(|card| card.prefab == CardPrefab::Token)
        .collect();

    assert!(
        !token_cards.is_empty(),
        "Token cards should appear during triggered ability resolution"
    );

    for token_card in &token_cards {
        if let Some(create_pos) = &token_card.create_position {
            assert_eq!(
                create_pos.position,
                Position::HiddenWithinCard(trigger_character_id.clone()),
                "Token card create position should be hidden within the triggering character"
            );
        }

        if let Some(destroy_pos) = &token_card.destroy_position {
            assert_eq!(
                destroy_pos.position,
                Position::HiddenWithinCard(trigger_character_id.clone()),
                "Token card destroy position should be hidden within the triggering character"
            );
        }
    }

    assert_eq!(s.user_client.cards.user_battlefield().len(), 2, "two characters on battlefield");
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "no cards on stack after resolution");

    let final_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(final_spark, Some(Spark(6)), "trigger character gained +1 spark");

    let final_token_cards = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| card.view.prefab == CardPrefab::Token)
        .count();

    assert_eq!(final_token_cards, 0, "No token cards should remain visible in final state");

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn triggered_ability_display_effect_command_applied_to_spark_gaining_character() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let trigger_character_id = s.create_and_play(
        DisplayPlayer::User,
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter,
    );

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one character on battlefield");

    let initial_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(initial_spark, Some(Spark(5)), "trigger character has base spark");

    s.create_and_play(DisplayPlayer::User, CardName::TestVanillaCharacter);

    let display_effect_command = s.find_command(DisplayPlayer::User, |command| {
        if let Command::DisplayEffect(display_effect) = command {
            if display_effect.target == GameObjectId::CardId(trigger_character_id.clone()) {
                Some(display_effect)
            } else {
                None
            }
        } else {
            None
        }
    });

    assert_eq!(
        display_effect_command.target,
        GameObjectId::CardId(trigger_character_id.clone()),
        "DisplayEffect command should target the character gaining spark"
    );

    assert_eq!(s.user_client.cards.user_battlefield().len(), 2, "two characters on battlefield");

    let final_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(final_spark, Some(Spark(6)), "trigger character gained +1 spark");

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn triggered_ability_gain_spark_on_play_card_enemy_turn() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    let trigger_character_id =
        s.create_and_play(DisplayPlayer::User, CardName::TestTriggerGainSparkOnPlayCardEnemyTurn);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one character on battlefield");

    let initial_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(initial_spark, Some(Spark(5)), "trigger character has base spark");

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let user_counterspell = s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspell);
    let enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_character),
        "enemy character on stack"
    );
    assert!(s.user_client.me.can_act(), "user can act during enemy turn");

    s.play_card_from_hand(DisplayPlayer::User, &user_counterspell);

    let final_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(final_spark, Some(Spark(7)), "trigger character gained +2 spark");

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn triggered_ability_gain_spark_multiple_cards_enemy_turn() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    let trigger_character_id =
        s.create_and_play(DisplayPlayer::User, CardName::TestTriggerGainSparkOnPlayCardEnemyTurn);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one character on battlefield");

    let initial_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(initial_spark, Some(Spark(5)), "trigger character has base spark");

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let user_draw1 = s.add_to_hand(DisplayPlayer::User, CardName::TestDrawOne);
    let user_draw2 = s.add_to_hand(DisplayPlayer::User, CardName::TestDrawOne);
    s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    s.play_card_from_hand(DisplayPlayer::User, &user_draw1);

    let first_trigger_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(
        first_trigger_spark,
        Some(Spark(7)),
        "trigger character gained +2 spark after first card"
    );

    s.play_card_from_hand(DisplayPlayer::User, &user_draw2);

    let final_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(
        final_spark,
        Some(Spark(9)),
        "trigger character gained +2 spark again after second card"
    );

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn triggered_ability_does_not_trigger_during_user_turn() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let trigger_character_id =
        s.create_and_play(DisplayPlayer::User, CardName::TestTriggerGainSparkOnPlayCardEnemyTurn);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one character on battlefield");

    let initial_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(initial_spark, Some(Spark(5)), "trigger character has base spark");

    s.create_and_play(DisplayPlayer::User, CardName::TestVanillaCharacter);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 2, "two characters on battlefield");

    let final_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(
        final_spark,
        Some(Spark(5)),
        "trigger character does not gain spark during user turn"
    );

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn triggered_ability_does_not_trigger_when_enemy_plays_cards() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    let trigger_character_id =
        s.create_and_play(DisplayPlayer::User, CardName::TestTriggerGainSparkOnPlayCardEnemyTurn);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one character on battlefield");

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
        "trigger character does not gain spark when enemy plays cards"
    );

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn triggered_ability_multiple_trigger_characters_enemy_turn() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    let first_trigger_character_id =
        s.create_and_play(DisplayPlayer::User, CardName::TestTriggerGainSparkOnPlayCardEnemyTurn);

    let second_trigger_character_id =
        s.create_and_play(DisplayPlayer::User, CardName::TestTriggerGainSparkOnPlayCardEnemyTurn);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 2, "two characters on battlefield");

    let first_initial_spark = s.user_client.cards.get_revealed(&first_trigger_character_id).spark;
    let second_initial_spark = s.user_client.cards.get_revealed(&second_trigger_character_id).spark;
    assert_eq!(first_initial_spark, Some(Spark(5)), "first trigger character has base spark");
    assert_eq!(second_initial_spark, Some(Spark(5)), "second trigger character has base spark");

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let user_counterspell = s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspell);
    let _enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    s.play_card_from_hand(DisplayPlayer::User, &user_counterspell);

    let first_final_spark = s.user_client.cards.get_revealed(&first_trigger_character_id).spark;
    let second_final_spark = s.user_client.cards.get_revealed(&second_trigger_character_id).spark;
    assert_eq!(first_final_spark, Some(Spark(7)), "first trigger character gained +2 spark");
    assert_eq!(second_final_spark, Some(Spark(7)), "second trigger character gained +2 spark");

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn triggered_ability_triggers_on_different_card_types_enemy_turn() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    let trigger_character_id =
        s.create_and_play(DisplayPlayer::User, CardName::TestTriggerGainSparkOnPlayCardEnemyTurn);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one character on battlefield");

    let initial_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(initial_spark, Some(Spark(5)), "trigger character has base spark");

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let user_fast_character =
        s.add_to_hand(DisplayPlayer::User, CardName::TestTriggerGainSparkOnPlayCardEnemyTurn);
    let user_draw = s.add_to_hand(DisplayPlayer::User, CardName::TestDrawOne);
    let _enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    s.play_card_from_hand(DisplayPlayer::User, &user_fast_character);

    let after_character_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(
        after_character_spark,
        Some(Spark(7)),
        "trigger character gained +2 spark after playing fast character"
    );

    s.play_card_from_hand(DisplayPlayer::User, &user_draw);

    let final_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(
        final_spark,
        Some(Spark(9)),
        "trigger character gained +2 spark again after playing event"
    );

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn triggered_ability_triggers_on_fast_character_enemy_turn() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    let trigger_character_id =
        s.create_and_play(DisplayPlayer::User, CardName::TestTriggerGainSparkOnPlayCardEnemyTurn);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one character on battlefield");

    let initial_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(initial_spark, Some(Spark(5)), "trigger character has base spark");

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let user_fast_character =
        s.add_to_hand(DisplayPlayer::User, CardName::TestTriggerGainSparkOnPlayCardEnemyTurn);
    let _enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    s.play_card_from_hand(DisplayPlayer::User, &user_fast_character);

    let final_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(
        final_spark,
        Some(Spark(7)),
        "trigger character gained +2 spark after playing fast character"
    );

    assert_eq!(s.user_client.cards.user_battlefield().len(), 2, "two characters on battlefield");

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn triggered_ability_enemy_turn_token_cards_and_display_effect() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    let trigger_character_id =
        s.create_and_play(DisplayPlayer::User, CardName::TestTriggerGainSparkOnPlayCardEnemyTurn);

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one character on battlefield");
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "no cards on stack initially");

    let initial_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(initial_spark, Some(Spark(5)), "trigger character has base spark");

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let user_counterspell = s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspell);
    let _enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    s.play_card_from_hand(DisplayPlayer::User, &user_counterspell);

    // Check for token cards appearing during triggered ability resolution
    let token_cards: Vec<&CardView> = s
        .find_all_commands(DisplayPlayer::User, |command| {
            if let Command::UpdateBattle(update_cmd) = command { Some(update_cmd) } else { None }
        })
        .iter()
        .flat_map(|update_cmd| &update_cmd.battle.cards)
        .filter(|card| card.prefab == CardPrefab::Token)
        .collect();

    assert!(
        !token_cards.is_empty(),
        "Token cards should appear during triggered ability resolution"
    );

    for token_card in &token_cards {
        if let Some(create_pos) = &token_card.create_position {
            assert_eq!(
                create_pos.position,
                Position::HiddenWithinCard(trigger_character_id.clone()),
                "Token card create position should be hidden within the triggering character"
            );
        }

        if let Some(destroy_pos) = &token_card.destroy_position {
            assert_eq!(
                destroy_pos.position,
                Position::HiddenWithinCard(trigger_character_id.clone()),
                "Token card destroy position should be hidden within the triggering character"
            );
        }
    }

    // Check for DisplayEffect command targeting the character gaining spark
    let display_effect_command = s.find_command(DisplayPlayer::User, |command| {
        if let Command::DisplayEffect(display_effect) = command {
            if display_effect.target == GameObjectId::CardId(trigger_character_id.clone()) {
                Some(display_effect)
            } else {
                None
            }
        } else {
            None
        }
    });

    assert_eq!(
        display_effect_command.target,
        GameObjectId::CardId(trigger_character_id.clone()),
        "DisplayEffect command should target the character gaining spark"
    );

    let final_spark = s.user_client.cards.get_revealed(&trigger_character_id).spark;
    assert_eq!(final_spark, Some(Spark(7)), "trigger character gained +2 spark");

    let final_token_cards = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| card.view.prefab == CardPrefab::Token)
        .count();

    assert_eq!(final_token_cards, 0, "No token cards should remain visible in final state");

    test_helpers::assert_clients_identical(&s);
}
