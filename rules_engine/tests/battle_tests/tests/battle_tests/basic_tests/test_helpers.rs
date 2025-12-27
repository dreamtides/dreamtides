use test_utils::session::test_revealed_card_extension::TestRevealedCardExtension;
use test_utils::session::test_session::TestSession;
use test_utils::session::test_session_battle_extension::TestSessionBattleExtension;

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

    assert_eq!(s.user_client.me.energy(), s.enemy_client.opponent.energy(), "energy match");
    assert_eq!(s.user_client.opponent.energy(), s.enemy_client.me.energy(), "enemy energy match");
    assert_eq!(
        s.user_client.me.produced_energy(),
        s.enemy_client.opponent.produced_energy(),
        "produced energy match"
    );
    assert_eq!(
        s.user_client.opponent.produced_energy(),
        s.enemy_client.me.produced_energy(),
        "enemy produced match"
    );
    assert_eq!(
        s.user_client.me.total_spark(),
        s.enemy_client.opponent.total_spark(),
        "spark match"
    );
    assert_eq!(
        s.user_client.opponent.total_spark(),
        s.enemy_client.me.total_spark(),
        "enemy spark match"
    );
    assert_eq!(s.user_client.me.score(), s.enemy_client.opponent.score(), "score match");
    assert_eq!(s.user_client.opponent.score(), s.enemy_client.me.score(), "enemy score match");

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
