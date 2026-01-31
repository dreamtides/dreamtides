use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_when_ally_dissolved_gains_reclaim_for_cost() {
    assert_round_trip(
        "When an ally is {dissolved}, this card gains {reclaim-for-cost} this turn.",
        "reclaim: 3",
    );
}

#[test]
fn test_round_trip_at_end_of_turn_gain_energy() {
    assert_round_trip("At the end of your turn, gain {e}.", "e: 2");
}

#[test]
fn test_round_trip_when_you_materialize_an_ally_gain_energy() {
    assert_round_trip("When you {materialize} an ally, gain {e}.", "e: 1");
}

#[test]
fn test_round_trip_when_you_materialize_a_subtype_reclaim_this_character() {
    assert_round_trip(
        "When you {materialize} {a-subtype}, {Reclaim} this character.",
        "subtype: warrior",
    );
}

#[test]
fn test_round_trip_when_you_materialize_a_character_this_character_gains_spark() {
    assert_round_trip(
        "When you {materialize} a character, this character gains +{s} spark.",
        "s: 2",
    );
}

#[test]
fn test_round_trip_when_you_materialize_an_allied_subtype_that_character_gains_spark() {
    assert_round_trip(
        "When you {materialize} an allied {subtype}, that character gains +{s} spark.",
        "subtype: warrior, s: 2",
    );
}

#[test]
fn test_round_trip_once_per_turn_when_you_materialize_a_character_gain_energy() {
    assert_round_trip(
        "Once per turn, when you {materialize} a character, gain {e}.",
        "e: 1",
    );
}

#[test]
fn test_round_trip_once_per_turn_when_you_materialize_a_character_with_cost_or_less_draw_cards() {
    assert_round_trip(
        "Once per turn, when you {materialize} a character with cost {e} or less, draw {cards}.",
        "e: 2, cards: 1",
    );
}

#[test]
fn test_round_trip_once_per_turn_when_you_materialize_a_subtype_draw_cards() {
    assert_round_trip(
        "Once per turn, when you {materialize} {a-subtype}, draw {cards}.",
        "subtype: warrior, cards: 2",
    );
}

#[test]
fn test_round_trip_once_per_turn_when_you_play_a_fast_card_draw_cards() {
    assert_round_trip(
        "Once per turn, when you play a {fast} card, draw {cards}.",
        "cards: 2",
    );
}

#[test]
fn test_round_trip_when_you_play_a_fast_card_this_character_gains_spark() {
    assert_round_trip(
        "When you play a {fast} card, this character gains +{s} spark.",
        "s: 2",
    );
}

#[test]
fn test_round_trip_once_per_turn_when_you_discard_a_card_gain_energy_and_kindle() {
    assert_round_trip_with_expected(
        "Once per turn, when you discard a card, gain {e} and {kindle}.",
        "e: 1, k: 1",
        "Once per turn, when you discard a card, Gain {e}. {Kindle}.",
        "e: 1, k: 1",
    );
}

#[test]
fn test_round_trip_when_you_abandon_an_ally_kindle() {
    assert_round_trip("When you abandon an ally, {Kindle}.", "k: 1");
}

#[test]
fn test_round_trip_when_an_ally_is_dissolved_gain_points() {
    assert_round_trip("When an ally is {dissolved}, gain {points}.", "points: 2");
}

#[test]
fn test_round_trip_when_an_ally_is_dissolved_draw_cards() {
    assert_round_trip("When an ally is {dissolved}, draw {cards}.", "cards: 2");
}

#[test]
fn test_round_trip_when_an_ally_is_dissolved_gain_energy() {
    assert_round_trip("When an ally is {dissolved}, gain {e}.", "e: 1");
}

#[test]
fn test_round_trip_when_an_ally_is_banished_kindle() {
    assert_round_trip("When an ally is {banished}, {Kindle}.", "k: 1");
}

#[test]
fn test_round_trip_when_an_ally_is_banished_this_character_gains_spark() {
    assert_round_trip(
        "When an ally is {banished}, this character gains +{s} spark.",
        "s: 2",
    );
}

#[test]
fn test_round_trip_when_you_play_a_subtype_draw_cards() {
    assert_round_trip(
        "When you play {a-subtype}, draw {cards}.",
        "subtype: warrior, cards: 2",
    );
}

#[test]
fn test_round_trip_when_you_play_a_subtype_put_cards_from_deck_into_void() {
    assert_round_trip(
        "When you play {a-subtype}, put the {top-n-cards} of your deck into your void.",
        "subtype: warrior, to-void: 3",
    );
}

#[test]
fn test_round_trip_when_you_discard_a_card_gain_points() {
    assert_round_trip("When you discard a card, gain {points}.", "points: 1");
}

#[test]
fn test_round_trip_when_you_discard_a_card_kindle() {
    assert_round_trip("When you discard a card, {Kindle}.", "k: 1");
}

#[test]
fn test_round_trip_when_you_discard_this_character_materialize_it() {
    assert_round_trip_with_expected(
        "When you discard this character, {materialize} it.",
        "",
        "When you discard this character, {Materialize} it.",
        "",
    );
}

#[test]
fn test_round_trip_when_you_play_an_event_gain_energy() {
    assert_round_trip("When you play an event, gain {e}.", "e: 1");
}

#[test]
fn test_round_trip_when_you_play_an_event_foresee() {
    assert_round_trip("When you play an event, {Foresee}.", "foresee: 1");
}

#[test]
fn test_round_trip_when_you_play_cards_in_turn_reclaim_this_character() {
    assert_round_trip(
        "When you play {cards-numeral} in a turn, {Reclaim} this character.",
        "cards: 2",
    );
}

#[test]
fn test_round_trip_when_you_abandon_an_ally_this_character_gains_spark() {
    assert_round_trip(
        "When you abandon an ally, this character gains +{s} spark.",
        "s: 2",
    );
}

#[test]
fn test_round_trip_when_you_abandon_count_allies_in_a_turn_dissolve_an_enemy() {
    assert_round_trip(
        "When you abandon {count-allies} in a turn, {Dissolve} an enemy.",
        "allies: 2",
    );
}

#[test]
fn test_round_trip_when_you_materialize_an_allied_subtype_gain_energy() {
    assert_round_trip(
        "When you {materialize} an allied {subtype}, gain {e}.",
        "subtype: warrior, e: 1",
    );
}

#[test]
fn test_round_trip_when_you_materialize_an_allied_subtype_this_character_gains_spark() {
    assert_round_trip(
        "When you {materialize} an allied {subtype}, this character gains +{s} spark.",
        "subtype: warrior, s: 2",
    );
}

#[test]
fn test_round_trip_when_you_play_a_fast_card_gain_points() {
    assert_round_trip("When you play a {fast} card, gain {points}.", "points: 1");
}

#[test]
fn test_round_trip_when_an_event_is_put_into_your_void_this_character_gains_spark() {
    assert_round_trip(
        "When an event is put into your void, this character gains +{s} spark.",
        "s: 2",
    );
}

#[test]
fn test_round_trip_dissolved_kindle() {
    assert_round_trip("{Dissolved} {Kindle}.", "k: 1");
}

#[test]
fn test_round_trip_when_allied_subtype_dissolved_kindle() {
    assert_round_trip(
        "When an allied {subtype} is {dissolved}, {Kindle}.",
        "subtype: warrior, k: 1",
    );
}

#[test]
fn test_round_trip_dissolved_subtype_in_void_gains_reclaim() {
    assert_round_trip(
        "{Dissolved} {ASubtype} in your void gains {reclaim} equal to its cost.",
        "subtype: warrior",
    );
}

#[test]
fn test_round_trip_dissolved_lowercase_subtype_directive_serializes_to_capital() {
    assert_round_trip_with_expected(
        "{Dissolved} {a-subtype} in your void gains {reclaim} equal to its cost.",
        "subtype: warrior",
        "{Dissolved} {ASubtype} in your void gains {reclaim} equal to its cost.",
        "subtype: warrior",
    );
}

#[test]
fn test_round_trip_dissolved_draw_cards() {
    assert_round_trip("{Dissolved} Draw {cards}.", "cards: 1");
}

#[test]
fn test_round_trip_once_per_turn_play_fast_character_gain_energy() {
    assert_round_trip(
        "Once per turn, when you play a {fast} character, gain {e}.",
        "e: 1",
    );
}

#[test]
fn test_round_trip_when_you_materialize_trigger_judgment_ability_each_ally() {
    assert_round_trip(
        "When you {materialize} a character, trigger the {Judgment} ability of each ally.",
        "",
    );
}

#[test]
fn test_round_trip_when_you_materialize_trigger_judgment_ability_each_enemy() {
    assert_round_trip(
        "When you {materialize} a character, trigger the {Judgment} ability of each enemy.",
        "",
    );
}

#[test]
fn test_round_trip_when_you_discard_card_it_gains_reclaim_equal_to_cost() {
    assert_round_trip(
        "When you discard a card, it gains {reclaim} equal to its cost this turn.",
        "",
    );
}

#[test]
fn test_round_trip_when_you_play_subtype_reclaim_random_character_with_cost_or_less() {
    assert_round_trip(
        "When you play {a-subtype}, {Reclaim} a random character with cost {e} or less.",
        "subtype: warrior, e: 3",
    );
}

#[test]
fn test_round_trip_when_you_play_card_during_opponent_turn_this_character_gains_spark() {
    assert_round_trip(
        "When you play a card during the opponent's turn, this character gains +{s} spark.",
        "s: 1",
    );
}

#[test]
fn test_round_trip_when_you_play_a_character_materialize_figment() {
    assert_round_trip(
        "When you play a character, {Materialize} {a-figment}.",
        "figment: shadow",
    );
}

#[test]
fn test_round_trip_when_opponent_plays_card_which_could_dissolve_ally_prevent_that_card() {
    assert_round_trip(
        "When the opponent plays an event which could {dissolve} an ally, {prevent} that card.",
        "",
    );
}

#[test]
fn test_round_trip_when_you_materialize_text_number_allies_in_turn_reclaim_this_character() {
    assert_round_trip(
        "When you {materialize} {text-number} allies in a turn, {Reclaim} this character.",
        "number: 2",
    );
}

#[test]
fn test_round_trip_when_you_materialize_text_number_allies_in_turn_gain_energy() {
    assert_round_trip(
        "When you {materialize} {text-number} allies in a turn, gain {e}.",
        "number: 3, e: 2",
    );
}

#[test]
fn test_round_trip_when_you_materialize_text_number_warriors_in_turn_draw_cards() {
    assert_round_trip(
        "When you {materialize} {text-number} allied {plural-subtype} in a turn, draw {cards}.",
        "number: 2, subtype: warrior, cards: 1",
    );
}

#[test]
fn test_round_trip_when_you_materialize_text_number_allies_with_spark_in_turn_gain_spark() {
    assert_round_trip(
        "When you {materialize} {text-number} allies with spark {s} or more in a turn, this character gains +{s} spark.",
        "number: 3, s: 2",
    );
}

#[test]
fn test_round_trip_when_you_materialize_text_number_characters_in_turn_kindle() {
    assert_round_trip(
        "When you {materialize} {text-number} characters in a turn, {Kindle}.",
        "number: 4, k: 1",
    );
}
