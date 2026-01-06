use chumsky::Parser;
use parser_v2::lexer::lexer_tokenize;
use parser_v2::parser::ability_parser;
use parser_v2::variables::parser_bindings::VariableBindings;
use parser_v2::variables::parser_substitutions;
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

struct ParseError {
    card_name: String,
    ability_text: String,
    variables: String,
    error: String,
}

struct ResolvedAbility {
    card_name: String,
    ability_text: String,
    variables: String,
    resolved_tokens: Vec<(parser_substitutions::ResolvedToken, chumsky::span::SimpleSpan)>,
}

#[test]
fn test_all_cards_toml_parse() {
    let cards_toml =
        std::fs::read_to_string("../../tabula/cards.toml").expect("Failed to read cards.toml");
    let cards_file: CardsFile = toml::from_str(&cards_toml).expect("Failed to parse cards.toml");

    let mut resolved_abilities = Vec::new();
    let mut resolution_errors = Vec::new();

    for card in &cards_file.cards {
        let Some(rules_text) = &card.rules_text else {
            continue;
        };

        let variables = card.variables.as_deref().unwrap_or("");

        for ability_block in rules_text.split("\n\n") {
            let ability_block = ability_block.trim();
            if ability_block.is_empty() {
                continue;
            }

            match resolve_ability(&card.name, ability_block, variables) {
                Ok(resolved) => resolved_abilities.push(resolved),
                Err(error) => resolution_errors.push(error),
            }
        }
    }

    let parser = ability_parser::ability_parser();

    let mut parse_errors = Vec::new();
    let mut success_count = 0;

    for resolved in &resolved_abilities {
        match parser.parse(&resolved.resolved_tokens).into_result() {
            Ok(_) => success_count += 1,
            Err(errors) => {
                let error_msg = if errors.is_empty() {
                    "Unknown parse error".to_string()
                } else {
                    format!("Parse error: {:?}", errors[0])
                };
                parse_errors.push(ParseError {
                    card_name: resolved.card_name.clone(),
                    ability_text: resolved.ability_text.clone(),
                    variables: resolved.variables.clone(),
                    error: error_msg,
                });
            }
        }
    }

    let all_errors: Vec<ParseError> =
        resolution_errors.into_iter().chain(parse_errors.into_iter()).collect();

    let total_abilities = resolved_abilities.len() + all_errors.len();
    print_results(success_count, total_abilities, &all_errors);

    if !all_errors.is_empty() {
        panic!("\n{} abilities failed to parse (see details above)", all_errors.len());
    }
}

fn resolve_ability(
    card_name: &str,
    ability_text: &str,
    variables: &str,
) -> Result<ResolvedAbility, ParseError> {
    let bindings = VariableBindings::parse(variables).map_err(|e| ParseError {
        card_name: card_name.to_string(),
        ability_text: ability_text.to_string(),
        variables: variables.to_string(),
        error: format!("Variable binding error: {:?}", e),
    })?;

    let lex_result = lexer_tokenize::lex(ability_text).map_err(|e| ParseError {
        card_name: card_name.to_string(),
        ability_text: ability_text.to_string(),
        variables: variables.to_string(),
        error: format!("Lexer error: {:?}", e),
    })?;

    let resolved_tokens = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings)
        .map_err(|e| ParseError {
            card_name: card_name.to_string(),
            ability_text: ability_text.to_string(),
            variables: variables.to_string(),
            error: format!("Variable resolution error: {}", e),
        })?;

    Ok(ResolvedAbility {
        card_name: card_name.to_string(),
        ability_text: ability_text.to_string(),
        variables: variables.to_string(),
        resolved_tokens,
    })
}

fn print_results(success_count: usize, total_abilities: usize, errors: &[ParseError]) {
    println!("\n========================================");
    println!("Card TOML Parsing Validation Results");
    println!("========================================\n");
    println!("Total abilities: {}", total_abilities);
    println!("Successfully parsed: {}", success_count);
    println!("Failed to parse: {}\n", errors.len());

    if errors.is_empty() {
        println!("✓ All abilities parsed successfully!");
    } else {
        println!("Parse Failures:\n");
        println!("{}", "=".repeat(80));

        for (i, error) in errors.iter().enumerate() {
            println!("\nFailure #{}", i + 1);
            println!("{}", "-".repeat(80));
            println!("Card: {}", error.card_name);
            println!("\nAbility Text:");
            println!("  {}", error.ability_text.replace('\n', "\n  "));

            if !error.variables.is_empty() {
                println!("\nVariables:");
                println!("  {}", error.variables.replace('\n', "\n  "));
            }

            println!("\nError:");
            println!("  {}", error.error);
            println!("{}", "-".repeat(80));
        }
    }
    println!("\n========================================\n");
}
