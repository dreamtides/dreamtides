use core_data::numerics::{Energy, Points, Spark};
use display_data::battle_view::PlayerView;

#[derive(Default)]
pub struct TestClientPlayer {
    pub view: Option<PlayerView>,
}

impl TestClientPlayer {
    /// Get the player's current score
    pub fn score(&self) -> Option<Points> {
        self.view.as_ref().map(|v| v.score)
    }

    /// Get the player's current energy
    pub fn energy(&self) -> Option<Energy> {
        self.view.as_ref().map(|v| v.energy)
    }

    /// Get the player's produced energy
    pub fn produced_energy(&self) -> Option<Energy> {
        self.view.as_ref().map(|v| v.produced_energy)
    }

    /// Get the player's total spark
    pub fn total_spark(&self) -> Option<Spark> {
        self.view.as_ref().map(|v| v.total_spark)
    }

    /// Check if this player can currently act
    pub fn can_act(&self) -> bool {
        self.view.as_ref().map(|v| v.can_act).unwrap_or(false)
    }

    /// Check if it's this player's turn
    pub fn is_current_turn(&self) -> bool {
        self.view.as_ref().map(|v| v.is_current_turn).unwrap_or(false)
    }

    /// Check if this player is about to win
    pub fn is_victory_imminent(&self) -> bool {
        self.view.as_ref().map(|v| v.is_victory_imminent).unwrap_or(false)
    }
}
