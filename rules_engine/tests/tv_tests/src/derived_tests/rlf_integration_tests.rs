use tv_lib::derived::rlf_integration::{format_expression, parse_variables};

#[test]
fn test_parse_variables_simple() {
    let result = parse_variables("e: 3\nn: 5").unwrap();
    let params = result.to_rlf_params();
    assert_eq!(params.len(), 2);
}

#[test]
fn test_parse_variables_empty() {
    let result = parse_variables("").unwrap();
    let params = result.to_rlf_params();
    assert_eq!(params.len(), 0);
}

#[test]
fn test_parse_variables_with_whitespace() {
    let result = parse_variables("  e : 3  \n  n : 5  ").unwrap();
    let params = result.to_rlf_params();
    assert_eq!(params.len(), 2);
}

#[test]
fn test_parse_variables_invalid_no_colon() {
    let result = parse_variables("invalid");
    assert!(result.is_err());
}

#[test]
fn test_parse_variables_invalid_empty_key() {
    let result = parse_variables(": 3");
    assert!(result.is_err());
}

#[test]
fn test_parse_variables_invalid_empty_value() {
    let result = parse_variables("e:");
    assert!(result.is_err());
}

#[test]
fn test_format_expression_simple_variable() {
    let vars = parse_variables("e: 3").unwrap();
    let params = vars.to_rlf_params();
    let result = format_expression("{$e}", params);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "3");
}

#[test]
fn test_format_expression_with_phrase() {
    let vars = parse_variables("k: 3").unwrap();
    let params = vars.to_rlf_params();
    let result = format_expression("{kindle($k)}", params);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("kindle"), "Output should contain 'kindle': {output}");
}

#[test]
fn test_format_expression_missing_phrase() {
    let vars = parse_variables("").unwrap();
    let params = vars.to_rlf_params();
    let result = format_expression("{nonexistent_phrase_xyz}", params);
    assert!(result.is_err());
}
