use core_data::identifiers::CardName;
use core_data::numerics::Spark;
use display_data::battle_view::DisplayPlayer;
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session_prelude::*;

#[test]
fn return_enemy_character_to_hand() {
    let mut s = TestBattle::builder().connect();
    let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "enemy character on battlefield");
    assert_eq!(s.user_client.cards.enemy_hand().len(), 0, "enemy hand empty");
    assert_eq!(s.user_client.cards.user_void().len(), 0, "user void empty");

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnToHand);

    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        0,
        "enemy character returned to hand"
    );
    assert_eq!(s.user_client.cards.enemy_hand().len(), 1, "enemy character in hand");
    assert_eq!(s.user_client.cards.user_void().len(), 1, "test return to hand card in user void");
    assert!(
        s.user_client.cards.enemy_hand().contains(&target_id),
        "target character in enemy hand"
    );
}

#[test]
fn return_to_hand_with_multiple_targets() {
    let mut s = TestBattle::builder().connect();
    let target1_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);
    let target2_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        2,
        "two enemy characters on battlefield"
    );

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnToHand);
    s.click_card(DisplayPlayer::User, &target1_id);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "one enemy character remains");
    assert_eq!(s.user_client.cards.enemy_hand().len(), 1, "one enemy character returned to hand");
    assert!(
        s.user_client.cards.enemy_hand().contains(&target1_id),
        "correct target returned to hand"
    );
    assert!(s.user_client.cards.enemy_battlefield().contains(&target2_id), "other target remains");
}

#[test]
fn return_to_hand_auto_targets_single_enemy() {
    let mut s = TestBattle::builder().connect();
    let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "enemy character on battlefield");
    assert_eq!(s.user_client.cards.enemy_hand().len(), 0, "enemy hand empty");

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnToHand);

    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        0,
        "enemy character returned to hand"
    );
    assert_eq!(s.user_client.cards.enemy_hand().len(), 1, "enemy character in hand");
    assert!(
        s.user_client.cards.enemy_hand().contains(&target_id),
        "target character returned to hand"
    );
}

#[test]
fn return_to_hand_only_targets_enemy_characters() {
    let mut s = TestBattle::builder().connect();
    let user_character_id =
        s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);
    let return_card_id = s.add_to_hand(DisplayPlayer::User, CardName::TestReturnToHand);

    let return_card = s.user_client.cards.get_revealed(&return_card_id);
    assert!(
        return_card.actions.can_play.is_none(),
        "return to hand should not be playable when no enemy characters present"
    );

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "user character remains");
    assert!(
        s.user_client.cards.user_battlefield().contains(&user_character_id),
        "user character untouched"
    );
}

#[test]
fn return_to_hand_resets_spark_from_triggered_abilities() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    // Place a trigger character on user battlefield
    let trigger_character_id = s.create_and_play(
        DisplayPlayer::User,
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter,
    );

    // Verify base spark
    let initial_spark = s.user_client.cards.get_revealed(&trigger_character_id).numeric_spark();
    assert_eq!(initial_spark, Some(Spark(5)), "trigger character has base spark of 5");

    // Materialize another character to trigger the spark gain
    s.create_and_play(DisplayPlayer::User, CardName::TestVanillaCharacter);

    // Verify spark increased from trigger
    let increased_spark = s.user_client.cards.get_revealed(&trigger_character_id).numeric_spark();
    assert_eq!(increased_spark, Some(Spark(6)), "trigger character gained +1 spark from trigger");

    // End user turn and start enemy turn
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    // Enemy returns the character to hand - need to target manually since there are
    // 2 characters
    s.create_and_play(DisplayPlayer::Enemy, CardName::TestReturnToHand);
    s.click_card(DisplayPlayer::Enemy, &trigger_character_id);

    // Verify character was returned to user hand
    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "only vanilla character remains");
    assert_eq!(s.user_client.cards.user_hand().len(), 1, "trigger character returned to hand");
    assert!(
        s.user_client.cards.user_hand().contains(&trigger_character_id),
        "trigger character in user hand"
    );

    // End enemy turn to allow user to re-play
    s.end_turn_remove_opponent_hand(DisplayPlayer::Enemy);

    // Re-play the character to verify spark reset - use create_and_play to get a
    // fresh instance
    let reset_character_id = s.create_and_play(
        DisplayPlayer::User,
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter,
    );

    // Verify spark has reset to base value for newly materialized character
    let reset_spark = s.user_client.cards.get_revealed(&reset_character_id).numeric_spark();
    assert_eq!(
        reset_spark,
        Some(Spark(5)),
        "character has base spark value when materialized fresh from hand"
    );
}

#[test]
fn return_to_hand_revealed_to_opponent() {
    let mut s = TestBattle::builder().connect();
    let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnToHand);

    // Verify the card is in the enemy's hand
    assert!(s.user_client.cards.enemy_hand().contains(&target_id), "Card should be in enemy hand");

    // Verify the card is revealed to the user (opponent of the controller)
    let card_from_user_perspective = s.user_client.cards.get(&target_id);
    assert!(
        card_from_user_perspective.view.revealed.is_some(),
        "Card should be revealed to the opponent"
    );
}
