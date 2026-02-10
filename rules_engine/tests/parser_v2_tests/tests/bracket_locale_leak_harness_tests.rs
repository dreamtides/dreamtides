use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};

use chumsky::Parser;
use parser_v2::lexer::lexer_tokenize;
use parser_v2::parser::ability_parser;
use parser_v2::serializer::ability_serializer;
use parser_v2::variables::parser_bindings::VariableBindings;
use parser_v2::variables::parser_substitutions;
use parser_v2_tests::test_helpers;
use serde::Deserialize;

const CARDS_TOML_PATH: &str = "../../tabula/cards.toml";
const TEST_CARDS_TOML_PATH: &str = "../../tabula/test-cards.toml";
const BRACKET_LANGUAGE: &str = "bracket";
const MAX_ALLOWED_BRACKET_RENDER_ERRORS: usize = 17;
const MAX_ALLOWED_BRACKET_LEAKS: usize = 0;
const MAX_REPORTED_ISSUES: usize = 40;

#[derive(Debug, Deserialize)]
struct CardsFile {
    cards: Vec<Card>,
}

#[derive(Debug, Deserialize)]
struct TestCardsFile {
    #[serde(rename = "test-cards")]
    test_cards: Vec<Card>,
}

#[derive(Debug, Deserialize)]
struct Card {
    name: String,
    #[serde(rename = "rules-text")]
    rules_text: Option<String>,
    variables: Option<String>,
}

#[derive(Debug)]
struct RenderError {
    source: &'static str,
    card_name: String,
    ability_index: usize,
    error_detail: String,
}

#[derive(Debug)]
struct BracketLeak {
    source: &'static str,
    card_name: String,
    ability_index: usize,
    token: String,
    position: usize,
    rendered_text: String,
}

#[derive(Debug)]
struct TokenLeak {
    token: String,
    position: usize,
}

struct LanguageGuard {
    previous_language: String,
}

impl Drop for LanguageGuard {
    fn drop(&mut self) {
        rlf::with_locale_mut(|locale| locale.set_language(&self.previous_language));
    }
}

fn activate_bracket_locale() -> LanguageGuard {
    let previous_language = rlf::with_locale(|locale| locale.language().to_string());
    rlf::with_locale_mut(|locale| locale.set_language(BRACKET_LANGUAGE));
    LanguageGuard { previous_language }
}

fn render_ability(ability_text: &str, variables: &str) -> Result<String, String> {
    let bindings =
        VariableBindings::parse(variables).map_err(|e| format!("variable parse error: {e:?}"))?;
    let lex_result =
        lexer_tokenize::lex(ability_text).map_err(|e| format!("lexer error: {e:?}"))?;
    let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings)
        .map_err(|e| format!("variable resolution error: {e}"))?;
    let ability = {
        let parser = ability_parser::ability_parser();
        parser.parse(&resolved).into_result().map_err(|e| format!("parser error: {e:?}"))?
    };
    catch_unwind(AssertUnwindSafe(|| ability_serializer::serialize_ability(&ability).text))
        .map_err(|panic| format!("serializer panic: {}", panic_message(panic)))
}

fn panic_message(panic: Box<dyn Any + Send>) -> String {
    if let Some(message) = panic.downcast_ref::<&'static str>() {
        (*message).to_string()
    } else if let Some(message) = panic.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

fn is_numeric_or_operator_token(token: &str) -> bool {
    !token.is_empty()
        && token.chars().all(|c| {
            c.is_ascii_digit()
                || matches!(
                    c,
                    '+' | '-'
                        | '*'
                        | '/'
                        | '='
                        | '<'
                        | '>'
                        | '≤'
                        | '≥'
                        | '('
                        | ')'
                        | '{'
                        | '}'
                        | '['
                        | ']'
                        | ':'
                        | ';'
                        | ','
                        | '.'
                        | '%'
                        | '&'
                        | '|'
                        | '^'
                        | '!'
                        | '?'
                        | '_'
                        | '~'
                        | '#'
                )
        })
}

fn find_unbracketed_text_leaks(rendered: &str) -> Vec<TokenLeak> {
    let mut leaks = Vec::new();
    let mut bracket_depth = 0usize;
    let mut index = 0usize;

    while index < rendered.len() {
        let mut chars = rendered[index..].chars();
        let Some(ch) = chars.next() else {
            break;
        };
        let ch_len = ch.len_utf8();

        if ch == '[' {
            bracket_depth += 1;
            index += ch_len;
            continue;
        }

        if ch == ']' {
            bracket_depth = bracket_depth.saturating_sub(1);
            index += ch_len;
            continue;
        }

        if bracket_depth > 0 || ch.is_whitespace() {
            index += ch_len;
            continue;
        }

        if ch == '<' {
            if let Some(close_offset) = rendered[index + ch_len..].find('>') {
                index += ch_len + close_offset + 1;
                continue;
            }
        }

        let start = index;
        let mut end = index;
        while end < rendered.len() {
            let mut token_chars = rendered[end..].chars();
            let Some(token_char) = token_chars.next() else {
                break;
            };
            if token_char.is_whitespace()
                || token_char == '['
                || token_char == ']'
                || token_char == '<'
            {
                break;
            }
            end += token_char.len_utf8();
        }

        let token = &rendered[start..end];
        if !token.is_empty()
            && !is_numeric_or_operator_token(token)
            && token.chars().any(char::is_alphabetic)
        {
            leaks.push(TokenLeak { token: token.to_string(), position: start });
        }

        index = end.max(index + ch_len);
    }

    leaks
}

fn collect_cards_leaks(
    cards: &[Card],
    source: &'static str,
    total_abilities: &mut usize,
    render_errors: &mut Vec<RenderError>,
    leaks: &mut Vec<BracketLeak>,
) {
    for card in cards {
        let Some(rules_text) = &card.rules_text else {
            continue;
        };

        let variables = card.variables.as_deref().unwrap_or("");
        for (ability_index, ability_block) in rules_text.split("\n\n").enumerate() {
            let ability_block = ability_block.trim();
            if ability_block.is_empty() {
                continue;
            }

            *total_abilities += 1;

            let rendered = match render_ability(ability_block, variables) {
                Ok(rendered) => rendered,
                Err(error_detail) => {
                    render_errors.push(RenderError {
                        source,
                        card_name: card.name.clone(),
                        ability_index,
                        error_detail,
                    });
                    continue;
                }
            };

            for token_leak in find_unbracketed_text_leaks(&rendered) {
                leaks.push(BracketLeak {
                    source,
                    card_name: card.name.clone(),
                    ability_index,
                    token: token_leak.token,
                    position: token_leak.position,
                    rendered_text: rendered.clone(),
                });
            }
        }
    }
}

fn format_render_error_output(render_errors: &[RenderError]) -> String {
    let mut output = Vec::new();
    for error in render_errors.iter().take(MAX_REPORTED_ISSUES) {
        output.push(format!(
            "- {} | {} | ability #{} | {}",
            error.source, error.card_name, error.ability_index, error.error_detail
        ));
    }
    if render_errors.len() > MAX_REPORTED_ISSUES {
        output.push(format!(
            "... {} more render errors omitted",
            render_errors.len() - MAX_REPORTED_ISSUES
        ));
    }
    output.join("\n")
}

fn format_leak_output(leaks: &[BracketLeak]) -> String {
    let mut output = Vec::new();
    for leak in leaks.iter().take(MAX_REPORTED_ISSUES) {
        output.push(format!(
            "- {} | {} | ability #{} | token {:?} at byte {} | rendered {:?}",
            leak.source,
            leak.card_name,
            leak.ability_index,
            leak.token,
            leak.position,
            leak.rendered_text
        ));
    }
    if leaks.len() > MAX_REPORTED_ISSUES {
        output.push(format!("... {} more leaks omitted", leaks.len() - MAX_REPORTED_ISSUES));
    }
    output.join("\n")
}

#[test]
fn test_full_card_bracket_locale_leak_detector() {
    test_helpers::register_bracket_test_locale().expect("bracket locale should load");
    let _language_guard = activate_bracket_locale();

    let cards_toml =
        std::fs::read_to_string(CARDS_TOML_PATH).expect("Failed to read cards.toml for leak test");
    let cards_file: CardsFile =
        toml::from_str(&cards_toml).expect("Failed to parse cards.toml for leak test");

    let test_cards_toml = std::fs::read_to_string(TEST_CARDS_TOML_PATH)
        .expect("Failed to read test-cards.toml for leak test");
    let test_cards_file: TestCardsFile =
        toml::from_str(&test_cards_toml).expect("Failed to parse test-cards.toml for leak test");

    let mut total_abilities = 0usize;
    let mut render_errors = Vec::new();
    let mut leaks = Vec::new();

    collect_cards_leaks(
        &cards_file.cards,
        "cards.toml",
        &mut total_abilities,
        &mut render_errors,
        &mut leaks,
    );
    collect_cards_leaks(
        &test_cards_file.test_cards,
        "test-cards.toml",
        &mut total_abilities,
        &mut render_errors,
        &mut leaks,
    );

    println!(
        "Bracket locale leak detector: checked {total_abilities} abilities, found {} render errors and {} unbracketed text leaks",
        render_errors.len(),
        leaks.len()
    );

    assert!(
        render_errors.len() <= MAX_ALLOWED_BRACKET_RENDER_ERRORS,
        "Bracket locale harness hit {} render errors (max allowed baseline: {MAX_ALLOWED_BRACKET_RENDER_ERRORS})\n{}",
        render_errors.len(),
        format_render_error_output(&render_errors)
    );

    assert!(
        leaks.len() <= MAX_ALLOWED_BRACKET_LEAKS,
        "Found {} unbracketed text leaks (max allowed baseline: {MAX_ALLOWED_BRACKET_LEAKS})\n{}",
        leaks.len(),
        format_leak_output(&leaks)
    );
}
