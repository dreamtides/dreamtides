//! Round-trip tests for activated abilities.
//!
//! Tests abilities with cost: effect patterns, including energy costs,
//! abandon costs, discard costs, and other activation requirements.

use parser_v2_tests::test_helpers::*;

#[test]
fn test_abandon_allies_count_reclaim_self() {
    assert_round_trip("Abandon {count-allies}: {Reclaim} this character.", "allies: 2");
}

#[test]
fn test_pay_energy_gain_spark_per_subtype() {
    assert_round_trip(
        "{e}: Gain +{s} spark for each allied {subtype}.",
        "e: 4\ns: 1\nsubtype: warrior",
    );
}

#[test]
fn test_pay_energy_banish_card_reclaim_self() {
    assert_round_trip("{e}, {Banish} another card in your void: {Reclaim} this character.", "e: 2");
}

#[test]
fn test_pay_energy_abandon_ally_by_spark_draw() {
    assert_round_trip(
        "{e}, Abandon an ally with spark {s} or less: Draw {cards}.",
        "e: 2\ns: 1\ncards: 2",
    );
}

#[test]
fn test_pay_energy_abandon_discard_hand_draw() {
    assert_round_trip(
        "{e}, Abandon a character, Discard your hand: Draw {cards}.",
        "e: 2\ncards: 3",
    );
}

#[test]
fn test_abandon_ally_gain_spark() {
    assert_round_trip("Abandon an ally: This character gains +{s} spark.", "s: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_pay_energy_discard_kindle() {
    assert_round_trip("{e}, Discard {discards}: {Kindle}.", "e: 1\ndiscards: 1\nk: 2");
}

#[test]
fn test_abandon_ally_gain_energy() {
    assert_round_trip("Abandon an ally: Gain {e}.", "e: 1");
}

#[test]
fn test_abandon_ally_put_to_void() {
    assert_round_trip(
        "Abandon an ally: Put the {top-n-cards} of your deck into your void.",
        "to-void: 2",
    );
}

#[test]
fn test_abandon_ally_kindle() {
    assert_round_trip("Abandon an ally: {Kindle}.", "k: 1");
}

#[test]
fn test_abandon_ally_once_per_turn_gain_points() {
    assert_round_trip("Abandon an ally, once per turn: Gain {points}.", "points: 1");
}

#[test]
fn test_abandon_ally_put_character_on_deck() {
    assert_round_trip(
        "Abandon an ally: You may put a character from your void on top of your deck.",
        "",
    );
}

#[test]
fn test_abandon_ally_dissolve_enemy_by_spark() {
    assert_round_trip(
        "Abandon an ally: You may {dissolve} an enemy with spark less than that ally's spark.",
        "",
    );
}

#[test]
fn test_abandon_ally_once_per_turn_reclaim_subtype() {
    assert_round_trip("Abandon an ally, once per turn: {Reclaim} {a-subtype}.", "subtype: warrior");
}

#[test]
fn test_abandon_discard_hand_gain_energy() {
    assert_round_trip("Abandon a character, Discard your hand: Gain {e}.", "e: 5");
}

#[test]
fn test_pay_energy_draw() {
    assert_round_trip("{e}: Draw {cards}.", "e: 3\ncards: 1");
}

#[test]
fn test_pay_energy_materialize_copy() {
    assert_round_trip("{e}: {Materialize} a copy of an ally.", "e: 4");
}

#[test]
fn test_pay_energy_subtype_spark_becomes() {
    assert_round_trip(
        "{e}: The spark of each allied {subtype} becomes {s}.",
        "e: 3\nsubtype: spirit-animal\ns: 5",
    );
}

#[test]
fn test_abandon_ally_gain_energy_equal_cost() {
    assert_round_trip("Abandon an ally: Gain {energy-symbol} equal to that character's cost.", "");
}

#[test]
fn test_pay_energy_draw_per_energy_spent() {
    assert_round_trip(
        "Pay 1 or more {energy-symbol}: Draw {cards} for each {energy-symbol} spent.",
        "cards: 1",
    );
}

#[test]
fn test_abandon_or_discard_dissolve_enemy() {
    assert_round_trip("Abandon an ally or discard {discards}: {Dissolve} an enemy.", "discards: 1");
}

#[test]
fn test_banish_from_hand_play_for_zero_prevent() {
    assert_round_trip("{Banish} a card from hand: Play this event for {e}.", "e: 0");
    assert_round_trip("{Prevent} a played card.", "");
}

#[test]
fn test_lose_max_energy_play_for_zero_prevent() {
    assert_round_trip("Lose {maximum-energy}: Play this event for {e}.", "max: 1\ne: 0");
    assert_round_trip("{Prevent} a played card.", "");
}

#[test]
fn test_banish_from_hand_play_for_zero_dissolve() {
    assert_round_trip("{Banish} a card from hand: Play this event for {e}.", "e: 0");
    assert_round_trip("{Dissolve} an enemy.", "");
}

#[test]
fn test_fast_abandon_self_prevent_event() {
    assert_round_trip("{Fast} -- Abandon this character: {Prevent} a played event.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_abandon_any_allies_draw_per_abandoned() {
    assert_round_trip(
        "Abandon any number of allies: Draw {cards} for each ally abandoned.",
        "cards: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_return_all_but_one_ally_draw_per_returned() {
    assert_round_trip(
        "Return all but one ally to hand: Draw {cards} for each ally returned.",
        "cards: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_pay_variable_energy_draw_per_energy() {
    assert_round_trip("Pay 1 or more {energy-symbol}: Draw {cards} for each {energy-symbol} spent, then discard {discards}.", "cards: 1\ndiscards: 2");
}
