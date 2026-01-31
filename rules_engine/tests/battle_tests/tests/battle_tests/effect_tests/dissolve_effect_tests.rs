use display_data::battle_view::DisplayPlayer;
use display_data::command::{Command, GameObjectId};
use tabula_generated::test_card;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn dissolve_enemy_character() {
    let mut s = TestBattle::builder().connect();
    let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "enemy character on battlefield");
    assert_eq!(s.user_client.cards.enemy_void().len(), 0, "enemy void empty");
    assert_eq!(s.user_client.cards.user_void().len(), 0, "user void empty");

    s.create_and_play(DisplayPlayer::User, test_card::TEST_DISSOLVE);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 0, "enemy character dissolved");
    assert_eq!(s.user_client.cards.enemy_void().len(), 1, "enemy character in void");
    assert_eq!(s.user_client.cards.user_void().len(), 1, "test dissolve card in user void");
    assert!(
        s.user_client.cards.enemy_void().contains(&target_id),
        "target character in enemy void"
    );
}

#[test]
fn dissolve_fire_projectile_command() {
    let mut s = TestBattle::builder().connect();
    let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

    let test_dissolve_id = s.create_and_play(DisplayPlayer::User, test_card::TEST_DISSOLVE);

    let fire_projectile_commands =
        s.find_all_commands(DisplayPlayer::User, |command| match command {
            Command::FireProjectile(cmd) => Some(cmd),
            _ => None,
        });

    assert!(
        !fire_projectile_commands.is_empty(),
        "fire projectile command should be present when Test Dissolve resolves"
    );

    let fire_projectile = fire_projectile_commands
        .iter()
        .find(|cmd| {
            cmd.source_id == GameObjectId::CardId(test_dissolve_id.clone())
                && cmd.target_id == GameObjectId::CardId(target_id.clone())
        })
        .expect("Should find fire projectile command with correct source and target");

    assert_eq!(
        fire_projectile.source_id,
        GameObjectId::CardId(test_dissolve_id),
        "fire projectile source should be the test dissolve card"
    );

    assert_eq!(
        fire_projectile.target_id,
        GameObjectId::CardId(target_id.clone()),
        "fire projectile target should be the target character"
    );

    assert!(s.user_client.cards.enemy_void().contains(&target_id), "target dissolved to void");
}

#[test]
fn dissolve_with_multiple_targets() {
    let mut s = TestBattle::builder().connect();
    let target1_id = s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);
    let target2_id = s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        2,
        "two enemy characters on battlefield"
    );

    s.create_and_play(DisplayPlayer::User, test_card::TEST_DISSOLVE);
    s.click_card(DisplayPlayer::User, &target1_id);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "one enemy character remains");
    assert_eq!(s.user_client.cards.enemy_void().len(), 1, "one enemy character dissolved");
    assert!(s.user_client.cards.enemy_void().contains(&target1_id), "correct target dissolved");
    assert!(s.user_client.cards.enemy_battlefield().contains(&target2_id), "other target remains");
}

#[test]
fn dissolve_card_command() {
    let mut s = TestBattle::builder().connect();
    let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

    s.create_and_play(DisplayPlayer::User, test_card::TEST_DISSOLVE);

    let dissolve_commands = s.find_all_commands(DisplayPlayer::User, |command| match command {
        Command::DissolveCard(cmd) => Some(cmd),
        _ => None,
    });

    assert!(
        !dissolve_commands.is_empty(),
        "dissolve card command should be present when Test Dissolve resolves"
    );

    let dissolve_card = dissolve_commands
        .iter()
        .find(|cmd| cmd.target == target_id)
        .expect("Should find dissolve command targeting the correct character");

    assert_eq!(
        dissolve_card.target, target_id,
        "dissolve card command should target the correct character"
    );

    assert!(s.user_client.cards.enemy_void().contains(&target_id), "target dissolved to void");
}

#[test]
fn prevent_event_which_could_dissolve_ally() {
    let mut s = TestBattle::builder().connect();
    let counterspell_id =
        s.add_to_hand(DisplayPlayer::User, test_card::TEST_PREVENT_EVENT_WHICH_COULD_DISSOLVE_ALLY);
    let user_character_id =
        s.add_to_battlefield(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    // Enemy plays a dissolve targeting user's character
    s.create_and_play(DisplayPlayer::Enemy, test_card::TEST_DISSOLVE);

    // User counters with the "prevent event which could dissolve ally" card
    s.play_card_from_hand(DisplayPlayer::User, &counterspell_id);

    // The dissolve should be prevented
    assert!(
        s.user_client.cards.stack_cards().is_empty(),
        "stack should be empty after cards resolve"
    );
    assert_eq!(
        s.user_client.cards.user_battlefield().len(),
        1,
        "user's character should still be on battlefield"
    );
    assert!(
        s.user_client.cards.user_battlefield().contains(&user_character_id),
        "user's character should not have been dissolved"
    );
}

#[test]
fn prevent_event_which_could_dissolve_ally_does_not_affect_non_dissolve() {
    let mut s = TestBattle::builder().connect();
    let counterspell_id =
        s.add_to_hand(DisplayPlayer::User, test_card::TEST_PREVENT_EVENT_WHICH_COULD_DISSOLVE_ALLY);
    s.add_to_battlefield(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    // Enemy plays a draw card (not a dissolve effect)
    s.create_and_play(DisplayPlayer::Enemy, test_card::TEST_DRAW_ONE);

    // The counterspell should NOT be playable because the enemy card is not a
    // dissolve
    let card_view = s.user_client.cards.card_map.get(&counterspell_id).expect("card should exist");
    let revealed = card_view.view.revealed.as_ref().expect("card should be revealed");
    assert!(
        revealed.actions.can_play.is_none(),
        "prevent event which could dissolve ally should not be playable for non-dissolve cards"
    );
}
