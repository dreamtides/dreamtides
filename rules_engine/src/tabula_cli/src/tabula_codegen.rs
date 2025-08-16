use std::fs;

use anyhow::Result;
use convert_case::Case;
use tabula_data::tabula::Tabula;

use crate::case_utils;

pub fn generate_string_ids(tabula: &Tabula, output_path: &str) -> Result<()> {
    let mut out = String::new();
    out.push_str("use tabula_data::localized_strings::StringId;\n");
    out.push_str("use uuid::uuid;\n\n");

    for row in tabula.strings.rows() {
        out.push_str(&format!("/// {}\n", row.description.replace('\n', " ").trim()));
        let const_declaration = format!(
            "pub const {}: StringId = StringId(uuid!(\"{}\"));",
            case_utils::cleaned_to_case(&row.name, Case::UpperSnake),
            row.id.0
        );
        if const_declaration.len() > 100 {
            out.push_str(&format!(
                "pub const {}: StringId =\n    StringId(uuid!(\"{}\"));\n\n",
                case_utils::cleaned_to_case(&row.name, Case::UpperSnake),
                row.id.0
            ));
        } else {
            out.push_str(&format!("{const_declaration}\n\n"));
        }
    }
    out = format!("{}\n", out.trim_end_matches('\n'));
    fs::write(output_path, out)?;
    Ok(())
}
