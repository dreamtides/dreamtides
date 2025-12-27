use ability_data::ability::Ability;
use ability_data::predicate::Predicate;
use ability_data::standard_effect::StandardEffect;
use ability_data::trigger_event::TriggerEvent;
use chumsky::prelude::*;
use parser_v2::builder::parser_builder;
use parser_v2::builder::parser_spans::SpannedAbility;
use parser_v2::lexer::lexer_tokenize;
use parser_v2::parser::{ability_parser, effect_parser, predicate_parser, trigger_parser};
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

pub fn parse_trigger(input: &str, vars: &str) -> TriggerEvent {
    let lex_result = lexer_tokenize::lex(input).unwrap();
    let bindings = VariableBindings::parse(vars).unwrap();
    let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();
    let parser = trigger_parser::trigger_event_parser();
    parser.parse(&resolved).into_result().unwrap()
}

pub fn parse_effect(input: &str, vars: &str) -> StandardEffect {
    let lex_result = lexer_tokenize::lex(input).unwrap();
    let bindings = VariableBindings::parse(vars).unwrap();
    let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();
    let parser = effect_parser::single_effect_parser();
    parser.parse(&resolved).into_result().unwrap()
}

pub fn parse_predicate(input: &str, vars: &str) -> Predicate {
    let lex_result = lexer_tokenize::lex(input).unwrap();
    let bindings = VariableBindings::parse(vars).unwrap();
    let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();
    let parser = predicate_parser::predicate_parser();
    parser.parse(&resolved).into_result().unwrap()
}
