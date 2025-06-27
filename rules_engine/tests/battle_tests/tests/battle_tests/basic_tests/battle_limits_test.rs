use battle_state::actions::debug_battle_action::DebugBattleAction;
use core_data::identifiers::CardName;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use display_data::battle_view::DisplayPlayer;
use display_data::command::GameMessageType;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn hand_size_limit_exceeded_gains_energy() {
    let mut s = TestBattle::builder().connect();

    let initial_energy = s.user_client.me.energy();

    for _ in 0..9 {
        s.add_to_hand(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    }

    assert_eq!(s.user_client.cards.user_hand().len(), 9, "user should have 9 cards in hand");

    let draw_id = s.add_to_hand(DisplayPlayer::User, CardName::Dreamscatter);
    let draw_cost = s.user_client.cards.get_cost(&draw_id);

    s.play_card_from_hand(DisplayPlayer::User, &draw_id);
    s.click_increment_button(DisplayPlayer::User);
    s.click_increment_button(DisplayPlayer::User);
    s.click_primary_button(DisplayPlayer::User, "Spend");

    assert_eq!(
        s.user_client.me.energy(),
        initial_energy - draw_cost - Energy(1),
        "User should have spent dreamscatter cost and 2 energy, then gained 1 energy from hand size limit"
    );
    assert_eq!(
        s.user_client.cards.user_hand().len(),
        10,
        "User should have drawn 1 card due to hand size limit"
    );
}

#[test]
fn character_limit_exceeded_abandons_character() {
    let mut s = TestBattle::builder().connect();
    let initial_void = s.user_client.cards.user_void().len();
    // Add 8  to the battlefield
    for _ in 0..8 {
        s.add_to_battlefield(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    }
    assert_eq!(
        s.user_client.cards.user_battlefield().len(),
        8,
        "User should have 8 characters on battlefield"
    );
    let char_id = s.add_to_hand(DisplayPlayer::User, CardName::MinstrelOfFallingLight);
    s.play_card_from_hand(DisplayPlayer::User, &char_id);

    assert_eq!(
        s.user_client.cards.user_battlefield().len(),
        8,
        "User should still have 8 characters on battlefield"
    );
    assert_eq!(
        s.user_client.cards.user_void().len(),
        initial_void + 1,
        "User void should have increased by 1"
    );
}

#[test]
fn draw_more_cards_than_deck_size_replenishes_deck() {
    let mut s = TestBattle::builder().connect();

    s.perform_user_action(DebugBattleAction::SetCardsRemainingInDeck {
        player: PlayerName::One,
        cards: 2,
    });
    let deck_size_before = s.user_client.cards.user_deck().len();
    assert_eq!(deck_size_before, 2, "User deck should have 2 cards");
    let initial_hand_size = s.user_client.cards.user_hand().len();
    s.create_and_play(DisplayPlayer::User, CardName::Dreamscatter);
    s.click_increment_button(DisplayPlayer::User);
    s.click_increment_button(DisplayPlayer::User);
    s.click_increment_button(DisplayPlayer::User);
    s.click_primary_button(DisplayPlayer::User, "Spend");

    let final_hand_size = s.user_client.cards.user_hand().len();
    assert!(
        final_hand_size > initial_hand_size,
        "User should have drawn cards, initial: {}, final: {}",
        initial_hand_size,
        final_hand_size
    );

    let deck_size_after = s.user_client.cards.user_deck().len();
    assert!(
        deck_size_after > deck_size_before,
        "Deck should have been replenished with new cards, but has {} cards",
        deck_size_after
    );
}

#[test]
fn turn_limit_exceeded_ends_battle_in_draw() {
    let mut s = TestBattle::builder().connect();

    for _ in 0..25 {
        s.end_turn_remove_opponent_hand(DisplayPlayer::User);
        s.end_turn_remove_opponent_hand(DisplayPlayer::Enemy);
    }

    assert!(
        s.user_client.last_game_message == Some(GameMessageType::Defeat),
        "User should see defeat message in a draw, but got {:?}",
        s.user_client.last_game_message
    );
    assert!(
        s.enemy_client.last_game_message == Some(GameMessageType::Defeat),
        "Enemy should see defeat message in a draw, but got {:?}",
        s.enemy_client.last_game_message
    );
}
