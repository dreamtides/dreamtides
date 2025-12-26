use ability_data::standard_effect::StandardEffect;
use chumsky::prelude::*;
use parser_v2::parser::effect_parser;
use parser_v2::serializer::formatter;
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
fn test_round_trip_draw_cards() {
    let original = "Draw {cards}.";
    let vars = "cards: 2";

    let parsed = parse_effect(original, vars);
    let serialized = formatter::serialize_standard_effect(&parsed);

    assert_eq!(serialized, original);

    let reparsed = parse_effect(&serialized, vars);
    assert_eq!(format!("{parsed:?}"), format!("{reparsed:?}"));
}

#[test]
fn test_round_trip_discard_cards() {
    let original = "Discard {cards}.";
    let vars = "cards: 3";

    let parsed = parse_effect(original, vars);
    let serialized = formatter::serialize_standard_effect(&parsed);

    assert_eq!(serialized, original);

    let reparsed = parse_effect(&serialized, vars);
    assert_eq!(format!("{parsed:?}"), format!("{reparsed:?}"));
}

#[test]
fn test_round_trip_gain_energy() {
    let original = "Gain {e}.";
    let vars = "e: 5";

    let parsed = parse_effect(original, vars);
    let serialized = formatter::serialize_standard_effect(&parsed);

    assert_eq!(serialized, original);

    let reparsed = parse_effect(&serialized, vars);
    assert_eq!(format!("{parsed:?}"), format!("{reparsed:?}"));
}
