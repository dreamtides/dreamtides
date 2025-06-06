use core_data::identifiers::AbilityNumber;
use core_data::types::PlayerName;
use serde::{Deserialize, Serialize};

use crate::battle::card_id::{CardId, CardIdType, CharacterId, StackCardId};

/// Describes the source of some mutation or query.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum EffectSource {
    /// Effect caused by the rules of the game, e.g. drawing a card for turn
    /// during a battle. The controller is the player whose turn caused the
    /// effect.
    Game { controller: PlayerName },

    /// Effect caused by a player action, e.g. playing a card.
    Player { controller: PlayerName },

    /// Effect caused by an ability of a card on the stack
    Event { controller: PlayerName, stack_card_id: StackCardId, ability_number: AbilityNumber },

    /// Effect caused by a character on the battlefield
    Character { controller: PlayerName, character_id: CharacterId },

    /// Effect caused by an activated ability of a character on the battlefield
    Activated { controller: PlayerName, character_id: CharacterId, ability_number: AbilityNumber },

    /// Effect caused by a triggered ability of a character on the battlefield
    Triggered { controller: PlayerName, character_id: CharacterId, ability_number: AbilityNumber },
}

impl EffectSource {
    /// Returns the controller of this effect.
    pub fn controller(&self) -> PlayerName {
        match self {
            EffectSource::Game { controller } => *controller,
            EffectSource::Player { controller } => *controller,
            EffectSource::Event { controller, .. } => *controller,
            EffectSource::Character { controller, .. } => *controller,
            EffectSource::Activated { controller, .. } => *controller,
            EffectSource::Triggered { controller, .. } => *controller,
        }
    }

    /// Returns the card ID of the source of this effect, if it is a card.
    pub fn card_id(&self) -> Option<CardId> {
        match self {
            EffectSource::Event { stack_card_id: card, .. } => Some(card.card_id()),
            EffectSource::Character { character_id: card, .. } => Some(card.card_id()),
            EffectSource::Activated { character_id: card, .. } => Some(card.card_id()),
            EffectSource::Triggered { character_id: card, .. } => Some(card.card_id()),
            _ => None,
        }
    }
}
