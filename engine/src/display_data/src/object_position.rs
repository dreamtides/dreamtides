use serde::{Deserialize, Serialize};
use specta::Type;

/// Represents the position of some object in the UI
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ObjectPosition {
    /// Position category
    pub position: Position,
    /// Sorting key, determines order within the position
    pub sorting_key: u32,
    /// Sub-key, used to break ties in sorting
    pub sorting_sub_key: u32,
}

impl Default for ObjectPosition {
    fn default() -> Self {
        Self { position: Position::Default, sorting_key: 0, sorting_sub_key: 0 }
    }
}

/// Possible types of display positions
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Ord, PartialOrd, Type)]
#[serde(rename_all = "camelCase")]
pub enum Position {
    /// Object position used in interface elements like the deck viewer which
    /// don't rely on game positioning.
    Default,

    /// Object is not visible.
    Offscreen,

    /// Object is prominently revealed, being shown at a large size after
    /// being played.
    Played,

    /// Object is on the stack
    Stack,

    /// Object is in a player's hand
    UserHand,
    EnemyHand,

    /// Object is in a player's deck
    UserDeck,
    EnemyDeck,

    /// Object is in a player's void
    UserVoid,
    EnemyVoid,

    /// Object is in this player's banished zone
    UserBanished,
    EnemyBanished,

    /// Object is on the battlefield
    UserBattlefield,
    EnemyBattlefield,

    /// Object is being displayed in a card browser, e.g. to select from a list
    /// of cards while searching
    Browser,

    /// Object is being displayed in a list of cards available to select in a
    /// card selector.
    CardSelectionChoices,

    /// Object is being displayed in a location for picking its relative order
    CardOrderLocationUnordered,
    CardOrderLocationDeck,
    CardOrderLocationDeckBottom,
    CardOrderLocationVoid,

    /// Object has just been revealed to this viewer
    Revealed,

    /// Object is in a temporary holding space for cards in hand while resolving
    /// some other 'play card' ability.
    HandStorage,
}
