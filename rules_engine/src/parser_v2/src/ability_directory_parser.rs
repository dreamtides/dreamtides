use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use ability_data::ability::Ability;
use chumsky::span::SimpleSpan;
use chumsky::Parser as ChumskyParser;
use toml::Value;

use crate::error::parser_diagnostics;
use crate::error::parser_errors::ParserError;
use crate::lexer::lexer_tokenize;
use crate::parser::ability_parser;
use crate::variables::parser_bindings::VariableBindings;
use crate::variables::parser_substitutions::{self, ResolvedToken};

/// Parses all TOML files in a directory and extracts abilities from entries
/// containing `id`, `rules-text`, and `variables` fields.
///
/// Returns a map of card IDs to their parsed abilities.
pub fn parse_abilities_from_directory(
    directory: &Path,
) -> Result<BTreeMap<String, Vec<Ability>>, Box<dyn std::error::Error>> {
    let mut results: BTreeMap<String, Vec<Ability>> = BTreeMap::new();
    let mut file_errors = Vec::new();

    let mut entries_to_parse: Vec<EntryToParse> = Vec::new();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "toml") {
            match collect_entries_from_file(&path) {
                Ok(file_entries) => entries_to_parse.extend(file_entries),
                Err(e) => file_errors.push(format!("{}: {e}", path.display())),
            }
        }
    }

    if !file_errors.is_empty() {
        eprintln!("Warnings while reading files:\n{}", file_errors.join("\n"));
    }

    let mut resolved_entries: Vec<ResolvedEntry> = Vec::new();

    for entry in entries_to_parse {
        let bindings = if entry.variables.is_empty() {
            VariableBindings::new()
        } else {
            match VariableBindings::parse(&entry.variables) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!(
                        "Error parsing variables for '{}' ({}): {}",
                        entry.name,
                        entry.id,
                        parser_diagnostics::format_error(
                            &ParserError::from(e),
                            &entry.rules_text,
                            &entry.name
                        )
                    );
                    continue;
                }
            }
        };

        let ability_texts: Vec<&str> =
            entry.rules_text.split("\n\n").map(str::trim).filter(|s| !s.is_empty()).collect();

        let mut ability_resolved_tokens = Vec::new();
        let mut had_error = false;

        for ability_text in ability_texts {
            let lex_result = match lexer_tokenize::lex(ability_text) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!(
                        "Error lexing '{}' ({}): {}",
                        entry.name,
                        entry.id,
                        parser_diagnostics::format_error(
                            &ParserError::from(e),
                            ability_text,
                            &entry.name
                        )
                    );
                    had_error = true;
                    break;
                }
            };

            let resolved =
                match parser_substitutions::resolve_variables(&lex_result.tokens, &bindings) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!(
                            "Error resolving variables for '{}' ({}): {}",
                            entry.name,
                            entry.id,
                            parser_diagnostics::format_error(
                                &ParserError::from(e),
                                ability_text,
                                &entry.name
                            )
                        );
                        had_error = true;
                        break;
                    }
                };

            ability_resolved_tokens.push((ability_text.to_string(), resolved));
        }

        if !had_error {
            resolved_entries.push(ResolvedEntry {
                id: entry.id,
                name: entry.name,
                resolved_abilities: ability_resolved_tokens,
            });
        }
    }

    let parser = ability_parser::ability_parser();

    for entry in &resolved_entries {
        let mut abilities = Vec::new();
        let mut had_error = false;

        for (ability_text, resolved) in &entry.resolved_abilities {
            match parser.parse(resolved).into_result() {
                Ok(ability) => {
                    abilities.push(ability);
                }
                Err(errors) => {
                    let error_strs: Vec<String> =
                        errors.iter().map(|e| format!("{:?}", e.reason())).collect();
                    eprintln!(
                        "Parse error in '{}' ({}): {} (text: \"{}\")",
                        entry.name,
                        entry.id,
                        error_strs.join(", "),
                        ability_text
                    );
                    had_error = true;
                    break;
                }
            }
        }

        if !had_error && !abilities.is_empty() {
            results.entry(entry.id.clone()).or_default().extend(abilities);
        }
    }

    Ok(results)
}

struct EntryToParse {
    id: String,
    name: String,
    rules_text: String,
    variables: String,
}

struct ResolvedEntry {
    id: String,
    name: String,
    resolved_abilities: Vec<(String, Vec<(ResolvedToken, SimpleSpan)>)>,
}

fn collect_entries_from_file(path: &Path) -> Result<Vec<EntryToParse>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let value: Value = toml::from_str(&content)?;

    let mut entries = Vec::new();

    let Some(table) = value.as_table() else {
        return Ok(entries);
    };

    for (_table_name, table_value) in table {
        let Some(array) = table_value.as_array() else {
            continue;
        };

        for entry in array {
            let Some(entry_table) = entry.as_table() else {
                continue;
            };

            let Some(id) = entry_table.get("id").and_then(|v| v.as_str()) else {
                continue;
            };

            let Some(rules_text) = entry_table.get("rules-text").and_then(|v| v.as_str()) else {
                continue;
            };

            if rules_text.trim().is_empty() {
                continue;
            }

            let variables = entry_table.get("variables").and_then(|v| v.as_str()).unwrap_or("");
            let name = entry_table.get("name").and_then(|v| v.as_str()).unwrap_or("<unknown>");

            entries.push(EntryToParse {
                id: id.to_string(),
                name: name.to_string(),
                rules_text: rules_text.to_string(),
                variables: variables.to_string(),
            });
        }
    }

    Ok(entries)
}
