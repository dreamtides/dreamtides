use crate::battle_player::player_map::PlayerMap;

/// Tracks history of actions and events during a turn
#[derive(Clone, Debug, Default)]
pub struct TurnHistory {
    /// History of actions and events while resolving a single action.
    pub current_action_history: PlayerMap<CurrentActionHistory>,
}

impl TurnHistory {
    pub fn clear_current_action_history(&mut self) {
        self.current_action_history = PlayerMap::default();
    }
}

/// Tracks history of actions and events while resolving a single action.
#[derive(Clone, Debug, Default)]
pub struct CurrentActionHistory {
    /// Whether the hand size limit was exceeded while resolving the action.
    pub hand_size_limit_exceeded: bool,
}
