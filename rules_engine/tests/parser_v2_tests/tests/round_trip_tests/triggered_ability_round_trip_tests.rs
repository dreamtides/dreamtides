//! Round-trip tests for triggered abilities.
//!
//! Tests abilities that trigger on events like "When you...",
//! "Once per turn, when...", "At the end of...", etc.

use parser_v2_tests::test_helpers::*;

#[test]
fn test_play_count_trigger_reclaim_self() {
    assert_round_trip(
        "When you play {cards_numeral(c)} in a turn, {reclaim} this character.",
        "c: 2",
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
    assert_round_trip("When you discard a card, gain {points(p)}.", "p: 1");
}

#[test]
fn test_discard_trigger_kindle() {
    assert_round_trip("When you discard a card, {kindle(k)}.", "k: 1");
}

#[test]
fn test_materialize_subtype_trigger_self_gains_spark() {
    assert_round_trip(
        "When you {materialize} an allied {subtype(t)}, this character gains +{s} spark.",
        "t: warrior\ns: 1",
    );
}

#[test]
fn test_materialize_character_by_cost_trigger_draw() {
    assert_round_trip(
        "Once per turn, when you {materialize} a character with cost {energy(e)} or less, draw {cards(c)}.",
        "e: 2\nc: 1",
    );
}

#[test]
fn test_materialize_character_trigger_gain_energy() {
    assert_round_trip(
        "Once per turn, when you {materialize} a character, gain {energy(e)}.",
        "e: 1",
    );
}

#[test]
fn test_materialize_subtype_trigger_draw() {
    assert_round_trip(
        "Once per turn, when you {materialize} {@a subtype(t)}, draw {cards(c)}.",
        "t: warrior\nc: 1",
    );
}

#[test]
fn test_play_subtype_trigger_reclaim_by_cost() {
    assert_round_trip(
        "When you play {@a subtype(t)}, {reclaim} a random character with cost {energy(e)} or less.",
        "t: warrior\ne: 3",
    );
}

#[test]
fn test_play_fast_card_trigger_draw() {
    assert_round_trip("Once per turn, when you play a {fast} card, draw {cards(c)}.", "c: 1");
}

#[test]
fn test_play_fast_card_trigger_gain_spark() {
    assert_round_trip("When you play a {fast} card, this character gains +{s} spark.", "s: 2");
}

#[test]
fn test_play_fast_card_trigger_gain_points() {
    assert_round_trip("When you play a {fast} card, gain {points(p)}.", "p: 1");
}

#[test]
fn test_end_of_turn_gain_energy() {
    assert_round_trip("At the end of your turn, gain {energy(e)}.", "e: 2");
}

#[test]
fn test_play_fast_character_trigger_gain_energy() {
    assert_round_trip("Characters in your hand have {fast}.", "");
    assert_round_trip("Once per turn, when you play a {fast} character, gain {energy(e)}.", "e: 1");
}

#[test]
fn test_discard_trigger_gain_energy_and_kindle() {
    assert_round_trip(
        "Once per turn, when you discard a card, gain {energy(e)}, then {kindle(k)}.",
        "e: 1\nk: 2",
    );
}

#[test]
fn test_materialize_subtype_trigger_gain_energy() {
    assert_round_trip(
        "When you {materialize} an allied {subtype(t)}, gain {energy(e)}.",
        "t:spirit-animal\ne: 1",
    );
}

#[test]
fn test_materialize_subtype_trigger_that_character_gains_spark() {
    assert_round_trip(
        "When you {materialize} an allied {subtype(t)}, that character gains +{s} spark.",
        "t:spirit-animal\ns:1",
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
    assert_round_trip("When you play {@a subtype(t)}, draw {cards(c)}.", "t: spirit-animal\nc: 1");
}

#[test]
fn test_ally_banished_trigger_gain_spark() {
    assert_round_trip("When an ally is {banished}, this character gains +{s} spark.", "s: 1");
}

#[test]
fn test_ally_banished_trigger_kindle() {
    assert_round_trip("When an ally is {banished}, {kindle(k)}.", "k: 1");
}

#[test]
fn test_ally_dissolved_trigger_draw() {
    assert_round_trip("When an ally is {dissolved}, draw {cards(c)}.", "c: 1");
}

#[test]
fn test_ally_dissolved_trigger_gain_energy() {
    assert_round_trip("When an ally is {dissolved}, gain {energy(e)}.", "e: 1");
}

#[test]
fn test_ally_dissolved_trigger_gain_points() {
    assert_round_trip("When an ally is {dissolved}, gain {points(p)}.", "p: 1");
}

#[test]
fn test_ally_dissolved_trigger_gain_reclaim_for_cost() {
    assert_round_trip(
        "When an ally is {dissolved}, this card gains {reclaim_for_cost(r)} this turn.",
        "r: 1",
    );
}

#[test]
fn test_abandon_ally_trigger_gain_spark() {
    assert_round_trip("When you abandon an ally, this character gains +{s} spark.", "s: 1");
}

#[test]
fn test_abandon_allies_count_trigger_dissolve() {
    assert_round_trip("When you abandon {count_allies(a)} in a turn, {dissolve} an enemy.", "a: 2");
}

#[test]
fn test_abandon_character_trigger_gain_points() {
    assert_round_trip("When you abandon a character, gain {points(p)}.", "p: 1");
}

#[test]
fn test_abandon_ally_trigger_kindle() {
    assert_round_trip("When you abandon an ally, {kindle(k)}.", "k: 2");
}

#[test]
fn test_abandon_character_trigger_draw() {
    assert_round_trip("When you abandon a character, draw {cards(c)}.", "c: 1");
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
    assert_round_trip("When you play an event, gain {energy(e)}.", "e: 1");
}

#[test]
fn test_play_event_trigger_foresee() {
    assert_round_trip("When you play an event, {foresee(f)}.", "f: 1");
}

#[test]
fn test_play_event_trigger_copy() {
    assert_round_trip("Events cost you {energy(e)} more.", "e: 2");
    assert_round_trip("When you play an event from your hand, copy it.", "");
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
    assert_round_trip("Until end of turn, when you play a character, draw {cards(c)}.", "c: 1");
}

#[test]
fn test_judgment_triggers_on_materialize() {
    assert_round_trip(
        "The '{Judgment}' ability of allies triggers when you {materialize} them.",
        "",
    );
}

#[test]
fn test_draw_count_in_void_trigger_reclaim() {
    assert_round_trip("When you draw {cards_numeral(c)} in a turn, while this card is in your void, it gains {reclaim_for_cost(r)} this turn.", "c: 2\nr: 1");
}

#[test]
fn test_play_character_trigger_materialize_figment() {
    assert_round_trip("When you play a character, {materialize} {a_figment(g)}.", "g: halcyon");
}

#[test]
fn test_materialize_ally_trigger_gain_energy() {
    assert_round_trip("When you {materialize} an ally, gain {energy(e)}.", "e: 1");
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
        "When you {materialize} {@a subtype(t)}, {reclaim} this character.",
        "t: survivor",
    );
}

#[test]
fn test_play_subtype_trigger_put_to_void() {
    assert_round_trip(
        "When you play {@a subtype(t)}, put the {top_n_cards(v)} of your deck into your void.",
        "t: survivor\nv: 2",
    );
}

#[test]
fn test_until_end_of_turn_ally_leaves_trigger_gain_energy() {
    assert_round_trip("Until end of turn, when an ally leaves play, gain {energy(e)}.", "e: 2");
}
