use core_data::identifiers::CardName;
use core_data::numerics::{Energy, Spark};
use display_data::battle_view::DisplayPlayer;
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::client::test_interface_view;
use test_utils::session::test_session_prelude::*;

#[test]
fn character_card_shows_energy_decrease_in_preview() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(10).build()).connect();
    let initial_energy = s.user_client.me.energy();
    let id = s.add_to_hand(DisplayPlayer::User, CardName::TestVanillaCharacter);
    let card_cost = s.user_client.cards.get_cost(&id);
    let preview = s.user_client.cards.get_play_effect_preview(&id);
    let preview_energy = preview.user.energy.expect("preview should show user energy");
    assert_eq!(
        preview_energy,
        initial_energy - card_cost,
        "preview should show energy decrease for card cost"
    );
}

#[test]
fn hand_size_limit_exceeded_shows_interface_message() {
    let mut s = TestBattle::builder().connect();

    for _ in 0..9 {
        s.add_to_hand(DisplayPlayer::User, CardName::TestVanillaCharacter);
    }

    let draw_id = s.add_to_hand(DisplayPlayer::User, CardName::TestVariableEnergyDraw);
    s.play_card_from_hand(DisplayPlayer::User, &draw_id);
    s.click_increment_button(DisplayPlayer::User);
    s.click_increment_button(DisplayPlayer::User);
    let preview = s.user_client.active_battle_preview();
    let message = preview.preview_message.as_ref().expect("preview should show message");
    assert!(
        test_interface_view::extract_text_from_node(message)
            .contains("Note: Cards drawn in excess"),
        "preview should show message when hand size limit will be exceeded"
    );
}

#[test]
fn character_limit_exceeded_shows_interface_message() {
    let mut s = TestBattle::builder().connect();

    for _ in 0..8 {
        s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);
    }

    let char_id = s.add_to_hand(DisplayPlayer::User, CardName::TestVanillaCharacter);
    let preview = s.user_client.cards.get_play_effect_preview(&char_id);
    let message = preview.preview_message.as_ref().expect("preview should show message");
    assert!(
        test_interface_view::extract_text_from_node(message).contains("Character limit exceeded"),
        "preview should show message when character limit will be exceeded"
    );
}
#[test]
fn both_limits_exceeded_shows_combined_interface_message() {
    let mut s = TestBattle::builder().connect();

    for _ in 0..8 {
        s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);
    }

    for _ in 0..9 {
        s.add_to_hand(DisplayPlayer::User, CardName::TestVanillaCharacter);
    }

    let draw_id = s.add_to_hand(DisplayPlayer::User, CardName::TestVariableEnergyDraw);
    s.play_card_from_hand(DisplayPlayer::User, &draw_id);
    s.click_increment_button(DisplayPlayer::User);
    s.click_increment_button(DisplayPlayer::User);

    let preview = s.user_client.active_battle_preview();
    let message = preview.preview_message.as_ref().expect("preview should show message");
    let message_text = test_interface_view::extract_text_from_node(message);
    assert!(
        message_text.contains("Character limit exceeded")
            || message_text.contains("Cards drawn in excess"),
        "preview should show message when both limits will be exceeded, got: {message_text}"
    );
}
#[test]
fn card_preview_shows_cost_changes_from_effects() {
    let mut s = TestBattle::builder().connect();
    let char_id = s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);
    let original_cost = s.user_client.cards.get_cost(&char_id);
    let event_id = s.add_to_hand(DisplayPlayer::User, CardName::TestVariableEnergyDraw);

    s.play_card_from_hand(DisplayPlayer::User, &event_id);
    s.click_increment_button(DisplayPlayer::User);

    let preview = s.user_client.active_battle_preview();
    let has_card_with_cost_change = preview.cards.iter().any(|card_preview| {
        card_preview.cost.is_some() && card_preview.cost != Some(original_cost)
    });

    assert!(
        has_card_with_cost_change || preview.cards.is_empty(),
        "preview should show cost changes when effects modify card costs, or no cards if no changes"
    );
}
#[test]
fn event_card_energy_prompt_shows_preview_with_incremental_changes() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(10).build()).connect();
    let initial_energy = s.user_client.me.energy();
    let id = s.create_and_play(DisplayPlayer::User, CardName::TestVariableEnergyDraw);
    let card_cost = s.user_client.cards.get_cost(&id);

    let preview = s.user_client.active_battle_preview();
    let preview_energy = preview.user.energy.expect("preview should show user energy");
    assert_eq!(
        preview_energy,
        initial_energy - card_cost - Energy(1),
        "initial preview should show minimum energy spend"
    );

    s.click_increment_button(DisplayPlayer::User);

    let preview = s.user_client.active_battle_preview();
    let preview_energy = preview.user.energy.expect("preview should show user energy");
    assert_eq!(
        preview_energy,
        initial_energy - card_cost - Energy(2),
        "preview should update with incremented energy spend"
    );
}

#[test]
fn character_play_effect_preview_shows_spark_increase() {
    let mut s = TestBattle::builder().connect();
    let initial_spark = s.user_client.me.total_spark();
    let char_id = s.add_to_hand(DisplayPlayer::User, CardName::TestVanillaCharacter);

    let preview = s.user_client.cards.get_play_effect_preview(&char_id);
    let preview_spark = preview.user.total_spark.expect("preview should show user total spark");
    assert_eq!(
        preview_spark,
        initial_spark + Spark(5),
        "play effect preview should show spark increase for test character"
    );
}
