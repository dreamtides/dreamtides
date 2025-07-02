use battle_state::actions::battle_actions::BattleAction;
use core_data::identifiers::CardName;
use core_data::numerics::{Points, Spark};
use display_data::battle_view::DisplayPlayer;
use display_data::command::Command;
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session_prelude::*;

#[test]
fn play_fast_card_during_enemy_end_step() {
    let mut s = TestBattle::builder().connect();

    let dissolve_id = s.add_to_hand(DisplayPlayer::User, CardName::TestDissolve);
    // Add another fast card to hand to prevent the next user turn from
    // automatically starting.
    s.add_to_hand(DisplayPlayer::User, CardName::TestVariableEnergyDraw);
    s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

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

    s.play_card_from_hand(DisplayPlayer::User, &dissolve_id);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 0, "character dissolved");
    assert_eq!(s.user_client.cards.enemy_void().len(), 1, "character in enemy void");
    assert_eq!(s.user_client.cards.user_void().len(), 1, "test dissolve in user void");

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

#[test]
fn judgment_command_fired_when_user_score_changes() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let _character_id = s.create_and_play(DisplayPlayer::User, CardName::TestVanillaCharacter);

    assert_eq!(s.user_client.me.total_spark(), Spark(5), "user has spark from character");
    assert_eq!(s.user_client.me.score(), Points(0), "user starts with no points");

    s.perform_user_action(BattleAction::EndTurn);
    s.perform_enemy_action(BattleAction::EndTurn);

    assert_eq!(s.user_client.me.score(), Points(5), "user gained points from judgment");

    let commands = s.last_commands.as_ref().expect("No commands found");

    let judgment_commands: Vec<_> = commands
        .groups
        .iter()
        .flat_map(|group| &group.commands)
        .filter_map(|command| match command {
            Command::DisplayJudgment(cmd) => Some(cmd),
            _ => None,
        })
        .collect();

    assert!(
        !judgment_commands.is_empty(),
        "DisplayJudgment command should be present when user score changes"
    );

    let user_judgment = judgment_commands.iter().find(|cmd| cmd.player == DisplayPlayer::User);
    assert!(
        user_judgment.is_some(),
        "DisplayJudgment command should be present for user. Found commands for: {:?}",
        judgment_commands.iter().map(|cmd| cmd.player).collect::<Vec<_>>()
    );

    let judgment_command = user_judgment.unwrap();
    assert_eq!(
        judgment_command.new_score,
        Some(Points(5)),
        "judgment command should show the new score of 5 points"
    );
}

#[test]
fn judgment_command_fired_when_no_score_change() {
    let mut s = TestBattle::builder().connect();

    assert_eq!(s.user_client.me.total_spark(), Spark(0), "user has no spark");
    assert_eq!(s.user_client.opponent.total_spark(), Spark(0), "enemy has no spark");
    assert_eq!(s.user_client.me.score(), Points(0), "user starts with no points");

    s.perform_user_action(BattleAction::EndTurn);
    s.perform_enemy_action(BattleAction::EndTurn);

    assert_eq!(s.user_client.me.score(), Points(0), "user score unchanged");

    let commands = s.last_commands.as_ref().expect("No commands found");

    let judgment_commands: Vec<_> = commands
        .groups
        .iter()
        .flat_map(|group| &group.commands)
        .filter_map(|command| match command {
            Command::DisplayJudgment(cmd) => Some(cmd),
            _ => None,
        })
        .collect();

    assert!(
        !judgment_commands.is_empty(),
        "DisplayJudgment command should still be present even when no score change"
    );

    let user_judgment = judgment_commands.iter().find(|cmd| cmd.player == DisplayPlayer::User);
    assert!(
        user_judgment.is_some(),
        "DisplayJudgment command should be present for user. Found commands for: {:?}",
        judgment_commands.iter().map(|cmd| cmd.player).collect::<Vec<_>>()
    );

    let judgment_command = user_judgment.unwrap();
    assert_eq!(
        judgment_command.new_score, None,
        "judgment command should show no new score when score doesn't change"
    );
}

#[test]
fn judgment_command_shows_total_score_not_points_gained() {
    let mut s =
        TestBattle::builder().user(TestPlayer::builder().energy(99).points(10).build()).connect();

    let _character_id = s.create_and_play(DisplayPlayer::User, CardName::TestVanillaCharacter);

    assert_eq!(s.user_client.me.total_spark(), Spark(5), "user has spark from character");
    assert_eq!(s.user_client.me.score(), Points(10), "user starts with 10 points");

    s.perform_user_action(BattleAction::EndTurn);
    s.perform_enemy_action(BattleAction::EndTurn);

    assert_eq!(s.user_client.me.score(), Points(15), "user gained 5 points from judgment");

    let commands = s.last_commands.as_ref().expect("No commands found");

    let judgment_commands: Vec<_> = commands
        .groups
        .iter()
        .flat_map(|group| &group.commands)
        .filter_map(|command| match command {
            Command::DisplayJudgment(cmd) => Some(cmd),
            _ => None,
        })
        .collect();

    assert!(
        !judgment_commands.is_empty(),
        "DisplayJudgment command should be present when user score changes"
    );

    let user_judgment = judgment_commands.iter().find(|cmd| cmd.player == DisplayPlayer::User);
    assert!(
        user_judgment.is_some(),
        "DisplayJudgment command should be present for user. Found commands for: {:?}",
        judgment_commands.iter().map(|cmd| cmd.player).collect::<Vec<_>>()
    );

    let judgment_command = user_judgment.unwrap();
    assert_eq!(
        judgment_command.new_score,
        Some(Points(15)),
        "judgment command should show the total new score of 15 points, not just the 5 points gained"
    );
}
