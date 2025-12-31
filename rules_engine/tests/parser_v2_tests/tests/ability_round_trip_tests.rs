use parser_v2::serializer::parser_formatter;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_abandon_an_ally_gain_energy() {
    let original = "Abandon an ally: Gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_an_ally_kindle() {
    let original = "Abandon an ally: {Kindle}.";
    let parsed = parse_ability(original, "k: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_at_end_of_turn_gain_energy() {
    let original = "At the end of your turn, gain {e}.";
    let parsed = parse_ability(original, "e: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_event_foresee() {
    let original = "{Foresee}.";
    let parsed = parse_ability(original, "foresee: 3");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_event_kindle() {
    let original = "{Kindle}.";
    let parsed = parse_ability(original, "k: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_event_discover() {
    let original = "{Discover} {a-subtype}.";
    let parsed = parse_ability(original, "subtype: warrior");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_event_prevent() {
    let original = "{Prevent} a card.";
    let parsed = parse_ability(original, "");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_event_dissolve() {
    let original = "{Dissolve} an enemy.";
    let parsed = parse_ability(original, "");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_foresee() {
    let original = "{Judgment} {Foresee}.";
    let parsed = parse_ability(original, "foresee: 3");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_kindle() {
    let original = "{Judgment} {Kindle}.";
    let parsed = parse_ability(original, "k: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_foresee() {
    let original = "{Materialized} {Foresee}.";
    let parsed = parse_ability(original, "foresee: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_judgment_kindle() {
    let original = "{MaterializedJudgment} {Kindle}.";
    let parsed = parse_ability(original, "k: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_gain_energy_draw_cards() {
    let original = "Gain {e}. Draw {cards}.";
    let parsed = parse_ability(original, "e: 2, cards: 3");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_gain_energy_draw_cards() {
    let original = "{Judgment} Gain {e}. Draw {cards}.";
    let parsed = parse_ability(original, "e: 1, cards: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_draw_cards_discard_cards() {
    let original = "Draw {cards}. Discard {discards}.";
    let parsed = parse_ability(original, "cards: 2, discards: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_draw_cards_discard_cards_gain_energy() {
    let original = "Draw {cards}. Discard {discards}. Gain {e}.";
    let parsed = parse_ability(original, "cards: 1, discards: 1, e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discard_cards_draw_cards() {
    let original = "Discard {discards}. Draw {cards}.";
    let parsed = parse_ability(original, "discards: 1, cards: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_enemy_you_lose_points() {
    let original = "{Dissolve} an enemy. You lose {points}.";
    let parsed = parse_ability(original, "points: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_enemy_opponent_gains_points() {
    let original = "{Dissolve} an enemy. The opponent gains {points}.";
    let parsed = parse_ability(original, "points: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_draw_cards_opponent_gains_points() {
    let original = "{Judgment} Draw {cards}. The opponent gains {points}.";
    let parsed = parse_ability(original, "cards: 2, points: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_return_enemy_or_ally_to_hand_draw_cards() {
    let original = "Return an enemy or ally to hand. Draw {cards}.";
    let parsed = parse_ability(original, "cards: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_enemy_with_spark_or_less() {
    let original = "{Dissolve} an enemy with spark {s} or less.";
    let parsed = parse_ability(original, "s: 3");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_enemy_with_spark_or_more() {
    let original = "{Dissolve} an enemy with spark {s} or more.";
    let parsed = parse_ability(original, "s: 5");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_banish_enemy_with_cost_or_less() {
    let original = "{Banish} an enemy with cost {e} or less.";
    let parsed = parse_ability(original, "e: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolve_enemy_with_cost_or_more() {
    let original = "{Dissolve} an enemy with cost {e} or more.";
    let parsed = parse_ability(original, "e: 4");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_draw_then_discard() {
    // Note: "then" separated effects are parsed as Effect::List and serialize
    // with periods instead of ", then"
    let original = "{Judgment} Draw {cards}, then discard {discards}.";
    let parsed = parse_ability(original, "cards: 2, discards: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!("{Judgment} Draw {cards}. Discard {discards}.", serialized);
}

#[test]
fn test_round_trip_materialized_discard_then_draw() {
    // Note: "then" separated effects are parsed as Effect::List and serialize
    // with periods instead of ", then"
    let original = "{Materialized} Discard {discards}, then draw {cards}.";
    let parsed = parse_ability(original, "discards: 1, cards: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!("{Materialized} Discard {discards}. Draw {cards}.", serialized);
}

#[test]
fn test_round_trip_materialized_dissolved_draw_cards() {
    let original = "{MaterializedDissolved} Draw {cards}.";
    let parsed = parse_ability(original, "cards: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_judgment_gain_energy() {
    let original = "{MaterializedJudgment} Gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_return_ally_to_hand() {
    let original = "{Materialized} Return an ally to hand.";
    let parsed = parse_ability(original, "");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_you_may_return_ally_to_hand() {
    let original = "{Materialized} You may return an ally to hand.";
    let parsed = parse_ability(original, "");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_discover_fast_event() {
    let original = "{Materialized} {Discover} a {fast} event.";
    let parsed = parse_ability(original, "");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_return_this_from_void_to_hand() {
    let original = "{Judgment} Return this character from your void to your hand.";
    let parsed = parse_ability(original, "");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discover_fast_character() {
    let original = "{Discover} a {fast} character.";
    let parsed = parse_ability(original, "");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discover_fast_card() {
    let original = "{Discover} a {fast} card.";
    let parsed = parse_ability(original, "");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discover_fast_subtype() {
    let original = "{Discover} a {fast} {subtype}.";
    let parsed = parse_ability(original, "subtype: warrior");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discover_fast_character_with_spark() {
    let original = "{Discover} a {fast} character with spark {s} or less.";
    let parsed = parse_ability(original, "s: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discover_fast_card_with_cost() {
    let original = "{Discover} a {fast} character with cost {e} or less.";
    let parsed = parse_ability(original, "e: 3");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_you_may_draw_then_discard() {
    let original = "{Judgment} You may draw {cards}, then discard {discards}.";
    let parsed = parse_ability(original, "cards: 2, discards: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_discard_a_card_gain_points() {
    let original = "When you discard a card, gain {points}.";
    let parsed = parse_ability(original, "points: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_discard_a_card_kindle() {
    let original = "When you discard a card, {Kindle}.";
    let parsed = parse_ability(original, "k: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_play_an_event_gain_energy() {
    let original = "When you play an event, gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_play_an_event_foresee() {
    let original = "When you play an event, {Foresee}.";
    let parsed = parse_ability(original, "foresee: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}
