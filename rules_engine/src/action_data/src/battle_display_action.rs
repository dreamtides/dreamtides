use core_data::numerics::Energy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum BattleDisplayAction {
    BrowseCards(CardBrowserType),
    CloseCardBrowser,

    /// Sets the selected amount of energy to pay as an additional cost to play
    /// a card.
    SetSelectedEnergyAdditionalCost(Energy),
}

#[derive(
    Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum CardBrowserType {
    UserDeck,
    EnemyDeck,
    UserVoid,
    EnemyVoid,
    UserStatus,
    EnemyStatus,
}
