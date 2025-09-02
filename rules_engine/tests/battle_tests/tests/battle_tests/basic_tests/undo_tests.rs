use action_data::game_action_data::GameAction;
use battle_state::actions::battle_actions::BattleAction;
use core_data::numerics::{Energy, Points, Spark};
use core_data::types::PlayerName;
use display_data::battle_view::DisplayPlayer;
use tabula_ids::test_card;
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session_battle_extension::TestPlayCard;
use test_utils::session::test_session_prelude::*;

use crate::battle_tests::basic_tests::test_helpers;

#[test]
fn undo_play_character_card() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(10).build()).connect();

    assert_eq!(s.user_client.me.energy(), Energy(10), "initial energy");
    assert_eq!(s.user_client.me.total_spark(), Spark(0), "initial spark");
    assert_eq!(s.user_client.cards.user_battlefield().len(), 0, "battlefield empty");
    assert_eq!(s.user_client.cards.user_hand().len(), 0, "hand empty");

    let card_id = s.add_to_hand(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    s.play_card_from_hand(DisplayPlayer::User, &card_id);

    assert_eq!(s.user_client.me.energy(), Energy(8), "energy spent");
    assert_eq!(s.user_client.me.total_spark(), Spark(5), "spark increased");
    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "character materialized");
    assert_eq!(s.user_client.cards.user_hand().len(), 0, "card played to battlefield");

    s.perform_user_action(GameAction::Undo(PlayerName::One));

    assert_eq!(s.user_client.me.energy(), Energy(10), "energy restored");
    assert_eq!(s.user_client.me.total_spark(), Spark(0), "spark restored");
    assert_eq!(s.user_client.cards.user_battlefield().len(), 0, "battlefield empty");
    assert_eq!(s.user_client.cards.user_hand().len(), 1, "card remains in hand");
    test_helpers::assert_clients_identical(&s);
}

#[test]
fn undo_play_event_card() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(10).build()).connect();
    let target = s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);
    s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

    assert_eq!(s.user_client.me.energy(), Energy(10), "initial energy");
    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 2, "enemy has characters");
    assert_eq!(s.user_client.cards.enemy_void().len(), 0, "enemy void empty");
    assert_eq!(s.user_client.cards.user_void().len(), 0, "user void empty");

    s.create_and_play(
        DisplayPlayer::User,
        TestPlayCard::new(test_card::TEST_DISSOLVE).target(&target),
    );

    assert_eq!(s.user_client.me.energy(), Energy(8), "energy spent");
    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "character dissolved");
    assert_eq!(s.user_client.cards.enemy_void().len(), 1, "character in enemy void");
    assert_eq!(s.user_client.cards.user_void().len(), 1, "event in user void");

    s.perform_user_action(GameAction::Undo(PlayerName::One));

    assert!(
        s.user_client.cards.get_revealed(&target).actions.on_click.is_some(),
        "target card can be clicked again"
    );
    assert_eq!(s.user_client.me.energy(), Energy(8), "energy at state before last non-auto action");
    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 2, "characters restored");
    assert_eq!(s.user_client.cards.enemy_void().len(), 0, "enemy void empty");
    assert_eq!(s.user_client.cards.user_void().len(), 0, "user void empty");
    assert_eq!(s.user_client.cards.user_hand().len(), 0, "no card in hand at that previous state");
    test_helpers::assert_clients_identical(&s);
}

#[test]
fn undo_with_card_on_stack() {
    let mut s = TestBattle::builder().connect();
    let user_card = s.create_and_play(DisplayPlayer::User, test_card::TEST_VARIABLE_ENERGY_DRAW);
    s.add_to_hand(DisplayPlayer::Enemy, test_card::TEST_VARIABLE_ENERGY_DRAW);
    s.click_primary_button(DisplayPlayer::User, "Spend");
    assert!(s.user_client.cards.stack_cards().contains(&user_card), "user card on stack");
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "one card on stack");
    assert_eq!(s.user_client.cards.user_hand().len(), 0, "card not in hand");

    s.perform_user_action(GameAction::Undo(PlayerName::One));

    assert!(
        s.user_client.interface.primary_action_button_contains("Spend"),
        "spend button is present again"
    );

    s.perform_user_action(GameAction::Undo(PlayerName::One));

    assert!(
        !s.user_client.cards.stack_cards().contains(&user_card),
        "user card removed from stack"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "stack empty");
    assert_eq!(s.user_client.cards.user_hand().len(), 1, "user card back in hand");
    test_helpers::assert_clients_identical(&s);
}

#[test]
fn undo_does_not_include_display_actions() {
    let mut s = TestBattle::builder().connect();
    let user_card = s.create_and_play(DisplayPlayer::User, test_card::TEST_VARIABLE_ENERGY_DRAW);
    s.add_to_hand(DisplayPlayer::Enemy, test_card::TEST_VARIABLE_ENERGY_DRAW);
    s.click_increment_button(DisplayPlayer::User);
    s.click_increment_button(DisplayPlayer::User);
    s.click_primary_button(DisplayPlayer::User, "Spend");
    assert!(s.user_client.cards.stack_cards().contains(&user_card), "user card on stack");
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "one card on stack");
    assert_eq!(s.user_client.cards.user_hand().len(), 0, "card not in hand");

    s.perform_user_action(GameAction::Undo(PlayerName::One));

    assert!(
        s.user_client.interface.primary_action_button_contains("Spend"),
        "spend button is present again"
    );

    s.perform_user_action(GameAction::Undo(PlayerName::One));

    assert!(
        !s.user_client.cards.stack_cards().contains(&user_card),
        "user card removed from stack"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "stack empty");
    assert_eq!(s.user_client.cards.user_hand().len(), 1, "user card back in hand");
    test_helpers::assert_clients_identical(&s);
}

#[test]
fn undo_restores_points_and_spark() {
    let mut s =
        TestBattle::builder().user(TestPlayer::builder().energy(10).points(5).build()).connect();

    s.create_and_play(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    s.perform_user_action(BattleAction::EndTurn);
    s.perform_enemy_action(BattleAction::EndTurn);

    assert_eq!(s.user_client.me.score(), Points(10), "points increased");
    assert_eq!(s.user_client.me.total_spark(), Spark(5), "spark from character");

    s.perform_user_action(GameAction::Undo(PlayerName::One));

    assert_eq!(s.user_client.me.score(), Points(5), "points reverted to before end turn");
    assert_eq!(s.user_client.me.total_spark(), Spark(5), "spark from character still present");
    test_helpers::assert_clients_identical(&s);
}

#[test]
fn undo_multiple_steps() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(20).build()).connect();
    let c1 = s.add_to_hand(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    let c2 = s.add_to_hand(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    s.play_card_from_hand(DisplayPlayer::User, &c1);
    s.play_card_from_hand(DisplayPlayer::User, &c2);
    assert_eq!(s.user_client.cards.user_battlefield().len(), 2, "two characters");
    s.perform_user_action(GameAction::Undo(PlayerName::One));
    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "one after undo");
    s.perform_user_action(GameAction::Undo(PlayerName::One));
    assert_eq!(s.user_client.cards.user_battlefield().len(), 0, "zero after second undo");
    test_helpers::assert_clients_identical(&s);
}

#[test]
fn undo_skips_micro_selection_actions() {
    let mut s = TestBattle::builder().connect();
    let card = s.create_and_play(DisplayPlayer::User, test_card::TEST_VARIABLE_ENERGY_DRAW);
    s.click_increment_button(DisplayPlayer::User);
    assert!(s.user_client.cards.stack_cards().contains(&card), "on stack");
    s.perform_user_action(GameAction::Undo(PlayerName::One));
    assert!(!s.user_client.cards.stack_cards().contains(&card), "no longer on stack");
    test_helpers::assert_clients_identical(&s);
}

#[test]
fn undo_empty_stack_does_not_crash() {
    let mut s = TestBattle::builder().connect();
    s.perform_user_action(GameAction::Undo(PlayerName::One));
    test_helpers::assert_clients_identical(&s);
}
