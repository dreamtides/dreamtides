use std::fs;

use anyhow::Result;
use convert_case::{Case, Casing};
use tabula::tabula::Tabula;

pub fn generate_string_ids(tabula: &Tabula, output_path: &str) -> Result<()> {
    let mut out = String::new();
    out.push_str("use uuid::{Uuid, uuid};\n\n");
    for row in &tabula.strings.table.0 {
        out.push_str(&format!("/// {}\n", row.description.replace('\n', " ").trim()));
        out.push_str(&format!(
            "pub const {}: Uuid = uuid!(\"{}\");\n\n",
            row.name.to_case(Case::UpperSnake),
            row.id.0
        ));
    }
    fs::write(output_path, out)?;
    Ok(())
}
