use std::fmt::{self, Debug, Formatter};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::battle_action_data::BattleAction;
use crate::debug_action_data::DebugAction;
use crate::panel_address::PanelAddress;

/// All possible user interface actions
#[derive(Copy, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GameAction {
    DebugAction(DebugAction),
    BattleAction(BattleAction),
    OpenPanel(PanelAddress),
    CloseCurrentPanel,
}

impl Debug for GameAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GameAction::DebugAction(action) => write!(f, "{:?}", action),
            GameAction::BattleAction(action) => write!(f, "{:?}", action),
            GameAction::OpenPanel(panel) => write!(f, "{:?}", panel),
            GameAction::CloseCurrentPanel => write!(f, "CloseCurrentPanel"),
        }
    }
}
