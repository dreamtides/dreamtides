use core_data::numerics::Energy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::panel_address::PanelAddress;

#[derive(Debug, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
pub enum BattleDisplayAction {
    BrowseCards(CardBrowserType),
    CloseCardBrowser,

    /// Sets the selected amount of energy to pay as an additional cost to play
    /// a card.
    SetSelectedEnergyAdditionalCost(Energy),

    /// Opens a panel based on its address.
    OpenPanel(PanelAddress),

    /// Closes the currently open panel.
    CloseCurrentPanel,

    /// Toggles the visibility of the stack.
    ToggleStackVisibility,
}

#[derive(
    Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, JsonSchema,
)]
pub enum CardBrowserType {
    UserDeck,
    EnemyDeck,
    UserVoid,
    EnemyVoid,
    UserStatus,
    EnemyStatus,
}
