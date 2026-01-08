use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_dissolve_ally_that_is_not_subtype() {
    assert_round_trip("{Dissolve} an ally that is not {a-subtype}.", "subtype: warrior");
}

#[test]
fn test_round_trip_abandon_ally_with_materialized_ability() {
    assert_round_trip(
        "Abandon an ally with a {materialized} ability: Gain {e}.",
        "e: 2",
    );
}

#[test]
fn test_round_trip_abandon_ally_with_activated_ability() {
    assert_round_trip(
        "Abandon an ally with an activated ability: Draw {cards}.",
        "cards: 1",
    );
}

#[test]
fn test_round_trip_when_materialize_fast_ally_draw_cards() {
    assert_round_trip(
        "When you {materialize} a fast ally, draw {cards}.",
        "cards: 1",
    );
}

#[test]
fn test_round_trip_dissolve_ally_with_cost() {
    assert_round_trip("{Dissolve} an ally with cost {e} or less.", "e: 3");
}

#[test]
fn test_round_trip_dissolve_fast_ally_with_cost() {
    assert_round_trip("{Dissolve} a fast ally with cost {e} or less.", "e: 2");
}

#[test]
fn test_round_trip_dissolve_all_allies_that_are_not_subtype() {
    assert_round_trip(
        "{Dissolve} all allies that are not {a-subtype}.",
        "subtype: warrior",
    );
}

#[test]
fn test_round_trip_when_play_fast_character_gain_energy() {
    assert_round_trip("When you play a {fast} character, gain {e}.", "e: 1");
}

#[test]
fn test_round_trip_abandon_ally_with_cost_greater_than() {
    assert_round_trip("Abandon an ally with cost {e} or more: {Kindle}.", "e: 4, k: 1");
}

#[test]
fn test_round_trip_dissolve_enemy_subtype() {
    assert_round_trip("{Dissolve} an enemy {subtype}.", "subtype: warrior");
}

#[test]
fn test_round_trip_dissolve_enemy_that_is_not_subtype() {
    assert_round_trip("{Dissolve} an enemy that is not {a-subtype}.", "subtype: warrior");
}

#[test]
fn test_round_trip_when_enemy_materialized_ability_enters_gain_energy() {
    assert_round_trip(
        "When an enemy with a {materialized} ability is {materialized}, gain {e}.",
        "e: 1",
    );
}

#[test]
fn test_round_trip_dissolve_enemy_with_activated_ability() {
    assert_round_trip("{Dissolve} an enemy with an activated ability.", "");
}

#[test]
fn test_round_trip_dissolve_enemy_with_cost_compared_to_abandoned() {
    assert_round_trip(
        "Abandon an ally: {Dissolve} an enemy with cost less than the abandoned ally's cost.",
        "",
    );
}

#[test]
fn test_round_trip_dissolve_enemy_with_spark_compared_to_abandoned_count() {
    assert_round_trip(
        "Abandon an ally: {Dissolve} an enemy with spark less than the number of allies abandoned this turn.",
        "",
    );
}

#[test]
fn test_round_trip_when_materialize_fast_enemy_gain_energy() {
    assert_round_trip(
        "When the opponent {materializes} a fast enemy, gain {e}.",
        "e: 1",
    );
}

#[test]
fn test_round_trip_dissolve_character_that_is_not_subtype() {
    assert_round_trip(
        "{Dissolve} a character that is not {a-subtype}.",
        "subtype: warrior",
    );
}

#[test]
fn test_round_trip_dissolve_character_with_spark() {
    assert_round_trip("{Dissolve} a character with spark {s} or less.", "s: 3");
}

#[test]
fn test_round_trip_dissolve_character_with_cost_compared_to_controlled() {
    assert_round_trip(
        "{Dissolve} a character with cost less than the number of allied characters.",
        "",
    );
}

#[test]
fn test_round_trip_dissolve_character_with_cost_compared_to_abandoned() {
    assert_round_trip(
        "Abandon an ally: {Dissolve} a character with cost less than the abandoned ally's cost.",
        "",
    );
}

#[test]
fn test_round_trip_dissolve_character_with_spark_compared_to_abandoned() {
    assert_round_trip(
        "Abandon an ally: {Dissolve} a character with spark less than the abandoned ally's spark.",
        "",
    );
}

#[test]
fn test_round_trip_dissolve_character_with_spark_compared_to_abandoned_count() {
    assert_round_trip(
        "Abandon an ally: {Dissolve} a character with spark less than the number of allies abandoned this turn.",
        "",
    );
}

#[test]
fn test_round_trip_dissolve_character_with_cost_compared_to_void_count() {
    assert_round_trip(
        "{Dissolve} a character with cost less than the number of cards in your void.",
        "",
    );
}

#[test]
fn test_round_trip_dissolve_all_characters_that_are_not_subtype() {
    assert_round_trip(
        "{Dissolve} all characters that are not {plural-subtype}.",
        "subtype: warrior",
    );
}

#[test]
fn test_round_trip_when_play_fast_character_with_spark() {
    assert_round_trip(
        "When you play a {fast} character with spark {s} or less, gain {e}.",
        "s: 2, e: 1",
    );
}

#[test]
fn test_round_trip_when_play_fast_character_that_is_not_subtype() {
    assert_round_trip(
        "When you play a {fast} character that is not {a-subtype}, draw {cards}.",
        "subtype: warrior, cards: 1",
    );
}

#[test]
fn test_round_trip_gain_energy_for_each_ally() {
    assert_round_trip("Gain {e} for each ally.", "e: 1");
}

#[test]
fn test_round_trip_gain_spark_for_each_allied_subtype() {
    assert_round_trip(
        "An ally gains +{s} spark for each allied {subtype}.",
        "subtype: warrior, s: 1",
    );
}

#[test]
fn test_round_trip_draw_cards_for_each_enemy() {
    assert_round_trip("Draw {cards} for each enemy.", "cards: 1");
}

#[test]
fn test_round_trip_gain_energy_for_each_character() {
    assert_round_trip("Gain {e} for each character.", "e: 1");
}

#[test]
fn test_round_trip_gain_points_for_each_card() {
    assert_round_trip("Gain {points} for each card.", "points: 1");
}

#[test]
fn test_round_trip_gain_energy_for_each_card_in_your_void() {
    assert_round_trip("Gain {e} for each card in your void.", "e: 1");
}

#[test]
fn test_round_trip_dissolve_all_characters_with_materialized_ability() {
    assert_round_trip("{Dissolve} all characters with {materialized} abilities.", "");
}

#[test]
fn test_round_trip_dissolve_all_characters_with_activated_ability() {
    assert_round_trip("{Dissolve} all characters with activated abilities.", "");
}

#[test]
fn test_round_trip_dissolve_all_enemies_with_materialized_ability() {
    assert_round_trip("{Dissolve} all enemies with {materialized} abilities.", "");
}

#[test]
fn test_round_trip_dissolve_all_allies_with_materialized_ability() {
    assert_round_trip("{Dissolve} all allies with {materialized} abilities.", "");
}

#[test]
fn test_round_trip_dissolve_all_allies_with_activated_ability() {
    assert_round_trip("{Dissolve} all allies with activated abilities.", "");
}

#[test]
fn test_round_trip_dissolve_all_allies_that_are_not_subtype() {
    assert_round_trip("{Dissolve} all allies that are not {plural-subtype}.", "subtype: warrior");
}

#[test]
fn test_round_trip_dissolve_all_allies_with_spark() {
    assert_round_trip("{Dissolve} all allies with spark {s} or less.", "s: 2");
}

#[test]
fn test_round_trip_dissolve_all_allies_with_cost() {
    assert_round_trip("{Dissolve} all allies with cost {e} or more.", "e: 4");
}

#[test]
fn test_round_trip_when_play_fast_character_with_materialized_ability() {
    assert_round_trip(
        "When you play a {fast} character with a {materialized} ability, gain {e}.",
        "e: 1",
    );
}

#[test]
fn test_round_trip_when_play_fast_character_with_activated_ability() {
    assert_round_trip(
        "When you play a {fast} character with an activated ability, draw {cards}.",
        "cards: 1",
    );
}

#[test]
fn test_round_trip_abandon_ally_with_cost_compared_to_controlled() {
    assert_round_trip(
        "Abandon an ally with cost less than the number of allied characters: Gain {e}.",
        "e: 2",
    );
}

#[test]
fn test_round_trip_abandon_ally_with_cost_compared_to_abandoned() {
    assert_round_trip(
        "Abandon an ally: Abandon an ally with cost less than the abandoned ally's cost.",
        "",
    );
}

#[test]
fn test_round_trip_abandon_ally_with_spark_compared_to_abandoned() {
    assert_round_trip(
        "Abandon an ally: Abandon an ally with spark less than the abandoned ally's spark.",
        "",
    );
}

#[test]
fn test_round_trip_abandon_ally_with_spark_compared_to_abandoned_count() {
    assert_round_trip(
        "Abandon an ally: Abandon an ally with spark less than the number of allies abandoned this turn.",
        "",
    );
}

#[test]
fn test_round_trip_abandon_ally_with_cost_compared_to_void_count() {
    assert_round_trip(
        "Abandon an ally with cost less than the number of cards in your void: Gain {e}.",
        "e: 1",
    );
}

#[test]
fn test_round_trip_dissolve_enemy_with_cost_compared_to_void_count() {
    assert_round_trip(
        "{Dissolve} an enemy with cost less than the number of cards in your void.",
        "",
    );
}

#[test]
fn test_round_trip_dissolve_all_enemies_with_spark_compared_to_energy_spent() {
    assert_round_trip(
        "{Dissolve} all enemies with spark less than the amount of {energy-symbol} paid.",
        "",
    );
}

#[test]
fn test_round_trip_dissolve_all_allies_with_cost_compared_to_abandoned() {
    assert_round_trip(
        "Abandon an ally: {Dissolve} all allies with cost less than the abandoned ally's cost.",
        "",
    );
}

#[test]
fn test_round_trip_dissolve_all_allies_with_spark_compared_to_abandoned() {
    assert_round_trip(
        "Abandon an ally: {Dissolve} all allies with spark less than the abandoned ally's spark.",
        "",
    );
}

#[test]
fn test_round_trip_dissolve_all_allies_with_spark_compared_to_abandoned_count() {
    assert_round_trip(
        "Abandon an ally: {Dissolve} all allies with spark less than the number of allies abandoned this turn.",
        "",
    );
}

#[test]
fn test_round_trip_dissolve_all_allies_with_cost_compared_to_void_count() {
    assert_round_trip(
        "{Dissolve} all allies with cost less than the number of cards in your void.",
        "",
    );
}

#[test]
fn test_round_trip_when_materialize_fast_fast_character() {
    assert_round_trip(
        "When you {materialize} a {fast} fast character, gain {e}.",
        "e: 1",
    );
}

#[test]
fn test_round_trip_gain_energy_for_each_event_in_your_void() {
    assert_round_trip("Gain {e} for each event in your void.", "e: 1");
}

#[test]
fn test_round_trip_gain_energy_for_each_character_in_your_void() {
    assert_round_trip("Gain {e} for each character in your void.", "e: 1");
}

#[test]
fn test_round_trip_gain_energy_for_each_card_in_enemy_void() {
    assert_round_trip("Gain {e} for each card in the opponent's void.", "e: 1");
}

#[test]
fn test_round_trip_gain_energy_for_each_character_in_enemy_void() {
    assert_round_trip(
        "Gain {e} for each character in the opponent's void.",
        "e: 1",
    );
}

#[test]
fn test_round_trip_gain_energy_for_each_event_in_enemy_void() {
    assert_round_trip("Gain {e} for each event in the opponent's void.", "e: 1");
}

#[test]
fn test_round_trip_gain_energy_for_each_ally_with_spark() {
    assert_round_trip("Gain {e} for each ally with spark {s} or more.", "e: 1, s: 2");
}

#[test]
fn test_round_trip_gain_energy_for_each_allied_character() {
    assert_round_trip("Gain {e} for each allied character.", "e: 1");
}

#[test]
fn test_round_trip_gain_energy_for_each_allied_event() {
    assert_round_trip("Gain {e} for each allied event.", "e: 1");
}

#[test]
fn test_round_trip_gain_energy_for_each_enemy_subtype() {
    assert_round_trip(
        "Gain {e} for each enemy {subtype}.",
        "e: 1, subtype: warrior",
    );
}

#[test]
fn test_round_trip_gain_energy_for_each_subtype() {
    assert_round_trip("Gain {e} for each {subtype}.", "e: 1, subtype: warrior");
}

#[test]
fn test_round_trip_gain_energy_for_each_event() {
    assert_round_trip("Gain {e} for each event.", "e: 1");
}

#[test]
fn test_round_trip_gain_energy_for_each_other_character() {
    assert_round_trip("Gain {e} for each other character.", "e: 1");
}

#[test]
fn test_round_trip_gain_energy_for_each_other_subtype() {
    assert_round_trip(
        "Gain {e} for each other {subtype}.",
        "e: 1, subtype: warrior",
    );
}

#[test]
fn test_round_trip_gain_energy_for_each_subtype_in_your_void() {
    assert_round_trip(
        "Gain {e} for each {subtype} in your void.",
        "e: 1, subtype: warrior",
    );
}

#[test]
fn test_round_trip_dissolve_all_other_characters() {
    assert_round_trip("{Dissolve} all other characters.", "");
}

#[test]
fn test_round_trip_dissolve_another_character() {
    assert_round_trip("{Dissolve} another character.", "");
}

#[test]
fn test_round_trip_dissolve_all_other_subtypes() {
    assert_round_trip("{Dissolve} all other {plural-subtype}.", "subtype: warrior");
}
