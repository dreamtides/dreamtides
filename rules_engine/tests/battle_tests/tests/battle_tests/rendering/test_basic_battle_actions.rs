use battle_state::actions::battle_actions::BattleAction;
use core_data::identifiers::CardName;
use core_data::numerics::{Energy, Points, Spark};
use display_data::battle_view::DisplayPlayer;
use display_data::command::GameMessageType;
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session::TestSession;
use test_utils::session::test_session_battle_extension::TestPlayCard;
use test_utils::session::test_session_prelude::*;

#[test]
fn test_connect() {
    let s = TestBattle::builder().connect();
    assert_clients_identical(&s);
    assert_eq!(s.user_client.cards.user_hand().len(), 0, "user hand empty");
    assert_eq!(s.user_client.cards.enemy_hand().len(), 0, "enemy hand empty");
    assert_eq!(s.user_client.cards.user_void().len(), 0, "user void empty");
    assert_eq!(s.user_client.cards.enemy_void().len(), 0, "enemy void empty");
    assert_eq!(s.user_client.cards.user_battlefield().len(), 0, "user battlefield empty");
    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 0, "enemy battlefield empty");
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "stack empty");
}

#[test]
fn test_end_turn() {
    let mut s = TestBattle::builder().connect();
    assert_eq!(s.user_client.last_game_message, None, "no initial message");
    s.perform_user_action(BattleAction::EndTurn);
    assert_eq!(
        s.user_client.last_game_message,
        Some(GameMessageType::EnemyTurn),
        "enemy turn message"
    );
    assert_clients_identical(&s);
}

#[test]
fn test_turn_cycle() {
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
    assert_clients_identical(&s);
}

#[test]
fn test_play_character_increase_spark() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();
    assert_eq!(s.user_client.user.energy(), Some(Energy(99)), "initial energy");
    assert_eq!(s.user_client.user.total_spark(), Some(Spark(0)), "initial spark");
    assert_eq!(s.user_client.cards.user_battlefield().len(), 0, "battlefield empty");
    s.create_and_play(CardName::MinstrelOfFallingLight);
    assert_eq!(s.user_client.user.energy(), Some(Energy(97)), "energy spent");
    assert_eq!(s.user_client.user.total_spark(), Some(Spark(5)), "spark increased");
    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "character materialized");
    assert_clients_identical(&s);
}

#[test]
fn test_play_character_score_points() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();
    s.create_and_play(CardName::MinstrelOfFallingLight);
    s.perform_user_action(BattleAction::EndTurn);
    assert_eq!(s.user_client.user.score(), Some(Points(0)), "score unchanged");
    s.perform_enemy_action(BattleAction::EndTurn);
    assert_eq!(s.user_client.user.score(), Some(Points(5)), "score increased");
    assert_clients_identical(&s);
}

#[test]
fn test_play_character_win_battle() {
    let mut s =
        TestBattle::builder().user(TestPlayer::builder().energy(99).points(20).build()).connect();
    s.create_and_play(CardName::MinstrelOfFallingLight);
    s.perform_user_action(BattleAction::EndTurn);
    s.perform_enemy_action(BattleAction::EndTurn);
    assert_eq!(s.user_client.user.score(), Some(Points(25)), "score increased");
    assert_eq!(s.user_client.last_game_message, Some(GameMessageType::Victory), "victory message");
    assert_eq!(s.enemy_client.last_game_message, Some(GameMessageType::Defeat), "defeat message");
    assert_clients_identical(&s);
}

#[test]
fn test_energy_increment_at_turn_start() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(0).produced_energy(0).build())
        .connect();
    s.perform_user_action(BattleAction::EndTurn);
    s.perform_enemy_action(BattleAction::EndTurn);

    assert_eq!(s.user_client.user.energy(), Some(Energy(1)), "energy incremented by 1 more");
    assert_eq!(
        s.user_client.user.produced_energy(),
        Some(Energy(1)),
        "produced energy incremented by 1 more"
    );
    assert_clients_identical(&s);
}

#[test]
fn test_create_and_play() {
    let mut s = TestBattle::builder().connect();
    s.add_to_battlefield(DisplayPlayer::Enemy, CardName::MinstrelOfFallingLight);
    s.create_and_play(TestPlayCard::builder().name(CardName::MinstrelOfFallingLight).build());
}

#[test]
fn test_play_card_dissolve_target() {
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
    assert_eq!(s.user_client.user.energy(), Some(Energy(99)), "initial energy");

    s.create_and_play(TestPlayCard::builder().name(CardName::Immolate).target(target_id).build());

    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        1,
        "one character remaining on enemy battlefield"
    );
    assert_eq!(s.user_client.cards.enemy_void().len(), 1, "dissolve character in enemy void");
    assert_eq!(s.user_client.cards.user_void().len(), 1, "event in user void");

    assert_clients_identical(&s);
}

#[test]
fn test_negate_card_on_stack() {
    let mut s = TestBattle::builder().connect();
    // Must be in hand already to not auto-resolve enemy action.
    let negate_id = s.add_to_hand(DisplayPlayer::User, CardName::Abolish);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    let enemy_character_id = s.create_and_play(
        TestPlayCard::builder()
            .name(CardName::MinstrelOfFallingLight)
            .as_player(DisplayPlayer::Enemy)
            .build(),
    );

    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_character_id),
        "enemy character on stack"
    );
    assert!(!s.user_client.enemy.can_act(), "enemy cannot act");
    assert!(s.user_client.user.can_act(), "user can act");
    s.play_card_from_hand(DisplayPlayer::User, &negate_id);
    assert_eq!(s.user_client.cards.user_hand().len(), 0, "card removed from hand");
    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        0,
        "card not present on enemy battlefield"
    );
    assert!(
        s.user_client.cards.enemy_void().contains(&enemy_character_id),
        "enemy character in void"
    );
    assert!(s.user_client.cards.user_void().contains(&negate_id), "negate in user void");
    assert_clients_identical(&s);
}

#[test]
fn test_stack_back_and_forth_with_targeting() {
    let mut s = TestBattle::builder().connect();
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let user_abolish1 = s.add_to_hand(DisplayPlayer::User, CardName::Abolish);
    let user_abolish2 = s.add_to_hand(DisplayPlayer::User, CardName::Abolish);
    let _user_abolish3 = s.add_to_hand(DisplayPlayer::User, CardName::Abolish);
    let enemy_abolish1 = s.add_to_hand(DisplayPlayer::Enemy, CardName::Abolish);
    let enemy_abolish2 = s.add_to_hand(DisplayPlayer::Enemy, CardName::Abolish);
    let _enemy_abolish3 = s.add_to_hand(DisplayPlayer::Enemy, CardName::Abolish);

    let enemy_character = s.create_and_play(
        TestPlayCard::builder()
            .name(CardName::MinstrelOfFallingLight)
            .as_player(DisplayPlayer::Enemy)
            .build(),
    );

    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_character),
        "enemy character on stack"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "one card on stack");
    assert!(s.user_client.user.can_act(), "user can act");

    s.play_card_from_hand(DisplayPlayer::User, &user_abolish1);
    assert!(s.user_client.cards.stack_cards().contains(&user_abolish1), "abolish on stack");
    assert_eq!(s.user_client.cards.stack_cards().len(), 2, "two cards on stack");
    assert!(s.user_client.enemy.can_act(), "enemy can act after abolish");

    s.play_card_from_hand(DisplayPlayer::Enemy, &enemy_abolish1);
    assert!(s.user_client.cards.stack_cards().contains(&enemy_abolish1), "enemy abolish on stack");
    assert_eq!(s.user_client.cards.stack_cards().len(), 3, "three cards on stack");
    assert!(s.user_client.user.can_act(), "user can act again");

    s.play_card_from_hand(DisplayPlayer::User, &user_abolish2);
    s.select_target(DisplayPlayer::User, &enemy_abolish1);
    assert!(
        s.user_client.cards.stack_cards().contains(&user_abolish2),
        "second user abolish on stack"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 4, "four cards on stack");
    assert!(s.user_client.enemy.can_act(), "enemy can act again");

    s.play_card_from_hand(DisplayPlayer::Enemy, &enemy_abolish2);
    s.select_target(DisplayPlayer::Enemy, &user_abolish2);
    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_abolish2),
        "second enemy abolish on stack"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 5, "five cards on stack");

    s.perform_user_action(BattleAction::PassPriority);

    assert!(s.user_client.enemy.can_act(), "enemy can act after their card resolves");
    assert!(
        s.user_client.cards.enemy_void().contains(&enemy_abolish2),
        "enemy abolish2 resolved to void"
    );
    assert!(
        s.user_client.cards.user_void().contains(&user_abolish2),
        "user abolish2 negated to void"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 3, "three cards after two resolve");

    s.perform_enemy_action(BattleAction::PassPriority);

    s.user_client.cards.stack_cards().print_ids();
    s.user_client.cards.user_void().print_ids();
    s.user_client.cards.enemy_void().print_ids();
    s.user_client.cards.enemy_battlefield().print_ids();

    assert!(
        s.user_client.cards.enemy_void().contains(&enemy_abolish1),
        "enemy abolish1 resolved to void"
    );
    assert!(
        s.user_client.cards.user_void().contains(&user_abolish1),
        "user abolish1 negated to void"
    );
    assert!(
        s.user_client.cards.enemy_battlefield().contains(&enemy_character),
        "enemy character resolved on battlefield"
    );

    assert_clients_identical(&s);
}

fn assert_clients_identical(s: &TestSession) {
    assert_eq!(
        s.user_client.cards.user_hand().len(),
        s.enemy_client.cards.enemy_hand().len(),
        "hand counts match"
    );
    assert_eq!(
        s.user_client.cards.enemy_hand().len(),
        s.enemy_client.cards.user_hand().len(),
        "enemy hand match"
    );
    assert_eq!(
        s.user_client.cards.user_battlefield().len(),
        s.enemy_client.cards.enemy_battlefield().len(),
        "battlefield counts match"
    );
    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        s.enemy_client.cards.user_battlefield().len(),
        "enemy battlefield match"
    );
    assert_eq!(
        s.user_client.cards.user_void().len(),
        s.enemy_client.cards.enemy_void().len(),
        "void counts match"
    );
    assert_eq!(
        s.user_client.cards.enemy_void().len(),
        s.enemy_client.cards.user_void().len(),
        "enemy void match"
    );
    assert_eq!(
        s.user_client.cards.stack_cards().len(),
        s.enemy_client.cards.stack_cards().len(),
        "stack counts match"
    );

    assert_eq!(s.user_client.user.energy(), s.enemy_client.enemy.energy(), "energy match");
    assert_eq!(s.user_client.enemy.energy(), s.enemy_client.user.energy(), "enemy energy match");
    assert_eq!(
        s.user_client.user.produced_energy(),
        s.enemy_client.enemy.produced_energy(),
        "produced energy match"
    );
    assert_eq!(
        s.user_client.enemy.produced_energy(),
        s.enemy_client.user.produced_energy(),
        "enemy produced match"
    );
    assert_eq!(s.user_client.user.total_spark(), s.enemy_client.enemy.total_spark(), "spark match");
    assert_eq!(
        s.user_client.enemy.total_spark(),
        s.enemy_client.user.total_spark(),
        "enemy spark match"
    );
    assert_eq!(s.user_client.user.score(), s.enemy_client.enemy.score(), "score match");
    assert_eq!(s.user_client.enemy.score(), s.enemy_client.user.score(), "enemy score match");

    assert_eq!(
        s.user_client
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
        s.user_client
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
