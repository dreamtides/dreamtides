use std::collections::{HashMap, HashSet};
use std::fs;

use parser_v2::variables::substitution::{
    COMPOUND_DIRECTIVES, DIRECTIVE_VARIABLE_MAPPINGS, VARIABLE_DIRECTIVES,
};

#[test]
fn test_variable_mappings_match_ftl_file() {
    let ftl_path = "../../src/tabula_cli/src/server/listeners/card_rules.ftl";
    let ftl_content =
        fs::read_to_string(ftl_path).expect("Failed to read card_rules.ftl - check relative path");

    let ftl_mappings = parse_ftl_directive_variables(&ftl_content);

    let rust_mappings: HashMap<&str, &str> = DIRECTIVE_VARIABLE_MAPPINGS.iter().copied().collect();
    let rust_variables: HashSet<&str> = VARIABLE_DIRECTIVES.iter().copied().collect();

    for (directive, ftl_var) in &ftl_mappings {
        if directive == ftl_var {
            assert!(
                rust_variables.contains(directive.as_str()),
                "Directive '{}' uses variable '{}' but '{}' is not in VARIABLE_DIRECTIVES",
                directive,
                ftl_var,
                ftl_var
            );
        } else {
            let rust_mapped_var = rust_mappings.get(directive.as_str());
            assert_eq!(
                rust_mapped_var,
                Some(&ftl_var.as_str()),
                "Directive '{}' maps to variable '{}' in .ftl but {:?} in Rust DIRECTIVE_VARIABLE_MAPPINGS",
                directive,
                ftl_var,
                rust_mapped_var
            );

            assert!(
                rust_variables.contains(ftl_var.as_str()),
                "Variable '{}' used by directive '{}' is not in VARIABLE_DIRECTIVES",
                ftl_var,
                directive
            );
        }
    }

    let expected_mappings: HashSet<&str> = ftl_mappings
        .iter()
        .filter(|(directive, var)| directive != var)
        .map(|(directive, _)| directive.as_str())
        .collect();

    let actual_mappings: HashSet<&str> = rust_mappings.keys().copied().collect();

    let missing_in_rust: Vec<_> = expected_mappings.difference(&actual_mappings).collect();
    assert!(
        missing_in_rust.is_empty(),
        "Directives in .ftl that map to different variables but missing from DIRECTIVE_VARIABLE_MAPPINGS: {:?}",
        missing_in_rust
    );

    let extra_in_rust: Vec<_> = actual_mappings.difference(&expected_mappings).collect();
    assert!(
        extra_in_rust.is_empty(),
        "Directives in DIRECTIVE_VARIABLE_MAPPINGS that don't exist in .ftl or don't need mapping: {:?}",
        extra_in_rust
    );
}

#[test]
fn test_all_ftl_variables_are_recognized() {
    let ftl_path = "../../src/tabula_cli/src/server/listeners/card_rules.ftl";
    let ftl_content =
        fs::read_to_string(ftl_path).expect("Failed to read card_rules.ftl - check relative path");

    let ftl_variables = extract_all_ftl_variables(&ftl_content);
    let rust_variables: HashSet<&str> = VARIABLE_DIRECTIVES.iter().copied().collect();

    for var in &ftl_variables {
        assert!(
            rust_variables.contains(var.as_str()),
            "Variable '{}' found in .ftl is not in VARIABLE_DIRECTIVES",
            var
        );
    }
}

fn parse_ftl_directive_variables(content: &str) -> HashMap<String, String> {
    let mut mappings = HashMap::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.starts_with('-') || line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        if let Some(directive_name) = line.strip_suffix(" =") {
            let directive = directive_name.trim();

            if COMPOUND_DIRECTIVES.contains(&directive) {
                i += 1;
                continue;
            }

            if i + 1 < lines.len() {
                let next_line = lines[i + 1].trim();

                if next_line == "{" && i + 2 < lines.len() {
                    let var_line = lines[i + 2].trim();
                    if let Some(var_name) = extract_variable_from_pattern(var_line) {
                        mappings.insert(directive.to_string(), var_name);
                    }
                }
            }
        }

        i += 1;
    }

    mappings
}

fn extract_variable_from_pattern(line: &str) -> Option<String> {
    if let Some(rest) = line.strip_prefix('$') {
        if let Some(end) = rest.find(" ->") {
            return Some(rest[..end].trim().to_string());
        }
    }
    None
}

fn extract_all_ftl_variables(content: &str) -> HashSet<String> {
    let mut variables = HashSet::new();

    for line in content.lines() {
        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '$' {
                let mut var_name = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_alphanumeric() || next_ch == '_' || next_ch == '-' {
                        var_name.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if !var_name.is_empty() {
                    variables.insert(var_name);
                }
            }
        }
    }

    variables.remove("trigger");
    variables.remove("k");
    variables.remove("f");
    variables.remove("value");

    variables
}
