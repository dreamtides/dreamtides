use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn prevent_dissolve_shows_looping_effect_on_battlefield() {
    let mut s = TestBattle::builder().connect();
    let character_id = s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestPreventDissolveThisTurn);

    let character_data = s.user_client.cards.get(&character_id);
    assert!(
        character_data.view.revealed.as_ref().unwrap().effects.looping_effect.is_some(),
        "Character should have a looping effect when anchored and on battlefield"
    );
}

#[test]
fn prevent_dissolve_removes_looping_effect_when_returned_to_hand() {
    let mut s = TestBattle::builder().connect();
    let character_id = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    s.create_and_play(DisplayPlayer::Enemy, CardName::TestPreventDissolveThisTurn);

    let character_data = s.enemy_client.cards.get(&character_id);
    assert!(
        character_data.view.revealed.as_ref().unwrap().effects.looping_effect.is_some(),
        "Character should have a looping effect when anchored and on battlefield"
    );

    s.end_turn_remove_opponent_hand(DisplayPlayer::Enemy);

    s.create_and_play(DisplayPlayer::User, CardName::TestReturnToHand);

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
    let character_id = s.add_to_battlefield(DisplayPlayer::User, CardName::TestVanillaCharacter);

    s.create_and_play(DisplayPlayer::User, CardName::TestPreventDissolveThisTurn);

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
