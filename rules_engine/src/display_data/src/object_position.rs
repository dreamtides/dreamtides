use battle_state::actions::battle_actions::CardOrderSelectionTargetDiscriminants;
use core_data::identifiers::SiteId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::battle_view::DisplayPlayer;
use crate::card_view::ClientCardId;
use crate::quest_view::TemptingOfferNumber;

/// Represents the position of some object in the UI
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct ObjectPosition {
    /// Position category
    pub position: Position,
    /// Sorting key, determines order within the position
    pub sorting_key: u32,
}

impl Default for ObjectPosition {
    fn default() -> Self {
        Self { position: Position::Default, sorting_key: 0 }
    }
}

/// Possible types of display positions
#[derive(
    Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd, JsonSchema,
)]
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
    CardOrderSelector(CardOrderSelectionTargetDiscriminants),

    /// Object is in a temporary holding space for cards in hand while resolving
    /// some other 'play card' ability.
    HandStorage,

    /// Object is in the dreamwell for a player (usually off-screen).
    InDreamwell(DisplayPlayer),

    /// Object is in a position to display itself as part of a dreamwell
    /// activation.
    DreamwellActivation,

    /// Object is hidden within a card
    HiddenWithinCard(ClientCardId),

    /// Object describes a game modifier or ongoing game effect
    GameModifier,

    /// Object is in the on-screen storage area, used to hold objects at a small
    /// size when they're not being focused on, e.g. when the user hides a
    /// card browser to get a better view of the battlefield.
    OnScreenStorage,

    /// Object is above the void, used to display void cards which are currently
    /// being targeted.
    AboveVoid(DisplayPlayer),

    /// User deck displayed in the quest view
    QuestDeck,

    /// Quest deck is open, showing all cards in a large scrollable format.
    QuestDeckBrowser,

    /// Object is hidden after being destroyed during a quest, for example when
    /// not selected for a draft pick.
    DestroyedQuestCards,

    /// Dreamsigns owned by the player in a quest
    DreamsignDisplay,

    /// Object is in a deck of cards displayed at a given dreamscape site, e.g.
    /// before being drafted.
    SiteDeck(SiteId),

    /// Object is being displayed in the possession of an NPC at a given
    /// dreamscape site (prior to animation to the display), e.g. a merchant.
    SiteNpc(SiteId),

    /// Object is being displayed as a potential draft pick choice
    DraftPickDisplay,

    /// Object is being displayed in the shop interface
    ShopDisplay,

    /// Object is being displayed as an option in a journey choice.
    JourneyDisplay,

    /// Object is being displayed as an option in a tempting offer choice.
    TemptingOfferDisplay(TemptingOfferPosition),

    /// Object is being displayed prominently, applying an effect to the
    /// currently active quest. Similar to a card being on the stack during a
    /// battle.
    QuestEffect,

    /// Object is being displayed as one of the identity cards participating in
    /// a battle.
    StartBattleDisplay(StartBattleDisplayType),
}

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd, JsonSchema,
)]
pub enum StackType {
    Default,
    TargetingUserBattlefield,
    TargetingEnemyBattlefield,
    TargetingBothBattlefields,
}

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd, JsonSchema,
)]
pub enum StartBattleDisplayType {
    UserIdentityCard,
    EnemyIdentityCard,
    UserDreamsigns,
    EnemyDreamsigns,
}

#[derive(
    Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd, JsonSchema,
)]
pub struct TemptingOfferPosition {
    pub number: TemptingOfferNumber,
    pub offer_type: TemptingOfferType,
}

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd, JsonSchema,
)]
pub enum TemptingOfferType {
    Journey,
    Cost,
}
