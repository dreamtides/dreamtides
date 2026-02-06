use std::collections::HashMap;

use ability_data::variable_value::VariableValue;
use core_data::card_types::CardSubtype;
use core_data::figment_type::FigmentType;
use parser_v2::variables::parser_bindings::VariableBindings;
use rlf::Value;
use strings::strings;

/// Evaluates a template string with RLF variable bindings.
///
/// Rewrites `{phrase(arg)}` to `{phrase:_p_arg}` selector syntax so that
/// variant phrases correctly select based on the argument's plural category.
/// Also normalizes CamelCase and hyphenated names to lowercase snake_case.
pub fn eval_str(template: &str, bindings: &VariableBindings) -> String {
    strings::register_source_phrases();
    let mut params = build_params(bindings);
    let rewritten = rewrite_template(template, &mut params);
    rlf::with_locale(|locale| {
        locale
            .eval_str(&rewritten, params)
            .unwrap_or_else(|e| panic!("Error evaluating template {template:?}: {e}"))
            .to_string()
    })
}

/// Converts [VariableBindings] to simple RLF parameters. Integer variables
/// become [Value::Number], subtypes and figments become [Value::Phrase].
fn build_params(bindings: &VariableBindings) -> HashMap<String, Value> {
    let mut params = HashMap::new();
    for (name, value) in bindings.iter() {
        let rlf_value = match value {
            VariableValue::Integer(n) => Value::Number(*n as i64),
            VariableValue::Subtype(subtype) => rlf::with_locale(|locale| {
                Value::Phrase(
                    locale
                        .get_phrase(subtype_phrase_name(*subtype))
                        .expect("subtype phrase should exist"),
                )
            }),
            VariableValue::Figment(figment) => rlf::with_locale(|locale| {
                Value::Phrase(
                    locale
                        .get_phrase(figment_phrase_name(*figment))
                        .expect("figment phrase should exist"),
                )
            }),
        };
        params.insert(name.clone(), rlf_value);
    }
    params
}

/// Rewrites all `{...}` interpolations in a template: normalizes names to
/// lowercase snake_case and converts function calls `{phrase(arg)}` to
/// selector syntax `{phrase:_p_arg}`. Bare parameter references like `{s}`
/// are rewritten to `{_p_s}`.
fn rewrite_template(template: &str, params: &mut HashMap<String, Value>) -> String {
    let original = std::mem::take(params);
    let param_names: Vec<String> = original.keys().cloned().collect();
    let mut sorted_keys: Vec<_> = original.keys().collect();
    sorted_keys.sort();
    for k in sorted_keys {
        params.insert(sanitize_param_name(k), original[k].clone());
    }

    let mut result = String::with_capacity(template.len());
    let mut chars = template.char_indices().peekable();

    while let Some((i, ch)) = chars.next() {
        if ch == '{'
            && let Some(close_pos) = template[i..].find('}')
        {
            let close_idx = i + close_pos;
            let content = &template[i + 1..close_idx];
            let rewritten = rewrite_interpolation(content, &param_names);
            result.push('{');
            result.push_str(&rewritten);
            result.push('}');
            while chars.peek().is_some_and(|&(j, _)| j < close_idx) {
                chars.next();
            }
            chars.next();
            continue;
        }
        result.push(ch);
    }

    result
}

/// Processes a single `{...}` interpolation block. Normalizes names and
/// converts function call syntax to selector syntax. Bare references that
/// match a parameter name are rewritten to use the `_p_` prefix.
fn rewrite_interpolation(content: &str, param_names: &[String]) -> String {
    let trimmed = content.trim();
    let (transforms_prefix, rest) = extract_transforms(trimmed);

    if let Some(paren_start) = rest.find('(')
        && let Some(paren_end) = rest.find(')')
    {
        let phrase_name = rest[..paren_start].trim();
        let normalized = normalize_rlf_name(phrase_name);
        let args_str = &rest[paren_start + 1..paren_end];
        let suffix = &rest[paren_end + 1..];
        let args: Vec<&str> = args_str.split(',').map(str::trim).collect();

        let prefixed_args: Vec<String> = args.iter().map(|arg| sanitize_param_name(arg)).collect();
        let selector_suffix: String = prefixed_args.iter().map(|a| format!(":{a}")).collect();

        format!("{transforms_prefix}{normalized}{selector_suffix}{suffix}")
    } else if param_names.iter().any(|p| p == rest) {
        format!("{transforms_prefix}{}", sanitize_param_name(rest))
    } else {
        let normalized = normalize_rlf_name(rest);
        format!("{transforms_prefix}{normalized}")
    }
}

/// Converts CamelCase, hyphenated, and mixed names to lowercase snake_case.
///
/// - `Kindle` -> `kindle`
/// - `Reclaim_for_cost` -> `reclaim_for_cost`
/// - `energy-symbol` -> `energy_symbol`
/// - `energy_symbol` -> `energy_symbol` (no-op)
fn normalize_rlf_name(name: &str) -> String {
    let mut result = String::with_capacity(name.len());
    let mut prev_was_lower = false;

    for ch in name.chars() {
        if ch == '-' {
            result.push('_');
            prev_was_lower = false;
        } else if ch.is_ascii_uppercase() && prev_was_lower {
            result.push('_');
            result.push(ch.to_ascii_lowercase());
            prev_was_lower = true;
        } else {
            result.push(ch.to_ascii_lowercase());
            prev_was_lower = ch.is_ascii_lowercase();
        }
    }

    result
}

/// Converts a parameter name to a prefixed, sanitized RLF identifier.
///
/// Adds `_p_` prefix and replaces hyphens with underscores since RLF
/// identifiers do not support hyphens.
fn sanitize_param_name(name: &str) -> String {
    format!("_p_{}", name.replace('-', "_"))
}

/// Extracts leading `@transform` prefixes from an interpolation, returning
/// the prefix string and the remaining content.
fn extract_transforms(content: &str) -> (String, &str) {
    let mut prefix = String::new();
    let mut rest = content;

    while let Some(stripped) = rest.strip_prefix('@') {
        if let Some(space_pos) = stripped.find(' ') {
            prefix.push('@');
            prefix.push_str(&stripped[..space_pos + 1]);
            rest = &stripped[space_pos + 1..];
        } else {
            break;
        }
    }

    (prefix, rest)
}

/// Returns the RLF phrase name for a [CardSubtype].
fn subtype_phrase_name(subtype: CardSubtype) -> &'static str {
    match subtype {
        CardSubtype::Agent => "agent",
        CardSubtype::Ancient => "ancient",
        CardSubtype::Avatar => "avatar",
        CardSubtype::Child => "child",
        CardSubtype::Detective => "detective",
        CardSubtype::Enigma => "enigma",
        CardSubtype::Explorer => "explorer",
        CardSubtype::Guide => "guide",
        CardSubtype::Hacker => "hacker",
        CardSubtype::Mage => "mage",
        CardSubtype::Monster => "monster",
        CardSubtype::Musician => "musician",
        CardSubtype::Outsider => "outsider",
        CardSubtype::Renegade => "renegade",
        CardSubtype::Robot => "robot",
        CardSubtype::SpiritAnimal => "spirit_animal",
        CardSubtype::Super => "super_",
        CardSubtype::Survivor => "survivor",
        CardSubtype::Synth => "synth",
        CardSubtype::Tinkerer => "tinkerer",
        CardSubtype::Trooper => "trooper",
        CardSubtype::Visionary => "visionary",
        CardSubtype::Visitor => "visitor",
        CardSubtype::Warrior => "warrior",
    }
}

/// Returns the RLF phrase name for a [FigmentType].
fn figment_phrase_name(figment: FigmentType) -> &'static str {
    match figment {
        FigmentType::Celestial => "celestial",
        FigmentType::Halcyon => "halcyon",
        FigmentType::Radiant => "radiant",
        FigmentType::Shadow => "shadow",
    }
}
