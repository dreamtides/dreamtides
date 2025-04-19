use core_data::identifiers::AbilityNumber;
use core_data::types::PlayerName;
use serde::{Deserialize, Serialize};

use crate::battle_cards::card_id::{CharacterId, StackCardId};

/// Describes the source of some mutation or query.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum EffectSource {
    /// Effect caused by the rules of the game, e.g. drawing a card for turn
    /// during a battle. The controller is the player whose turn caused the
    /// effect.
    Game { controller: PlayerName },

    /// Effect caused by an ability of a card on the stack
    Event { controller: PlayerName, card: StackCardId, ability_number: AbilityNumber },

    /// Effect caused by an activated ability of a character on the battlefield
    Activated { controller: PlayerName, card: CharacterId, ability_number: AbilityNumber },

    /// Effect caused by a triggered ability of a character on the battlefield
    Triggered { controller: PlayerName, card: CharacterId, ability_number: AbilityNumber },
}

impl EffectSource {
    pub fn controller(&self) -> PlayerName {
        match self {
            EffectSource::Game { controller } => *controller,
            EffectSource::Event { controller, .. } => *controller,
            EffectSource::Activated { controller, .. } => *controller,
            EffectSource::Triggered { controller, .. } => *controller,
        }
    }
}
