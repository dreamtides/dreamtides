use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn prevent_dissolve_in_response() {
    let mut s = TestBattle::builder().connect();
    let character_id = s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    let prevent_dissolve_id =
        s.add_to_hand(DisplayPlayer::User, CardName::TestPreventDissolveThisTurn);

    s.create_and_play(DisplayPlayer::Enemy, CardName::TestDissolve);

    assert!(
        s.user_client.cards.get_revealed(&prevent_dissolve_id).actions.can_play.is_some(),
        "user can respond with prevent dissolve"
    );

    s.play_card_from_hand(DisplayPlayer::User, &prevent_dissolve_id);

    assert_eq!(
        s.user_client.cards.user_battlefield().len(),
        1,
        "character should remain on battlefield"
    );
    assert!(
        s.user_client.cards.user_battlefield().contains(&character_id),
        "character should be protected"
    );
}

#[test]
fn prevent_dissolve_makes_character_invalid_target() {
    let mut s = TestBattle::builder().connect();
    s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    let dissolve_id = s.add_to_hand(DisplayPlayer::Enemy, CardName::TestDissolve);
    let prevent_dissolve_id =
        s.add_to_hand(DisplayPlayer::User, CardName::TestPreventDissolveThisTurn);

    assert!(
        s.enemy_client.cards.get_revealed(&dissolve_id).actions.can_play.is_some(),
        "enemy can currently play dissolve"
    );

    // Enemy adds 'draw one' to stack
    s.create_and_play(DisplayPlayer::Enemy, CardName::TestDrawOne);
    // User responds with prevent dissolve on stack
    s.play_card_from_hand(DisplayPlayer::User, &prevent_dissolve_id);
    // Pass priority, prevent dissolve resolves on stack
    s.click_primary_button(DisplayPlayer::Enemy, "Resolve");

    assert!(
        s.enemy_client.cards.get_revealed(&dissolve_id).actions.can_play.is_none(),
        "enemy cannot play dissolve, no valid targets"
    );
}

#[test]
fn prevent_dissolve_with_multiple_characters() {
    let mut s = TestBattle::builder().connect();
    let character1_id = s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);
    let character2_id = s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    let dissolve_id = s.add_to_hand(DisplayPlayer::Enemy, CardName::TestDissolve);
    let prevent_dissolve_id =
        s.add_to_hand(DisplayPlayer::User, CardName::TestPreventDissolveThisTurn);

    assert!(
        s.enemy_client.cards.get_revealed(&dissolve_id).actions.can_play.is_some(),
        "enemy can currently play dissolve"
    );

    s.create_and_play(DisplayPlayer::Enemy, CardName::TestDrawOne);

    assert!(
        s.user_client.cards.get_revealed(&prevent_dissolve_id).actions.can_play.is_some(),
        "user can play prevent dissolve card"
    );
    s.play_card_from_hand(DisplayPlayer::User, &prevent_dissolve_id);
    s.click_card(DisplayPlayer::User, &character1_id);

    s.click_primary_button(DisplayPlayer::Enemy, "Resolve");

    assert_eq!(
        s.user_client.cards.user_battlefield().len(),
        2,
        "both characters should still be on battlefield"
    );
    assert!(
        s.user_client.cards.user_battlefield().contains(&character1_id),
        "character1 should be on battlefield"
    );
    assert!(
        s.user_client.cards.user_battlefield().contains(&character2_id),
        "character2 should be on battlefield"
    );
}

#[test]
fn multiple_dissolve_effects_with_prevent_dissolve() {
    let mut s = TestBattle::builder().connect();
    let character1_id = s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);
    let character2_id = s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    let dissolve1_id = s.add_to_hand(DisplayPlayer::Enemy, CardName::TestDissolve);
    let dissolve2_id = s.add_to_hand(DisplayPlayer::Enemy, CardName::TestDissolve);
    let prevent_dissolve_id =
        s.add_to_hand(DisplayPlayer::User, CardName::TestPreventDissolveThisTurn);

    s.play_card_from_hand(DisplayPlayer::Enemy, &dissolve1_id);
    s.click_card(DisplayPlayer::Enemy, &character1_id);
    s.play_card_from_hand(DisplayPlayer::User, &prevent_dissolve_id);
    s.click_card(DisplayPlayer::User, &character1_id);
    s.play_card_from_hand(DisplayPlayer::Enemy, &dissolve2_id);
    s.click_card(DisplayPlayer::Enemy, &character2_id);

    assert_eq!(
        s.user_client.cards.user_battlefield().len(),
        1,
        "only protected character should remain"
    );
    assert!(
        s.user_client.cards.user_battlefield().contains(&character1_id),
        "protected character should remain"
    );
    assert!(
        s.user_client.cards.user_void().contains(&character2_id),
        "unprotected character should be dissolved"
    );
}

#[test]
fn prevent_dissolve_in_end_step() {
    let mut s = TestBattle::builder().connect();
    let dissolve_id = s.add_to_hand(DisplayPlayer::Enemy, CardName::TestDissolve);
    let character_id = s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);

    let other_character_1 =
        s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);
    let other_character_2 =
        s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestPreventDissolveThisTurn);
    s.click_card(DisplayPlayer::User, &character_id);
    s.click_primary_button(DisplayPlayer::Enemy, "Resolve");

    s.click_primary_button(DisplayPlayer::User, "End Turn");
    s.play_card_from_hand(DisplayPlayer::Enemy, &dissolve_id);

    assert!(
        s.enemy_client.cards.get(&other_character_1).has_on_click_action(),
        "other character 1 should be a valid target"
    );
    assert!(
        s.enemy_client.cards.get(&other_character_2).has_on_click_action(),
        "other character 2 should be a valid target"
    );
    assert!(
        !s.enemy_client.cards.get(&character_id).has_on_click_action(),
        "protected character should not be a valid target"
    );
}

#[test]
fn prevent_dissolve_expires_next_turn() {
    let mut s = TestBattle::builder().connect();
    let character_id = s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);
    s.create_and_play(DisplayPlayer::User, CardName::TestPreventDissolveThisTurn);

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    s.create_and_play(DisplayPlayer::Enemy, CardName::TestDissolve);
    assert_eq!(s.user_client.cards.user_battlefield().len(), 0, "character should be dissolved");
    assert!(s.user_client.cards.user_void().contains(&character_id), "character should be in void");
}
