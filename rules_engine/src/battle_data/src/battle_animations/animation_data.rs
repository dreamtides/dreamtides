use crate::battle::battle_data::BattleData;
use crate::battle_animations::battle_animation::BattleAnimation;

/// Tracks game animations which trigger during rules engine execution
#[derive(Clone, Debug, Default)]
pub struct AnimationData {
    /// Steps in this animation
    pub steps: Vec<AnimationStep>,
}

/// A single animation & associated battle snapshot
#[derive(Clone, Debug)]
pub struct AnimationStep {
    /// Snapshot of the battle state when this animation was applied.
    pub snapshot: BattleData,

    /// Animation to show
    pub animation: BattleAnimation,
}
