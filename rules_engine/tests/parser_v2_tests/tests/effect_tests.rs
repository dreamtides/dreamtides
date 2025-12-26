use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use core_data::numerics::Energy;
use parser_v2::parser::effect_parser;
use parser_v2::variables::binding::VariableBindings;
use parser_v2::variables::substitution::resolve_variables;

fn parse_effect(input: &str, vars: &str) -> StandardEffect {
    let lex_result = parser_v2::lexer::tokenize::lex(input).unwrap();
    let bindings = VariableBindings::parse(vars).unwrap();
    let resolved = resolve_variables(&lex_result.tokens, &bindings).unwrap();

    let parser = effect_parser::single_effect_parser();
    parser.parse(&resolved).into_result().unwrap()
}

#[test]
fn test_draw_cards() {
    let result = parse_effect("Draw {cards}.", "cards: 2");

    match result {
        StandardEffect::DrawCards { count } => {
            assert_eq!(count, 2);
        }
        other => panic!("Expected DrawCards, got {other:?}"),
    }
}

#[test]
fn test_discard_cards() {
    let result = parse_effect("Discard {cards}.", "cards: 3");

    match result {
        StandardEffect::DiscardCards { count } => {
            assert_eq!(count, 3);
        }
        other => panic!("Expected DiscardCards, got {other:?}"),
    }
}

#[test]
fn test_gain_energy() {
    let result = parse_effect("Gain {e}.", "e: 5");

    match result {
        StandardEffect::GainEnergy { gains } => {
            assert_eq!(gains, Energy(5));
        }
        other => panic!("Expected GainEnergy, got {other:?}"),
    }
}
