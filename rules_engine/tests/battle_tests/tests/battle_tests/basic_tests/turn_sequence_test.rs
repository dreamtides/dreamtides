use battle_state::actions::battle_actions::BattleAction;
use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn play_fast_card_during_enemy_end_step() {
    let mut s = TestBattle::builder().connect();

    let immolate_id = s.add_to_hand(DisplayPlayer::User, CardName::Immolate);
    // Add another fast card to hand to prevent the next user turn from
    // automatically starting.
    s.add_to_hand(DisplayPlayer::User, CardName::Dreamscatter);
    s.add_to_battlefield(DisplayPlayer::Enemy, CardName::MinstrelOfFallingLight);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "enemy has one character");
    assert!(s.user_client.me.can_act(), "user can act on their turn");

    assert!(
        s.user_client.me.is_current_turn(),
        "user should have is_current_turn=true during their turn"
    );
    assert!(
        !s.user_client.opponent.is_current_turn(),
        "enemy should have is_current_turn=false during user's turn"
    );

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    assert!(s.user_client.opponent.can_act(), "enemy can act on their turn");
    assert!(!s.user_client.me.can_act(), "user cannot act during enemy turn");

    assert!(
        !s.user_client.me.is_current_turn(),
        "user should have is_current_turn=false during enemy's turn"
    );
    assert!(
        s.user_client.opponent.is_current_turn(),
        "enemy should have is_current_turn=true during their turn"
    );

    s.perform_enemy_action(BattleAction::EndTurn);

    assert!(!s.user_client.opponent.can_act(), "enemy cannot act after ending turn");
    assert!(s.user_client.me.can_act(), "user can act during enemy end step");

    assert!(
        !s.user_client.me.is_current_turn(),
        "user should have is_current_turn=false during enemy end step"
    );
    assert!(
        s.user_client.opponent.is_current_turn(),
        "enemy should have is_current_turn=true during their end step"
    );

    s.play_card_from_hand(DisplayPlayer::User, &immolate_id);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 0, "character dissolved");
    assert_eq!(s.user_client.cards.enemy_void().len(), 1, "character in enemy void");
    assert_eq!(s.user_client.cards.user_void().len(), 1, "immolate in user void");

    s.perform_user_action(BattleAction::StartNextTurn);

    assert!(s.user_client.me.can_act(), "user can act on their new turn");
    assert!(!s.user_client.opponent.can_act(), "enemy cannot act during user turn");

    assert!(
        s.user_client.me.is_current_turn(),
        "user should have is_current_turn=true during their new turn"
    );
    assert!(
        !s.user_client.opponent.is_current_turn(),
        "enemy should have is_current_turn=false during user's new turn"
    );
}
