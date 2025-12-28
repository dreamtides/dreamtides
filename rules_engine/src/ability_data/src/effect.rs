use core_data::numerics::Energy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::condition::Condition;
use crate::cost::Cost;
use crate::standard_effect::StandardEffect;

/// Represents a mutation to the game state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    Effect(StandardEffect),
    WithOptions(EffectWithOptions),
    List(Vec<EffectWithOptions>),
    Modal(Vec<ModalEffectChoice>),
}

/// Identifies a [ModalEffectChoice] in a [Effect::Modal].
#[derive(
    Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, JsonSchema,
)]
pub struct ModelEffectChoiceIndex(pub usize);

/// Represents a choice of effect to apply. These are written as a bulleted list
/// of options with associated costs and the text "Choose One".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalEffectChoice {
    pub energy_cost: Energy,
    pub effect: Effect,
}

/// Provides an effect along with configuration options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectWithOptions {
    /// Effect to apply
    pub effect: StandardEffect,

    /// True if this is an effect which the controller may choose to apply,
    /// usually phrased as: "You may {perform effect}".
    pub optional: bool,

    /// A cost to apply this effect, if any. Usually written as "You may pay
    /// {cost} to perform {effect}" on triggered abilities.
    ///
    /// This is used for costs that apply on resolution of the effect. It is
    /// *not* used for additional costs to play event cards, which are paid
    /// before placing the card on the stack.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_cost: Option<Cost>,

    /// Indicates an effect which occurs only if some condition is met,
    /// usually phrased as "If {condition}, {effect}"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<Condition>,
}

impl ModelEffectChoiceIndex {
    pub fn value(self) -> usize {
        self.0
    }
}

impl EffectWithOptions {
    pub fn new(effect: StandardEffect) -> Self {
        Self { effect, optional: false, trigger_cost: None, condition: None }
    }

    pub fn with_condition(&self, condition: Condition) -> Self {
        let mut result = self.clone();
        result.condition = Some(condition);
        result
    }

    pub fn to_effect(self) -> Effect {
        if !self.optional && self.condition.is_none() && self.trigger_cost.is_none() {
            Effect::Effect(self.effect)
        } else {
            Effect::WithOptions(self)
        }
    }
}
