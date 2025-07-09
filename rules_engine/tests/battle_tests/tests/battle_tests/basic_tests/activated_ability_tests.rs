use core_data::identifiers::CardName;
use core_data::numerics::{Energy, Spark};
use display_data::battle_view::DisplayPlayer;
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session_prelude::*;

#[test]
fn activate_ability_basic_draw_card() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let character_id =
        s.add_to_battlefield(DisplayPlayer::User, CardName::TestActivatedAbilityDrawCardCharacter);

    assert_eq!(s.user_client.cards.user_hand().len(), 1, "activated ability token in hand");
    assert_eq!(s.user_client.me.energy(), Energy(99), "initial energy");

    let token_card_id = format!("A{character_id}/0");
    let hand_cards_before: Vec<String> =
        s.user_client.cards.user_hand().iter().map(|c| c.id.clone()).collect();
    assert_eq!(hand_cards_before, vec![token_card_id.clone()], "token card should be in hand");

    s.activate_ability(DisplayPlayer::User, &character_id, 0);

    let hand_cards_after: Vec<String> =
        s.user_client.cards.user_hand().iter().map(|c| c.id.clone()).collect();
    assert_eq!(s.user_client.cards.user_hand().len(), 1, "hand should still have one card");
    assert_ne!(hand_cards_after[0], token_card_id, "token card should be replaced by drawn card");
    assert_eq!(s.user_client.me.energy(), Energy(98), "energy spent on activation");
}

#[test]
fn activate_ability_multi_use_same_turn() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let character_id = s.add_to_battlefield(
        DisplayPlayer::User,
        CardName::TestMultiActivatedAbilityDrawCardCharacter,
    );

    assert_eq!(s.user_client.cards.user_hand().len(), 1, "activated ability token in hand");

    s.activate_ability(DisplayPlayer::User, &character_id, 0);
    assert_eq!(s.user_client.cards.user_hand().len(), 2, "drew first card, token regenerated");

    s.activate_ability(DisplayPlayer::User, &character_id, 0);
    assert_eq!(s.user_client.cards.user_hand().len(), 3, "drew second card");

    s.activate_ability(DisplayPlayer::User, &character_id, 0);
    assert_eq!(s.user_client.cards.user_hand().len(), 4, "drew third card");

    assert_eq!(s.user_client.me.energy(), Energy(96), "energy spent on three activations");
}

#[test]
fn activate_ability_single_use_per_turn_cycle() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let character_id =
        s.add_to_battlefield(DisplayPlayer::User, CardName::TestActivatedAbilityDrawCardCharacter);

    s.activate_ability(DisplayPlayer::User, &character_id, 0);
    assert_eq!(s.user_client.cards.user_hand().len(), 1, "token used, replaced by drawn card");

    let token_card_id = format!("A{character_id}/0");
    let token_card = s.user_client.cards.card_map.get(&token_card_id);
    assert!(
        token_card.is_none(),
        "activated ability token should not be available after single use"
    );
}

#[test]
fn activate_ability_insufficient_energy() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(0).build()).connect();

    let character_id =
        s.add_to_battlefield(DisplayPlayer::User, CardName::TestActivatedAbilityDrawCardCharacter);

    let token_card_id = format!("A{character_id}/0");
    let token_card = s.user_client.cards.card_map.get(&token_card_id);

    if let Some(card) = token_card {
        assert!(
            card.view.revealed.as_ref().unwrap().actions.can_play.is_none(),
            "activated ability should not be playable with insufficient energy"
        );
    } else {
        assert!(
            token_card.is_none(),
            "activated ability token should not appear with insufficient energy"
        );
    }
}

#[test]
fn activate_ability_fast_during_enemy_turn() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    let character_id = s.add_to_battlefield(
        DisplayPlayer::User,
        CardName::TestFastActivatedAbilityDrawCardCharacter,
    );

    assert_eq!(s.user_client.cards.user_hand().len(), 1, "activated ability token in hand");

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_character),
        "enemy character on stack"
    );
    assert!(s.user_client.me.can_act(), "user can act during enemy turn");

    s.activate_ability(DisplayPlayer::User, &character_id, 0);

    assert_eq!(s.user_client.cards.user_hand().len(), 1, "token used, replaced by drawn card");
    assert_eq!(s.user_client.me.energy(), Energy(98), "energy spent on activation");
}

#[test]
fn activate_ability_fast_multi_during_enemy_turn() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    let character_id = s.add_to_battlefield(
        DisplayPlayer::User,
        CardName::TestFastMultiActivatedAbilityDrawCardCharacter,
    );

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let _enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    s.activate_ability(DisplayPlayer::User, &character_id, 0);
    assert_eq!(s.user_client.cards.user_hand().len(), 2, "drew first card, token regenerated");

    s.activate_ability(DisplayPlayer::User, &character_id, 0);
    assert_eq!(s.user_client.cards.user_hand().len(), 3, "drew second card");

    assert_eq!(s.user_client.me.energy(), Energy(97), "energy spent on two activations");
}

#[test]
fn activate_ability_non_fast_not_available_during_enemy_turn() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    let character_id =
        s.add_to_battlefield(DisplayPlayer::User, CardName::TestActivatedAbilityDrawCardCharacter);

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let _enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    let token_card_id = format!("A{character_id}/0");
    let token_card = s.user_client.cards.card_map.get(&token_card_id);

    if let Some(card) = token_card {
        assert!(
            card.view.revealed.as_ref().unwrap().actions.can_play.is_none(),
            "non-fast activated ability should not be playable during enemy turn"
        );
    } else {
        assert!(
            token_card.is_none(),
            "non-fast activated ability token should not appear during enemy turn"
        );
    }
}

#[test]
fn activate_ability_token_card_properties() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let character_id =
        s.add_to_battlefield(DisplayPlayer::User, CardName::TestActivatedAbilityDrawCardCharacter);

    let token_card_id = format!("A{character_id}/0");
    let token_card = s.user_client.cards.card_map.get(&token_card_id);

    assert!(token_card.is_some(), "activated ability token should be present");

    let token_card = token_card.unwrap();
    let revealed = token_card.view.revealed.as_ref().unwrap();

    assert_eq!(revealed.cost, Some(Energy(1)), "activated ability should show cost");
    assert_eq!(revealed.card_type, "Activated Ability", "should show activated ability type");
    assert!(revealed.name.contains("Activated"), "ability name should contain character name");
    assert!(revealed.actions.can_play.is_some(), "activated ability should be playable");
}

#[test]
fn activate_ability_spark_unchanged() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let character_id =
        s.add_to_battlefield(DisplayPlayer::User, CardName::TestActivatedAbilityDrawCardCharacter);

    let initial_spark = s.user_client.cards.get_revealed(&character_id).spark;
    assert_eq!(initial_spark, Some(Spark(3)), "character has initial spark");

    s.activate_ability(DisplayPlayer::User, &character_id, 0);

    let final_spark = s.user_client.cards.get_revealed(&character_id).spark;
    assert_eq!(final_spark, Some(Spark(3)), "character spark unchanged after activation");
}
