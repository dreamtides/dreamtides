use parser_v2::serializer::parser_formatter;
use parser_v2_tests::test_helpers::parse_ability;

#[test]
fn test_round_trip_when_you_discard_gain_points() {
    let original = "When you discard a card, gain {points}.";
    let parsed = parse_ability(original, "points: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_at_end_of_turn_gain_energy() {
    let original = "At the end of your turn, gain {e}.";
    let parsed = parse_ability(original, "e: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_once_per_turn_triggered() {
    let original = "Once per turn, when you discard a card, gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_abandon_ally_gain_energy() {
    let original = "When you abandon an ally, gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_ally_is_dissolved_draw_cards() {
    let original = "When an ally is {dissolved}, draw {cards}.";
    let parsed = parse_ability(original, "cards: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}
