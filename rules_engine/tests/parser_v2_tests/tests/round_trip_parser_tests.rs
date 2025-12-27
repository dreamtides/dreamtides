use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use parser_v2::lexer::lexer_tokenize;
use parser_v2::parser::effect_parser;
use parser_v2::serializer::parser_formatter;
use parser_v2::variables::parser_bindings::VariableBindings;
use parser_v2::variables::parser_substitutions::resolve_variables;

fn parse_effect(input: &str, vars: &str) -> StandardEffect {
    let lex_result = lexer_tokenize::lex(input).unwrap();
    let bindings = VariableBindings::parse(vars).unwrap();
    let resolved = resolve_variables(&lex_result.tokens, &bindings).unwrap();

    let parser = effect_parser::single_effect_parser();
    parser.parse(&resolved).into_result().unwrap()
}

#[test]
fn test_round_trip_draw_cards() {
    let original = "draw {cards}.";
    let vars = "cards: 2";

    let parsed = parse_effect(original, vars);
    let serialized = parser_formatter::serialize_standard_effect(&parsed);

    assert_eq!(serialized, original);

    let reparsed = parse_effect(&serialized, vars);
    assert_eq!(format!("{parsed:?}"), format!("{reparsed:?}"));
}

#[test]
fn test_round_trip_discard_cards() {
    let original = "discard {discards}.";
    let vars = "discards: 3";

    let parsed = parse_effect(original, vars);
    let serialized = parser_formatter::serialize_standard_effect(&parsed);

    assert_eq!(serialized, original);

    let reparsed = parse_effect(&serialized, vars);
    assert_eq!(format!("{parsed:?}"), format!("{reparsed:?}"));
}

#[test]
fn test_round_trip_gain_energy() {
    let original = "gain {e}.";
    let vars = "e: 5";

    let parsed = parse_effect(original, vars);
    let serialized = parser_formatter::serialize_standard_effect(&parsed);

    assert_eq!(serialized, original);

    let reparsed = parse_effect(&serialized, vars);
    assert_eq!(format!("{parsed:?}"), format!("{reparsed:?}"));
}
