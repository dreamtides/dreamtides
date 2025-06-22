use battle_state::actions::battle_actions::BattleAction;
use core_data::identifiers::CardName;
use core_data::numerics::{Energy, Points, Spark};
use display_data::command::GameMessageType;
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session::TestSession;
use test_utils::session::test_session_prelude::*;

#[test]
fn test_connect() {
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
fn test_end_turn() {
    let mut s = TestBattle::builder().connect();
    assert_eq!(s.client.last_game_message, None, "no initial message");
    s.perform_action(BattleAction::EndTurn);
    assert_eq!(s.client.last_game_message, Some(GameMessageType::EnemyTurn), "enemy turn message");
    assert_clients_identical(&s);
}

#[test]
fn test_turn_cycle() {
    let mut s = TestBattle::builder().connect();
    assert_eq!(s.client.last_game_message, None, "no initial message");
    assert_eq!(s.client.cards.user_hand().len(), 0, "user hand empty");
    assert_eq!(s.client.cards.enemy_hand().len(), 0, "enemy hand empty");
    s.perform_action(BattleAction::EndTurn);
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
fn test_play_character_increase_spark() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();
    assert_eq!(s.client.user.energy(), Some(Energy(99)), "initial energy");
    assert_eq!(s.client.user.total_spark(), Some(Spark(0)), "initial spark");
    assert_eq!(s.client.cards.user_battlefield().len(), 0, "battlefield empty");
    s.create_and_play(CardName::MinstrelOfFallingLight);
    assert_eq!(s.client.user.energy(), Some(Energy(97)), "energy spent");
    assert_eq!(s.client.user.total_spark(), Some(Spark(5)), "spark increased");
    assert_eq!(s.client.cards.user_battlefield().len(), 1, "character materialized");
    assert_clients_identical(&s);
}

#[test]
fn test_play_character_score_points() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();
    s.create_and_play(CardName::MinstrelOfFallingLight);
    s.perform_action(BattleAction::EndTurn);
    assert_eq!(s.client.user.score(), Some(Points(0)), "score unchanged");
    s.perform_enemy_action(BattleAction::EndTurn);
    assert_eq!(s.client.user.score(), Some(Points(5)), "score increased");
    assert_clients_identical(&s);
}

#[test]
fn test_play_character_win_battle() {
    let mut s =
        TestBattle::builder().user(TestPlayer::builder().energy(99).points(20).build()).connect();
    s.create_and_play(CardName::MinstrelOfFallingLight);
    s.perform_action(BattleAction::EndTurn);
    s.perform_enemy_action(BattleAction::EndTurn);
    assert_eq!(s.client.user.score(), Some(Points(25)), "score increased");
    assert_eq!(s.client.last_game_message, Some(GameMessageType::Victory), "victory message");
    assert_eq!(s.enemy_client.last_game_message, Some(GameMessageType::Defeat), "defeat message");
    assert_clients_identical(&s);
}

#[test]
fn test_energy_increment_at_turn_start() {
    let mut s = TestBattle::builder().connect();
    assert_eq!(s.client.user.energy(), Some(Energy(2)), "initial energy");
    assert_eq!(s.client.user.produced_energy(), Some(Energy(2)), "initial produced energy");

    s.perform_action(BattleAction::EndTurn);
    s.perform_enemy_action(BattleAction::EndTurn);

    assert_eq!(s.client.user.energy(), Some(Energy(3)), "energy incremented by 1 more");
    assert_eq!(
        s.client.user.produced_energy(),
        Some(Energy(3)),
        "produced energy incremented by 1 more"
    );
    assert_clients_identical(&s);
}

fn assert_clients_identical(s: &TestSession) {
    assert_eq!(
        s.client.cards.user_hand().len(),
        s.enemy_client.cards.enemy_hand().len(),
        "hand counts match"
    );
    assert_eq!(
        s.client.cards.enemy_hand().len(),
        s.enemy_client.cards.user_hand().len(),
        "enemy hand match"
    );
    assert_eq!(
        s.client.cards.user_battlefield().len(),
        s.enemy_client.cards.enemy_battlefield().len(),
        "battlefield counts match"
    );
    assert_eq!(
        s.client.cards.enemy_battlefield().len(),
        s.enemy_client.cards.user_battlefield().len(),
        "enemy battlefield match"
    );
    assert_eq!(
        s.client.cards.user_void().len(),
        s.enemy_client.cards.enemy_void().len(),
        "void counts match"
    );
    assert_eq!(
        s.client.cards.enemy_void().len(),
        s.enemy_client.cards.user_void().len(),
        "enemy void match"
    );
    assert_eq!(
        s.client.cards.stack_cards().len(),
        s.enemy_client.cards.stack_cards().len(),
        "stack counts match"
    );

    assert_eq!(s.client.user.energy(), s.enemy_client.enemy.energy(), "energy match");
    assert_eq!(s.client.enemy.energy(), s.enemy_client.user.energy(), "enemy energy match");
    assert_eq!(
        s.client.user.produced_energy(),
        s.enemy_client.enemy.produced_energy(),
        "produced energy match"
    );
    assert_eq!(
        s.client.enemy.produced_energy(),
        s.enemy_client.user.produced_energy(),
        "enemy produced match"
    );
    assert_eq!(s.client.user.total_spark(), s.enemy_client.enemy.total_spark(), "spark match");
    assert_eq!(
        s.client.enemy.total_spark(),
        s.enemy_client.user.total_spark(),
        "enemy spark match"
    );
    assert_eq!(s.client.user.score(), s.enemy_client.enemy.score(), "score match");
    assert_eq!(s.client.enemy.score(), s.enemy_client.user.score(), "enemy score match");

    assert_eq!(
        s.client
            .cards
            .user_battlefield()
            .iter()
            .filter_map(|c| c.view.revealed.as_ref().map(|r| &r.name))
            .collect::<Vec<_>>(),
        s.enemy_client
            .cards
            .enemy_battlefield()
            .iter()
            .filter_map(|c| c.view.revealed.as_ref().map(|r| &r.name))
            .collect::<Vec<_>>(),
        "battlefield names match"
    );

    assert_eq!(
        s.client
            .cards
            .enemy_battlefield()
            .iter()
            .filter_map(|c| c.view.revealed.as_ref().map(|r| &r.name))
            .collect::<Vec<_>>(),
        s.enemy_client
            .cards
            .user_battlefield()
            .iter()
            .filter_map(|c| c.view.revealed.as_ref().map(|r| &r.name))
            .collect::<Vec<_>>(),
        "enemy battlefield names"
    );
}
