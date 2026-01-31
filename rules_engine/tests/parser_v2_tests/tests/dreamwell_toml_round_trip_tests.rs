use parser_v2_tests::test_helpers::*;

#[test]
fn test_dreamwell_skypath() {
    assert_round_trip("{Foresee}.", "foresee: 1");
}

#[test]
fn test_dreamwell_autumn_glade() {
    assert_round_trip("Gain {points}.", "points: 1");
}

#[test]
fn test_dreamwell_twilight_radiance() {
    assert_round_trip("Gain {e}.", "e: 1");
}

#[test]
fn test_dreamwell_astral_interface() {
    assert_round_trip_with_expected(
        "Draw {cards}. Discard {discards}.",
        "cards: 1\ndiscards: 1",
        "Draw {cards} and discard {discards}.",
        "cards: 1, discards: 1",
    );
}

#[test]
fn test_dreamwell_auroral_passage() {
    assert_round_trip("Put the {top-n-cards} of your deck into your void.", "to-void: 3");
}
