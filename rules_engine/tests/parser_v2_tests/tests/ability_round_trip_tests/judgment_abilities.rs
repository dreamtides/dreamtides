use parser_v2::serializer::ability_serializer;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_round_trip_materialized_you_may_banish_ally_then_materialize_it() {
    let original = "{Materialized} You may {Banish} an ally, then {Materialize} it.";
    let parsed =
        parse_ability("{Materialized} You may {banish} an ally, then {materialize} it.", "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_banish_any_number_of_allies_then_materialize_them() {
    let original = "{Materialized} {Banish} any number of allies, then {materialize} them.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!("{Materialized} {Banish} any number of allies. {Materialize} them.", serialized);
}

#[test]
fn test_round_trip_judgment_you_may_banish_ally_then_materialize_it() {
    let original = "{Judgment} You may {Banish} an ally, then {Materialize} it.";
    let parsed = parse_ability("{Judgment} You may {banish} an ally, then {materialize} it.", "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_foresee() {
    let original = "{Judgment} {Foresee}.";
    let parsed = parse_ability(original, "foresee: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_kindle() {
    let original = "{Judgment} {Kindle}.";
    let parsed = parse_ability(original, "k: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_foresee() {
    let original = "{Materialized} {Foresee}.";
    let parsed = parse_ability(original, "foresee: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_judgment_kindle() {
    let original = "{MaterializedJudgment} {Kindle}.";
    let parsed = parse_ability(original, "k: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_gain_control_enemy_with_cost_or_less() {
    let original = "{Materialized} Gain control of an enemy with cost {e} or less.";
    let parsed = parse_ability(original, "e: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_dissolved_draw_cards() {
    let original = "{MaterializedDissolved} Draw {cards}.";
    let parsed = parse_ability(original, "cards: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_dissolved_put_cards_from_deck_into_void() {
    let original = "{MaterializedDissolved} Put the {top-n-cards} of your deck into your void.";
    let parsed = parse_ability(original, "to-void: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_judgment_gain_energy() {
    let original = "{MaterializedJudgment} Gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_return_ally_to_hand() {
    let original = "{Materialized} Return an ally to hand.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_pay_energy_to_kindle_and_banish_cards_from_opponent_void() {
    let original = "{Judgment} Pay {e} to {Kindle} and {Banish} {cards} from the opponent's void.";
    let parsed = parse_ability(original, "e: 1, k: 1, cards: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_you_may_return_ally_to_hand() {
    let original = "{Materialized} You may return an ally to hand.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_return_character_from_void_to_hand() {
    let original = "{Materialized} Return a character from your void to your hand.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_discover_fast_event() {
    let original = "{Materialized} {Discover} a {fast} event.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_return_this_from_void_to_hand() {
    let original = "{Judgment} Return this character from your void to your hand.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_banish_opponent_void() {
    let original = "{Materialized} {Banish} the opponent's void.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_you_may_pay_to_return_this_from_void_to_hand() {
    let original =
        "{Judgment} You may pay {e} to return this character from your void to your hand.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_you_may_pay_to_have_each_allied_gain_spark() {
    let original = "{Judgment} You may pay {e} to have each allied {subtype} gain +{s} spark.";
    let parsed = parse_ability(original, "e: 1, subtype: warrior, s: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_dissolved_you_may_pay_to_return_this_to_hand() {
    let original = "{Dissolved} You may pay {e} to return this character to your hand.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_prevent_played_card_with_cost() {
    let original = "{Materialized} {Prevent} a played card with cost {e} or less.";
    let parsed = parse_ability(original, "e: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_you_may_draw_then_discard() {
    let original = "{Judgment} You may draw {cards}, then discard {discards}.";
    let parsed = parse_ability(original, "cards: 2, discards: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_you_may_discard_draw_gain_points() {
    let original = "{Judgment} You may discard {discards} to draw {cards} and gain {points}.";
    let parsed = parse_ability(original, "discards: 2, cards: 1, points: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
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
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_gain_energy_for_each_allied_subtype() {
    let original = "{Judgment} Gain {e} for each allied {subtype}.";
    let parsed = parse_ability(original, "subtype: warrior, e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_gain_energy_for_each_allied_character() {
    let original = "{Judgment} Gain {e} for each allied character.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_with_count_allied_subtype_gain_energy() {
    let original = "{Judgment} With {count-allied-subtype}, gain {e}.";
    let parsed = parse_ability(original, "subtype: warrior, allies: 2, e: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_judgment_with_count_allied_subtype_gain_energy() {
    let original = "{MaterializedJudgment} With {count-allied-subtype}, gain {e}.";
    let parsed = parse_ability(original, "subtype: warrior, allies: 2, e: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_judgment_with_count_allied_subtype_draw_cards() {
    let original = "{MaterializedJudgment} With {count-allied-subtype}, draw {cards}.";
    let parsed = parse_ability(original, "subtype: warrior, allies: 2, cards: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_each_player_discards() {
    let original = "{Materialized} Each player discards {discards}.";
    let parsed = parse_ability(original, "discards: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_each_player_abandons_character() {
    let original = "{Judgment} Each player abandons a character.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_discard_chosen_card_from_opponent_hand_they_draw() {
    let original =
        "{Materialized} Discard a chosen card from the opponent's hand. They draw {cards}.";
    let parsed = parse_ability(original, "cards: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_draw_cards_for_each_ally_abandoned_this_turn() {
    let original = "{Materialized} Draw {cards} for each ally abandoned this turn.";
    let parsed = parse_ability(original, "cards: 2");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_card_with_cost_in_void_gains_reclaim() {
    let original = "{Materialized} A card with cost {e} or less in your void gains {reclaim-for-cost} this turn.";
    let parsed = parse_ability(original, "e: 3, reclaim: 0");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_banish_cards_from_your_void_to_dissolve_enemy_with_cost() {
    let original = "{Judgment} You may {Banish} {cards} from your void to {Dissolve} an enemy with cost {e} or less.";
    let parsed = parse_ability(
        "{Judgment} You may {banish} {cards} from your void to {dissolve} an enemy with cost {e} or less.",
        "cards: 3, e: 2",
    );
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_banish_cards_from_opponent_void_to_gain_energy() {
    let original = "{Judgment} You may {Banish} {cards} from the opponent's void to gain {e}.";
    let parsed = parse_ability(
        "{Judgment} You may {banish} {cards} from the opponent's void to gain {e}.",
        "cards: 1, e: 1",
    );
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_abandon_to_discover_and_materialize() {
    let original = "{Judgment} You may abandon {subtype} to {Discover} {subtype} with cost {e} or more and {materialize} it.";
    let parsed = parse_ability(
        "{Judgment} You may abandon {a-subtype} to {discover} {a-subtype} with cost {e} higher and {materialize} it.",
        "subtype: warrior, e: 2",
    );
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_pay_to_banish_allies_then_materialize() {
    let original = "{Judgment} You may pay {e} to {Banish} {up-to-n-allies}, then {Materialize} them.";
    let parsed = parse_ability(
        "{Judgment} You may pay {e} to {banish} {up-to-n-allies}, then {materialize} {it-or-them}.",
        "e: 1, number: 2",
    );
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_banish_enemy_until_character_leaves_play() {
    let original = "{Materialized} {Banish} an enemy until this character leaves play.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_banish_enemy_until_next_main_phase() {
    let original = "{Materialized} {Banish} an enemy until your next main phase.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_event_in_void_gains_reclaim() {
    let original = "{Materialized} An event in your void gains {reclaim} equal to its cost.";
    let parsed = parse_ability("{Materialized} An event in your void gains {reclaim} equal to its cost this turn.", "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_copy_next_event() {
    let original = "{Materialized} Copy the next event you play {this-turn-times}.";
    let parsed = parse_ability(original, "number: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_disable_activated_abilities() {
    let original =
        "{Materialized} Disable the activated abilities of an enemy while this character is in play.";
    let parsed = parse_ability(original, "");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_materialized_draw_subtype() {
    let original = "{Materialized} Draw {a-subtype} from your deck.";
    let parsed = parse_ability(original, "subtype: warrior");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_each_allied_subtype_gains_spark_for_each_allied_subtype() {
    let original = "{Judgment} Each allied {subtype} gains {s} equal to the number of allied {plural-subtype}.";
    let parsed = parse_ability(original, "subtype: warrior, s: 1");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_judgment_each_player_shuffles_hand_and_void_and_draws() {
    let original = "{Judgment} Each player shuffles their hand and void into their deck and then draws {cards}.";
    let parsed = parse_ability(original, "cards: 3");
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}
