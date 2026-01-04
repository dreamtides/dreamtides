use parser_v2::serializer::ability_serializer;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_gain_energy_draw_cards() {
    let original = "Gain {e}. Draw {cards}.";
    let parsed = parse_ability(original, "e: 2, cards: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_gain_energy_draw_cards() {
    let original = "{Judgment} Gain {e}. Draw {cards}.";
    let parsed = parse_ability(original, "e: 1, cards: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_draw_cards_discard_cards() {
    let original = "Draw {cards}. Discard {discards}.";
    let parsed = parse_ability(original, "cards: 2, discards: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_draw_cards_discard_cards_gain_energy() {
    let original = "Draw {cards}. Discard {discards}. Gain {e}.";
    let parsed = parse_ability(original, "cards: 1, discards: 1, e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_put_cards_from_deck_into_void_draw_cards() {
    let original = "Put the {top-n-cards} of your deck into your void. Draw {cards}.";
    let parsed = parse_ability(original, "to-void: 3, cards: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discard_cards_draw_cards() {
    let original = "Discard {discards}. Draw {cards}.";
    let parsed = parse_ability(original, "discards: 1, cards: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_enemy_you_lose_points() {
    let original = "{Dissolve} an enemy. You lose {points}.";
    let parsed = parse_ability(original, "points: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_enemy_opponent_gains_points() {
    let original = "{Dissolve} an enemy. The opponent gains {points}.";
    let parsed = parse_ability(original, "points: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_draw_cards_opponent_gains_points() {
    let original = "{Judgment} Draw {cards}. The opponent gains {points}.";
    let parsed = parse_ability(original, "cards: 2, points: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_return_enemy_or_ally_to_hand_draw_cards() {
    let original = "Return an enemy or ally to hand. Draw {cards}.";
    let parsed = parse_ability(original, "cards: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_draw_then_discard() {
    // Note: "then" separated effects are parsed as Effect::List and serialize
    // with periods instead of ", then"
    let original = "{Judgment} Draw {cards}, then discard {discards}.";
    let parsed = parse_ability(original, "cards: 2, discards: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!("{Judgment} Draw {cards}. Discard {discards}.", serialized);
}

#[test]
fn test_round_trip_materialized_discard_then_draw() {
    // Note: "then" separated effects are parsed as Effect::List and serialize
    // with periods instead of ", then"
    let original = "{Materialized} Discard {discards}, then draw {cards}.";
    let parsed = parse_ability(original, "discards: 1, cards: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!("{Materialized} Discard {discards}. Draw {cards}.", serialized);
}

#[test]
fn test_round_trip_materialized_draw_discard() {
    let original = "{Materialized} Draw {cards}. Discard {discards}.";
    let parsed = parse_ability(original, "cards: 2, discards: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_you_may_return_character_from_void_draw_cards() {
    assert_eq!(
        "You may return a character from your void to your hand, then draw {cards}.",
        ability_serializer::serialize_ability(&parse_ability(
            "You may return a character from your void to your hand. Draw {cards}.",
            "cards: 2",
        )),
    );
}

#[test]
fn test_round_trip_events_cost_less() {
    let original = "Events cost you {e} less.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_characters_cost_less() {
    let original = "Characters cost you {e} less.";
    let parsed = parse_ability(original, "e: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_opponent_events_cost_more() {
    let original = "The opponent's events cost {e} more.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_allied_plural_subtype_have_spark() {
    let original = "Allied {plural-subtype} have +{s} spark.";
    let parsed = parse_ability(original, "subtype: warrior, s: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_banish_from_hand_play_for_alternate_cost() {
    let original = "{Banish} a card from hand: Play this event for {e}.";
    let parsed = parse_ability(original, "e: 0");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}
