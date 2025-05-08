use ability_data::effect::Effect;
use bit_set::BitSet;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use smallvec::SmallVec;
use strum_macros::EnumDiscriminants;

use crate::battle_cards::stack_card_state::StackCardTargets;
use crate::core::effect_source::EffectSource;

/// Data for a prompt to be displayed to a player.
#[derive(Debug, Clone)]
pub struct PromptData {
    /// Source which caused this prompt to be displayed
    pub source: EffectSource,

    /// Player to display the prompt to.
    pub player: PlayerName,

    /// Prompt to display.
    pub prompt_type: PromptType,

    /// Why is this prompt being shown? Controls UI displayed to communicate to
    /// the player what is happening.
    pub context: PromptContext,

    /// Configuration options for the prompt.
    pub configuration: PromptConfiguration,
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants()]
pub enum PromptType {
    ChooseCharacter { valid: BitSet<usize> },
    ChooseStackCard { valid: BitSet<usize> },
    Choose { choices: SmallVec<[PromptChoice; 2]> },
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
}

#[derive(Debug, Clone)]
pub struct PromptChoice {
    /// Label to display for this choice.
    pub label: PromptChoiceLabel,

    /// Effect to apply when this choice is selected. This effect is resolved as
    /// applied by the 'controller' player.
    pub effect: &'static Effect,

    /// Optionally, targets to apply the effect to.
    pub targets: StackCardTargets,
}

#[derive(Debug, Clone, Copy)]
pub enum PromptChoiceLabel {
    Resolve,
    PayEnergy(Energy),
}
