use display_data::battle_view::DisplayPlayer;
use tabula_ids::test_card;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn prevent_dissolve_shows_looping_effect_on_battlefield() {
    let mut s = TestBattle::builder().connect();
    let character_id = s.add_to_battlefield(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);

    s.create_and_play(DisplayPlayer::User, test_card::TEST_PREVENT_DISSOLVE_THIS_TURN);

    let character_data = s.user_client.cards.get(&character_id);
    assert!(
        character_data.view.revealed.as_ref().unwrap().effects.looping_effect.is_some(),
        "Character should have a looping effect when anchored and on battlefield"
    );
}

#[test]
fn prevent_dissolve_removes_looping_effect_when_returned_to_hand() {
    let mut s = TestBattle::builder().connect();
    let character_id =
        s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    s.create_and_play(DisplayPlayer::Enemy, test_card::TEST_PREVENT_DISSOLVE_THIS_TURN);

    let character_data = s.enemy_client.cards.get(&character_id);
    assert!(
        character_data.view.revealed.as_ref().unwrap().effects.looping_effect.is_some(),
        "Character should have a looping effect when anchored and on battlefield"
    );

    s.end_turn_remove_opponent_hand(DisplayPlayer::Enemy);

    s.create_and_play(DisplayPlayer::User, test_card::TEST_RETURN_TO_HAND);

    let character_data = s.user_client.cards.get(&character_id);
    assert!(
        character_data.view.revealed.as_ref().unwrap().effects.looping_effect.is_none(),
        "Character should not have a looping effect when anchored but not on battlefield"
    );
    assert!(
        s.user_client.cards.enemy_hand().contains(&character_id),
        "Character should be in enemy hand"
    );
}

#[test]
fn prevent_dissolve_looping_effect_expires_next_turn() {
    let mut s = TestBattle::builder().connect();
    let character_id = s.add_to_battlefield(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);

    s.create_and_play(DisplayPlayer::User, test_card::TEST_PREVENT_DISSOLVE_THIS_TURN);

    let character_data = s.user_client.cards.get(&character_id);
    assert!(
        character_data.view.revealed.as_ref().unwrap().effects.looping_effect.is_some(),
        "Character should have a looping effect when anchored"
    );

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let character_data = s.user_client.cards.get(&character_id);
    assert!(
        character_data.view.revealed.as_ref().unwrap().effects.looping_effect.is_none(),
        "Character should not have a looping effect after anchor expires"
    );
}
