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
    DrawCard(PlayerName),
    /// Set the energy of the player
    SetEnergy(PlayerName, Energy),
    /// Set the points total of the player
    SetPoints(PlayerName, Points),
    /// Set the produced energy of the player
    SetProducedEnergy(PlayerName, Energy),
    /// Set the spark bonus of the player
    SetSparkBonus(PlayerName, Spark),
    /// Add a specific card to hand
    AddCardToHand(PlayerName, CardName),
    /// Add a specific card to battlefield
    AddCardToBattlefield(PlayerName, CardName),
    /// Add a specific card to void
    AddCardToVoid(PlayerName, CardName),
    /// Move all cards from hand to deck
    MoveHandToDeck(PlayerName),
}
