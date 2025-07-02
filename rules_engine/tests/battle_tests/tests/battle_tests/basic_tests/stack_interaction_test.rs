use battle_state::actions::battle_actions::BattleAction;
use core_data::identifiers::CardName;
use display_data::battle_view::DisplayPlayer;
use display_data::card_view::ClientCardId;
use display_data::command::GameObjectId;
use test_utils::battle::test_battle::TestBattle;
use test_utils::session::test_session::TestSession;
use test_utils::session::test_session_prelude::*;

#[test]
fn negate_card_on_stack() {
    let mut s = TestBattle::builder().connect();
    let negate_id = s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspell);
    s.end_turn_remove_opponent_hand(DisplayPlayer::User);
    let enemy_character_id =
        s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

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

    let user_counterspell1 = s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspell);
    let user_counterspell2 = s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspell);
    let _user_counterspell3 = s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspell);
    let enemy_counterspell1 = s.add_to_hand(DisplayPlayer::Enemy, CardName::TestCounterspell);
    let enemy_counterspell2 = s.add_to_hand(DisplayPlayer::Enemy, CardName::TestCounterspell);
    let _enemy_counterspell3 = s.add_to_hand(DisplayPlayer::Enemy, CardName::TestCounterspell);

    let enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);

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
    s.select_target(DisplayPlayer::User, &enemy_counterspell1);
    assert!(
        s.user_client.cards.stack_cards().contains(&user_counterspell2),
        "second user counterspell on stack"
    );
    assert_eq!(s.user_client.cards.stack_cards().len(), 4, "four cards on stack");
    assert!(s.user_client.opponent.can_act(), "enemy can act again");

    s.play_card_from_hand(DisplayPlayer::Enemy, &enemy_counterspell2);
    s.select_target(DisplayPlayer::Enemy, &user_counterspell2);
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

    let user_counterspell1 = s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspell);
    let user_counterspell2 = s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspell);
    let _user_extra = s.add_to_hand(DisplayPlayer::User, CardName::TestCounterspell);
    let enemy_variable = s.add_to_hand(DisplayPlayer::Enemy, CardName::TestVariableEnergyDraw);
    let _enemy_extra = s.add_to_hand(DisplayPlayer::Enemy, CardName::TestCounterspell);

    let enemy_character = s.create_and_play(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);
    assert_eq!(s.user_client.cards.stack_cards().len(), 1, "one card on stack");

    s.play_card_from_hand(DisplayPlayer::User, &user_counterspell1);
    assert_eq!(s.user_client.cards.stack_cards().len(), 2, "two cards on stack");

    s.play_card_from_hand(DisplayPlayer::Enemy, &enemy_variable);
    s.click_primary_button(DisplayPlayer::Enemy, "Spend");
    assert_eq!(s.user_client.cards.stack_cards().len(), 3, "three cards on stack");

    s.play_card_from_hand(DisplayPlayer::User, &user_counterspell2);
    s.select_target(DisplayPlayer::User, &enemy_character);
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
    let dissolve1 = s.add_to_hand(DisplayPlayer::User, CardName::TestDissolve);
    let dissolve2 = s.add_to_hand(DisplayPlayer::User, CardName::TestDissolve);
    let _extra = s.add_to_hand(DisplayPlayer::User, CardName::TestVariableEnergyDraw);
    let draw = s.add_to_hand(DisplayPlayer::Enemy, CardName::TestVariableEnergyDraw);
    let _extra2 = s.add_to_hand(DisplayPlayer::Enemy, CardName::TestVariableEnergyDraw);

    let character = s.add_to_battlefield(DisplayPlayer::Enemy, CardName::TestVanillaCharacter);
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

    assert!(arrow_exists, "Expected arrow from {} to {}", source_card_id, target_card_id);
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
        .unwrap_or_else(|| panic!("Source card {} not found", source_card_id));

    let has_targeting = source_card
        .view
        .revealed
        .as_ref()
        .and_then(|revealed| revealed.info_zoom_data.as_ref())
        .map(|info_zoom| info_zoom.icons.iter().any(|icon| &icon.card_id == target_card_id))
        .unwrap_or(false);

    assert!(
        has_targeting,
        "Expected info zoom targeting from {} to {}",
        source_card_id, target_card_id
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

    assert!(!arrow_exists, "Expected no arrow from {} to {}", source_card_id, target_card_id);
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
        .unwrap_or_else(|| panic!("Source card {} not found", source_card_id));

    let has_targeting = source_card
        .view
        .revealed
        .as_ref()
        .and_then(|revealed| revealed.info_zoom_data.as_ref())
        .map(|info_zoom| info_zoom.icons.iter().any(|icon| &icon.card_id == target_card_id))
        .unwrap_or(false);

    assert!(
        !has_targeting,
        "Expected no info zoom targeting from {} to {}",
        source_card_id, target_card_id
    );
}
