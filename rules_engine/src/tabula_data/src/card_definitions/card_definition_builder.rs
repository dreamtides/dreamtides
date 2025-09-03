use std::collections::BTreeMap;

use ability_data::ability::{Ability, DisplayedAbility};
use core_data::card_types::CardSubtype;
use core_data::display_types::SpriteAddress;
use core_data::identifiers::{BaseCardId, DreamwellCardId};
use core_data::initialization_error::{ErrorCode, InitializationError};
use core_data::numerics::{Energy, Spark};

use crate::card_definitions::base_card_definition_raw::BaseCardDefinitionRaw;
use crate::card_definitions::base_card_definition_type::BaseCardDefinitionType;
use crate::card_definitions::card_definition::CardDefinition;
use crate::card_definitions::dreamwell_card_definition::{
    DreamwellCardDefinition, DreamwellCardDefinitionRaw,
};
use crate::localized_strings::LanguageId;
use crate::tabula::TabulaBuildContext;
use crate::tabula_table::Table;

struct CommonCardFields {
    displayed_name: String,
    displayed_rules_text: String,
    displayed_prompts: Vec<String>,
    image: SpriteAddress,
    abilities: Vec<Ability>,
    displayed_abilities: Vec<DisplayedAbility>,
}

fn common_fields<T: BaseCardDefinitionType>(
    sheet_name: &str,
    context: &TabulaBuildContext,
    row_index: usize,
    row: &T,
    errors: &mut Vec<InitializationError>,
) -> Option<CommonCardFields> {
    let displayed_name = match context.current_language {
        LanguageId::EnglishUnitedStates => row.name_en_us().to_string(),
    };
    let displayed_rules_text = match context.current_language {
        LanguageId::EnglishUnitedStates => row.rules_text_en_us().to_string(),
    };
    let displayed_prompts = row.prompts_en_us().map(|s| vec![s.to_string()]).unwrap_or_default();
    let image = SpriteAddress::new(format!(
        "Assets/ThirdParty/GameAssets/CardImages/{}/shutterstock_{}.png",
        row.image_directory(),
        row.image_number()
    ));

    let Some(abilities) = row.abilities_ref() else {
        let mut ierr = InitializationError::with_details(
            ErrorCode::AbilitiesNotPresent,
            "Abilities not present on card definition",
            "Please run tabula_cli to populate this field",
        );
        ierr.tabula_sheet = Some(sheet_name.to_string());
        ierr.tabula_row = Some(row_index);
        errors.push(ierr);
        return None;
    };

    let Some(displayed_abilities) = row.displayed_abilities_ref() else {
        let mut ierr = InitializationError::with_details(
            ErrorCode::AbilitiesNotPresent,
            "Abilities not present on card definition",
            "Please run tabula_cli to populate this field",
        );
        ierr.tabula_sheet = Some(sheet_name.to_string());
        ierr.tabula_row = Some(row_index);
        errors.push(ierr);
        return None;
    };

    Some(CommonCardFields {
        displayed_name,
        displayed_rules_text,
        displayed_prompts,
        image,
        abilities: abilities.clone(),
        displayed_abilities: displayed_abilities.clone(),
    })
}

pub fn build_base_cards(
    sheet_name: &str,
    context: &TabulaBuildContext,
    table: &Table<BaseCardId, BaseCardDefinitionRaw>,
) -> Result<BTreeMap<BaseCardId, CardDefinition>, Vec<InitializationError>> {
    let mut errors: Vec<InitializationError> = Vec::new();
    let mut out: BTreeMap<BaseCardId, CardDefinition> = BTreeMap::new();
    for (row_index, row) in table.as_slice().iter().enumerate() {
        let energy_cost: Option<Energy> = match &row.energy_cost {
            Some(s) => match s.parse::<u32>() {
                Ok(v) => Some(Energy(v)),
                Err(_) => {
                    if s == "*" {
                        None
                    } else {
                        let mut ierr = InitializationError::with_details(
                            ErrorCode::InvalidUnsignedInteger,
                            String::from("Invalid energy_cost"),
                            row.energy_cost.clone().unwrap_or_default(),
                        );
                        ierr.tabula_sheet = Some(sheet_name.to_string());
                        ierr.tabula_column = Some(String::from("energy_cost"));
                        ierr.tabula_row = Some(row_index);
                        errors.push(ierr);
                        None
                    }
                }
            },
            None => None,
        };

        let card_subtype = match &row.subtype {
            Some(s) => match CardSubtype::try_from(s.as_str()) {
                Ok(v) => Some(v),
                Err(_) => {
                    let mut ierr = InitializationError::with_details(
                        ErrorCode::InvalidCardSubtype,
                        String::from("Invalid card subtype"),
                        s.clone(),
                    );
                    ierr.tabula_sheet = Some(sheet_name.to_string());
                    ierr.tabula_column = Some(String::from("subtype"));
                    ierr.tabula_row = Some(row_index);
                    errors.push(ierr);
                    None
                }
            },
            None => None,
        };

        let spark: Option<Spark> = match &row.spark {
            Some(s) => match s.parse::<u32>() {
                Ok(v) => Some(Spark(v)),
                Err(_) => {
                    let mut ierr = InitializationError::with_details(
                        ErrorCode::InvalidUnsignedInteger,
                        String::from("Invalid spark"),
                        row.spark.clone().unwrap_or_default(),
                    );
                    ierr.tabula_sheet = Some(sheet_name.to_string());
                    ierr.tabula_column = Some(String::from("spark"));
                    ierr.tabula_row = Some(row_index);
                    errors.push(ierr);
                    None
                }
            },
            None => None,
        };
        let Some(CommonCardFields {
            displayed_name,
            displayed_rules_text,
            displayed_prompts,
            image,
            abilities,
            displayed_abilities,
        }) = common_fields(sheet_name, context, row_index, row, &mut errors)
        else {
            continue;
        };

        out.insert(row.id, CardDefinition {
            base_card_id: row.id,
            displayed_name,
            energy_cost,
            abilities,
            displayed_abilities,
            displayed_rules_text,
            displayed_prompts,
            card_type: row.card_type,
            card_subtype,
            is_fast: row.is_fast,
            spark,
            is_test_card: row.is_test_card,
            rarity: row.rarity.clone(),
            image,
        });
    }

    if errors.is_empty() { Ok(out) } else { Err(errors) }
}

pub fn build_dreamwell_cards(
    sheet_name: &str,
    context: &TabulaBuildContext,
    table: &Table<DreamwellCardId, DreamwellCardDefinitionRaw>,
) -> Result<BTreeMap<DreamwellCardId, DreamwellCardDefinition>, Vec<InitializationError>> {
    let mut errors: Vec<InitializationError> = Vec::new();
    let mut out: BTreeMap<DreamwellCardId, DreamwellCardDefinition> = BTreeMap::new();
    for (row_index, row) in table.as_slice().iter().enumerate() {
        let Some(CommonCardFields {
            displayed_name,
            displayed_rules_text,
            displayed_prompts,
            image,
            abilities,
            displayed_abilities,
        }) = common_fields(sheet_name, context, row_index, row, &mut errors)
        else {
            continue;
        };

        let energy_produced = match row.energy_produced.parse::<u32>() {
            Ok(v) => Energy(v),
            Err(_) => {
                let mut ierr = InitializationError::with_details(
                    ErrorCode::InvalidUnsignedInteger,
                    String::from("Invalid energy_produced"),
                    row.energy_produced.clone(),
                );
                ierr.tabula_sheet = Some(sheet_name.to_string());
                ierr.tabula_column = Some(String::from("energy_produced"));
                ierr.tabula_row = Some(row_index);
                errors.push(ierr);
                continue;
            }
        };

        let phase = match &row.phase {
            Some(s) => match s.parse::<usize>() {
                Ok(v) => v,
                Err(_) => {
                    let mut ierr = InitializationError::with_details(
                        ErrorCode::InvalidUnsignedInteger,
                        String::from("Invalid phase"),
                        s.clone(),
                    );
                    ierr.tabula_sheet = Some(sheet_name.to_string());
                    ierr.tabula_column = Some(String::from("phase"));
                    ierr.tabula_row = Some(row_index);
                    errors.push(ierr);
                    continue;
                }
            },
            None => 0,
        };
        out.insert(row.id, DreamwellCardDefinition {
            base_card_id: row.id,
            displayed_name,
            energy_produced,
            abilities,
            displayed_abilities,
            displayed_rules_text,
            displayed_prompts,
            phase,
            is_test_card: row.is_test_card,
            image,
        });
    }

    if errors.is_empty() { Ok(out) } else { Err(errors) }
}
