use core_data::identifiers::CardName;
use core_data::numerics::{Energy, Points, Spark};
use core_data::types::PlayerName;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
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
    AddCardToHand { player: PlayerName, card: CardName },
    /// Add a specific card to battlefield
    AddCardToBattlefield { player: PlayerName, card: CardName },
    /// Add a specific card to void
    AddCardToVoid { player: PlayerName, card: CardName },
    /// Move all cards from hand to deck
    MoveHandToDeck { player: PlayerName },
    /// Set the number of cards remaining in a player's deck. All other cards
    /// are moved to the void.
    SetCardsRemainingInDeck { player: PlayerName, cards: usize },
    /// Play a card for the opponent, with random prompt choices
    OpponentPlayCard { card: CardName },
}
