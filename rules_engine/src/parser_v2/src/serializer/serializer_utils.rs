use ability_data::predicate::Operator;

pub fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
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
