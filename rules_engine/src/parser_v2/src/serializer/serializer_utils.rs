use ability_data::predicate::Operator;

/// Lowercases a leading keyword like "{Banish}" -> "{banish}" in a string.
pub fn lowercase_leading_keyword(s: &str) -> String {
    if s.starts_with('{') {
        if let Some(end) = s.find('}') {
            let keyword = &s[1..end];
            return format!("{{{}}}{}", keyword.to_lowercase(), &s[end + 1..]);
        }
    }
    s.to_string()
}

/// Capitalizes the first letter of a string, or the first letter of a
/// leading action keyword in braces (e.g., "{kindle}" -> "{Kindle}").
///
/// Only capitalizes known action keywords (kindle, foresee, prevent,
/// dissolve, banish, materialize, reclaim, discover), not other directives
/// like {e} (energy).
pub fn capitalize_first_letter(s: &str) -> String {
    if s.starts_with('{') {
        if let Some(end) = s.find('}') {
            let keyword = &s[1..end];
            if is_capitalizable_keyword(keyword) {
                let capitalized = capitalize_string(keyword);
                return format!("{{{}}}{}", capitalized, &s[end + 1..]);
            }
        }
    }
    capitalize_string(s)
}

/// Serializes an operator to its string representation.
pub fn serialize_operator<T>(operator: &Operator<T>) -> String {
    match operator {
        Operator::OrLess => "or less".to_string(),
        Operator::OrMore => "or more".to_string(),
        Operator::Exactly => "exactly".to_string(),
        Operator::LowerBy(_) => "lower".to_string(),
        Operator::HigherBy(_) => "higher".to_string(),
    }
}

fn capitalize_string(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn is_capitalizable_keyword(keyword: &str) -> bool {
    matches!(
        keyword.to_lowercase().as_str(),
        "kindle"
            | "foresee"
            | "prevent"
            | "dissolve"
            | "banish"
            | "materialize"
            | "reclaim"
            | "discover"
    )
}
