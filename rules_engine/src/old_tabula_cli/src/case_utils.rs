use convert_case::{Case, Casing};

pub fn cleaned_to_case(input: &str, case: Case) -> String {
    input
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .to_case(case)
}
