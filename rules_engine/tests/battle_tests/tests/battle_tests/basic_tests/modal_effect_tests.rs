use core_data::numerics::Energy;
use display_data::battle_view::DisplayPlayer;
use tabula_ids::test_card;
use test_utils::battle::test_battle::TestBattle;
use test_utils::battle::test_player::TestPlayer;
use test_utils::session::test_session_prelude::*;

use crate::battle_tests::basic_tests::test_helpers;

#[test]
fn modal_effect_displays_browser_cards_with_correct_costs() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();
    s.create_and_play(DisplayPlayer::User, test_card::TEST_MODAL_DRAW_ONE_OR_DRAW_TWO);

    let browser_cards = s.user_client.cards.browser_cards();
    assert_eq!(browser_cards.len(), 2, "two browser cards displayed for modal choice");

    let card_1 = browser_cards.cards[0];
    let card_2 = browser_cards.cards[1];

    assert_eq!(card_1.revealed().cost, Some("1".to_string()), "first choice shows cost 1");
    assert_eq!(card_2.revealed().cost, Some("3".to_string()), "second choice shows cost 3");

    let rules_text_1 = &card_1.revealed().rules_text;
    let rules_text_2 = &card_2.revealed().rules_text;
    assert!(
        rules_text_1.contains("Draw a") || rules_text_1.contains("draw a"),
        "first choice rules text mentions drawing 1 card: {rules_text_1}"
    );
    assert!(
        rules_text_2.contains("Draw 2") || rules_text_2.contains("draw 2"),
        "second choice rules text mentions drawing 2 cards: {rules_text_2}"
    );

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn select_draw_one_effect_costs_one_energy() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(5).build()).connect();
    s.create_and_play(DisplayPlayer::User, test_card::TEST_MODAL_DRAW_ONE_OR_DRAW_TWO);

    assert_eq!(s.user_client.me.energy(), Energy(5), "no energy spent to play modal card");
    assert_eq!(s.user_client.cards.user_hand().len(), 0, "hand empty before choice");

    let browser_cards = s.user_client.cards.browser_cards();
    let draw_one_card_id = browser_cards.cards[0].id.clone();
    s.click_card(DisplayPlayer::User, &draw_one_card_id);

    assert_eq!(s.user_client.me.energy(), Energy(4), "1 energy spent for Draw 1");
    assert_eq!(s.user_client.cards.user_hand().len(), 1, "drew 1 card");
    assert_eq!(s.user_client.cards.browser_cards().len(), 0, "browser cleared");
    assert_eq!(s.user_client.cards.user_void().len(), 1, "modal card moved to void");

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn select_draw_two_effect_costs_three_energy() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(7).build()).connect();
    s.create_and_play(DisplayPlayer::User, test_card::TEST_MODAL_DRAW_ONE_OR_DRAW_TWO);

    assert_eq!(s.user_client.me.energy(), Energy(7), "no energy spent to play modal card");
    assert_eq!(s.user_client.cards.user_hand().len(), 0, "hand empty before choice");

    let browser_cards = s.user_client.cards.browser_cards();
    let draw_two_card_id = browser_cards.cards[1].id.clone();
    s.click_card(DisplayPlayer::User, &draw_two_card_id);

    assert_eq!(s.user_client.me.energy(), Energy(4), "3 energy spent for Draw 2");
    assert_eq!(s.user_client.cards.user_hand().len(), 2, "drew 2 cards");
    assert_eq!(s.user_client.cards.browser_cards().len(), 0, "browser cleared");
    assert_eq!(s.user_client.cards.user_void().len(), 1, "modal card moved to void");

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn modal_effect_choice_browser_cards_have_click_actions() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();
    s.create_and_play(DisplayPlayer::User, test_card::TEST_MODAL_DRAW_ONE_OR_DRAW_TWO);

    let browser_cards = s.user_client.cards.browser_cards();
    assert_eq!(browser_cards.len(), 2, "two browser cards displayed");

    let card_1_actions = &browser_cards.cards[0].revealed().actions;
    let card_2_actions = &browser_cards.cards[1].revealed().actions;

    assert!(card_1_actions.on_click.is_some(), "first choice has click action");
    assert!(card_2_actions.on_click.is_some(), "second choice has click action");
    assert!(card_1_actions.can_play.is_none(), "first choice has no play action");
    assert!(card_2_actions.can_play.is_none(), "second choice has no play action");

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn insufficient_energy_prevents_selection() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(2).build()).connect();
    s.create_and_play(DisplayPlayer::User, test_card::TEST_MODAL_DRAW_ONE_OR_DRAW_TWO);

    assert_eq!(s.user_client.me.energy(), Energy(2), "no energy spent to play modal card");

    let browser_cards = s.user_client.cards.browser_cards();
    assert_eq!(browser_cards.len(), 2, "two browser cards displayed");

    let card_1_actions = &browser_cards.cards[0].revealed().actions;
    let card_2_actions = &browser_cards.cards[1].revealed().actions;

    assert!(card_1_actions.on_click.is_some(), "can select Draw 1 with 2 energy");
    assert!(card_2_actions.on_click.is_none(), "cannot select Draw 2 with insufficient energy");

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn modal_choices_unplayable_with_no_energy() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(0).build()).connect();
    let modal_card_id =
        s.add_to_hand(DisplayPlayer::User, test_card::TEST_MODAL_DRAW_ONE_OR_DRAW_TWO);

    let modal_card = s.user_client.cards.get(&modal_card_id);
    assert!(
        modal_card.revealed().actions.can_play.is_none(),
        "modal card itself should not be playable"
    );
}

#[test]
fn modal_draw_or_dissolve_auto_targets_single_enemy() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();
    let enemy_id = s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        1,
        "one enemy character on battlefield"
    );
    assert_eq!(s.user_client.cards.enemy_void().len(), 0, "enemy void empty");
    assert_eq!(s.user_client.cards.user_hand().len(), 0, "user hand empty");

    s.create_and_play(DisplayPlayer::User, test_card::TEST_MODAL_DRAW_ONE_OR_DISSOLVE_ENEMY);

    let browser_cards = s.user_client.cards.browser_cards();
    assert_eq!(browser_cards.len(), 2, "two browser cards displayed for modal choice");

    let dissolve_card_id = browser_cards.cards[1].id.clone();
    s.click_card(DisplayPlayer::User, &dissolve_card_id);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 0, "enemy character dissolved");
    assert_eq!(s.user_client.cards.enemy_void().len(), 1, "enemy character in void");
    assert!(s.user_client.cards.enemy_void().contains(&enemy_id), "correct enemy dissolved");
    assert_eq!(s.user_client.cards.user_void().len(), 1, "modal card in user void");

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn modal_draw_or_dissolve_manual_targeting_multiple_enemies() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();
    let enemy1_id = s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);
    let enemy2_id = s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        2,
        "two enemy characters on battlefield"
    );
    assert_eq!(s.user_client.cards.enemy_void().len(), 0, "enemy void empty");

    s.create_and_play(DisplayPlayer::User, test_card::TEST_MODAL_DRAW_ONE_OR_DISSOLVE_ENEMY);

    let browser_cards = s.user_client.cards.browser_cards();
    assert_eq!(browser_cards.len(), 2, "two browser cards displayed for modal choice");

    let dissolve_card_id = browser_cards.cards[1].id.clone();
    s.click_card(DisplayPlayer::User, &dissolve_card_id);

    s.click_card(DisplayPlayer::User, &enemy1_id);

    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "one enemy character remains");
    assert_eq!(s.user_client.cards.enemy_void().len(), 1, "one enemy character dissolved");
    assert!(s.user_client.cards.enemy_void().contains(&enemy1_id), "correct enemy dissolved");
    assert!(s.user_client.cards.enemy_battlefield().contains(&enemy2_id), "other enemy remains");

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn modal_draw_or_dissolve_draw_option_no_targeting() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();
    let enemy_id = s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

    assert_eq!(s.user_client.cards.user_hand().len(), 0, "user hand empty");
    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "enemy still on battlefield");

    s.create_and_play(DisplayPlayer::User, test_card::TEST_MODAL_DRAW_ONE_OR_DISSOLVE_ENEMY);

    let browser_cards = s.user_client.cards.browser_cards();
    assert_eq!(browser_cards.len(), 2, "two browser cards displayed for modal choice");

    let draw_card_id = browser_cards.cards[0].id.clone();
    s.click_card(DisplayPlayer::User, &draw_card_id);

    assert_eq!(s.user_client.cards.user_hand().len(), 1, "drew one card");
    assert_eq!(s.user_client.cards.enemy_battlefield().len(), 1, "enemy character untouched");
    assert!(
        s.user_client.cards.enemy_battlefield().contains(&enemy_id),
        "enemy still on battlefield"
    );
    assert_eq!(s.user_client.cards.browser_cards().len(), 0, "browser cleared");
    assert_eq!(s.user_client.cards.user_void().len(), 1, "modal card in user void");

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn modal_draw_or_dissolve_costs_and_targeting() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();
    s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);
    s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

    s.create_and_play(DisplayPlayer::User, test_card::TEST_MODAL_DRAW_ONE_OR_DISSOLVE_ENEMY);

    let browser_cards = s.user_client.cards.browser_cards();
    assert_eq!(browser_cards.len(), 2, "two browser cards displayed for modal choice");

    let draw_card = &browser_cards.cards[0];
    let dissolve_card = &browser_cards.cards[1];

    assert_eq!(draw_card.revealed().cost, Some("1".to_string()), "draw option costs 1 energy");
    assert_eq!(
        dissolve_card.revealed().cost,
        Some("2".to_string()),
        "dissolve option costs 2 energy"
    );

    let draw_rules_text = &draw_card.revealed().rules_text;
    let dissolve_rules_text = &dissolve_card.revealed().rules_text;

    assert!(
        draw_rules_text.contains("Draw") || draw_rules_text.contains("draw"),
        "draw option rules text mentions drawing: {draw_rules_text}"
    );
    assert!(
        dissolve_rules_text.contains("Dissolve") || dissolve_rules_text.contains("dissolve"),
        "dissolve option rules text mentions dissolving: {dissolve_rules_text}"
    );

    test_helpers::assert_clients_identical(&s);
}

#[test]
fn modal_dissolve_choice_unavailable_without_targets() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    s.create_and_play(DisplayPlayer::User, test_card::TEST_MODAL_DRAW_ONE_OR_DISSOLVE_ENEMY);

    let browser_cards = s.user_client.cards.browser_cards();
    assert_eq!(browser_cards.len(), 2, "two browser cards displayed for modal choice");

    let draw_card = &browser_cards.cards[0];
    let dissolve_card = &browser_cards.cards[1];

    assert!(draw_card.revealed().actions.on_click.is_some(), "draw option should be clickable");
    assert!(
        dissolve_card.revealed().actions.on_click.is_none(),
        "dissolve option should not be clickable without valid targets"
    );
}

#[test]
fn modal_dissolve_choice_available_with_targets() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();
    s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

    s.create_and_play(DisplayPlayer::User, test_card::TEST_MODAL_DRAW_ONE_OR_DISSOLVE_ENEMY);

    let browser_cards = s.user_client.cards.browser_cards();
    assert_eq!(browser_cards.len(), 2, "two browser cards displayed for modal choice");

    let draw_card = &browser_cards.cards[0];
    let dissolve_card = &browser_cards.cards[1];

    assert!(draw_card.revealed().actions.on_click.is_some(), "draw option should be clickable");
    assert!(
        dissolve_card.revealed().actions.on_click.is_some(),
        "dissolve option should be clickable with valid targets"
    );
}

#[test]
fn modal_choice_energy_and_target_validation() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(1).build()).connect();

    s.create_and_play(DisplayPlayer::User, test_card::TEST_MODAL_DRAW_ONE_OR_DISSOLVE_ENEMY);

    let browser_cards = s.user_client.cards.browser_cards();
    assert_eq!(browser_cards.len(), 2, "two browser cards displayed for modal choice");

    let draw_card = &browser_cards.cards[0];
    let dissolve_card = &browser_cards.cards[1];

    assert!(
        draw_card.revealed().actions.on_click.is_some(),
        "draw option should be clickable with 1 energy"
    );
    assert!(
        dissolve_card.revealed().actions.on_click.is_none(),
        "dissolve option should not be clickable due to insufficient energy AND no targets"
    );
}

#[test]
fn modal_effect_choices_only_visible_to_prompted_player() {
    let mut s = TestBattle::builder().user(TestPlayer::builder().energy(99).build()).connect();

    // User plays a modal card that creates a prompt
    s.create_and_play(DisplayPlayer::User, test_card::TEST_MODAL_DRAW_ONE_OR_DRAW_TWO);

    // Verify the user can see the modal choices
    let user_browser_cards = s.user_client.cards.browser_cards();
    assert_eq!(user_browser_cards.len(), 2, "User should see 2 modal choice cards");

    // Verify the enemy (opponent) cannot see the modal choices
    let enemy_browser_cards = s.enemy_client.cards.browser_cards();
    assert_eq!(enemy_browser_cards.len(), 0, "Enemy should not see any modal choice cards");
}
