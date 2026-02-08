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
        "{Judgment} You may draw {cards($c)}, then discard {cards($d)}.",
        "c: 2\nd: 3",
    );
}

#[test]
fn test_judgment_foresee() {
    assert_round_trip("{Judgment} {Foresee($f)}.", "f: 1");
}

#[test]
fn test_judgment_gain_energy() {
    assert_round_trip("{Judgment} Gain {energy($e)}.", "e: 1");
}

#[test]
fn test_judgment_banish_from_void_to_dissolve() {
    assert_round_trip("{Judgment} You may {banish} {cards($c)} from your void to {dissolve} an enemy with cost {energy($e)} or less.", "c: 3\ne: 2");
}

#[test]
fn test_judgment_draw_then_discard() {
    assert_round_trip("{Judgment} Draw {cards($c)}, then discard {cards($d)}.", "c: 1\nd: 1");
}

#[test]
fn test_judgment_draw_opponent_gains_points() {
    assert_round_trip(
        "{Judgment} Draw {cards($c)}, then the opponent gains {points($p)}.",
        "c: 1\np: 2",
    );
}

#[test]
fn test_judgment_may_discard_to_draw_and_gain_points() {
    assert_round_trip(
        "{Judgment} You may discard {cards($d)} to draw {cards($c)} and gain {points($p)}.",
        "d: 1\nc: 1\np: 1",
    );
}

#[test]
fn test_judgment_may_discard_to_dissolve_by_spark() {
    assert_round_trip(
        "{Judgment} You may discard {cards($d)} to {dissolve} an enemy with spark {$s} or less.",
        "s: 1\nd: 1",
    );
}

#[test]
fn test_judgment_pay_to_kindle_and_banish_opponent_void() {
    assert_round_trip(
        "{Judgment} Pay {energy($e)} to {kindle($k)} and {banish} {cards($c)} from the opponent's void.",
        "e: 1\nk: 1\nc: 1",
    );
}

#[test]
fn test_judgment_may_pay_to_kindle_and_banish_opponent_void() {
    assert_round_trip(
        "{Judgment} You may pay {energy($e)} to {kindle($k)} and {banish} {cards($c)} from the opponent's void.",
        "e: 1\nk: 1\nc: 1",
    );
}

#[test]
fn test_judgment_gain_points() {
    assert_round_trip("{Judgment} Gain {points($p)}.", "p: 1");
}

#[test]
fn test_judgment_gain_two_points() {
    assert_round_trip("{Judgment} Gain {points($p)}.", "p: 2");
}

#[test]
fn test_judgment_gain_two_energy() {
    assert_round_trip("{Judgment} Gain {energy($e)}.", "e: 2");
}

#[test]
fn test_judgment_each_player_abandons() {
    assert_round_trip("{Judgment} Each player abandons a character.", "");
}

#[test]
fn test_judgment_gain_energy_per_subtype() {
    assert_round_trip(
        "{Judgment} Gain {energy($e)} for each allied {subtype($t)}.",
        "e: 1\nt: SpiritAnimal",
    );
}

#[test]
fn test_judgment_gain_energy_per_character() {
    assert_round_trip("{Judgment} Gain {energy($e)} for each allied character.", "e: 1");
}

#[test]
fn test_judgment_may_banish_ally_then_materialize() {
    assert_round_trip("{Judgment} You may {banish} an ally, then {materialize} it.", "");
}

#[test]
fn test_judgment_may_pay_to_banish_and_materialize_multiple() {
    assert_round_trip(
        "{Judgment} You may pay {energy($e)} to {banish} {up_to_n_allies($n)}, then {materialize} {it_or_them($n)}.",
        "e: 3\nn: 2",
    );
}

#[test]
fn test_judgment_may_pay_subtype_gains_spark() {
    assert_round_trip(
        "{Judgment} You may pay {energy($e)} to have each allied {subtype($t)} gain +{$s} spark.",
        "e: 4\nt: SpiritAnimal\ns: 2",
    );
}

#[test]
fn test_judgment_may_banish_from_opponent_void_gain_energy() {
    assert_round_trip(
        "{Judgment} You may {banish} {cards($c)} from the opponent's void to gain {energy($e)}.",
        "c: 1\ne: 1",
    );
}

#[test]
fn test_judgment_may_pay_return_from_void_to_hand() {
    assert_round_trip(
        "{Judgment} You may pay {energy($e)} to return this character from your void to your hand.",
        "e: 1",
    );
}

#[test]
fn test_judgment_with_allies_that_share_type_draw() {
    assert_round_trip("Has all character types.", "");
    assert_round_trip(
        "{Judgment} With {count_allies($a)} that share a character type, draw {cards($c)}.",
        "a: 3\nc: 1",
    );
}

#[test]
fn test_judgment_with_allied_subtype_count_gain_energy() {
    assert_round_trip(
        "{Materialized_Judgment} With {count_allied_subtype($a, $t)}, gain {energy($e)}.",
        "t: Warrior\na: 2\ne: 1",
    );
}

#[test]
fn test_judgment_with_allied_subtype_count_draw() {
    assert_round_trip(
        "{Materialized_Judgment} With {count_allied_subtype($a, $t)}, draw {cards($c)}.",
        "a: 2\nt: Survivor\nc: 1",
    );
}

#[test]
fn test_judgment_draw_one() {
    assert_round_trip("{Judgment} Draw {cards($c)}.", "c: 1");
}

#[test]
fn test_judgment_may_abandon_subtype_to_discover_and_materialize() {
    assert_round_trip("{Judgment} You may abandon {@a subtype($t)} to {discover} {@a subtype($t)} with cost {energy($e)} higher and {materialize} it.", "t: Warrior\ne: 1");
}

#[test]
fn test_judgment_with_allied_subtype_gain_energy() {
    assert_round_trip(
        "{Judgment} With {count_allied_subtype($a, $t)}, gain {energy($e)}.",
        "t: SpiritAnimal\na: 2\ne: 2",
    );
}

#[test]
fn test_judgment_banish_ally_by_spark_then_materialize() {
    assert_round_trip(
        "{Materialized_Judgment} {Banish} an ally with spark {$s} or less, then {materialize} it.",
        "s: 2",
    );
}
