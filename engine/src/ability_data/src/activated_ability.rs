use serde::{Deserialize, Serialize};

use crate::cost::Cost;
use crate::effect::Effect;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivatedAbilityOptions {
    /// True if this ability can be activated in response to enemy game actions.
    pub is_fast: bool,

    /// True if this ability can be used on the turn in which its controlling
    /// character was played.
    pub is_immediate: bool,

    /// True if this ability can be used multiple times per turn.
    pub is_multi: bool,
}

/// An activated ability is present on a character card and allows the
/// controlling player to pay some cost in order to achieve an effect. This is
/// written as "> cost: effect".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivatedAbility {
    /// Costs to activate this ability, paid before it is put on the stack.
    pub costs: Vec<Cost>,

    /// Effect of this ability, applied as it resolves on the stack.
    pub effect: Effect,

    /// Configuration for this activated ability
    pub options: Option<ActivatedAbilityOptions>,
}
