use convert_case::{Case, Casing};

pub fn normalize_column_name(name: &str) -> String {
    normalize(name, "column")
}

pub fn normalize_table_name(name: &str) -> String {
    normalize(name, "table")
}

fn normalize(name: &str, default_name: &str) -> String {
    let kebab = name.to_case(Case::Kebab);
    let mut cleaned = String::new();
    let mut last_hyphen = false;

    for ch in kebab.chars() {
        if ch.is_ascii_alphanumeric() {
            cleaned.push(ch);
            last_hyphen = false;
        } else if ch == '-' {
            if !cleaned.is_empty() && !last_hyphen {
                cleaned.push('-');
                last_hyphen = true;
            }
        } else if !cleaned.is_empty() && !last_hyphen {
            cleaned.push('-');
            last_hyphen = true;
        }
    }

    while cleaned.ends_with('-') {
        cleaned.pop();
    }

    if cleaned.is_empty() { default_name.to_string() } else { cleaned }
}
