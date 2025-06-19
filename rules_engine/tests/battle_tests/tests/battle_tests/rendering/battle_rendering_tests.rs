use battle_state::actions::battle_actions::BattleAction;
use display_data::command::GameMessageType;
use test_utils::battle::test_battle::TestBattle;

#[test]
fn test_connect() {
    let s = TestBattle::new().connect();
    assert_eq!(s.client.cards.user_hand().len(), 5);
    assert_eq!(s.client.cards.enemy_hand().len(), 5);
    assert_eq!(s.client.cards.user_void().len(), 0);
    assert_eq!(s.client.cards.enemy_void().len(), 0);
    assert_eq!(s.client.cards.user_battlefield().len(), 0);
    assert_eq!(s.client.cards.enemy_battlefield().len(), 0);
    assert_eq!(s.client.cards.stack_cards().len(), 0);
}

#[test]
fn test_perform_action_completes() {
    let mut s = TestBattle::new().connect();
    assert_eq!(s.client.last_game_message, None);
    s.perform_action(BattleAction::EndTurn);
    assert_eq!(s.client.last_game_message, Some(GameMessageType::EnemyTurn));
}
