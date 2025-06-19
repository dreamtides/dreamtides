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
