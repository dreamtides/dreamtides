use core_data::identifiers::CardName;
use core_data::numerics::Energy;
use display_data::battle_view::DisplayPlayer;
use display_data::card_view::CardPrefab;
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session_prelude::*;

#[test]
fn reclaim_basic_play_twice_then_banish() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    assert_eq!(s.user_client.cards.user_hand().len(), 0, "hand empty initially");
    assert_eq!(s.user_client.cards.user_void().len(), 0, "void empty initially");
    assert_eq!(s.user_client.cards.user_banished().len(), 0, "banished empty initially");

    let card_id = s.create_and_play(DisplayPlayer::User, CardName::TestDrawOneReclaim);

    assert_eq!(
        s.user_client.cards.user_hand().len(),
        2,
        "drew card + reclaim token from first play"
    );
    assert_eq!(s.user_client.cards.user_void().len(), 1, "card in void after first play");
    assert_eq!(s.user_client.cards.user_banished().len(), 0, "not banished after first play");
    assert!(s.user_client.cards.user_void().contains(&card_id), "reclaim card in void");
    assert_eq!(s.user_client.me.energy(), Energy(97), "2 energy spent");

    let user_hand = s.user_client.cards.user_hand();
    let reclaim_token_cards: Vec<_> =
        user_hand.iter().filter(|card| card.view.prefab == CardPrefab::Token).collect();

    assert_eq!(reclaim_token_cards.len(), 1, "one reclaim token in hand");
    let token_card = &reclaim_token_cards[0];
    assert!(
        token_card.view.revealed.as_ref().unwrap().name.contains("Draw 1 Reclaim"),
        "token shows card name"
    );
    let token_cost = token_card.view.revealed.as_ref().unwrap().numeric_cost();
    assert_eq!(token_cost, Some(Energy(1)), "token shows reclaim cost");

    s.play_card_from_void(DisplayPlayer::User, &card_id);

    assert_eq!(s.user_client.cards.user_hand().len(), 2, "drew second card from reclaim");
    assert_eq!(s.user_client.cards.user_void().len(), 0, "card no longer in void after reclaim");
    assert_eq!(s.user_client.cards.user_banished().len(), 1, "card banished after reclaim");
    assert!(s.user_client.cards.user_banished().contains(&card_id), "reclaim card banished");
    assert_eq!(s.user_client.me.energy(), Energy(96), "1 energy spent");
}

#[test]
fn reclaim_multiple_cards_in_void() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let card1_id = s.create_and_play(DisplayPlayer::User, CardName::TestDrawOneReclaim);
    let card2_id = s.create_and_play(DisplayPlayer::User, CardName::TestDrawOneReclaim);

    assert_eq!(s.user_client.cards.user_void().len(), 2, "two reclaim cards in void");
    assert_eq!(s.user_client.cards.user_hand().len(), 4, "2 drawn cards + 2 reclaim tokens");

    let user_hand = s.user_client.cards.user_hand();
    let reclaim_token_cards: Vec<_> =
        user_hand.iter().filter(|card| card.view.prefab == CardPrefab::Token).collect();

    assert_eq!(reclaim_token_cards.len(), 2, "two reclaim tokens in hand");

    s.play_card_from_void(DisplayPlayer::User, &card1_id);

    assert_eq!(s.user_client.cards.user_void().len(), 1, "one card remains in void");
    assert_eq!(s.user_client.cards.user_banished().len(), 1, "first card banished");
    assert!(s.user_client.cards.user_banished().contains(&card1_id), "first card banished");
    assert!(s.user_client.cards.user_void().contains(&card2_id), "second card still in void");

    s.play_card_from_void(DisplayPlayer::User, &card2_id);

    assert_eq!(s.user_client.cards.user_void().len(), 0, "void empty after both reclaims");
    assert_eq!(s.user_client.cards.user_banished().len(), 2, "both cards banished");
    assert!(s.user_client.cards.user_banished().contains(&card2_id), "second card banished");
}

#[test]
fn reclaim_token_card_properties() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let _card_id = s.create_and_play(DisplayPlayer::User, CardName::TestDrawOneReclaim);

    let user_hand = s.user_client.cards.user_hand();
    let reclaim_token_cards: Vec<_> =
        user_hand.iter().filter(|card| card.view.prefab == CardPrefab::Token).collect();

    assert_eq!(reclaim_token_cards.len(), 1, "one reclaim token in hand");
    let token_card = &reclaim_token_cards[0];
    let revealed = token_card.view.revealed.as_ref().unwrap();

    assert!(revealed.name.contains("Draw 1 Reclaim"), "token shows original card name");
    assert_eq!(revealed.numeric_cost(), Some(Energy(1)), "token shows reclaim cost");
    assert!(revealed.rules_text.contains("Draw a card"), "token shows original rules text");
    assert!(revealed.is_fast, "reclaim card is fast");
    assert!(revealed.actions.can_play.is_some(), "token can be played");
    assert_eq!(token_card.view.prefab, CardPrefab::Token, "uses token prefab");
}

#[test]
fn reclaim_during_enemy_turn() {
    let mut s = TestBattle::builder()
        .user(TestPlayer::builder().energy(99).build())
        .enemy(TestPlayer::builder().energy(99).build())
        .connect();

    let card_id = s.create_and_play(DisplayPlayer::User, CardName::TestDrawOneReclaim);

    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let _enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

    assert!(s.user_client.me.can_act(), "user can act during enemy turn with fast card");

    let user_hand = s.user_client.cards.user_hand();
    let reclaim_token_cards: Vec<_> =
        user_hand.iter().filter(|card| card.view.prefab == CardPrefab::Token).collect();

    assert_eq!(reclaim_token_cards.len(), 1, "reclaim token available during enemy turn");
    let token_card = &reclaim_token_cards[0];
    let revealed = token_card.view.revealed.as_ref().unwrap();
    assert!(revealed.actions.can_play.is_some(), "reclaim token playable during enemy turn");

    s.play_card_from_void(DisplayPlayer::User, &card_id);

    assert_eq!(
        s.user_client.cards.user_banished().len(),
        1,
        "card banished after reclaim during enemy turn"
    );
    assert_eq!(s.user_client.me.energy(), Energy(96), "3 energy spent");
}

#[test]
fn reclaim_card_banished_when_leaves_play() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let card_id = s.create_and_play(DisplayPlayer::User, CardName::TestDrawOneReclaim);

    assert_eq!(s.user_client.cards.user_void().len(), 1, "card in void after first play");
    assert_eq!(s.user_client.cards.user_banished().len(), 0, "not banished after first play");

    s.play_card_from_void(DisplayPlayer::User, &card_id);

    assert_eq!(s.user_client.cards.user_void().len(), 0, "card leaves void when reclaimed");
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "card resolves and leaves stack");
    assert_eq!(
        s.user_client.cards.user_banished().len(),
        1,
        "card banished instead of returning to void"
    );
    assert!(s.user_client.cards.user_banished().contains(&card_id), "correct card banished");
}

#[test]
fn reclaim_vs_normal_play_from_hand_cost() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let hand_card_id = s.add_to_hand(DisplayPlayer::User, CardName::TestDrawOneReclaim);
    let void_card_id = s.add_to_void(DisplayPlayer::User, CardName::TestDrawOneReclaim);

    let hand_card = s.user_client.cards.get_revealed(&hand_card_id);
    assert_eq!(hand_card.numeric_cost(), Some(Energy(2)), "normal play cost is 2");

    let user_hand = s.user_client.cards.user_hand();
    let reclaim_token_cards: Vec<_> =
        user_hand.iter().filter(|card| card.view.prefab == CardPrefab::Token).collect();

    assert_eq!(reclaim_token_cards.len(), 1, "one reclaim token for void card");
    let token_card = &reclaim_token_cards[0];
    let revealed = token_card.view.revealed.as_ref().unwrap();
    assert_eq!(revealed.numeric_cost(), Some(Energy(1)), "reclaim cost is 1");

    s.play_card_from_hand(DisplayPlayer::User, &hand_card_id);
    assert_eq!(s.user_client.me.energy(), Energy(97), "2 energy spent on normal play");

    s.play_card_from_void(DisplayPlayer::User, &void_card_id);
    assert_eq!(s.user_client.me.energy(), Energy(96), "1 energy spent on reclaim");
}

#[test]
fn reclaim_mixed_with_other_void_cards() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    let reclaim_card_id = s.create_and_play(DisplayPlayer::User, CardName::TestDrawOneReclaim);
    let normal_card_id = s.add_to_void(DisplayPlayer::User, CardName::TestDrawOne);

    assert_eq!(s.user_client.cards.user_void().len(), 2, "two cards in void");

    let user_hand = s.user_client.cards.user_hand();
    let reclaim_token_cards: Vec<_> =
        user_hand.iter().filter(|card| card.view.prefab == CardPrefab::Token).collect();

    assert_eq!(reclaim_token_cards.len(), 1, "only one reclaim token (for reclaim card only)");

    let token_card = &reclaim_token_cards[0];
    let revealed = token_card.view.revealed.as_ref().unwrap();
    assert!(revealed.name.contains("Draw 1 Reclaim"), "token is for the reclaim card");

    s.play_card_from_void(DisplayPlayer::User, &reclaim_card_id);

    assert_eq!(s.user_client.cards.user_void().len(), 1, "normal card remains in void");
    assert_eq!(s.user_client.cards.user_banished().len(), 1, "reclaim card banished");
    assert!(s.user_client.cards.user_void().contains(&normal_card_id), "normal card still in void");
    assert!(
        s.user_client.cards.user_banished().contains(&reclaim_card_id),
        "reclaim card banished"
    );
}

#[test]
fn reclaim_token_always_in_hand_even_when_unplayable() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(2).build()).connect();

    let card_id = s.create_and_play(DisplayPlayer::User, CardName::TestDrawOneReclaim);
    assert_eq!(s.user_client.me.energy(), Energy(0), "2 energy spent on initial play");

    let user_hand = s.user_client.cards.user_hand();
    let reclaim_token_cards: Vec<_> =
        user_hand.iter().filter(|card| card.view.prefab == CardPrefab::Token).collect();

    assert_eq!(
        reclaim_token_cards.len(),
        1,
        "reclaim token still in hand despite insufficient energy"
    );
    let token_card = &reclaim_token_cards[0];
    let revealed = token_card.view.revealed.as_ref().unwrap();
    assert!(revealed.name.contains("Draw 1 Reclaim"), "token shows original card name");
    assert_eq!(revealed.numeric_cost(), Some(Energy(1)), "token shows reclaim cost");
    assert!(
        revealed.actions.can_play.is_none(),
        "token cannot be played due to insufficient energy"
    );

    assert_eq!(s.user_client.cards.user_void().len(), 1, "card still in void");
    assert!(s.user_client.cards.user_void().contains(&card_id), "reclaim card in void");
}

#[test]
fn reclaim_card_shows_reclaimed_in_rules_text_when_on_stack() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();
    let card_id = s.create_and_play(DisplayPlayer::User, CardName::TestDrawOneReclaim);
    s.add_to_hand(DisplayPlayer::Enemy, CardName::TestDrawOne);
    s.play_card_from_void(DisplayPlayer::User, &card_id);

    let stack_cards: Vec<&_> = s.user_client.cards.stack_cards().iter().map(|c| &c.view).collect();
    assert_eq!(stack_cards.len(), 1, "card is on the stack");

    let stack_card = &stack_cards[0];
    let revealed = stack_card.revealed.as_ref().unwrap();

    assert!(
        revealed.rules_text.contains("(Reclaimed)"),
        "card shows (Reclaimed) in rules text when played from void. Rules text: {}",
        revealed.rules_text
    );

    assert!(revealed.rules_text.contains("Draw a card"), "card still shows original rules text");
}
