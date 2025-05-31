use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::actions::battle_actions::BattleAction;
use crate::battle::all_cards::AllCards;
use crate::battle::animation_data::{AnimationData, AnimationStep};
use crate::battle::battle_animation::BattleAnimation;
use crate::battle::battle_history::BattleHistory;
use crate::battle::battle_status::BattleStatus;
use crate::battle::battle_turn_phase::BattleTurnPhase;
use crate::battle::turn_data::TurnData;
use crate::battle::turn_history::TurnHistory;
use crate::battle_player::battle_player_state::BattlePlayerState;
use crate::battle_player::player_map::PlayerMap;
use crate::battle_trace::battle_tracing::BattleTracing;
use crate::prompt_types::prompt_data::PromptData;

#[derive(Clone, Debug)]
pub struct BattleState {
    /// Unique identifier for this battle
    pub id: BattleId,

    /// All cards in this battle
    pub cards: AllCards,

    /// Player data for all players in this battle
    pub players: PlayerMap<BattlePlayerState>,

    /// Status of this battle, including whether it has ended.
    pub status: BattleStatus,

    /// Player who is currently next to act when a stack is active.
    pub stack_priority: Option<PlayerName>,

    /// Current turn
    pub turn: TurnData,

    /// Current phase within the turn
    pub phase: BattleTurnPhase,

    /// Seed used to initialize the random number generator
    pub seed: u64,

    /// Random number generator for this battle
    pub rng: Xoshiro256PlusPlus,

    /// Prompt to display to a player.
    ///
    /// Only one prompt may be active at a time. It is an error to attempt to
    /// display another prompt while a choice is pending.
    pub prompt: Option<PromptData>,

    /// Animation tracker for this battle. If this is None it means we are not
    /// currently rendering for display.
    pub animations: Option<AnimationData>,

    /// Debug tracing data for this battle
    pub tracing: Option<BattleTracing>,

    /// History of actions and events during this battle.
    ///
    /// Can be None if history tracking is disabled, e.g. during AI simulation.
    pub action_history: Option<BattleHistory>,

    /// History of actions and events during the current turn.
    pub turn_history: TurnHistory,
}

impl BattleState {
    /// Returns a clone of this battle state with only the elements populated
    /// that directly affect game logic.
    ///
    /// Suitable for use in e.g. AI simulation.
    pub fn logical_clone(&self) -> Self {
        Self {
            id: self.id,
            cards: self.cards.clone(),
            players: self.players.clone(),
            status: self.status.clone(),
            stack_priority: self.stack_priority,
            turn: self.turn,
            phase: self.phase,
            seed: self.seed,
            rng: self.rng.clone(),
            prompt: self.prompt.clone(),
            animations: None,
            tracing: None,
            action_history: None,
            turn_history: self.turn_history.clone(),
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
                cards: self.cards.clone(),
                players: self.players.clone(),
                status: self.status.clone(),
                stack_priority: self.stack_priority,
                turn: self.turn,
                phase: self.phase,
                seed: self.seed,
                rng: self.rng.clone(),
                prompt: self.prompt.clone(),
                animations: None,
                tracing: None,
                action_history: None,
                turn_history: self.turn_history.clone(),
            };
            animations.steps.push(AnimationStep { snapshot, animation: update() });
        }
    }

    /// Optional version of [Self::push_animation].
    pub fn push_animation_optional(&mut self, update: impl FnOnce() -> Option<BattleAnimation>) {
        if self.animations.is_some()
            && let Some(animation) = update()
        {
            self.push_animation(|| animation);
        }
    }

    /// Adds a new action to the history of this battle.
    pub fn push_history_action(&mut self, player: PlayerName, action: BattleAction) {
        if let Some(history) = &mut self.action_history {
            history.push_action(player, action);
        }
    }
}
