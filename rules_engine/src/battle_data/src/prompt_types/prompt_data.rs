use ability_data::effect::Effect;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use strum_macros::EnumDiscriminants;

use crate::battle::effect_source::EffectSource;
use crate::battle_cards::card_data::TargetId;
use crate::battle_cards::card_id::{CharacterId, StackCardId};
use crate::battle_cards::zone::Zone;

/// Data for a prompt to be displayed to a player.
#[derive(Debug, Clone)]
pub struct PromptData {
    /// Source which caused this prompt to be displayed
    pub source: EffectSource,

    /// Player to display the prompt to.
    pub player: PlayerName,

    /// Propmt to display.
    pub prompt: Prompt,

    /// Why is this prompt being shown? Controls UI displayed to communicate to
    /// the player what is happening.
    pub context: PromptContext,

    /// Configuration options for the prompt.
    pub configuration: PromptConfiguration,
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants()]
pub enum Prompt {
    ChooseCharacter { valid: Vec<CharacterId> },
    ChooseStackCard { valid: Vec<StackCardId> },
    Choose { choices: Vec<PromptChoice> },
    ChooseEnergyValue { minimum: Energy, current: Energy, maximum: Energy },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromptContext {
    TargetNegativeEffect,
    TargetPositiveEffect,
    PickAmountOfEnergyToSpend,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PromptConfiguration {
    /// Can the player select no option to resolve this prompt?
    pub optional: bool,

    /// A zone to move the source card of this prompt to after a choice is made.
    pub move_source_to: Option<Zone>,
}

#[derive(Debug, Clone)]
pub struct PromptChoice {
    /// Label to display for this choice.
    pub label: String,

    /// Effect to apply when this choice is selected. This effect is resolved as
    /// applied by the 'controller' player.
    pub effect: Effect,

    /// Optionally, a list of targets to apply the effect to.
    pub targets: Vec<TargetId>,
}
