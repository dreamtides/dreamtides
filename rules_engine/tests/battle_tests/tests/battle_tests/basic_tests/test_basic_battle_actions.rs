use battle_state::actions::battle_actions::BattleAction;
use core_data::identifiers::CardName;
use core_data::numerics::{Energy, Points, Spark};
use display_data::battle_view::DisplayPlayer;
use display_data::command::GameMessageType;
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session_battle_extension::TestPlayCard;
use test_utils::session::test_session_prelude::*;

use super::test_helpers::assert_clients_identical;

#[test]
fn connect() {
    let s = TestBattle::builder().connect();
    assert_clients_identical(&s);
    assert_eq!(s.client.cards.user_hand().len(), 0, "user hand empty");
    assert_eq!(s.client.cards.enemy_hand().len(), 0, "enemy hand empty");
    assert_eq!(s.client.cards.user_void().len(), 0, "user void empty");
    assert_eq!(s.client.cards.enemy_void().len(), 0, "enemy void empty");
    assert_eq!(s.client.cards.user_battlefield().len(), 0, "user battlefield empty");
    assert_eq!(s.client.cards.enemy_battlefield().len(), 0, "enemy battlefield empty");
    assert_eq!(s.client.cards.stack_cards().len(), 0, "stack empty");
}

#[test]
fn end_turn() {
    let mut s = TestBattle::builder().connect();
    assert_eq!(s.client.last_game_message, None, "no initial message");
    s.perform_user_action(BattleAction::EndTurn);
    assert_eq!(s.client.last_game_message, Some(GameMessageType::EnemyTurn), "enemy turn message");
    assert_clients_identical(&s);
}

#[test]
fn turn_cycle() {
    let mut s = TestBattle::builder().connect();
    assert_eq!(s.client.last_game_message, None, "no initial message");
    assert_eq!(s.client.cards.user_hand().len(), 0, "user hand empty");
    assert_eq!(s.client.cards.enemy_hand().len(), 0, "enemy hand empty");
    s.perform_user_action(BattleAction::EndTurn);
    assert_eq!(s.client.last_game_message, Some(GameMessageType::EnemyTurn), "enemy turn message");
    assert_eq!(
        s.enemy_client.last_game_message,
        Some(GameMessageType::YourTurn),
        "your turn message"
    );
    s.perform_enemy_action(BattleAction::EndTurn);
    assert_eq!(s.client.last_game_message, Some(GameMessageType::YourTurn), "your turn message");
    assert_eq!(
        s.enemy_client.last_game_message,
        Some(GameMessageType::EnemyTurn),
        "enemy turn message"
    );
    assert_eq!(s.client.cards.user_hand().len(), 1, "user drew card");
    assert_eq!(s.client.cards.enemy_hand().len(), 1, "enemy drew card");
    assert_clients_identical(&s);
}

#[test]
fn play_character_increase_spark() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();
    assert_eq!(s.client.user.energy(), Energy(99), "initial energy");
    assert_eq!(s.client.user.total_spark(), Spark(0), "initial spark");
    assert_eq!(s.client.cards.user_battlefield().len(), 0, "battlefield empty");
    s.create_and_play(CardName::MinstrelOfFallingLight);
    assert_eq!(s.client.user.energy(), Energy(97), "energy spent");
    assert_eq!(s.client.user.total_spark(), Spark(5), "spark increased");
    assert_eq!(s.client.cards.user_battlefield().len(), 1, "character materialized");
    assert_clients_identical(&s);
}

#[test]
fn play_character_score_points() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();
    s.create_and_play(CardName::MinstrelOfFallingLight);
    s.perform_user_action(BattleAction::EndTurn);
    assert_eq!(s.client.user.score(), Points(0), "score unchanged");
    s.perform_enemy_action(BattleAction::EndTurn);
    assert_eq!(s.client.user.score(), Points(5), "score increased");
    assert_clients_identical(&s);
}

#[test]
fn play_character_win_battle() {
    let mut s =
        TestBattle::builder().user(TestPlayer::builder().energy(99).points(20).build()).connect();
    s.create_and_play(CardName::MinstrelOfFallingLight);
    s.perform_user_action(BattleAction::EndTurn);
    s.perform_enemy_action(BattleAction::EndTurn);
    assert_eq!(s.client.user.score(), Points(25), "score increased");
    assert_eq!(s.client.last_game_message, Some(GameMessageType::Victory), "victory message");
    assert_eq!(s.enemy_client.last_game_message, Some(GameMessageType::Defeat), "defeat message");
    assert_clients_identical(&s);
}

#[test]
fn energy_increment_at_turn_start() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(0).produced_energy(0).build())
        .connect();
    s.perform_user_action(BattleAction::EndTurn);
    s.perform_enemy_action(BattleAction::EndTurn);

    assert_eq!(s.client.user.energy(), Energy(1), "energy incremented by 1 more");
    assert_eq!(s.client.user.produced_energy(), Energy(1), "produced energy incremented by 1 more");
    assert_clients_identical(&s);
}

#[test]
fn create_and_play() {
    let mut s = TestBattle::builder().connect();
    s.add_to_battlefield(DisplayPlayer::Enemy, CardName::MinstrelOfFallingLight);
    s.create_and_play(TestPlayCard::builder().name(CardName::MinstrelOfFallingLight).build());
}

#[test]
fn play_card_dissolve_target() {
    let mut s = TestBattle::builder().connect();
    // Note that if a single target is present then no prompt for targeting is
    // shown.
    let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::MinstrelOfFallingLight);
    s.add_to_battlefield(DisplayPlayer::Enemy, CardName::MinstrelOfFallingLight);

    assert_eq!(s.client.cards.enemy_battlefield().len(), 2, "two characters on enemy battlefield");
    assert_eq!(s.client.cards.enemy_void().len(), 0, "enemy void empty");
    assert_eq!(s.client.cards.user_void().len(), 0, "user void empty");
    assert_eq!(s.client.user.energy(), Energy(99), "initial energy");

    s.create_and_play(TestPlayCard::builder().name(CardName::Immolate).target(target_id).build());

    assert_eq!(
        s.client.cards.enemy_battlefield().len(),
        1,
        "one character remaining on enemy battlefield"
    );
    assert_eq!(s.client.cards.enemy_void().len(), 1, "dissolve character in enemy void");
    assert_eq!(s.client.cards.user_void().len(), 1, "event in user void");

    assert_clients_identical(&s);
}
