use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_materialized_you_may_banish_ally_then_materialize_it() {
    assert_round_trip_with_expected(
        "{Materialized} You may {banish} an ally, then {materialize} it.",
        "",
        "{Materialized} You may {Banish} an ally, then {Materialize} it.",
        "",
    );
}

#[test]
fn test_round_trip_materialized_banish_any_number_of_allies_then_materialize_them() {
    assert_round_trip_with_expected(
        "{Materialized} {Banish} any number of allies, then {materialize} them.",
        "",
        "{Materialized} {Banish} any number of allies. {Materialize} them.",
        "",
    );
}

#[test]
fn test_round_trip_judgment_you_may_banish_ally_then_materialize_it() {
    assert_round_trip_with_expected(
        "{Judgment} You may {banish} an ally, then {materialize} it.",
        "",
        "{Judgment} You may {Banish} an ally, then {Materialize} it.",
        "",
    );
}

#[test]
fn test_round_trip_judgment_foresee() {
    assert_round_trip("{Judgment} {Foresee}.", "foresee: 3");
}

#[test]
fn test_round_trip_judgment_kindle() {
    assert_round_trip("{Judgment} {Kindle}.", "k: 2");
}

#[test]
fn test_round_trip_materialized_foresee() {
    assert_round_trip("{Materialized} {Foresee}.", "foresee: 2");
}

#[test]
fn test_round_trip_materialized_judgment_kindle() {
    assert_round_trip("{MaterializedJudgment} {Kindle}.", "k: 1");
}

#[test]
fn test_round_trip_materialized_gain_control_enemy_with_cost_or_less() {
    assert_round_trip("{Materialized} Gain control of an enemy with cost {e} or less.", "e: 2");
}

#[test]
fn test_round_trip_materialized_dissolved_draw_cards() {
    assert_round_trip("{MaterializedDissolved} Draw {cards}.", "cards: 1");
}

#[test]
fn test_round_trip_materialized_dissolved_put_cards_from_deck_into_void() {
    assert_round_trip(
        "{MaterializedDissolved} Put the {top-n-cards} of your deck into your void.",
        "to-void: 2",
    );
}

#[test]
fn test_round_trip_materialized_judgment_gain_energy() {
    assert_round_trip("{MaterializedJudgment} Gain {e}.", "e: 1");
}

#[test]
fn test_round_trip_materialized_return_ally_to_hand() {
    assert_round_trip("{Materialized} Return an ally to hand.", "");
}

#[test]
fn test_round_trip_judgment_pay_energy_to_kindle_and_banish_cards_from_opponent_void() {
    assert_round_trip(
        "{Judgment} Pay {e} to {Kindle} and {Banish} {cards} from the opponent's void.",
        "e: 1, k: 1, cards: 2",
    );
}

#[test]
fn test_round_trip_materialized_you_may_return_ally_to_hand() {
    assert_round_trip("{Materialized} You may return an ally to hand.", "");
}

#[test]
fn test_round_trip_materialized_return_character_from_void_to_hand() {
    assert_round_trip("{Materialized} Return a character from your void to your hand.", "");
}

#[test]
fn test_round_trip_materialized_discover_fast_event() {
    assert_round_trip("{Materialized} {Discover} a {fast} event.", "");
}

#[test]
fn test_round_trip_judgment_return_this_from_void_to_hand() {
    assert_round_trip("{Judgment} Return this character from your void to your hand.", "");
}

#[test]
fn test_round_trip_materialized_banish_opponent_void() {
    assert_round_trip("{Materialized} {Banish} the opponent's void.", "");
}

#[test]
fn test_round_trip_judgment_you_may_pay_to_return_this_from_void_to_hand() {
    assert_round_trip(
        "{Judgment} You may pay {e} to return this character from your void to your hand.",
        "e: 1",
    );
}

#[test]
fn test_round_trip_judgment_you_may_pay_to_have_each_allied_gain_spark() {
    assert_round_trip(
        "{Judgment} You may pay {e} to have each allied {subtype} gain +{s} spark.",
        "e: 1, subtype: warrior, s: 2",
    );
}

#[test]
fn test_round_trip_dissolved_you_may_pay_to_return_this_to_hand() {
    assert_round_trip("{Dissolved} You may pay {e} to return this character to your hand.", "e: 1");
}

#[test]
fn test_round_trip_materialized_prevent_played_card_with_cost() {
    assert_round_trip("{Materialized} {Prevent} a played card with cost {e} or less.", "e: 3");
}

#[test]
fn test_round_trip_judgment_you_may_draw_then_discard() {
    assert_round_trip(
        "{Judgment} You may draw {cards}, then discard {discards}.",
        "cards: 2, discards: 1",
    );
}

#[test]
fn test_round_trip_judgment_you_may_discard_draw_gain_points() {
    assert_round_trip(
        "{Judgment} You may discard {discards} to draw {cards} and gain {points}.",
        "discards: 2, cards: 1, points: 3",
    );
}

#[test]
fn test_round_trip_judgment_you_may_discard_dissolve_enemy() {
    assert_round_trip(
        "{Judgment} You may discard a card to {dissolve} an enemy with spark {s} or less.",
        "s: 2",
    );
}

#[test]
fn test_round_trip_judgment_gain_energy_for_each_allied_subtype() {
    assert_round_trip("{Judgment} Gain {e} for each allied {subtype}.", "subtype: warrior, e: 1");
}

#[test]
fn test_round_trip_judgment_gain_energy_for_each_allied_character() {
    assert_round_trip("{Judgment} Gain {e} for each allied character.", "e: 1");
}

#[test]
fn test_round_trip_judgment_with_count_allied_subtype_gain_energy() {
    assert_round_trip(
        "{Judgment} With {count-allied-subtype}, gain {e}.",
        "subtype: warrior, allies: 2, e: 3",
    );
}

#[test]
fn test_round_trip_materialized_judgment_with_count_allied_subtype_gain_energy() {
    assert_round_trip(
        "{MaterializedJudgment} With {count-allied-subtype}, gain {e}.",
        "subtype: warrior, allies: 2, e: 3",
    );
}

#[test]
fn test_round_trip_materialized_judgment_with_count_allied_subtype_draw_cards() {
    assert_round_trip(
        "{MaterializedJudgment} With {count-allied-subtype}, draw {cards}.",
        "subtype: warrior, allies: 2, cards: 1",
    );
}

#[test]
fn test_round_trip_judgment_with_count_allies_gain_energy() {
    assert_round_trip("{Judgment} With {count-allies}, gain {e}.", "allies: 3, e: 2");
}

#[test]
fn test_round_trip_materialized_each_player_discards() {
    assert_round_trip("{Materialized} Each player discards {discards}.", "discards: 1");
}

#[test]
fn test_round_trip_judgment_each_player_abandons_character() {
    assert_round_trip("{Judgment} Each player abandons a character.", "");
}

#[test]
fn test_round_trip_materialized_discard_chosen_card_from_opponent_hand_they_draw() {
    assert_round_trip(
        "{Materialized} Discard a chosen card from the opponent's hand. They draw {cards}.",
        "cards: 1",
    );
}

#[test]
fn test_round_trip_materialized_draw_cards_for_each_ally_abandoned_this_turn() {
    assert_round_trip("{Materialized} Draw {cards} for each ally abandoned this turn.", "cards: 2");
}

#[test]
fn test_round_trip_materialized_draw_cards_for_each_allied_subtype_abandoned_this_turn() {
    assert_round_trip(
        "{Materialized} Draw {cards} for each allied {subtype} abandoned this turn.",
        "subtype: warrior, cards: 1",
    );
}

#[test]
fn test_round_trip_materialized_card_with_cost_in_void_gains_reclaim() {
    assert_round_trip(
        "{Materialized} A card with cost {e} or less in your void gains {reclaim-for-cost} this turn.",
        "e: 3, reclaim: 0",
    );
}

#[test]
fn test_round_trip_judgment_banish_cards_from_your_void_to_dissolve_enemy_with_cost() {
    assert_round_trip(
        "{Judgment} You may {banish} {cards} from your void to {dissolve} an enemy with cost {e} or less.",
        "cards: 3, e: 2",
    );
}

#[test]
fn test_round_trip_judgment_banish_cards_from_opponent_void_to_gain_energy() {
    assert_round_trip(
        "{Judgment} You may {banish} {cards} from the opponent's void to gain {e}.",
        "cards: 1, e: 1",
    );
}

#[test]
fn test_round_trip_judgment_abandon_to_discover_and_materialize() {
    assert_round_trip_with_expected(
        "{Judgment} You may abandon {a-subtype} to {discover} {a-subtype} with cost {e} higher and {materialize} it.",
        "subtype: warrior, e: 2",
        "{Judgment} You may abandon {subtype} to {Discover} {subtype} with cost {e} or more and {materialize} it.",
        "subtype: warrior, e: 2",
    );
}

#[test]
fn test_round_trip_judgment_pay_to_banish_allies_then_materialize() {
    assert_round_trip_with_expected(
        "{Judgment} You may pay {e} to {banish} {up-to-n-allies}, then {materialize} {it-or-them}.",
        "e: 1, number: 2",
        "{Judgment} You may pay {e} to {Banish} {up-to-n-allies}, then {Materialize} them.",
        "e: 1, number: 2",
    );
}

#[test]
fn test_round_trip_materialized_banish_enemy_until_character_leaves_play() {
    assert_round_trip("{Materialized} {Banish} an enemy until this character leaves play.", "");
}

#[test]
fn test_round_trip_materialized_banish_enemy_until_next_main_phase() {
    assert_round_trip("{Materialized} {Banish} an enemy until your next main phase.", "");
}

#[test]
fn test_round_trip_materialized_event_in_void_gains_reclaim() {
    assert_round_trip_with_expected(
        "{Materialized} An event in your void gains {reclaim} equal to its cost this turn.",
        "",
        "{Materialized} An event in your void gains {reclaim} equal to its cost.",
        "",
    );
}

#[test]
fn test_round_trip_materialized_copy_next_event() {
    assert_round_trip(
        "{Materialized} Copy the next event you play {this-turn-times}.",
        "number: 1",
    );
}

#[test]
fn test_round_trip_materialized_disable_activated_abilities() {
    assert_round_trip(
        "{Materialized} Disable the activated abilities of an enemy while this character is in play.",
        "",
    );
}

#[test]
fn test_round_trip_materialized_draw_subtype() {
    assert_round_trip("{Materialized} Draw {a-subtype} from your deck.", "subtype: warrior");
}

#[test]
fn test_round_trip_judgment_each_allied_subtype_gains_spark_for_each_allied_subtype() {
    assert_round_trip(
        "{Judgment} Each allied {subtype} gains spark equal to the number of allied {plural-subtype}.",
        "subtype: warrior",
    );
}

#[test]
fn test_round_trip_judgment_each_player_shuffles_hand_and_void_and_draws() {
    assert_round_trip(
        "{Judgment} Each player shuffles their hand and void into their deck and then draws {cards}.",
        "cards: 3",
    );
}
