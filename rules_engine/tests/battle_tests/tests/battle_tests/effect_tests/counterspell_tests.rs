use core_data::identifiers::CardName;
use core_data::numerics::Energy;
use display_data::battle_view::DisplayPlayer;
use display_data::command::{Command, GameObjectId};
use display_data::object_position::Position;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn counterspell_unless_pays_cost() {
    let mut s = TestBattle::builder().connect();
    let counterspell_id = s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspellUnlessPays);
    s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let initial_enemy_energy = s.user_client.opponent.energy();

    let event_id = s.create_and_play(DisplayPlayer::Enemy, CardName::TestDissolve);
    let event_cost = s.user_client.cards.get_cost(&event_id);
    s.play_card_from_hand(DisplayPlayer::User, &counterspell_id);
    s.click_primary_button(DisplayPlayer::Enemy, "Spend");
    assert!(
        s.user_client.cards.stack_cards().is_empty(),
        "stack should be empty after cards resolve"
    );
    assert_eq!(
        s.user_client.cards.user_battlefield().len(),
        0,
        "character should be dissolved by Test Dissolve"
    );
    assert_eq!(
        s.user_client.opponent.energy(),
        initial_enemy_energy - Energy(2) - event_cost,
        "enemy should have spent 2 more energy"
    );
}

#[test]
fn counterspell_unless_pays_cost_decline() {
    let mut s = TestBattle::builder().connect();
    let counterspell_id = s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspellUnlessPays);
    s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let initial_enemy_energy = s.user_client.opponent.energy();

    let event_id = s.create_and_play(DisplayPlayer::Enemy, CardName::TestDissolve);
    let event_cost = s.user_client.cards.get_cost(&event_id);
    s.play_card_from_hand(DisplayPlayer::User, &counterspell_id);
    s.click_secondary_button(DisplayPlayer::Enemy, "Decline");
    assert!(
        s.user_client.cards.stack_cards().is_empty(),
        "stack should be empty after cards resolve"
    );
    assert_eq!(
        s.user_client.cards.user_battlefield().len(),
        1,
        "character should not be dissolved by Test Dissolve"
    );
    assert_eq!(
        s.user_client.opponent.energy(),
        initial_enemy_energy - event_cost,
        "enemy should have only spent the original event cost"
    );
}

#[test]
fn test_counterspell_unless_pays_fire_projectile_only_on_decline() {
    let mut s = TestBattle::builder().connect();
    let counterspell_id = s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspellUnlessPays);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    s.create_and_play(DisplayPlayer::Enemy, CardName::TestVariableEnergyDraw);
    s.click_primary_button(DisplayPlayer::Enemy, "Spend");

    s.play_card_from_hand(DisplayPlayer::User, &counterspell_id);

    let commands_after_resolve = s.last_commands.as_ref().expect("No commands found");
    let fire_projectile_after_resolve =
        commands_after_resolve.groups.iter().flat_map(|group| &group.commands).find_map(
            |command| match command {
                Command::FireProjectile(_) => Some(command),
                _ => None,
            },
        );
    assert!(
        fire_projectile_after_resolve.is_none(),
        "no fire projectile command should occur when Test Counterspell Unless Pays resolves"
    );

    s.click_primary_button(DisplayPlayer::Enemy, "Spend");

    let commands_after_spend = s.last_commands.as_ref().expect("No commands found");
    let fire_projectile_after_spend =
        commands_after_spend.groups.iter().flat_map(|group| &group.commands).find_map(|command| {
            match command {
                Command::FireProjectile(_) => Some(command),
                _ => None,
            }
        });
    assert!(
        fire_projectile_after_spend.is_none(),
        "no fire projectile command should occur when Spend button is clicked"
    );

    let mut s2 = TestBattle::builder().connect();
    s2.end_turn_remove_opponent_hand(DisplayPlayer::User);
    let counterspell_id2 =
        s2.add_to_hand(DisplayPlayer::User, CardName::TestCounterspellUnlessPays);
    let event_id2 = s2.create_and_play(DisplayPlayer::Enemy, CardName::TestVariableEnergyDraw);
    s2.click_primary_button(DisplayPlayer::Enemy, "Spend");
    s2.play_card_from_hand(DisplayPlayer::User, &counterspell_id2);
    s2.click_secondary_button(DisplayPlayer::Enemy, "Decline");

    let commands_after_decline = s2.last_commands.as_ref().expect("No commands found");
    let fire_projectile_after_decline =
        commands_after_decline.groups.iter().flat_map(|group| &group.commands).find_map(
            |command| match command {
                Command::FireProjectile(cmd) => Some(cmd),
                _ => None,
            },
        );
    assert!(
        fire_projectile_after_decline.is_some(),
        "fire projectile command should occur when Decline button is clicked"
    );

    let fire_projectile = fire_projectile_after_decline.unwrap();
    assert_eq!(
        fire_projectile.source_id,
        GameObjectId::CardId(counterspell_id2),
        "fire projectile source should be the Test Counterspell Unless Pays card"
    );
    assert_eq!(
        fire_projectile.target_id,
        GameObjectId::CardId(event_id2),
        "fire projectile target should be the event being counterspelled"
    );
}

#[test]
fn test_counterspell_unless_pays_stays_on_stack_during_prompt() {
    let mut s = TestBattle::builder().connect();
    let counterspell_id = s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspellUnlessPays);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    s.create_and_play(DisplayPlayer::Enemy, CardName::TestVariableEnergyDraw);
    s.click_primary_button(DisplayPlayer::Enemy, "Spend");

    s.play_card_from_hand(DisplayPlayer::User, &counterspell_id);

    let card_view = s
        .user_client
        .cards
        .card_map
        .get(&counterspell_id)
        .expect("Test Counterspell Unless Pays card should exist");

    assert!(
        matches!(card_view.view.position.position, Position::OnStack(_)),
        "Test Counterspell Unless Pays should be on stack during prompt, but was at position: {:?}",
        card_view.view.position.position
    );

    s.click_secondary_button(DisplayPlayer::Enemy, "Decline");

    let card_view_after_decline = s
        .user_client
        .cards
        .card_map
        .get(&counterspell_id)
        .expect("Test Counterspell Unless Pays card should exist");

    assert!(
        !matches!(card_view_after_decline.view.position.position, Position::OnStack(_)),
        "Test Counterspell Unless Pays should no longer be on stack after prompt resolves, but was at position: {:?}",
        card_view_after_decline.view.position.position
    );
}
