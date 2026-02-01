//! Round-trip tests for event effects.
//!
//! Tests simple one-shot effects like dissolve, banish, prevent,
//! draw, discard, discover, reclaim, gain energy/points, kindle, etc.

use parser_v2_tests::test_helpers::*;

// ============================================================================
// Discover effects
// ============================================================================

#[test]
fn test_discover_card_by_cost() {
    assert_round_trip("{Discover} a card with cost {e}.", "e: 2");
}

#[test]
fn test_discover_subtype() {
    assert_round_trip("{Discover} {a-subtype}.", "subtype: warrior");
}

#[test]
fn test_discover_subtype_survivor() {
    assert_round_trip("{Discover} {a-subtype}.", "subtype: survivor");
}

#[test]
fn test_discover_event() {
    assert_round_trip("{Discover} an event.", "");
}

#[test]
fn test_discover_character_with_activated_ability() {
    assert_round_trip("{Discover} a character with an activated ability.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_discover_character_with_materialized_ability() {
    assert_round_trip("{Discover} a character with a {Materialized} ability.", "");
}

// ============================================================================
// Dissolve effects
// ============================================================================

#[test]
fn test_dissolve_enemy() {
    assert_round_trip("{Dissolve} an enemy.", "");
}

#[test]
fn test_dissolve_all_characters() {
    assert_round_trip("{Dissolve} all characters.", "");
}

#[test]
fn test_dissolve_enemy_lose_points() {
    assert_round_trip("{Dissolve} an enemy. You lose {points}.", "points: 4");
}

#[test]
fn test_dissolve_enemy_opponent_gains_points() {
    assert_round_trip("{Dissolve} an enemy. The opponent gains {points}.", "points: 3");
}

#[test]
fn test_dissolve_enemy_by_spark_or_less() {
    assert_round_trip("{Dissolve} an enemy with spark {s} or less.", "s: 1");
}

#[test]
fn test_dissolve_enemy_by_spark_or_more() {
    assert_round_trip("{Dissolve} an enemy with spark {s} or more.", "s: 3");
}

#[test]
fn test_dissolve_enemy_by_cost_or_less() {
    assert_round_trip("{Dissolve} an enemy with cost {e} or less.", "e: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_dissolve_enemy_by_cost_or_more_with_reclaim() {
    assert_round_trip("{Dissolve} an enemy with cost {e} or more.", "e: 3\nreclaim: 2");
    assert_round_trip("{ReclaimForCost}", "e: 3\nreclaim: 2");
}

#[test]
fn test_dissolve_enemy_by_subtype_count() {
    assert_round_trip(
        "{Dissolve} an enemy with cost less than the number of allied {plural-subtype}.",
        "subtype: warrior",
    );
}

#[test]
fn test_dissolve_enemy_by_void_count() {
    assert_round_trip(
        "{Dissolve} an enemy with cost less than the number of cards in your void.",
        "",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_dissolve_enemy_draw() {
    assert_round_trip("{Dissolve} an enemy. Draw {cards}.", "cards: 1\ne: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_dissolve_by_energy_paid() {
    assert_round_trip("Pay 1 or more {energy-symbol}: {Dissolve} each character with spark less than the amount of {energy-symbol} paid.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_dissolve_enemy_with_reclaim_cost() {
    assert_round_trip("{Dissolve} an enemy with cost {e} or less.", "e: 2");
    assert_round_trip("{Reclaim} -- Abandon an ally", "e: 2");
}

// ============================================================================
// Banish effects
// ============================================================================

#[test]
fn test_banish_enemy_by_cost() {
    assert_round_trip("{Banish} an enemy with cost {e} or less.", "e: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_banish_non_subtype_enemy() {
    assert_round_trip("{Banish} a non-{subtype} enemy.", "subtype: warrior");
}

#[test]
fn test_banish_allies_then_materialize() {
    assert_round_trip("{Banish} {up-to-n-allies}, then {materialize} {it-or-them}.", "number: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_banish_ally_materialize_at_end_of_turn() {
    assert_round_trip("{Banish} an ally. {Materialize} it at end of turn.", "reclaim: 1");
    assert_round_trip("{ReclaimForCost}", "reclaim: 1");
}

// ============================================================================
// Prevent effects
// ============================================================================

#[test]
fn test_prevent_played_card() {
    assert_round_trip("{Prevent} a played card.", "");
}

#[test]
fn test_prevent_played_event_unless_pay() {
    assert_round_trip("{Prevent} a played event unless the opponent pays {e}.", "e: 2");
}

#[test]
fn test_prevent_played_fast_card() {
    assert_round_trip("{Prevent} a played {fast} card.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_prevent_dissolve_event() {
    assert_round_trip("{Prevent} a played event which could {dissolve} an ally.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_prevent_played_character() {
    assert_round_trip("{Prevent} a played character.", "");
}

#[test]
fn test_prevent_played_card_put_on_deck() {
    assert_round_trip("{Prevent} a played card. Put it on top of the opponent's deck.", "");
}

// ============================================================================
// Draw and discard effects
// ============================================================================

#[test]
fn test_draw_cards() {
    assert_round_trip("Draw {cards}.", "cards: 3");
}

#[test]
fn test_draw_one() {
    assert_round_trip("Draw {cards}.", "cards: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_draw_discard_with_reclaim() {
    assert_round_trip("Draw {cards}. Discard {discards}.", "cards: 2\ndiscards: 2\nreclaim :2");
    assert_round_trip("{ReclaimForCost}", "cards: 2\ndiscards: 2\nreclaim :2");
}

#[test]
fn test_draw_discard() {
    assert_round_trip("Draw {cards}. Discard {discards}.", "cards: 3\ndiscards: 2");
}

#[test]
fn test_discard_draw() {
    assert_round_trip("Discard {discards}. Draw {cards}.", "discards: 1\ncards: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_discard_chosen_from_opponent_hand_by_cost() {
    assert_round_trip(
        "Discard a chosen card with cost {e} or less from the opponent's hand.",
        "e: 3",
    );
}

#[test]
fn test_discard_chosen_character_from_opponent_hand() {
    assert_round_trip("Discard a chosen character from the opponent's hand.", "");
}

#[test]
fn test_return_character_to_hand_draw() {
    assert_round_trip("Return an enemy or ally to hand. Draw {cards}.", "cards: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_may_return_character_from_void_draw() {
    assert_round_trip(
        "You may return a character from your void to your hand. Draw {cards}.",
        "cards: 1",
    );
}

#[test]
fn test_draw_per_cards_played() {
    assert_round_trip("Draw {cards} for each card you have played this turn.", "cards: 1");
}

#[test]
fn test_return_events_from_void() {
    assert_round_trip("Return {up-to-n-events} from your void to your hand.", "number: 2");
}

#[test]
fn test_put_top_cards_to_void_draw() {
    assert_round_trip(
        "Put the {top-n-cards} of your deck into your void. Draw {cards}.",
        "to-void: 3\ncards: 1",
    );
}

// ============================================================================
// Energy and points effects
// ============================================================================

#[test]
fn test_gain_energy() {
    assert_round_trip("Gain {e}.", "e: 3");
}

#[test]
fn test_gain_four_energy() {
    assert_round_trip("Gain {e}.", "e: 4");
}

#[test]
fn test_gain_six_energy() {
    assert_round_trip("Gain {e}.", "e: 6");
}

#[test]
fn test_gain_points_per_cards_played() {
    assert_round_trip("Gain {points} for each card you have played this turn.", "points: 1");
}

#[test]
fn test_multiply_energy() {
    assert_round_trip("{MultiplyBy} the amount of {energy-symbol} you have.", "number: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_multiply_energy_gain() {
    assert_round_trip(
        "{MultiplyBy} the amount of {energy-symbol} you gain from card effects this turn.",
        "number: 2",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_multiply_draw() {
    assert_round_trip(
        "{MultiplyBy} the number of cards you draw from card effects this turn.",
        "number: 2",
    );
}

#[test]
fn test_gain_energy_draw() {
    assert_round_trip("Gain {e}. Draw {cards}.", "e: 2\ncards: 1");
}

#[test]
fn test_draw_discard_gain_energy() {
    assert_round_trip("Draw {cards}. Discard {discards}. Gain {e}.", "cards: 2\ndiscards: 2\ne: 2");
}

// ============================================================================
// Reclaim and foresee effects
// ============================================================================

#[test]
fn test_all_void_gains_reclaim_equal_cost() {
    assert_round_trip(
        "All cards currently in your void gain {reclaim} equal to their cost this turn.",
        "",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_event_gains_reclaim_for_cost() {
    assert_round_trip("An event in your void gains {reclaim-for-cost} this turn.", "reclaim: 0");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_foresee_draw_with_reclaim() {
    assert_round_trip("{Foresee}. Draw {cards}.", "foresee: 1\ncards: 1\nreclaim: 3");
    assert_round_trip("{ReclaimForCost}", "foresee: 1\ncards: 1\nreclaim: 3");
}

// ============================================================================
// Turn and phase effects
// ============================================================================

#[test]
fn test_extra_judgment_phase() {
    assert_round_trip(
        "At the end of this turn, trigger an additional {JudgmentPhaseName} phase.",
        "",
    );
}

#[test]
fn test_extra_turn() {
    assert_round_trip("Take an extra turn after this one.", "");
}

// ============================================================================
// Copy effects
// ============================================================================

#[ignore = "Round-trip mismatch"]
#[test]
fn test_copy_next_event_times() {
    assert_round_trip("Copy the next event you play {this-turn-times}.", "number: 3\nreclaim: 2");
    assert_round_trip("{ReclaimForCost}", "number: 3\nreclaim: 2");
}

// ============================================================================
// Shuffle effects
// ============================================================================

#[test]
fn test_shuffle_and_draw() {
    assert_round_trip(
        "Each player shuffles their hand and void into their deck and then draws {cards}.",
        "cards: 5",
    );
}

// ============================================================================
// Spark modification effects
// ============================================================================

#[test]
fn test_ally_gains_spark_per_subtype() {
    assert_round_trip(
        "An ally gains +{s} spark for each allied {subtype}.",
        "s: 1\nsubtype: warrior",
    );
}

// ============================================================================
// Materialize effects
// ============================================================================

#[test]
fn test_materialize_figments_per_cards_played() {
    assert_round_trip(
        "{Materialize} {a-figment} for each card you have played this turn.",
        "figment: celestial",
    );
}

#[test]
fn test_materialize_multiple_figments() {
    assert_round_trip("{Materialize} {n-figments}.", "number: 3\nfigment: radiant");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_materialize_random_characters_from_deck() {
    assert_round_trip(
        "{Materialize} {n-random-characters} with cost {e} or less from your deck.",
        "e: 3\nnumber: 2",
    );
}

// ============================================================================
// Choose one effects
// ============================================================================

#[ignore = "Round-trip mismatch"]
#[test]
fn test_choose_one_return_or_draw() {
    assert_round_trip("{ChooseOne}\n{bullet} {mode1-cost}: Return an enemy to hand.\n{bullet} {mode2-cost}: Draw {cards}.", "mode1-cost: 2\nmode2-cost: 3\ncards: 2");
}
