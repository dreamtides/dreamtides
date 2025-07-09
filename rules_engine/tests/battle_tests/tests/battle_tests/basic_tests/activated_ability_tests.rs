use battle_state::actions::battle_actions::BattleAction;
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

    // After activation, energy is spent immediately and ability resolves due to
    // auto-execution
    assert_eq!(s.user_client.me.energy(), Energy(98), "energy spent on activation");
    assert_eq!(
        s.user_client.cards.user_hand().len(),
        1,
        "activated ability token removed from hand and card drawn"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "stack empty after auto-resolution");
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

#[test]
fn activate_ability_goes_on_stack_requires_priority_passing() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    // Add character with activated ability to user's battlefield
    let character_id =
        s.add_to_battlefield(DisplayPlayer::User, CardName::TestActivatedAbilityDrawCardCharacter);

    // Give enemy a fast card so they have multiple legal actions when user
    // activates ability
    let _enemy_fast_card = s.add_to_hand(DisplayPlayer::Enemy, CardName::TestDrawOne);

    assert_eq!(s.user_client.cards.user_hand().len(), 1, "activated ability token in hand");
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "no cards on stack initially");

    // Activate the ability
    s.activate_ability(DisplayPlayer::User, &character_id, 0);

    // Verify the ability is now on the stack (doesn't auto-resolve because enemy
    // has other actions)
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "activated ability should be on stack");

    let stack_cards: Vec<&_> = s.user_client.cards.stack_cards().iter().map(|c| &c.view).collect();
    assert_eq!(stack_cards.len(), 1, "one item on stack");

    let stack_ability = &stack_cards[0];
    assert!(
        stack_ability.revealed.as_ref().unwrap().name.contains("Ability"),
        "stack item should be the activated ability"
    );

    // Verify opponent has priority (can act)
    assert!(!s.user_client.me.can_act(), "user should not have priority after activating ability");

    // Verify the ability hasn't resolved yet (hand count unchanged)
    assert_eq!(
        s.user_client.cards.user_hand().len(),
        0,
        "activated ability token should be gone from hand"
    );

    // Enemy passes priority to resolve the ability
    s.perform_enemy_action(BattleAction::PassPriority);

    // Now the ability should have resolved
    assert_eq!(
        s.user_client.cards.stack_cards().len(),
        0,
        "stack should be empty after resolution"
    );
    assert_eq!(s.user_client.cards.user_hand().len(), 1, "should have drawn a card from ability");
    assert_eq!(s.user_client.me.energy(), Energy(98), "energy should be spent");
}

#[test]
fn activate_ability_can_be_responded_to_with_fast_cards() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    // Add character with activated ability to user's battlefield
    let user_character_id =
        s.add_to_battlefield(DisplayPlayer::User, CardName::TestActivatedAbilityDrawCardCharacter);

    // Add fast activated ability character to enemy's battlefield
    let enemy_character_id = s.add_to_battlefield(
        DisplayPlayer::Enemy,
        CardName::TestFastActivatedAbilityDrawCardCharacter,
    );

    // Give both players additional cards so they have multiple legal actions
    let _user_fast_card = s.add_to_hand(DisplayPlayer::User, CardName::TestDrawOne);
    let _enemy_fast_card = s.add_to_hand(DisplayPlayer::Enemy, CardName::TestDrawOne);

    // User activates their ability
    s.activate_ability(DisplayPlayer::User, &user_character_id, 0);

    // Verify user ability is on stack
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "user ability on stack");

    // Enemy can respond with their fast ability since they have priority
    assert!(!s.user_client.me.can_act(), "user doesn't have priority");

    // Switch perspective to enemy to activate their fast ability
    s.activate_ability(DisplayPlayer::Enemy, &enemy_character_id, 0);

    // Now both abilities should be on stack
    assert_eq!(s.user_client.cards.stack_cards().len(), 2, "both abilities on stack");

    // Enemy ability should resolve first (top of stack)
    s.perform_user_action(BattleAction::PassPriority);
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "enemy ability resolved");

    // Then user ability resolves
    s.perform_enemy_action(BattleAction::PassPriority);
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "both abilities resolved");

    // Both players should have drawn cards
    assert_eq!(s.user_client.cards.user_hand().len(), 2, "user drew from their ability");
}

#[test]
fn activate_ability_fast_can_respond_during_enemy_turn() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    // Add fast activated ability character to user's battlefield
    let user_character_id = s.add_to_battlefield(
        DisplayPlayer::User,
        CardName::TestFastActivatedAbilityDrawCardCharacter,
    );

    // End user turn
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    // Give both players an additional fast card for multiple legal actions
    let _user_fast_card = s.add_to_hand(DisplayPlayer::User, CardName::TestDrawOne);
    let _enemy_fast_card = s.add_to_hand(DisplayPlayer::Enemy, CardName::TestDrawOne);

    // Enemy plays a card
    let enemy_card = s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    // Verify enemy card is on stack and user can act (has priority to respond)
    assert!(s.user_client.cards.stack_cards().contains(&enemy_card), "enemy card on stack");
    assert!(s.user_client.me.can_act(), "user can respond during enemy turn");

    // User responds with fast activated ability
    s.activate_ability(DisplayPlayer::User, &user_character_id, 0);

    // Both items should be on stack
    assert_eq!(s.user_client.cards.stack_cards().len(), 2, "both card and ability on stack");

    // User ability resolves first (top of stack)
    s.perform_enemy_action(BattleAction::PassPriority);
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "user ability resolved");
    assert_eq!(s.user_client.cards.user_hand().len(), 2, "user drew card and has counterspell");

    // Enemy card resolves second
    s.perform_user_action(BattleAction::PassPriority);
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "enemy card resolved");
}
