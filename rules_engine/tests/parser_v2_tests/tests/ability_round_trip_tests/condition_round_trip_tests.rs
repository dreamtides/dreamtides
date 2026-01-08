use parser_v2_tests::test_helpers::*;

// Tests for PredicateCount with count = 1 (single allied subtype)
#[test]
fn test_round_trip_with_an_allied_subtype_gain_energy() {
    assert_round_trip(
        "{Judgment} With an allied {subtype}, gain {e}.",
        "subtype: warrior, e: 2",
    );
}

#[test]
fn test_round_trip_with_an_allied_subtype_draw_cards() {
    assert_round_trip(
        "{Judgment} With an allied {subtype}, draw {cards}.",
        "subtype: warrior, cards: 1",
    );
}

// Tests for PredicateCount with count > 1 (multiple allied subtypes)
#[test]
fn test_round_trip_with_count_allied_subtype_gain_energy() {
    assert_round_trip(
        "{Judgment} With {count-allied-subtype}, gain {e}.",
        "subtype: warrior, allies: 2, e: 3",
    );
}

#[test]
fn test_round_trip_with_count_allied_subtype_draw_cards() {
    assert_round_trip(
        "{Judgment} With {count-allied-subtype}, draw {cards}.",
        "subtype: spirit-animal, allies: 3, cards: 2",
    );
}

#[test]
fn test_round_trip_materialized_with_count_allied_subtype() {
    assert_round_trip(
        "{Materialized} With {count-allied-subtype}, gain {e}.",
        "subtype: warrior, allies: 2, e: 1",
    );
}

// Tests for AlliesThatShareACharacterType condition
#[test]
fn test_round_trip_with_count_allies_that_share_a_character_type_draw_cards() {
    assert_round_trip(
        "{Judgment} With {count-allies} that share a character type, draw {cards}.",
        "allies: 3, cards: 2",
    );
}

#[test]
fn test_round_trip_with_count_allies_that_share_a_character_type_gain_energy() {
    assert_round_trip(
        "{Judgment} With {count-allies} that share a character type, gain {e}.",
        "allies: 4, e: 3",
    );
}

#[test]
fn test_round_trip_materialized_judgment_with_allies_that_share_a_character_type() {
    assert_round_trip(
        "{MaterializedJudgment} With {count-allies} that share a character type, gain {e}.",
        "allies: 2, e: 1",
    );
}
