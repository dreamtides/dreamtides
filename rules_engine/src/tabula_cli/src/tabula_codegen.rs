use std::fs;

use anyhow::Result;
use convert_case::{Case, Casing};
use tabula::tabula::Tabula;

pub fn generate_string_ids(tabula: &Tabula, output_path: &str) -> Result<()> {
    let mut out = String::new();
    out.push_str("use tabula::localized_strings::StringId;\n");
    out.push_str("use uuid::uuid;\n\n");

    for row in &tabula.strings.table.0 {
        out.push_str(&format!("/// {}\n", row.description.replace('\n', " ").trim()));
        let const_declaration = format!(
            "pub const {}: StringId = StringId(uuid!(\"{}\"));",
            row.name.to_case(Case::UpperSnake),
            row.id.0
        );
        if const_declaration.len() > 100 {
            out.push_str(&format!(
                "pub const {}: StringId =\n    StringId(uuid!(\"{}\"));\n\n",
                row.name.to_case(Case::UpperSnake),
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
