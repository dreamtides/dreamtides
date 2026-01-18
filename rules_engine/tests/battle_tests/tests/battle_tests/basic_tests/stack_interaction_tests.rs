use battle_state::actions::battle_actions::BattleAction;
use core_data::display_color;
use display_data::battle_view::DisplayPlayer;
use display_data::card_view::ClientCardId;
use display_data::command::{ArrowStyle, GameObjectId};
use tabula_ids::test_card;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session::TestSession;
use test_utils::session::test_session_prelude::*;

#[test]
fn negate_card_on_stack() {
    let mut s = TestBattle::builder().connect();
    let negate_id = s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    let enemy_character_id =
        s.create_and_play(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_character_id),
        "enemy character on stack"
    );
    assert!(!s.user_client.opponent.can_act(), "enemy cannot act");
    assert!(s.user_client.me.can_act(), "user can act");
    s.play_card_from_hand(DisplayPlayer::User, &negate_id);
    assert_eq!(s.user_client.cards.user_hand().len(), 0, "card removed from hand");
    assert_eq!(
        s.user_client.cards.enemy_battlefield().len(),
        0,
        "card not present on enemy battlefield"
    );
    assert!(
        s.user_client.cards.enemy_void().contains(&enemy_character_id),
        "enemy character in void"
    );
    assert!(s.user_client.cards.user_void().contains(&negate_id), "negate in user void");
}

#[test]
fn stack_back_and_forth_with_targeting() {
    let mut s = TestBattle::builder().connect();
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let user_counterspell1 = s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL);
    let user_counterspell2 = s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL);
    let _user_counterspell3 = s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL);
    let enemy_counterspell1 = s.add_to_hand(DisplayPlayer::Enemy, test_card::TEST_COUNTERSPELL);
    let enemy_counterspell2 = s.add_to_hand(DisplayPlayer::Enemy, test_card::TEST_COUNTERSPELL);
    let _enemy_counterspell3 = s.add_to_hand(DisplayPlayer::Enemy, test_card::TEST_COUNTERSPELL);

    let enemy_character =
        s.create_and_play(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_character),
        "enemy character on stack"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "one card on stack");
    assert!(s.user_client.me.can_act(), "user can act");

    s.play_card_from_hand(DisplayPlayer::User, &user_counterspell1);
    assert!(
        s.user_client.cards.stack_cards().contains(&user_counterspell1),
        "counterspell on stack"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 2, "two cards on stack");
    assert!(s.user_client.opponent.can_act(), "enemy can act after counterspell");

    s.play_card_from_hand(DisplayPlayer::Enemy, &enemy_counterspell1);
    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_counterspell1),
        "enemy counterspell on stack"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 3, "three cards on stack");
    assert!(s.user_client.me.can_act(), "user can act again");

    s.play_card_from_hand(DisplayPlayer::User, &user_counterspell2);
    s.click_card(DisplayPlayer::User, &enemy_counterspell1);
    assert!(
        s.user_client.cards.stack_cards().contains(&user_counterspell2),
        "second user counterspell on stack"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 4, "four cards on stack");
    assert!(s.user_client.opponent.can_act(), "enemy can act again");

    s.play_card_from_hand(DisplayPlayer::Enemy, &enemy_counterspell2);
    s.click_card(DisplayPlayer::Enemy, &user_counterspell2);
    assert!(
        s.user_client.cards.stack_cards().contains(&enemy_counterspell2),
        "second enemy counterspell on stack"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 5, "five cards on stack");

    assert_arrow_between_cards(&s, &user_counterspell1, &enemy_character);
    assert_arrow_between_cards(&s, &enemy_counterspell1, &user_counterspell1);
    assert_arrow_between_cards(&s, &user_counterspell2, &enemy_counterspell1);
    assert_arrow_between_cards(&s, &enemy_counterspell2, &user_counterspell2);

    assert_info_zoom_targeting(&s, &user_counterspell1, &enemy_character);
    assert_info_zoom_targeting(&s, &enemy_counterspell1, &user_counterspell1);
    assert_info_zoom_targeting(&s, &user_counterspell2, &enemy_counterspell1);
    assert_info_zoom_targeting(&s, &enemy_counterspell2, &user_counterspell2);

    s.perform_user_action(BattleAction::PassPriority);

    assert!(s.user_client.opponent.can_act(), "enemy can act after their card resolves");
    assert!(
        s.user_client.cards.enemy_void().contains(&enemy_counterspell2),
        "enemy counterspell2 resolved to void"
    );
    assert!(
        s.user_client.cards.user_void().contains(&user_counterspell2),
        "user counterspell2 negated to void"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 3, "three cards after two resolve");

    assert_arrow_between_cards(&s, &user_counterspell1, &enemy_character);
    assert_arrow_between_cards(&s, &enemy_counterspell1, &user_counterspell1);

    assert_info_zoom_targeting(&s, &user_counterspell1, &enemy_character);
    assert_info_zoom_targeting(&s, &enemy_counterspell1, &user_counterspell1);

    s.perform_enemy_action(BattleAction::PassPriority);

    s.user_client.cards.stack_cards().print_ids();
    s.user_client.cards.user_void().print_ids();
    s.user_client.cards.enemy_void().print_ids();
    s.user_client.cards.enemy_battlefield().print_ids();

    assert!(
        s.user_client.cards.enemy_void().contains(&enemy_counterspell1),
        "enemy counterspell1 resolved to void"
    );
    assert!(
        s.user_client.cards.user_void().contains(&user_counterspell1),
        "user counterspell1 negated to void"
    );
    assert!(
        s.user_client.cards.enemy_battlefield().contains(&enemy_character),
        "enemy character resolved on battlefield"
    );
}

#[test]
fn resolve_negate_with_removed_target() {
    let mut s = TestBattle::builder().connect();
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);

    let user_counterspell1 = s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL);
    let user_counterspell2 = s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL);
    let _user_extra = s.add_to_hand(DisplayPlayer::User, test_card::TEST_COUNTERSPELL);
    let enemy_variable = s.add_to_hand(DisplayPlayer::Enemy, test_card::TEST_VARIABLE_ENERGY_DRAW);
    let _enemy_extra = s.add_to_hand(DisplayPlayer::Enemy, test_card::TEST_COUNTERSPELL);

    let enemy_character =
        s.create_and_play(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "one card on stack");

    s.play_card_from_hand(DisplayPlayer::User, &user_counterspell1);
    assert_eq!(s.user_client.cards.stack_cards().len(), 2, "two cards on stack");

    s.play_card_from_hand(DisplayPlayer::Enemy, &enemy_variable);
    s.click_primary_button(DisplayPlayer::Enemy, "Spend");
    assert_eq!(s.user_client.cards.stack_cards().len(), 3, "three cards on stack");

    s.play_card_from_hand(DisplayPlayer::User, &user_counterspell2);
    s.click_card(DisplayPlayer::User, &enemy_character);
    assert_eq!(s.user_client.cards.stack_cards().len(), 4, "four cards on stack");

    assert_arrow_between_cards(&s, &user_counterspell1, &enemy_character);
    assert_arrow_between_cards(&s, &user_counterspell2, &enemy_character);

    assert_info_zoom_targeting(&s, &user_counterspell1, &enemy_character);
    assert_info_zoom_targeting(&s, &user_counterspell2, &enemy_character);

    s.perform_enemy_action(BattleAction::PassPriority);

    assert!(
        s.user_client.cards.user_void().contains(&user_counterspell2),
        "user counterspell2 resolved to void"
    );
    assert!(
        s.user_client.cards.enemy_void().contains(&enemy_character),
        "enemy character removed to void"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 2, "two cards left on stack");
    assert!(s.user_client.me.can_act(), "user has priority after counterspell2 resolves");

    assert_no_arrow_between_cards(&s, &user_counterspell1, &enemy_character);
    assert_no_info_zoom_targeting(&s, &user_counterspell1, &enemy_character);

    s.perform_user_action(BattleAction::PassPriority);

    assert!(
        s.user_client.cards.enemy_void().contains(&enemy_variable),
        "enemy fast card resolved to void"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "one card left on stack");
    assert!(s.user_client.opponent.can_act(), "enemy has priority after enemy fast card resolves");

    s.perform_enemy_action(BattleAction::PassPriority);

    assert!(
        s.user_client.cards.user_void().contains(&user_counterspell1),
        "user counterspell1 resolved to void with no effect"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "stack is empty");
}

#[test]
fn resolve_dissolve_with_removed_target() {
    let mut s = TestBattle::builder().connect();
    let dissolve1 = s.add_to_hand(DisplayPlayer::User, test_card::TEST_DISSOLVE);
    let dissolve2 = s.add_to_hand(DisplayPlayer::User, test_card::TEST_DISSOLVE);
    let _extra = s.add_to_hand(DisplayPlayer::User, test_card::TEST_VARIABLE_ENERGY_DRAW);
    let draw = s.add_to_hand(DisplayPlayer::Enemy, test_card::TEST_VARIABLE_ENERGY_DRAW);
    let _extra2 = s.add_to_hand(DisplayPlayer::Enemy, test_card::TEST_VARIABLE_ENERGY_DRAW);

    let character = s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);
    s.play_card_from_hand(DisplayPlayer::User, &dissolve1);
    s.play_card_from_hand(DisplayPlayer::Enemy, &draw);
    s.click_primary_button(DisplayPlayer::Enemy, "Spend");
    s.play_card_from_hand(DisplayPlayer::User, &dissolve2);

    assert_eq!(s.user_client.cards.stack_cards().len(), 3, "three cards on stack");
    assert!(s.user_client.opponent.can_act(), "enemy has priority");

    assert_arrow_between_cards(&s, &dissolve1, &character);
    assert_arrow_between_cards(&s, &dissolve2, &character);

    assert_info_zoom_targeting(&s, &dissolve1, &character);
    assert_info_zoom_targeting(&s, &dissolve2, &character);

    s.perform_enemy_action(BattleAction::PassPriority);

    assert!(s.user_client.me.can_act(), "user has priority after dissolve 2 resolves");
    assert_eq!(s.user_client.cards.stack_cards().len(), 2, "two cards on stack");
    assert!(s.user_client.cards.user_void().contains(&dissolve2), "dissolve 2 resolved to void");
    assert!(s.user_client.cards.enemy_void().contains(&character), "character dissolved to void");

    assert_no_arrow_between_cards(&s, &dissolve1, &character);
    assert_no_info_zoom_targeting(&s, &dissolve1, &character);

    s.perform_user_action(BattleAction::PassPriority);

    assert!(s.user_client.opponent.can_act(), "enemy has priority after draw resolves");
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "one card on stack");
    assert!(s.user_client.cards.enemy_void().contains(&draw), "draw resolved to void");

    s.perform_enemy_action(BattleAction::PassPriority);

    assert!(s.user_client.cards.user_void().contains(&dissolve1), "dissolve 1 resolved to void");
    assert_eq!(s.user_client.cards.stack_cards().len(), 0, "stack is empty");
}

fn assert_arrow_between_cards(
    s: &TestSession,
    source_card_id: &ClientCardId,
    target_card_id: &ClientCardId,
) {
    let arrow_exists = s.user_client.arrows.iter().any(|arrow| {
        matches!(&arrow.source, GameObjectId::CardId(id) if id == source_card_id)
            && matches!(&arrow.target, GameObjectId::CardId(id) if id == target_card_id)
    });

    assert!(arrow_exists, "Expected arrow from {source_card_id} to {target_card_id}");
}

fn assert_info_zoom_targeting(
    s: &TestSession,
    source_card_id: &ClientCardId,
    target_card_id: &ClientCardId,
) {
    let source_card = s
        .user_client
        .cards
        .card_map
        .get(source_card_id)
        .unwrap_or_else(|| panic!("Source card {source_card_id} not found"));

    let has_targeting = source_card
        .view
        .revealed
        .as_ref()
        .and_then(|revealed| revealed.info_zoom_data.as_ref())
        .map(|info_zoom| info_zoom.icons.iter().any(|icon| &icon.card_id == target_card_id))
        .unwrap_or(false);

    assert!(
        has_targeting,
        "Expected info zoom targeting from {source_card_id} to {target_card_id}"
    );
}

fn assert_no_arrow_between_cards(
    s: &TestSession,
    source_card_id: &ClientCardId,
    target_card_id: &ClientCardId,
) {
    let arrow_exists = s.user_client.arrows.iter().any(|arrow| {
        matches!(&arrow.source, GameObjectId::CardId(id) if id == source_card_id)
            && matches!(&arrow.target, GameObjectId::CardId(id) if id == target_card_id)
    });

    assert!(!arrow_exists, "Expected no arrow from {source_card_id} to {target_card_id}");
}

fn assert_no_info_zoom_targeting(
    s: &TestSession,
    source_card_id: &ClientCardId,
    target_card_id: &ClientCardId,
) {
    let source_card = s
        .user_client
        .cards
        .card_map
        .get(source_card_id)
        .unwrap_or_else(|| panic!("Source card {source_card_id} not found"));

    let has_targeting = source_card
        .view
        .revealed
        .as_ref()
        .and_then(|revealed| revealed.info_zoom_data.as_ref())
        .map(|info_zoom| info_zoom.icons.iter().any(|icon| &icon.card_id == target_card_id))
        .unwrap_or(false);

    assert!(
        !has_targeting,
        "Expected no info zoom targeting from {source_card_id} to {target_card_id}"
    );
}

#[test]
fn prevent_dissolve_this_turn_shows_green_arrows() {
    let mut s = TestBattle::builder().connect();
    let user_character =
        s.add_to_battlefield(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    let prevent_dissolve_card =
        s.add_to_hand(DisplayPlayer::User, test_card::TEST_PREVENT_DISSOLVE_THIS_TURN);

    // Give opponent a fast card so they can respond, keeping the card on stack
    let _enemy_fast_card = s.add_to_hand(DisplayPlayer::Enemy, test_card::TEST_DRAW_ONE);

    s.play_card_from_hand(DisplayPlayer::User, &prevent_dissolve_card);
    // Target is automatically selected since there's only one valid target

    // TestPreventDissolveThisTurn should now be on stack since opponent can respond
    assert!(
        s.user_client.cards.stack_cards().contains(&prevent_dissolve_card),
        "TestPreventDissolveThisTurn card should be on stack"
    );

    // Verify that the arrow from the prevent dissolve card to user's character is
    // green
    let arrow_exists = s.user_client.arrows.iter().any(|arrow| {
        matches!(&arrow.source, display_data::command::GameObjectId::CardId(id) if id == &prevent_dissolve_card)
            && matches!(&arrow.target, display_data::command::GameObjectId::CardId(id) if id == &user_character)
            && matches!(arrow.color, display_data::command::ArrowStyle::Green)
    });

    assert!(
        arrow_exists,
        "Expected green arrow from TestPreventDissolveThisTurn to user's own character"
    );
}

#[test]
fn targeting_outline_colors_green_for_own_red_for_enemy() {
    let mut s = TestBattle::builder().connect();
    let user_character1 =
        s.add_to_battlefield(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    let user_character2 =
        s.add_to_battlefield(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    let enemy_character1 =
        s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);
    let enemy_character2 =
        s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

    // Give opponent a fast card so they can respond, keeping the card on stack
    s.add_to_hand(DisplayPlayer::Enemy, test_card::TEST_DRAW_ONE);

    // Play a TestDissolve card which targets characters - this will require user to
    // choose a target
    let dissolve_card = s.add_to_hand(DisplayPlayer::User, test_card::TEST_DISSOLVE);
    s.play_card_from_hand(DisplayPlayer::User, &dissolve_card);

    // Check that enemy characters have red outline (hostile targeting)
    let enemy1_card_view = s.user_client.cards.get(&enemy_character1);
    let enemy1_outline_color = enemy1_card_view.view.revealed.as_ref().unwrap().outline_color;
    assert!(enemy1_outline_color.is_some(), "Enemy character should have outline color");

    let enemy2_card_view = s.user_client.cards.get(&enemy_character2);
    let enemy2_outline_color = enemy2_card_view.view.revealed.as_ref().unwrap().outline_color;
    assert!(enemy2_outline_color.is_some(), "Enemy character should have outline color");

    // Verify both enemy characters have red outlines
    assert_eq!(
        enemy1_outline_color.unwrap(),
        display_color::RED_500,
        "Enemy character outline should be RED_500"
    );
    assert_eq!(
        enemy2_outline_color.unwrap(),
        display_color::RED_500,
        "Enemy character outline should be RED_500"
    );

    // Select a target to clear the targeting prompt and let the dissolve resolve
    s.click_card(DisplayPlayer::User, &enemy_character1);

    // Pass priority and let the dissolve resolve
    s.perform_enemy_action(BattleAction::PassPriority);

    // Now test with TestPreventDissolveThisTurn which targets own characters
    let prevent_dissolve_card =
        s.add_to_hand(DisplayPlayer::User, test_card::TEST_PREVENT_DISSOLVE_THIS_TURN);

    s.play_card_from_hand(DisplayPlayer::User, &prevent_dissolve_card);

    // Check that user's own characters have green outline (friendly targeting)
    let user1_card_view = s.user_client.cards.get(&user_character1);
    let user1_outline_color = user1_card_view.view.revealed.as_ref().unwrap().outline_color;
    assert!(user1_outline_color.is_some(), "User character 1 should have outline color");

    let user2_card_view = s.user_client.cards.get(&user_character2);
    let user2_outline_color = user2_card_view.view.revealed.as_ref().unwrap().outline_color;
    assert!(user2_outline_color.is_some(), "User character 2 should have outline color");

    // Verify both user characters have green outlines
    assert_eq!(
        user1_outline_color.unwrap(),
        display_color::GREEN_500,
        "User character outline should be GREEN_500"
    );
    assert_eq!(
        user2_outline_color.unwrap(),
        display_color::GREEN_500,
        "User character outline should be GREEN_500"
    );
}

#[test]
fn arrow_colors_based_on_stack_item_controller_vs_target_controller() {
    let mut s = TestBattle::builder().connect();

    // Create multiple characters for both players to force manual targeting
    let _user_character1 =
        s.add_to_battlefield(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    let _user_character2 =
        s.add_to_battlefield(DisplayPlayer::User, test_card::TEST_VANILLA_CHARACTER);
    let enemy_character1 =
        s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);
    let _enemy_character2 =
        s.add_to_battlefield(DisplayPlayer::Enemy, test_card::TEST_VANILLA_CHARACTER);

    // Give opponent a fast card so they can respond, keeping cards on stack
    s.add_to_hand(DisplayPlayer::Enemy, test_card::TEST_DRAW_ONE);

    // User plays dissolve targeting enemy character (should be red arrow)
    let user_dissolve = s.add_to_hand(DisplayPlayer::User, test_card::TEST_DISSOLVE);
    s.play_card_from_hand(DisplayPlayer::User, &user_dissolve);

    // Select a target to create the arrow
    s.click_card(DisplayPlayer::User, &enemy_character1);

    // Verify red arrow from user's card to enemy character
    let red_arrow_exists = s.user_client.arrows.iter().any(|arrow| {
        matches!(&arrow.source, GameObjectId::CardId(id) if id == &user_dissolve)
            && matches!(&arrow.target, GameObjectId::CardId(id) if id == &enemy_character1)
            && matches!(arrow.color, ArrowStyle::Red)
    });

    assert!(red_arrow_exists, "Expected red arrow from user's dissolve to enemy character");
}
