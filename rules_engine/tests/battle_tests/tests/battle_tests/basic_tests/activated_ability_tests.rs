use core_data::identifiers::CardName;
use core_data::numerics::{Energy, Spark};
use display_data::battle_view::DisplayPlayer;
use display_data::card_view::{CardPrefab, CardView};
use display_data::command::Command;
use display_data::object_position::Position;
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

#[test]
fn activate_ability_enemy_perspective_token_card_on_stack() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    // Add a character with activated ability to enemy's battlefield
    let enemy_character_id =
        s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestActivatedAbilityDrawCardCharacter);

    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "no cards on stack initially");

    // Switch to enemy turn
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    // Enemy should now have the activated ability token available
    s.activate_ability(DisplayPlayer::Enemy, &enemy_character_id, 0);

    // Check that token cards appeared during the activation from user's perspective
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
        "Token cards should appear during activated ability resolution from user perspective"
    );

    for token_card in &token_cards {
        if let Some(create_pos) = &token_card.create_position {
            assert_eq!(
                create_pos.position,
                Position::HiddenWithinCard(enemy_character_id.clone()),
                "Token card create position should be hidden within the activating character"
            );
        }

        if let Some(destroy_pos) = &token_card.destroy_position {
            assert_eq!(
                destroy_pos.position,
                Position::HiddenWithinCard(enemy_character_id.clone()),
                "Token card destroy position should be hidden within the activating character"
            );
        }
    }

    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "no cards on stack after resolution");

    let final_token_cards = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| card.view.prefab == CardPrefab::Token)
        .count();

    assert_eq!(final_token_cards, 0, "No token cards should remain visible in final state");
}
