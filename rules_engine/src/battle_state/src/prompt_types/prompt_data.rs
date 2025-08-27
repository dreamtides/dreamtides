use ability_data::effect::{Effect, ModalEffectChoice};
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum_macros::EnumDiscriminants;
use tabula_data::localized_strings::StringId;

use crate::battle::battle_state::PendingEffectIndex;
use crate::battle::card_id::{BattleDeckCardId, CharacterId, StackCardId, VoidCardId};
use crate::battle_cards::card_set::CardSet;
use crate::battle_cards::stack_card_state::{EffectTargets, StackItemId};
use crate::core::effect_source::EffectSource;

/// Describes which object should be updated based on the results of a prompt.
#[derive(
    Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, JsonSchema,
)]
pub enum OnSelected {
    AddStackTargets(StackItemId),
    AddPendingEffectTarget(PendingEffectIndex),
}

/// Data for a prompt to be displayed to a player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptData {
    /// Source which caused this prompt to be displayed
    pub source: EffectSource,

    /// Player to display the prompt to.
    pub player: PlayerName,

    /// Prompt to display.
    pub prompt_type: PromptType,

    /// Configuration options for the prompt.
    #[serde(default)]
    pub configuration: PromptConfiguration,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize)]
#[strum_discriminants()]
pub enum PromptType {
    ChooseCharacter { on_selected: OnSelected, valid: CardSet<CharacterId> },
    ChooseStackCard { on_selected: OnSelected, valid: CardSet<StackCardId> },
    ChooseVoidCard(ChooseVoidCardPrompt),
    Choose { choices: Vec<PromptChoice> },
    ChooseEnergyValue { minimum: Energy, maximum: Energy },
    ModalEffect(ModalEffectPrompt),
    SelectDeckCardOrder { prompt: SelectDeckCardOrderPrompt },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChooseVoidCardPrompt {
    pub on_selected: OnSelected,
    pub valid: CardSet<VoidCardId>,
    pub selected: CardSet<VoidCardId>,
    pub maximum_selection: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModalEffectPrompt {
    pub on_selected: OnSelected,
    pub choices: Vec<ModalEffectChoice>,
}

/// State for a prompt to select a deck card order.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SelectDeckCardOrderPrompt {
    /// Initial list of cards we are considering for ordering.
    pub initial: Vec<BattleDeckCardId>,

    /// Cards which have had a `SelectOrderForDeckCard` action performed on
    /// them. Used by the AI to prevent loops.
    pub moved: CardSet<BattleDeckCardId>,

    /// Cards currently in the deck position, in the selected order.
    pub deck: Vec<BattleDeckCardId>,

    /// Cards currently in the void position, unordered.
    pub void: CardSet<BattleDeckCardId>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PromptConfiguration {
    /// Can the player select no option to resolve this prompt?
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptChoice {
    /// Label to display for this choice.
    pub label: PromptChoiceLabel,

    /// Effect to apply when this choice is selected. This effect is resolved as
    /// applied by the 'controller' player.
    pub effect: Effect,

    /// Optionally, targets to apply the effect to.
    pub targets: Option<EffectTargets>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PromptChoiceLabel {
    String(StringId),
    StringWithEnergy(StringId, Energy),
}
