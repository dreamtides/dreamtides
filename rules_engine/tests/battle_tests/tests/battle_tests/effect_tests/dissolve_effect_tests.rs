use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use display_data::command::{Command, GameObjectId};
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn dissolve_enemy_character() {
    let mut s = TestBattle::builder().connect();
    let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "enemy character on battlefield");
    assert_eq!(s.user_client.cards.enemy_void().len(), 0, "enemy void empty");
    assert_eq!(s.user_client.cards.user_void().len(), 0, "user void empty");

    s.create_and_play(DisplayPlayer::User, CardName::TestDissolve);

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
    let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    let test_dissolve_id = s.create_and_play(DisplayPlayer::User, CardName::TestDissolve);

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
    let target1_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);
    let target2_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        2,
        "two enemy characters on battlefield"
    );

    s.create_and_play(DisplayPlayer::User, CardName::TestDissolve);
    s.select_target(DisplayPlayer::User, &target1_id);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "one enemy character remains");
    assert_eq!(s.user_client.cards.enemy_void().len(), 1, "one enemy character dissolved");
    assert!(s.user_client.cards.enemy_void().contains(&target1_id), "correct target dissolved");
    assert!(s.user_client.cards.enemy_battlefield().contains(&target2_id), "other target remains");
}

#[test]
fn dissolve_card_command() {
    let mut s = TestBattle::builder().connect();
    let target_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestDissolve);

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
