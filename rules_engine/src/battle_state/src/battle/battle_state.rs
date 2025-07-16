use std::collections::VecDeque;
use std::path::PathBuf;

use ability_data::effect::Effect;
use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use rand_xoshiro::Xoshiro256PlusPlus;
use serde::{Deserialize, Serialize};

use crate::actions::battle_actions::BattleAction;
use crate::battle::all_cards::AllCards;
use crate::battle::animation_data::{AnimationData, AnimationStep};
use crate::battle::battle_animation::BattleAnimation;
use crate::battle::battle_history::BattleHistory;
use crate::battle::battle_status::BattleStatus;
use crate::battle::battle_turn_phase::BattleTurnPhase;
use crate::battle::turn_data::TurnData;
use crate::battle::turn_history::TurnHistory;
use crate::battle_cards::ability_state::AbilityState;
use crate::battle_cards::activated_ability_state::ActivatedAbilityState;
use crate::battle_cards::stack_card_state::EffectTargets;
use crate::battle_player::battle_player_state::BattlePlayerState;
use crate::battle_player::player_map::PlayerMap;
use crate::battle_trace::battle_tracing::BattleTracing;
use crate::core::effect_source::EffectSource;
use crate::prompt_types::prompt_data::PromptData;
use crate::triggers::trigger_state::TriggerState;

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

    /// Queue of prompts to display to players in order to make choices.
    ///
    /// The first element in the queue (index 0) is the currently-active prompt.
    pub prompts: VecDeque<PromptData>,

    /// State of the trigger system.
    pub triggers: TriggerState,

    /// State of activated abilities for players in this battle.
    pub activated_abilities: PlayerMap<ActivatedAbilityState>,

    /// State of abilities in this battle.
    pub ability_state: AbilityState,

    /// Effects that are waiting to be applied.
    pub pending_effects: VecDeque<PendingEffect>,

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

    /// Information about why & how we are currently running the rules engine.
    pub request_context: RequestContext,
}

/// A unique identifier for a pending effect.
#[derive(Clone, Debug, Copy, Eq, PartialEq, Hash)]
pub struct PendingEffectIndex(pub usize);

/// Information about effects that are waiting to be applied.
#[derive(Clone, Debug)]
pub struct PendingEffect {
    /// Source of the effect.
    pub source: EffectSource,

    /// Effect that is waiting to be applied.
    pub effect: Effect,

    /// Targets that were requested for the effect.
    pub requested_targets: Option<EffectTargets>,
}

/// Information about why & how we are currently running the rules engine.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestContext {
    pub logging_options: LoggingOptions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingOptions {
    /// If specified, the directory to write logs to.
    pub log_directory: Option<PathBuf>,

    /// If true, log AI search diagrams in graphviz format to the log directory.
    pub log_ai_search_diagram: bool,

    /// If true, perform action legality checks before executing actions.
    pub enable_action_legality_check: bool,
}

impl Default for LoggingOptions {
    fn default() -> Self {
        Self {
            log_directory: None,
            log_ai_search_diagram: false,
            enable_action_legality_check: true,
        }
    }
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
            prompts: self.prompts.clone(),
            triggers: self.triggers.clone(),
            activated_abilities: self.activated_abilities.clone(),
            ability_state: self.ability_state.clone(),
            pending_effects: self.pending_effects.clone(),
            animations: None,
            tracing: None,
            action_history: None,
            turn_history: self.turn_history.clone(),
            request_context: self.request_context.clone(),
        }
    }

    /// Pushes a new animation step onto the animation tracker, if animation
    /// tracking is enabled.
    ///
    /// This takes a function instead of a [BattleAnimation]. If you need to do
    /// any computation to determine the animation values, put it within the
    /// function so it won't run when animations are not being tracked.
    pub fn push_animation(
        &mut self,
        source: EffectSource,
        update: impl FnOnce() -> BattleAnimation,
    ) {
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
                prompts: self.prompts.clone(),
                triggers: self.triggers.clone(),
                activated_abilities: self.activated_abilities.clone(),
                ability_state: self.ability_state.clone(),
                pending_effects: self.pending_effects.clone(),
                animations: None,
                tracing: None,
                action_history: None,
                turn_history: self.turn_history.clone(),
                request_context: self.request_context.clone(),
            };
            animations.steps.push(AnimationStep { source, snapshot, animation: update() });
        }
    }

    /// Adds a new action to the history of this battle.
    pub fn push_history_action(&mut self, player: PlayerName, action: BattleAction) {
        if let Some(history) = &mut self.action_history {
            history.push_action(player, action);
        }
    }

    /// Returns the pending effect at the given index, if any.
    pub fn pending_effect(&self, index: PendingEffectIndex) -> Option<&PendingEffect> {
        self.pending_effects.get(index.0)
    }

    /// Returns a mutable reference to the pending effect at the given index, if
    /// any.
    pub fn pending_effect_mut(&mut self, index: PendingEffectIndex) -> Option<&mut PendingEffect> {
        self.pending_effects.get_mut(index.0)
    }
}
