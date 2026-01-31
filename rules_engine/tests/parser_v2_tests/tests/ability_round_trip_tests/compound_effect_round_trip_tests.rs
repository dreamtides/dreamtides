use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_gain_energy_draw_cards() {
    assert_round_trip("Gain {e}. Draw {cards}.", "e: 2, cards: 3");
}

#[test]
fn test_round_trip_judgment_gain_energy_draw_cards() {
    assert_round_trip("{Judgment} Gain {e}. Draw {cards}.", "e: 1, cards: 2");
}

#[test]
fn test_round_trip_draw_cards_discard_cards() {
    assert_round_trip("Draw {cards}. Discard {discards}.", "cards: 2, discards: 1");
}

#[test]
fn test_round_trip_draw_cards_discard_cards_gain_energy() {
    assert_round_trip(
        "Draw {cards}. Discard {discards}. Gain {e}.",
        "cards: 1, discards: 1, e: 1",
    );
}

#[test]
fn test_round_trip_put_cards_from_deck_into_void_draw_cards() {
    assert_round_trip(
        "Put the {top-n-cards} of your deck into your void. Draw {cards}.",
        "to-void: 3, cards: 2",
    );
}

#[test]
fn test_round_trip_discard_cards_draw_cards() {
    assert_round_trip("Discard {discards}. Draw {cards}.", "discards: 1, cards: 2");
}

#[test]
fn test_round_trip_dissolve_enemy_you_lose_points() {
    assert_round_trip("{Dissolve} an enemy. You lose {points}.", "points: 1");
}

#[test]
fn test_round_trip_dissolve_enemy_opponent_gains_points() {
    assert_round_trip(
        "{Dissolve} an enemy. The opponent gains {points}.",
        "points: 1",
    );
}

#[test]
fn test_round_trip_judgment_draw_cards_opponent_gains_points() {
    assert_round_trip(
        "{Judgment} Draw {cards}. The opponent gains {points}.",
        "cards: 2, points: 1",
    );
}

#[test]
fn test_round_trip_return_enemy_or_ally_to_hand_draw_cards() {
    assert_round_trip("Return an enemy or ally to hand. Draw {cards}.", "cards: 1");
}

#[test]
fn test_round_trip_judgment_draw_then_discard() {
    assert_round_trip_with_expected(
        "{Judgment} Draw {cards}, then discard {discards}.",
        "cards: 2, discards: 1",
        "{Judgment} Draw {cards}. Discard {discards}.",
        "cards: 2, discards: 1",
    );
}

#[test]
fn test_round_trip_materialized_discard_then_draw() {
    assert_round_trip_with_expected(
        "{Materialized} Discard {discards}, then draw {cards}.",
        "discards: 1, cards: 2",
        "{Materialized} Discard {discards}. Draw {cards}.",
        "discards: 1, cards: 2",
    );
}

#[test]
fn test_round_trip_materialized_draw_discard() {
    assert_round_trip(
        "{Materialized} Draw {cards}. Discard {discards}.",
        "cards: 2, discards: 1",
    );
}

#[test]
fn test_round_trip_you_may_return_character_from_void_draw_cards() {
    assert_round_trip_with_expected(
        "You may return a character from your void to your hand. Draw {cards}.",
        "cards: 2",
        "You may return a character from your void to your hand, then draw {cards}.",
        "cards: 2",
    );
}

#[test]
fn test_round_trip_events_cost_less() {
    assert_round_trip("Events cost you {e} less.", "e: 1");
}

#[test]
fn test_round_trip_characters_cost_less() {
    assert_round_trip("Characters cost you {e} less.", "e: 2");
}

#[test]
fn test_round_trip_opponent_events_cost_more() {
    assert_round_trip("The opponent's events cost {e} more.", "e: 1");
}

#[test]
fn test_round_trip_allied_plural_subtype_have_spark() {
    assert_round_trip("Allied {plural-subtype} have +{s} spark.", "subtype: warrior, s: 1");
}

#[test]
fn test_round_trip_banish_from_hand_play_for_alternate_cost() {
    assert_round_trip("{Banish} a card from hand: Play this event for {e}.", "e: 0");
}

#[test]
fn test_round_trip_abandon_ally_play_character_for_alternate_cost() {
    assert_round_trip(
        "Abandon an ally: Play this character for {e}, then abandon it.",
        "e: 0",
    );
}

#[test]
fn test_round_trip_banish_ally_materialize_at_end_of_turn() {
    assert_round_trip("{Banish} an ally. {Materialize} it at end of turn.", "");
}

#[test]
fn test_round_trip_banish_ally_then_materialize_it() {
    assert_round_trip("{Banish} an ally, then {materialize} it.", "");
}

#[test]
fn test_round_trip_banish_any_number_of_allies_then_materialize_them() {
    assert_round_trip("{Banish} any number of allies, then {materialize} them.", "");
}

#[test]
fn test_round_trip_banish_up_to_n_allies_then_materialize_them() {
    assert_round_trip(
        "{Banish} {up-to-n-allies}, then {materialize} {it-or-them}.",
        "number: 2",
    );
}

#[test]
fn test_round_trip_you_may_banish_ally_then_materialize_it() {
    assert_round_trip("You may {banish} an ally, then {materialize} it.", "");
}

#[test]
fn test_round_trip_judgment_you_may_banish_ally_then_materialize_it() {
    assert_round_trip("{Judgment} You may {banish} an ally, then {materialize} it.", "");
}
