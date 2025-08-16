use core_data::identifiers::BaseCardId;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDefinition {
    /// Identifies this card definition.
    pub id: BaseCardId,

    /// Name of this card (U.S. English).
    pub name_en_us: String,

    /// Energy cost of this card, if any.
    pub cost: Option<String>,

    /// Rules text of this card (U.S. English).
    pub rules_text_en_us: String,

    /// Type of this card.
    pub card_type: CardType,

    /// Subtype of this card (U.S. English).
    pub subtype_en_us: Option<String>,

    /// Whether this card is fast.
    pub fast: bool,

    /// Spark value of this card, if any.
    pub spark: Option<String>,

    /// Rarity of this card.
    pub rarity: Rarity,

    /// Identifies this card's image in the game's assets.
    pub image_number: String,
}
