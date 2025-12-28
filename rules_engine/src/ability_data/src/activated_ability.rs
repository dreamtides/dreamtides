use serde::{Deserialize, Serialize};

use crate::cost::Cost;
use crate::effect::Effect;

/// An activated ability is present on a character card and allows the
/// controlling player to pay some cost in order to achieve an effect. This is
/// written as "> cost: effect".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivatedAbility {
    /// Costs to activate this ability, paid before it is put on the stack.
    pub costs: Vec<Cost>,

    /// Effect of this ability, applied as it resolves on the stack.
    pub effect: Effect,

    /// Configuration for this activated ability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ActivatedAbilityOptions>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivatedAbilityOptions {
    /// True if this ability can be activated in response to enemy game actions.
    pub is_fast: bool,

    /// True if this ability can be used multiple times per turn.
    pub is_multi: bool,
}

impl ActivatedAbility {
    pub fn is_fast(&self) -> bool {
        self.options.as_ref().map(|o| o.is_fast).unwrap_or_default()
    }

    pub fn is_multi(&self) -> bool {
        self.options.as_ref().map(|o| o.is_multi).unwrap_or_default()
    }
}
