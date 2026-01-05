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
fn test_round_trip_character_costs_if_discarded_card_this_turn() {
    let original = "This character costs {e} if you have discarded a card this turn.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_lose_maximum_energy_play_for_alternate_cost() {
    let original = "Lose {maximum-energy}: Play this event for {e}.";
    let parsed = parse_ability(original, "max: 1, e: 0");
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

#[test]
fn test_round_trip_disable_enemy_materialized_abilities() {
    let original = "Disable the {Materialized} abilities of enemies.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}
