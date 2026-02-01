//! Per-card round-trip tests for cards.toml.
//!
//! Each test verifies that a card's ability text round-trips
//! correctly through parse -> serialize.
//!
//! GENERATED FILE - Do not edit manually.
//! Regenerate with: python scripts/generate_round_trip_tests.py

use parser_v2_tests::test_helpers::*;

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_titan_of_forgotten_echoes() {
    assert_round_trip(
        "When you play {cards-numeral} in a turn, {reclaim} this character.",
        "cards: 2",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_beacon_of_tomorrow() {
    assert_round_trip("{Discover} a card with cost {e}.", "e: 2");
}

#[test]
fn test_round_trip_card_scrap_reclaimer() {
    assert_round_trip("{Judgment} Return this character from your void to your hand.", "");
}

#[test]
fn test_round_trip_card_evacuation_enforcer() {
    assert_round_trip(
        "{Judgment} You may draw {cards}, then discard {discards}.",
        "cards: 2\ndiscards: 3",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_moonlit_voyage() {
    assert_round_trip("Draw {cards}. Discard {discards}.", "cards: 2\ndiscards: 2\nreclaim :2");
    assert_round_trip("{ReclaimForCost}", "cards: 2\ndiscards: 2\nreclaim :2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_ridge_vortex_explorer() {
    assert_round_trip("When you discard this character, {materialize} it.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_pattern_seeker() {
    assert_round_trip(
        "{Judgment} You may discard {discards} to draw {cards} and gain {points}.",
        "discards: 1\ncards: 1\npoints: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_ashmaze_guide() {
    assert_round_trip(
        "When you discard a card, it gains {reclaim} equal to its cost this turn.",
        "",
    );
}

#[test]
fn test_round_trip_card_synaptic_sentinel() {
    assert_round_trip("{Judgment} {Foresee}.", "foresee: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_the_rising_god() {
    assert_round_trip("Abandon {count-allies}: {Reclaim} this character.", "allies: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_apocalypse_vigilante() {
    assert_round_trip("When you discard a card, gain {points}.", "points: 1");
}

#[test]
fn test_round_trip_card_ethereal_trailblazer() {
    assert_round_trip("{Judgment} Gain {e}.", "e: 1");
}

#[test]
fn test_round_trip_card_glimpse_of_infinity() {
    assert_round_trip("Gain {e}.", "e: 3");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_unleash_ruin() {
    assert_round_trip("{Dissolve} an enemy. You lose {points}.", "points: 4");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_forgotten_titan() {
    assert_round_trip("This character costs {e} if you have discarded a card this turn.", "e: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_eclipse_herald() {
    assert_round_trip("{Judgment} You may {banish} {cards} from your void to {dissolve} an enemy with cost {e} or less.", "cards: 3\ne: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_chronicle_reclaimer() {
    assert_round_trip("{Judgment} Draw {cards}, then discard {discards}.", "cards: 1\ndiscards: 1");
}

#[test]
fn test_round_trip_card_apocalypse() {
    assert_round_trip("{Dissolve} all characters.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_mother_of_flames() {
    assert_round_trip("When you discard a card, {kindle}.", "k: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_the_calling_night() {
    assert_round_trip(
        "{Judgment} Draw {cards}. The opponent gains {points}.",
        "cards: 1\npoints: 2",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_starsea_traveler() {
    assert_round_trip(
        "Once per turn, you may play a character with cost {e} or less from your void.",
        "e:2",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_urban_cipher() {
    assert_round_trip(
        "{Materialized} Discard {discards}, then draw {cards}.",
        "discards: 2\ncards: 2",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_reunion() {
    assert_round_trip(
        "You may return a character from your void to your hand. Draw {cards}.",
        "cards: 1",
    );
}

#[test]
fn test_round_trip_card_immolate() {
    assert_round_trip("{Dissolve} an enemy.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_duneveil_vanguard() {
    assert_round_trip(
        "{Judgment} You may discard a card to {dissolve} an enemy with spark {s} or less.",
        "s: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_torchbearer_of_the_abyss() {
    assert_round_trip(
        "Once per turn, when you discard a card, gain {e} and {kindle}.",
        "e: 1\nk: 2",
    );
}

#[test]
fn test_round_trip_card_the_devourer() {
    assert_round_trip(
        "{Banish} your void with {count} or more cards: {Reclaim} this character.",
        "count: 8",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_nocturne() {
    assert_round_trip(
        "{Materialized} Draw {cards}. Discard {discards}.",
        "cards: 1\ndiscards: 1\nreclaim: 3",
    );
    assert_round_trip("{ReclaimForCost}", "cards: 1\ndiscards: 1\nreclaim: 3");
}

#[test]
fn test_round_trip_card_abomination_of_memory() {
    assert_round_trip("This character's spark is equal to the number of cards in your void.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_the_dread_sovereign() {
    assert_round_trip("{Judgment} You may abandon {a-subtype} to {discover} {a-subtype} with cost {e} higher and {materialize} it.", "subtype: warrior\ne: 1");
}

#[test]
fn test_round_trip_card_tranquil_duelist() {
    assert_round_trip("{MaterializedDissolved} Draw {cards}.", "cards: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_voidshield_guardian() {
    assert_round_trip(
        "When the opponent plays an event which could {dissolve} an ally, {prevent} that card.",
        "",
    );
}

#[test]
fn test_round_trip_card_assault_leader() {
    assert_round_trip(
        "{e}: Gain +{s} spark for each allied {subtype}.",
        "e: 4\ns: 1\nsubtype: warrior",
    );
}

#[test]
fn test_round_trip_card_ride_of_the_vanguard() {
    assert_round_trip(
        "An ally gains +{s} spark for each allied {subtype}.",
        "s: 1\nsubtype: warrior",
    );
}

#[test]
fn test_round_trip_card_company_commander() {
    assert_round_trip(
        "When you {materialize} an allied {subtype}, this character gains +{s} spark.",
        "subtype: warrior\ns: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_lumin_gate_seer() {
    assert_round_trip(
        "Once per turn, when you {materialize} a character with cost {e} or less, draw {cards}.",
        "e: 2\ncards: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_veil_shatter() {
    assert_round_trip("{Banish} an enemy with cost {e} or less.", "e: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_judgment_of_the_blade() {
    assert_round_trip("{Banish} a non-{subtype} enemy.", "subtype: warrior");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_shatter_the_frail() {
    assert_round_trip("{Dissolve} an enemy with spark {s} or less.", "s: 1");
}

#[test]
fn test_round_trip_card_fury_of_the_clan() {
    assert_round_trip(
        "{Dissolve} an enemy with cost less than the number of allied {plural-subtype}.",
        "subtype: warrior",
    );
}

#[test]
fn test_round_trip_card_summons_of_the_bonded() {
    assert_round_trip("{Discover} {a-subtype}.", "subtype: warrior");
}

#[test]
fn test_round_trip_card_ashen_avenger() {
    assert_round_trip("{e}, {Banish} another card in your void: {Reclaim} this character.", "e: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_dreadcall_warden() {
    assert_round_trip(
        "{e}, Abandon an ally with spark {s} or less: Draw {cards}.",
        "e: 2\ns: 1\ncards: 2",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_wolfbond_chieftain() {
    assert_round_trip(
        "{MaterializedJudgment} With {count-allied-subtype}, gain {e}.",
        "subtype: warrior\nallies: 2\ne: 1",
    );
}

#[test]
fn test_round_trip_card_dawnblade_wanderer() {
    assert_round_trip("{MaterializedJudgment} Gain {e}.", "e: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_bloomweaver() {
    assert_round_trip("Once per turn, when you {materialize} a character, gain {e}.", "e: 1");
}

#[test]
fn test_round_trip_card_seeker_for_the_way() {
    assert_round_trip("{Materialized} Draw {a-subtype} from your deck.", "subtype: warrior");
}

#[test]
fn test_round_trip_card_rebirth_ritualist() {
    assert_round_trip(
        "{e}, Abandon a character, Discard your hand: Draw {cards}.",
        "e: 2\ncards: 3",
    );
}

#[test]
fn test_round_trip_card_pallid_arbiter() {
    assert_round_trip("Disable the {Materialized} abilities of enemies.", "");
}

#[test]
fn test_round_trip_card_skyflame_commander() {
    assert_round_trip("Allied {plural-subtype} have +{s} spark.", "subtype: warrior\ns: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_spirit_field_reclaimer() {
    assert_round_trip(
        "{Judgment} Pay {e} to {kindle} and {banish} {cards} from the opponent's void.",
        "e: 1\nk: 1\ncards: 1",
    );
}

#[test]
fn test_round_trip_card_cloaked_sentinel() {
    assert_round_trip("The opponent's events cost {e} more.", "e: 1");
}

#[test]
fn test_round_trip_card_twilight_suppressor() {
    assert_round_trip("{Materialized} Disable the activated abilities of an enemy while this character is in play.", "");
}

#[test]
fn test_round_trip_card_frost_visionary() {
    assert_round_trip("{Materialized} Draw {cards}.", "cards: 1");
}

#[test]
fn test_round_trip_card_forsworn_champion() {
    assert_round_trip("Abandon an ally: This character gains +{s} spark.", "s: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_blade_of_unity() {
    assert_round_trip(
        "This character's spark is equal to the number of allied {plural-subtype}.",
        "subtype: warrior",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_invoker_of_myths() {
    assert_round_trip(
        "Once per turn, when you {materialize} {a-subtype}, draw {cards}.",
        "subtype: warrior\ncards: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_speaker_for_the_forgotten() {
    assert_round_trip(
        "When you play {a-subtype}, {reclaim} a random character with cost {e} or less.",
        "subtype: warrior\ne: 3",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_infernal_rest() {
    assert_round_trip("Lose {maximum-energy}: Play this event for {e}.", "max: 1\ne: 0");
    assert_round_trip("{Prevent} a played card.", "max: 1\ne: 0");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_echoing_denial() {
    assert_round_trip("{Banish} a card from hand: Play this event for {e}.", "e: 0");
    assert_round_trip("{Prevent} a played card.", "e: 0");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_ripple_of_defiance() {
    assert_round_trip("{Prevent} a played event unless the opponent pays {e}.", "e: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_break_the_veil() {
    assert_round_trip(
        "Discard a chosen card with cost {e} or less from the opponent's hand.",
        "e: 3",
    );
}

#[test]
fn test_round_trip_card_lurking_dread() {
    assert_round_trip("Discard a chosen character from the opponent's hand.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_fell_the_mighty() {
    assert_round_trip("{Banish} a card from hand: Play this event for {e}.", "e: 0");
    assert_round_trip("{Dissolve} an enemy.", "e: 0");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_shattering_gambit() {
    assert_round_trip("{Dissolve} an enemy. The opponent gains {points}.", "points: 3");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_abolish() {
    assert_round_trip("{Prevent} a played card.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_guiding_light() {
    assert_round_trip("{Foresee}. Draw {cards}.", "foresee: 1\ncards: 1\nreclaim: 3");
    assert_round_trip("{ReclaimForCost}", "foresee: 1\ncards: 1\nreclaim: 3");
}

#[test]
fn test_round_trip_card_pyrokinetic_surge() {
    assert_round_trip("Abandon an ally or discard a card: {Dissolve} an enemy.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_threadbreaker() {
    assert_round_trip("{Materialized} {Prevent} a played card with cost {e} or less.", "e: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_cosmic_puppeteer() {
    assert_round_trip("{Materialized} Gain control of an enemy with cost {e} or less.", "e: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_obliterator_of_worlds() {
    assert_round_trip("Abandon an ally: Play this character for {e}, then abandon it.", "e: 0");
    assert_round_trip("{Materialized} {Dissolve} an enemy.", "e: 0");
}

#[test]
fn test_round_trip_card_horizon_follower() {
    assert_round_trip("{Judgment} Gain {points}.", "points: 1");
}

#[test]
fn test_round_trip_card_astral_navigators() {
    assert_round_trip("{Materialized} {Foresee}.", "foresee: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_boundless_wanderer() {
    assert_round_trip("Has all character types.", "allies: 3\ncards: 1");
    assert_round_trip(
        "{Judgment} With {count-allies} that share a character type, draw {cards}.",
        "allies: 3\ncards: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_sage_of_the_prelude() {
    assert_round_trip("Once per turn, when you play a {fast} card, draw {cards}.", "cards: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_intermezzo_balladeer() {
    assert_round_trip("When you play a {fast} card, this character gains +{s} spark.", "s: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_melodist_of_the_finale() {
    assert_round_trip("When you play a {fast} card, gain {points}.", "points: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_the_waking_titan() {
    assert_round_trip(
        "To play this card, return an ally with cost {e} or more to hand.",
        "e: 3\ncards: 1",
    );
    assert_round_trip("{Judgment} Draw {cards}.", "e: 3\ncards: 1");
}

#[test]
fn test_round_trip_card_virtuoso_of_harmony() {
    assert_round_trip("At the end of your turn, gain {e}.", "e: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_moonlit_dancer() {
    assert_round_trip("Characters in your hand have {fast}.", "e: 1");
    assert_round_trip("Once per turn, when you play a {fast} character, gain {e}.", "e: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_secrets_of_the_deep() {
    assert_round_trip("Pay 1 or more {energy-symbol}: Draw {cards} for each {energy-symbol} spent, then discard {discards}.", "cards: 1\ndiscards: 2");
}

#[test]
fn test_round_trip_card_abyssal_enforcer() {
    assert_round_trip("{Materialized} Return an enemy to hand.", "");
}

#[test]
fn test_round_trip_card_lantern_keeper() {
    assert_round_trip("{Judgment} Gain {points}.", "points: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_ashlight_caller() {
    assert_round_trip(
        "{Materialized} An event in your void gains {reclaim} equal to its cost this turn.",
        "",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_eternal_sentry() {
    assert_round_trip("When you draw {cards-numeral} in a turn, if this card is in your void, it gains {reclaim-for-cost} this turn.", "cards: 2\nreclaim: 1");
}

#[test]
fn test_round_trip_card_epiphany_unfolded() {
    assert_round_trip("Draw {cards}.", "cards: 3");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_return_to_nowhere() {
    assert_round_trip("Return an enemy or ally to hand. Draw {cards}.", "cards: 1");
}

#[test]
fn test_round_trip_card_keeper_of_the_tides() {
    assert_round_trip("{Materialized} {Discover} a {fast} event.", "");
}

#[test]
fn test_round_trip_card_illumination_of_glory() {
    assert_round_trip("Gain {points} for each card you have played this turn.", "points: 1");
}

#[test]
fn test_round_trip_card_echoes_of_the_journey() {
    assert_round_trip("Draw {cards} for each card you have played this turn.", "cards: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_echo_architect() {
    assert_round_trip("Events cost you {e} more.", "e: 2");
    assert_round_trip("When you play an event from your hand, copy it.", "e: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_cascade_of_reflections() {
    assert_round_trip("Until end of turn, when you play an event, copy it.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_echoes_of_eternity() {
    assert_round_trip("Copy the next event you play {this-turn-times}.", "number: 3\nreclaim: 2");
    assert_round_trip("{ReclaimForCost}", "number: 3\nreclaim: 2");
}

#[test]
fn test_round_trip_card_path_to_redemption() {
    assert_round_trip(
        "All cards currently in your void gain {reclaim} equal to their cost this turn.",
        "",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_skies_of_change() {
    assert_round_trip("Discard {discards}. Draw {cards}.", "discards: 1\ncards: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_the_power_within() {
    assert_round_trip(
        "{MultiplyBy} the number of cards you draw from card effects this turn.",
        "number: 2",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_whisper_of_the_past() {
    assert_round_trip("An event in your void gains {reclaim-for-cost} this turn.", "reclaim: 0");
}

#[test]
fn test_round_trip_card_surge_of_fury() {
    assert_round_trip(
        "At the end of this turn, trigger an additional {JudgmentPhaseName} phase.",
        "",
    );
}

#[test]
fn test_round_trip_card_moment_rewound() {
    assert_round_trip("Take an extra turn after this one.", "");
}

#[test]
fn test_round_trip_card_keeper_of_the_lightpath() {
    assert_round_trip("Events cost you {e} less.", "e: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_starcatcher() {
    assert_round_trip("When you play an event, gain {e}.", "e: 1");
}

#[test]
fn test_round_trip_card_flash_of_power() {
    assert_round_trip("Gain {e}.", "e: 6");
}

#[test]
fn test_round_trip_card_genesis_burst() {
    assert_round_trip("{MultiplyBy} the amount of {energy-symbol} you have.", "number: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_catalyst_ignition() {
    assert_round_trip(
        "{MultiplyBy} the amount of {energy-symbol} you gain from card effects this turn.",
        "number: 2",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_data_pulse() {
    assert_round_trip("Gain {e}. Draw {cards}.", "e: 2\ncards: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_a_new_advenure() {
    assert_round_trip("Draw {cards}. Discard {discards}. Gain {e}.", "cards: 2\ndiscards: 2\ne: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_the_ringleader() {
    assert_round_trip(
        "{Materialized} Copy the next event you play {this-turn-times}.",
        "number: 3",
    );
}

#[test]
fn test_round_trip_card_wheel_of_the_heavens() {
    assert_round_trip(
        "Each player shuffles their hand and void into their deck and then draws {cards}.",
        "cards: 5",
    );
}

#[test]
fn test_round_trip_card_arc_gate_opening() {
    assert_round_trip("Gain {e}.", "e: 4");
}

#[test]
fn test_round_trip_card_spirit_reaping() {
    assert_round_trip("Abandon an ally: Gain {energy-symbol} equal to that character's cost.", "");
}

#[test]
fn test_round_trip_card_weight_of_memory() {
    assert_round_trip(
        "{Dissolve} an enemy with cost less than the number of cards in your void.",
        "",
    );
}

#[test]
fn test_round_trip_card_knowledge_restored() {
    assert_round_trip("Draw {cards}.", "cards: 3");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_harvest_the_forgotten() {
    assert_round_trip(
        "Put the {top-n-cards} of your deck into your void. Draw {cards}.",
        "to-void: 3\ncards: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_maelstrom_denial() {
    assert_round_trip("{Prevent} a played {fast} card.", "");
}

#[test]
fn test_round_trip_card_door_to_possibility() {
    assert_round_trip("{Discover} an event.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_fragments_of_vision() {
    assert_round_trip("Draw {cards}. Discard {discards}.", "cards: 3\ndiscards: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_oracle_of_shifting_skies() {
    assert_round_trip("When you play an event, {foresee}.", "foresee: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_ripple_through_reality() {
    assert_round_trip("{Prevent} a played card. Put it on top of the opponent's deck.", "");
}

#[test]
fn test_round_trip_card_spirit_of_smoldering_echoes() {
    assert_round_trip(
        "When an event is put into your void, this character gains +{s} spark.",
        "s: 1",
    );
}

#[test]
fn test_round_trip_card_nexus_wayfinder() {
    assert_round_trip("Characters cost you {e} less.", "e: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_conduit_of_resonance() {
    assert_round_trip(
        "When you {materialize} a character, trigger the {Judgment} ability of each ally.",
        "",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_celestial_reverie() {
    assert_round_trip("Until end of turn, when you play a character, draw {cards}.", "cards: 1");
}

#[test]
fn test_round_trip_card_ghostlight_wolves() {
    assert_round_trip(
        "{Judgment} Gain {e} for each allied {subtype}.",
        "e: 1\nsubtype: spirit-animal",
    );
}

#[test]
fn test_round_trip_card_ethereal_courser() {
    assert_round_trip("{Materialized} You may return an ally to hand.", "");
}

#[test]
fn test_round_trip_card_nomad_of_endless_paths() {
    assert_round_trip("{Materialized} Return an ally to hand.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_blazing_emberwing() {
    assert_round_trip(
        "The '{Judgment}' ability of allies triggers when you {materialize} them.",
        "",
    );
}

#[test]
fn test_round_trip_card_dawnprowler_panther() {
    assert_round_trip(
        "When you {materialize} an allied {subtype}, gain {e}.",
        "subtype:spirit-animal\ne: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_luminwings() {
    assert_round_trip(
        "{Judgment} With {count-allied-subtype}, gain {e}.",
        "subtype:spirit-animal\nallies: 2\ne: 2",
    );
}

#[test]
fn test_round_trip_card_spirit_of_the_greenwood() {
    assert_round_trip("{Judgment} Gain {e} for each allied character.", "e: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_spirit_bond() {
    assert_round_trip(
        "Each allied {subtype} gains spark equal to the number of allied {plural-subtype}.",
        "subtype: spirit-animal",
    );
}

#[test]
fn test_round_trip_card_lumineth() {
    assert_round_trip("When you have no cards in your deck, you win the game.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_key_to_the_moment() {
    assert_round_trip(
        "Return all but one ally to hand: Draw {cards} for each ally returned.",
        "cards: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_the_bondweaver() {
    assert_round_trip(
        "When you {materialize} a character, this character gains +{s} spark.",
        "s: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_eternal_stag() {
    assert_round_trip(
        "When you play {a-subtype}, draw {cards}.",
        "subtype: spirit-animal\ncards: 1",
    );
}

#[test]
fn test_round_trip_card_spiritbound_alpha() {
    assert_round_trip(
        "{Judgment} You may pay {e} to have each allied {subtype} gain +{s} spark.",
        "e: 4\nsubtype:spirit-animal\ns: 2",
    );
}

#[test]
fn test_round_trip_card_sunshadow_eagle() {
    assert_round_trip(
        "When you {materialize} an allied {subtype}, that character gains +{s} spark.",
        "subtype:spirit-animal\ns:1",
    );
}

#[test]
fn test_round_trip_card_ebonwing() {
    assert_round_trip("{MaterializedJudgment} {Kindle}.", "k: 1");
}

#[test]
fn test_round_trip_card_shadowpaw() {
    assert_round_trip("{Materialized} Return a character from your void to your hand.", "");
}

#[test]
fn test_round_trip_card_looming_oracle() {
    assert_round_trip("{Materialized} Draw {cards}.", "cards: 1");
}

#[test]
fn test_round_trip_card_driftcaller_sovereign() {
    assert_round_trip("{MaterializedJudgment} Gain {e}.", "e: 1");
}

#[test]
fn test_round_trip_card_emerald_guardian() {
    assert_round_trip("{Judgment} Gain {e}.", "e: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_dreamborne_leviathan() {
    assert_round_trip("Reveal the top card of your deck.", "");
    assert_round_trip("You may play characters from the top of your deck.", "");
}

#[test]
fn test_round_trip_card_soulflame_predator() {
    assert_round_trip("{Materialized} {Banish} the opponent's void.", "");
}

#[test]
fn test_round_trip_card_seeker_of_the_radiant_wilds() {
    assert_round_trip(
        "{Materialized} Draw {cards} for each allied {subtype}.",
        "cards: 1\nsubtype:spirit-animal",
    );
}

#[test]
fn test_round_trip_card_mystic_runefish() {
    assert_round_trip(
        "{e}: The spark of each allied {subtype} becomes {s}.",
        "e: 3\nsubtype: spirit-animal\ns: 5",
    );
}

#[test]
fn test_round_trip_card_starlight_guide() {
    assert_round_trip("{Materialized} You may {banish} an ally, then {materialize} it.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_flickerveil_adept() {
    assert_round_trip(
        "{MaterializedJudgment} {Banish} an ally with spark {s} or less, then {materialize} it.",
        "s: 2",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_reclaimer_of_lost_paths() {
    assert_round_trip(
        "{Materialized} A card with cost {e} or less in your void gains {reclaim-for-cost}.",
        "e: 3\nreclaim: 0",
    );
}

#[test]
fn test_round_trip_card_aurora_rider() {
    assert_round_trip("{Materialized} {Banish} any number of allies, then {materialize} them.", "");
}

#[test]
fn test_round_trip_card_wraith_of_twisting_shadows() {
    assert_round_trip(
        "{Materialized} Discard a chosen card from the opponent's hand. They draw {cards}.",
        "cards: 1",
    );
}

#[test]
fn test_round_trip_card_keeper_of_forgotten_light() {
    assert_round_trip("{Materialized} Draw {cards}.", "cards: 2");
}

#[test]
fn test_round_trip_card_paradox_enforcer() {
    assert_round_trip("{Materialized} {Banish} an enemy until this character leaves play.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_passage_through_oblivion() {
    assert_round_trip("{Banish} an ally. {Materialize} it at end of turn.", "reclaim: 1");
    assert_round_trip("{ReclaimForCost}", "reclaim: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_herald_of_the_last_light() {
    assert_round_trip("{Fast} -- Abandon this character: {Prevent} a played event.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_call_to_the_unknown() {
    assert_round_trip("{Discover} a character with a {Materialized} ability.", "");
}

#[test]
fn test_round_trip_card_blooming_path_wanderer() {
    assert_round_trip("{Judgment} You may {banish} an ally, then {materialize} it.", "");
}

#[test]
fn test_round_trip_card_tideborne_voyager() {
    assert_round_trip("When an ally is {banished}, this character gains +{s} spark.", "s: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_pyrestone_avatar() {
    assert_round_trip("When an ally is {banished}, {kindle}.", "k: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_light_of_emergence() {
    assert_round_trip(
        "{Materialize} {n-random-characters} with cost {e} or less from your deck.",
        "e: 3\nnumber: 2",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_cragfall() {
    assert_round_trip("{Prevent} a played character.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_scorched_reckoning() {
    assert_round_trip("{Dissolve} an enemy with spark {s} or more.", "s: 3");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_abyssal_plunge() {
    assert_round_trip("{Dissolve} an enemy with cost {e} or more.", "e: 3\nreclaim: 2");
    assert_round_trip("{ReclaimForCost}", "e: 3\nreclaim: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_burst_of_obliteration() {
    assert_round_trip("Pay 1 or more {energy-symbol}: {Dissolve} each character with spark less than the amount of {energy-symbol} paid.", "");
}

#[test]
fn test_round_trip_card_dimensional_pathfinder() {
    assert_round_trip(
        "{Judgment} You may pay {e} to {banish} {up-to-n-allies}, then {materialize} {it-or-them}.",
        "e: 3\nnumber: 2",
    );
}

#[test]
fn test_round_trip_card_riftwalker() {
    assert_round_trip("{Materialized} {Banish} an enemy until your next main phase.", "");
}

#[test]
fn test_round_trip_card_portal_of_twin_paths() {
    assert_round_trip("{Banish} {up-to-n-allies}, then {materialize} {it-or-them}.", "number: 2");
}

#[test]
fn test_round_trip_card_aurora_channeler() {
    assert_round_trip("{Materialized} Gain {e}.", "e: 3");
}

#[test]
fn test_round_trip_card_mirrorlight_architect() {
    assert_round_trip("{e}: {Materialize} a copy of an ally.", "e: 4");
}

#[test]
fn test_round_trip_card_starlit_cascade() {
    assert_round_trip("Until end of turn, when an ally leaves play, gain {e}.", "e: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_kindred_sparks() {
    assert_round_trip(
        "With an allied {subtype}, you may play this card from your hand or void for {e}.",
        "subtype: survivor\ne: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_emberwatch_veteran() {
    assert_round_trip("{e}, Discard {discards}: {kindle}.", "e: 1\ndiscards: 1\nk: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_hope_s_vanguard() {
    assert_round_trip(
        "{MaterializedJudgment} With {count-allied-subtype}, draw {cards}.",
        "allies: 2\nsubtype: survivor\ncards: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_silent_avenger() {
    assert_round_trip("{Dissolved} {Kindle}.", "k: 2\nsubtype: survivor");
    assert_round_trip(
        "When an allied {subtype} is {dissolved}, {kindle}.",
        "k: 2\nsubtype: survivor",
    );
}

#[test]
fn test_round_trip_card_soulbinder() {
    assert_round_trip("Abandon an ally: Gain {e}.", "e: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_seer_of_the_fallen() {
    assert_round_trip("{Dissolved} Draw {cards}.", "cards: 1\nsubtype: survivor");
    assert_round_trip(
        "When an allied {subtype} is {dissolved}, draw {cards}.",
        "cards: 1\nsubtype: survivor",
    );
}

#[test]
fn test_round_trip_card_ashborn_necromancer() {
    assert_round_trip(
        "Abandon an ally: Put the {top-n-cards} of your deck into your void.",
        "to-void: 2",
    );
}

#[test]
fn test_round_trip_card_sunset_chronicler() {
    assert_round_trip("When an ally is {dissolved}, draw {cards}.", "cards: 1");
}

#[test]
fn test_round_trip_card_wasteland_arbitrator() {
    assert_round_trip("{Materialized} Each player discards {discards}.", "discards: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_dustborn_veteran() {
    assert_round_trip(
        "When an ally is {dissolved}, this card gains {reclaim-for-cost} this turn.",
        "reclaim: 1",
    );
}

#[test]
fn test_round_trip_card_avatar_of_cosmic_reckoning() {
    assert_round_trip("When an ally is {dissolved}, gain {points}.", "points: 1");
}

#[test]
fn test_round_trip_card_resilient_wanderer() {
    assert_round_trip("{Dissolved} You may pay {e} to return this character to your hand.", "e: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_twilight_reclaimer() {
    assert_round_trip(
        "{Dissolved} {ASubtype} in your void gains {reclaim} equal to its cost.",
        "subtype: survivor",
    );
}

#[test]
fn test_round_trip_card_wreckheap_survivor() {
    assert_round_trip(
        "{Judgment} You may pay {e} to return this character from your void to your hand.",
        "e: 1",
    );
}

#[test]
fn test_round_trip_card_revenant_of_the_lost() {
    assert_round_trip("You may only play this character from your void.", "");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_soulkindler() {
    assert_round_trip(
        "If this card is in your void, allied {plural-subtype} have +{s} spark.",
        "subtype: survivor\ns: 2",
    );
}

#[test]
fn test_round_trip_card_exiles_of_the_last_light() {
    assert_round_trip("Abandon an ally: {Kindle}.", "k: 1");
}

#[test]
fn test_round_trip_card_ruin_scavenger() {
    assert_round_trip(
        "{Judgment} You may {banish} {cards} from the opponent's void to gain {e}.",
        "cards: 1\ne: 1",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_unleashed_destruction() {
    assert_round_trip("{Dissolve} an enemy with cost {e} or less.", "e: 2");
    assert_round_trip("{Reclaim} -- Abandon an ally", "e: 2");
}

#[test]
fn test_round_trip_card_searcher_in_the_mists() {
    assert_round_trip(
        "{MaterializedDissolved} Put the {top-n-cards} of your deck into your void.",
        "to-void: 4",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_veil_of_the_wastes() {
    assert_round_trip(
        "When you {materialize} {a-subtype}, {reclaim} this character.",
        "subtype: survivor",
    );
}

#[test]
fn test_round_trip_card_through_the_rift() {
    assert_round_trip("{Discover} {a-subtype}.", "subtype: survivor");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_flagbearer_of_decay() {
    assert_round_trip(
        "When you play {a-subtype}, put the {top-n-cards} of your deck into your void.",
        "subtype: survivor\nto-void: 2",
    );
}

#[test]
fn test_round_trip_card_harvester_of_despair() {
    assert_round_trip("When you abandon an ally, this character gains +{s} spark.", "s: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_blade_of_oblivion() {
    assert_round_trip(
        "When you abandon {count-allies} in a turn, {dissolve} an enemy.",
        "allies: 2",
    );
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_fathomless_maw() {
    assert_round_trip("When you abandon a character, gain {points}.", "points: 1");
}

#[test]
fn test_round_trip_card_the_forsaker() {
    assert_round_trip("Abandon an ally, once per turn: Gain {points}.", "points: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_infernal_ascendant() {
    assert_round_trip("When you abandon an ally, {kindle}.", "k: 2");
}

#[test]
fn test_round_trip_card_ashen_remnant() {
    assert_round_trip(
        "Abandon an ally: You may put a character from your void on top of your deck.",
        "",
    );
}

#[test]
fn test_round_trip_card_shardwoven_tyrant() {
    assert_round_trip(
        "Abandon an ally: You may {dissolve} an enemy with spark less than that ally's spark.",
        "",
    );
}

#[test]
fn test_round_trip_card_volcanic_channeler() {
    assert_round_trip("When an ally is {dissolved}, gain {e}.", "e: 1");
}

#[test]
fn test_round_trip_card_grim_reclaimer() {
    assert_round_trip("Abandon an ally, once per turn: {Reclaim} {a-subtype}.", "subtype: warrior");
}

#[test]
fn test_round_trip_card_angel_of_the_eclipse() {
    assert_round_trip("When you {materialize} an ally, gain {e}.", "e: 1");
}

#[test]
fn test_round_trip_card_packcaller_of_shadows() {
    assert_round_trip(
        "{Materialize} {a-figment} for each card you have played this turn.",
        "figment: celestial",
    );
}

#[test]
fn test_round_trip_card_radiant_trio() {
    assert_round_trip("{Materialize} {n-figments}.", "number: 3\nfigment: radiant");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_endless_projection() {
    assert_round_trip("When you play a character, {materialize} {a-figment}.", "figment: halcyon");
}

#[test]
fn test_round_trip_card_prophet_of_the_consumed() {
    assert_round_trip("{Materialized} Draw {cards} for each ally abandoned this turn.", "cards: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_specter_of_silent_snow() {
    assert_round_trip("When you abandon a character, draw {cards}.", "cards: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_desperation() {
    assert_round_trip(
        "Abandon any number of allies: Draw {cards} for each ally abandoned.",
        "cards: 1",
    );
}

#[test]
fn test_round_trip_card_rite_of_summoning() {
    assert_round_trip("{Discover} a character with an activated ability.", "");
}

#[test]
fn test_round_trip_card_architect_of_memory() {
    assert_round_trip("While you have {count} or more cards in your void, they have {reclaim} equal to their cost.", "count: 7");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_momentum_of_the_fallen() {
    assert_round_trip("{Dissolve} an enemy. Draw {cards}.", "cards: 1\ne: 1");
    assert_round_trip("This event costs {e} if a character dissolved this turn.", "cards: 1\ne: 1");
}

#[test]
fn test_round_trip_card_pulse_of_sacrifice() {
    assert_round_trip("Abandon a character, Discard your hand: Gain {e}.", "e: 5");
}

#[test]
fn test_round_trip_card_nightmare_manifest() {
    assert_round_trip("{Judgment} Each player abandons a character.", "");
}

#[test]
fn test_round_trip_card_dreamscatter() {
    assert_round_trip(
        "Pay 1 or more {energy-symbol}: Draw {cards} for each {energy-symbol} spent.",
        "cards: 1",
    );
}

#[test]
fn test_round_trip_card_minstrel_of_falling_light() {
    assert_round_trip("{e}: Draw {cards}.", "e: 3\ncards: 1");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_sundown_surfer() {
    assert_round_trip(
        "When you play a card during the opponent's turn, this character gains +{s} spark.",
        "s: 1",
    );
}

#[test]
fn test_round_trip_card_archive_of_the_forgotten() {
    assert_round_trip("Return {up-to-n-events} from your void to your hand.", "number: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_break_the_sequence() {
    assert_round_trip("{ChooseOne}\n{bullet} {mode1-cost}: Return an enemy to hand.\n{bullet} {mode2-cost}: Draw {cards}.", "mode1-cost: 2\nmode2-cost: 3\ncards: 2");
}

#[ignore = "Round-trip mismatch"]
#[test]
fn test_round_trip_card_together_against_the_tide() {
    assert_round_trip("{Prevent} a played event which could {dissolve} an ally.", "");
}

#[test]
fn test_round_trip_card_nightmare() {
    assert_round_trip("Draw {cards}.", "cards: 1");
}
