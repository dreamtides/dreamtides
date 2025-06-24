use core_data::numerics::{Energy, Points, Spark};
use display_data::battle_view::PlayerView;

#[derive(Default)]
pub struct TestClientPlayer {
    pub view: Option<PlayerView>,
}

impl TestClientPlayer {
    /// Get the player's current score
    pub fn score(&self) -> Points {
        self.view.as_ref().map(|v| v.score).expect("Player has no score")
    }

    /// Get the player's current energy
    pub fn energy(&self) -> Energy {
        self.view.as_ref().map(|v| v.energy).expect("Player has no energy")
    }

    /// Get the player's produced energy
    pub fn produced_energy(&self) -> Energy {
        self.view.as_ref().map(|v| v.produced_energy).expect("Player has no produced energy")
    }

    /// Get the player's total spark
    pub fn total_spark(&self) -> Spark {
        self.view.as_ref().map(|v| v.total_spark).expect("Player has no total spark")
    }

    /// Check if this player can currently act
    pub fn can_act(&self) -> bool {
        self.view.as_ref().map(|v| v.can_act).expect("Player has no can_act")
    }

    /// Check if it's this player's turn
    pub fn is_current_turn(&self) -> bool {
        self.view.as_ref().map(|v| v.is_current_turn).expect("Player has no is_current_turn")
    }

    /// Check if this player is about to win
    pub fn is_victory_imminent(&self) -> bool {
        self.view
            .as_ref()
            .map(|v| v.is_victory_imminent)
            .expect("Player has no is_victory_imminent")
    }
}
