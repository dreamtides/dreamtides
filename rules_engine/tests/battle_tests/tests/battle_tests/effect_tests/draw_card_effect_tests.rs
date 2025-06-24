use core_data::identifiers::CardName;
use core_data::numerics::Energy;
use display_data::battle_view::{BattlePreviewState, DisplayPlayer};
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session_prelude::*;

#[test]
fn draw_card_for_each_energy_spent() {
    let mut s = TestBattle::builder().connect();
    let starting_energy = s.user_client.me.energy();
    let id = s.create_and_play(DisplayPlayer::User, CardName::Dreamscatter);
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
fn battle_preview_shows_energy_changes_for_incremental_spending() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(10).build()).connect();
    let starting_energy = s.user_client.me.energy();
    let id = s.create_and_play(DisplayPlayer::User, CardName::Dreamscatter);
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
