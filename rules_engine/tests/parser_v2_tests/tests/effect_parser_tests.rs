use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use insta::assert_ron_snapshot;
use parser_v2::lexer::lexer_tokenize;
use parser_v2::parser::effect_parser;
use parser_v2::variables::parser_bindings::VariableBindings;
use parser_v2::variables::parser_substitutions;

fn parse_effect(input: &str, vars: &str) -> StandardEffect {
    let lex_result = lexer_tokenize::lex(input).unwrap();
    let bindings = VariableBindings::parse(vars).unwrap();
    let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();

    let parser = effect_parser::single_effect_parser();
    parser.parse(&resolved).into_result().unwrap()
}

fn try_parse_effect(input: &str, vars: &str) -> Option<StandardEffect> {
    let lex_result = lexer_tokenize::lex(input).ok()?;
    let bindings = VariableBindings::parse(vars).ok()?;
    let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).ok()?;

    let parser = effect_parser::single_effect_parser();
    parser.parse(&resolved).into_result().ok()
}

#[test]
fn test_draw_cards() {
    let result = parse_effect("Draw {cards}.", "cards: 2");
    assert_ron_snapshot!(result, @r###"
    DrawCards(
      count: 2,
    )
    "###);
}

#[test]
fn test_discard_cards() {
    let result = parse_effect("Discard {discards}.", "discards: 3");
    assert_ron_snapshot!(result, @r###"
    DiscardCards(
      count: 3,
    )
    "###);
}

#[test]
fn test_gain_energy() {
    let result = parse_effect("Gain {e}.", "e: 5");
    assert_ron_snapshot!(result, @r###"
    GainEnergy(
      gains: Energy(5),
    )
    "###);
}

#[test]
fn test_gain_points() {
    let result = parse_effect("Gain {points}.", "points: 5");
    assert_ron_snapshot!(result, @r###"
    GainPoints(
      gains: Points(5),
    )
    "###);
}

#[test]
fn test_draw_cards_requires_cards_directive() {
    let result = try_parse_effect("Draw {discards}.", "discards: 2");
    assert_ron_snapshot!(result, @"None");
}
