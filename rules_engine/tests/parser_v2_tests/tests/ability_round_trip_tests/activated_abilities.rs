use parser_v2::serializer::ability_serializer;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_abandon_an_ally_gain_energy() {
    let original = "Abandon an ally: Gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_an_ally_once_per_turn_gain_points() {
    let original = "Abandon an ally, once per turn: Gain {points}.";
    let parsed = parse_ability(original, "points: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_an_ally_once_per_turn_reclaim_subtype() {
    let original = "Abandon an ally, once per turn: {Reclaim} a {subtype}.";
    let parsed = parse_ability(original, "subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!("Abandon an ally, once per turn: {Reclaim} {a-subtype}.", serialized);
}

#[test]
fn test_round_trip_abandon_an_ally_kindle() {
    let original = "Abandon an ally: {Kindle}.";
    let parsed = parse_ability(original, "k: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_an_ally_put_cards_from_deck_into_void() {
    let original = "Abandon an ally: Put the {top-n-cards} of your deck into your void.";
    let parsed = parse_ability(original, "to-void: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_an_ally_put_character_from_void_on_top_of_deck() {
    let original = "Abandon an ally: You may put a character from your void on top of your deck.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_or_discard_dissolve_enemy() {
    let original = "Abandon an ally or discard a card: {Dissolve} an enemy.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_energy_discard_kindle() {
    let original = "{e}, Discard {discards}: {kindle}.";
    let parsed = parse_ability(original, "e: 1, discards: 2, k: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!("{e}, Discard {discards}: {Kindle}.", serialized);
}

#[test]
fn test_round_trip_energy_banish_reclaim_this_character() {
    let original = "{e}, {Banish} another card in your void: {Reclaim} this character.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_energy_abandon_ally_with_spark_draw_cards() {
    let original = "{e}, Abandon an ally with spark {s} or less: Draw {cards}.";
    let parsed = parse_ability(original, "e: 1, s: 2, cards: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_energy_abandon_character_discard_hand_draw_cards() {
    let original = "{e}, Abandon a character, Discard your hand: Draw {cards}.";
    let parsed = parse_ability(original, "e: 2, cards: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_character_discard_hand_gain_energy() {
    let original = "Abandon a character, Discard your hand: Gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_energy_materialize_copy_of_ally() {
    let original = "{e}: {Materialize} a copy of an ally.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_energy_gain_spark_for_each_allied_subtype() {
    let original = "{e}: Gain +{s} spark for each allied {subtype}.";
    let parsed = parse_ability(original, "e: 1, s: 2, subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_an_ally_this_character_gains_spark() {
    let original = "Abandon an ally: This character gains +{s} spark.";
    let parsed = parse_ability(original, "s: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_count_allies_reclaim_this_character() {
    let original = "Abandon {count-allies}: {Reclaim} this character.";
    let parsed = parse_ability(original, "allies: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_ally_gain_energy_equal_to_cost() {
    let original = "Abandon an ally: Gain {energy-symbol} equal to that character's cost.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_ally_dissolve_enemy_with_spark_less_than_abandoned() {
    let original = "Abandon an ally: You may {dissolve} an enemy with spark less than that ally's spark.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_banish_void_with_min_count_reclaim_this_character() {
    let original = "{Banish} your void with {count} or more cards: {Reclaim} this character.";
    let parsed = parse_ability(original, "count: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_energy_spark_of_each_allied_subtype_becomes() {
    let original = "{e}: The spark of each allied {subtype} becomes {s}.";
    let parsed = parse_ability(original, "e: 1, subtype: warrior, s: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_fast_abandon_this_character_prevent_played_event() {
    let original = "{Fast} -- Abandon this character: {Prevent} a played event.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_pay_one_or_more_energy_draw_for_each_energy_spent() {
    let original = "Pay 1 or more {energy-symbol}: Draw {cards} for each {energy-symbol} spent, then discard {discards}.";
    let parsed = parse_ability(original, "cards: 1, discards: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_pay_one_or_more_dissolve_each_character() {
    let original = "Pay 1 or more {energy-symbol}: {Dissolve} each character with spark less than the amount of {energy-symbol} paid.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_spend_one_or_more_energy_draw_for_each_energy_spent() {
    let original = "Spend 1 or more {energy-symbol}: Draw {cards} for each {energy-symbol} spent.";
    let parsed = parse_ability(original, "cards: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!("Pay 1 or more {energy-symbol}: Draw {cards} for each {energy-symbol} spent.", serialized);
}
