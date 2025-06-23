use test_utils::session::test_session::TestSession;

pub fn assert_clients_identical(s: &TestSession) {
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
