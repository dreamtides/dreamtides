use ability_data::ability::{Ability, DisplayedAbility};
use core_data::display_types::SpriteAddress;
use core_data::identifiers::DreamwellCardId;
use core_data::numerics::Energy;
use serde::{Deserialize, Serialize};

pub type DreamwellCardPhase = usize;

/// Describes the rules for a dreamwell card.
///
/// Dreamwell cards are special cards drawn during battles to increase the
/// amount of energy available to a player and provide random effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamwellCardDefinition {
    /// The base card that this card definition is based on.
    pub base_card_id: DreamwellCardId,

    /// Name of this card in the currently-active language.
    pub displayed_name: String,

    /// Energy production added by this card when it resolves.
    pub energy_produced: Energy,

    /// Abilities of this card.
    pub abilities: Vec<Ability>,

    /// Abilities of this card, formatted for display.
    pub displayed_abilities: Vec<DisplayedAbility>,

    /// Rules text of this card in the currently-active language, formatted for
    /// display.
    pub displayed_rules_text: String,

    /// Prompts to display for this card in the currently-active language,
    /// formatted for display.
    pub displayed_prompts: Vec<String>,

    /// "Phase" of this card, controls the order in which it appears in the
    /// shuffled dreamwell deck. Cards with a lower phase number appear first in
    /// the randomized order.
    pub phase: DreamwellCardPhase,

    /// Image to display for this card.
    pub image: SpriteAddress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamwellCardDefinitionRaw {
    /// Identifies this card.
    pub id: DreamwellCardId,

    /// Name of this card (U.S. English).
    pub name_en_us: String,

    /// Energy produced by this card when it resolves.
    pub energy_produced: String,

    /// Rules text of this card (U.S. English).
    pub rules_text_en_us: String,

    /// Prompts for this card (U.S. English).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts_en_us: Option<String>,

    /// Phase of this card, used to determine the order in which it appears in
    /// the shuffled dreamwell deck.
    pub phase: Option<String>,

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

    /// Identifies this card's image in the game's assets.
    pub image_number: String,
}
