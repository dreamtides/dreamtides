use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use core_data::card_property_data::Rarity;
use core_data::card_types::CardType;
use core_data::identifiers::BaseCardId;
use parser::{ability_parser, displayed_ability_parser};
use tabula_cli::{ability_parsing, tabula_codegen};
use tabula_data::card_definitions::base_card_definition_raw::BaseCardDefinitionRaw;
use tabula_data::localized_strings::LanguageId;
use tabula_data::tabula::{self, TabulaBuildContext, TabulaRaw};
use tabula_data::tabula_table::from_vec;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "tabula_add_card", version, about = "Append a test card to tabula.json")]
struct Args {
    #[arg(long, value_name = "PATH", help = "Path to tabula.json file to update")]
    tabula_path: PathBuf,

    #[arg(long, value_name = "STRING", default_value = "New Test Card")]
    name: String,

    #[arg(long, value_name = "UUID", help = "Explicit card UUID; generated if omitted")]
    id: Option<Uuid>,

    #[arg(long, value_name = "STRING", default_value = " ")]
    text: String,

    #[arg(long, value_name = "STRING")]
    prompts: Option<String>,

    #[arg(long, value_name = "STRING", help = "Energy cost as string, e.g. 2")]
    cost: Option<String>,

    #[arg(long, value_name = "CARD_TYPE", default_value = "Event")]
    card_type: String,

    #[arg(long, value_name = "SUBTYPE")]
    subtype: Option<String>,

    #[arg(long, value_name = "BOOL", default_value_t = false)]
    is_fast: bool,

    #[arg(long, value_name = "STRING", help = "Spark value as string")]
    spark: Option<String>,

    #[arg(long, value_name = "RARITY")]
    rarity: Option<String>,

    #[arg(long, value_name = "STRING", default_value = "0")]
    image_number: String,

    #[arg(long, help = "Run code generation after adding the card")]
    codegen: bool,

    #[arg(long, value_name = "PATH", help = "Generate string_id.rs at the given path")]
    string_ids: Option<String>,

    #[arg(long, value_name = "PATH", help = "Generate test_card_id.rs at the given path")]
    test_card_ids: Option<String>,

    #[arg(long, value_name = "PATH", help = "Generate card_lists.rs at the given path")]
    card_lists: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let path = args.tabula_path;
    let data = fs::read_to_string(&path)
        .with_context(|| format!("failed to read tabula json at {}", path.display()))?;

    let mut raw: TabulaRaw = serde_json::from_str(&data)
        .with_context(|| format!("failed to parse tabula json at {}", path.display()))?;

    let new_id = BaseCardId(args.id.unwrap_or_else(Uuid::new_v4));
    let card_type = parse_card_type(&args.card_type)?;
    let rarity = match args.rarity.as_deref() {
        Some(s) => Some(parse_rarity(s)?),
        None => None,
    };

    let parsed_abilities = ability_parser::parse(&args.text).map_err(|errs| {
        anyhow::anyhow!(errs.into_iter().map(|e| e.format()).collect::<Vec<_>>().join("\n"))
    })?;
    let parsed_displayed = displayed_ability_parser::parse_with(&parsed_abilities, &args.text)
        .map_err(|errs| {
            anyhow::anyhow!(errs.into_iter().map(|e| e.format()).collect::<Vec<_>>().join("\n"))
        })?;

    let row = BaseCardDefinitionRaw {
        id: new_id,
        name_en_us: args.name,
        energy_cost: args.cost,
        rules_text_en_us: args.text,
        prompts_en_us: args.prompts,
        abilities: Some(parsed_abilities),
        displayed_abilities: Some(parsed_displayed),
        card_type,
        subtype: args.subtype,
        is_fast: args.is_fast,
        spark: args.spark,
        is_test_card: true,
        rarity,
        image_number: args.image_number,
    };

    let mut items = raw.cards.0;
    items.push(row);
    raw.cards = from_vec(items);

    let pretty = serde_json::to_string_pretty(&raw).context("failed to serialize tabula json")?;
    fs::write(&path, pretty)
        .with_context(|| format!("failed to write tabula json at {}", path.display()))?;

    if args.codegen {
        run_codegen(
            &mut raw,
            args.string_ids.as_deref(),
            args.test_card_ids.as_deref(),
            args.card_lists.as_deref(),
        )?;
    }

    Ok(())
}

fn run_codegen(
    raw: &mut TabulaRaw,
    string_ids: Option<&str>,
    test_card_ids: Option<&str>,
    card_lists: Option<&str>,
) -> Result<()> {
    ability_parsing::parse_all_abilities_for_raw_tabula(raw)?;

    let _ = tabula::build(
        &TabulaBuildContext { current_language: LanguageId::EnglishUnitedStates },
        raw,
    )
    .map_err(|errs| {
        anyhow::anyhow!(errs.into_iter().map(|e| e.format()).collect::<Vec<_>>().join("\n"))
    })?;

    if let Some(path) = string_ids {
        tabula_codegen::generate_string_ids(raw, path)?;
    }

    if let Some(path) = test_card_ids {
        tabula_codegen::generate_test_card_ids(raw, path)?;
    }

    if let Some(path) = card_lists {
        tabula_codegen::generate_card_lists(raw, path)?;
    }

    Ok(())
}

fn parse_card_type(s: &str) -> Result<CardType> {
    match s.to_lowercase().as_str() {
        "character" => Ok(CardType::Character),
        "event" => Ok(CardType::Event),
        "dreamsign" => Ok(CardType::Dreamsign),
        "dreamcaller" => Ok(CardType::Dreamcaller),
        "dreamwell" => Ok(CardType::Dreamwell),
        _ => Err(anyhow::anyhow!("unknown card type: {s}")),
    }
}

fn parse_rarity(s: &str) -> Result<Rarity> {
    match s.to_lowercase().as_str() {
        "common" => Ok(Rarity::Common),
        "uncommon" => Ok(Rarity::Uncommon),
        "rare" => Ok(Rarity::Rare),
        "special" => Ok(Rarity::Special),
        _ => Err(anyhow::anyhow!("unknown rarity: {s}")),
    }
}
