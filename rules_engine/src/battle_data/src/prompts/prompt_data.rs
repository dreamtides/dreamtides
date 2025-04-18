use core_data::types::PlayerName;

use crate::battle_cards::card_id::{CharacterId, StackCardId};

/// Data for a prompt to be displayed to a player.
#[derive(Debug, Clone)]
pub struct PromptData {
    /// Player to display the prompt to.
    pub player: PlayerName,

    /// Propmt to display.
    pub prompt: Prompt,

    /// Can the player select no option to resolve this prompt?
    pub optional: bool,

    /// Why is this prompt being shown? Controls UI displayed to communicate to
    /// the player what is happening.
    pub context: PromptContext,
}

#[derive(Debug, Clone)]
pub enum Prompt {
    ChooseCharacter { valid: Vec<CharacterId> },
    ChooseStackCard { valid: Vec<StackCardId> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromptContext {
    TargetNegativeEffect,
    TargetPositiveEffect,
}
