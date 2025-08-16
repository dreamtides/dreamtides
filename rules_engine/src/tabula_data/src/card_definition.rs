use core_data::display_types::SpriteAddress;
use core_data::identifiers::BaseCardId;
use core_data::numerics::{Energy, Spark};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CardType {
    Event,
    Character,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Special,
}

/// Base card definition from the Tabula database.
///
/// This is the definition of a card that is used to create a card instance,
/// associated with a [BaseCardId]. Base cards can have various modifications
/// and upgrades applied to them, which are represented by the `CardIdentity`
/// and `CardDescriptor` types.
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

    /// Rules text of this card in the currently-active language, formatted for
    /// display.
    pub displayed_rules_text: String,

    /// Type of this card.
    pub card_type: CardType,

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

    /// Rarity of this card.
    pub rarity: Rarity,

    /// Image to display for this card.
    pub image: SpriteAddress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDefinitionRaw {
    pub id: BaseCardId,

    /// Name of this card (U.S. English).
    pub name_en_us: String,

    /// Energy cost of this card, if any.
    pub energy_cost: Option<String>,

    /// Rules text of this card (U.S. English).
    pub rules_text_en_us: String,

    /// Type of this card.
    pub card_type: CardType,

    /// Subtype of this card (U.S. English).
    pub subtype_en_us: Option<String>,

    /// Whether this card is fast.
    pub is_fast: bool,

    /// Spark value of this card, if any.
    pub spark: Option<String>,

    /// Rarity of this card.
    pub rarity: Rarity,

    /// Identifies this card's image in the game's assets.
    pub image_number: String,
}
