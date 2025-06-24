use battle_state::actions::battle_actions::BattleAction;
use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn negate_card_on_stack() {
    let mut s = TestBattle::builder().connect();
    let negate_id = s.add_to_hand(DisplayPlayer::User, CardName::Abolish);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    let enemy_character_id =
        s.create_and_play(DisplayPlayer::Enemy, CardName::MinstrelOfFallingLight);

    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_character_id),
        "enemy character on stack"
    );
    assert!(!s.user_client.opponent.can_act(), "enemy cannot act");
    assert!(s.user_client.me.can_act(), "user can act");
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
}

#[test]
fn stack_back_and_forth_with_targeting() {
    let mut s = TestBattle::builder().connect();
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let user_abolish1 = s.add_to_hand(DisplayPlayer::User, CardName::Abolish);
    let user_abolish2 = s.add_to_hand(DisplayPlayer::User, CardName::Abolish);
    let _user_abolish3 = s.add_to_hand(DisplayPlayer::User, CardName::Abolish);
    let enemy_abolish1 = s.add_to_hand(DisplayPlayer::Enemy, CardName::Abolish);
    let enemy_abolish2 = s.add_to_hand(DisplayPlayer::Enemy, CardName::Abolish);
    let _enemy_abolish3 = s.add_to_hand(DisplayPlayer::Enemy, CardName::Abolish);

    let enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::MinstrelOfFallingLight);

    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_character),
        "enemy character on stack"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "one card on stack");
    assert!(s.user_client.me.can_act(), "user can act");

    s.play_card_from_hand(DisplayPlayer::User, &user_abolish1);
    assert!(s.user_client.cards.stack_cards().contains(&user_abolish1), "abolish on stack");
    assert_eq!(s.user_client.cards.stack_cards().len(), 2, "two cards on stack");
    assert!(s.user_client.opponent.can_act(), "enemy can act after abolish");

    s.play_card_from_hand(DisplayPlayer::Enemy, &enemy_abolish1);
    assert!(s.user_client.cards.stack_cards().contains(&enemy_abolish1), "enemy abolish on stack");
    assert_eq!(s.user_client.cards.stack_cards().len(), 3, "three cards on stack");
    assert!(s.user_client.me.can_act(), "user can act again");

    s.play_card_from_hand(DisplayPlayer::User, &user_abolish2);
    s.select_target(DisplayPlayer::User, &enemy_abolish1);
    assert!(
        s.user_client.cards.stack_cards().contains(&user_abolish2),
        "second user abolish on stack"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 4, "four cards on stack");
    assert!(s.user_client.opponent.can_act(), "enemy can act again");

    s.play_card_from_hand(DisplayPlayer::Enemy, &enemy_abolish2);
    s.select_target(DisplayPlayer::Enemy, &user_abolish2);
    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_abolish2),
        "second enemy abolish on stack"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 5, "five cards on stack");

    s.perform_user_action(BattleAction::PassPriority);

    assert!(s.user_client.opponent.can_act(), "enemy can act after their card resolves");
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
}

#[test]
fn resolve_negate_with_removed_target() {
    let mut s = TestBattle::builder().connect();
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let user_abolish1 = s.add_to_hand(DisplayPlayer::User, CardName::Abolish);
    let user_abolish2 = s.add_to_hand(DisplayPlayer::User, CardName::Abolish);
    let _user_extra = s.add_to_hand(DisplayPlayer::User, CardName::Abolish);
    let enemy_dreamscatter = s.add_to_hand(DisplayPlayer::Enemy, CardName::Dreamscatter);
    let _enemy_extra = s.add_to_hand(DisplayPlayer::Enemy, CardName::Abolish);

    let enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::MinstrelOfFallingLight);
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "one card on stack");

    s.play_card_from_hand(DisplayPlayer::User, &user_abolish1);
    assert_eq!(s.user_client.cards.stack_cards().len(), 2, "two cards on stack");

    s.play_card_from_hand(DisplayPlayer::Enemy, &enemy_dreamscatter);
    s.click_primary_button(DisplayPlayer::Enemy, "Spend");
    assert_eq!(s.user_client.cards.stack_cards().len(), 3, "three cards on stack");

    s.play_card_from_hand(DisplayPlayer::User, &user_abolish2);
    s.select_target(DisplayPlayer::User, &enemy_character);
    assert_eq!(s.user_client.cards.stack_cards().len(), 4, "four cards on stack");

    s.perform_enemy_action(BattleAction::PassPriority);

    assert!(
        s.user_client.cards.user_void().contains(&user_abolish2),
        "user abolish2 resolved to void"
    );
    assert!(
        s.user_client.cards.enemy_void().contains(&enemy_character),
        "enemy character removed to void"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 2, "two cards left on stack");
    assert!(s.user_client.me.can_act(), "user has priority after abolish2 resolves");

    s.perform_user_action(BattleAction::PassPriority);

    assert!(
        s.user_client.cards.enemy_void().contains(&enemy_dreamscatter),
        "enemy dreamscatter resolved to void"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "one card left on stack");
    assert!(s.user_client.opponent.can_act(), "enemy has priority after dreamscatter resolves");

    s.perform_enemy_action(BattleAction::PassPriority);

    assert!(
        s.user_client.cards.user_void().contains(&user_abolish1),
        "user abolish1 resolved to void with no effect"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "stack is empty");
}

#[test]
fn resolve_immolate_with_removed_target() {
    let mut s = TestBattle::builder().connect();
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let user_immolate1 = s.add_to_hand(DisplayPlayer::User, CardName::Immolate);
    let user_immolate2 = s.add_to_hand(DisplayPlayer::User, CardName::Immolate);
    s.add_to_hand(DisplayPlayer::Enemy, CardName::Immolate);

    let enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::MinstrelOfFallingLight);
    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "enemy character on battlefield");

    s.play_card_from_hand(DisplayPlayer::User, &user_immolate1);
    s.select_target(DisplayPlayer::User, &enemy_character);
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "one card on stack");

    let enemy_immolate = s.create_and_play(DisplayPlayer::Enemy, CardName::Immolate);
    s.select_target(DisplayPlayer::Enemy, &enemy_character);
    assert_eq!(s.user_client.cards.stack_cards().len(), 2, "two cards on stack");

    s.play_card_from_hand(DisplayPlayer::User, &user_immolate2);
    s.select_target(DisplayPlayer::User, &enemy_character);
    assert_eq!(s.user_client.cards.stack_cards().len(), 3, "three cards on stack");

    s.perform_enemy_action(BattleAction::PassPriority);

    assert!(
        s.user_client.cards.user_void().contains(&user_immolate2),
        "second user immolate resolved to void"
    );
    assert!(
        s.user_client.cards.enemy_void().contains(&enemy_character),
        "enemy character dissolved"
    );
    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 0, "no characters on battlefield");
    assert_eq!(s.user_client.cards.stack_cards().len(), 2, "two cards left on stack");
    assert!(s.user_client.me.can_act(), "user has priority after immolate2 resolves");

    s.perform_user_action(BattleAction::PassPriority);

    assert!(
        s.user_client.cards.enemy_void().contains(&enemy_immolate),
        "enemy immolate resolved to void with no effect"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "one card left on stack");
    assert!(s.user_client.opponent.can_act(), "enemy has priority after enemy immolate resolves");

    s.perform_enemy_action(BattleAction::PassPriority);

    assert!(
        s.user_client.cards.user_void().contains(&user_immolate1),
        "first user immolate resolved to void with no effect"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "stack is empty");
}
