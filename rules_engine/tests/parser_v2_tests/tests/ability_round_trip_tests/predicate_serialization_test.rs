use parser_v2::serializer::ability_serializer;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_dissolve_ally_that_is_not_subtype() {
    let original = "{Dissolve} an ally that is not {a-subtype}.";
    let parsed = parse_ability(original, "subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_ally_with_materialized_ability() {
    let original = "Abandon an ally with a {materialized} ability: Gain {e}.";
    let parsed = parse_ability(original, "e: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_ally_with_activated_ability() {
    let original = "Abandon an ally with an activated ability: Draw {cards}.";
    let parsed = parse_ability(original, "cards: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_materialize_fast_ally_draw_cards() {
    let original = "When you {materialize} a fast ally, draw {cards}.";
    let parsed = parse_ability(original, "cards: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_ally_with_cost() {
    let original = "{Dissolve} an ally with cost {e} or less.";
    let parsed = parse_ability(original, "e: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_fast_ally_with_cost() {
    let original = "{Dissolve} a fast ally with cost {e} or less.";
    let parsed = parse_ability(original, "e: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_all_allies_that_are_not_subtype() {
    let original = "{Dissolve} all allies that are not {a-subtype}.";
    let parsed = parse_ability(original, "subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_play_fast_character_gain_energy() {
    let original = "When you play a {fast} character, gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_ally_with_cost_greater_than() {
    let original = "Abandon an ally with cost {e} or more: {Kindle}.";
    let parsed = parse_ability(original, "e: 4, k: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}
