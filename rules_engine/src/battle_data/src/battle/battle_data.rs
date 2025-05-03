use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::actions::battle_action_data::BattleAction;
use crate::battle::battle_history::BattleHistory;
use crate::battle::battle_status::BattleStatus;
use crate::battle::battle_tracing::{BattleTraceEvent, BattleTracing};
use crate::battle::battle_turn_step::BattleTurnStep;
use crate::battle::request_context::RequestContext;
use crate::battle::turn_data::TurnData;
use crate::battle_animations::animation_data::{AnimationData, AnimationStep};
use crate::battle_animations::battle_animation::BattleAnimation;
use crate::battle_cards::all_cards::AllCards;
use crate::battle_player::player_data::PlayerData;
use crate::debug_snapshots::debug_battle_data::DebugBattleData;
use crate::prompt_types::prompt_data::{PromptData, PromptResumeAction};

/// Contains data types for a "battle", a single instance of playing a game
/// against an enemy.
#[derive(Clone, Debug)]
pub struct BattleData {
    /// Unique identifier for this battle
    pub id: BattleId,

    /// Context of the request which triggered this rules engine execution
    pub request_context: RequestContext,

    /// Player data for the starting player
    pub player_one: PlayerData,

    /// Player data for the non-starting player
    pub player_two: PlayerData,

    /// All cards in this battle
    pub cards: AllCards,

    /// Status of this battle, including whether it has ended.
    pub status: BattleStatus,

    /// Player who currently has priority in this game, or who last held
    /// priority if the game has ended.
    ///
    /// This is the player who is next to act, *except* if a `prompt` is
    /// specified below, in which case the prompted player takes precedence over
    /// this value.
    pub priority: PlayerName,

    /// Current turn
    pub turn: TurnData,

    /// Current step within the turn
    pub step: BattleTurnStep,

    /// Seed used to initialize the random number generator
    pub seed: u64,

    /// Random number generator for this battle
    pub rng: Xoshiro256PlusPlus,

    /// Animation tracker for this battle. If this is None it means we are not
    /// currently rendering for display.
    pub animations: Option<AnimationData>,

    /// Prompt to display to a player.
    ///
    /// Only one prompt may be active at a time. It is an error to attempt to
    /// display another prompt while a choice is pending.
    pub prompt: Option<PromptData>,

    /// Action to take after a prompt is resolved, used if the prompt
    /// interrupted some other mutation.
    pub prompt_resume_action: Option<PromptResumeAction>,

    /// Debug tracing data for this battle
    pub tracing: Option<BattleTracing>,

    /// History of actions and events during this battle.
    ///
    /// Can be None if history tracking is disabled, e.g. during AI simulation.
    pub history: Option<BattleHistory>,
}

impl BattleData {
    pub fn player(&self, player_name: PlayerName) -> &PlayerData {
        match player_name {
            PlayerName::One => &self.player_one,
            PlayerName::Two => &self.player_two,
        }
    }

    pub fn player_mut(&mut self, player_name: PlayerName) -> &mut PlayerData {
        match player_name {
            PlayerName::One => &mut self.player_one,
            PlayerName::Two => &mut self.player_two,
        }
    }

    /// Returns a clone of this battle data with the animation tracker removed.
    pub fn clone_for_ai_search(&self) -> Self {
        Self { animations: None, tracing: None, history: None, ..self.clone() }
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
                player_one: self.player_one.clone(),
                player_two: self.player_two.clone(),
                cards: self.cards.clone(),
                status: self.status.clone(),
                priority: self.priority,
                turn: self.turn,
                step: self.step,
                seed: self.seed,
                rng: self.rng.clone(),
                animations: None,
                prompt: self.prompt.clone(),
                prompt_resume_action: None,
                tracing: None,
                history: None,
            };
            animations.steps.push(AnimationStep { snapshot, animation: update() });
        }
    }

    /// Adds a new tracing event for the current turn
    pub fn add_tracing_event(&mut self, event: BattleTraceEvent) {
        if let Some(tracing) = &mut self.tracing {
            if tracing.turn != self.turn.turn_id {
                tracing.turn = self.turn.turn_id;
                tracing.current.clear();
            }
            tracing.current.push(event);
        }
    }

    /// Adds a new action to the history of this battle.
    pub fn push_history_action(&mut self, player: PlayerName, action: BattleAction) {
        if let Some(history) = &mut self.history {
            history.push_action(player, action);
        }
    }

    /// Returns a debug snapshot of this battle, with string representations of
    /// the current state. This is intended for use with a debugger like GDB,
    /// to enable a readable description of the state in a flat hierarchy of
    /// variables.
    pub fn debug_snapshot(&self) -> DebugBattleData {
        DebugBattleData::new(self.clone())
    }
}
