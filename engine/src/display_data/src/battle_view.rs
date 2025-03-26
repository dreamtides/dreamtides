use core_data::identifiers::BattleId;
use core_data::numerics::{Energy, Points, Spark};
use masonry::flex_node::FlexNode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::card_view::CardView;

/// Represents the visual state of an ongoing dream battle
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BattleView {
    /// Unique identifier for this dream battle
    pub id: BattleId,

    /// Player who is operating the client
    pub user: PlayerView,

    /// Opponent of user
    pub enemy: PlayerView,

    /// Visual state of cards in the game
    pub cards: Vec<CardView>,

    /// Describes the status of the game, e.g. which phase & step the game is in
    pub status_description: String,

    /// UI to display to the player.
    pub interface: InterfaceView,
}

/// User interaction options
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceView {
    /// Content to display on top of all other game UI.
    pub screen_overlay: Option<FlexNode>,

    /// Label for the primary action button, if one should be shown.
    pub primary_action_button: Option<String>,
}

/// Button to perform some game action
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ButtonView {
    pub label: String,
    pub kind: ButtonKind,
}

/// Controls color for buttons
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ButtonKind {
    /// Emphasized button, primary game action
    Primary,

    /// Deemphasized button, additional game actions
    Default,
}

/// Represents the visual state of a player in a game
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayerView {
    /// Current score total
    pub score: Points,

    /// Can this player currently take a game action?
    pub can_act: bool,

    /// Energy available to this player
    pub energy: Energy,

    /// Energy produced by this player at the start of the current turn
    pub produced_energy: Energy,

    /// Total spark for this player
    pub total_spark: Spark,
}
