use serde::{Deserialize, Serialize};

use crate::battle_player::player_map::PlayerMap;

/// Tracks history of actions and events during a turn
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TurnHistory {
    /// History of actions and events while resolving a single action.
    pub current_action_history: PlayerMap<CurrentActionHistory>,
}

/// Tracks history of actions and events while resolving a single action.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CurrentActionHistory {
    /// Whether the hand size limit was exceeded while resolving the action.
    pub hand_size_limit_exceeded: bool,
}

impl TurnHistory {
    pub fn clear_current_action_history(&mut self) {
        self.current_action_history = PlayerMap::default();
    }
}
