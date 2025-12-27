use std::fs;
use std::path::Path;

use chumsky::Parser;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use parser_v2::lexer::tokenize;
use parser_v2::parser::ability;
use parser_v2::variables::binding::VariableBindings;
use parser_v2::variables::substitution;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
struct BenchmarkCardsFile {
    cards: Vec<BenchmarkCard>,
}

#[derive(Debug, Deserialize, Clone)]
struct BenchmarkCard {
    #[serde(rename = "rules-text")]
    rules_text: Option<String>,
    variables: Option<String>,
}

fn pipeline_benchmarks(c: &mut Criterion) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let cards_path = Path::new(manifest_dir).join("../../tabula/cards.toml");
    let content = fs::read_to_string(cards_path.clone())
        .unwrap_or_else(|_| panic!("Failed to read cards.toml at path {}", cards_path.display()));
    let cards_file: BenchmarkCardsFile =
        toml::from_str(&content).expect("Failed to parse cards.toml as toml");

    c.bench_function("parse_all_cards_toml", |b| {
        b.iter_batched(
            || cards_file.clone(),
            |cards_file| {
                for card in cards_file.cards {
                    let rules_text = match &card.rules_text {
                        Some(text) => text,
                        None => continue,
                    };

                    let bindings = if let Some(vars) = &card.variables {
                        VariableBindings::parse(vars).expect("Failed to parse variables")
                    } else {
                        VariableBindings::new()
                    };

                    let lex_result = tokenize::lex(rules_text).expect("Failed to lex rules text");
                    let resolved = substitution::resolve_variables(&lex_result.tokens, &bindings)
                        .expect("Failed to resolve variables");
                    let parser = ability::ability_parser();
                    // Ignore parse errors, we're only interested in the time it
                    // takes to parse the ability.
                    let _ = parser.parse(&resolved).into_result();
                }
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, pipeline_benchmarks);
criterion_main!(benches);
