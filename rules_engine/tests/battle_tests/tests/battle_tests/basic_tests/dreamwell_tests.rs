use battle_state::actions::battle_actions::{BattleAction, CardOrderSelectionTarget};
use battle_state::actions::debug_battle_action::DebugBattleAction;
use core_data::numerics::{Energy, Points};
use core_data::types::PlayerName;
use display_data::battle_view::DisplayPlayer;
use tabula_ids::card_lists::DreamwellCardIdList;
use tabula_ids::test_card;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session_prelude::*;

#[test]
fn test_gain_energy_dreamwell() {
    let mut s =
        TestBattle::builder().with_dreamwell(DreamwellCardIdList::TestDreamwellBasic5).connect();
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    s.set_next_dreamwell_card(DisplayPlayer::Enemy, test_card::DREAMWELL_GAIN_ENERGY);
    let user_energy = s.user_client.me.energy();
    let user_produced_energy = s.user_client.me.produced_energy();
    s.end_turn_remove_opponent_hand(DisplayPlayer::Enemy);
    assert_eq!(
        s.user_client.me.energy(),
        user_energy + Energy(2),
        "Should have gained 2 more energy"
    );
    assert_eq!(
        s.user_client.me.produced_energy(),
        user_produced_energy + Energy(1),
        "Should have gained only 1 produced energy"
    );
    assert!(
        s.user_client.me.energy() > s.user_client.me.produced_energy(),
        "Should have more energy than produced energy"
    );
}

#[test]
fn test_gain_points_dreamwell() {
    let mut s =
        TestBattle::builder().with_dreamwell(DreamwellCardIdList::TestDreamwellBasic5).connect();
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    s.set_next_dreamwell_card(DisplayPlayer::Enemy, test_card::DREAMWELL_GAIN_POINTS);
    let user_points = s.user_client.me.score();
    let enemy_points = s.enemy_client.me.score();
    s.end_turn_remove_opponent_hand(DisplayPlayer::Enemy);
    assert_eq!(s.user_client.me.score(), user_points + Points(2), "Should have gained 2 points");
    assert_eq!(s.enemy_client.me.score(), enemy_points, "Enemy should not gain points");
}

#[test]
fn test_dreamwell_foresee_one() {
    let mut s =
        TestBattle::builder().with_dreamwell(DreamwellCardIdList::TestDreamwellBasic5).connect();
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    s.set_next_dreamwell_card(DisplayPlayer::Enemy, test_card::DREAMWELL_FORESEE);
    s.perform_user_action(DebugBattleAction::SetCardsRemainingInDeck {
        player: PlayerName::One,
        cards: 1,
    });
    s.end_turn_remove_opponent_hand(DisplayPlayer::Enemy);
    assert!(
        s.user_client.interface().card_order_selector.is_some(),
        "Card order selector should appear for dreamwell foresee"
    );
    let cards_in_browser: Vec<_> = s
        .user_client
        .cards
        .card_map
        .values()
        .filter(|card| {
            matches!(
                card.view.position.position,
                display_data::object_position::Position::CardOrderSelector(_)
            )
        })
        .collect();
    assert_eq!(cards_in_browser.len(), 1, "Should show exactly one card for foresee 1");
    let card_id = cards_in_browser[0].id.clone();
    s.select_card_order(DisplayPlayer::User, &card_id, CardOrderSelectionTarget::Deck(0));
    s.click_primary_button(DisplayPlayer::User, "Submit");
    assert!(
        s.user_client.interface().card_order_selector.is_none(),
        "Card order selector should close after submission"
    );
}

#[test]
fn test_dreamwell_draw_discard() {
    let mut s =
        TestBattle::builder().with_dreamwell(DreamwellCardIdList::TestDreamwellBasic5).connect();
    let discard_candidate = s.add_to_hand(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    let keep_candidate = s.add_to_hand(
        DisplayPlayer::User,
        test_card::TEST_FAST_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER,
    );
    let initial_hand_len = s.user_client.cards.user_hand().len();
    assert_eq!(initial_hand_len, 2, "Setup: two cards in hand");
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    s.set_next_dreamwell_card(DisplayPlayer::Enemy, test_card::DREAMWELL_DRAW_DISCARD);

    // End enemy turn without removing user hand to keep candidates
    s.perform_enemy_action(BattleAction::EndTurn);

    let user_hand = s.user_client.cards.user_hand();
    let target_to_discard = if user_hand.iter().any(|c| c.id == discard_candidate) {
        discard_candidate.clone()
    } else {
        panic!("Discard candidate not available during dreamwell draw-discard effect");
    };

    s.click_card(DisplayPlayer::User, &target_to_discard);
    s.click_primary_button(DisplayPlayer::User, "Submit");

    assert_eq!(
        s.user_client.cards.user_hand().len(),
        initial_hand_len + 1,
        "Final hand should be initial + 1 (start-of-turn draw + dreamwell draw - discard)"
    );
    assert!(
        !s.user_client.cards.user_hand().contains(&target_to_discard),
        "Chosen discard card removed from hand"
    );
    assert!(
        s.user_client.cards.user_void().contains(&target_to_discard),
        "Chosen discard card placed into void"
    );
    assert!(
        s.user_client.cards.user_hand().contains(&keep_candidate),
        "Kept card still present in hand"
    );
}

#[test]
fn test_dreamwell_mill_three() {
    let mut s =
        TestBattle::builder().with_dreamwell(DreamwellCardIdList::TestDreamwellBasic5).connect();
    let deck_before = s.user_client.cards.user_deck().len();
    assert!(deck_before >= 3, "Precondition: deck has at least 3 cards");
    let void_before = s.user_client.cards.user_void().len();

    // Advance to enemy turn, configure next dreamwell card to mill 3, then return
    // to user turn
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    s.set_next_dreamwell_card(DisplayPlayer::Enemy, test_card::DREAMWELL_MILL_3);
    s.end_turn_remove_opponent_hand(DisplayPlayer::Enemy);

    let deck_after = s.user_client.cards.user_deck().len();
    let void_after = s.user_client.cards.user_void().len();

    assert_eq!(deck_after, deck_before - 3, "Deck should decrease by 3 after milling");
    assert_eq!(void_after, void_before + 3, "Void should increase by 3 after milling");
}
