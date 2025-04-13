use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::battle::battle_status::BattleStatus;
use crate::battle::battle_turn_step::BattleTurnStep;
use crate::battle::request_context::RequestContext;
use crate::battle::turn_data::TurnData;
use crate::battle_animations::animation_data::{AnimationData, AnimationStep};
use crate::battle_animations::battle_animation::BattleAnimation;
use crate::battle_cards::all_cards::AllCards;
use crate::battle_player::player_data::PlayerData;

/// Contains data types for a "battle", a single instance of playing a game
/// against an enemy.
#[derive(Clone, Debug)]
pub struct BattleData {
    /// Unique identifier for this battle
    pub id: BattleId,

    /// Context of the request which triggered this rules engine execution
    pub request_context: RequestContext,

    /// Player data for the user
    pub user: PlayerData,

    /// Player data for the enemy
    pub enemy: PlayerData,

    /// All cards in this battle
    pub cards: AllCards,

    /// Status of this battle, including whether it has ended.
    pub status: BattleStatus,

    /// Current turn
    pub turn: TurnData,

    /// Current step within the turn
    pub step: BattleTurnStep,

    /// Random number generator for this battle
    pub rng: Xoshiro256PlusPlus,

    /// Animation tracker for this battle. If this is None it means we are not
    /// currently rendering for display.
    pub animations: Option<AnimationData>,
}

impl BattleData {
    pub fn player(&self, player_name: PlayerName) -> &PlayerData {
        match player_name {
            PlayerName::User => &self.user,
            PlayerName::Enemy => &self.enemy,
        }
    }

    pub fn player_mut(&mut self, player_name: PlayerName) -> &mut PlayerData {
        match player_name {
            PlayerName::User => &mut self.user,
            PlayerName::Enemy => &mut self.enemy,
        }
    }

    /// Pushes a new animation step onto the animation tracker, if animation
    /// tracking is enabled.
    ///
    /// This takes a function instead of a [BattleAnimation]. If you need to do
    /// any computation to determine the animation values, put it within the
    /// function so it won't run when animations are not being tracked.
    pub fn push_animation(&mut self, update: impl FnOnce() -> BattleAnimation) {
        if let Some(animations) = &mut self.animations {
            let snapshot = Self {
                id: self.id,
                request_context: self.request_context,
                user: self.user.clone(),
                enemy: self.enemy.clone(),
                cards: self.cards.clone(),
                status: self.status.clone(),
                turn: self.turn,
                step: self.step,
                rng: self.rng.clone(),
                animations: None,
            };
            animations.steps.push(AnimationStep { snapshot, animation: update() });
        }
    }
}
