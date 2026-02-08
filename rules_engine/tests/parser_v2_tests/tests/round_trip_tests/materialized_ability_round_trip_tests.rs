//! Round-trip tests for Materialized/Dissolved phase abilities.
//!
//! Tests abilities that trigger when characters are materialized or dissolved.

use parser_v2_tests::test_helpers::*;

#[test]
fn test_materialized_discard_then_draw() {
    assert_round_trip("{Materialized} Discard {cards($d)}, then draw {cards($c)}.", "d: 2\nc: 2");
}

#[test]
fn test_materialized_dissolved_draw() {
    assert_round_trip("{Materialized_Dissolved} Draw {cards($c)}.", "c: 1");
}

#[test]
fn test_materialized_draw_subtype_from_deck() {
    assert_round_trip("{Materialized} Draw {@a subtype($t)} from your deck.", "t: Warrior");
}

#[test]
fn test_materialized_disable_enemy_abilities() {
    assert_round_trip("Disable the {Materialized} abilities of enemies.", "");
}

#[test]
fn test_materialized_disable_activated_abilities() {
    assert_round_trip("{Materialized} Disable the activated abilities of an enemy while this character is in play.", "");
}

#[test]
fn test_materialized_draw_one() {
    assert_round_trip("{Materialized} Draw {cards($c)}.", "c: 1");
}

#[test]
fn test_materialized_foresee() {
    assert_round_trip("{Materialized} {Foresee($f)}.", "f: 2");
}

#[test]
fn test_materialized_prevent_played_card_by_cost() {
    assert_round_trip(
        "{Materialized} {Prevent} a played card with cost {energy($e)} or less.",
        "e: 2",
    );
}

#[test]
fn test_materialized_gain_control_by_cost() {
    assert_round_trip(
        "{Materialized} Gain control of an enemy with cost {energy($e)} or less.",
        "e: 2",
    );
}

#[test]
fn test_materialized_dissolve_with_abandon_cost() {
    assert_round_trip(
        "Abandon an ally: Play this character for {energy($e)}, then abandon it.",
        "e: 0",
    );
    assert_round_trip("{Materialized} {Dissolve} an enemy.", "");
}

#[test]
fn test_materialized_give_event_reclaim_equal_cost() {
    assert_round_trip(
        "{Materialized} An event in your void gains {reclaim} equal to its cost this turn.",
        "",
    );
}

#[test]
fn test_materialized_draw_discard_with_reclaim() {
    assert_round_trip("{Materialized} Draw {cards($c)}, then discard {cards($d)}.", "c: 1\nd: 1");
    assert_round_trip("{Reclaim_For_Cost($r)}", "r: 3");
}

#[test]
fn test_materialized_return_enemy_to_hand() {
    assert_round_trip("{Materialized} Return an enemy to hand.", "");
}

#[test]
fn test_materialized_discover_fast_event() {
    assert_round_trip("{Materialized} {Discover} a {fast} event.", "");
}

#[test]
fn test_materialized_copy_event_multiple_times() {
    assert_round_trip("{Materialized} Copy the next event you play {this_turn_times($n)}.", "n: 3");
}

#[test]
fn test_materialized_judgment_gain_energy() {
    assert_round_trip("{Materialized_Judgment} Gain {energy($e)}.", "e: 2");
}

#[test]
fn test_materialized_judgment_kindle() {
    assert_round_trip("{Materialized_Judgment} {Kindle($k)}.", "k: 1");
}

#[test]
fn test_materialized_return_character_from_void() {
    assert_round_trip("{Materialized} Return a character from your void to your hand.", "");
}

#[test]
fn test_materialized_judgment_gain_one_energy() {
    assert_round_trip("{Materialized_Judgment} Gain {energy($e)}.", "e: 1");
}

#[test]
fn test_materialized_banish_opponent_void() {
    assert_round_trip("{Materialized} {Banish} the opponent's void.", "");
}

#[test]
fn test_materialized_draw_per_subtype() {
    assert_round_trip(
        "{Materialized} Draw {cards($c)} for each allied {subtype($t)}.",
        "c: 1\nt: SpiritAnimal",
    );
}

#[test]
fn test_materialized_may_banish_ally_then_materialize() {
    assert_round_trip("{Materialized} You may {banish} an ally, then {materialize} it.", "");
}

#[test]
fn test_materialized_card_gains_reclaim_for_cost() {
    assert_round_trip("{Materialized} A card in your void gains {reclaim} equal to its cost.", "");
}

#[test]
fn test_materialized_banish_any_allies_then_materialize() {
    assert_round_trip("{Materialized} {Banish} any number of allies, then {materialize} them.", "");
}

#[test]
fn test_materialized_discard_chosen_from_opponent_hand() {
    assert_round_trip(
        "{Materialized} Discard a chosen card from the opponent's hand. They draw {cards($c)}.",
        "c: 1",
    );
}

#[test]
fn test_materialized_draw_two() {
    assert_round_trip("{Materialized} Draw {cards($c)}.", "c: 2");
}

#[test]
fn test_materialized_banish_enemy_until_leaves_play() {
    assert_round_trip("{Materialized} {Banish} an enemy until this character leaves play.", "");
}

#[test]
fn test_materialized_banish_enemy_until_next_main_phase() {
    assert_round_trip("{Materialized} {Banish} an enemy until your next main phase.", "");
}

#[test]
fn test_materialized_gain_three_energy() {
    assert_round_trip("{Materialized} Gain {energy($e)}.", "e: 3");
}

#[test]
fn test_materialized_may_return_ally_to_hand() {
    assert_round_trip("{Materialized} You may return an ally to hand.", "");
}

#[test]
fn test_materialized_return_ally_to_hand() {
    assert_round_trip("{Materialized} Return an ally to hand.", "");
}

#[test]
fn test_materialized_each_player_discards() {
    assert_round_trip("{Materialized} Each player discards {cards($d)}.", "d: 1");
}

#[test]
fn test_materialized_draw_per_ally_abandoned() {
    assert_round_trip("{Materialized} Draw {cards($c)} for each ally abandoned this turn.", "c: 1");
}

#[test]
fn test_materialized_dissolved_put_top_cards_to_void() {
    assert_round_trip(
        "{Materialized_Dissolved} Put the {top_n_cards($v)} of your deck into your void.",
        "v: 4",
    );
}

#[test]
fn test_dissolved_kindle_on_subtype() {
    assert_round_trip("{Dissolved} {Kindle($k)}.", "k: 2");
    assert_round_trip(
        "When an allied {subtype($t)} is {dissolved}, {kindle($k)}.",
        "k: 2\nt: Survivor",
    );
}

#[test]
fn test_dissolved_draw_on_subtype() {
    assert_round_trip("{Dissolved} Draw {cards($c)}.", "c: 1");
    assert_round_trip(
        "When an allied {subtype($t)} is {dissolved}, draw {cards($c)}.",
        "c: 1\nt: Survivor",
    );
}

#[test]
fn test_dissolved_may_pay_return_to_hand() {
    assert_round_trip(
        "{Dissolved} You may pay {energy($e)} to return this character to your hand.",
        "e: 1",
    );
}

#[test]
fn test_dissolved_subtype_gains_reclaim() {
    assert_round_trip(
        "{Dissolved} {@cap @a subtype($t)} in your void gains {reclaim} equal to its cost.",
        "t: Survivor",
    );
}

#[test]
fn test_reveal_top_card_play_characters_from_top() {
    assert_round_trip("Reveal the top card of your deck.", "");
    assert_round_trip("You may play characters from the top of your deck.", "");
}
