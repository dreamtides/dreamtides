use std::sync::{Arc, OnceLock};

use fluent::FluentBundle;
use fluent_bundle::types::FluentNumber;
use fluent_bundle::{FluentArgs, FluentError, FluentResource, FluentValue};

static GLOBAL_FLUENT_RESOURCE: OnceLock<Arc<FluentResource>> = OnceLock::new();

/// Initializes the global Fluent resource from the embedded strings.ftl file.
pub fn initialize_fluent_resource() {
    GLOBAL_FLUENT_RESOURCE.get_or_init(|| {
        let ftl_source = include_str!("../../../../../tabula/strings.ftl");
        match FluentResource::try_new(ftl_source.to_string()) {
            Ok(resource) => {
                tracing::info!(
                    component = "derived.rules_preview",
                    "Loaded FluentResource from strings.ftl"
                );
                Arc::new(resource)
            }
            Err((resource, errors)) => {
                let error_vec: Vec<FluentError> =
                    errors.into_iter().map(FluentError::ParserError).collect();
                tracing::error!(
                    component = "derived.rules_preview",
                    errors = %format_fluent_errors(&error_vec),
                    "Fluent parse errors in strings.ftl"
                );
                Arc::new(resource)
            }
        }
    });
}

/// Returns a reference to the global Fluent resource.
pub fn global_fluent_resource() -> Option<&'static Arc<FluentResource>> {
    GLOBAL_FLUENT_RESOURCE.get()
}

/// Formats an expression using the global Fluent resource and provided arguments.
pub fn format_expression(expression: &str, args: &FluentArgs) -> Result<String, String> {
    tracing::debug!(
        component = "derived.rules_preview",
        expression_len = expression.len(),
        arg_count = args.iter().count(),
        "Formatting Fluent expression"
    );

    let Some(resource) = global_fluent_resource() else {
        return Err("Fluent resource not initialized".to_string());
    };

    let mut bundle: FluentBundle<Arc<FluentResource>> = FluentBundle::default();
    bundle.set_use_isolating(false);

    if let Err(errors) = bundle.add_resource(Arc::clone(resource)) {
        return Err(format!("Failed to add Fluent resource: {}", format_fluent_errors(&errors)));
    }

    let temp_ftl = build_temp_message(expression);
    let temp_resource = match FluentResource::try_new(temp_ftl) {
        Ok(r) => r,
        Err((_, errors)) => {
            let error_vec: Vec<FluentError> =
                errors.into_iter().map(FluentError::ParserError).collect();
            tracing::error!(
                component = "derived.rules_preview",
                expression = expression,
                errors = %format_fluent_errors(&error_vec),
                "Failed to parse Fluent expression"
            );
            return Err(format!("Failed to parse expression: {}", format_fluent_errors(&error_vec)));
        }
    };

    if let Err(errors) = bundle.add_resource(Arc::new(temp_resource)) {
        return Err(format!(
            "Failed to add temporary message: {}",
            format_fluent_errors(&errors)
        ));
    }

    let Some(message) = bundle.get_message("temp-message") else {
        return Err("Temporary message not found in bundle".to_string());
    };

    let Some(pattern) = message.value() else {
        return Err("Message has no value".to_string());
    };

    let mut errors = Vec::new();
    let formatted = bundle.format_pattern(pattern, Some(args), &mut errors);

    if !errors.is_empty() {
        tracing::warn!(
            component = "derived.rules_preview",
            expression = expression,
            errors = %format_fluent_errors(&errors),
            "Missing variable or term references during Fluent formatting"
        );
        return Err(format!("Fluent formatting errors: {}", format_fluent_errors(&errors)));
    }

    Ok(formatted.into_owned())
}

/// Parsed variable key-value pairs that can be converted to FluentArgs.
#[derive(Debug, Clone)]
pub struct ParsedVariables {
    pairs: Vec<(String, String)>,
}

impl ParsedVariables {
    /// Converts the parsed variables into FluentArgs.
    pub fn to_fluent_args(&self) -> FluentArgs<'static> {
        let mut args = FluentArgs::new();
        for (key, value) in &self.pairs {
            let fluent_value = if let Ok(n) = value.parse::<f64>() {
                FluentValue::Number(FluentNumber::new(n, Default::default()))
            } else {
                FluentValue::String(value.clone().into())
            };
            args.set(key.clone(), fluent_value);
        }
        args
    }
}

/// Parses a variables string into ParsedVariables.
pub fn parse_variables(variables_text: &str) -> Result<ParsedVariables, String> {
    let mut pairs = Vec::new();

    for line in variables_text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            return Err(format!("Invalid variable definition: '{line}'"));
        };

        let key = key.trim();
        let value = value.trim();

        if key.is_empty() || value.is_empty() {
            return Err(format!("Invalid variable definition: '{line}'"));
        }

        pairs.push((key.to_string(), value.to_string()));
    }

    Ok(ParsedVariables { pairs })
}

fn expand_plain_variables(expression: &str) -> String {
    let mut result = String::with_capacity(expression.len());
    let mut chars = expression.chars().peekable();
    let mut depth = 0;

    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                depth += 1;
                result.push(ch);
            }
            '}' => {
                if depth > 0 {
                    depth -= 1;
                }
                result.push(ch);
            }
            '$' if depth == 0 => {
                let mut name = String::new();
                while let Some(&next) = chars.peek() {
                    if next == '_' || next == '-' || next.is_ascii_alphanumeric() {
                        name.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if name.is_empty() {
                    result.push('$');
                } else {
                    result.push_str("{ $");
                    result.push_str(&name);
                    result.push_str(" }");
                }
            }
            _ => result.push(ch),
        }
    }

    result
}

fn build_temp_message(expression: &str) -> String {
    let expanded = expand_plain_variables(expression);
    let normalized = expanded.replace("\r\n", "\n").replace('\r', "\n");

    if !normalized.contains('\n') {
        return format!("temp-message = {normalized}");
    }

    let mut lines = normalized.split('\n');
    let first = lines.next().unwrap_or("");
    let mut temp =
        if first.is_empty() { "temp-message =".to_string() } else { format!("temp-message = {first}") };

    for line in lines {
        temp.push('\n');
        temp.push_str("    ");
        temp.push_str(line);
    }

    temp
}

fn format_fluent_errors(errors: &[FluentError]) -> String {
    errors
        .iter()
        .map(|e| match e {
            FluentError::Overriding { kind, id } => format!("Overriding {kind} id={id}"),
            FluentError::ParserError(pe) => format!("Parser error: {pe}"),
            FluentError::ResolverError(re) => format!("Resolver error: {re}"),
        })
        .collect::<Vec<_>>()
        .join("; ")
}