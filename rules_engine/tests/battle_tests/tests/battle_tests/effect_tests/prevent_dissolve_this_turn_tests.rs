use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

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
    // Two characters in play, one protected by prevent dissolve, one not
}

#[test]
fn dissolve_on_stack_fails_when_prevent_dissolve_resolves_first() {
    // Enemy plays dissolve, then user plays prevent dissolve, character is
    // protected (LIFO order)
}

#[test]
fn prevent_dissolve_resolves_after_dissolve_on_stack() {
    // User plays dissolve, then enemy plays prevent dissolve, character is
    // dissolved (LIFO order)
}

#[test]
fn multiple_dissolve_effects_on_stack_all_fail() {
    // Multiple dissolve effects on stack all fail when prevent dissolve
}

#[test]
fn prevent_dissolve_expires_next_turn() {
    // Protect this turn, dissolve next turn, character is dissolved
}
