use std::path::Path;

use ability_data::ability::Ability;
use core_data::card_property_data::Rarity;
use core_data::card_types::{CardSubtype, CardType};
use core_data::display_types::SpriteAddress;
use core_data::identifiers::{BaseCardId, DreamwellCardId};
use core_data::numerics::{Energy, Spark};
use uuid::Uuid;

use crate::card_definition::CardDefinition;
use crate::card_definition_raw::CardDefinitionRaw;
use crate::dreamwell_definition::{DreamwellCardDefinition, DreamwellCardPhase};
use crate::tabula_error::TabulaError;

/// Builds a [CardDefinition] from raw TOML data and pre-parsed abilities.
pub fn build_card(
    raw: &CardDefinitionRaw,
    abilities: Vec<Ability>,
    file: &Path,
) -> Result<CardDefinition, TabulaError> {
    let card_id = require_field(raw.id, "id", file, None)?;
    let name = require_field(raw.name.as_ref(), "name", file, Some(card_id))?;
    let card_type_str = require_field(raw.card_type.as_ref(), "card-type", file, Some(card_id))?;
    let image_number = require_field(raw.image_number, "image-number", file, Some(card_id))?;

    Ok(CardDefinition {
        base_card_id: BaseCardId(card_id),
        displayed_name: name.clone(),
        energy_cost: parse_energy_cost(raw, file)?,
        abilities,
        displayed_rules_text: raw.rules_text.clone().unwrap_or_default(),
        displayed_prompts: raw.prompts.as_ref().map(|s| vec![s.clone()]).unwrap_or_default(),
        card_type: parse_card_type(card_type_str, file, Some(card_id))?,
        card_subtype: parse_card_subtype(raw, file, Some(card_id))?,
        is_fast: raw.is_fast.unwrap_or(false),
        spark: parse_spark(raw, file, Some(card_id))?,
        rarity: parse_rarity(raw, file, Some(card_id))?,
        image: build_sprite_address(image_number),
    })
}

/// Builds a [DreamwellCardDefinition] from raw TOML data and pre-parsed
/// abilities.
pub fn build_dreamwell(
    raw: &CardDefinitionRaw,
    abilities: Vec<Ability>,
    file: &Path,
) -> Result<DreamwellCardDefinition, TabulaError> {
    let card_id = require_field(raw.id, "id", file, None)?;
    let name = require_field(raw.name.as_ref(), "name", file, Some(card_id))?;
    let energy_produced =
        require_field(raw.energy_produced, "energy-produced", file, Some(card_id))?;
    let image_number = require_field(raw.image_number, "image-number", file, Some(card_id))?;

    Ok(DreamwellCardDefinition {
        base_card_id: DreamwellCardId(card_id),
        displayed_name: name.clone(),
        energy_produced: Energy(energy_produced as u32),
        abilities,
        displayed_rules_text: raw.rules_text.clone().unwrap_or_default(),
        displayed_prompts: raw.prompts.as_ref().map(|s| vec![s.clone()]).unwrap_or_default(),
        phase: parse_phase(raw, file, Some(card_id))?,
        image: build_dreamwell_sprite_address(image_number),
    })
}

fn require_field<T: Clone>(
    value: Option<T>,
    field: &'static str,
    file: &Path,
    card_id: Option<Uuid>,
) -> Result<T, TabulaError> {
    value.ok_or_else(|| TabulaError::MissingField { file: file.to_path_buf(), card_id, field })
}

fn parse_energy_cost(raw: &CardDefinitionRaw, file: &Path) -> Result<Option<Energy>, TabulaError> {
    let Some(value) = &raw.energy_cost else {
        return Ok(None);
    };

    if let Some(i) = value.as_integer() {
        return Ok(Some(Energy(i as u32)));
    }

    if let Some(s) = value.as_str() {
        if s == "*" {
            return Ok(None);
        }
        if let Ok(v) = s.parse::<u32>() {
            return Ok(Some(Energy(v)));
        }
    }

    Err(TabulaError::InvalidField {
        file: file.to_path_buf(),
        card_id: raw.id,
        field: "energy-cost",
        message: format!("expected integer or '*', got {value:?}"),
    })
}

fn parse_card_type(
    type_str: &str,
    file: &Path,
    card_id: Option<Uuid>,
) -> Result<CardType, TabulaError> {
    CardType::try_from(type_str).map_err(|_| TabulaError::InvalidField {
        file: file.to_path_buf(),
        card_id,
        field: "card-type",
        message: format!("unknown card type '{type_str}'"),
    })
}

fn parse_card_subtype(
    raw: &CardDefinitionRaw,
    file: &Path,
    card_id: Option<Uuid>,
) -> Result<Option<CardSubtype>, TabulaError> {
    let Some(subtype_str) = &raw.subtype else {
        return Ok(None);
    };

    if subtype_str.is_empty() {
        return Ok(None);
    }

    CardSubtype::try_from(subtype_str.as_str()).map(Some).map_err(|_| TabulaError::InvalidField {
        file: file.to_path_buf(),
        card_id,
        field: "subtype",
        message: format!("unknown card subtype '{subtype_str}'"),
    })
}

fn parse_spark(
    raw: &CardDefinitionRaw,
    file: &Path,
    card_id: Option<Uuid>,
) -> Result<Option<Spark>, TabulaError> {
    let Some(spark) = raw.spark else {
        return Ok(None);
    };

    if spark < 0 {
        return Err(TabulaError::InvalidField {
            file: file.to_path_buf(),
            card_id,
            field: "spark",
            message: format!("spark cannot be negative, got {spark}"),
        });
    }

    Ok(Some(Spark(spark as u32)))
}

fn parse_rarity(
    raw: &CardDefinitionRaw,
    file: &Path,
    card_id: Option<Uuid>,
) -> Result<Option<Rarity>, TabulaError> {
    let Some(rarity_str) = &raw.rarity else {
        return Ok(None);
    };

    Rarity::try_from(rarity_str.as_str()).map(Some).map_err(|_| TabulaError::InvalidField {
        file: file.to_path_buf(),
        card_id,
        field: "rarity",
        message: format!("unknown rarity '{rarity_str}'"),
    })
}

fn parse_phase(
    raw: &CardDefinitionRaw,
    file: &Path,
    card_id: Option<Uuid>,
) -> Result<DreamwellCardPhase, TabulaError> {
    let Some(phase) = raw.phase else {
        return Ok(0);
    };

    if phase < 0 {
        return Err(TabulaError::InvalidField {
            file: file.to_path_buf(),
            card_id,
            field: "phase",
            message: format!("phase cannot be negative, got {phase}"),
        });
    }

    Ok(phase as DreamwellCardPhase)
}

fn build_sprite_address(image_number: i64) -> SpriteAddress {
    SpriteAddress::new(format!(
        "Assets/ThirdParty/GameAssets/CardImages/Base/shutterstock_{image_number}.png"
    ))
}

fn build_dreamwell_sprite_address(image_number: i64) -> SpriteAddress {
    SpriteAddress::new(format!(
        "Assets/ThirdParty/GameAssets/CardImages/Dreamwell/shutterstock_{image_number}.png"
    ))
}
