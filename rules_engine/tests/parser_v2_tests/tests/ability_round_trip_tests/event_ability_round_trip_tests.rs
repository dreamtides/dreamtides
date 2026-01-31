use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_until_end_of_turn_when_you_play_event_copy_it() {
    assert_round_trip("Until end of turn, when you play an event, copy it.", "");
}

#[test]
fn test_round_trip_multiply_your_energy() {
    assert_round_trip(
        "{MultiplyBy} the amount of {energy-symbol} you have.",
        "number: 2",
    );
}

#[test]
fn test_round_trip_materialize_random_characters_with_cost() {
    assert_round_trip(
        "{Materialize} {n-random-characters} with cost {e} or less from your deck.",
        "number: 3, e: 5",
    );
}

#[test]
fn test_round_trip_materialize_random_subtype_from_deck() {
    assert_round_trip(
        "{Materialize} {n-random-characters} {subtype} from your deck.",
        "number: 2, subtype: warrior",
    );
}

#[test]
fn test_round_trip_until_end_of_turn_when_you_play_a_character_draw_cards() {
    assert_round_trip(
        "Until end of turn, when you play a character, draw {cards}.",
        "cards: 2",
    );
}

#[test]
fn test_round_trip_until_end_of_turn_when_an_ally_leaves_play_gain_energy() {
    assert_round_trip(
        "Until end of turn, when an ally leaves play, gain {e}.",
        "e: 1",
    );
}

#[test]
fn test_round_trip_ally_gains_spark_for_each_allied_subtype() {
    assert_round_trip(
        "An ally gains +{s} spark for each allied {subtype}.",
        "s: 2, subtype: warrior",
    );
}

#[test]
fn test_round_trip_materialized_draw_cards_for_each_allied_subtype() {
    assert_round_trip(
        "{Materialized} Draw {cards} for each allied {subtype}.",
        "cards: 2, subtype: warrior",
    );
}

#[test]
fn test_round_trip_score_points_for_each_card_played_this_turn() {
    assert_round_trip(
        "Gain {points} for each card you have played this turn.",
        "points: 3",
    );
}

#[test]
fn test_round_trip_draw_cards_for_each_card_played_this_turn() {
    assert_round_trip(
        "Draw {cards} for each card you have played this turn.",
        "cards: 2",
    );
}

#[test]
fn test_round_trip_event_foresee() {
    assert_round_trip("{Foresee}.", "foresee: 3");
}

#[test]
fn test_round_trip_event_kindle() {
    assert_round_trip("{Kindle}.", "k: 1");
}

#[test]
fn test_round_trip_event_discover() {
    assert_round_trip("{Discover} {a-subtype}.", "subtype: warrior");
}

#[test]
fn test_round_trip_event_prevent() {
    assert_round_trip("{Prevent} a played card.", "");
}

#[test]
fn test_round_trip_event_put_on_top_of_opponent_deck() {
    assert_round_trip("Put it on top of the opponent's deck.", "");
}

#[test]
fn test_round_trip_event_dissolve() {
    assert_round_trip("{Dissolve} an enemy.", "");
}

#[test]
fn test_round_trip_dissolve_enemy_with_spark_or_less() {
    assert_round_trip("{Dissolve} an enemy with spark {s} or less.", "s: 3");
}

#[test]
fn test_round_trip_dissolve_enemy_with_spark_or_more() {
    assert_round_trip("{Dissolve} an enemy with spark {s} or more.", "s: 5");
}

#[test]
fn test_round_trip_banish_enemy_with_cost_or_less() {
    assert_round_trip("{Banish} an enemy with cost {e} or less.", "e: 2");
}

#[test]
fn test_round_trip_banish_ally_when_it_leaves_play() {
    assert_round_trip("{Banish} an ally when it leaves play.", "");
}

#[test]
fn test_round_trip_banish_this_character_when_it_leaves_play() {
    assert_round_trip("{Banish} this character when it leaves play.", "");
}

#[test]
fn test_round_trip_banish_enemy_when_it_leaves_play() {
    assert_round_trip("{Banish} an enemy when it leaves play.", "");
}

#[test]
fn test_round_trip_dissolve_enemy_with_cost_or_more() {
    assert_round_trip("{Dissolve} an enemy with cost {e} or more.", "e: 4");
}

#[test]
fn test_round_trip_discard_chosen_character_from_opponent_hand() {
    assert_round_trip("Discard a chosen character from the opponent's hand.", "");
}

#[test]
fn test_round_trip_discard_chosen_card_with_cost_from_opponent_hand() {
    assert_round_trip(
        "Discard a chosen card with cost {e} or less from the opponent's hand.",
        "e: 2",
    );
}

#[test]
fn test_round_trip_prevent_event_unless_opponent_pays() {
    assert_round_trip(
        "{Prevent} a played event unless the opponent pays {e}.",
        "e: 1",
    );
}

#[test]
fn test_round_trip_discover_fast_character() {
    assert_round_trip("{Discover} a {fast} character.", "");
}

#[test]
fn test_round_trip_discover_fast_card() {
    assert_round_trip("{Discover} a {fast} card.", "");
}

#[test]
fn test_round_trip_discover_fast_subtype() {
    assert_round_trip("{Discover} a {fast} {subtype}.", "subtype: warrior");
}

#[test]
fn test_round_trip_discover_fast_character_with_spark() {
    assert_round_trip(
        "{Discover} a {fast} character with spark {s} or less.",
        "s: 2",
    );
}

#[test]
fn test_round_trip_discover_fast_card_with_cost() {
    assert_round_trip(
        "{Discover} a {fast} character with cost {e} or less.",
        "e: 3",
    );
}

#[test]
fn test_round_trip_return_up_to_n_events_from_void_to_hand() {
    assert_round_trip(
        "Return {up-to-n-events} from your void to your hand.",
        "number: 3",
    );
}

#[test]
fn test_round_trip_event_in_void_gains_reclaim_this_turn() {
    assert_round_trip(
        "An event in your void gains {reclaim-for-cost} this turn.",
        "reclaim: 0",
    );
}

#[test]
fn test_round_trip_prevent_a_played_enemy_card() {
    assert_round_trip("{Prevent} a played enemy card.", "");
}

#[test]
fn test_round_trip_discover_character_with_materialized_ability() {
    assert_round_trip("{Discover} a character with a {materialized} ability.", "");
}

#[test]
fn test_round_trip_discover_character_with_activated_ability() {
    assert_round_trip("{Discover} a character with an activated ability.", "");
}

#[test]
fn test_round_trip_materialize_n_figments() {
    assert_round_trip(
        "{Materialize} {n-figments}.",
        "figment: celestial, number: 2",
    );
}

#[test]
fn test_round_trip_materialize_a_figment_for_each_card_played_this_turn() {
    assert_round_trip(
        "{Materialize} {a-figment} for each card you have played this turn.",
        "figment: shadow",
    );
}

#[test]
fn test_round_trip_materialize_a_figment_for_each_ally() {
    assert_round_trip(
        "{Materialize} {a-figment} for each ally.",
        "figment: shadow",
    );
}

#[test]
fn test_round_trip_materialize_n_figments_for_each_allied_subtype() {
    assert_round_trip(
        "{Materialize} {n-figments} for each allied {subtype}.",
        "figment: celestial, number: 2, subtype: warrior",
    );
}

#[test]
fn test_round_trip_materialize_a_copy_of_target() {
    assert_round_trip("{Materialize} a copy of an ally.", "");
}

#[test]
fn test_round_trip_materialize_n_copies_of_target() {
    assert_round_trip("{Materialize} 3 copies of an enemy.", "");
}

#[test]
fn test_round_trip_materialize_copies_equal_to_allies() {
    assert_round_trip(
        "{Materialize} a number of copies of that character equal to the number of allies.",
        "",
    );
}

#[test]
fn test_round_trip_materialize_copies_equal_to_energy_spent() {
    assert_round_trip(
        "{Materialize} a number of copies of an ally equal to the amount of {energy-symbol} spent.",
        "",
    );
}

#[test]
fn test_round_trip_copy_next_event() {
    assert_round_trip(
        "Copy the next event you play {this-turn-times}.",
        "number: 2",
    );
}

#[test]
fn test_round_trip_all_cards_in_void_gain_reclaim_equal_to_cost() {
    assert_round_trip(
        "All cards currently in your void gain {reclaim} equal to their cost this turn.",
        "reclaim: 0",
    );
}

#[test]
fn test_round_trip_up_to_two_characters_in_void_gain_reclaim() {
    assert_round_trip(
        "Up to 2 characters in your void gain {reclaim} equal to their cost this turn.",
        "reclaim-up-to: 2",
    );
}

#[test]
fn test_round_trip_any_number_of_events_in_void_gain_reclaim() {
    assert_round_trip(
        "Any number of events in your void gain {reclaim} equal to their cost this turn.",
        "reclaim-any-number: true",
    );
}

#[test]
fn test_round_trip_three_characters_in_void_gain_reclaim() {
    assert_round_trip(
        "3 characters in your void gain {reclaim} equal to their cost this turn.",
        "reclaim-exactly: 3",
    );
}

#[test]
fn test_round_trip_two_or_more_characters_in_void_gain_reclaim() {
    assert_round_trip(
        "2 or more characters in your void gain {reclaim} equal to their cost this turn.",
        "reclaim-or-more: 2",
    );
}

#[test]
fn test_round_trip_trigger_additional_judgment_phase() {
    assert_round_trip(
        "At the end of this turn, trigger an additional {JudgmentPhaseName} phase.",
        "",
    );
}

#[test]
fn test_round_trip_copy_next_fast_character() {
    assert_round_trip(
        "Copy the next {fast} character you play {this-turn-times}.",
        "number: 1",
    );
}

#[test]
fn test_round_trip_multiply_energy_gain_from_card_effects() {
    assert_round_trip(
        "{MultiplyBy} the amount of {energy-symbol} you gain from card effects this turn.",
        "number: 2",
    );
}

#[test]
fn test_round_trip_multiply_card_draw_from_card_effects() {
    assert_round_trip(
        "{MultiplyBy} the number of cards you draw from card effects this turn.",
        "number: 3",
    );
}

#[test]
fn test_round_trip_each_allied_subtype_gains_spark_for_each_allied_subtype() {
    assert_round_trip(
        "Each allied {subtype} gains spark equal to the number of allied {plural-subtype}.",
        "subtype: warrior",
    );
}

#[test]
fn test_round_trip_each_player_shuffles_hand_and_void_and_draws() {
    assert_round_trip(
        "Each player shuffles their hand and void into their deck and then draws {cards}.",
        "cards: 5",
    );
}

#[test]
fn test_round_trip_return_all_but_one_ally_draw_cards() {
    assert_round_trip(
        "Return all but one ally to hand. Draw {cards} for each ally returned.",
        "cards: 2",
    );
}

#[test]
fn test_round_trip_return_allies_draw_cards_for_each_allied_subtype_returned() {
    assert_round_trip(
        "Return all allies to hand. Draw {cards} for each allied {subtype} returned.",
        "subtype: warrior, cards: 1",
    );
}

#[test]
fn test_round_trip_take_an_extra_turn_after_this_one() {
    assert_round_trip("Take an extra turn after this one.", "");
}

#[test]
fn test_round_trip_modal_return_enemy_or_draw_cards() {
    assert_round_trip(
        "{ChooseOne}\n{bullet} {mode1-cost}: Return an enemy to hand.\n{bullet} {mode2-cost}: Draw {cards}.",
        "mode1-cost: 1, mode2-cost: 2, cards: 3",
    );
}

#[test]
fn test_round_trip_prevent_played_event_which_could_dissolve_ally() {
    assert_round_trip(
        "{Prevent} a played event which could {dissolve} an ally.",
        "",
    );
}

#[test]
fn test_round_trip_gain_energy_equal_to_that_characters_cost() {
    assert_round_trip("Gain {energy-symbol} equal to that character's cost.", "");
}

#[test]
fn test_round_trip_gain_energy_equal_to_this_characters_cost() {
    assert_round_trip("Gain {energy-symbol} equal to this character's cost.", "");
}

#[test]
fn test_round_trip_gain_energy_equal_to_an_allys_cost() {
    assert_round_trip("Gain {energy-symbol} equal to an ally's cost.", "");
}

#[test]
fn test_round_trip_put_a_card_from_void_on_top_of_deck() {
    assert_round_trip("Put a card from your void on top of your deck.", "");
}

#[test]
fn test_round_trip_put_up_to_n_cards_from_void_on_top_of_deck() {
    assert_round_trip(
        "Put {up-to-n-cards} cards from your void on top of your deck.",
        "number: 3",
    );
}

#[test]
fn test_round_trip_put_up_to_n_events_from_void_on_top_of_deck() {
    assert_round_trip(
        "Put {up-to-n-cards} events from your void on top of your deck.",
        "number: 2",
    );
}

#[test]
fn test_round_trip_trigger_judgment_ability_of_each_ally() {
    assert_round_trip("Trigger the {Judgment} ability of each ally.", "");
}

#[test]
fn test_round_trip_trigger_judgment_ability_of_an_ally() {
    assert_round_trip("Trigger the {Judgment} ability of an ally.", "");
}

#[test]
fn test_round_trip_abandon_and_gain_energy_for_spark() {
    assert_round_trip(
        "Abandon an ally and gain {energy-symbol} for each point of spark that character had.",
        "",
    );
}

#[test]
fn test_round_trip_abandon_at_end_of_turn() {
    assert_round_trip("Abandon an ally at end of turn.", "");
}

#[test]
fn test_round_trip_trigger_judgment_ability_of_three_allies() {
    assert_round_trip("Trigger the {Judgment} ability of 3 allies.", "");
}

#[test]
fn test_round_trip_dissolve_enemies_with_cost_less_than_or_equal_to_allies() {
    assert_round_trip(
        "{Dissolve} all enemies with cost less than or equal to the number of allied characters.",
        "",
    );
}

#[test]
fn test_round_trip_dissolve_allies_with_cost_less_than_or_equal_to_abandoned_count() {
    assert_round_trip(
        "{Dissolve} all allies with cost less than or equal to the number of allies abandoned this turn.",
        "",
    );
}

#[test]
fn test_round_trip_ally_cannot_be_dissolved_this_turn() {
    assert_round_trip("An ally cannot be {dissolved} this turn.", "");
}

#[test]
fn test_round_trip_enemy_cannot_be_dissolved_this_turn() {
    assert_round_trip("An enemy cannot be {dissolved} this turn.", "");
}

#[test]
fn test_round_trip_banish_enemy_opponent_gains_points_equal_to_its_spark() {
    assert_round_trip(
        "{Banish} an enemy. The opponent gains points equal to its spark.",
        "",
    );
}

#[test]
fn test_round_trip_dissolve_it_opponent_gains_points_equal_to_its_spark() {
    assert_round_trip(
        "{Dissolve} it. The opponent gains points equal to its spark.",
        "",
    );
}

#[test]
fn test_round_trip_dissolve_ally_opponent_loses_points() {
    assert_round_trip(
        "{Dissolve} an ally. The opponent loses {points}.",
        "points: 2",
    );
}

#[test]
fn test_round_trip_opponent_loses_points() {
    assert_round_trip("The opponent loses {points}.", "points: 1");
}

#[test]
fn test_round_trip_ally_gains_aegis_this_turn() {
    assert_round_trip("An ally gains {Aegis} this turn.", "");
}

#[test]
fn test_round_trip_this_character_gains_aegis_this_turn() {
    assert_round_trip("This character gains {Aegis} this turn.", "");
}

#[test]
fn test_round_trip_ally_gains_spark_until_next_main_for_each_ally() {
    assert_round_trip(
        "An ally gains +{s} spark until your next main phase for each ally.",
        "s: 1",
    );
}

#[test]
fn test_round_trip_this_character_gains_spark_until_next_main_for_each_subtype() {
    assert_round_trip(
        "This character gains +{s} spark until your next main phase for each allied {subtype}.",
        "s: 2, subtype: warrior",
    );
}

#[test]
fn test_round_trip_gain_twice_that_much_energy_instead() {
    assert_round_trip("Gain twice that much {energy-symbol} instead.", "");
}

#[test]
fn test_round_trip_materialize_character_from_void() {
    assert_round_trip("{Materialize} a character from your void.", "");
}

#[test]
fn test_round_trip_materialize_subtype_from_void() {
    assert_round_trip("{Materialize} {a-subtype} from your void.", "subtype: warrior");
}

#[test]
fn test_round_trip_banish_ally_then_materialize_it() {
    assert_round_trip("{Banish} an ally, then {Materialize} it.", "");
}

#[test]
fn test_round_trip_opponent_pays_energy_cost() {
    assert_round_trip("The opponent pays {e}.", "e: 2");
}

#[test]
fn test_round_trip_pay_energy_cost() {
    assert_round_trip("Pay {e}.", "e: 1");
}

#[test]
fn test_round_trip_spend_all_energy_dissolve_enemy() {
    assert_round_trip(
        "Spend all your {energy-symbol}. {Dissolve} an enemy with cost less than or equal to the amount spent.",
        "",
    );
}

#[test]
fn test_round_trip_spend_all_energy_draw_and_discard() {
    assert_round_trip(
        "Spend all your {energy-symbol}. Draw cards equal to the amount spent, then discard that many cards.",
        "",
    );
}
