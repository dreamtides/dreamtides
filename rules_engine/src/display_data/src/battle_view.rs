use action_data::game_action::GameAction;
use core_data::display_types::Milliseconds;
use core_data::identifiers::BattleId;
use core_data::numerics::{Energy, Points, Spark};
use masonry::flex_node::FlexNode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::card_view::{CardPreviewView, CardView};

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

    /// UI to display to the player.
    pub interface: InterfaceView,
}

/// Preview of a potential future state of a battle, shown e.g. in response to a
/// card being selected to be played.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BattlePreviewView {
    pub user: PlayerPreviewView,
    pub enemy: PlayerPreviewView,
    pub cards: Vec<CardPreviewView>,
    pub preview_message: Option<FlexNode>,
}

/// User interaction options
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceView {
    /// Content to display on top of all other game UI.
    pub screen_overlay: Option<FlexNode>,

    /// Primary action button, used for confirming selections and ending the
    /// turn. None indicates no button should be shown.
    pub primary_action_button: Option<PrimaryActionButtonView>,

    /// Options for display of the card order selector
    pub card_order_selector: Option<CardOrderSelectorView>,

    /// Button most often used for toggling the visibility of card browsers.
    pub bottom_right_button: Option<ButtonView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CardOrderSelectorView {
    /// Include the user's deck as a card drop target
    pub include_deck: bool,

    /// Include the user's void as a card drop target
    pub include_void: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PrimaryActionButtonView {
    pub label: String,
    pub action: GameAction,

    /// If provided, when the button is not visible, the button will wait for
    /// this duration after the last "update" before appearing. If this is None
    /// the button will display immediately.
    pub show_on_idle_duration: Option<Milliseconds>,
}

/// Button to perform some game action
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ButtonView {
    pub label: String,
    pub action: GameAction,
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

/// Preview of a potential future state of a player
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayerPreviewView {
    /// New score total
    pub score: Option<Points>,

    /// New energy available to this player
    pub energy: Option<Energy>,

    /// New energy produced total
    pub produced_energy: Option<Energy>,

    /// New total spark for this player
    pub total_spark: Option<Spark>,
}
