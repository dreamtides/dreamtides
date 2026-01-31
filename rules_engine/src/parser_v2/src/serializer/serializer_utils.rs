use ability_data::predicate::Operator;

pub fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Capitalizes the first keyword (inside curly braces) or the first letter.
///
/// If the string starts with `{keyword}`, capitalizes the keyword inside the
/// braces. Otherwise, capitalizes the first letter.
pub fn capitalize_first_keyword_or_letter(s: &str) -> String {
    if s.starts_with('{') {
        if let Some(end) = s.find('}') {
            let keyword = &s[1..end];
            format!("{{{}}}{}", capitalize_first_letter(keyword), &s[end + 1..])
        } else {
            capitalize_first_letter(s)
        }
    } else {
        capitalize_first_letter(s)
    }
}

pub fn serialize_operator<T>(operator: &Operator<T>) -> String {
    match operator {
        Operator::OrLess => "or less".to_string(),
        Operator::OrMore => "or more".to_string(),
        Operator::Exactly => "exactly".to_string(),
        Operator::LowerBy(_) => "lower".to_string(),
        Operator::HigherBy(_) => "higher".to_string(),
    }
}
