use battle_state::actions::battle_actions::BattleAction;
use core_data::identifiers::CardName;
use core_data::numerics::{Energy, Points, Spark};
use display_data::battle_view::DisplayPlayer;
use display_data::card_view::CardPrefab;
use display_data::command::GameMessageType;
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session_battle_extension::TestPlayCard;
use test_utils::session::test_session_prelude::*;

use crate::battle_tests::basic_tests::test_helpers;

#[test]
fn connect() {
    let s = TestBattle::builder().connect();
    test_helpers::assert_clients_identical(&s);
    assert_eq!(s.user_client.cards.user_hand().len(), 0, "user hand empty");
    assert_eq!(s.user_client.cards.enemy_hand().len(), 0, "enemy hand empty");
    assert_eq!(s.user_client.cards.user_void().len(), 0, "user void empty");
    assert_eq!(s.user_client.cards.enemy_void().len(), 0, "enemy void empty");
    assert_eq!(s.user_client.cards.user_battlefield().len(), 0, "user battlefield empty");
    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 0, "enemy battlefield empty");
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "stack empty");
}

#[test]
fn end_turn() {
    let mut s = TestBattle::builder().connect();
    assert_eq!(s.user_client.last_game_message, None, "no initial message");
    s.perform_user_action(BattleAction::EndTurn);
    assert_eq!(
        s.user_client.last_game_message,
        Some(GameMessageType::EnemyTurn),
        "enemy turn message"
    );
    test_helpers::assert_clients_identical(&s);
}

#[test]
fn turn_cycle() {
    let mut s = TestBattle::builder().connect();
    assert_eq!(s.user_client.last_game_message, None, "no initial message");
    assert_eq!(s.user_client.cards.user_hand().len(), 0, "user hand empty");
    assert_eq!(s.user_client.cards.enemy_hand().len(), 0, "enemy hand empty");
    s.perform_user_action(BattleAction::EndTurn);
    assert_eq!(
        s.user_client.last_game_message,
        Some(GameMessageType::EnemyTurn),
        "enemy turn message"
    );
    assert_eq!(
        s.enemy_client.last_game_message,
        Some(GameMessageType::YourTurn),
        "your turn message"
    );
    s.perform_enemy_action(BattleAction::EndTurn);
    assert_eq!(
        s.user_client.last_game_message,
        Some(GameMessageType::YourTurn),
        "your turn message"
    );
    assert_eq!(
        s.enemy_client.last_game_message,
        Some(GameMessageType::EnemyTurn),
        "enemy turn message"
    );
    assert_eq!(s.user_client.cards.user_hand().len(), 1, "user drew card");
    assert_eq!(s.user_client.cards.enemy_hand().len(), 1, "enemy drew card");
    test_helpers::assert_clients_identical(&s);
}

#[test]
fn play_character_increase_spark() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();
    assert_eq!(s.user_client.me.energy(), Energy(99), "initial energy");
    assert_eq!(s.user_client.me.total_spark(), Spark(0), "initial spark");
    assert_eq!(s.user_client.cards.user_battlefield().len(), 0, "battlefield empty");
    s.create_and_play(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    assert_eq!(s.user_client.me.energy(), Energy(97), "energy spent");
    assert_eq!(s.user_client.me.total_spark(), Spark(5), "spark increased");
    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "character materialized");
    test_helpers::assert_clients_identical(&s);
}

#[test]
fn play_character_score_points() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();
    s.create_and_play(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    s.perform_user_action(BattleAction::EndTurn);
    assert_eq!(s.user_client.me.score(), Points(0), "score unchanged");
    s.perform_enemy_action(BattleAction::EndTurn);
    assert_eq!(s.user_client.me.score(), Points(5), "score increased");
    test_helpers::assert_clients_identical(&s);
}

#[test]
fn play_character_win_battle() {
    let mut s =
        TestBattle::builder().user(TestPlayer::builder().energy(99).points(20).build()).connect();
    s.create_and_play(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    s.perform_user_action(BattleAction::EndTurn);
    s.perform_enemy_action(BattleAction::EndTurn);
    assert_eq!(s.user_client.me.score(), Points(25), "score increased");
    assert_eq!(s.user_client.last_game_message, Some(GameMessageType::Victory), "victory message");
    assert_eq!(s.enemy_client.last_game_message, Some(GameMessageType::Defeat), "defeat message");
    test_helpers::assert_clients_identical(&s);
}

#[test]
fn energy_increment_at_turn_start() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(0).produced_energy(0).build())
        .connect();
    s.perform_user_action(BattleAction::EndTurn);
    s.perform_enemy_action(BattleAction::EndTurn);

    assert_eq!(s.user_client.me.energy(), Energy(1), "energy incremented by 1 more");
    assert_eq!(
        s.user_client.me.produced_energy(),
        Energy(1),
        "produced energy incremented by 1 more"
    );
    test_helpers::assert_clients_identical(&s);
}

#[test]
fn create_and_play() {
    let mut s = TestBattle::builder().connect();
    s.add_to_battlefield(DisplayPlayer::Enemy, CardName::MinstrelOfFallingLight);
    s.create_and_play(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
}

#[test]
fn play_card_dissolve_target() {
    let mut s = TestBattle::builder().connect();
    // Note that if a single target is present then no prompt for targeting is
    // shown.
    let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::MinstrelOfFallingLight);
    s.add_to_battlefield(DisplayPlayer::Enemy, CardName::MinstrelOfFallingLight);

    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        2,
        "two characters on enemy battlefield"
    );
    assert_eq!(s.user_client.cards.enemy_void().len(), 0, "enemy void empty");
    assert_eq!(s.user_client.cards.user_void().len(), 0, "user void empty");
    assert_eq!(s.user_client.me.energy(), Energy(99), "initial energy");

    s.create_and_play(
        DisplayPlayer::User,
        TestPlayCard::new(CardName::Immolate).target(&target_id),
    );

    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        1,
        "one character remaining on enemy battlefield"
    );
    assert_eq!(s.user_client.cards.enemy_void().len(), 1, "dissolve character in enemy void");
    assert_eq!(s.user_client.cards.user_void().len(), 1, "event in user void");

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn cards_in_hand_properties() {
    let mut s = TestBattle::builder().connect();

    let character_id = s.add_to_hand(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    let event_id = s.add_to_hand(DisplayPlayer::User, CardName::Immolate);

    assert_eq!(s.user_client.cards.user_hand().len(), 2, "user has 2 cards in hand");

    let character_card = s.user_client.cards.get(&character_id);
    let event_card = s.user_client.cards.get(&event_id);

    assert_eq!(s.user_client.cards.get_cost(&character_id), Energy(2), "minstrel character cost");
    assert_eq!(s.user_client.cards.get_cost(&event_id), Energy(2), "immolate event cost");

    let character_revealed = s.user_client.cards.get_revealed(&character_id);
    let event_revealed = s.user_client.cards.get_revealed(&event_id);

    assert_eq!(character_revealed.spark, Some(Spark(5)), "minstrel character spark");
    assert_eq!(character_revealed.name, "Minstrel of Falling Light", "character name");
    assert_eq!(character_revealed.card_type, "Musician", "character type");

    assert_eq!(event_revealed.spark, None, "event card should have no spark");
    assert_eq!(event_revealed.name, "Immolate", "event name");
    assert_eq!(event_revealed.card_type, "\u{f0e7} Event", "event type");

    assert_eq!(character_card.view.prefab, CardPrefab::Character, "character uses character frame");
    assert_eq!(
        event_card.view.prefab,
        CardPrefab::Character,
        "event currently uses character frame"
    );

    test_helpers::assert_clients_identical(&s);
}
