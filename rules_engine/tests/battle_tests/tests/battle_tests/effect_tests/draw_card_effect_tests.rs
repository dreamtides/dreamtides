use core_data::identifiers::CardName;
use core_data::numerics::Energy;
use display_data::battle_view::{BattlePreviewState, DisplayPlayer};
use display_data::command::Command;
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session_prelude::*;
use ui_components::icon;

#[test]
fn draw_card_for_each_energy_spent() {
    let mut s = TestBattle::builder().connect();
    let starting_energy = s.user_client.me.energy();
    let id = s.create_and_play(DisplayPlayer::User, CardName::TestVariableEnergyDraw);
    let cost = s.user_client.cards.get_cost(&id);
    s.click_increment_button(DisplayPlayer::User);
    s.click_increment_button(DisplayPlayer::User);
    s.click_primary_button(DisplayPlayer::User, "Spend");
    assert_eq!(
        s.user_client.me.energy(),
        starting_energy - cost - Energy(3),
        "user should have spent 3 energy but has {} energy",
        s.user_client.me.energy()
    );
    assert_eq!(
        s.user_client.cards.user_hand().len(),
        3,
        "user should have drawn 3 cards but has {} cards",
        s.user_client.cards.user_hand().len()
    );
}

#[test]
fn draw_card_animation_command() {
    let mut s = TestBattle::builder().connect();
    s.create_and_play(DisplayPlayer::User, CardName::TestVariableEnergyDraw);
    s.click_increment_button(DisplayPlayer::User);
    s.click_primary_button(DisplayPlayer::User, "Spend");

    let commands = s.last_user_commands.as_ref().expect("No commands found");

    let draw_cards_cmd = commands.groups.iter().flat_map(|group| &group.commands).find_map(
        |command| match command {
            Command::DrawUserCards(cmd) => Some(cmd),
            _ => None,
        },
    );

    assert!(
        draw_cards_cmd.is_some(),
        "draw user cards command should be present when cards are drawn"
    );

    let draw_command = draw_cards_cmd.unwrap();

    assert_eq!(
        draw_command.cards.len(),
        2,
        "draw command should contain 2 cards for 2 energy spent"
    );
}

#[test]
fn draw_multiple_cards_animation() {
    let mut s = TestBattle::builder().connect();
    s.create_and_play(DisplayPlayer::User, CardName::TestVariableEnergyDraw);
    s.click_increment_button(DisplayPlayer::User);
    s.click_increment_button(DisplayPlayer::User);
    s.click_increment_button(DisplayPlayer::User);
    s.click_primary_button(DisplayPlayer::User, "Spend");

    let commands = s.last_user_commands.as_ref().expect("No commands found");

    let draw_cards_cmd = commands.groups.iter().flat_map(|group| &group.commands).find_map(
        |command| match command {
            Command::DrawUserCards(cmd) => Some(cmd),
            _ => None,
        },
    );

    assert!(
        draw_cards_cmd.is_some(),
        "draw user cards command should be present when multiple cards are drawn"
    );

    let draw_command = draw_cards_cmd.unwrap();

    assert_eq!(
        draw_command.cards.len(),
        4,
        "draw command should contain 4 cards for 4 energy spent"
    );

    for card in &draw_command.cards {
        assert!(card.revealed.is_some(), "drawn cards should be revealed in the animation");
    }
}

#[test]
fn battle_preview_shows_energy_changes_for_incremental_spending() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(10).build()).connect();
    let starting_energy = s.user_client.me.energy();
    let id = s.create_and_play(DisplayPlayer::User, CardName::TestVariableEnergyDraw);
    let card_cost = s.user_client.cards.get_cost(&id);

    assert!(
        matches!(s.user_client.preview, Some(BattlePreviewState::Active(_))),
        "battle preview should be active when energy prompt is shown"
    );

    if let Some(BattlePreviewState::Active(preview)) = &s.user_client.preview {
        let preview_energy = preview.user.energy.expect("preview should show user energy");
        assert_eq!(
            preview_energy,
            starting_energy - card_cost - Energy(1),
            "initial preview should show energy after card cost and minimum spend"
        );
    }

    s.click_increment_button(DisplayPlayer::User);

    if let Some(BattlePreviewState::Active(preview)) = &s.user_client.preview {
        let preview_energy = preview.user.energy.expect("preview should show user energy");
        assert_eq!(
            preview_energy,
            starting_energy - card_cost - Energy(2),
            "preview should show energy after card cost and 2 additional energy"
        );
    }

    s.click_increment_button(DisplayPlayer::User);

    if let Some(BattlePreviewState::Active(preview)) = &s.user_client.preview {
        let preview_energy = preview.user.energy.expect("preview should show user energy");
        assert_eq!(
            preview_energy,
            starting_energy - card_cost - Energy(3),
            "preview should show energy after card cost and 3 additional energy"
        );
    }

    s.click_decrement_button(DisplayPlayer::User);

    if let Some(BattlePreviewState::Active(preview)) = &s.user_client.preview {
        let preview_energy = preview.user.energy.expect("preview should show user energy");
        assert_eq!(
            preview_energy,
            starting_energy - card_cost - Energy(2),
            "preview should show energy after decrementing back to 2 additional energy"
        );
    }

    s.click_primary_button(DisplayPlayer::User, "Spend");

    assert!(
        matches!(s.user_client.preview, Some(BattlePreviewState::None)),
        "battle preview should be cleared after confirming action"
    );

    assert_eq!(
        s.user_client.me.energy(),
        starting_energy - card_cost - Energy(2),
        "user should have spent card cost plus 2 additional energy"
    );

    assert_eq!(
        s.user_client.cards.user_hand().len(),
        2,
        "user should have drawn 2 cards for 2 additional energy spent"
    );
}

#[test]
fn energy_prompt_rules_text_shows_energy_spent_on_stack() {
    let mut s = TestBattle::builder().connect();
    s.add_to_hand(DisplayPlayer::Enemy, CardName::TestCounterspell);

    let energy_prompt_id = s.create_and_play(DisplayPlayer::User, CardName::TestVariableEnergyDraw);
    s.click_increment_button(DisplayPlayer::User);
    s.click_increment_button(DisplayPlayer::User);
    s.click_primary_button(DisplayPlayer::User, "Spend");

    assert!(
        s.user_client.cards.stack_cards().contains(&energy_prompt_id),
        "Energy prompt should be on the stack"
    );

    let rules_text = s.user_client.cards.get_revealed(&energy_prompt_id).rules_text.clone();

    assert!(
        rules_text.contains(&format!("(3{} paid)", icon::ENERGY)),
        "Rules text should show 3 energy paid in parentheses, but got: '{rules_text}'"
    );
}
