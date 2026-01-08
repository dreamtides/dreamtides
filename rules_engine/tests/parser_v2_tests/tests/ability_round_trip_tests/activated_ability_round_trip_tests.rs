use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_abandon_an_ally_gain_energy() {
    assert_round_trip("Abandon an ally: Gain {e}.", "e: 1");
}

#[test]
fn test_round_trip_abandon_an_ally_once_per_turn_gain_points() {
    assert_round_trip("Abandon an ally, once per turn: Gain {points}.", "points: 1");
}

#[test]
fn test_round_trip_abandon_an_ally_once_per_turn_reclaim_subtype() {
    assert_round_trip_with_expected(
        "Abandon an ally, once per turn: {Reclaim} a {subtype}.",
        "subtype: warrior",
        "Abandon an ally, once per turn: {Reclaim} {a-subtype}.",
        "subtype: warrior",
    );
}

#[test]
fn test_round_trip_abandon_an_ally_kindle() {
    assert_round_trip("Abandon an ally: {Kindle}.", "k: 1");
}

#[test]
fn test_round_trip_abandon_an_ally_put_cards_from_deck_into_void() {
    assert_round_trip(
        "Abandon an ally: Put the {top-n-cards} of your deck into your void.",
        "to-void: 2",
    );
}

#[test]
fn test_round_trip_abandon_an_ally_put_character_from_void_on_top_of_deck() {
    assert_round_trip(
        "Abandon an ally: You may put a character from your void on top of your deck.",
        "",
    );
}

#[test]
fn test_round_trip_abandon_or_discard_dissolve_enemy() {
    assert_round_trip("Abandon an ally or discard a card: {Dissolve} an enemy.", "");
}

#[test]
fn test_round_trip_energy_discard_kindle() {
    assert_round_trip_with_expected(
        "{e}, Discard {discards}: {kindle}.",
        "e: 1, discards: 2, k: 1",
        "{e}, Discard {discards}: {Kindle}.",
        "e: 1, discards: 2, k: 1",
    );
}

#[test]
fn test_round_trip_energy_banish_reclaim_this_character() {
    assert_round_trip(
        "{e}, {Banish} another card in your void: {Reclaim} this character.",
        "e: 1",
    );
}

#[test]
fn test_round_trip_energy_abandon_ally_with_spark_draw_cards() {
    assert_round_trip(
        "{e}, Abandon an ally with spark {s} or less: Draw {cards}.",
        "e: 1, s: 2, cards: 3",
    );
}

#[test]
fn test_round_trip_energy_abandon_character_discard_hand_draw_cards() {
    assert_round_trip(
        "{e}, Abandon a character, Discard your hand: Draw {cards}.",
        "e: 2, cards: 3",
    );
}

#[test]
fn test_round_trip_abandon_character_discard_hand_gain_energy() {
    assert_round_trip("Abandon a character, Discard your hand: Gain {e}.", "e: 1");
}

#[test]
fn test_round_trip_energy_materialize_copy_of_ally() {
    assert_round_trip("{e}: {Materialize} a copy of an ally.", "e: 1");
}

#[test]
fn test_round_trip_energy_gain_spark_for_each_allied_subtype() {
    assert_round_trip(
        "{e}: Gain +{s} spark for each allied {subtype}.",
        "e: 1, s: 2, subtype: warrior",
    );
}

#[test]
fn test_round_trip_abandon_an_ally_this_character_gains_spark() {
    assert_round_trip("Abandon an ally: This character gains +{s} spark.", "s: 2");
}

#[test]
fn test_round_trip_abandon_count_allies_reclaim_this_character() {
    assert_round_trip("Abandon {count-allies}: {Reclaim} this character.", "allies: 3");
}

#[test]
fn test_round_trip_abandon_any_number_of_allies_draw_cards_for_each_abandoned() {
    assert_round_trip(
        "Abandon any number of allies: Draw {cards} for each ally abandoned.",
        "cards: 1",
    );
}

#[test]
fn test_round_trip_abandon_allies_draw_cards_for_each_allied_subtype_abandoned() {
    assert_round_trip(
        "Abandon any number of allies: Draw {cards} for each allied {subtype} abandoned.",
        "subtype: warrior, cards: 1",
    );
}

#[test]
fn test_round_trip_abandon_ally_gain_energy_equal_to_cost() {
    assert_round_trip(
        "Abandon an ally: Gain {energy-symbol} equal to that character's cost.",
        "",
    );
}

#[test]
fn test_round_trip_abandon_ally_dissolve_enemy_with_spark_less_than_abandoned() {
    assert_round_trip(
        "Abandon an ally: You may {dissolve} an enemy with spark less than that ally's spark.",
        "",
    );
}

#[test]
fn test_round_trip_banish_void_with_min_count_reclaim_this_character() {
    assert_round_trip(
        "{Banish} your void with {count} or more cards: {Reclaim} this character.",
        "count: 3",
    );
}

#[test]
fn test_round_trip_energy_spark_of_each_allied_subtype_becomes() {
    assert_round_trip(
        "{e}: The spark of each allied {subtype} becomes {s}.",
        "e: 1, subtype: warrior, s: 3",
    );
}

#[test]
fn test_round_trip_fast_abandon_this_character_prevent_played_event() {
    assert_round_trip("{Fast} -- Abandon this character: {Prevent} a played event.", "");
}

#[test]
fn test_round_trip_pay_one_or_more_energy_draw_for_each_energy_spent() {
    assert_round_trip(
        "Pay 1 or more {energy-symbol}: Draw {cards} for each {energy-symbol} spent, then discard {discards}.",
        "cards: 1, discards: 1",
    );
}

#[test]
fn test_round_trip_pay_one_or_more_dissolve_each_character() {
    assert_round_trip(
        "Pay 1 or more {energy-symbol}: {Dissolve} each character with spark less than the amount of {energy-symbol} paid.",
        "",
    );
}

#[test]
fn test_round_trip_spend_one_or_more_energy_draw_for_each_energy_spent() {
    assert_round_trip_with_expected(
        "Spend 1 or more {energy-symbol}: Draw {cards} for each {energy-symbol} spent.",
        "cards: 2",
        "Pay 1 or more {energy-symbol}: Draw {cards} for each {energy-symbol} spent.",
        "cards: 2",
    );
}

#[test]
fn test_round_trip_abandon_a_dreamscape_gain_energy() {
    assert_round_trip("Abandon a dreamscape: Gain {e}.", "e: 1");
}

#[test]
fn test_round_trip_abandon_multiple_dreamscapes_draw_cards() {
    assert_round_trip("Abandon 3 dreamscapes: Draw {cards}.", "cards: 2");
}

#[test]
fn test_round_trip_banish_your_void_reclaim_this_character() {
    assert_round_trip("{Banish} your void: {Reclaim} this character.", "");
}

#[test]
fn test_round_trip_cost_list_abandon_and_discard() {
    assert_round_trip("Abandon an ally and discard a card: Draw {cards}.", "cards: 2");
}

#[test]
fn test_round_trip_return_up_to_allies_to_hand_gain_energy() {
    assert_round_trip("Return up to 3 allies to hand: Gain {e}.", "e: 2");
}
