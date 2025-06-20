use battle_state::actions::battle_actions::BattleAction;
use display_data::command::GameMessageType;
use test_utils::battle::test_battle::TestBattle;

#[test]
fn test_connect() {
    let s = TestBattle::new().connect();
    assert_eq!(s.client.cards.user_hand().len(), 0);
    assert_eq!(s.client.cards.enemy_hand().len(), 0);
    assert_eq!(s.client.cards.user_void().len(), 0);
    assert_eq!(s.client.cards.enemy_void().len(), 0);
    assert_eq!(s.client.cards.user_battlefield().len(), 0);
    assert_eq!(s.client.cards.enemy_battlefield().len(), 0);
    assert_eq!(s.client.cards.stack_cards().len(), 0);
}

#[test]
fn test_end_turn() {
    let mut s = TestBattle::new().connect();
    assert_eq!(s.client.last_game_message, None);
    s.perform_action(BattleAction::EndTurn);
    assert_eq!(s.client.last_game_message, Some(GameMessageType::EnemyTurn));
}

#[test]
fn test_turn_cycle() {
    let mut s = TestBattle::new().connect();
    assert_eq!(s.client.last_game_message, None);
    assert_eq!(s.enemy_client.last_game_message, None);
    assert_eq!(s.client.cards.user_hand().len(), 0);
    assert_eq!(s.enemy_client.cards.user_hand().len(), 0);
    assert_eq!(s.client.cards.enemy_hand().len(), 0);
    assert_eq!(s.enemy_client.cards.enemy_hand().len(), 0);
    s.perform_action(BattleAction::EndTurn);
    assert_eq!(s.client.last_game_message, Some(GameMessageType::EnemyTurn));
    assert_eq!(s.enemy_client.last_game_message, Some(GameMessageType::YourTurn));
    s.perform_enemy_action(BattleAction::EndTurn);
    assert_eq!(s.client.last_game_message, Some(GameMessageType::YourTurn));
    assert_eq!(s.enemy_client.last_game_message, Some(GameMessageType::EnemyTurn));
    assert_eq!(s.client.cards.user_hand().len(), 1);
    assert_eq!(s.enemy_client.cards.enemy_hand().len(), 1);
    assert_eq!(s.client.cards.enemy_hand().len(), 1);
    assert_eq!(s.enemy_client.cards.user_hand().len(), 1);
}
