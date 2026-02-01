//! Round-trip tests for triggered abilities.
//!
//! Tests abilities that trigger on events like "When you...",
//! "Once per turn, when...", "At the end of...", etc.

use parser_v2_tests::test_helpers::*;

#[test]
fn test_play_count_trigger_reclaim_self() {
    assert_round_trip(
        "When you play {cards-numeral} in a turn, {reclaim} this character.",
        "cards: 2",
    );
}

#[test]
fn test_discard_self_trigger_materialize() {
    assert_round_trip("When you discard this character, {materialize} it.", "");
}

#[test]
fn test_discard_trigger_gain_reclaim_equal_cost() {
    assert_round_trip(
        "When you discard a card, it gains {reclaim} equal to its cost this turn.",
        "",
    );
}

#[test]
fn test_discard_trigger_gain_points() {
    assert_round_trip("When you discard a card, gain {points}.", "points: 1");
}

#[test]
fn test_discard_trigger_kindle() {
    assert_round_trip("When you discard a card, {kindle}.", "k: 1");
}

#[test]
fn test_materialize_subtype_trigger_self_gains_spark() {
    assert_round_trip(
        "When you {materialize} an allied {subtype}, this character gains +{s} spark.",
        "subtype: warrior\ns: 1",
    );
}

#[test]
fn test_materialize_character_by_cost_trigger_draw() {
    assert_round_trip(
        "Once per turn, when you {materialize} a character with cost {e} or less, draw {cards}.",
        "e: 2\ncards: 1",
    );
}

#[test]
fn test_materialize_character_trigger_gain_energy() {
    assert_round_trip("Once per turn, when you {materialize} a character, gain {e}.", "e: 1");
}

#[test]
fn test_materialize_subtype_trigger_draw() {
    assert_round_trip(
        "Once per turn, when you {materialize} {a-subtype}, draw {cards}.",
        "subtype: warrior\ncards: 1",
    );
}

#[test]
fn test_play_subtype_trigger_reclaim_by_cost() {
    assert_round_trip(
        "When you play {a-subtype}, {reclaim} a random character with cost {e} or less.",
        "subtype: warrior\ne: 3",
    );
}

#[test]
fn test_play_fast_card_trigger_draw() {
    assert_round_trip("Once per turn, when you play a {fast} card, draw {cards}.", "cards: 1");
}

#[test]
fn test_play_fast_card_trigger_gain_spark() {
    assert_round_trip("When you play a {fast} card, this character gains +{s} spark.", "s: 2");
}

#[test]
fn test_play_fast_card_trigger_gain_points() {
    assert_round_trip("When you play a {fast} card, gain {points}.", "points: 1");
}

#[test]
fn test_end_of_turn_gain_energy() {
    assert_round_trip("At the end of your turn, gain {e}.", "e: 2");
}

#[test]
fn test_play_fast_character_trigger_gain_energy() {
    assert_round_trip("Characters in your hand have {fast}.", "");
    assert_round_trip("Once per turn, when you play a {fast} character, gain {e}.", "e: 1");
}

#[test]
fn test_discard_trigger_gain_energy_and_kindle() {
    assert_round_trip(
        "Once per turn, when you discard a card, Gain {e}, then {kindle}.",
        "e: 1\nk: 2",
    );
}

#[test]
fn test_materialize_subtype_trigger_gain_energy() {
    assert_round_trip(
        "When you {materialize} an allied {subtype}, gain {e}.",
        "subtype:spirit-animal\ne: 1",
    );
}

#[test]
fn test_materialize_subtype_trigger_that_character_gains_spark() {
    assert_round_trip(
        "When you {materialize} an allied {subtype}, that character gains +{s} spark.",
        "subtype:spirit-animal\ns:1",
    );
}

#[test]
fn test_materialize_character_trigger_gain_spark() {
    assert_round_trip(
        "When you {materialize} a character, this character gains +{s} spark.",
        "s: 1",
    );
}

#[test]
fn test_play_subtype_trigger_draw() {
    assert_round_trip(
        "When you play {a-subtype}, draw {cards}.",
        "subtype: spirit-animal\ncards: 1",
    );
}

#[test]
fn test_ally_banished_trigger_gain_spark() {
    assert_round_trip("When an ally is {banished}, this character gains +{s} spark.", "s: 1");
}

#[test]
fn test_ally_banished_trigger_kindle() {
    assert_round_trip("When an ally is {banished}, {kindle}.", "k: 1");
}

#[test]
fn test_ally_dissolved_trigger_draw() {
    assert_round_trip("When an ally is {dissolved}, draw {cards}.", "cards: 1");
}

#[test]
fn test_ally_dissolved_trigger_gain_energy() {
    assert_round_trip("When an ally is {dissolved}, gain {e}.", "e: 1");
}

#[test]
fn test_ally_dissolved_trigger_gain_points() {
    assert_round_trip("When an ally is {dissolved}, gain {points}.", "points: 1");
}

#[test]
fn test_ally_dissolved_trigger_gain_reclaim_for_cost() {
    assert_round_trip(
        "When an ally is {dissolved}, this character gains {reclaim-for-cost} this turn.",
        "reclaim: 1",
    );
}

#[test]
fn test_abandon_ally_trigger_gain_spark() {
    assert_round_trip("When you abandon an ally, this character gains +{s} spark.", "s: 1");
}

#[test]
fn test_abandon_allies_count_trigger_dissolve() {
    assert_round_trip(
        "When you abandon {count-allies} in a turn, {dissolve} an enemy.",
        "allies: 2",
    );
}

#[test]
fn test_abandon_character_trigger_gain_points() {
    assert_round_trip("When you abandon a character, gain {points}.", "points: 1");
}

#[test]
fn test_abandon_ally_trigger_kindle() {
    assert_round_trip("When you abandon an ally, {kindle}.", "k: 2");
}

#[test]
fn test_abandon_character_trigger_draw() {
    assert_round_trip("When you abandon a character, draw {cards}.", "cards: 1");
}

#[test]
fn test_event_to_void_trigger_gain_spark() {
    assert_round_trip(
        "When an event is put into your void, this character gains +{s} spark.",
        "s: 1",
    );
}

#[test]
fn test_play_event_trigger_gain_energy() {
    assert_round_trip("When you play an event, gain {e}.", "e: 1");
}

#[test]
fn test_play_event_trigger_foresee() {
    assert_round_trip("When you play an event, {foresee}.", "foresee: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_play_event_trigger_copy() {
    assert_round_trip("Events cost you {e} more.", "e: 2");
    assert_round_trip("When you play an event from your hand, copy it.", "e: 2");
}

#[test]
fn test_until_end_of_turn_play_event_trigger_copy() {
    assert_round_trip("Until end of turn, when you play an event, copy it.", "");
}

#[test]
fn test_materialize_character_trigger_judgment() {
    assert_round_trip(
        "When you {materialize} a character, trigger the {Judgment} ability of each ally.",
        "",
    );
}

#[test]
fn test_until_end_of_turn_play_character_trigger_draw() {
    assert_round_trip("Until end of turn, when you play a character, draw {cards}.", "cards: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_judgment_triggers_on_materialize() {
    assert_round_trip(
        "The '{Judgment}' ability of allies triggers when you {materialize} them.",
        "",
    );
}

#[test]
fn test_draw_count_in_void_trigger_reclaim() {
    assert_round_trip("When you draw {cards-numeral} in a turn, if this card is in your void, it gains {reclaim-for-cost} this turn.", "cards: 2\nreclaim: 1");
}

#[test]
fn test_play_character_trigger_materialize_figment() {
    assert_round_trip("When you play a character, {materialize} {a-figment}.", "figment: halcyon");
}

#[test]
fn test_materialize_ally_trigger_gain_energy() {
    assert_round_trip("When you {materialize} an ally, gain {e}.", "e: 1");
}

#[test]
fn test_play_card_opponent_turn_trigger_gain_spark() {
    assert_round_trip(
        "When you play a card during the opponent's turn, this character gains +{s} spark.",
        "s: 1",
    );
}

#[test]
fn test_materialize_subtype_trigger_reclaim_self() {
    assert_round_trip(
        "When you {materialize} {a-subtype}, {reclaim} this character.",
        "subtype: survivor",
    );
}

#[test]
fn test_play_subtype_trigger_put_to_void() {
    assert_round_trip(
        "When you play {a-subtype}, put the {top-n-cards} of your deck into your void.",
        "subtype: survivor\nto-void: 2",
    );
}

#[test]
fn test_until_end_of_turn_ally_leaves_trigger_gain_energy() {
    assert_round_trip("Until end of turn, when an ally leaves play, gain {e}.", "e: 2");
}
