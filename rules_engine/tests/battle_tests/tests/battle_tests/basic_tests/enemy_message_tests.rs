use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use display_data::command::Command;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn enemy_message_displayed_when_spending_energy_on_dreamscatter() {
    let mut s = TestBattle::builder().connect();
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    s.create_and_play(DisplayPlayer::Enemy, CardName::Dreamscatter);
    s.click_increment_button(DisplayPlayer::Enemy);
    s.click_increment_button(DisplayPlayer::Enemy);
    s.click_primary_button(DisplayPlayer::Enemy, "Spend");

    let commands = s.last_commands.as_ref().expect("No commands found");

    let enemy_message_cmd = commands.groups.iter().flat_map(|group| &group.commands).find_map(
        |command| match command {
            Command::DisplayEnemyMessage(cmd) => Some(cmd),
            _ => None,
        },
    );

    assert!(
        enemy_message_cmd.is_some(),
        "enemy message command should be present when enemy makes choice"
    );

    let message_command = enemy_message_cmd.unwrap();
    assert!(
        message_command.message.contains("Spend") && message_command.message.contains("3"),
        "enemy message should show the energy amount spent, got: '{}'",
        message_command.message
    );
}

#[test]
fn enemy_message_displayed_when_declining_to_pay_for_ripple_of_defiance() {
    let mut s = TestBattle::builder().connect();
    let prevent_id = s.add_to_hand(DisplayPlayer::User, CardName::RippleOfDefiance);
    s.add_to_battlefield(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    s.create_and_play(DisplayPlayer::Enemy, CardName::Immolate);
    s.play_card_from_hand(DisplayPlayer::User, &prevent_id);
    s.click_secondary_button(DisplayPlayer::Enemy, "Decline");

    let commands = s.last_commands.as_ref().expect("No commands found");

    let enemy_message_cmd = commands.groups.iter().flat_map(|group| &group.commands).find_map(
        |command| match command {
            Command::DisplayEnemyMessage(cmd) => Some(cmd),
            _ => None,
        },
    );

    assert!(
        enemy_message_cmd.is_some(),
        "enemy message command should be present when enemy declines payment"
    );

    let message_command = enemy_message_cmd.unwrap();
    assert_eq!(
        message_command.message, "Decline",
        "enemy message should show 'Decline' when enemy declines to pay"
    );
}

#[test]
fn enemy_message_displayed_when_paying_for_ripple_of_defiance() {
    let mut s = TestBattle::builder().connect();
    let prevent_id = s.add_to_hand(DisplayPlayer::User, CardName::RippleOfDefiance);
    s.add_to_battlefield(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    s.create_and_play(DisplayPlayer::Enemy, CardName::Immolate);
    s.play_card_from_hand(DisplayPlayer::User, &prevent_id);
    s.click_primary_button(DisplayPlayer::Enemy, "Spend");

    let commands = s.last_commands.as_ref().expect("No commands found");

    let enemy_message_cmd = commands.groups.iter().flat_map(|group| &group.commands).find_map(
        |command| match command {
            Command::DisplayEnemyMessage(cmd) => Some(cmd),
            _ => None,
        },
    );

    assert!(
        enemy_message_cmd.is_some(),
        "enemy message command should be present when enemy pays for prevention"
    );

    let message_command = enemy_message_cmd.unwrap();
    assert!(
        message_command.message.contains("Spend") && message_command.message.contains("2"),
        "enemy message should show spending 2 energy to pay for prevention, got: '{}'",
        message_command.message
    );
}

#[test]
fn enemy_message_displayed_when_spending_minimum_energy_on_dreamscatter() {
    let mut s = TestBattle::builder().connect();
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    s.create_and_play(DisplayPlayer::Enemy, CardName::Dreamscatter);
    s.click_primary_button(DisplayPlayer::Enemy, "Spend");

    let commands = s.last_commands.as_ref().expect("No commands found");

    let enemy_message_cmd = commands.groups.iter().flat_map(|group| &group.commands).find_map(
        |command| match command {
            Command::DisplayEnemyMessage(cmd) => Some(cmd),
            _ => None,
        },
    );

    assert!(
        enemy_message_cmd.is_some(),
        "enemy message command should be present when enemy spends minimum energy"
    );

    let message_command = enemy_message_cmd.unwrap();
    assert!(
        message_command.message.contains("Spend") && message_command.message.contains("1"),
        "enemy message should show spending 1 energy for minimum, got: '{}'",
        message_command.message
    );
}
