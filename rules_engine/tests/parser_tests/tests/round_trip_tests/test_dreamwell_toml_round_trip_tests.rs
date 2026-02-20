//! Round-trip tests for all dreamwells in test-dreamwell.toml.
//!
//! Verifies that parsing and serializing each test dreamwell's rules text
//! produces rendered output matching the directly-rendered input text.

use parser_tests::test_helpers;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TestDreamwellFile {
    #[serde(rename = "test-dreamwell")]
    test_dreamwell: Vec<TestDreamwell>,
}

#[derive(Debug, Deserialize)]
struct TestDreamwell {
    name: String,
    #[serde(rename = "rules-text")]
    rules_text: Option<String>,
    variables: Option<String>,
}

#[test]
fn test_all_test_dreamwell_toml_round_trip() {
    let test_dreamwell_toml = std::fs::read_to_string("../../tabula/test-dreamwell.toml")
        .expect("Failed to read test-dreamwell.toml");
    let test_dreamwell_file: TestDreamwellFile =
        toml::from_str(&test_dreamwell_toml).expect("Failed to parse test-dreamwell.toml");

    let mut errors = Vec::new();
    let mut success_count = 0;
    let mut total_abilities = 0;

    for dreamwell in &test_dreamwell_file.test_dreamwell {
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

    test_helpers::print_bulk_results(
        "test-dreamwell.toml",
        success_count,
        total_abilities,
        &errors,
    );

    if !errors.is_empty() {
        panic!("\n{} abilities failed rendered comparison (see details above)", errors.len());
    }
}
