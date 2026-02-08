use ability_data::ability::Ability;
use chumsky::prelude::*;
use chumsky::span::{SimpleSpan, Span};
use parser_v2::builder::parser_builder;
use parser_v2::builder::parser_spans::SpannedAbility;
use parser_v2::lexer::lexer_tokenize;
use parser_v2::parser::ability_parser;
use parser_v2::serializer::ability_serializer;
use parser_v2::variables::parser_bindings::VariableBindings;
use parser_v2::variables::parser_substitutions;

/// Stack red zone size - if less than this remains, grow the stack.
/// The parser hierarchy is deep and needs significant stack during
/// construction. We use a very large red zone to ensure we catch the overflow
/// early, before the thread's stack is exhausted.
const STACK_RED_ZONE: usize = 1024 * 1024; // 1 MB

/// Stack size to grow by when needed.
const STACK_SIZE: usize = 4 * 1024 * 1024; // 4 MB

/// Wrapper that ensures sufficient stack space for parser operations.
/// The deep Chumsky parser hierarchy requires significant stack during
/// construction, which can cause stack overflow in low-memory environments
/// (like Docker) when tests run in parallel.
fn with_stack<T>(f: impl FnOnce() -> T) -> T {
    stacker::maybe_grow(STACK_RED_ZONE, STACK_SIZE, f)
}

pub fn parse_ability(input: &str, vars: &str) -> Ability {
    with_stack(|| {
        let lex_result = lexer_tokenize::lex(input).unwrap();
        let bindings = VariableBindings::parse(vars).unwrap();
        let resolved =
            parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();
        let parser = ability_parser::ability_parser();
        parser.parse(&resolved).into_result().unwrap()
    })
}

pub fn parse_abilities(input: &str, vars: &str) -> Vec<Ability> {
    with_stack(|| {
        let bindings = VariableBindings::parse(vars).unwrap();
        let mut abilities = Vec::new();
        for block in input.split("\n\n") {
            let block = block.trim();
            if block.is_empty() {
                continue;
            }
            let lex_result = lexer_tokenize::lex(block).unwrap();
            let resolved =
                parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();
            let parser = ability_parser::ability_parser();
            abilities.push(parser.parse(&resolved).into_result().unwrap());
        }
        abilities
    })
}

pub fn parse_spanned_ability(input: &str, vars: &str) -> SpannedAbility {
    with_stack(|| {
        let lex_result = lexer_tokenize::lex(input).unwrap();
        let bindings = VariableBindings::parse(vars).unwrap();
        let resolved =
            parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();
        let parser = ability_parser::ability_parser();
        let ability = parser.parse(&resolved).into_result().unwrap();
        parser_builder::build_spanned_ability(&ability, &lex_result).unwrap()
    })
}

pub fn parse_spanned_abilities(input: &str, vars: &str) -> Vec<SpannedAbility> {
    with_stack(|| {
        let bindings = VariableBindings::parse(vars).unwrap();
        let mut abilities = Vec::new();
        for block in input.split("\n\n") {
            let block = block.trim();
            if block.is_empty() {
                continue;
            }
            let lex_result = lexer_tokenize::lex(block).unwrap();
            let resolved =
                parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();
            let parser = ability_parser::ability_parser();
            let ability = parser.parse(&resolved).into_result().unwrap();
            abilities.push(parser_builder::build_spanned_ability(&ability, &lex_result).unwrap());
        }
        abilities
    })
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

/// Asserts that an ability round-trips correctly, verifying text equality,
/// variable bindings, and AST structural equality.
pub fn assert_round_trip(expected_text: &str, vars: &str) {
    let parsed = parse_ability(expected_text, vars);
    let serialized = ability_serializer::serialize_ability(&parsed);
    assert_eq!(expected_text, serialized.text);
    assert_eq!(VariableBindings::parse(vars).unwrap(), serialized.variables);
    let reparsed = parse_ability(&serialized.text, vars);
    assert_eq!(
        parsed, reparsed,
        "AST mismatch: parse(input) != parse(serialize(parse(input)))\n  input: {expected_text:?}"
    );
}
