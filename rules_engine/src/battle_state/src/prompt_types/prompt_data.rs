use ability_data::effect::Effect;
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use strum_macros::EnumDiscriminants;

use crate::battle::battle_state::PendingEffectId;
use crate::battle::card_id::{CharacterId, DeckCardId, StackCardId};
use crate::battle_cards::card_set::CardSet;
use crate::battle_cards::stack_card_state::{EffectTargets, StackItemId};
use crate::core::effect_source::EffectSource;

/// Describes which object should be updated based on the results of a prompt.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PromptFor {
    AddingItemToStack(StackItemId),
    PendingEffect(PendingEffectId),
}

/// Data for a prompt to be displayed to a player.
#[derive(Debug, Clone)]
pub struct PromptData {
    /// Source which caused this prompt to be displayed
    pub source: EffectSource,

    /// Player to display the prompt to.
    pub player: PlayerName,

    /// Prompt to display.
    pub prompt_type: PromptType,

    /// Configuration options for the prompt.
    pub configuration: PromptConfiguration,
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants()]
pub enum PromptType {
    ChooseCharacter { prompt_for: PromptFor, valid: CardSet<CharacterId> },
    ChooseStackCard { prompt_for: PromptFor, valid: CardSet<StackCardId> },
    Choose { choices: Vec<PromptChoice> },
    ChooseEnergyValue { minimum: Energy, maximum: Energy },
    SelectDeckCardOrder { prompt: SelectDeckCardOrderPrompt },
}

/// State for a prompt to select a deck card order.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SelectDeckCardOrderPrompt {
    /// Initial list of cards we are considering for ordering.
    pub initial: Vec<DeckCardId>,

    /// Cards which have had a `SelectOrderForDeckCard` action performed on
    /// them. Used by the AI to prevent loops.
    pub moved: CardSet<DeckCardId>,

    /// Cards currently in the deck position, in the selected order.
    pub deck: Vec<DeckCardId>,

    /// Cards currently in the void position, unordered.
    pub void: CardSet<DeckCardId>,
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
    pub targets: Option<EffectTargets>,
}

#[derive(Debug, Clone, Copy)]
pub enum PromptChoiceLabel {
    Decline,
    PayEnergy(Energy),
}
