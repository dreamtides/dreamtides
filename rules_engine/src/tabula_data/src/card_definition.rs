use ability_data::ability::Ability;
use core_data::card_property_data::Rarity;
use core_data::card_types::{CardSubtype, CardType};
use core_data::display_types::SpriteAddress;
use core_data::identifiers::BaseCardId;
use core_data::numerics::{Energy, Spark};

/// Describes the rules for a card.
///
/// A card definition includes a "base card" (the rules from the tabula database
/// for a card) as well as zero or more card modifications.
///
/// Card definitions are stored in two different places: base card definitions
/// exist directly in the tabula database, while modified card definitions exist
/// in the quest state.
#[derive(Debug, Clone)]
pub struct CardDefinition {
    /// The base card that this card definition is based on.
    pub base_card_id: BaseCardId,

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

    /// Prompts to display for this card in the currently-active language,
    /// formatted for display.
    pub displayed_prompts: Vec<String>,

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
