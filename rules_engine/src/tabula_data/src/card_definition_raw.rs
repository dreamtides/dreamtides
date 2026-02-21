use serde::{Deserialize, Deserializer};
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
    #[serde(default, deserialize_with = "deserialize_optional_i32")]
    pub spark: Option<i32>,
    /// Phase ordering for dreamwell cards.
    #[serde(default, deserialize_with = "deserialize_optional_i32")]
    pub phase: Option<i32>,
    /// Rules text describing card abilities.
    pub rules_text: Option<String>,
    /// Variable definitions for rules text placeholders.
    pub variables: Option<String>,
    /// Image asset identifier.
    pub image_number: Option<i64>,
    /// Card rarity (e.g., "Common", "Rare").
    pub rarity: Option<String>,
    /// Energy produced when this dreamwell card is drawn.
    #[serde(default, deserialize_with = "deserialize_optional_i32")]
    pub energy_produced: Option<i32>,
    /// Whether this card can be played at fast timing.
    pub is_fast: Option<bool>,
}

/// Custom deserializer that treats empty strings and "*" as None for optional
/// i32 fields.
fn deserialize_optional_i32<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<TomlValue>::deserialize(deserializer)?;
    match value {
        None => Ok(None),
        Some(TomlValue::Integer(i)) => Ok(Some(i as i32)),
        Some(TomlValue::String(s)) if s.is_empty() || s == "*" => Ok(None),
        Some(TomlValue::String(s)) => s.parse::<i32>().map(Some).map_err(serde::de::Error::custom),
        Some(other) => Err(serde::de::Error::custom(format!(
            "expected integer or empty string, got {other:?}"
        ))),
    }
}
