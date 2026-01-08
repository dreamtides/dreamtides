use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_draw_cards_for_each_card_drawn_this_turn() {
    assert_round_trip(
        "Draw {cards} for each card you have drawn this turn.",
        "cards: 1",
    );
}

#[test]
fn test_round_trip_gain_points_for_each_character_drawn_this_turn() {
    assert_round_trip(
        "Gain {points} for each character you have drawn this turn.",
        "points: 2",
    );
}

#[test]
fn test_round_trip_gain_energy_for_each_card_discarded_this_turn() {
    assert_round_trip(
        "Gain {e} for each card you have discarded this turn.",
        "e: 1",
    );
}

#[test]
fn test_round_trip_draw_cards_for_each_event_discarded_this_turn() {
    assert_round_trip(
        "Draw {cards} for each event you have discarded this turn.",
        "cards: 2",
    );
}

#[test]
fn test_round_trip_gain_points_for_each_character_which_dissolved_this_turn() {
    assert_round_trip(
        "Gain {points} for each character which dissolved this turn.",
        "points: 3",
    );
}

#[test]
fn test_round_trip_gain_energy_for_each_event_which_dissolved_this_turn() {
    assert_round_trip(
        "Gain {e} for each event which dissolved this turn.",
        "e: 2",
    );
}

#[test]
fn test_round_trip_draw_cards_for_each_card_returned() {
    assert_round_trip("Draw {cards} for each card returned.", "cards: 1");
}

#[test]
fn test_round_trip_gain_points_for_each_event_returned() {
    assert_round_trip("Gain {points} for each event returned.", "points: 2");
}

#[test]
fn test_round_trip_gain_energy_for_each_card_abandoned() {
    assert_round_trip("Gain {e} for each card abandoned.", "e: 1");
}

#[test]
fn test_round_trip_draw_cards_for_each_event_abandoned() {
    assert_round_trip("Draw {cards} for each event abandoned.", "cards: 2");
}

#[test]
fn test_round_trip_gain_points_for_each_card_abandoned_this_turn() {
    assert_round_trip(
        "Gain {points} for each card abandoned this turn.",
        "points: 1",
    );
}

#[test]
fn test_round_trip_draw_cards_for_each_event_abandoned_this_turn() {
    assert_round_trip(
        "Draw {cards} for each event abandoned this turn.",
        "cards: 2",
    );
}
