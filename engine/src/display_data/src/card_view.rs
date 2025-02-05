use core_data::identifiers::CardId;
use core_data::numerics::{Energy, Spark};
use core_data::types::{CardFacing, Url};
use masonry::flex_node::FlexNode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::object_position::ObjectPosition;

/// Represents the visual state of a card or ability in a game
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardView {
    /// Identifier for this card
    pub id: CardId,

    /// Position of this card in the UI
    pub position: ObjectPosition,

    /// Card back image
    pub card_back: Url,

    /// If this card is revealed to the viewer, contains information on the
    /// revealed face of the card.
    pub revealed: Option<RevealedCardView>,

    /// True if this card is in a hidden zone but known to one or more opponents
    pub revealed_to_opponents: bool,

    /// Face up/face down state for this card
    pub card_facing: CardFacing,

    /// Optionally, a position at which to create this card.
    ///
    /// If this card does not already exist, it will be created at this position
    /// before being animated to [Self::position].
    pub create_position: Option<ObjectPosition>,

    /// Optionally, a position at which to destroy this card.
    ///
    /// If provided, the card will be animated to this position before being
    /// destroyed.
    pub destroy_position: Option<ObjectPosition>,
}

/// Visual state of a revealed card
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RevealedCardView {
    /// Image URL for this card
    pub image: DisplayImage,

    /// Name of this card
    pub name: String,

    /// Cost of this card
    pub cost: Energy,

    /// Spark value for this card
    pub spark: Option<Spark>,

    /// Type or subtype of this card
    pub card_type: String,

    /// Rules text to display for this car
    pub rules_text: String,

    /// Visual status of this card
    pub status: Option<RevealedCardStatus>,

    /// Frame to display for this card
    pub frame: CardFrame,

    /// Additional help text about this card, describing its abilities.
    pub supplemental_card_info: Option<FlexNode>,

    /// True if this card can be played during the opponent's turn
    pub is_fast: bool,

    /// True if this card can currently be played.
    pub can_play: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DisplayImage {
    /// Image URL for this card
    pub image: Url,

    /// X offset position of this image
    pub image_offset_x: Option<u32>,

    /// Y offset position of this image
    pub image_offset_y: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum RevealedCardStatus {
    Selected,
    CanSelect,
    CanPlay,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CardFrame {
    Character,
    Event,
}
