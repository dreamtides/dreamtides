use std::fmt::{self, Debug, Formatter};

use battle_state::actions::battle_actions::BattleAction;
use core_data::types::PlayerName;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::battle_display_action::BattleDisplayAction;
use crate::debug_action_data::DebugAction;
/// All possible user interface actions
#[derive(Copy, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GameAction {
    NoOp,
    DebugAction(DebugAction),
    BattleAction(BattleAction),
    BattleDisplayAction(BattleDisplayAction),
    Undo(PlayerName),
}

impl Debug for GameAction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            GameAction::NoOp => write!(f, "NoOp"),
            GameAction::DebugAction(action) => write!(f, "{:?}", action),
            GameAction::BattleAction(action) => write!(f, "{:?}", action),
            GameAction::BattleDisplayAction(action) => write!(f, "{:?}", action),
            GameAction::Undo(player) => write!(f, "Undo({:?})", player),
        }
    }
}

impl From<BattleAction> for GameAction {
    fn from(action: BattleAction) -> Self {
        GameAction::BattleAction(action)
    }
}

impl From<BattleDisplayAction> for GameAction {
    fn from(action: BattleDisplayAction) -> Self {
        GameAction::BattleDisplayAction(action)
    }
}
