use std::collections::BTreeMap;

use ability_data::ability::Ability;
use core_data::card_properties::Rarity;
use core_data::card_types::{CardSubtype, CardType};
use core_data::display_types::SpriteAddress;
use core_data::identifiers::BaseCardId;
use core_data::initialization_error::{ErrorCode, InitializationError};
use core_data::numerics::{Energy, Spark};
use serde::{Deserialize, Serialize};

use crate::localized_strings::LanguageId;
use crate::tabula::TabulaBuildContext;
use crate::tabula_table::Table;

/// Base card definition from the Tabula database.
///
/// This is the definition of a card that is used to create a card instance,
/// associated with a [BaseCardId]. Base cards can have various modifications
/// and upgrades applied to them, which are represented by the `CardIdentity`
/// and `CardDescriptor` types.
#[derive(Debug, Clone)]
pub struct BaseCardDefinition {
    /// Identifies this card definition.
    pub id: BaseCardId,

    /// Name of this card in the currently-active language.
    pub displayed_name: String,

    /// Base energy cost of this card, if any.
    ///
    /// A card with a variable cost (e.g. modal cards) will have no energy cost
    /// specified here.
    pub energy_cost: Option<Energy>,

    /// Abilities of this card.
    pub abilities: Vec<Ability>,

    /// Rules text of this card in the currently-active language, formatted for
    /// display.
    pub displayed_rules_text: String,

    /// Type of this card.
    pub card_type: CardType,

    /// Subtype of this card, if any.
    pub card_subtype: Option<CardSubtype>,

    /// Whether this card is fast.
    ///
    /// Fast cards can be played "in response" to the opponent playing a card,
    /// or at the end of the opponent's turn.
    pub is_fast: bool,

    /// Base spark value of this card, if any.
    ///
    /// A character card with a spark defined by a static ability value will
    /// have no spark value specified here.
    pub spark: Option<Spark>,

    /// Rarity of this card, if any.
    pub rarity: Option<Rarity>,

    /// Image to display for this card.
    pub image: SpriteAddress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseCardDefinitionRaw {
    pub id: BaseCardId,

    /// Name of this card (U.S. English).
    pub name_en_us: String,

    /// Energy cost of this card, if any.
    pub energy_cost: Option<String>,

    /// Rules text of this card (U.S. English).
    pub rules_text_en_us: String,

    /// Abilities of this card in serialized form.
    ///
    /// If not present here, the Tabula CLI will populate this field by parsing
    /// the English rules text.
    pub abilities: Option<Vec<Ability>>,

    /// Type of this card.
    pub card_type: CardType,

    /// Subtype of this card
    pub subtype: Option<String>,

    /// Whether this card is fast.
    pub is_fast: bool,

    /// Spark value of this card, if any.
    pub spark: Option<String>,

    /// Rarity of this card.
    pub rarity: Option<Rarity>,

    /// Identifies this card's image in the game's assets.
    pub image_number: String,
}

pub fn build(
    sheet_name: &str,
    context: &TabulaBuildContext,
    table: &Table<BaseCardId, BaseCardDefinitionRaw>,
) -> Result<BTreeMap<BaseCardId, BaseCardDefinition>, Vec<InitializationError>> {
    let mut errors: Vec<InitializationError> = Vec::new();
    let mut out: BTreeMap<BaseCardId, BaseCardDefinition> = BTreeMap::new();
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

        let card_subtype: Option<CardSubtype> = match &row.subtype {
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

        let displayed_name = match context.current_language {
            LanguageId::EnglishUnitedStates => row.name_en_us.clone(),
        };
        let displayed_rules_text = match context.current_language {
            LanguageId::EnglishUnitedStates => row.rules_text_en_us.clone(),
        };

        let image = SpriteAddress::new(format!(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_{}.png",
            row.image_number
        ));

        let Some(abilities) = &row.abilities else {
            let mut ierr = InitializationError::with_details(
                ErrorCode::AbilitiesNotPresent,
                "Abilities not present on card definition",
                "Please run tabula_cli to populate this field",
            );
            ierr.tabula_sheet = Some(sheet_name.to_string());
            ierr.tabula_row = Some(row_index);
            errors.push(ierr);
            continue;
        };

        out.insert(row.id, BaseCardDefinition {
            id: row.id,
            displayed_name,
            energy_cost,
            abilities: abilities.clone(),
            displayed_rules_text,
            card_type: row.card_type,
            card_subtype,
            is_fast: row.is_fast,
            spark,
            rarity: row.rarity.clone(),
            image,
        });
    }

    if errors.is_empty() { Ok(out) } else { Err(errors) }
}
