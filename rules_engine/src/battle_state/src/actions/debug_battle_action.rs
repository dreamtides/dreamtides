use core_data::identifiers::{BaseCardId, DreamwellCardId};
use core_data::numerics::{Energy, Points, Spark};
use core_data::types::PlayerName;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, JsonSchema,
)]
pub enum DebugBattleAction {
    /// Draw a card
    DrawCard { player: PlayerName },
    /// Set the energy of the player
    SetEnergy { player: PlayerName, energy: Energy },
    /// Set the points total of the player
    SetPoints { player: PlayerName, points: Points },
    /// Set the produced energy of the player
    SetProducedEnergy { player: PlayerName, energy: Energy },
    /// Set the spark bonus of the player
    SetSparkBonus { player: PlayerName, spark: Spark },
    /// Add a specific card to hand
    AddCardToHand { player: PlayerName, card: BaseCardId },
    /// Add a specific card to battlefield
    AddCardToBattlefield { player: PlayerName, card: BaseCardId },
    /// Add a specific card to void
    AddCardToVoid { player: PlayerName, card: BaseCardId },
    /// Move all cards from hand to deck
    MoveHandToDeck { player: PlayerName },
    /// Set the number of cards remaining in a player's deck. All other cards
    /// are moved to the void.
    SetCardsRemainingInDeck { player: PlayerName, cards: usize },
    /// Play a card for the opponent, with prompt choices
    OpponentPlayCard { card: BaseCardId },
    /// Cause the opponent to take a 'continue' legal action
    OpponentContinue,
    /// Sets the `next_index` for the dreamwell to draw the card with the
    /// indicated definition ID. Panics if this card is not present in the
    /// dreamwell.
    SetNextDreamwellCard { base_card_id: DreamwellCardId },
}
