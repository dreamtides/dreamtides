use serde::{Deserialize, Serialize};

use crate::condition::Condition;
use crate::cost::Cost;
use crate::standard_effect::StandardEffect;

/// Represents a mutation to the game state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Effect {
    Effect(StandardEffect),
    WithOptions(EffectWithOptions),
    List(Vec<EffectWithOptions>),
}

/// Provides an effect along with configuration options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectWithOptions {
    /// Effect to apply
    pub effect: StandardEffect,

    /// True if this is an effect which the controller may choose to apply,
    /// usually phrased as: "You may {perform effect}".
    pub optional: bool,

    /// A cost to apply this effect, if any. Usually written as "You may pay
    /// {cost} to perform {effect}" on activated or triggered ability cards.
    ///
    /// This is used for costs that apply on resolution of the effect. It is
    /// *not* used for additional costs to play event cards, which are paid
    /// before placing the card on the stack.
    pub cost: Option<Cost>,

    /// Indicates an effect set which occurs only if some condition is met,
    /// usually phrased as "If {condition}, {effect}"
    pub condition: Option<Condition>,
}

impl EffectWithOptions {
    pub fn new(effect: StandardEffect) -> Self {
        Self { effect, optional: false, cost: None, condition: None }
    }

    pub fn with_condition(&self, condition: Condition) -> Self {
        let mut result = self.clone();
        result.condition = Some(condition);
        result
    }

    pub fn to_effect(self) -> Effect {
        if !self.optional && self.condition.is_none() && self.cost.is_none() {
            Effect::Effect(self.effect)
        } else {
            Effect::WithOptions(self)
        }
    }
}
