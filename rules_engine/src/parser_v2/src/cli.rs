use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use parser_v2::lexer::token::Token;
use parser_v2::lexer::tokenize;
use parser_v2::variables::binding::VariableBindings;
use parser_v2::variables::substitution::{self, ResolvedToken};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(
    name = "parser_v2",
    about = "Dreamtides card ability parser",
    after_long_help = "EXAMPLES:
  # Lex a simple draw effect
  parser_v2 parse \"Draw {cards}.\" --vars \"cards: 2\" --stage lex

  # Resolve variables for a judgment trigger
  parser_v2 parse \"{Judgment} Gain {e}.\" --vars \"e: 3\" --stage resolve-variables --format json

  # Parse compound directive (figments)
  parser_v2 parse \"{Materialize} {n-figments}.\" --vars \"number: 3, figment: radiant\" --stage resolve-variables

  # Parse ability with subtype variable
  parser_v2 parse \"Allied {plural-subtype} have +{s} spark.\" --vars \"subtype: Warrior, s: 2\" --stage resolve-variables --format ron

  # Parse multi-variable ability with debug output
  parser_v2 parse \"{Judgment} You may discard {discards} to draw {cards} and gain {points}.\" \\
    --vars \"discards: 1, cards: 1, points: 1\" --stage resolve-variables --format debug

  # Parse all cards from TOML file
  parser_v2 parse-file --input tabula/cards.toml --output parsed_cards.json

  # Verify all cards can be lexed and variables resolved
  parser_v2 verify tabula/cards.toml

  # Parse event with reclaim ability
  parser_v2 parse \"Draw {cards}. Discard {discards}.\\n\\n{ReclaimForCost}\" \\
    --vars \"cards: 2, discards: 2, reclaim: 2\" --stage lex --format json

  # Parse triggered ability with cost
  parser_v2 parse \"Abandon {count-allies}: {Reclaim} this character.\" \\
    --vars \"allies: 3\" --stage resolve-variables

  # Parse static ability
  parser_v2 parse \"Once per turn, you may play a character with cost {e} or less from your void.\" \\
    --vars \"e: 2\" --stage lex"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Parse {
        text: String,

        #[arg(short, long)]
        vars: Option<String>,

        #[arg(short, long, default_value = "ron")]
        format: OutputFormat,

        #[arg(short, long, default_value = "full")]
        stage: Stage,
    },

    ParseFile {
        #[arg(short, long)]
        input: PathBuf,

        #[arg(short, long)]
        output: PathBuf,
    },

    Verify {
        input: PathBuf,
    },
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Json,
    Ron,
    Debug,
}

#[derive(Clone, ValueEnum)]
enum Stage {
    Lex,
    ResolveVariables,
    Full,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Parse { text, vars, format, stage } => {
            parse_command(&text, vars.as_deref(), format, stage)?;
        }
        Command::ParseFile { input, output } => {
            parse_file_command(&input, &output)?;
        }
        Command::Verify { input } => {
            verify_command(&input)?;
        }
    }

    Ok(())
}

fn parse_command(
    text: &str,
    vars: Option<&str>,
    format: OutputFormat,
    stage: Stage,
) -> Result<(), Box<dyn std::error::Error>> {
    let bindings = if let Some(vars_str) = vars {
        VariableBindings::parse(vars_str)?
    } else {
        VariableBindings::new()
    };

    match stage {
        Stage::Lex => {
            let lex_result = tokenize::lex(text)?;
            let tokens: Vec<&Token> = lex_result.tokens.iter().map(|(t, _)| t).collect();
            output_format(&tokens, format);
        }
        Stage::ResolveVariables => {
            let lex_result = tokenize::lex(text)?;
            let resolved = substitution::resolve_variables(&lex_result.tokens, &bindings)?;
            let resolved_tokens: Vec<&ResolvedToken> = resolved.iter().map(|(t, _)| t).collect();
            output_format(&resolved_tokens, format);
        }
        Stage::Full => {
            return Err("Full parsing not yet implemented".into());
        }
    }

    Ok(())
}

fn parse_file_command(input: &PathBuf, output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(input)?;
    let cards_file: CardsFile = toml::from_str(&content)?;

    let mut results = Vec::new();

    for card in cards_file.cards {
        let rules_text = match &card.rules_text {
            Some(text) => text,
            None => continue,
        };

        let bindings = if let Some(vars) = &card.variables {
            VariableBindings::parse(vars)?
        } else {
            VariableBindings::new()
        };

        let lex_result = tokenize::lex(rules_text)?;
        let resolved = substitution::resolve_variables(&lex_result.tokens, &bindings)?;

        results.push(CardParseResult {
            name: card.name,
            rules_text: card.rules_text.unwrap(),
            variables: card.variables,
            tokens: resolved.iter().map(|(t, _)| t.clone()).collect(),
        });
    }

    let output_content = serde_json::to_string_pretty(&results)?;
    fs::write(output, output_content)?;

    println!("Parsed {} cards to {}", results.len(), output.display());

    Ok(())
}

fn verify_command(input: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(input)?;
    let cards_file: CardsFile = toml::from_str(&content)?;

    let mut success_count = 0;
    let mut error_count = 0;
    let mut skipped_count = 0;

    for card in cards_file.cards {
        let rules_text = match &card.rules_text {
            Some(text) => text,
            None => {
                skipped_count += 1;
                continue;
            }
        };

        let bindings = if let Some(vars) = &card.variables {
            match VariableBindings::parse(vars) {
                Ok(b) => b,
                Err(e) => {
                    println!("❌ {}: Variable parse error: {e}", card.name);
                    error_count += 1;
                    continue;
                }
            }
        } else {
            VariableBindings::new()
        };

        match tokenize::lex(rules_text) {
            Ok(lex_result) => {
                match substitution::resolve_variables(&lex_result.tokens, &bindings) {
                    Ok(_) => {
                        println!("✓ {}", card.name);
                        success_count += 1;
                    }
                    Err(e) => {
                        println!("❌ {}: Variable resolution error: {e}", card.name);
                        error_count += 1;
                    }
                }
            }
            Err(e) => {
                println!("❌ {}: Lex error: {e}", card.name);
                error_count += 1;
            }
        }
    }

    println!("\nResults: {success_count} succeeded, {error_count} failed, {skipped_count} skipped");

    if error_count > 0 {
        Err("Verification failed".into())
    } else {
        Ok(())
    }
}

fn output_format<T: serde::Serialize + std::fmt::Debug>(value: &T, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(value).unwrap());
        }
        OutputFormat::Ron => {
            println!(
                "{}",
                ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::default()).unwrap()
            );
        }
        OutputFormat::Debug => {
            println!("{value:#?}");
        }
    }
}

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

#[derive(Debug, Serialize)]
struct CardParseResult {
    name: String,
    rules_text: String,
    variables: Option<String>,
    tokens: Vec<ResolvedToken>,
}
