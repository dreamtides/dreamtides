use action_data::game_action_data::GameAction;
use core_data::identifiers::{QuestId, SiteId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::battle_view::{ButtonView, InterfaceView};
use crate::card_view::CardView;

/// Represents the visual state of a dream quest
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct QuestView {
    /// Unique identifier for this quest
    pub id: QuestId,

    /// Cards which are currently being displayed
    pub cards: Vec<CardView>,

    /// UI to display to the player.
    pub interface: InterfaceView,

    /// Sites which can be visited within the currently-active dreamscape.
    pub sites: Vec<SiteView>,

    /// State for the 'tempting offer' view.
    pub tempting_offer: Option<TemptingOfferView>,
}

/// Represents the visual state of a site within a dreamscape
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SiteView {
    /// Unique identifier for this site
    pub id: SiteId,

    /// Icon representing this site
    pub icon: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TemptingOfferView {
    /// Action to perform when the close button is clicked.
    pub close_button: Option<CloseButtonView>,

    /// Offers to display to the player.
    pub actions: Vec<TemptingOfferAction>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CloseButtonView {
    /// Action to perform when the close button is clicked.
    pub action: GameAction,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TemptingOfferAction {
    /// Identifier for this offer
    pub number: TemptingOfferNumber,

    /// Button to display to accept this offer.
    pub button: ButtonView,
}

/// Identifier for a tempting offer.
#[derive(
    Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize, JsonSchema, Ord, PartialOrd,
)]
pub struct TemptingOfferNumber(pub u32);
