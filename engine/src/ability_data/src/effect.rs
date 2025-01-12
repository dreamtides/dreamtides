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

    /// Present if this is an effect which the controller may choose to apply,
    /// usually phrased as "You may {perform effect}". A cost to perform the
    /// effect can also be specified via "You may {pay cost} to {perform
    /// effect}." templating.
    pub optional: Option<Cost>,

    /// Indicates an effect set which occurs only if some condition is met,
    /// usually phrased as "If {condition}, {effect}"
    pub condition: Option<Condition>,
}

impl EffectWithOptions {
    pub fn new(effect: StandardEffect) -> Self {
        Self { effect, optional: None, condition: None }
    }

    pub fn with_condition(&self, condition: Condition) -> Self {
        let mut result = self.clone();
        result.condition = Some(condition);
        result
    }

    pub fn is_optional(&self) -> bool {
        self.optional.is_some()
    }

    pub fn to_effect(self) -> Effect {
        if !self.is_optional() && self.condition.is_none() {
            Effect::Effect(self.effect)
        } else {
            Effect::WithOptions(self)
        }
    }
}
