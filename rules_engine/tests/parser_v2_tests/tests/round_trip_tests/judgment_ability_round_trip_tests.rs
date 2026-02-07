//! Round-trip tests for Judgment phase abilities.
//!
//! Tests abilities that trigger during the Judgment phase.

use parser_v2_tests::test_helpers::*;

#[test]
fn test_judgment_return_self_from_void() {
    assert_round_trip("{Judgment} Return this character from your void to your hand.", "");
}

#[test]
fn test_judgment_may_draw_then_discard() {
    assert_round_trip(
        "{Judgment} You may draw {cards(cards)}, then discard {cards(discards)}.",
        "cards: 2\ndiscards: 3",
    );
}

#[test]
fn test_judgment_foresee() {
    assert_round_trip("{Judgment} {Foresee(foresee)}.", "foresee: 1");
}

#[test]
fn test_judgment_gain_energy() {
    assert_round_trip("{Judgment} Gain {energy(e)}.", "e: 1");
}

#[test]
fn test_judgment_banish_from_void_to_dissolve() {
    assert_round_trip("{Judgment} You may {banish} {cards(cards)} from your void to {dissolve} an enemy with cost {energy(e)} or less.", "cards: 3\ne: 2");
}

#[test]
fn test_judgment_draw_then_discard() {
    assert_round_trip(
        "{Judgment} Draw {cards(cards)}, then discard {cards(discards)}.",
        "cards: 1\ndiscards: 1",
    );
}

#[test]
fn test_judgment_draw_opponent_gains_points() {
    assert_round_trip(
        "{Judgment} Draw {cards(cards)}, then the opponent gains {points(points)}.",
        "cards: 1\npoints: 2",
    );
}

#[test]
fn test_judgment_may_discard_to_draw_and_gain_points() {
    assert_round_trip(
        "{Judgment} You may discard {cards(discards)} to draw {cards(cards)} and gain {points(points)}.",
        "discards: 1\ncards: 1\npoints: 1",
    );
}

#[test]
fn test_judgment_may_discard_to_dissolve_by_spark() {
    assert_round_trip(
        "{Judgment} You may discard {cards(discards)} to {dissolve} an enemy with spark {s} or less.",
        "s: 1\ndiscards: 1",
    );
}

#[test]
fn test_judgment_pay_to_kindle_and_banish_opponent_void() {
    assert_round_trip(
        "{Judgment} Pay {energy(e)} to {kindle(k)} and {banish} {cards(cards)} from the opponent's void.",
        "e: 1\nk: 1\ncards: 1",
    );
}

#[test]
fn test_judgment_gain_points() {
    assert_round_trip("{Judgment} Gain {points(points)}.", "points: 1");
}

#[test]
fn test_judgment_gain_two_points() {
    assert_round_trip("{Judgment} Gain {points(points)}.", "points: 2");
}

#[test]
fn test_judgment_gain_two_energy() {
    assert_round_trip("{Judgment} Gain {energy(e)}.", "e: 2");
}

#[test]
fn test_judgment_each_player_abandons() {
    assert_round_trip("{Judgment} Each player abandons a character.", "");
}

#[test]
fn test_judgment_gain_energy_per_subtype() {
    assert_round_trip(
        "{Judgment} Gain {energy(e)} for each allied {subtype(subtype)}.",
        "e: 1\nsubtype: spirit-animal",
    );
}

#[test]
fn test_judgment_gain_energy_per_character() {
    assert_round_trip("{Judgment} Gain {energy(e)} for each allied character.", "e: 1");
}

#[test]
fn test_judgment_may_banish_ally_then_materialize() {
    assert_round_trip("{Judgment} You may {banish} an ally, then {materialize} it.", "");
}

#[test]
fn test_judgment_may_pay_to_banish_and_materialize_multiple() {
    assert_round_trip(
        "{Judgment} You may pay {energy(e)} to {banish} {up_to_n_allies(number)}, then {materialize} {it_or_them(number)}.",
        "e: 3\nnumber: 2",
    );
}

#[test]
fn test_judgment_may_pay_subtype_gains_spark() {
    assert_round_trip(
        "{Judgment} You may pay {energy(e)} to have each allied {subtype(subtype)} gain +{s} spark.",
        "e: 4\nsubtype:spirit-animal\ns: 2",
    );
}

#[test]
fn test_judgment_may_banish_from_opponent_void_gain_energy() {
    assert_round_trip(
        "{Judgment} You may {banish} {cards(cards)} from the opponent's void to gain {energy(e)}.",
        "cards: 1\ne: 1",
    );
}

#[test]
fn test_judgment_may_pay_return_from_void_to_hand() {
    assert_round_trip(
        "{Judgment} You may pay {energy(e)} to return this character from your void to your hand.",
        "e: 1",
    );
}

#[test]
fn test_judgment_with_allies_that_share_type_draw() {
    assert_round_trip("Has all character types.", "");
    assert_round_trip(
        "{Judgment} With {count_allies(allies)} that share a character type, draw {cards(cards)}.",
        "allies: 3\ncards: 1",
    );
}

#[test]
fn test_judgment_with_allied_subtype_count_gain_energy() {
    assert_round_trip(
        "{Materialized_Judgment} With {count_allied_subtype(subtype, allies)}, gain {energy(e)}.",
        "subtype: warrior\nallies: 2\ne: 1",
    );
}

#[test]
fn test_judgment_with_allied_subtype_count_draw() {
    assert_round_trip(
        "{Materialized_Judgment} With {count_allied_subtype(subtype, allies)}, draw {cards(cards)}.",
        "allies: 2\nsubtype: survivor\ncards: 1",
    );
}

#[test]
fn test_judgment_draw_one() {
    assert_round_trip("{Judgment} Draw {cards(cards)}.", "cards: 1");
}

#[test]
fn test_judgment_may_abandon_subtype_to_discover_and_materialize() {
    assert_round_trip("{Judgment} You may abandon {@a subtype(subtype)} to {discover} {@a subtype(subtype)} with cost {energy(e)} higher and {materialize} it.", "subtype: warrior\ne: 1");
}

#[test]
fn test_judgment_with_allied_subtype_gain_energy() {
    assert_round_trip(
        "{Judgment} With {count_allied_subtype(subtype, allies)}, gain {energy(e)}.",
        "subtype:spirit-animal\nallies: 2\ne: 2",
    );
}

#[test]
fn test_judgment_banish_ally_by_spark_then_materialize() {
    assert_round_trip(
        "{Materialized_Judgment} {Banish} an ally with spark {s} or less, then {materialize} it.",
        "s: 2",
    );
}
