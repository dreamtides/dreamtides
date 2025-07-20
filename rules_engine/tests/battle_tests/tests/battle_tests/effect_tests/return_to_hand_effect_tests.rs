use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn return_enemy_character_to_hand() {
    let mut s = TestBattle::builder().connect();
    let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "enemy character on battlefield");
    assert_eq!(s.user_client.cards.enemy_hand().len(), 0, "enemy hand empty");
    assert_eq!(s.user_client.cards.user_void().len(), 0, "user void empty");

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnToHand);

    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        0,
        "enemy character returned to hand"
    );
    assert_eq!(s.user_client.cards.enemy_hand().len(), 1, "enemy character in hand");
    assert_eq!(s.user_client.cards.user_void().len(), 1, "test return to hand card in user void");
    assert!(
        s.user_client.cards.enemy_hand().contains(&target_id),
        "target character in enemy hand"
    );
}

#[test]
fn return_to_hand_with_multiple_targets() {
    let mut s = TestBattle::builder().connect();
    let target1_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);
    let target2_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        2,
        "two enemy characters on battlefield"
    );

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnToHand);
    s.invoke_click(DisplayPlayer::User, &target1_id);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "one enemy character remains");
    assert_eq!(s.user_client.cards.enemy_hand().len(), 1, "one enemy character returned to hand");
    assert!(
        s.user_client.cards.enemy_hand().contains(&target1_id),
        "correct target returned to hand"
    );
    assert!(s.user_client.cards.enemy_battlefield().contains(&target2_id), "other target remains");
}

#[test]
fn return_to_hand_auto_targets_single_enemy() {
    let mut s = TestBattle::builder().connect();
    let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "enemy character on battlefield");
    assert_eq!(s.user_client.cards.enemy_hand().len(), 0, "enemy hand empty");

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnToHand);

    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        0,
        "enemy character returned to hand"
    );
    assert_eq!(s.user_client.cards.enemy_hand().len(), 1, "enemy character in hand");
    assert!(
        s.user_client.cards.enemy_hand().contains(&target_id),
        "target character returned to hand"
    );
}

#[test]
fn return_to_hand_only_targets_enemy_characters() {
    let mut s = TestBattle::builder().connect();
    let user_character_id =
        s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);
    let return_card_id = s.add_to_hand(DisplayPlayer::User, CardName::TestReturnToHand);

    let return_card = s.user_client.cards.get_revealed(&return_card_id);
    assert!(
        return_card.actions.can_play.is_none(),
        "return to hand should not be playable when no enemy characters present"
    );

    assert_eq!(s.user_client.cards.user_battlefield().len(), 1, "user character remains");
    assert!(
        s.user_client.cards.user_battlefield().contains(&user_character_id),
        "user character untouched"
    );
}
