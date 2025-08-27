use std::collections::HashSet;
use std::fs;

use anyhow::Result;
use convert_case::Case;
use tabula_data::tabula::TabulaRaw;

use crate::case_utils;

pub fn generate_string_ids(tabula_raw: &TabulaRaw, output_path: &str) -> Result<()> {
    let mut out = String::new();
    out.push_str("use tabula_data::localized_strings::StringId;\n");
    out.push_str("use uuid::uuid;\n\n");

    let mut seen_names: HashSet<String> = HashSet::new();
    for row in tabula_raw.strings.as_slice() {
        let const_name = case_utils::cleaned_to_case(&row.name, Case::UpperSnake);
        if seen_names.contains(&const_name) {
            continue;
        }
        seen_names.insert(const_name.clone());
        out.push_str(&format!("/// {}\n", row.description.replace('\n', " ").trim()));
        let const_declaration =
            format!("pub const {}: StringId = StringId(uuid!(\"{}\"));", const_name, row.id.0);
        if const_declaration.len() > 100 {
            out.push_str(&format!(
                "pub const {}: StringId =\n    StringId(uuid!(\"{}\"));\n\n",
                const_name, row.id.0
            ));
        } else {
            out.push_str(&format!("{const_declaration}\n\n"));
        }
    }
    out = format!("{}\n", out.trim_end_matches('\n'));
    fs::write(output_path, out)?;
    Ok(())
}

pub fn generate_test_card_ids(tabula_raw: &TabulaRaw, output_path: &str) -> Result<()> {
    let mut out = String::new();
    out.push_str("use core_data::identifiers::BaseCardId;\n");
    out.push_str("use uuid::uuid;\n\n");

    let mut seen_names: HashSet<String> = HashSet::new();
    let mut const_names: Vec<String> = Vec::new();
    for row in tabula_raw.test_cards.as_slice() {
        let const_name = case_utils::cleaned_to_case(&row.name_en_us, Case::UpperSnake);
        if seen_names.contains(&const_name) {
            continue;
        }
        seen_names.insert(const_name.clone());
        const_names.push(const_name.clone());
        if !row.rules_text_en_us.trim().is_empty() {
            out.push_str(&format!("/// {}\n", row.rules_text_en_us.replace('\n', " ").trim()));
        }
        let const_declaration =
            format!("pub const {}: BaseCardId = BaseCardId(uuid!(\"{}\"));", const_name, row.id.0);
        if const_declaration.len() > 100 {
            out.push_str(&format!(
                "pub const {}: BaseCardId =\n    BaseCardId(uuid!(\"{}\"));\n\n",
                const_name, row.id.0
            ));
        } else {
            out.push_str(&format!("{const_declaration}\n\n"));
        }
    }

    if !const_names.is_empty() {
        out.push_str("pub const ALL_TEST_CARD_IDS: &[BaseCardId] = &[\n");
        for name in const_names.iter() {
            out.push_str(&format!("    {name},\n"));
        }
        out.push_str("];\n\n");
    }
    out = format!("{}\n", out.trim_end_matches('\n'));
    fs::write(output_path, out)?;
    Ok(())
}
