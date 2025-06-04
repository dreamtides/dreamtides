use ability_data::effect::Effect;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use strum_macros::EnumDiscriminants;

use crate::battle::card_id::{CardId, CharacterId, StackCardId};
use crate::battle_cards::card_set::CardSet;
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
    ChooseCharacter { valid: CardSet<CharacterId> },
    ChooseStackCard { valid: CardSet<StackCardId> },
    Choose { choices: Vec<PromptChoice> },
    ChooseEnergyValue { minimum: Energy, maximum: Energy },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromptContext {
    // A choice about which cards to target for an effect
    SelectTargetForNegativeEffect,
    SelectTargetForPositiveEffect,

    // A choice about whether to apply an effect to a given card
    ApplyNegativeEffectChoice(CardId),
    ApplyPositiveEffectChoice(CardId),

    // Pick energy additional costs while playing a card
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
    pub effect: Effect,

    /// Optionally, targets to apply the effect to.
    pub targets: Option<StackCardTargets>,
}

#[derive(Debug, Clone, Copy)]
pub enum PromptChoiceLabel {
    Decline,
    PayEnergy(Energy),
}
