use action_data::game_action_data::GameAction;
use battle_state::battle::card_id::CardId;
use core_data::display_color::DisplayColor;
use core_data::display_types::{AudioClipAddress, PrefabAddress, ProjectileAddress, SpriteAddress};
use core_data::numerics::{Energy, Spark};
use core_data::types::CardFacing;
use masonry::flex_node::FlexNode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::battle_view::BattlePreviewView;
use crate::object_position::ObjectPosition;

/// Represents the visual state of a card or ability in a game
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardView {
    /// Identifier for this card
    pub id: CardId,

    /// Position of this card in the UI
    pub position: ObjectPosition,

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

    /// Represents the general category of card being displayed.
    pub prefab: CardPrefab,
}

/// Preview of a potential future state of a card.
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardPreviewView {
    pub card_id: CardId,

    /// New cost value for this card
    pub cost: Option<Energy>,

    /// New spark value for this card
    pub spark: Option<Spark>,

    /// Icon to display over this card on the battlefield
    pub battlefield_icon: Option<String>,

    /// Color of the battlefield icon
    pub battlefield_icon_color: Option<DisplayColor>,
}

/// Visual state of a revealed card
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RevealedCardView {
    /// Image for this card
    pub image: DisplayImage,

    /// Name of this card
    pub name: String,

    /// Cost of this card
    pub cost: Option<Energy>,

    /// Energy produced by this card
    pub produced: Option<Energy>,

    /// Spark value for this card
    pub spark: Option<Spark>,

    /// Type or subtype of this card
    pub card_type: String,

    /// Rules text to display for this car
    pub rules_text: String,

    /// Outline color of this card
    pub outline_color: Option<DisplayColor>,

    /// True if this card can be played during the opponent's turn
    pub is_fast: bool,

    /// Actions available for this card
    pub actions: CardActions,

    /// Special effects to display for this card
    pub effects: CardEffects,

    /// Data providing supplemental information about this card on long
    /// press/hover.
    pub info_zoom_data: Option<InfoZoomData>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum DisplayImage {
    Sprite(SpriteAddress),
    Prefab(PrefabAddress),
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardActions {
    /// True if this card can currently be played from hand.
    pub can_play: bool,

    /// True if this card can currently be dragged within a Card Order Selector.
    pub can_select_order: bool,

    /// Sound to play when this card is played.
    pub on_play_sound: Option<AudioClipAddress>,

    /// Action to perform when this card is clicked.
    pub on_click: Option<GameAction>,

    /// Preview of the battle state after this card is played.
    pub play_effect_preview: Option<BattlePreviewView>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardEffects {
    /// Projectile to display as a trail behind this card.
    pub card_trail: Option<ProjectileAddress>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InfoZoomData {
    /// Additional help text about this card, describing its abilities.
    pub supplemental_card_info: Option<FlexNode>,

    /// Icons to display on other cards during info zoom, e.g. indicating
    /// targets.
    pub icons: Vec<InfoZoomIcon>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InfoZoomIcon {
    pub card_id: CardId,
    pub icon: String,
    pub color: DisplayColor,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CardPrefab {
    Character,
    Event,
    Identity,
    Token,
    Dreamwell,
    Enemy,
    Dreamsign,
}
