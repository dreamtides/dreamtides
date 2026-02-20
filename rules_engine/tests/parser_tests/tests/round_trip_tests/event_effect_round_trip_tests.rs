//! Round-trip tests for event effects.
//!
//! Tests simple one-shot effects like dissolve, banish, prevent,
//! draw, discard, discover, reclaim, gain energy/points, kindle, etc.

use parser_tests::test_helpers::*;

// ============================================================================
// Discover effects
// ============================================================================

#[test]
fn test_discover_card_by_cost() {
    assert_rendered_match("{Discover} a card with cost {energy($e)}.", "e: 2");
}

#[test]
fn test_discover_subtype() {
    assert_rendered_match("{Discover} {@a subtype($t)}.", "t: Warrior");
}

#[test]
fn test_discover_subtype_survivor() {
    assert_rendered_match("{Discover} {@a subtype($t)}.", "t: Survivor");
}

#[test]
fn test_discover_event() {
    assert_rendered_match("{Discover} an event.", "");
}

#[test]
fn test_discover_character_with_activated_ability() {
    assert_rendered_match("{Discover} a character with an activated ability.", "");
}

#[test]
fn test_discover_character_with_materialized_ability() {
    assert_rendered_match("{Discover} a character with a {materialized} ability.", "");
}

// ============================================================================
// Dissolve effects
// ============================================================================

#[test]
fn test_dissolve_enemy() {
    assert_rendered_match("{Dissolve} an enemy.", "");
}

#[test]
fn test_dissolve_all_characters() {
    assert_rendered_match("{Dissolve} all characters.", "");
}

#[test]
fn test_dissolve_enemy_lose_points() {
    assert_rendered_match("{Dissolve} an enemy. You lose {points($p)}.", "p: 4");
}

#[test]
fn test_dissolve_enemy_opponent_gains_points() {
    assert_rendered_match("{Dissolve} an enemy. The opponent gains {points($p)}.", "p: 3");
}

#[test]
fn test_dissolve_enemy_by_spark_or_less() {
    assert_rendered_match("{Dissolve} an enemy with spark {$s} or less.", "s: 1");
}

#[test]
fn test_dissolve_enemy_by_spark_or_more() {
    assert_rendered_match("{Dissolve} an enemy with spark {$s} or more.", "s: 3");
}

#[test]
fn test_dissolve_enemy_by_cost_or_less() {
    assert_rendered_match("{Dissolve} an enemy with cost {energy($e)} or less.", "e: 2");
}

#[test]
fn test_dissolve_enemy_by_cost_or_more_with_reclaim() {
    assert_rendered_match("{Dissolve} an enemy with cost {energy($e)} or more.", "e: 3");
    assert_rendered_match("{Reclaim_For_Cost($r)}", "r: 2");
}

#[test]
fn test_dissolve_enemy_by_subtype_count() {
    assert_rendered_match(
        "{Dissolve} an enemy with cost less than the number of allied {@plural subtype($t)}.",
        "t: Warrior",
    );
}

#[test]
fn test_dissolve_enemy_by_void_count() {
    assert_rendered_match(
        "{Dissolve} an enemy with cost less than the number of cards in your void.",
        "",
    );
}

#[test]
fn test_dissolve_enemy_draw() {
    assert_rendered_match("{Dissolve} an enemy. Draw {cards($c)}.", "c: 1");
}

#[test]
fn test_dissolve_by_energy_paid() {
    assert_rendered_match("Pay 1 or more {energy_symbol}: {Dissolve} all characters with spark less than the amount of {energy_symbol} paid.", "");
}

#[test]
fn test_dissolve_enemy_with_reclaim_cost() {
    assert_rendered_match("{Dissolve} an enemy with cost {energy($e)} or less.", "e: 2");
    assert_rendered_match("{Reclaim} -- Abandon an ally", "");
}

// ============================================================================
// Banish effects
// ============================================================================

#[test]
fn test_banish_enemy_by_cost() {
    assert_rendered_match("{Banish} an enemy with cost {energy($e)} or less.", "e: 2");
}

#[test]
fn test_banish_non_subtype_enemy() {
    assert_rendered_match("{Banish} a non-{subtype($t)} enemy.", "t: Warrior");
}

#[test]
fn test_banish_allies_then_materialize() {
    assert_rendered_match(
        "{Banish} {up_to_n_allies($n)}, then {materialize} {pronoun:$n}.",
        "n: 2",
    );
}

#[test]
fn test_banish_ally_materialize_at_end_of_turn() {
    assert_rendered_match("{Banish} an ally. {Materialize} it at end of turn.", "");
    assert_rendered_match("{Reclaim_For_Cost($r)}", "r: 1");
}

// ============================================================================
// Prevent effects
// ============================================================================

#[test]
fn test_prevent_played_card() {
    assert_rendered_match("{Prevent} a played card.", "");
}

#[test]
fn test_prevent_played_event_unless_pay() {
    assert_rendered_match(
        "{Prevent} a played event unless the opponent pays {energy($e)}.",
        "e: 2",
    );
}

#[test]
fn test_prevent_played_fast_card() {
    assert_rendered_match("{Prevent} a played {fast} card.", "");
}

#[test]
fn test_prevent_dissolve_event() {
    assert_rendered_match("{Prevent} a played event which could {dissolve} an ally.", "");
}

#[test]
fn test_prevent_played_character() {
    assert_rendered_match("{Prevent} a played character.", "");
}

#[test]
fn test_prevent_played_card_put_on_deck() {
    assert_rendered_match("{Prevent} a played card. Put it on top of the opponent's deck.", "");
}

// ============================================================================
// Draw and discard effects
// ============================================================================

#[test]
fn test_draw_cards() {
    assert_rendered_match("Draw {cards($c)}.", "c: 3");
}

#[test]
fn test_draw_one() {
    assert_rendered_match("Draw {cards($c)}.", "c: 1");
}

#[test]
fn test_draw_discard_with_reclaim() {
    assert_rendered_match("Draw {cards($c)}. Discard {cards($d)}.", "c: 2\nd: 2");
    assert_rendered_match("{Reclaim_For_Cost($r)}", "r: 2");
}

#[test]
fn test_draw_discard() {
    assert_rendered_match("Draw {cards($c)}. Discard {cards($d)}.", "c: 3\nd: 2");
}

#[test]
fn test_discard_draw() {
    assert_rendered_match("Discard {cards($d)}. Draw {cards($c)}.", "d: 1\nc: 2");
}

#[test]
fn test_discard_chosen_from_opponent_hand_by_cost() {
    assert_rendered_match(
        "Discard a chosen card with cost {energy($e)} or less from the opponent's hand.",
        "e: 3",
    );
}

#[test]
fn test_discard_chosen_character_from_opponent_hand() {
    assert_rendered_match("Discard a chosen character from the opponent's hand.", "");
}

#[test]
fn test_return_character_to_hand_draw() {
    assert_rendered_match("Return an enemy or ally to hand. Draw {cards($c)}.", "c: 1");
}

#[test]
fn test_may_return_character_from_void_draw() {
    assert_rendered_match(
        "You may return a character from your void to your hand, then draw {cards($c)}.",
        "c: 1",
    );
}

#[test]
fn test_draw_per_cards_played() {
    assert_rendered_match("Draw {cards($c)} for each card you have played this turn.", "c: 1");
}

#[test]
fn test_return_events_from_void() {
    assert_rendered_match("Return {up_to_n_events($n)} from your void to your hand.", "n: 2");
}

#[test]
fn test_put_top_cards_to_void_draw() {
    assert_rendered_match(
        "Put the {top_n_cards($v)} of your deck into your void. Draw {cards($c)}.",
        "v: 3\nc: 1",
    );
}

// ============================================================================
// Energy and points effects
// ============================================================================

#[test]
fn test_gain_energy() {
    assert_rendered_match("Gain {energy($e)}.", "e: 3");
}

#[test]
fn test_gain_four_energy() {
    assert_rendered_match("Gain {energy($e)}.", "e: 4");
}

#[test]
fn test_gain_six_energy() {
    assert_rendered_match("Gain {energy($e)}.", "e: 6");
}

#[test]
fn test_gain_points_per_cards_played() {
    assert_rendered_match("Gain {points($p)} for each card you have played this turn.", "p: 1");
}

#[test]
fn test_multiply_energy() {
    assert_rendered_match("{multiply_by($n)} the amount of {energy_symbol} you have.", "n: 2");
}

#[test]
fn test_multiply_energy_gain() {
    assert_rendered_match(
        "{multiply_by($n)} the amount of {energy_symbol} you gain from card effects this turn.",
        "n: 2",
    );
}

#[test]
fn test_multiply_draw() {
    assert_rendered_match(
        "{multiply_by($n)} the number of cards you draw from card effects this turn.",
        "n: 2",
    );
}

#[test]
fn test_gain_energy_draw() {
    assert_rendered_match("Gain {energy($e)}. Draw {cards($c)}.", "e: 2\nc: 1");
}

#[test]
fn test_draw_discard_gain_energy() {
    assert_rendered_match(
        "Draw {cards($c)}. Discard {cards($d)}. Gain {energy($e)}.",
        "c: 2\nd: 2\ne: 2",
    );
}

// ============================================================================
// Reclaim and foresee effects
// ============================================================================

#[test]
fn test_all_void_gains_reclaim_equal_cost() {
    assert_rendered_match(
        "All cards currently in your void gain {reclaim} equal to their cost this turn.",
        "",
    );
}

#[test]
fn test_event_gains_reclaim_for_cost() {
    assert_rendered_match("An event in your void gains {reclaim} equal to its cost this turn.", "");
}

#[test]
fn test_foresee_draw_with_reclaim() {
    assert_rendered_match("{Foresee($f)}. Draw {cards($c)}.", "f: 1\nc: 1");
    assert_rendered_match("{Reclaim_For_Cost($r)}", "r: 3");
}

// ============================================================================
// Turn and phase effects
// ============================================================================

#[test]
fn test_extra_judgment_phase() {
    assert_rendered_match(
        "At the end of this turn, trigger an additional {judgment_phase_name} phase.",
        "",
    );
}

#[test]
fn test_extra_turn() {
    assert_rendered_match("Take an extra turn after this one.", "");
}

// ============================================================================
// Copy effects
// ============================================================================

#[test]
fn test_copy_next_event_times() {
    assert_rendered_match("Copy the next event you play {this_turn_times($n)}.", "n: 3");
    assert_rendered_match("{Reclaim_For_Cost($r)}", "r: 2");
}

// ============================================================================
// Shuffle effects
// ============================================================================

#[test]
fn test_shuffle_and_draw() {
    assert_rendered_match(
        "Each player shuffles their hand and void into their deck and then draws {cards($c)}.",
        "c: 5",
    );
}

// ============================================================================
// Spark modification effects
// ============================================================================

#[test]
fn test_ally_gains_spark_per_subtype() {
    assert_rendered_match(
        "An ally gains +{$s} spark for each allied {subtype($t)}.",
        "s: 1\nt: Warrior",
    );
}

// ============================================================================
// Materialize effects
// ============================================================================

#[test]
fn test_materialize_figments_per_cards_played() {
    assert_rendered_match(
        "{Materialize} {@a figment($g)} for each card you have played this turn.",
        "g: celestial",
    );
}

#[test]
fn test_materialize_multiple_figments() {
    assert_rendered_match("{Materialize} {n_figments($n, $g)}.", "n: 3\ng: radiant");
}

#[test]
fn test_materialize_random_characters_from_deck() {
    assert_rendered_match(
        "{Materialize} {n_random_characters($n)} with cost {energy($e)} or less from your deck.",
        "e: 3\nn: 2",
    );
}

// ============================================================================
// Choose one effects
// ============================================================================

#[test]
fn test_choose_one_return_or_draw() {
    assert_rendered_match("{choose_one}\n{bullet} {energy($e1)}: Return an enemy to hand.\n{bullet} {energy($e2)}: Draw {cards($c)}.", "e1: 2\ne2: 3\nc: 2");
}

// ============================================================================
// Pronoun-sensitive effect round-trip tests
// ============================================================================

#[test]
fn test_it_gains_reclaim_equal_cost_this_turn() {
    assert_rendered_match("{Discover} a card. It gains {reclaim} equal to its cost this turn.", "");
}

#[test]
fn test_it_gains_reclaim_for_cost_this_turn() {
    assert_rendered_match("{Discover} a card. It gains {reclaim_for_cost($r)} this turn.", "r: 1");
}
