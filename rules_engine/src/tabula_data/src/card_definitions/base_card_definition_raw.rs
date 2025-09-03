use ability_data::ability::{Ability, DisplayedAbility};
use core_data::card_property_data::Rarity;
use core_data::card_types::CardType;
use core_data::identifiers::BaseCardId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseCardDefinitionRaw {
    pub id: BaseCardId,

    /// Name of this card (U.S. English).
    pub name_en_us: String,

    /// Energy cost of this card, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy_cost: Option<String>,

    /// Rules text of this card (U.S. English).
    pub rules_text_en_us: String,

    /// Prompts for this card (U.S. English).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts_en_us: Option<String>,

    /// Abilities of this card in serialized form.
    ///
    /// If not present here, the Tabula CLI will populate this field by parsing
    /// the English rules text.
    pub abilities: Option<Vec<Ability>>,

    /// Abilities of this card in serialized form, formatted for display.
    ///
    /// If not present here, the Tabula CLI will populate this field by parsing
    /// the English rules text.
    pub displayed_abilities: Option<Vec<DisplayedAbility>>,

    /// Type of this card.
    pub card_type: CardType,

    /// Subtype of this card
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,

    /// Whether this card is fast.
    pub is_fast: bool,

    /// Spark value of this card, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spark: Option<String>,

    /// Whether this card is a test card.
    #[serde(default)]
    pub is_test_card: bool,

    /// Rarity of this card.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<Rarity>,

    /// Identifies this card's image in the game's assets.
    pub image_number: String,
}
