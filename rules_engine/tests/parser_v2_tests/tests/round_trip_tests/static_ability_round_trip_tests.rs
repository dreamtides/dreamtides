//! Round-trip tests for static abilities.
//!
//! Tests continuous effects, cost modifications, spark modifications,
//! and other static properties of cards.

use parser_v2_tests::test_helpers::*;

#[ignore = "Round-trip mismatch"]
#[test]
fn test_conditional_cost_if_discarded() {
    assert_round_trip("This character costs {e} if you have discarded a card this turn.", "e: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_play_from_void_by_cost() {
    assert_round_trip(
        "Once per turn, you may play a character with cost {e} or less from your void.",
        "e:2",
    );
}

#[test]
fn test_spark_equals_void_count() {
    assert_round_trip("This character's spark is equal to the number of cards in your void.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_prevent_dissolve_event() {
    assert_round_trip(
        "When the opponent plays an event which could {dissolve} an ally, {prevent} that card.",
        "",
    );
}

#[test]
fn test_subtypes_have_spark_bonus() {
    assert_round_trip("Allied {plural-subtype} have +{s} spark.", "subtype: warrior\ns: 1");
}

#[test]
fn test_opponent_events_cost_more() {
    assert_round_trip("The opponent's events cost {e} more.", "e: 1");
}

#[test]
fn test_spark_equals_subtype_count() {
    assert_round_trip(
        "This character's spark is equal to the number of allied {plural-subtype}.",
        "subtype: warrior",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_require_return_ally_to_play() {
    assert_round_trip(
        "To play this card, return an ally with cost {e} or more to hand.",
        "e: 3\ncards: 1",
    );
}

#[test]
fn test_characters_cost_less() {
    assert_round_trip("Characters cost you {e} less.", "e: 2");
}

#[test]
fn test_events_cost_less() {
    assert_round_trip("Events cost you {e} less.", "e: 1");
}

#[test]
fn test_win_if_empty_deck() {
    assert_round_trip("When you have no cards in your deck, you win the game.", "");
}

#[test]
fn test_void_cards_have_reclaim_equal_cost() {
    assert_round_trip("While you have {count} or more cards in your void, they have {reclaim} equal to their cost.", "count: 7");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_subtype_in_void_allies_have_spark() {
    assert_round_trip(
        "If this card is in your void, allied {plural-subtype} have +{s} spark.",
        "subtype: survivor\ns: 2",
    );
}

#[test]
fn test_only_play_from_void() {
    assert_round_trip("You may only play this character from your void.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_with_allied_subtype_play_from_hand_or_void() {
    assert_round_trip(
        "With an allied {subtype}, you may play this card from your hand or void for {e}.",
        "subtype: survivor\ne: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_conditional_cost_if_dissolved() {
    assert_round_trip("{Dissolve} an enemy. Draw {cards}.", "cards: 1\ne: 1");
    assert_round_trip("This event costs {e} if a character dissolved this turn.", "cards: 1\ne: 1");
}

#[test]
fn test_banish_void_with_count_reclaim_self() {
    assert_round_trip(
        "{Banish} your void with {count} or more cards: {Reclaim} this character.",
        "count: 8",
    );
}

#[test]
fn test_has_all_character_types() {
    assert_round_trip("Has all character types.", "");
}

#[test]
fn test_characters_in_hand_have_fast() {
    assert_round_trip("Characters in your hand have {fast}.", "");
}

#[test]
fn test_events_cost_more_with_copy() {
    // Note: This is tested in combination with a triggered ability
    assert_round_trip("Events cost you {e} more.", "e: 2");
}

#[test]
fn test_subtype_gains_spark_equal_count() {
    assert_round_trip(
        "Each allied {subtype} gains spark equal to the number of allied {plural-subtype}.",
        "subtype: spirit-animal",
    );
}
