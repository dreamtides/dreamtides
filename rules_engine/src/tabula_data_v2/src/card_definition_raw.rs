use serde::Deserialize;
use toml::Value as TomlValue;
use uuid::Uuid;

/// Unified raw card definition deserialized from TOML files.
///
/// Contains the superset of all fields from cards, dreamwell cards, and test
/// cards. All fields are optional; validation happens during the build phase.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CardDefinitionRaw {
    /// Unique identifier for this card.
    pub id: Option<Uuid>,
    /// Display name of the card.
    pub name: Option<String>,
    /// Card type (e.g., "Character", "Event").
    pub card_type: Option<String>,
    /// Card subtype (e.g., "Musician", "Scholar").
    pub subtype: Option<String>,
    /// Energy cost to play. Can be an integer or "*" for variable cost.
    pub energy_cost: Option<TomlValue>,
    /// Spark value for characters (victory points generated).
    pub spark: Option<i32>,
    /// Phase ordering for dreamwell cards.
    pub phase: Option<i32>,
    /// Rules text describing card abilities.
    pub rules_text: Option<String>,
    /// Prompt text shown when the card requires player input.
    pub prompts: Option<String>,
    /// Variable definitions for rules text placeholders.
    pub variables: Option<String>,
    /// Image asset identifier.
    pub image_number: Option<i64>,
    /// Card rarity (e.g., "Common", "Rare").
    pub rarity: Option<String>,
    /// Whether the card art is owned/licensed.
    pub art_owned: Option<bool>,
    /// Card number in the set.
    pub card_number: Option<i32>,
    /// Energy produced when this dreamwell card is drawn.
    pub energy_produced: Option<i32>,
    /// Whether this card can be played at fast timing.
    pub is_fast: Option<bool>,
}
