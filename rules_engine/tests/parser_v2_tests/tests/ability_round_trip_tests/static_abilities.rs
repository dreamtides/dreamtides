use parser_v2::serializer::ability_serializer;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_event_costs_if_character_dissolved() {
    let original = "This event costs {e} if a character dissolved this turn.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_additional_cost_to_play() {
    let original = "To play this card, return an ally with cost {e} or more to hand.";
    let parsed = parse_ability(original, "e: 4");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_characters_in_hand_have_fast() {
    let original = "Characters in your hand have {fast}.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}
