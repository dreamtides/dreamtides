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
fn test_round_trip_abandon_an_ally_once_per_turn_gain_points() {
    let original = "Abandon an ally, once per turn: Gain {points}.";
    let parsed = parse_ability(original, "points: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_an_ally_once_per_turn_reclaim_subtype() {
    let original = "Abandon an ally, once per turn: {Reclaim} a {subtype}.";
    let parsed = parse_ability(original, "subtype: warrior");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!("Abandon an ally, once per turn: {Reclaim} {a-subtype}.", serialized);
}

#[test]
fn test_round_trip_abandon_an_ally_kindle() {
    let original = "Abandon an ally: {Kindle}.";
    let parsed = parse_ability(original, "k: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_an_ally_put_cards_from_deck_into_void() {
    let original = "Abandon an ally: Put the {top-n-cards} of your deck into your void.";
    let parsed = parse_ability(original, "to-void: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_or_discard_dissolve_enemy() {
    let original = "Abandon an ally or discard a card: {Dissolve} an enemy.";
    let parsed = parse_ability(original, "");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_energy_discard_kindle() {
    let original = "{e}, Discard {discards}: {kindle}.";
    let parsed = parse_ability(original, "e: 1, discards: 2, k: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!("{e}, Discard {discards}: {Kindle}.", serialized);
}

#[test]
fn test_round_trip_energy_banish_reclaim_this_character() {
    let original = "{e}, {Banish} another card in your void: {Reclaim} this character.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_energy_abandon_ally_with_spark_draw_cards() {
    let original = "{e}, Abandon an ally with spark {s} or less: Draw {cards}.";
    let parsed = parse_ability(original, "e: 1, s: 2, cards: 3");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_energy_abandon_character_discard_hand_draw_cards() {
    let original = "{e}, Abandon a character, Discard your hand: Draw {cards}.";
    let parsed = parse_ability(original, "e: 2, cards: 3");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_character_discard_hand_gain_energy() {
    let original = "Abandon a character, Discard your hand: Gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_energy_materialize_copy_of_ally() {
    let original = "{e}: {Materialize} a copy of an ally.";
    let parsed = parse_ability(original, "e: 1");
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
fn test_round_trip_events_cost_less() {
    let original = "Events cost you {e} less.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_characters_cost_less() {
    let original = "Characters cost you {e} less.";
    let parsed = parse_ability(original, "e: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_opponent_events_cost_more() {
    let original = "The opponent's events cost {e} more.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_materialize_an_ally_gain_energy() {
    let original = "When you {materialize} an ally, gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_materialize_a_subtype_reclaim_this_character() {
    let original = "When you {materialize} {a-subtype}, {Reclaim} this character.";
    let parsed = parse_ability(original, "subtype: warrior");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_materialize_a_character_this_character_gains_spark() {
    let original = "When you {materialize} a character, this character gains +{s} spark.";
    let parsed = parse_ability(original, "s: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_once_per_turn_when_you_materialize_a_character_gain_energy() {
    let original = "Once per turn, when you {materialize} a character, gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_once_per_turn_when_you_discard_a_card_gain_energy_and_kindle() {
    let original = "Once per turn, when you discard a card, gain {e} and {kindle}.";
    let parsed = parse_ability(original, "e: 1, k: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!("Once per turn, when you discard a card, Gain {e}. {Kindle}.", serialized);
}

#[test]
fn test_round_trip_when_you_abandon_an_ally_kindle() {
    let original = "When you abandon an ally, {Kindle}.";
    let parsed = parse_ability(original, "k: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_an_ally_is_dissolved_gain_points() {
    let original = "When an ally is {dissolved}, gain {points}.";
    let parsed = parse_ability(original, "points: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_an_ally_is_dissolved_draw_cards() {
    let original = "When an ally is {dissolved}, draw {cards}.";
    let parsed = parse_ability(original, "cards: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_an_ally_is_dissolved_gain_energy() {
    let original = "When an ally is {dissolved}, gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_an_ally_is_banished_kindle() {
    let original = "When an ally is {banished}, {Kindle}.";
    let parsed = parse_ability(original, "k: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_an_ally_is_banished_this_character_gains_spark() {
    let original = "When an ally is {banished}, this character gains +{s} spark.";
    let parsed = parse_ability(original, "s: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_you_may_banish_ally_then_materialize_it() {
    let original = "{Materialized} You may {Banish} an ally, then {Materialize} it.";
    let parsed =
        parse_ability("{Materialized} You may {banish} an ally, then {materialize} it.", "");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_you_may_banish_ally_then_materialize_it() {
    let original = "{Judgment} You may {Banish} an ally, then {Materialize} it.";
    let parsed = parse_ability("{Judgment} You may {banish} an ally, then {materialize} it.", "");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_play_a_subtype_draw_cards() {
    let original = "When you play {a-subtype}, draw {cards}.";
    let parsed = parse_ability(original, "subtype: warrior, cards: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_play_a_subtype_put_cards_from_deck_into_void() {
    let original = "When you play {a-subtype}, put the {top-n-cards} of your deck into your void.";
    let parsed = parse_ability(original, "subtype: warrior, to-void: 3");
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
    let original = "{Prevent} a played card.";
    let parsed = parse_ability(original, "");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_event_put_on_top_of_opponent_deck() {
    let original = "Put it on top of the opponent's deck.";
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
fn test_round_trip_materialized_gain_control_enemy_with_cost_or_less() {
    let original = "{Materialized} Gain control of an enemy with cost {e} or less.";
    let parsed = parse_ability(original, "e: 2");
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
fn test_round_trip_put_cards_from_deck_into_void_draw_cards() {
    let original = "Put the {top-n-cards} of your deck into your void. Draw {cards}.";
    let parsed = parse_ability(original, "to-void: 3, cards: 2");
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
fn test_round_trip_materialized_draw_discard() {
    let original = "{Materialized} Draw {cards}. Discard {discards}.";
    let parsed = parse_ability(original, "cards: 2, discards: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_dissolved_draw_cards() {
    let original = "{MaterializedDissolved} Draw {cards}.";
    let parsed = parse_ability(original, "cards: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_dissolved_put_cards_from_deck_into_void() {
    let original = "{MaterializedDissolved} Put the {top-n-cards} of your deck into your void.";
    let parsed = parse_ability(original, "to-void: 2");
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
fn test_round_trip_materialized_return_character_from_void_to_hand() {
    let original = "{Materialized} Return a character from your void to your hand.";
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
fn test_round_trip_materialized_banish_opponent_void() {
    let original = "{Materialized} {Banish} the opponent's void.";
    let parsed = parse_ability(original, "");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_you_may_return_character_from_void_draw_cards() {
    assert_eq!(
        "You may return a character from your void to your hand, then draw {cards}.",
        parser_formatter::serialize_ability(&parse_ability(
            "You may return a character from your void to your hand. Draw {cards}.",
            "cards: 2",
        )),
    );
}

#[test]
fn test_round_trip_judgment_you_may_pay_to_return_this_from_void_to_hand() {
    let original =
        "{Judgment} You may pay {e} to return this character from your void to your hand.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolved_you_may_pay_to_return_this_to_hand() {
    let original = "{Dissolved} You may pay {e} to return this character to your hand.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discard_chosen_character_from_opponent_hand() {
    let original = "Discard a chosen character from the opponent's hand.";
    let parsed = parse_ability(original, "");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_discard_chosen_card_with_cost_from_opponent_hand() {
    let original = "Discard a chosen card with cost {e} or less from the opponent's hand.";
    let parsed = parse_ability(original, "e: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_prevent_event_unless_opponent_pays() {
    let original = "{Prevent} a played event unless the opponent pays {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_prevent_played_card_with_cost() {
    let original = "{Materialized} {Prevent} a played card with cost {e} or less.";
    let parsed = parse_ability(original, "e: 3");
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
fn test_round_trip_judgment_you_may_discard_draw_gain_points() {
    let original = "{Judgment} You may discard {discards} to draw {cards} and gain {points}.";
    let parsed = parse_ability(original, "discards: 2, cards: 1, points: 3");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_you_may_discard_dissolve_enemy() {
    let original =
        "{Judgment} You may discard a card to {Dissolve} an enemy with spark {s} or less.";
    let parsed = parse_ability(
        "{Judgment} You may discard a card to {dissolve} an enemy with spark {s} or less.",
        "s: 2",
    );
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
fn test_round_trip_when_you_discard_this_character_materialize_it() {
    let original = "When you discard this character, {Materialize} it.";
    let parsed = parse_ability("When you discard this character, {materialize} it.", "");
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

#[test]
fn test_round_trip_allied_plural_subtype_have_spark() {
    let original = "Allied {plural-subtype} have +{s} spark.";
    let parsed = parse_ability(original, "subtype: warrior, s: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_an_ally_this_character_gains_spark() {
    let original = "Abandon an ally: This character gains +{s} spark.";
    let parsed = parse_ability(original, "s: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_abandon_an_ally_this_character_gains_spark() {
    let original = "When you abandon an ally, this character gains +{s} spark.";
    let parsed = parse_ability(original, "s: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_materialize_an_allied_subtype_gain_energy() {
    let original = "When you {materialize} an allied {subtype}, gain {e}.";
    let parsed = parse_ability(original, "subtype: warrior, e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_play_a_fast_card_gain_points() {
    let original = "When you play a {fast} card, gain {points}.";
    let parsed = parse_ability(original, "points: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_gain_energy_for_each_allied_subtype() {
    let original = "{Judgment} Gain {e} for each allied {subtype}.";
    let parsed = parse_ability(original, "subtype: warrior, e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_gain_energy_for_each_allied_character() {
    let original = "{Judgment} Gain {e} for each allied character.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_with_count_allied_subtype_gain_energy() {
    let original = "{Judgment} With {count-allied-subtype}, gain {e}.";
    let parsed = parse_ability(original, "subtype: warrior, allies: 2, e: 3");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_judgment_with_count_allied_subtype_gain_energy() {
    let original = "{MaterializedJudgment} With {count-allied-subtype}, gain {e}.";
    let parsed = parse_ability(original, "subtype: warrior, allies: 2, e: 3");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_judgment_with_count_allied_subtype_draw_cards() {
    let original = "{MaterializedJudgment} With {count-allied-subtype}, draw {cards}.";
    let parsed = parse_ability(original, "subtype: warrior, allies: 2, cards: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_abandon_count_allies_reclaim_this_character() {
    let original = "Abandon {count-allies}: {Reclaim} this character.";
    let parsed = parse_ability(original, "allies: 3");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_each_player_discards() {
    let original = "{Materialized} Each player discards {discards}.";
    let parsed = parse_ability(original, "discards: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_each_player_abandons_character() {
    let original = "{Judgment} Each player abandons a character.";
    let parsed = parse_ability(original, "");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}
