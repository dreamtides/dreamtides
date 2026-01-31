//! Per-card round-trip tests for cards.toml.
//!
//! Each test verifies that a card's ability text round-trips
//! correctly through parse -> serialize.

use parser_v2_tests::test_helpers::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CardsFile {
    cards: Vec<Card>,
}

#[derive(Debug, Deserialize)]
struct Card {
    name: String,
    #[serde(rename = "rules-text")]
    rules_text: Option<String>,
    variables: Option<String>,
}

/// Path to cards.toml relative to test directory.
const CARDS_TOML_PATH: &str = "../../tabula/cards.toml";

/// Loads cards.toml and returns the list of cards.
fn load_cards() -> Vec<Card> {
    let cards_toml = std::fs::read_to_string(CARDS_TOML_PATH).expect("Failed to read cards.toml");
    let cards_file: CardsFile = toml::from_str(&cards_toml).expect("Failed to parse cards.toml");
    cards_file.cards
}

/// Finds a card by name.
fn find_card<'a>(cards: &'a [Card], name: &str) -> &'a Card {
    cards.iter().find(|c| c.name == name).unwrap_or_else(|| panic!("Card not found: {name}"))
}

/// Tests round-trip for all ability blocks of a card.
fn test_card_round_trip(card: &Card) {
    let Some(rules_text) = &card.rules_text else {
        return;
    };

    let variables = card.variables.as_deref().unwrap_or("");

    for ability_block in rules_text.split("\n\n") {
        let ability_block = ability_block.trim();
        if ability_block.is_empty() {
            continue;
        }
        assert_round_trip(ability_block, variables);
    }
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_titan_of_forgotten_echoes() {
    let cards = load_cards();
    let card = find_card(&cards, "Titan of Forgotten Echoes");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_beacon_of_tomorrow() {
    let cards = load_cards();
    let card = find_card(&cards, "Beacon of Tomorrow");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_scrap_reclaimer() {
    let cards = load_cards();
    let card = find_card(&cards, "Scrap Reclaimer");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_evacuation_enforcer() {
    let cards = load_cards();
    let card = find_card(&cards, "Evacuation Enforcer");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_moonlit_voyage() {
    let cards = load_cards();
    let card = find_card(&cards, "Moonlit Voyage");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_ridge_vortex_explorer() {
    let cards = load_cards();
    let card = find_card(&cards, "Ridge Vortex Explorer");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_pattern_seeker() {
    let cards = load_cards();
    let card = find_card(&cards, "Pattern Seeker");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_ashmaze_guide() {
    let cards = load_cards();
    let card = find_card(&cards, "Ashmaze Guide");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_synaptic_sentinel() {
    let cards = load_cards();
    let card = find_card(&cards, "Synaptic Sentinel");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_the_rising_god() {
    let cards = load_cards();
    let card = find_card(&cards, "The Rising God");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_apocalypse_vigilante() {
    let cards = load_cards();
    let card = find_card(&cards, "Apocalypse Vigilante");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_ethereal_trailblazer() {
    let cards = load_cards();
    let card = find_card(&cards, "Ethereal Trailblazer");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_glimpse_of_infinity() {
    let cards = load_cards();
    let card = find_card(&cards, "Glimpse of Infinity");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_unleash_ruin() {
    let cards = load_cards();
    let card = find_card(&cards, "Unleash Ruin");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_forgotten_titan() {
    let cards = load_cards();
    let card = find_card(&cards, "Forgotten Titan");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_eclipse_herald() {
    let cards = load_cards();
    let card = find_card(&cards, "Eclipse Herald");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_chronicle_reclaimer() {
    let cards = load_cards();
    let card = find_card(&cards, "Chronicle Reclaimer");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_apocalypse() {
    let cards = load_cards();
    let card = find_card(&cards, "Apocalypse");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_mother_of_flames() {
    let cards = load_cards();
    let card = find_card(&cards, "Mother of Flames");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_the_calling_night() {
    let cards = load_cards();
    let card = find_card(&cards, "The Calling Night");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_starsea_traveler() {
    let cards = load_cards();
    let card = find_card(&cards, "Starsea Traveler");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_urban_cipher() {
    let cards = load_cards();
    let card = find_card(&cards, "Urban Cipher");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_reunion() {
    let cards = load_cards();
    let card = find_card(&cards, "Reunion");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_immolate() {
    let cards = load_cards();
    let card = find_card(&cards, "Immolate");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_duneveil_vanguard() {
    let cards = load_cards();
    let card = find_card(&cards, "Duneveil Vanguard");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_torchbearer_of_the_abyss() {
    let cards = load_cards();
    let card = find_card(&cards, "Torchbearer of the Abyss");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_the_devourer() {
    let cards = load_cards();
    let card = find_card(&cards, "The Devourer");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_nocturne() {
    let cards = load_cards();
    let card = find_card(&cards, "Nocturne");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_abomination_of_memory() {
    let cards = load_cards();
    let card = find_card(&cards, "Abomination of Memory");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_the_dread_sovereign() {
    let cards = load_cards();
    let card = find_card(&cards, "The Dread Sovereign");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_tranquil_duelist() {
    let cards = load_cards();
    let card = find_card(&cards, "Tranquil Duelist");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_voidshield_guardian() {
    let cards = load_cards();
    let card = find_card(&cards, "Voidshield Guardian");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_assault_leader() {
    let cards = load_cards();
    let card = find_card(&cards, "Assault Leader");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_ride_of_the_vanguard() {
    let cards = load_cards();
    let card = find_card(&cards, "Ride of the Vanguard");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_company_commander() {
    let cards = load_cards();
    let card = find_card(&cards, "Company Commander");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_lumin_gate_seer() {
    let cards = load_cards();
    let card = find_card(&cards, "Lumin-Gate Seer");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_veil_shatter() {
    let cards = load_cards();
    let card = find_card(&cards, "Veil Shatter");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_judgment_of_the_blade() {
    let cards = load_cards();
    let card = find_card(&cards, "Judgment of the Blade");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_shatter_the_frail() {
    let cards = load_cards();
    let card = find_card(&cards, "Shatter the Frail");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_fury_of_the_clan() {
    let cards = load_cards();
    let card = find_card(&cards, "Fury of the Clan");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_summons_of_the_bonded() {
    let cards = load_cards();
    let card = find_card(&cards, "Summons of the Bonded");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_ashen_avenger() {
    let cards = load_cards();
    let card = find_card(&cards, "Ashen Avenger");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_dreadcall_warden() {
    let cards = load_cards();
    let card = find_card(&cards, "Dreadcall Warden");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_wolfbond_chieftain() {
    let cards = load_cards();
    let card = find_card(&cards, "Wolfbond Chieftain");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_dawnblade_wanderer() {
    let cards = load_cards();
    let card = find_card(&cards, "Dawnblade Wanderer");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_bloomweaver() {
    let cards = load_cards();
    let card = find_card(&cards, "Bloomweaver");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_seeker_for_the_way() {
    let cards = load_cards();
    let card = find_card(&cards, "Seeker for the Way");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_rebirth_ritualist() {
    let cards = load_cards();
    let card = find_card(&cards, "Rebirth Ritualist");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_pallid_arbiter() {
    let cards = load_cards();
    let card = find_card(&cards, "Pallid Arbiter");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_skyflame_commander() {
    let cards = load_cards();
    let card = find_card(&cards, "Skyflame Commander");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_spirit_field_reclaimer() {
    let cards = load_cards();
    let card = find_card(&cards, "Spirit Field Reclaimer");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_cloaked_sentinel() {
    let cards = load_cards();
    let card = find_card(&cards, "Cloaked Sentinel");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_twilight_suppressor() {
    let cards = load_cards();
    let card = find_card(&cards, "Twilight Suppressor");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_frost_visionary() {
    let cards = load_cards();
    let card = find_card(&cards, "Frost Visionary");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_forsworn_champion() {
    let cards = load_cards();
    let card = find_card(&cards, "Forsworn Champion");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_blade_of_unity() {
    let cards = load_cards();
    let card = find_card(&cards, "Blade of Unity");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_invoker_of_myths() {
    let cards = load_cards();
    let card = find_card(&cards, "Invoker of Myths");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_speaker_for_the_forgotten() {
    let cards = load_cards();
    let card = find_card(&cards, "Speaker for the Forgotten");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_infernal_rest() {
    let cards = load_cards();
    let card = find_card(&cards, "Infernal Rest");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_echoing_denial() {
    let cards = load_cards();
    let card = find_card(&cards, "Echoing Denial");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_ripple_of_defiance() {
    let cards = load_cards();
    let card = find_card(&cards, "Ripple of Defiance");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_break_the_veil() {
    let cards = load_cards();
    let card = find_card(&cards, "Break the Veil");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_lurking_dread() {
    let cards = load_cards();
    let card = find_card(&cards, "Lurking Dread");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_fell_the_mighty() {
    let cards = load_cards();
    let card = find_card(&cards, "Fell the Mighty");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_shattering_gambit() {
    let cards = load_cards();
    let card = find_card(&cards, "Shattering Gambit");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_abolish() {
    let cards = load_cards();
    let card = find_card(&cards, "Abolish");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_guiding_light() {
    let cards = load_cards();
    let card = find_card(&cards, "Guiding Light");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_pyrokinetic_surge() {
    let cards = load_cards();
    let card = find_card(&cards, "Pyrokinetic Surge");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_threadbreaker() {
    let cards = load_cards();
    let card = find_card(&cards, "Threadbreaker");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_cosmic_puppeteer() {
    let cards = load_cards();
    let card = find_card(&cards, "Cosmic Puppeteer");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_obliterator_of_worlds() {
    let cards = load_cards();
    let card = find_card(&cards, "Obliterator of Worlds");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_horizon_follower() {
    let cards = load_cards();
    let card = find_card(&cards, "Horizon Follower");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_astral_navigators() {
    let cards = load_cards();
    let card = find_card(&cards, "Astral Navigators");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_boundless_wanderer() {
    let cards = load_cards();
    let card = find_card(&cards, "Boundless Wanderer");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_sage_of_the_prelude() {
    let cards = load_cards();
    let card = find_card(&cards, "Sage of the Prelude");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_intermezzo_balladeer() {
    let cards = load_cards();
    let card = find_card(&cards, "Intermezzo Balladeer");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_melodist_of_the_finale() {
    let cards = load_cards();
    let card = find_card(&cards, "Melodist of the Finale");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_the_waking_titan() {
    let cards = load_cards();
    let card = find_card(&cards, "The Waking Titan");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_virtuoso_of_harmony() {
    let cards = load_cards();
    let card = find_card(&cards, "Virtuoso of Harmony");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_moonlit_dancer() {
    let cards = load_cards();
    let card = find_card(&cards, "Moonlit Dancer");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_secrets_of_the_deep() {
    let cards = load_cards();
    let card = find_card(&cards, "Secrets of the Deep");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_abyssal_enforcer() {
    let cards = load_cards();
    let card = find_card(&cards, "Abyssal Enforcer");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_lantern_keeper() {
    let cards = load_cards();
    let card = find_card(&cards, "Lantern Keeper");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_ashlight_caller() {
    let cards = load_cards();
    let card = find_card(&cards, "Ashlight Caller");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_eternal_sentry() {
    let cards = load_cards();
    let card = find_card(&cards, "Eternal Sentry");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_epiphany_unfolded() {
    let cards = load_cards();
    let card = find_card(&cards, "Epiphany Unfolded");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_return_to_nowhere() {
    let cards = load_cards();
    let card = find_card(&cards, "Return to Nowhere");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_keeper_of_the_tides() {
    let cards = load_cards();
    let card = find_card(&cards, "Keeper of the Tides");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_illumination_of_glory() {
    let cards = load_cards();
    let card = find_card(&cards, "Illumination of Glory");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_echoes_of_the_journey() {
    let cards = load_cards();
    let card = find_card(&cards, "Echoes of the Journey");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_echo_architect() {
    let cards = load_cards();
    let card = find_card(&cards, "Echo Architect");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_cascade_of_reflections() {
    let cards = load_cards();
    let card = find_card(&cards, "Cascade of Reflections");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_echoes_of_eternity() {
    let cards = load_cards();
    let card = find_card(&cards, "Echoes of Eternity");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_path_to_redemption() {
    let cards = load_cards();
    let card = find_card(&cards, "Path to Redemption");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_skies_of_change() {
    let cards = load_cards();
    let card = find_card(&cards, "Skies of Change");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_the_power_within() {
    let cards = load_cards();
    let card = find_card(&cards, "The Power Within");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_whisper_of_the_past() {
    let cards = load_cards();
    let card = find_card(&cards, "Whisper of the Past");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_surge_of_fury() {
    let cards = load_cards();
    let card = find_card(&cards, "Surge of Fury");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_moment_rewound() {
    let cards = load_cards();
    let card = find_card(&cards, "Moment Rewound");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_keeper_of_the_lightpath() {
    let cards = load_cards();
    let card = find_card(&cards, "Keeper of the Lightpath");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_starcatcher() {
    let cards = load_cards();
    let card = find_card(&cards, "Starcatcher");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_flash_of_power() {
    let cards = load_cards();
    let card = find_card(&cards, "Flash of Power");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_genesis_burst() {
    let cards = load_cards();
    let card = find_card(&cards, "Genesis Burst");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_catalyst_ignition() {
    let cards = load_cards();
    let card = find_card(&cards, "Catalyst Ignition");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_data_pulse() {
    let cards = load_cards();
    let card = find_card(&cards, "Data Pulse");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_a_new_advenure() {
    let cards = load_cards();
    let card = find_card(&cards, "A New Advenure");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_the_ringleader() {
    let cards = load_cards();
    let card = find_card(&cards, "The Ringleader");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_wheel_of_the_heavens() {
    let cards = load_cards();
    let card = find_card(&cards, "Wheel of the Heavens");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_arc_gate_opening() {
    let cards = load_cards();
    let card = find_card(&cards, "Arc Gate Opening");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_spirit_reaping() {
    let cards = load_cards();
    let card = find_card(&cards, "Spirit Reaping");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_weight_of_memory() {
    let cards = load_cards();
    let card = find_card(&cards, "Weight of Memory");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_knowledge_restored() {
    let cards = load_cards();
    let card = find_card(&cards, "Knowledge Restored");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_harvest_the_forgotten() {
    let cards = load_cards();
    let card = find_card(&cards, "Harvest the Forgotten");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_maelstrom_denial() {
    let cards = load_cards();
    let card = find_card(&cards, "Maelstrom Denial");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_door_to_possibility() {
    let cards = load_cards();
    let card = find_card(&cards, "Door to Possibility");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_fragments_of_vision() {
    let cards = load_cards();
    let card = find_card(&cards, "Fragments of Vision");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_oracle_of_shifting_skies() {
    let cards = load_cards();
    let card = find_card(&cards, "Oracle of Shifting Skies");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_ripple_through_reality() {
    let cards = load_cards();
    let card = find_card(&cards, "Ripple Through Reality");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_spirit_of_smoldering_echoes() {
    let cards = load_cards();
    let card = find_card(&cards, "Spirit of Smoldering Echoes");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_nexus_wayfinder() {
    let cards = load_cards();
    let card = find_card(&cards, "Nexus Wayfinder");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_conduit_of_resonance() {
    let cards = load_cards();
    let card = find_card(&cards, "Conduit of Resonance");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_celestial_reverie() {
    let cards = load_cards();
    let card = find_card(&cards, "Celestial Reverie");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_ghostlight_wolves() {
    let cards = load_cards();
    let card = find_card(&cards, "Ghostlight Wolves");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_ethereal_courser() {
    let cards = load_cards();
    let card = find_card(&cards, "Ethereal Courser");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_nomad_of_endless_paths() {
    let cards = load_cards();
    let card = find_card(&cards, "Nomad of Endless Paths");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_blazing_emberwing() {
    let cards = load_cards();
    let card = find_card(&cards, "Blazing Emberwing");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_dawnprowler_panther() {
    let cards = load_cards();
    let card = find_card(&cards, "Dawnprowler Panther");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_luminwings() {
    let cards = load_cards();
    let card = find_card(&cards, "Luminwings");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_spirit_of_the_greenwood() {
    let cards = load_cards();
    let card = find_card(&cards, "Spirit of the Greenwood");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_spirit_bond() {
    let cards = load_cards();
    let card = find_card(&cards, "Spirit Bond");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_lumineth() {
    let cards = load_cards();
    let card = find_card(&cards, "Lumineth");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_key_to_the_moment() {
    let cards = load_cards();
    let card = find_card(&cards, "Key to the Moment");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_the_bondweaver() {
    let cards = load_cards();
    let card = find_card(&cards, "The Bondweaver");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_eternal_stag() {
    let cards = load_cards();
    let card = find_card(&cards, "Eternal Stag");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_spiritbound_alpha() {
    let cards = load_cards();
    let card = find_card(&cards, "Spiritbound Alpha");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_sunshadow_eagle() {
    let cards = load_cards();
    let card = find_card(&cards, "Sunshadow Eagle");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_ebonwing() {
    let cards = load_cards();
    let card = find_card(&cards, "Ebonwing");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_shadowpaw() {
    let cards = load_cards();
    let card = find_card(&cards, "Shadowpaw");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_looming_oracle() {
    let cards = load_cards();
    let card = find_card(&cards, "Looming Oracle");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_driftcaller_sovereign() {
    let cards = load_cards();
    let card = find_card(&cards, "Driftcaller Sovereign");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_emerald_guardian() {
    let cards = load_cards();
    let card = find_card(&cards, "Emerald Guardian");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_dreamborne_leviathan() {
    let cards = load_cards();
    let card = find_card(&cards, "Dreamborne Leviathan");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_soulflame_predator() {
    let cards = load_cards();
    let card = find_card(&cards, "Soulflame Predator");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_seeker_of_the_radiant_wilds() {
    let cards = load_cards();
    let card = find_card(&cards, "Seeker of the Radiant Wilds");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_mystic_runefish() {
    let cards = load_cards();
    let card = find_card(&cards, "Mystic Runefish");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_starlight_guide() {
    let cards = load_cards();
    let card = find_card(&cards, "Starlight Guide");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_flickerveil_adept() {
    let cards = load_cards();
    let card = find_card(&cards, "Flickerveil Adept");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_reclaimer_of_lost_paths() {
    let cards = load_cards();
    let card = find_card(&cards, "Reclaimer of Lost Paths");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_aurora_rider() {
    let cards = load_cards();
    let card = find_card(&cards, "Aurora Rider");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_wraith_of_twisting_shadows() {
    let cards = load_cards();
    let card = find_card(&cards, "Wraith of Twisting Shadows");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_keeper_of_forgotten_light() {
    let cards = load_cards();
    let card = find_card(&cards, "Keeper of Forgotten Light");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_paradox_enforcer() {
    let cards = load_cards();
    let card = find_card(&cards, "Paradox Enforcer");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_passage_through_oblivion() {
    let cards = load_cards();
    let card = find_card(&cards, "Passage Through Oblivion");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_herald_of_the_last_light() {
    let cards = load_cards();
    let card = find_card(&cards, "Herald of the Last Light");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_call_to_the_unknown() {
    let cards = load_cards();
    let card = find_card(&cards, "Call to the Unknown");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_blooming_path_wanderer() {
    let cards = load_cards();
    let card = find_card(&cards, "Blooming Path Wanderer");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_tideborne_voyager() {
    let cards = load_cards();
    let card = find_card(&cards, "Tideborne Voyager");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_pyrestone_avatar() {
    let cards = load_cards();
    let card = find_card(&cards, "Pyrestone Avatar");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_light_of_emergence() {
    let cards = load_cards();
    let card = find_card(&cards, "Light of Emergence");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_cragfall() {
    let cards = load_cards();
    let card = find_card(&cards, "Cragfall");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_scorched_reckoning() {
    let cards = load_cards();
    let card = find_card(&cards, "Scorched Reckoning");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_abyssal_plunge() {
    let cards = load_cards();
    let card = find_card(&cards, "Abyssal Plunge");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_burst_of_obliteration() {
    let cards = load_cards();
    let card = find_card(&cards, "Burst of Obliteration");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_dimensional_pathfinder() {
    let cards = load_cards();
    let card = find_card(&cards, "Dimensional Pathfinder");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_riftwalker() {
    let cards = load_cards();
    let card = find_card(&cards, "Riftwalker");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_portal_of_twin_paths() {
    let cards = load_cards();
    let card = find_card(&cards, "Portal of Twin Paths");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_aurora_channeler() {
    let cards = load_cards();
    let card = find_card(&cards, "Aurora Channeler");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_mirrorlight_architect() {
    let cards = load_cards();
    let card = find_card(&cards, "Mirrorlight Architect");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_starlit_cascade() {
    let cards = load_cards();
    let card = find_card(&cards, "Starlit Cascade");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_kindred_sparks() {
    let cards = load_cards();
    let card = find_card(&cards, "Kindred Sparks");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_emberwatch_veteran() {
    let cards = load_cards();
    let card = find_card(&cards, "Emberwatch Veteran");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_hope_s_vanguard() {
    let cards = load_cards();
    let card = find_card(&cards, "Hope's Vanguard");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_silent_avenger() {
    let cards = load_cards();
    let card = find_card(&cards, "Silent Avenger");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_soulbinder() {
    let cards = load_cards();
    let card = find_card(&cards, "Soulbinder");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_seer_of_the_fallen() {
    let cards = load_cards();
    let card = find_card(&cards, "Seer of the Fallen");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_ashborn_necromancer() {
    let cards = load_cards();
    let card = find_card(&cards, "Ashborn Necromancer");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_sunset_chronicler() {
    let cards = load_cards();
    let card = find_card(&cards, "Sunset Chronicler");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_wasteland_arbitrator() {
    let cards = load_cards();
    let card = find_card(&cards, "Wasteland Arbitrator");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_dustborn_veteran() {
    let cards = load_cards();
    let card = find_card(&cards, "Dustborn Veteran");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_avatar_of_cosmic_reckoning() {
    let cards = load_cards();
    let card = find_card(&cards, "Avatar of Cosmic Reckoning");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_resilient_wanderer() {
    let cards = load_cards();
    let card = find_card(&cards, "Resilient Wanderer");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_twilight_reclaimer() {
    let cards = load_cards();
    let card = find_card(&cards, "Twilight Reclaimer");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_wreckheap_survivor() {
    let cards = load_cards();
    let card = find_card(&cards, "Wreckheap Survivor");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_revenant_of_the_lost() {
    let cards = load_cards();
    let card = find_card(&cards, "Revenant of the Lost");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_soulkindler() {
    let cards = load_cards();
    let card = find_card(&cards, "Soulkindler");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_exiles_of_the_last_light() {
    let cards = load_cards();
    let card = find_card(&cards, "Exiles of the Last Light");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_ruin_scavenger() {
    let cards = load_cards();
    let card = find_card(&cards, "Ruin Scavenger");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_unleashed_destruction() {
    let cards = load_cards();
    let card = find_card(&cards, "Unleashed Destruction");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_searcher_in_the_mists() {
    let cards = load_cards();
    let card = find_card(&cards, "Searcher in the Mists");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_veil_of_the_wastes() {
    let cards = load_cards();
    let card = find_card(&cards, "Veil of the Wastes");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_through_the_rift() {
    let cards = load_cards();
    let card = find_card(&cards, "Through the Rift");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_flagbearer_of_decay() {
    let cards = load_cards();
    let card = find_card(&cards, "Flagbearer of Decay");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_harvester_of_despair() {
    let cards = load_cards();
    let card = find_card(&cards, "Harvester of Despair");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_blade_of_oblivion() {
    let cards = load_cards();
    let card = find_card(&cards, "Blade of Oblivion");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_fathomless_maw() {
    let cards = load_cards();
    let card = find_card(&cards, "Fathomless Maw");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_the_forsaker() {
    let cards = load_cards();
    let card = find_card(&cards, "The Forsaker");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_infernal_ascendant() {
    let cards = load_cards();
    let card = find_card(&cards, "Infernal Ascendant");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_ashen_remnant() {
    let cards = load_cards();
    let card = find_card(&cards, "Ashen Remnant");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_shardwoven_tyrant() {
    let cards = load_cards();
    let card = find_card(&cards, "Shardwoven Tyrant");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_volcanic_channeler() {
    let cards = load_cards();
    let card = find_card(&cards, "Volcanic Channeler");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_grim_reclaimer() {
    let cards = load_cards();
    let card = find_card(&cards, "Grim Reclaimer");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_angel_of_the_eclipse() {
    let cards = load_cards();
    let card = find_card(&cards, "Angel of the Eclipse");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_packcaller_of_shadows() {
    let cards = load_cards();
    let card = find_card(&cards, "Packcaller of Shadows");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_radiant_trio() {
    let cards = load_cards();
    let card = find_card(&cards, "Radiant Trio");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_endless_projection() {
    let cards = load_cards();
    let card = find_card(&cards, "Endless Projection");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_prophet_of_the_consumed() {
    let cards = load_cards();
    let card = find_card(&cards, "Prophet of the Consumed");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_specter_of_silent_snow() {
    let cards = load_cards();
    let card = find_card(&cards, "Specter of Silent Snow");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_desperation() {
    let cards = load_cards();
    let card = find_card(&cards, "Desperation");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_rite_of_summoning() {
    let cards = load_cards();
    let card = find_card(&cards, "Rite of Summoning");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_architect_of_memory() {
    let cards = load_cards();
    let card = find_card(&cards, "Architect of Memory");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_momentum_of_the_fallen() {
    let cards = load_cards();
    let card = find_card(&cards, "Momentum of the Fallen");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_pulse_of_sacrifice() {
    let cards = load_cards();
    let card = find_card(&cards, "Pulse of Sacrifice");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_nightmare_manifest() {
    let cards = load_cards();
    let card = find_card(&cards, "Nightmare Manifest");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_dreamscatter() {
    let cards = load_cards();
    let card = find_card(&cards, "Dreamscatter");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_minstrel_of_falling_light() {
    let cards = load_cards();
    let card = find_card(&cards, "Minstrel of Falling Light");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_sundown_surfer() {
    let cards = load_cards();
    let card = find_card(&cards, "Sundown Surfer");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_archive_of_the_forgotten() {
    let cards = load_cards();
    let card = find_card(&cards, "Archive of the Forgotten ");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_break_the_sequence() {
    let cards = load_cards();
    let card = find_card(&cards, "Break the Sequence");
    test_card_round_trip(card);
}

#[ignore = "Round-trip mismatch - see PLAN.md"]
#[test]
fn test_round_trip_card_together_against_the_tide() {
    let cards = load_cards();
    let card = find_card(&cards, "Together Against the Tide");
    test_card_round_trip(card);
}

#[test]
fn test_round_trip_card_nightmare() {
    let cards = load_cards();
    let card = find_card(&cards, "Nightmare");
    test_card_round_trip(card);
}
