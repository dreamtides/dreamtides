use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn dissolve_targeting_prompt_with_multiple_targets() {
    let mut s = TestBattle::builder().connect();
    s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);
    s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);
    s.create_and_play(DisplayPlayer::User, CardName::TestDissolve);

    assert!(
        s.user_client.interface.screen_overlay_contains("Select an enemy character"),
        "Test dissolve should show targeting prompt when multiple enemy characters are present"
    );
}

#[test]
fn energy_additional_cost_prompt() {
    let mut s = TestBattle::builder().connect();
    s.create_and_play(DisplayPlayer::User, CardName::TestVariableEnergyDraw);

    assert!(
        s.user_client.interface.screen_overlay_contains("Pay one or more"),
        "Energy prompt should always show additional cost prompt when played"
    );
}

#[test]
fn no_prompt_for_cards_without_prompts() {
    let mut s = TestBattle::builder().connect();
    s.create_and_play(DisplayPlayer::User, CardName::TestVanillaCharacter);

    assert!(
        s.user_client.interface.screen_overlay_text().is_empty(),
        "Character should not show any prompt when played"
    );
}
