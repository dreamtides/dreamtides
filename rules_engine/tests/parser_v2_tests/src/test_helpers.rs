use ability_data::ability::Ability;
use chumsky::prelude::*;
use chumsky::span::{SimpleSpan, Span};
use parser_v2::builder::parser_builder;
use parser_v2::builder::parser_spans::SpannedAbility;
use parser_v2::lexer::lexer_tokenize;
use parser_v2::parser::ability_parser;
use parser_v2::variables::parser_bindings::VariableBindings;
use parser_v2::variables::parser_substitutions;

pub fn parse_ability(input: &str, vars: &str) -> Ability {
    let lex_result = lexer_tokenize::lex(input).unwrap();
    let bindings = VariableBindings::parse(vars).unwrap();
    let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();
    let parser = ability_parser::ability_parser();
    parser.parse(&resolved).into_result().unwrap()
}

pub fn parse_spanned_ability(input: &str, vars: &str) -> SpannedAbility {
    let lex_result = lexer_tokenize::lex(input).unwrap();
    let bindings = VariableBindings::parse(vars).unwrap();
    let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();
    let parser = ability_parser::ability_parser();
    let ability = parser.parse(&resolved).into_result().unwrap();
    parser_builder::build_spanned_ability(&ability, &lex_result).unwrap()
}

pub fn build_spanned_ability(ability: &Ability, input: &str) -> SpannedAbility {
    parser_builder::build_spanned_ability(ability, &lexer_tokenize::lex(input).unwrap()).unwrap()
}

/// Verifies that a span is valid (non-empty).
pub fn assert_valid_span(span: &SimpleSpan) {
    assert!(
        span.end() > span.start(),
        "Span should be non-empty: start={}, end={}",
        span.start(),
        span.end()
    );
}
