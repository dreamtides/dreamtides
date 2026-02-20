//! Round-trip tests for static abilities.
//!
//! Tests continuous effects, cost modifications, spark modifications,
//! and other static properties of cards.

use parser_tests::test_helpers::*;

#[test]
fn test_conditional_cost_if_discarded() {
    assert_rendered_match(
        "This character costs {energy($e)} if you have discarded a card this turn.",
        "e: 1",
    );
}

#[test]
fn test_play_from_void_by_cost() {
    assert_rendered_match(
        "Once per turn, you may play a character with cost {energy($e)} or less from your void.",
        "e:2",
    );
}

#[test]
fn test_spark_equals_void_count() {
    assert_rendered_match(
        "This character's spark is equal to the number of cards in your void.",
        "",
    );
}

#[test]
fn test_prevent_dissolve_event() {
    assert_rendered_match(
        "When the opponent plays an event which could {dissolve} an ally, {prevent} that card.",
        "",
    );
}

#[test]
fn test_subtypes_have_spark_bonus() {
    assert_rendered_match("Allied {@plural subtype($t)} have +{$s} spark.", "t: Warrior\ns: 1");
}

#[test]
fn test_opponent_events_cost_more() {
    assert_rendered_match("The opponent's events cost {energy($e)} more.", "e: 1");
}

#[test]
fn test_spark_equals_subtype_count() {
    assert_rendered_match(
        "This character's spark is equal to the number of allied {@plural subtype($t)}.",
        "t: Warrior",
    );
}

#[test]
fn test_require_return_ally_to_play() {
    assert_rendered_match(
        "To play this card, return an ally with cost {energy($e)} or more to hand.",
        "e: 3",
    );
}

#[test]
fn test_characters_cost_less() {
    assert_rendered_match("Characters cost you {energy($e)} less.", "e: 2");
}

#[test]
fn test_events_cost_less() {
    assert_rendered_match("Events cost you {energy($e)} less.", "e: 1");
}

#[test]
fn test_win_if_empty_deck() {
    assert_rendered_match("When you have no cards in your deck, you win the game.", "");
}

#[test]
fn test_void_cards_have_reclaim_equal_cost() {
    assert_rendered_match("While you have {count($n)} or more cards in your void, they have {reclaim} equal to their cost.", "n: 7");
}

#[test]
fn test_subtype_in_void_allies_have_spark() {
    assert_rendered_match(
        "If this card is in your void, allied {@plural subtype($t)} have +{$s} spark.",
        "t: Survivor\ns: 2",
    );
}

#[test]
fn test_only_play_from_void() {
    assert_rendered_match("You may only play this character from your void.", "");
}

#[test]
fn test_with_allied_subtype_play_from_hand_or_void() {
    assert_rendered_match(
        "With an allied {subtype($t)}, you may play this card from your hand or void for {energy($e)}.",
        "t: Survivor\ne: 1",
    );
}

#[test]
fn test_conditional_cost_if_dissolved() {
    assert_rendered_match("{Dissolve} an enemy. Draw {cards($c)}.", "c: 1");
    assert_rendered_match(
        "This event costs {energy($e)} if a character dissolved this turn.",
        "e: 1",
    );
}

#[test]
fn test_banish_void_with_count_reclaim_self() {
    assert_rendered_match(
        "{Banish} your void with {count($n)} or more cards: {Reclaim} this character.",
        "n: 8",
    );
}

#[test]
fn test_has_all_character_types() {
    assert_rendered_match("Has all character types.", "");
}

#[test]
fn test_characters_in_hand_have_fast() {
    assert_rendered_match("Characters in your hand have {fast}.", "");
}

#[test]
fn test_events_cost_more_with_copy() {
    // Note: This is tested in combination with a triggered ability
    assert_rendered_match("Events cost you {energy($e)} more.", "e: 2");
}

#[test]
fn test_subtype_gains_spark_equal_count() {
    assert_rendered_match(
        "Each allied {subtype($t)} gains spark equal to the number of allied {@plural subtype($t)}.",
        "t: SpiritAnimal",
    );
}
