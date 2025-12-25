use ability_data::figment_type::FigmentType;
use ability_data::variable_value::VariableValue;
use core_data::card_types::CardSubtype;
use parser_v2::lexer::span::Span;
use parser_v2::lexer::token::Token;
use parser_v2::variables::binding::VariableBindings;
use parser_v2::variables::substitution::{resolve_variables, ResolvedToken};

#[test]
fn test_parse_integer_variable() {
    let bindings = VariableBindings::parse("cards: 2").unwrap();
    assert_eq!(bindings.get("cards"), Some(&VariableValue::Integer(2)));
}

#[test]
fn test_parse_multiple_integer_variables() {
    let bindings = VariableBindings::parse("cards: 2, e: 3, points: 1").unwrap();
    assert_eq!(bindings.get("cards"), Some(&VariableValue::Integer(2)));
    assert_eq!(bindings.get("e"), Some(&VariableValue::Integer(3)));
    assert_eq!(bindings.get("points"), Some(&VariableValue::Integer(1)));
}

#[test]
fn test_parse_subtype_variable() {
    let bindings = VariableBindings::parse("subtype: Warrior").unwrap();
    assert_eq!(bindings.get("subtype"), Some(&VariableValue::Subtype(CardSubtype::Warrior)));
}

#[test]
fn test_parse_figment_variable() {
    let bindings = VariableBindings::parse("figment: radiant").unwrap();
    assert_eq!(bindings.get("figment"), Some(&VariableValue::Figment(FigmentType::Radiant)));
}

#[test]
fn test_parse_mixed_variables() {
    let bindings = VariableBindings::parse("cards: 2, subtype: Explorer, figment: shadow").unwrap();
    assert_eq!(bindings.get("cards"), Some(&VariableValue::Integer(2)));
    assert_eq!(bindings.get("subtype"), Some(&VariableValue::Subtype(CardSubtype::Explorer)));
    assert_eq!(bindings.get("figment"), Some(&VariableValue::Figment(FigmentType::Shadow)));
}

#[test]
fn test_parse_newline_separated() {
    let bindings = VariableBindings::parse("cards: 2\ne: 3").unwrap();
    assert_eq!(bindings.get("cards"), Some(&VariableValue::Integer(2)));
    assert_eq!(bindings.get("e"), Some(&VariableValue::Integer(3)));
}

#[test]
fn test_parse_with_whitespace() {
    let bindings = VariableBindings::parse("  cards : 2  ,  e : 3  ").unwrap();
    assert_eq!(bindings.get("cards"), Some(&VariableValue::Integer(2)));
    assert_eq!(bindings.get("e"), Some(&VariableValue::Integer(3)));
}

#[test]
fn test_parse_empty_string() {
    let bindings = VariableBindings::parse("").unwrap();
    assert_eq!(bindings.get("cards"), None);
}

#[test]
fn test_parse_invalid_format() {
    let result = VariableBindings::parse("cards 2");
    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_value() {
    let result = VariableBindings::parse("value: invalid");
    assert!(result.is_err());
}

#[test]
fn test_get_integer_helper() {
    let bindings = VariableBindings::parse("cards: 2").unwrap();
    assert_eq!(bindings.get_integer("cards"), Some(2));
    assert_eq!(bindings.get_integer("missing"), None);
}

#[test]
fn test_get_subtype_helper() {
    let bindings = VariableBindings::parse("subtype: Warrior").unwrap();
    assert_eq!(bindings.get_subtype("subtype"), Some(CardSubtype::Warrior));
    assert_eq!(bindings.get_subtype("missing"), None);
}

#[test]
fn test_get_figment_helper() {
    let bindings = VariableBindings::parse("figment: radiant").unwrap();
    assert_eq!(bindings.get_figment("figment"), Some(FigmentType::Radiant));
    assert_eq!(bindings.get_figment("missing"), None);
}

#[test]
fn test_resolve_simple_integer() {
    let tokens = vec![(Token::Directive("cards".to_string()), Span::new(0, 7))];
    let bindings = VariableBindings::parse("cards: 2").unwrap();

    let resolved = resolve_variables(&tokens, &bindings).unwrap();
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].0, ResolvedToken::Integer(2));
}

#[test]
fn test_resolve_simple_subtype() {
    let tokens = vec![(Token::Directive("subtype".to_string()), Span::new(0, 9))];
    let bindings = VariableBindings::parse("subtype: Warrior").unwrap();

    let resolved = resolve_variables(&tokens, &bindings).unwrap();
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].0, ResolvedToken::Subtype(CardSubtype::Warrior));
}

#[test]
fn test_resolve_non_variable_directive() {
    let tokens = vec![(Token::Directive("Judgment".to_string()), Span::new(0, 10))];
    let bindings = VariableBindings::new();

    let resolved = resolve_variables(&tokens, &bindings).unwrap();
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].0, ResolvedToken::Token(Token::Directive("Judgment".to_string())));
}

#[test]
fn test_resolve_word_token() {
    let tokens = vec![(Token::Word("draw".to_string()), Span::new(0, 4))];
    let bindings = VariableBindings::new();

    let resolved = resolve_variables(&tokens, &bindings).unwrap();
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].0, ResolvedToken::Token(Token::Word("draw".to_string())));
}

#[test]
fn test_resolve_compound_n_figments() {
    let tokens = vec![(Token::Directive("n-figments".to_string()), Span::new(0, 12))];
    let bindings = VariableBindings::parse("number: 3, figment: radiant").unwrap();

    let resolved = resolve_variables(&tokens, &bindings).unwrap();
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].0, ResolvedToken::FigmentCount {
        count: 3,
        figment_type: FigmentType::Radiant
    });
}

#[test]
fn test_resolve_compound_a_figment() {
    let tokens = vec![(Token::Directive("a-figment".to_string()), Span::new(0, 11))];
    let bindings = VariableBindings::parse("figment: shadow").unwrap();

    let resolved = resolve_variables(&tokens, &bindings).unwrap();
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].0, ResolvedToken::FigmentSingle { figment_type: FigmentType::Shadow });
}

#[test]
fn test_resolve_compound_missing_number() {
    let tokens = vec![(Token::Directive("n-figments".to_string()), Span::new(0, 12))];
    let bindings = VariableBindings::parse("figment: radiant").unwrap();

    let result = resolve_variables(&tokens, &bindings);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.name, "number");
}

#[test]
fn test_resolve_compound_missing_figment() {
    let tokens = vec![(Token::Directive("a-figment".to_string()), Span::new(0, 11))];
    let bindings = VariableBindings::new();

    let result = resolve_variables(&tokens, &bindings);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.name, "figment");
}

#[test]
fn test_resolve_missing_variable() {
    let tokens = vec![(Token::Directive("cards".to_string()), Span::new(0, 7))];
    let bindings = VariableBindings::new();

    let result = resolve_variables(&tokens, &bindings);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.name, "cards");
}

#[test]
fn test_resolve_mixed_tokens() {
    let tokens = vec![
        (Token::Word("draw".to_string()), Span::new(0, 4)),
        (Token::Directive("cards".to_string()), Span::new(5, 12)),
        (Token::Period, Span::new(12, 13)),
    ];
    let bindings = VariableBindings::parse("cards: 2").unwrap();

    let resolved = resolve_variables(&tokens, &bindings).unwrap();
    assert_eq!(resolved.len(), 3);
    assert_eq!(resolved[0].0, ResolvedToken::Token(Token::Word("draw".to_string())));
    assert_eq!(resolved[1].0, ResolvedToken::Integer(2));
    assert_eq!(resolved[2].0, ResolvedToken::Token(Token::Period));
}

#[test]
fn test_representative_card_1() {
    let tokens = vec![
        (Token::Word("when".to_string()), Span::new(0, 4)),
        (Token::Word("you".to_string()), Span::new(5, 8)),
        (Token::Word("play".to_string()), Span::new(9, 13)),
        (Token::Directive("cards-numeral".to_string()), Span::new(14, 29)),
    ];
    let bindings = VariableBindings::parse("cards: 2").unwrap();

    let resolved = resolve_variables(&tokens, &bindings).unwrap();
    assert_eq!(resolved.len(), 4);
    assert_eq!(resolved[3].0, ResolvedToken::Integer(2));
}

#[test]
fn test_representative_card_4() {
    let tokens = vec![
        (Token::Directive("discards".to_string()), Span::new(0, 10)),
        (Token::Word("to".to_string()), Span::new(11, 13)),
        (Token::Word("draw".to_string()), Span::new(14, 18)),
        (Token::Directive("cards".to_string()), Span::new(19, 26)),
        (Token::Word("and".to_string()), Span::new(27, 30)),
        (Token::Word("gain".to_string()), Span::new(31, 35)),
        (Token::Directive("points".to_string()), Span::new(36, 44)),
    ];
    let bindings = VariableBindings::parse("discards: 1, cards: 1, points: 1").unwrap();

    let resolved = resolve_variables(&tokens, &bindings).unwrap();
    assert_eq!(resolved.len(), 7);
    assert_eq!(resolved[0].0, ResolvedToken::Integer(1));
    assert_eq!(resolved[3].0, ResolvedToken::Integer(1));
    assert_eq!(resolved[6].0, ResolvedToken::Integer(1));
}

#[test]
fn test_representative_card_7() {
    let tokens = vec![
        (Token::Word("character".to_string()), Span::new(0, 9)),
        (Token::Word("with".to_string()), Span::new(10, 14)),
        (Token::Word("cost".to_string()), Span::new(15, 19)),
        (Token::Directive("e".to_string()), Span::new(20, 23)),
    ];
    let bindings = VariableBindings::parse("e: 2").unwrap();

    let resolved = resolve_variables(&tokens, &bindings).unwrap();
    assert_eq!(resolved.len(), 4);
    assert_eq!(resolved[3].0, ResolvedToken::Integer(2));
}

#[test]
fn test_representative_card_9() {
    let tokens = vec![
        (Token::Directive("Materialize".to_string()), Span::new(0, 13)),
        (Token::Directive("n-figments".to_string()), Span::new(14, 26)),
        (Token::Period, Span::new(26, 27)),
    ];
    let bindings = VariableBindings::parse("number: 3, figment: radiant").unwrap();

    let resolved = resolve_variables(&tokens, &bindings).unwrap();
    assert_eq!(resolved.len(), 3);
    assert_eq!(resolved[0].0, ResolvedToken::Token(Token::Directive("Materialize".to_string())));
    assert_eq!(resolved[1].0, ResolvedToken::FigmentCount {
        count: 3,
        figment_type: FigmentType::Radiant
    });
}

#[test]
fn test_round_trip_bindings() {
    let original = "cards: 2, e: 3, subtype: Warrior, figment: radiant";
    let bindings1 = VariableBindings::parse(original).unwrap();

    let bindings2 = VariableBindings::parse(original).unwrap();
    assert_eq!(bindings1, bindings2);
}

#[test]
fn test_insert_variable() {
    let mut bindings = VariableBindings::new();
    bindings.insert("cards".to_string(), VariableValue::Integer(2));
    assert_eq!(bindings.get_integer("cards"), Some(2));
}

#[test]
fn test_variable_directive_recognition() {
    let tokens = vec![
        (Token::Directive("e".to_string()), Span::new(0, 3)),
        (Token::Directive("cards".to_string()), Span::new(4, 11)),
        (Token::Directive("discards".to_string()), Span::new(12, 22)),
        (Token::Directive("points".to_string()), Span::new(23, 31)),
        (Token::Directive("s".to_string()), Span::new(32, 35)),
        (Token::Directive("k".to_string()), Span::new(36, 39)),
        (Token::Directive("subtype".to_string()), Span::new(40, 49)),
    ];
    let bindings = VariableBindings::parse(
        "e: 1, cards: 2, discards: 3, points: 4, s: 5, k: 6, subtype: Warrior",
    )
    .unwrap();

    let resolved = resolve_variables(&tokens, &bindings).unwrap();
    assert_eq!(resolved.len(), 7);
    assert_eq!(resolved[0].0, ResolvedToken::Integer(1));
    assert_eq!(resolved[1].0, ResolvedToken::Integer(2));
    assert_eq!(resolved[2].0, ResolvedToken::Integer(3));
    assert_eq!(resolved[3].0, ResolvedToken::Integer(4));
    assert_eq!(resolved[4].0, ResolvedToken::Integer(5));
    assert_eq!(resolved[5].0, ResolvedToken::Integer(6));
    assert_eq!(resolved[6].0, ResolvedToken::Subtype(CardSubtype::Warrior));
}
