use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use display_data::command::{Command, GameObjectId};
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn immolate_dissolve_enemy_character() {
    let mut s = TestBattle::builder().connect();
    let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::MinstrelOfFallingLight);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "enemy character on battlefield");
    assert_eq!(s.user_client.cards.enemy_void().len(), 0, "enemy void empty");
    assert_eq!(s.user_client.cards.user_void().len(), 0, "user void empty");

    s.create_and_play(DisplayPlayer::User, CardName::Immolate);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 0, "enemy character dissolved");
    assert_eq!(s.user_client.cards.enemy_void().len(), 1, "enemy character in void");
    assert_eq!(s.user_client.cards.user_void().len(), 1, "immolate card in user void");
    assert!(
        s.user_client.cards.enemy_void().contains(&target_id),
        "target character in enemy void"
    );
}

#[test]
fn immolate_fire_projectile_command() {
    let mut s = TestBattle::builder().connect();
    let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::MinstrelOfFallingLight);

    let immolate_id = s.create_and_play(DisplayPlayer::User, CardName::Immolate);

    let commands = s.last_commands.as_ref().expect("No commands found");

    let fire_projectile_cmd = commands.groups.iter().flat_map(|group| &group.commands).find_map(
        |command| match command {
            Command::FireProjectile(cmd) => Some(cmd),
            _ => None,
        },
    );

    assert!(
        fire_projectile_cmd.is_some(),
        "fire projectile command should be present when Immolate resolves"
    );

    let fire_projectile = fire_projectile_cmd.unwrap();

    assert_eq!(
        fire_projectile.source_id,
        GameObjectId::CardId(immolate_id),
        "fire projectile source should be the immolate card"
    );

    assert_eq!(
        fire_projectile.target_id,
        GameObjectId::CardId(target_id.clone()),
        "fire projectile target should be the target character"
    );

    assert!(s.user_client.cards.enemy_void().contains(&target_id), "target dissolved to void");
}

#[test]
fn immolate_with_multiple_targets() {
    let mut s = TestBattle::builder().connect();
    let target1_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::MinstrelOfFallingLight);
    let target2_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::MinstrelOfFallingLight);

    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        2,
        "two enemy characters on battlefield"
    );

    s.create_and_play(DisplayPlayer::User, CardName::Immolate);
    s.select_target(DisplayPlayer::User, &target1_id);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "one enemy character remains");
    assert_eq!(s.user_client.cards.enemy_void().len(), 1, "one enemy character dissolved");
    assert!(s.user_client.cards.enemy_void().contains(&target1_id), "correct target dissolved");
    assert!(s.user_client.cards.enemy_battlefield().contains(&target2_id), "other target remains");
}
