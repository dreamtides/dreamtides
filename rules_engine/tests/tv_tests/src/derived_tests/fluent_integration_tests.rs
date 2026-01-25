use tv_lib::derived::fluent_integration::{
    format_expression, global_fluent_resource, initialize_fluent_resource, parse_variables,
};

#[test]
fn test_fluent_resource_initialization() {
    initialize_fluent_resource();
    assert!(global_fluent_resource().is_some());
}

#[test]
fn test_parse_variables_simple() {
    let result = parse_variables("e: 3\nn: 5").unwrap();
    let args = result.to_fluent_args();
    assert!(args.iter().count() == 2);
}

#[test]
fn test_parse_variables_empty() {
    let result = parse_variables("").unwrap();
    let args = result.to_fluent_args();
    assert!(args.iter().count() == 0);
}

#[test]
fn test_parse_variables_with_whitespace() {
    let result = parse_variables("  e : 3  \n  n : 5  ").unwrap();
    let args = result.to_fluent_args();
    assert!(args.iter().count() == 2);
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
    initialize_fluent_resource();
    let vars = parse_variables("e: 3").unwrap();
    let args = vars.to_fluent_args();
    let result = format_expression("{ $e }", &args);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "3");
}

#[test]
fn test_format_expression_with_term() {
    initialize_fluent_resource();
    let vars = parse_variables("k: 3").unwrap();
    let args = vars.to_fluent_args();
    let result = format_expression("{kindle}", &args);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("kindle"), "Output should contain 'kindle': {output}");
}

#[test]
fn test_format_expression_missing_variable() {
    initialize_fluent_resource();
    let vars = parse_variables("").unwrap();
    let args = vars.to_fluent_args();
    let result = format_expression("{ $missing_var }", &args);
    assert!(result.is_err());
}
