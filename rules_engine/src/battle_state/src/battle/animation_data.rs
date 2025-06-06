use crate::battle::battle_animation::BattleAnimation;
use crate::battle::battle_state::BattleState;
use crate::core::effect_source::EffectSource;

/// Tracks game animations which trigger during rules engine execution
#[derive(Clone, Debug, Default)]
pub struct AnimationData {
    /// Steps in this animation
    pub steps: Vec<AnimationStep>,
}

/// A single animation & associated battle snapshot
#[derive(Clone, Debug)]
pub struct AnimationStep {
    /// The source of the animation
    pub source: EffectSource,

    /// Snapshot of the battle state when this animation was applied.
    pub snapshot: BattleState,

    /// Animation to show
    pub animation: BattleAnimation,
}
