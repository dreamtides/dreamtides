use std::path::Path;
use std::{fs, hint};

use chumsky::Parser;
use parser_v2::lexer::lexer_tokenize;
use parser_v2::parser::ability_parser;
use parser_v2::variables::parser_bindings::VariableBindings;
use parser_v2::variables::parser_substitutions;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct BenchmarkCardsFile {
    pub cards: Vec<BenchmarkCard>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BenchmarkCard {
    #[serde(rename = "rules-text")]
    pub rules_text: Option<String>,
    pub variables: Option<String>,
}

pub fn parse_single_card(text: &str, vars: &str) {
    let bindings = VariableBindings::parse(vars).expect("Failed to parse variables");
    let lex_result = lexer_tokenize::lex(text).expect("Failed to lex text");
    let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings)
        .expect("Failed to resolve variables");
    let parser = ability_parser::ability_parser();
    parser.parse(&resolved).into_result().expect("Failed to parse ability");
}

pub fn load_cards_toml() -> BenchmarkCardsFile {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let cards_path = Path::new(manifest_dir).join("../../tabula/cards.toml");
    let content = fs::read_to_string(cards_path.clone())
        .unwrap_or_else(|_| panic!("Failed to read cards.toml at path {}", cards_path.display()));
    toml::from_str(&content).expect("Failed to parse cards.toml as toml")
}

pub fn parse_all_cards(cards_file: BenchmarkCardsFile) {
    let resolved_cards = cards_file
        .cards
        .into_iter()
        .filter_map(|card| {
            let Some(rules_text) = &card.rules_text else { return None };

            let bindings = if let Some(vars) = &card.variables {
                VariableBindings::parse(vars).expect("Failed to parse variables")
            } else {
                VariableBindings::new()
            };

            let lex_result = lexer_tokenize::lex(rules_text).expect("Failed to lex rules text");
            Some(
                parser_substitutions::resolve_variables(&lex_result.tokens, &bindings)
                    .expect("Failed to resolve variables"),
            )
        })
        .collect::<Vec<_>>();

    let parser = ability_parser::ability_parser();

    for resolved in &resolved_cards {
        let _ = hint::black_box(parser.parse(resolved)).into_result();
    }
}
