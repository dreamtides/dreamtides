use display_data::battle_view::DisplayPlayer;
use display_data::command::Command;
use tabula_ids::test_card;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn enemy_message_displayed_when_spending_energy_on_energy_prompt() {
    let mut s = TestBattle::builder().connect();
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    s.create_and_play(DisplayPlayer::Enemy, test_card::TEST_VARIABLE_ENERGY_DRAW);
    s.click_increment_button(DisplayPlayer::Enemy);
    s.click_increment_button(DisplayPlayer::Enemy);
    s.click_primary_button(DisplayPlayer::Enemy, "Spend");

    let commands = s.last_user_commands.as_ref().expect("No commands found");

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
fn enemy_message_displayed_when_declining_to_pay_for_counterspell_unless_pays() {
    let mut s = TestBattle::builder().connect();
    let counterspell_id =
        s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL_UNLESS_PAYS);
    s.add_to_battlefield(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    s.create_and_play(DisplayPlayer::Enemy, test_card::TEST_DISSOLVE);
    s.play_card_from_hand(DisplayPlayer::User, &counterspell_id);
    s.click_secondary_button(DisplayPlayer::Enemy, "Decline");

    let commands = s.last_user_commands.as_ref().expect("No commands found");

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
fn enemy_message_displayed_when_paying_for_counterspell_unless_pays() {
    let mut s = TestBattle::builder().connect();
    let counterspell_id =
        s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL_UNLESS_PAYS);
    s.add_to_battlefield(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    s.create_and_play(DisplayPlayer::Enemy, test_card::TEST_DISSOLVE);
    s.play_card_from_hand(DisplayPlayer::User, &counterspell_id);
    s.click_primary_button(DisplayPlayer::Enemy, "Spend");

    let commands = s.last_user_commands.as_ref().expect("No commands found");

    let enemy_message_cmd = commands.groups.iter().flat_map(|group| &group.commands).find_map(
        |command| match command {
            Command::DisplayEnemyMessage(cmd) => Some(cmd),
            _ => None,
        },
    );

    assert!(
        enemy_message_cmd.is_some(),
        "enemy message command should be present when enemy pays for counterspell"
    );

    let message_command = enemy_message_cmd.unwrap();
    assert!(
        message_command.message.contains("Spend") && message_command.message.contains("2"),
        "enemy message should show spending 2 energy to pay for counterspell, got: '{}'",
        message_command.message
    );
}

#[test]
fn enemy_message_displayed_when_spending_minimum_energy_on_energy_prompt() {
    let mut s = TestBattle::builder().connect();
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    s.create_and_play(DisplayPlayer::Enemy, test_card::TEST_VARIABLE_ENERGY_DRAW);
    s.click_primary_button(DisplayPlayer::Enemy, "Spend");

    let commands = s.last_user_commands.as_ref().expect("No commands found");

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
