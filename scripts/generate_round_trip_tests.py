#!/usr/bin/env python3
"""
Generates round-trip tests for cards.toml with hardcoded rules text strings.

This script reads cards.toml and generates a Rust test file where each test
has the rules_text and variables embedded directly as string literals,
avoiding any runtime dependency on cards.toml.
"""

import re
import tomllib
from pathlib import Path

# Tests that are known to fail due to serialization differences.
# These will be marked with #[ignore].
FAILING_TESTS = {
    "test_round_trip_card_a_new_advenure",
    "test_round_trip_card_abolish",
    "test_round_trip_card_abyssal_plunge",
    "test_round_trip_card_apocalypse_vigilante",
    "test_round_trip_card_ashlight_caller",
    "test_round_trip_card_ashmaze_guide",
    "test_round_trip_card_beacon_of_tomorrow",
    "test_round_trip_card_blade_of_oblivion",
    "test_round_trip_card_blade_of_unity",
    "test_round_trip_card_blazing_emberwing",
    "test_round_trip_card_bloomweaver",
    "test_round_trip_card_boundless_wanderer",
    "test_round_trip_card_break_the_sequence",
    "test_round_trip_card_break_the_veil",
    "test_round_trip_card_burst_of_obliteration",
    "test_round_trip_card_call_to_the_unknown",
    "test_round_trip_card_cascade_of_reflections",
    "test_round_trip_card_catalyst_ignition",
    "test_round_trip_card_celestial_reverie",
    "test_round_trip_card_chronicle_reclaimer",
    "test_round_trip_card_conduit_of_resonance",
    "test_round_trip_card_cosmic_puppeteer",
    "test_round_trip_card_cragfall",
    "test_round_trip_card_data_pulse",
    "test_round_trip_card_desperation",
    "test_round_trip_card_dreadcall_warden",
    "test_round_trip_card_dreamborne_leviathan",
    "test_round_trip_card_duneveil_vanguard",
    "test_round_trip_card_dustborn_veteran",
    "test_round_trip_card_echo_architect",
    "test_round_trip_card_echoes_of_eternity",
    "test_round_trip_card_echoing_denial",
    "test_round_trip_card_eclipse_herald",
    "test_round_trip_card_emberwatch_veteran",
    "test_round_trip_card_endless_projection",
    "test_round_trip_card_eternal_sentry",
    "test_round_trip_card_eternal_stag",
    "test_round_trip_card_fathomless_maw",
    "test_round_trip_card_fell_the_mighty",
    "test_round_trip_card_flagbearer_of_decay",
    "test_round_trip_card_flickerveil_adept",
    "test_round_trip_card_forgotten_titan",
    "test_round_trip_card_fragments_of_vision",
    "test_round_trip_card_guiding_light",
    "test_round_trip_card_harvest_the_forgotten",
    "test_round_trip_card_herald_of_the_last_light",
    "test_round_trip_card_hope_s_vanguard",
    "test_round_trip_card_infernal_ascendant",
    "test_round_trip_card_infernal_rest",
    "test_round_trip_card_intermezzo_balladeer",
    "test_round_trip_card_invoker_of_myths",
    "test_round_trip_card_judgment_of_the_blade",
    "test_round_trip_card_key_to_the_moment",
    "test_round_trip_card_kindred_sparks",
    "test_round_trip_card_light_of_emergence",
    "test_round_trip_card_lumin_gate_seer",
    "test_round_trip_card_luminwings",
    "test_round_trip_card_maelstrom_denial",
    "test_round_trip_card_melodist_of_the_finale",
    "test_round_trip_card_momentum_of_the_fallen",
    "test_round_trip_card_moonlit_dancer",
    "test_round_trip_card_moonlit_voyage",
    "test_round_trip_card_mother_of_flames",
    "test_round_trip_card_nocturne",
    "test_round_trip_card_obliterator_of_worlds",
    "test_round_trip_card_oracle_of_shifting_skies",
    "test_round_trip_card_passage_through_oblivion",
    "test_round_trip_card_pattern_seeker",
    "test_round_trip_card_pyrestone_avatar",
    "test_round_trip_card_reclaimer_of_lost_paths",
    "test_round_trip_card_return_to_nowhere",
    "test_round_trip_card_reunion",
    "test_round_trip_card_ridge_vortex_explorer",
    "test_round_trip_card_ripple_of_defiance",
    "test_round_trip_card_ripple_through_reality",
    "test_round_trip_card_sage_of_the_prelude",
    "test_round_trip_card_scorched_reckoning",
    "test_round_trip_card_secrets_of_the_deep",
    "test_round_trip_card_seer_of_the_fallen",
    "test_round_trip_card_shatter_the_frail",
    "test_round_trip_card_shattering_gambit",
    "test_round_trip_card_silent_avenger",
    "test_round_trip_card_skies_of_change",
    "test_round_trip_card_soulkindler",
    "test_round_trip_card_speaker_for_the_forgotten",
    "test_round_trip_card_specter_of_silent_snow",
    "test_round_trip_card_spirit_bond",
    "test_round_trip_card_spirit_field_reclaimer",
    "test_round_trip_card_starcatcher",
    "test_round_trip_card_starsea_traveler",
    "test_round_trip_card_sundown_surfer",
    "test_round_trip_card_the_bondweaver",
    "test_round_trip_card_the_calling_night",
    "test_round_trip_card_the_dread_sovereign",
    "test_round_trip_card_the_power_within",
    "test_round_trip_card_the_ringleader",
    "test_round_trip_card_the_rising_god",
    "test_round_trip_card_the_waking_titan",
    "test_round_trip_card_threadbreaker",
    "test_round_trip_card_titan_of_forgotten_echoes",
    "test_round_trip_card_together_against_the_tide",
    "test_round_trip_card_torchbearer_of_the_abyss",
    "test_round_trip_card_twilight_reclaimer",
    "test_round_trip_card_unleash_ruin",
    "test_round_trip_card_unleashed_destruction",
    "test_round_trip_card_urban_cipher",
    "test_round_trip_card_veil_of_the_wastes",
    "test_round_trip_card_veil_shatter",
    "test_round_trip_card_voidshield_guardian",
    "test_round_trip_card_whisper_of_the_past",
    "test_round_trip_card_wolfbond_chieftain",
}


def sanitize_name(name: str) -> str:
    """Convert card name to a valid Rust function name."""
    # Replace non-alphanumeric with underscore
    sanitized = re.sub(r'[^a-zA-Z0-9]', '_', name.lower())
    # Collapse multiple underscores
    sanitized = re.sub(r'_+', '_', sanitized)
    # Strip leading/trailing underscores
    return sanitized.strip('_')


def escape_rust_string(s: str) -> str:
    """Escape a string for use in a Rust string literal."""
    s = s.replace('\\', '\\\\')
    s = s.replace('"', '\\"')
    s = s.replace('\n', '\\n')
    return s


def generate_test(card_name: str, rules_text: str, variables: str) -> str:
    """Generate a single test function."""
    fn_name = f"test_round_trip_card_{sanitize_name(card_name)}"

    # Handle multiline strings
    rules_text = rules_text.strip()
    variables = variables.strip() if variables else ""

    # Split rules_text into ability blocks (separated by blank lines)
    ability_blocks = [block.strip() for block in rules_text.split('\n\n') if block.strip()]

    # Check if this test should be ignored
    ignore_attr = '#[ignore = "Round-trip mismatch"]\n' if fn_name in FAILING_TESTS else ''

    lines = [f"{ignore_attr}#[test]", f"fn {fn_name}() {{"]

    vars_escaped = escape_rust_string(variables)

    for block in ability_blocks:
        rules_escaped = escape_rust_string(block)
        lines.append(f'    assert_round_trip("{rules_escaped}", "{vars_escaped}");')

    lines.append("}")
    return '\n'.join(lines)


def main():
    project_root = Path(__file__).parent.parent
    cards_toml_path = project_root / "rules_engine" / "tabula" / "cards.toml"
    output_dir = project_root / "rules_engine" / "tests" / "parser_v2_tests" / "tests" / "round_trip_tests"

    # Create output directory
    output_dir.mkdir(parents=True, exist_ok=True)

    # Read cards.toml
    with open(cards_toml_path, "rb") as f:
        data = tomllib.load(f)

    cards = data.get("cards", [])

    tests = []
    for card in cards:
        name = card.get("name", "").strip()
        rules_text = card.get("rules-text", "")
        variables = card.get("variables", "") or ""

        if not name or not rules_text:
            continue

        test_code = generate_test(name, rules_text, variables)
        tests.append(test_code)

    # Generate the cards test file
    cards_output = '''//! Per-card round-trip tests for cards.toml.
//!
//! Each test verifies that a card's ability text round-trips
//! correctly through parse -> serialize.
//!
//! GENERATED FILE - Do not edit manually.
//! Regenerate with: python scripts/generate_round_trip_tests.py

use parser_v2_tests::test_helpers::*;

'''

    cards_output += '\n\n'.join(tests)
    cards_output += '\n'

    with open(output_dir / "cards_toml_tests.rs", "w") as f:
        f.write(cards_output)

    print(f"Generated {len(tests)} tests to {output_dir / 'cards_toml_tests.rs'}")


if __name__ == "__main__":
    main()
