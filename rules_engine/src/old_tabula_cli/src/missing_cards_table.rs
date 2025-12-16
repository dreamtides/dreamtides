use std::fs::File;
use std::io::Write;
use std::path::Path;

use core_data::identifiers::BaseCardId;
use tabula_data::card_definitions::card_definition::CardDefinition;

pub fn write_missing_cards_html(
    path: impl AsRef<Path>,
    missing: &[(BaseCardId, CardDefinition)],
) -> anyhow::Result<()> {
    let mut out = String::new();
    out.push_str("<table>\n");
    out.push_str(
        "<tr><th>ID</th><th>Image</th><th>Name</th><th>Energy Cost</th><th>Rules Text</th><th>Prompts</th><th>Card Type</th><th>Subtype</th><th>Is Fast?</th><th>Spark</th><th>Rarity</th></tr>\n",
    );

    for (id, def) in missing {
        let energy = def.energy_cost.map(|e| e.0.to_string()).unwrap_or_default();
        let prompts = if def.displayed_prompts.is_empty() {
            String::new()
        } else {
            def.displayed_prompts.join("\\n")
        };
        let subtype = def.card_subtype.as_ref().map(|s| format!("{s:?}")).unwrap_or_default();
        let spark = def.spark.map(|s| s.0.to_string()).unwrap_or_default();
        let rarity = def.rarity.as_ref().map(|r| format!("{r:?}")).unwrap_or_default();

        out.push_str("<tr>");
        out.push_str(&format!("<td>{}</td>", id.0));
        out.push_str("<td></td>");
        out.push_str(&format!("<td>{}</td>", html_escape(&def.displayed_name)));
        out.push_str(&format!("<td>{energy}</td>"));
        out.push_str(&format!("<td>{}</td>", html_escape(&def.displayed_rules_text)));
        out.push_str(&format!("<td>{}</td>", html_escape(&prompts)));
        out.push_str(&format!("<td>{:?}</td>", def.card_type));
        out.push_str(&format!("<td>{subtype}</td>"));
        out.push_str(&format!("<td>{}</td>", def.is_fast));
        out.push_str(&format!("<td>{spark}</td>"));
        out.push_str(&format!("<td>{rarity}</td>"));
        out.push_str("</tr>\n");
    }

    out.push_str("</table>\n");

    let mut file = File::create(path)?;
    file.write_all(out.as_bytes())?;
    Ok(())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
