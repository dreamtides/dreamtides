use core_data::identifiers::QuestId;
use core_data::numerics::Essence;
use user_state::user::user_state::UserState;

use crate::quest::deck::Deck;

/// Contains data types for the "quest" gameplay, which contains all card
/// drafting and deck building mechanics.
///
/// Note that both human and AI players have their own quests and QuestStates,
/// although some of these details are hidden from the user. AI enemies build
/// their decks by participating in simulated quests.
#[derive(Clone, Debug)]
pub struct QuestState {
    /// Unique identifier for this quest.
    pub id: QuestId,

    /// The player's global state, unrelated to the current quest.
    pub user: UserState,

    /// The player's deck for this quest.
    pub deck: Deck,

    /// The players's essence, a currency used during quests.
    pub essence: Essence,
}
