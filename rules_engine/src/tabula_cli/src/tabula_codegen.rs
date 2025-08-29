use std::collections::{BTreeMap, BTreeSet, HashSet};
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
    out.push_str("use core_data::identifiers::{BaseCardId, DreamwellCardId};\n");
    out.push_str("use uuid::uuid;\n\n");

    let mut seen_names: HashSet<String> = HashSet::new();
    let mut base_const_names: Vec<String> = Vec::new();
    for row in tabula_raw.test_cards.as_slice() {
        let const_name = case_utils::cleaned_to_case(&row.name_en_us, Case::UpperSnake);
        if seen_names.contains(&const_name) {
            continue;
        }
        seen_names.insert(const_name.clone());
        base_const_names.push(const_name.clone());
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

    let mut dreamwell_const_names: Vec<String> = Vec::new();
    for row in tabula_raw.dreamwell_cards.as_slice() {
        if !row.is_test_card {
            continue;
        }
        let const_name = case_utils::cleaned_to_case(&row.name_en_us, Case::UpperSnake);
        if seen_names.contains(&const_name) {
            continue;
        }
        seen_names.insert(const_name.clone());
        dreamwell_const_names.push(const_name.clone());
        if !row.rules_text_en_us.trim().is_empty() {
            out.push_str(&format!("/// {}\n", row.rules_text_en_us.replace('\n', " ").trim()));
        }
        let const_declaration = format!(
            "pub const {}: DreamwellCardId = DreamwellCardId(uuid!(\"{}\"));",
            const_name, row.id.0
        );
        if const_declaration.len() > 100 {
            out.push_str(&format!(
                "pub const {}: DreamwellCardId =\n    DreamwellCardId(uuid!(\"{}\"));\n\n",
                const_name, row.id.0
            ));
        } else {
            out.push_str(&format!("{const_declaration}\n\n"));
        }
    }

    if !base_const_names.is_empty() {
        out.push_str("pub const ALL_TEST_CARD_IDS: &[BaseCardId] = &[\n");
        for name in base_const_names.iter() {
            out.push_str(&format!("    {name},\n"));
        }
        out.push_str("];\n\n");
    }

    if !dreamwell_const_names.is_empty() {
        out.push_str("pub const ALL_TEST_DREAMWELL_CARD_IDS: &[DreamwellCardId] = &[\n");
        for name in dreamwell_const_names.iter() {
            out.push_str(&format!("    {name},\n"));
        }
        out.push_str("];\n\n");
    }
    out = format!("{}\n", out.trim_end_matches('\n'));
    fs::write(output_path, out)?;
    Ok(())
}

pub fn generate_card_lists(tabula_raw: &TabulaRaw, output_path: &str) -> Result<()> {
    let mut out = String::new();
    out.push_str("use schemars::JsonSchema;\n");
    out.push_str("use serde::{Deserialize, Serialize};\n");
    out.push_str("use uuid::uuid;\n\n");

    let mut list_types: BTreeSet<String> = BTreeSet::new();
    for row in tabula_raw.card_lists.as_slice() {
        list_types.insert(row.list_type.clone());
    }
    if !list_types.is_empty() {
        out.push_str("use core_data::identifiers::{");
        out.push_str(&list_types.iter().cloned().collect::<Vec<_>>().join(", "));
        out.push_str("};\n\n");
    }

    let mut type_to_names: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut type_name_to_items: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();

    for row in tabula_raw.card_lists.as_slice() {
        let type_key = row.list_type.clone();
        let name_key = row.list_name.clone();

        if !type_to_names.contains_key(&type_key) {
            type_to_names.insert(type_key.clone(), Vec::new());
        }
        if !type_to_names.get(&type_key).unwrap().iter().any(|n| n == &name_key) {
            type_to_names.get_mut(&type_key).unwrap().push(name_key.clone());
        }

        if !type_name_to_items.contains_key(&type_key) {
            type_name_to_items.insert(type_key.clone(), BTreeMap::new());
        }
        if !type_name_to_items.get(&type_key).unwrap().contains_key(&name_key) {
            type_name_to_items.get_mut(&type_key).unwrap().insert(name_key.clone(), Vec::new());
        }
        let copies = row.copies as usize;
        for _ in 0..copies {
            type_name_to_items
                .get_mut(&type_key)
                .unwrap()
                .get_mut(&name_key)
                .unwrap()
                .push(row.card_id.to_string());
        }
    }

    for (type_key, names) in type_to_names.iter() {
        let enum_name = format!("{type_key}List");
        out.push_str("#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]\n");
        out.push_str(&format!("pub enum {enum_name} {{\n"));
        for name in names.iter() {
            let variant_name = case_utils::cleaned_to_case(name, Case::UpperCamel);
            out.push_str(&format!("    {variant_name},\n"));
        }
        out.push_str("}\n\n");

        let fn_name = case_utils::cleaned_to_case(&enum_name, Case::Snake);
        out.push_str(&format!("pub fn {fn_name}(list: {enum_name}) -> &'static [{type_key}] {{\n"));
        out.push_str("    match list {\n");
        for name in names.iter() {
            let variant_name = case_utils::cleaned_to_case(name, Case::UpperCamel);
            let const_name = case_utils::cleaned_to_case(name, Case::UpperSnake);
            out.push_str(&format!("        {enum_name}::{variant_name} => {const_name},\n"));
        }
        out.push_str("    }\n}\n\n");
    }

    for (type_key, name_to_items) in type_name_to_items.iter() {
        for (name_key, items) in name_to_items.iter() {
            let const_name = case_utils::cleaned_to_case(name_key, Case::UpperSnake);
            out.push_str(&format!("pub const {const_name}: &[{type_key}] = &[\n"));
            for id_str in items.iter() {
                out.push_str(&format!("    {type_key}(uuid!(\"{id_str}\")),\n"));
            }
            out.push_str("];\n\n");
        }
    }

    out = format!("{}\n", out.trim_end_matches('\n'));
    fs::write(output_path, out)?;
    Ok(())
}
