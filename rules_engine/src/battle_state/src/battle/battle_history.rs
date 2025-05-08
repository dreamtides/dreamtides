use core_data::types::PlayerName;
use serde::{Deserialize, Serialize};

use crate::actions::battle_actions::BattleAction;

/// Tracks history of actions and events during a battle
#[derive(Clone, Debug, Default)]
pub struct BattleHistory {
    pub actions: Vec<BattleHistoryAction>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BattleHistoryAction {
    pub player: PlayerName,
    pub action: BattleAction,
}

impl BattleHistory {
    pub fn push_action(&mut self, player: PlayerName, action: BattleAction) {
        self.actions.push(BattleHistoryAction { player, action });
    }
}
