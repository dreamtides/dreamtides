use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_event_costs_if_character_dissolved() {
    assert_round_trip("This event costs {e} if a character dissolved this turn.", "e: 1");
}

#[test]
fn test_round_trip_character_costs_if_discarded_card_this_turn() {
    assert_round_trip("This character costs {e} if you have discarded a card this turn.", "e: 1");
}

#[test]
fn test_round_trip_character_costs_if_discarded_character_this_turn() {
    assert_round_trip(
        "This character costs {e} if you have discarded a character this turn.",
        "e: 1",
    );
}

#[test]
fn test_round_trip_character_costs_if_discarded_subtype_this_turn() {
    assert_round_trip(
        "This character costs {e} if you have discarded {a-subtype} this turn.",
        "e: 1, subtype: warrior",
    );
}

#[test]
fn test_round_trip_lose_maximum_energy_play_for_alternate_cost() {
    assert_round_trip("Lose {maximum-energy}: Play this event for {e}.", "max: 1, e: 0");
}

#[test]
fn test_round_trip_additional_cost_to_play() {
    assert_round_trip("To play this card, return an ally with cost {e} or more to hand.", "e: 4");
}

#[test]
fn test_round_trip_characters_in_hand_have_fast() {
    assert_round_trip("Characters in your hand have {fast}.", "");
}

#[test]
fn test_round_trip_disable_enemy_materialized_abilities() {
    assert_round_trip("Disable the {Materialized} abilities of enemies.", "");
}

#[test]
fn test_round_trip_once_per_turn_play_from_void() {
    assert_round_trip(
        "Once per turn, you may play a character with cost {e} or less from your void.",
        "e: 0",
    );
}

#[test]
fn test_round_trip_reveal_top_card_of_deck() {
    assert_round_trip("Reveal the top card of your deck.", "");
}

#[test]
fn test_round_trip_play_characters_from_top_of_deck() {
    assert_round_trip("You may play characters from the top of your deck.", "");
}

#[test]
fn test_round_trip_judgment_ability_of_allies_triggers_when_materialize() {
    assert_round_trip(
        "The '{Judgment}' ability of allies triggers when you {materialize} them.",
        "",
    );
}

#[test]
fn test_round_trip_spark_equal_to_allied_subtype() {
    assert_round_trip(
        "This character's spark is equal to the number of allied {plural-subtype}.",
        "subtype: warrior",
    );
}

#[test]
fn test_round_trip_spark_equal_to_cards_in_void() {
    assert_round_trip("This character's spark is equal to the number of cards in your void.", "");
}

#[test]
fn test_round_trip_while_in_void_allied_subtype_have_spark() {
    assert_round_trip(
        "While this card is in your void, allied {plural-subtype} have +{s} spark.",
        "subtype: warrior, s: 1",
    );
}

#[test]
fn test_round_trip_while_count_or_more_cards_in_void_have_reclaim() {
    assert_round_trip(
        "While you have {count} or more cards in your void, they have {reclaim} equal to their cost.",
        "count: 3",
    );
}

#[test]
fn test_round_trip_play_only_from_void() {
    assert_round_trip("You may only play this character from your void.", "");
}

#[test]
fn test_round_trip_with_allied_subtype_play_from_hand_or_void_for_cost() {
    assert_round_trip(
        "With an allied {subtype}, you may play this card from your hand or void for {e}.",
        "subtype: warrior, e: 2",
    );
}

#[test]
fn test_round_trip_cost_reduction_for_each_allied_character() {
    assert_round_trip("This card costs {e} less for each ally.", "e: 1");
}

#[test]
fn test_round_trip_cost_reduction_for_each_allied_subtype() {
    assert_round_trip(
        "This card costs {e} less for each allied {subtype}.",
        "e: 1, subtype: warrior",
    );
}

#[test]
fn test_round_trip_allies_have_spark_bonus() {
    assert_round_trip("Allies have +{s} spark.", "s: 1");
}

#[test]
fn test_round_trip_allied_subtype_have_spark_bonus() {
    assert_round_trip("Allied {plural-subtype} have +{s} spark.", "subtype: warrior, s: 1");
}

#[test]
fn test_round_trip_play_from_void_for_cost() {
    assert_round_trip("Play this card from your void for {e}.", "e: 0");
}

#[test]
fn test_round_trip_play_from_void_with_additional_cost() {
    assert_round_trip("Return an ally to hand: Play this card from your void for {e}.", "e: 0");
}

#[test]
fn test_round_trip_play_from_void_then_banish_when_leaves_play() {
    assert_round_trip(
        "Play this card from your void for {e}, then {banish} it when it leaves play.",
        "e: 0",
    );
}

#[test]
fn test_round_trip_play_from_void_with_cost_then_banish() {
    assert_round_trip(
        "Return an ally to hand: Play this card from your void for {e}, then {banish} it when it leaves play.",
        "e: 0",
    );
}

#[test]
fn test_round_trip_play_from_void_then_gain_spark() {
    assert_round_trip("Play this card from your void for {e}, then gain +{s} spark.", "e: 0, s: 1");
}

#[test]
fn test_round_trip_play_from_void_then_dissolve_enemy() {
    assert_round_trip("Play this card from your void for {e}, then {Dissolve} an enemy.", "e: 0");
}
