use parser_v2::serializer::ability_serializer;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_event_costs_if_character_dissolved() {
    let original = "This event costs {e} if a character dissolved this turn.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_character_costs_if_discarded_card_this_turn() {
    let original = "This character costs {e} if you have discarded a card this turn.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_lose_maximum_energy_play_for_alternate_cost() {
    let original = "Lose {maximum-energy}: Play this event for {e}.";
    let parsed = parse_ability(original, "max: 1, e: 0");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_additional_cost_to_play() {
    let original = "To play this card, return an ally with cost {e} or more to hand.";
    let parsed = parse_ability(original, "e: 4");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_characters_in_hand_have_fast() {
    let original = "Characters in your hand have {fast}.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_disable_enemy_materialized_abilities() {
    let original = "Disable the {Materialized} abilities of enemies.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_once_per_turn_play_from_void() {
    let original = "Once per turn, you may play a character with cost {e} or less from your void.";
    let parsed = parse_ability(original, "e: 0");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_reveal_top_card_of_deck() {
    let original = "Reveal the top card of your deck.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_play_characters_from_top_of_deck() {
    let original = "You may play characters from the top of your deck.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_ability_of_allies_triggers_when_materialize() {
    let original = "The '{Judgment}' ability of allies triggers when you {materialize} them.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_spark_equal_to_allied_subtype() {
    let original = "This character's spark is equal to the number of allied {plural-subtype}.";
    let parsed = parse_ability(original, "subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_spark_equal_to_cards_in_void() {
    let original = "This character's spark is equal to the number of cards in your void.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_while_in_void_allied_subtype_have_spark() {
    let original = "While this card is in your void, allied {plural-subtype} have +{s} spark.";
    let parsed = parse_ability(original, "subtype: warrior, s: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_while_count_or_more_cards_in_void_have_reclaim() {
    let original = "While you have {count} or more cards in your void, they have {reclaim} equal to their cost.";
    let parsed = parse_ability(original, "count: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_play_only_from_void() {
    let original = "You may only play this character from your void.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_with_allied_subtype_play_from_hand_or_void_for_cost() {
    let original = "With an allied {subtype}, you may play this card from your hand or void for {e}.";
    let parsed = parse_ability(original, "subtype: warrior, e: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}
