use parser_v2::serializer::ability_serializer;
use parser_v2_tests::test_helpers::*;

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
