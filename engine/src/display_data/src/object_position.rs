use action_data::battle_action::CardOrderSelectionTarget;
use core_data::identifiers::CardId;
use core_data::types::PlayerName;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents the position of some object in the UI
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
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
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum Position {
    /// Object position used in interface elements like the deck viewer which
    /// don't rely on game positioning.
    Default,

    /// Object is not visible.
    Offscreen,

    /// Object is on the stack, typically used by cards which were just played.
    ///
    /// There are three types of stacks. By default, cards display at a large
    /// display size, blocking the view of the battlefield. However, if any
    /// cards are present on the stack which target a character on the
    /// battlefield, the cards are displayed at a small size in the user or
    /// enemy stack position in order to enable viewing & selecting targets
    /// appropriately. In that case the stack of whichever player first played a
    /// card is used.
    OnStack(StackType),

    /// Position for cards to be shown to the user immediately after they're
    /// drawn.
    Drawn,

    /// Object is in a player's hand
    InHand(PlayerName),

    /// Object is on top of a player's deck
    OnTopOfDeck(PlayerName),

    /// Object is shuffled into a player's deck
    InDeck(PlayerName),

    /// Object is in a player's void
    InVoid(PlayerName),

    /// Object is in this player's banished zone
    InBanished(PlayerName),

    /// Object is on the battlefield
    OnBattlefield(PlayerName),

    /// Object is in a player's status zone
    InPlayerStatus(PlayerName),

    /// Object is being displayed in a card browser, e.g. to select from a list
    /// of cards while searching
    Browser,

    /// Object is being displayed in a selector to determine the order of cards,
    /// e.g. when resolving the "forsee" effect.
    CardOrderSelector(CardOrderSelectionTarget),

    /// Object is in a temporary holding space for cards in hand while resolving
    /// some other 'play card' ability.
    HandStorage,

    /// Object is in the dreamwell for a player (usually off-screen).
    InDreamwell(PlayerName),

    /// Object is in a position to display itself as part of a dreamwell
    /// activation.
    DreamwellActivation,

    /// Object is hidden within a card
    HiddenWithinCard(CardId),

    /// Object describes a game modifier or ongoing game effect
    GameModifier,

    /// Object is in the on-screen storage area, used to hold objects at a small
    /// size when they're not being focused on, e.g. when the user hides a
    /// card browser to get a better view of the battlefield.
    OnScreenStorage,
}

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum StackType {
    Default,
    User,
    Enemy,
}
