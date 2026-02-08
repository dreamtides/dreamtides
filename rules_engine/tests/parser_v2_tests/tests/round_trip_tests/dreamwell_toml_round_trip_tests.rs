//! Round-trip tests for all dreamwells in dreamwell.toml.
//!
//! Verifies that parsing and serializing each dreamwell's rules text produces
//! rendered output matching the directly-rendered input text.

use parser_v2_tests::test_helpers;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DreamwellFile {
    dreamwell: Vec<Dreamwell>,
}

#[derive(Debug, Deserialize)]
struct Dreamwell {
    name: String,
    #[serde(rename = "rules-text")]
    rules_text: Option<String>,
    variables: Option<String>,
}

#[test]
fn test_all_dreamwell_toml_round_trip() {
    let dreamwell_toml = std::fs::read_to_string("../../tabula/dreamwell.toml")
        .expect("Failed to read dreamwell.toml");
    let dreamwell_file: DreamwellFile =
        toml::from_str(&dreamwell_toml).expect("Failed to parse dreamwell.toml");

    let mut errors = Vec::new();
    let mut success_count = 0;
    let mut total_abilities = 0;

    for dreamwell in &dreamwell_file.dreamwell {
        let Some(rules_text) = &dreamwell.rules_text else {
            continue;
        };

        let variables = dreamwell.variables.as_deref().unwrap_or("");

        for ability_block in rules_text.split("\n\n") {
            let ability_block = ability_block.trim();
            if ability_block.is_empty() {
                continue;
            }

            total_abilities += 1;

            match test_helpers::assert_rendered_match_for_toml(
                &dreamwell.name,
                ability_block,
                variables,
            ) {
                Ok(()) => success_count += 1,
                Err(error) => errors.push(error),
            }
        }
    }

    test_helpers::print_bulk_results("dreamwell.toml", success_count, total_abilities, &errors);

    if !errors.is_empty() {
        panic!("\n{} abilities failed rendered comparison (see details above)", errors.len());
    }
}
