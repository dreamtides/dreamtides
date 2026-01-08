use parser_v2::serializer::ability_serializer;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_dissolve_all_allies() {
    let original = "{Dissolve} all allies.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_dissolve_all_enemies() {
    let original = "{Dissolve} all enemies.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_dissolve_count_allies() {
    let original = "{Dissolve} {count} allies.";
    let parsed = parse_ability(original, "count: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_dissolve_count_enemies() {
    let original = "{Dissolve} {count} enemies.";
    let parsed = parse_ability(original, "count: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_dissolve_up_to_count_allies() {
    let original = "{Dissolve} up to {count} allies.";
    let parsed = parse_ability(original, "count: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_dissolve_up_to_count_enemies() {
    let original = "{Dissolve} up to {count} enemies.";
    let parsed = parse_ability(original, "count: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_dissolve_any_number_of_allies() {
    let original = "{Dissolve} any number of allies.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_dissolve_any_number_of_enemies() {
    let original = "{Dissolve} any number of enemies.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_banish_all_allies() {
    let original = "{Banish} all allies.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_banish_all_enemies() {
    let original = "{Banish} all enemies.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_banish_count_allies() {
    let original = "{Banish} {count} allies.";
    let parsed = parse_ability(original, "count: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_banish_count_enemies() {
    let original = "{Banish} {count} enemies.";
    let parsed = parse_ability(original, "count: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_banish_up_to_count_allies() {
    let original = "{Banish} up to {count} allies.";
    let parsed = parse_ability(original, "count: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_banish_up_to_count_enemies() {
    let original = "{Banish} up to {count} enemies.";
    let parsed = parse_ability(original, "count: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_banish_any_number_of_allies() {
    let original = "{Banish} any number of allies.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_banish_any_number_of_enemies() {
    let original = "{Banish} any number of enemies.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_materialize_all_allies() {
    let original = "{Materialize} all allies.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_materialize_all_allied_subtype() {
    let original = "{Materialize} all allied {plural-subtype}.";
    let parsed = parse_ability(original, "subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_materialize_up_to_count_allies() {
    let original = "{Materialize} up to {count} allies.";
    let parsed = parse_ability(original, "count: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_materialize_up_to_count_allied_subtype() {
    let original = "{Materialize} up to {count} allied {plural-subtype}.";
    let parsed = parse_ability(original, "count: 3, subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_materialize_any_number_of_allies() {
    let original = "{Materialize} any number of allies.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}

#[test]
fn test_round_trip_materialize_any_number_of_allied_subtype() {
    let original = "{Materialize} any number of allied {plural-subtype}.";
    let parsed = parse_ability(original, "subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized.text);
}
