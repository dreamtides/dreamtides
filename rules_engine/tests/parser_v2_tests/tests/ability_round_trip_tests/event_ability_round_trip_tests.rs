use parser_v2::serializer::ability_serializer;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_until_end_of_turn_when_you_play_event_copy_it() {
    let original = "Until end of turn, when you play an event, copy it.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_multiply_your_energy() {
    let original = "{MultiplyBy} the amount of {energy-symbol} you have.";
    let parsed = parse_ability(original, "number: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialize_random_characters_with_cost() {
    let original = "{Materialize} {n-random-characters} with cost {e} or less from your deck.";
    let parsed = parse_ability(original, "number: 3, e: 5");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialize_random_subtype_from_deck() {
    let original = "{Materialize} {n-random-characters} {subtype} from your deck.";
    let parsed = parse_ability(original, "number: 2, subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_until_end_of_turn_when_you_play_a_character_draw_cards() {
    let original = "Until end of turn, when you play a character, draw {cards}.";
    let parsed = parse_ability(original, "cards: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_until_end_of_turn_when_an_ally_leaves_play_gain_energy() {
    let original = "Until end of turn, when an ally leaves play, gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_ally_gains_spark_for_each_allied_subtype() {
    let original = "An ally gains +{s} spark for each allied {subtype}.";
    let parsed = parse_ability(original, "s: 2, subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_draw_cards_for_each_allied_subtype() {
    let original = "{Materialized} Draw {cards} for each allied {subtype}.";
    let parsed = parse_ability(original, "cards: 2, subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_score_points_for_each_card_played_this_turn() {
    let original = "Gain {points} for each card you have played this turn.";
    let parsed = parse_ability(original, "points: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_draw_cards_for_each_card_played_this_turn() {
    let original = "Draw {cards} for each card you have played this turn.";
    let parsed = parse_ability(original, "cards: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_event_foresee() {
    let original = "{Foresee}.";
    let parsed = parse_ability(original, "foresee: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_event_kindle() {
    let original = "{Kindle}.";
    let parsed = parse_ability(original, "k: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_event_discover() {
    let original = "{Discover} {a-subtype}.";
    let parsed = parse_ability(original, "subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_event_prevent() {
    let original = "{Prevent} a played card.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_event_put_on_top_of_opponent_deck() {
    let original = "Put it on top of the opponent's deck.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_event_dissolve() {
    let original = "{Dissolve} an enemy.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_enemy_with_spark_or_less() {
    let original = "{Dissolve} an enemy with spark {s} or less.";
    let parsed = parse_ability(original, "s: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_enemy_with_spark_or_more() {
    let original = "{Dissolve} an enemy with spark {s} or more.";
    let parsed = parse_ability(original, "s: 5");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_banish_enemy_with_cost_or_less() {
    let original = "{Banish} an enemy with cost {e} or less.";
    let parsed = parse_ability(original, "e: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_banish_ally_when_it_leaves_play() {
    let original = "{Banish} an ally when it leaves play.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_banish_this_character_when_it_leaves_play() {
    let original = "{Banish} this character when it leaves play.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_banish_enemy_when_it_leaves_play() {
    let original = "{Banish} an enemy when it leaves play.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_enemy_with_cost_or_more() {
    let original = "{Dissolve} an enemy with cost {e} or more.";
    let parsed = parse_ability(original, "e: 4");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discard_chosen_character_from_opponent_hand() {
    let original = "Discard a chosen character from the opponent's hand.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discard_chosen_card_with_cost_from_opponent_hand() {
    let original = "Discard a chosen card with cost {e} or less from the opponent's hand.";
    let parsed = parse_ability(original, "e: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_prevent_event_unless_opponent_pays() {
    let original = "{Prevent} a played event unless the opponent pays {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discover_fast_character() {
    let original = "{Discover} a {fast} character.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discover_fast_card() {
    let original = "{Discover} a {fast} card.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discover_fast_subtype() {
    let original = "{Discover} a {fast} {subtype}.";
    let parsed = parse_ability(original, "subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discover_fast_character_with_spark() {
    let original = "{Discover} a {fast} character with spark {s} or less.";
    let parsed = parse_ability(original, "s: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discover_fast_card_with_cost() {
    let original = "{Discover} a {fast} character with cost {e} or less.";
    let parsed = parse_ability(original, "e: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_return_up_to_n_events_from_void_to_hand() {
    let original = "Return {up-to-n-events} from your void to your hand.";
    let parsed = parse_ability(original, "number: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_event_in_void_gains_reclaim_this_turn() {
    let original = "An event in your void gains {reclaim-for-cost} this turn.";
    let parsed = parse_ability(original, "reclaim: 0");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_prevent_a_played_enemy_card() {
    let original = "{Prevent} a played enemy card.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discover_character_with_materialized_ability() {
    let original = "{Discover} a character with a {materialized} ability.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discover_character_with_activated_ability() {
    let original = "{Discover} a character with an activated ability.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialize_n_figments() {
    let original = "{Materialize} {n-figments}.";
    let parsed = parse_ability(original, "figment: celestial, number: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialize_a_figment_for_each_card_played_this_turn() {
    let original = "{Materialize} {a-figment} for each card you have played this turn.";
    let parsed = parse_ability(original, "figment: shadow");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialize_a_figment_for_each_ally() {
    let original = "{Materialize} {a-figment} for each ally.";
    let parsed = parse_ability(original, "figment: shadow");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialize_n_figments_for_each_allied_subtype() {
    let original = "{Materialize} {n-figments} for each allied {subtype}.";
    let parsed = parse_ability(original, "figment: celestial, number: 2, subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialize_a_copy_of_target() {
    let original = "{Materialize} a copy of an ally.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialize_n_copies_of_target() {
    let original = "{Materialize} 3 copies of an enemy.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialize_copies_equal_to_allies() {
    let original =
        "{Materialize} a number of copies of that character equal to the number of allies.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialize_copies_equal_to_energy_spent() {
    let original =
        "{Materialize} a number of copies of an ally equal to the amount of {energy-symbol} spent.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_copy_next_event() {
    let original = "Copy the next event you play {this-turn-times}.";
    let parsed = parse_ability(original, "number: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_all_cards_in_void_gain_reclaim_equal_to_cost() {
    let original = "All cards currently in your void gain {reclaim} equal to their cost this turn.";
    let parsed = parse_ability(original, "reclaim: 0");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_trigger_additional_judgment_phase() {
    let original = "At the end of this turn, trigger an additional {JudgmentPhaseName} phase.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_copy_next_event() {
    let original = "Copy the next event you play {this-turn-times}.";
    let parsed = parse_ability(original, "number: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_copy_next_fast_character() {
    let original = "Copy the next {fast} character you play {this-turn-times}.";
    let parsed = parse_ability(original, "number: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_multiply_energy_gain_from_card_effects() {
    let original =
        "{MultiplyBy} the amount of {energy-symbol} you gain from card effects this turn.";
    let parsed = parse_ability(original, "number: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_multiply_card_draw_from_card_effects() {
    let original = "{MultiplyBy} the number of cards you draw from card effects this turn.";
    let parsed = parse_ability(original, "number: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_each_allied_subtype_gains_spark_for_each_allied_subtype() {
    let original =
        "Each allied {subtype} gains spark equal to the number of allied {plural-subtype}.";
    let parsed = parse_ability(original, "subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_each_player_shuffles_hand_and_void_and_draws() {
    let original =
        "Each player shuffles their hand and void into their deck and then draws {cards}.";
    let parsed = parse_ability(original, "cards: 5");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_return_all_but_one_ally_draw_cards() {
    let original = "Return all but one ally to hand. Draw {cards} for each ally returned.";
    let parsed = parse_ability(original, "cards: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_take_an_extra_turn_after_this_one() {
    let original = "Take an extra turn after this one.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_modal_return_enemy_or_draw_cards() {
    let original = "{ChooseOne}\n{bullet} {mode1-cost}: Return an enemy to hand.\n{bullet} {mode2-cost}: Draw {cards}.";
    let parsed = parse_ability(original, "mode1-cost: 1, mode2-cost: 2, cards: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_prevent_played_event_which_could_dissolve_ally() {
    let original = "{Prevent} a played event which could {dissolve} an ally.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_gain_energy_equal_to_that_characters_cost() {
    let original = "Gain {energy-symbol} equal to that character's cost.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_gain_energy_equal_to_this_characters_cost() {
    let original = "Gain {energy-symbol} equal to this character's cost.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_gain_energy_equal_to_an_allys_cost() {
    let original = "Gain {energy-symbol} equal to an ally's cost.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_put_a_card_from_void_on_top_of_deck() {
    let original = "Put a card from your void on top of your deck.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_put_up_to_n_cards_from_void_on_top_of_deck() {
    let original = "Put {up-to-n-cards} cards from your void on top of your deck.";
    let parsed = parse_ability(original, "number: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_put_up_to_n_events_from_void_on_top_of_deck() {
    let original = "Put {up-to-n-cards} events from your void on top of your deck.";
    let parsed = parse_ability(original, "number: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_trigger_judgment_ability_of_each_ally() {
    let original = "Trigger the {Judgment} ability of each ally.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_trigger_judgment_ability_of_an_ally() {
    let original = "Trigger the {Judgment} ability of an ally.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_and_gain_energy_for_spark() {
    let original =
        "Abandon an ally and gain {energy-symbol} for each point of spark that character had.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_at_end_of_turn() {
    let original = "Abandon an ally at end of turn.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_trigger_judgment_ability_of_three_allies() {
    let original = "Trigger the {Judgment} ability of 3 allies.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_enemies_with_cost_less_than_or_equal_to_allies() {
    let original =
        "{Dissolve} all enemies with cost less than or equal to the number of allied characters.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_allies_with_cost_less_than_or_equal_to_abandoned_count() {
    let original =
        "{Dissolve} all allies with cost less than or equal to the number of allies abandoned this turn.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_ally_cannot_be_dissolved_this_turn() {
    let original = "An ally cannot be {dissolved} this turn.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_enemy_cannot_be_dissolved_this_turn() {
    let original = "An enemy cannot be {dissolved} this turn.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_banish_enemy_opponent_gains_points_equal_to_its_spark() {
    let original = "{Banish} an enemy. The opponent gains points equal to its spark.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_it_opponent_gains_points_equal_to_its_spark() {
    let original = "{Dissolve} it. The opponent gains points equal to its spark.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_ally_opponent_loses_points() {
    let original = "{Dissolve} an ally. The opponent loses {points}.";
    let parsed = parse_ability(original, "points: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_opponent_loses_points() {
    let original = "The opponent loses {points}.";
    let parsed = parse_ability(original, "points: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_ally_gains_aegis_this_turn() {
    let original = "An ally gains {Aegis} this turn.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_this_character_gains_aegis_this_turn() {
    let original = "This character gains {Aegis} this turn.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_ally_gains_spark_until_next_main_for_each_ally() {
    let original = "An ally gains +{s} spark until your next main phase for each ally.";
    let parsed = parse_ability(original, "s: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_this_character_gains_spark_until_next_main_for_each_subtype() {
    let original =
        "This character gains +{s} spark until your next main phase for each allied {subtype}.";
    let parsed = parse_ability(original, "s: 2, subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}
