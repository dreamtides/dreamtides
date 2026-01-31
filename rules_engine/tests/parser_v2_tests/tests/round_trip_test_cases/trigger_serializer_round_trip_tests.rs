use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_judgment_keyword_trigger() {
    assert_round_trip("{Judgment} Gain {e}.", "e: 1");
}

#[test]
fn test_round_trip_materialized_keyword_trigger() {
    assert_round_trip("{Materialized} Draw {cards}.", "cards: 1");
}

#[test]
fn test_round_trip_dissolved_keyword_trigger() {
    assert_round_trip("{Dissolved} Gain {points}.", "points: 1");
}

#[test]
fn test_round_trip_materialized_judgment_combined_trigger() {
    assert_round_trip("{MaterializedJudgment} Draw {cards}.", "cards: 2");
}

#[test]
fn test_round_trip_materialized_dissolved_combined_trigger() {
    assert_round_trip("{MaterializedDissolved} Gain {e}.", "e: 1");
}

#[test]
fn test_round_trip_judgment_with_complex_effect() {
    assert_round_trip("{Judgment} Draw {cards}, then discard {discards}.", "cards: 2, discards: 1");
}

#[test]
fn test_round_trip_materialized_with_conditional_effect() {
    assert_round_trip_with_expected(
        "{Materialized} You may pay {e} to draw {cards}.",
        "e: 2, cards: 1",
        "{Materialized} You may pay {e} to draw {cards}.",
        "e: 2, cards: 1",
    );
}

#[test]
fn test_round_trip_dissolved_with_multiple_effects() {
    assert_round_trip_with_expected(
        "{Dissolved} Gain {e}. Draw {cards}.",
        "e: 1, cards: 1",
        "{Dissolved} Gain {e}. Draw {cards}.",
        "e: 1, cards: 1",
    );
}

#[test]
fn test_round_trip_materialized_judgment_with_targeting() {
    assert_round_trip("{MaterializedJudgment} This character gains +{s} spark.", "s: 2");
}

#[test]
fn test_round_trip_materialized_dissolved_with_each_effect() {
    assert_round_trip("{MaterializedDissolved} Each player draws {cards}.", "cards: 1");
}

#[test]
fn test_round_trip_judgment_with_banish_effect() {
    assert_round_trip_with_expected(
        "{Judgment} {Banish} an enemy.",
        "",
        "{Judgment} {Banish} an enemy.",
        "",
    );
}

#[test]
fn test_round_trip_materialized_with_kindle() {
    assert_round_trip("{Materialized} {Kindle}.", "k: 1");
}

#[test]
fn test_round_trip_dissolved_with_reclaim() {
    assert_round_trip_with_expected(
        "{Dissolved} {Reclaim} this character.",
        "",
        "{Dissolved} {Reclaim} this character.",
        "",
    );
}

#[test]
fn test_round_trip_judgment_with_foresee() {
    assert_round_trip("{Judgment} {Foresee}.", "foresee: 2");
}

#[test]
fn test_round_trip_materialized_with_prevent() {
    assert_round_trip_with_expected(
        "{Materialized} {Prevent} a played card.",
        "",
        "{Materialized} {Prevent} a played card.",
        "",
    );
}

#[test]
fn test_round_trip_dissolved_with_materialize() {
    assert_round_trip_with_expected(
        "{Dissolved} {Materialize} an ally from your void.",
        "",
        "{Dissolved} {Materialize} an ally from your void.",
        "",
    );
}

#[test]
fn test_round_trip_materialized_judgment_with_gain_control() {
    assert_round_trip(
        "{MaterializedJudgment} Gain control of an enemy with cost {e} or less.",
        "e: 3",
    );
}

#[test]
fn test_round_trip_materialized_dissolved_with_void_effect() {
    assert_round_trip(
        "{MaterializedDissolved} Put the {top-n-cards} of your deck into your void.",
        "to-void: 3",
    );
}

#[test]
fn test_round_trip_judgment_with_abandon() {
    assert_round_trip_with_expected(
        "{Judgment} Abandon an ally.",
        "",
        "{Judgment} Abandon an ally.",
        "",
    );
}

#[test]
fn test_round_trip_materialized_with_discover() {
    assert_round_trip_with_expected(
        "{Materialized} {Discover} an event.",
        "",
        "{Materialized} {Discover} an event.",
        "",
    );
}

#[test]
fn test_round_trip_dissolved_with_return_to_hand() {
    assert_round_trip("{Dissolved} Return this character to your hand.", "");
}

#[test]
fn test_round_trip_judgment_with_for_each() {
    assert_round_trip("{Judgment} Gain {e} for each ally.", "e: 1");
}

#[test]
fn test_round_trip_materialized_with_until_condition() {
    assert_round_trip("{Materialized} This character gains +{s} spark until end of turn.", "s: 1");
}

#[test]
fn test_round_trip_materialized_judgment_with_count_condition() {
    assert_round_trip("{MaterializedJudgment} With {count-allies}, gain {e}.", "allies: 3, e: 2");
}

#[test]
fn test_round_trip_dissolved_with_all_players() {
    assert_round_trip("{Dissolved} Each player gains {e}.", "e: 1");
}
