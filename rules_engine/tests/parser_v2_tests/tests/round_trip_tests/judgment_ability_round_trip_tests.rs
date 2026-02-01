//! Round-trip tests for Judgment phase abilities.
//!
//! Tests abilities that trigger during the Judgment phase.
//!
//! GENERATED FILE - Do not edit manually.
//! Regenerate with: python scripts/generate_round_trip_tests.py

use parser_v2_tests::test_helpers::*;

#[test]
fn test_judgment_return_self_from_void() {
    assert_round_trip("{Judgment} Return this character from your void to your hand.", "");
}

#[test]
fn test_judgment_may_draw_then_discard() {
    assert_round_trip(
        "{Judgment} You may draw {cards}, then discard {discards}.",
        "cards: 2\ndiscards: 3",
    );
}

#[test]
fn test_judgment_foresee() {
    assert_round_trip("{Judgment} {Foresee}.", "foresee: 1");
}

#[test]
fn test_judgment_gain_energy() {
    assert_round_trip("{Judgment} Gain {e}.", "e: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_judgment_banish_from_void_to_dissolve() {
    assert_round_trip("{Judgment} You may {banish} {cards} from your void to {dissolve} an enemy with cost {e} or less.", "cards: 3\ne: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_judgment_draw_then_discard() {
    assert_round_trip("{Judgment} Draw {cards}, then discard {discards}.", "cards: 1\ndiscards: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_judgment_draw_opponent_gains_points() {
    assert_round_trip(
        "{Judgment} Draw {cards}. The opponent gains {points}.",
        "cards: 1\npoints: 2",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_judgment_may_discard_to_draw_and_gain_points() {
    assert_round_trip(
        "{Judgment} You may discard {discards} to draw {cards} and gain {points}.",
        "discards: 1\ncards: 1\npoints: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_judgment_may_discard_to_dissolve_by_spark() {
    assert_round_trip(
        "{Judgment} You may discard a card to {dissolve} an enemy with spark {s} or less.",
        "s: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_judgment_pay_to_kindle_and_banish_opponent_void() {
    assert_round_trip(
        "{Judgment} Pay {e} to {kindle} and {banish} {cards} from the opponent's void.",
        "e: 1\nk: 1\ncards: 1",
    );
}

#[test]
fn test_judgment_gain_points() {
    assert_round_trip("{Judgment} Gain {points}.", "points: 1");
}

#[test]
fn test_judgment_gain_two_points() {
    assert_round_trip("{Judgment} Gain {points}.", "points: 2");
}

#[test]
fn test_judgment_gain_two_energy() {
    assert_round_trip("{Judgment} Gain {e}.", "e: 2");
}

#[test]
fn test_judgment_each_player_abandons() {
    assert_round_trip("{Judgment} Each player abandons a character.", "");
}

#[test]
fn test_judgment_gain_energy_per_subtype() {
    assert_round_trip(
        "{Judgment} Gain {e} for each allied {subtype}.",
        "e: 1\nsubtype: spirit-animal",
    );
}

#[test]
fn test_judgment_gain_energy_per_character() {
    assert_round_trip("{Judgment} Gain {e} for each allied character.", "e: 1");
}

#[test]
fn test_judgment_may_banish_ally_then_materialize() {
    assert_round_trip("{Judgment} You may {banish} an ally, then {materialize} it.", "");
}

#[test]
fn test_judgment_may_pay_to_banish_and_materialize_multiple() {
    assert_round_trip(
        "{Judgment} You may pay {e} to {banish} {up-to-n-allies}, then {materialize} {it-or-them}.",
        "e: 3\nnumber: 2",
    );
}

#[test]
fn test_judgment_may_pay_subtype_gains_spark() {
    assert_round_trip(
        "{Judgment} You may pay {e} to have each allied {subtype} gain +{s} spark.",
        "e: 4\nsubtype:spirit-animal\ns: 2",
    );
}

#[test]
fn test_judgment_may_banish_from_opponent_void_gain_energy() {
    assert_round_trip(
        "{Judgment} You may {banish} {cards} from the opponent's void to gain {e}.",
        "cards: 1\ne: 1",
    );
}

#[test]
fn test_judgment_may_pay_return_from_void_to_hand() {
    assert_round_trip(
        "{Judgment} You may pay {e} to return this character from your void to your hand.",
        "e: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_judgment_with_allies_that_share_type_draw() {
    assert_round_trip("Has all character types.", "allies: 3\ncards: 1");
    assert_round_trip(
        "{Judgment} With {count-allies} that share a character type, draw {cards}.",
        "allies: 3\ncards: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_judgment_with_allied_subtype_count_gain_energy() {
    assert_round_trip(
        "{MaterializedJudgment} With {count-allied-subtype}, gain {e}.",
        "subtype: warrior\nallies: 2\ne: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_judgment_with_allied_subtype_count_draw() {
    assert_round_trip(
        "{MaterializedJudgment} With {count-allied-subtype}, draw {cards}.",
        "allies: 2\nsubtype: survivor\ncards: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_judgment_draw_one() {
    assert_round_trip("{Judgment} Draw {cards}.", "e: 3\ncards: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_judgment_may_abandon_subtype_to_discover_and_materialize() {
    assert_round_trip("{Judgment} You may abandon {a-subtype} to {discover} {a-subtype} with cost {e} higher and {materialize} it.", "subtype: warrior\ne: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_judgment_with_allied_subtype_gain_energy() {
    assert_round_trip(
        "{Judgment} With {count-allied-subtype}, gain {e}.",
        "subtype:spirit-animal\nallies: 2\ne: 2",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_judgment_banish_ally_by_spark_then_materialize() {
    assert_round_trip(
        "{MaterializedJudgment} {Banish} an ally with spark {s} or less, then {materialize} it.",
        "s: 2",
    );
}
