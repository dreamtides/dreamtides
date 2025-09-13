use core_data::identifiers::QuestId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::battle_view::InterfaceView;
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
}
