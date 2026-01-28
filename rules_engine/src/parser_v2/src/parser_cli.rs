use std::fs;
use std::path::{Path, PathBuf};

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::error::Rich;
use chumsky::span::{SimpleSpan, Span};
use chumsky::Parser as ChumskyParser;
use clap::{Parser, Subcommand, ValueEnum};
use parser_v2::ability_directory_parser;
use parser_v2::error::parser_errors::ParserError;
use parser_v2::error::{parser_diagnostics, parser_error_suggestions};
use parser_v2::lexer::lexer_token::Token;
use parser_v2::lexer::lexer_tokenize;
use parser_v2::parser::ability_parser;
use parser_v2::variables::parser_bindings::VariableBindings;
use parser_v2::variables::parser_substitutions::{self, ResolvedToken};
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

  # Parse abilities from all TOML files in a directory
  parser_v2 parse-abilities --directory tabula --output parsed_abilities.json

  # Verify a parsed abilities JSON file matches the TOML source
  parser_v2 verify-abilities --directory tabula --input parsed_abilities.json

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

    ParseAbilities {
        #[arg(short, long)]
        directory: PathBuf,

        #[arg(short, long, default_value = "parsed_abilities.json")]
        output: PathBuf,
    },

    VerifyAbilities {
        #[arg(short, long)]
        directory: PathBuf,

        #[arg(short, long)]
        input: PathBuf,
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
        eprintln!("{e}");
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
        Command::ParseAbilities { directory, output } => {
            parse_abilities_command(&directory, &output)?;
        }
        Command::VerifyAbilities { directory, input } => {
            verify_abilities_command(&directory, &input)?;
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
        VariableBindings::parse(vars_str).map_err(ParserError::from)?
    } else {
        VariableBindings::new()
    };

    match stage {
        Stage::Lex => {
            let lex_result = lexer_tokenize::lex(text).map_err(|e| {
                let error = ParserError::from(e);
                parser_diagnostics::format_error(&error, text, "<input>")
            })?;
            let tokens: Vec<&Token> = lex_result.tokens.iter().map(|(t, _)| t).collect();
            output_format(&tokens, format);
        }
        Stage::ResolveVariables => {
            let lex_result = lexer_tokenize::lex(text).map_err(|e| {
                let error = ParserError::from(e);
                parser_diagnostics::format_error(&error, text, "<input>")
            })?;
            let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings)
                .map_err(|e| {
                let error = ParserError::from(e);
                parser_diagnostics::format_error(&error, text, "<input>")
            })?;
            let resolved_tokens: Vec<&ResolvedToken> = resolved.iter().map(|(t, _)| t).collect();
            output_format(&resolved_tokens, format);
        }
        Stage::Full => {
            let lex_result = lexer_tokenize::lex(text).map_err(|e| {
                let error = ParserError::from(e);
                parser_diagnostics::format_error(&error, text, "<input>")
            })?;
            let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings)
                .map_err(|e| {
                let error = ParserError::from(e);
                parser_diagnostics::format_error(&error, text, "<input>")
            })?;

            let parser = ability_parser::ability_parser();
            let parsed = parser
                .parse(&resolved)
                .into_result()
                .map_err(|errors| format_parse_errors(errors, text))?;

            output_format(&parsed, format);
        }
    }

    Ok(())
}

fn parse_abilities_command(
    directory: &Path,
    output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = ability_directory_parser::parse_abilities_from_directory(directory)?;
    let output_content = serde_json::to_string(&results)?;
    fs::write(output, output_content)?;
    println!("Parsed abilities for {} cards to {}", results.len(), output.display());
    Ok(())
}

fn verify_abilities_command(
    directory: &Path,
    input: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let expected = ability_directory_parser::parse_abilities_from_directory(directory)?;
    let expected_json = serde_json::to_string(&expected)?;

    let actual_json = fs::read_to_string(input)?;

    if expected_json == actual_json {
        println!("✓ {} is up to date ({} cards)", input.display(), expected.len());
        Ok(())
    } else {
        let expected_parsed: serde_json::Value = serde_json::from_str(&expected_json)?;
        let actual_parsed: serde_json::Value = serde_json::from_str(&actual_json)?;

        if expected_parsed == actual_parsed {
            println!(
                "✓ {} matches but has different formatting ({} cards)",
                input.display(),
                expected.len()
            );
            Ok(())
        } else {
            eprintln!(
                "✗ {} does not match parsed abilities from {}",
                input.display(),
                directory.display()
            );
            Err("Verification failed: JSON file does not match parsed abilities".into())
        }
    }
}

fn parse_file_command(input: &PathBuf, output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(input)?;
    let cards_file: CardsFile = toml::from_str(&content)?;

    let mut results = Vec::new();

    for card in cards_file.cards {
        let Some(rules_text) = &card.rules_text else { continue };

        let bindings = if let Some(vars) = &card.variables {
            VariableBindings::parse(vars)?
        } else {
            VariableBindings::new()
        };

        let lex_result = lexer_tokenize::lex(rules_text)?;
        let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings)?;

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
        let Some(rules_text) = &card.rules_text else {
            skipped_count += 1;
            continue;
        };

        let bindings = if let Some(vars) = &card.variables {
            match VariableBindings::parse(vars) {
                Ok(b) => b,
                Err(e) => {
                    let error = ParserError::from(e);
                    let formatted =
                        parser_diagnostics::format_error(&error, rules_text, &card.name);
                    eprintln!("\n{formatted}");
                    error_count += 1;
                    continue;
                }
            }
        } else {
            VariableBindings::new()
        };

        match lexer_tokenize::lex(rules_text) {
            Ok(lex_result) => {
                match parser_substitutions::resolve_variables(&lex_result.tokens, &bindings) {
                    Ok(_) => {
                        println!("✓ {}", card.name);
                        success_count += 1;
                    }
                    Err(e) => {
                        let error = ParserError::from(e);
                        let formatted =
                            parser_diagnostics::format_error(&error, rules_text, &card.name);
                        eprintln!("\n{formatted}");
                        error_count += 1;
                    }
                }
            }
            Err(e) => {
                let error = ParserError::from(e);
                let formatted = parser_diagnostics::format_error(&error, rules_text, &card.name);
                eprintln!("\n{formatted}");
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

fn format_parse_errors<'a>(
    errors: Vec<Rich<'a, (ResolvedToken, SimpleSpan)>>,
    text: &str,
) -> String {
    let error_strs: Vec<String> = errors
        .into_iter()
        .map(|e| {
            let span = *e.span();
            let mut output = Vec::new();

            let found_token = e.found().map(|(token, _)| token);
            let (message, label_message, note) = match found_token {
                Some(ResolvedToken::Token(Token::Word(word))) => {
                    if let Some(suggestions) = parser_error_suggestions::suggest_word(word) {
                        (
                            format!("Unexpected word '{word}'"),
                            format!(
                                "Found '{word}' here. Did you mean '{}'?",
                                suggestions.join("', '")
                            ),
                            Some(format!("Similar words: {}", suggestions.join(", "))),
                        )
                    } else {
                        (format!("Unexpected word '{word}'"), format!("Found '{word}' here"), None)
                    }
                }
                Some(ResolvedToken::Token(Token::Directive(name))) => {
                    if let Some(suggestions) = parser_error_suggestions::suggest_directive(name) {
                        (
                            format!("Unexpected directive '{{{name}}}'"),
                            format!(
                                "Found '{{{name}}}' here. Did you mean '{{{}}}'?",
                                suggestions.join("', '{")
                            ),
                            Some(format!("Similar directives: {}", suggestions.join(", "))),
                        )
                    } else {
                        (
                            format!("Unexpected directive '{{{name}}}'"),
                            format!("Found '{{{name}}}' here"),
                            None,
                        )
                    }
                }
                _ => {
                    let message = format!("Parse error: {:?}", e.reason());
                    (message, "Failed to parse ability text here".to_string(), None)
                }
            };

            let mut report = Report::<(&str, std::ops::Range<usize>)>::build(
                ReportKind::Error,
                "<input>",
                span.start(),
            )
            .with_message(&message);

            let label = Label::new(("<input>", span.start()..span.end()))
                .with_message(&label_message)
                .with_color(Color::Red);

            report = report.with_label(label);

            if let Some(note_text) = note {
                report = report.with_note(note_text);
            }

            report.finish().write(("<input>", Source::from(text)), &mut output).unwrap();
            String::from_utf8(output).unwrap()
        })
        .collect();

    error_strs.join("\n\n")
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
