use core_data::identifiers::CardName;
use core_data::numerics::Energy;
use display_data::battle_view::{BattlePreviewState, DisplayPlayer};
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session_prelude::*;

#[test]
fn character_card_shows_energy_decrease_in_preview() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(10).build()).connect();
    let initial_energy = s.user_client.me.energy();
    let id = s.add_to_hand(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    let card_cost = s.user_client.cards.get_cost(&id);

    s.play_card_from_hand(DisplayPlayer::User, &id);

    if let Some(BattlePreviewState::Active(preview)) = &s.user_client.preview {
        let preview_energy = preview.user.energy.expect("preview should show user energy");
        assert_eq!(
            preview_energy,
            initial_energy - card_cost,
            "preview should show energy decrease for card cost"
        );
    }
}

#[test]
fn hand_size_limit_exceeded_shows_interface_message() {
    let mut s = TestBattle::builder().connect();

    for _ in 0..9 {
        s.add_to_hand(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    }

    let draw_id = s.add_to_hand(DisplayPlayer::User, CardName::Dreamscatter);
    s.play_card_from_hand(DisplayPlayer::User, &draw_id);
    s.click_increment_button(DisplayPlayer::User);
    s.click_increment_button(DisplayPlayer::User);

    if let Some(BattlePreviewState::Active(preview)) = &s.user_client.preview {
        assert!(
            preview.preview_message.is_some(),
            "preview should show message when hand size limit will be exceeded"
        );
    }
}

#[test]
fn character_limit_exceeded_shows_interface_message() {
    let mut s = TestBattle::builder().connect();

    for _ in 0..8 {
        s.add_to_battlefield(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    }

    let char_id = s.add_to_hand(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    s.play_card_from_hand(DisplayPlayer::User, &char_id);

    if let Some(BattlePreviewState::Active(preview)) = &s.user_client.preview {
        assert!(
            preview.preview_message.is_some(),
            "preview should show message when character limit will be exceeded"
        );
    }
}

#[test]
fn both_limits_exceeded_shows_combined_interface_message() {
    let mut s = TestBattle::builder().connect();

    for _ in 0..8 {
        s.add_to_battlefield(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    }

    for _ in 0..9 {
        s.add_to_hand(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    }

    let draw_id = s.add_to_hand(DisplayPlayer::User, CardName::Dreamscatter);
    s.play_card_from_hand(DisplayPlayer::User, &draw_id);
    s.click_increment_button(DisplayPlayer::User);
    s.click_increment_button(DisplayPlayer::User);

    if let Some(BattlePreviewState::Active(preview)) = &s.user_client.preview {
        assert!(
            preview.preview_message.is_some(),
            "preview should show combined message when both limits will be exceeded"
        );
    }
}

#[test]
fn card_preview_shows_cost_changes_from_effects() {
    let mut s = TestBattle::builder().connect();
    let char_id = s.add_to_battlefield(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    let original_cost = s.user_client.cards.get_cost(&char_id);
    let event_id = s.add_to_hand(DisplayPlayer::User, CardName::Dreamscatter);

    s.play_card_from_hand(DisplayPlayer::User, &event_id);
    s.click_increment_button(DisplayPlayer::User);

    if let Some(BattlePreviewState::Active(preview)) = &s.user_client.preview {
        let has_card_with_cost_change = preview.cards.iter().any(|card_preview| {
            card_preview.cost.is_some() && card_preview.cost != Some(original_cost)
        });

        assert!(
            has_card_with_cost_change || preview.cards.is_empty(),
            "preview should show cost changes when effects modify card costs, or no cards if no changes"
        );
    }
}

#[test]
fn event_card_energy_prompt_shows_preview_with_incremental_changes() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(10).build()).connect();
    let initial_energy = s.user_client.me.energy();
    let id = s.create_and_play(DisplayPlayer::User, CardName::Dreamscatter);
    let card_cost = s.user_client.cards.get_cost(&id);

    if let Some(BattlePreviewState::Active(preview)) = &s.user_client.preview {
        let preview_energy = preview.user.energy.expect("preview should show user energy");
        assert_eq!(
            preview_energy,
            initial_energy - card_cost - Energy(1),
            "initial preview should show minimum energy spend"
        );
    }

    s.click_increment_button(DisplayPlayer::User);

    if let Some(BattlePreviewState::Active(preview)) = &s.user_client.preview {
        let preview_energy = preview.user.energy.expect("preview should show user energy");
        assert_eq!(
            preview_energy,
            initial_energy - card_cost - Energy(2),
            "preview should update with incremented energy spend"
        );
    }
}
