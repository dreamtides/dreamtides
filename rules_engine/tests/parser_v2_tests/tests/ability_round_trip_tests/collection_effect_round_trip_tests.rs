use parser_v2_tests::test_helpers::*;

// Note: These tests are for collection effect quantifiers that are not yet
// implemented in the parser. They document intended future behavior.

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_dissolve_all_allies() {
    assert_round_trip("{Dissolve} all allies.", "");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_dissolve_all_enemies() {
    assert_round_trip("{Dissolve} all enemies.", "");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_dissolve_count_allies() {
    assert_round_trip("{Dissolve} {count} allies.", "count: 2");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_dissolve_count_enemies() {
    assert_round_trip("{Dissolve} {count} enemies.", "count: 3");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_dissolve_up_to_count_allies() {
    assert_round_trip("{Dissolve} up to {count} allies.", "count: 2");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_dissolve_up_to_count_enemies() {
    assert_round_trip("{Dissolve} up to {count} enemies.", "count: 3");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_dissolve_any_number_of_allies() {
    assert_round_trip("{Dissolve} any number of allies.", "");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_dissolve_any_number_of_enemies() {
    assert_round_trip("{Dissolve} any number of enemies.", "");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_banish_all_allies() {
    assert_round_trip("{Banish} all allies.", "");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_banish_all_enemies() {
    assert_round_trip("{Banish} all enemies.", "");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_banish_count_allies() {
    assert_round_trip("{Banish} {count} allies.", "count: 2");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_banish_count_enemies() {
    assert_round_trip("{Banish} {count} enemies.", "count: 3");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_banish_up_to_count_allies() {
    assert_round_trip("{Banish} up to {count} allies.", "count: 2");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_banish_up_to_count_enemies() {
    assert_round_trip("{Banish} up to {count} enemies.", "count: 3");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_banish_any_number_of_allies() {
    assert_round_trip("{Banish} any number of allies.", "");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_banish_any_number_of_enemies() {
    assert_round_trip("{Banish} any number of enemies.", "");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_materialize_all_allies() {
    assert_round_trip("{Materialize} all allies.", "");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_materialize_all_allied_subtype() {
    assert_round_trip("{Materialize} all allied {plural-subtype}.", "subtype: warrior");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_materialize_up_to_count_allies() {
    assert_round_trip("{Materialize} up to {count} allies.", "count: 2");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_materialize_up_to_count_allied_subtype() {
    assert_round_trip(
        "{Materialize} up to {count} allied {plural-subtype}.",
        "count: 3, subtype: warrior",
    );
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_materialize_any_number_of_allies() {
    assert_round_trip("{Materialize} any number of allies.", "");
}

#[test]
#[ignore = "Collection effect quantifiers not yet implemented"]
fn test_round_trip_materialize_any_number_of_allied_subtype() {
    assert_round_trip("{Materialize} any number of allied {plural-subtype}.", "subtype: warrior");
}
