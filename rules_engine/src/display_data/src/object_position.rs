use battle_state::actions::battle_actions::CardOrderSelectionTarget;
use battle_state::battle::card_id::CardId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::battle_view::DisplayPlayer;

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
    /// There are four types of stacks. By default, cards display at a large
    /// display size, blocking the view of the battlefield. However, if any
    /// cards are present on the stack which target a character on the
    /// battlefield, the cards are displayed at a smaller size in order to
    /// enable viewing & selecting targets appropriately, based on the set of
    /// cards which are current or eligible targets.
    OnStack(StackType),

    /// Position for cards to be shown to the user immediately after they're
    /// drawn.
    Drawn,

    /// Object is in a player's hand
    InHand(DisplayPlayer),

    /// Object is on top of a player's deck
    OnTopOfDeck(DisplayPlayer),

    /// Object is shuffled into a player's deck
    InDeck(DisplayPlayer),

    /// Object is in a player's void
    InVoid(DisplayPlayer),

    /// Object is in this player's banished zone
    InBanished(DisplayPlayer),

    /// Object is on the battlefield
    OnBattlefield(DisplayPlayer),

    /// Object is in a player's status zone
    InPlayerStatus(DisplayPlayer),

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
    InDreamwell(DisplayPlayer),

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
    TargetingUserBattlefield,
    TargetingEnemyBattlefield,
    TargetingBothBattlefields,
}
