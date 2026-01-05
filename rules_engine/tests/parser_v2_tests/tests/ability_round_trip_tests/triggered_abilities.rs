use parser_v2::serializer::ability_serializer;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_at_end_of_turn_gain_energy() {
    let original = "At the end of your turn, gain {e}.";
    let parsed = parse_ability(original, "e: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_materialize_an_ally_gain_energy() {
    let original = "When you {materialize} an ally, gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_materialize_a_subtype_reclaim_this_character() {
    let original = "When you {materialize} {a-subtype}, {Reclaim} this character.";
    let parsed = parse_ability(original, "subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_materialize_a_character_this_character_gains_spark() {
    let original = "When you {materialize} a character, this character gains +{s} spark.";
    let parsed = parse_ability(original, "s: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_materialize_an_allied_subtype_that_character_gains_spark() {
    let original = "When you {materialize} an allied {subtype}, that character gains +{s} spark.";
    let parsed = parse_ability(original, "subtype: warrior, s: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_once_per_turn_when_you_materialize_a_character_gain_energy() {
    let original = "Once per turn, when you {materialize} a character, gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_once_per_turn_when_you_materialize_a_character_with_cost_or_less_draw_cards() {
    let original =
        "Once per turn, when you {materialize} a character with cost {e} or less, draw {cards}.";
    let parsed = parse_ability(original, "e: 2, cards: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_once_per_turn_when_you_materialize_a_subtype_draw_cards() {
    let original = "Once per turn, when you {materialize} {a-subtype}, draw {cards}.";
    let parsed = parse_ability(original, "subtype: warrior, cards: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_once_per_turn_when_you_play_a_fast_card_draw_cards() {
    let original = "Once per turn, when you play a {fast} card, draw {cards}.";
    let parsed = parse_ability(original, "cards: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_play_a_fast_card_this_character_gains_spark() {
    let original = "When you play a {fast} card, this character gains +{s} spark.";
    let parsed = parse_ability(original, "s: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_once_per_turn_when_you_discard_a_card_gain_energy_and_kindle() {
    let original = "Once per turn, when you discard a card, gain {e} and {kindle}.";
    let parsed = parse_ability(original, "e: 1, k: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!("Once per turn, when you discard a card, Gain {e}. {Kindle}.", serialized);
}

#[test]
fn test_round_trip_when_you_abandon_an_ally_kindle() {
    let original = "When you abandon an ally, {Kindle}.";
    let parsed = parse_ability(original, "k: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_an_ally_is_dissolved_gain_points() {
    let original = "When an ally is {dissolved}, gain {points}.";
    let parsed = parse_ability(original, "points: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_an_ally_is_dissolved_draw_cards() {
    let original = "When an ally is {dissolved}, draw {cards}.";
    let parsed = parse_ability(original, "cards: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_an_ally_is_dissolved_gain_energy() {
    let original = "When an ally is {dissolved}, gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_an_ally_is_banished_kindle() {
    let original = "When an ally is {banished}, {Kindle}.";
    let parsed = parse_ability(original, "k: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_an_ally_is_banished_this_character_gains_spark() {
    let original = "When an ally is {banished}, this character gains +{s} spark.";
    let parsed = parse_ability(original, "s: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_play_a_subtype_draw_cards() {
    let original = "When you play {a-subtype}, draw {cards}.";
    let parsed = parse_ability(original, "subtype: warrior, cards: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_play_a_subtype_put_cards_from_deck_into_void() {
    let original = "When you play {a-subtype}, put the {top-n-cards} of your deck into your void.";
    let parsed = parse_ability(original, "subtype: warrior, to-void: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_discard_a_card_gain_points() {
    let original = "When you discard a card, gain {points}.";
    let parsed = parse_ability(original, "points: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_discard_a_card_kindle() {
    let original = "When you discard a card, {Kindle}.";
    let parsed = parse_ability(original, "k: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_discard_this_character_materialize_it() {
    let original = "When you discard this character, {Materialize} it.";
    let parsed = parse_ability("When you discard this character, {materialize} it.", "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_play_an_event_gain_energy() {
    let original = "When you play an event, gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_play_an_event_foresee() {
    let original = "When you play an event, {Foresee}.";
    let parsed = parse_ability(original, "foresee: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_play_cards_in_turn_reclaim_this_character() {
    let original = "When you play {cards-numeral} in a turn, {Reclaim} this character.";
    let parsed = parse_ability(original, "cards: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_abandon_an_ally_this_character_gains_spark() {
    let original = "When you abandon an ally, this character gains +{s} spark.";
    let parsed = parse_ability(original, "s: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_materialize_an_allied_subtype_gain_energy() {
    let original = "When you {materialize} an allied {subtype}, gain {e}.";
    let parsed = parse_ability(original, "subtype: warrior, e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_materialize_an_allied_subtype_this_character_gains_spark() {
    let original = "When you {materialize} an allied {subtype}, this character gains +{s} spark.";
    let parsed = parse_ability(original, "subtype: warrior, s: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_play_a_fast_card_gain_points() {
    let original = "When you play a {fast} card, gain {points}.";
    let parsed = parse_ability(original, "points: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_an_event_is_put_into_your_void_this_character_gains_spark() {
    let original = "When an event is put into your void, this character gains +{s} spark.";
    let parsed = parse_ability(original, "s: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolved_kindle() {
    let original = "{Dissolved} {Kindle}.";
    let parsed = parse_ability(original, "k: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_allied_subtype_dissolved_kindle() {
    let original = "When an allied {subtype} is {dissolved}, {Kindle}.";
    let parsed = parse_ability(original, "subtype: warrior, k: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolved_subtype_in_void_gains_reclaim() {
    let original = "{Dissolved} {A-subtype} in your void gains {reclaim} equal to its cost.";
    let parsed = parse_ability(original, "subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolved_draw_cards() {
    let original = "{Dissolved} Draw {cards}.";
    let parsed = parse_ability(original, "cards: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}
