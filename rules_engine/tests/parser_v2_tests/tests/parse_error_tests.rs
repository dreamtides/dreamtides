use parser_v2::error::parser_diagnostics;
use parser_v2::error::parser_errors::ParserError;
use parser_v2::lexer::tokenize;
use parser_v2::variables::binding::VariableBindings;
use parser_v2::variables::substitution;

#[test]
fn test_unclosed_brace_error() {
    let input = "Draw {cards.";
    let result = tokenize::lex(input);

    assert!(result.is_err());
    let error = ParserError::from(result.unwrap_err());
    let formatted = parser_diagnostics::format_error(&error, input, "test");

    assert!(formatted.contains("Unclosed brace"));
    assert!(formatted.contains("Expected closing '}'"));
}

#[test]
fn test_empty_directive_error() {
    let input = "Draw {}.";
    let result = tokenize::lex(input);

    assert!(result.is_err());
    let error = ParserError::from(result.unwrap_err());
    let formatted = parser_diagnostics::format_error(&error, input, "test");

    assert!(formatted.contains("Empty directive"));
}

#[test]
fn test_unresolved_variable_with_suggestion() {
    let input = "Draw {cards}.";
    let lex_result = tokenize::lex(input).unwrap();
    let bindings = VariableBindings::new();

    let result = substitution::resolve_variables(&lex_result.tokens, &bindings);

    assert!(result.is_err());
    let error = ParserError::from(result.unwrap_err());
    let formatted = parser_diagnostics::format_error(&error, input, "test");

    assert!(formatted.contains("Unresolved variable: cards"));
}

#[test]
fn test_unresolved_variable_with_typo_in_directive() {
    let input = "Gain {e}.";
    let lex_result = tokenize::lex(input).unwrap();
    let bindings = VariableBindings::new();

    let result = substitution::resolve_variables(&lex_result.tokens, &bindings);

    assert!(result.is_err());
    let error = ParserError::from(result.unwrap_err());
    let formatted = parser_diagnostics::format_error(&error, input, "test");

    assert!(formatted.contains("Unresolved variable"));
}

#[test]
fn test_missing_variable_binding() {
    let input = "Draw {cards}.";
    let lex_result = tokenize::lex(input).unwrap();
    let bindings = VariableBindings::new();

    let result = substitution::resolve_variables(&lex_result.tokens, &bindings);

    assert!(result.is_err());
    let error = ParserError::from(result.unwrap_err());
    let formatted = parser_diagnostics::format_error(&error, input, "test");

    assert!(formatted.contains("cards"));
    assert!(formatted.contains("not found"));
}

#[test]
fn test_error_formatting_includes_span() {
    let input = "This is some text {unclosed";
    let result = tokenize::lex(input);

    assert!(result.is_err());
    let error = ParserError::from(result.unwrap_err());
    let formatted = parser_diagnostics::format_error(&error, input, "test.txt");

    assert!(formatted.contains("test.txt"));
}

#[test]
fn test_multiple_errors_first_is_reported() {
    let input = "{unclosed {another";
    let result = tokenize::lex(input);

    assert!(result.is_err());
}

#[test]
fn test_unresolved_figment_variable() {
    let input = "{Materialize} {n-figments}.";
    let lex_result = tokenize::lex(input).unwrap();
    let bindings = VariableBindings::new();

    let result = substitution::resolve_variables(&lex_result.tokens, &bindings);

    assert!(result.is_err());
    let error = ParserError::from(result.unwrap_err());
    let formatted = parser_diagnostics::format_error(&error, input, "test");

    assert!(formatted.contains("figment") || formatted.contains("number"));
}

#[test]
fn test_unresolved_subtype_variable() {
    let input = "Allied {plural-subtype} have +2 spark.";
    let lex_result = tokenize::lex(input).unwrap();
    let bindings = VariableBindings::new();

    let result = substitution::resolve_variables(&lex_result.tokens, &bindings);

    assert!(result.is_err());
    let error = ParserError::from(result.unwrap_err());
    let formatted = parser_diagnostics::format_error(&error, input, "test");

    assert!(formatted.contains("subtype"));
}

#[test]
fn test_variable_parse_error() {
    let vars = "invalid format without colon";
    let result = VariableBindings::parse(vars);

    assert!(result.is_err());
}

#[test]
fn test_suggestions_for_close_variable_names() {
    let input = "Discard {discards}.";
    let lex_result = tokenize::lex(input).unwrap();
    let bindings = VariableBindings::new();

    let result = substitution::resolve_variables(&lex_result.tokens, &bindings);

    assert!(result.is_err());
    let error = ParserError::from(result.unwrap_err());
    let formatted = parser_diagnostics::format_error(&error, input, "test");

    assert!(formatted.contains("discards"));
}
