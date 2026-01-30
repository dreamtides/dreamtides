use ability_data::ability::Ability;
use core_data::display_types::SpriteAddress;
use core_data::identifiers::DreamwellCardId;
use core_data::numerics::Energy;
use serde::{Deserialize, Serialize};

pub type DreamwellCardPhase = usize;

/// Describes the rules for a dreamwell card.
///
/// Dreamwell cards are special cards drawn during battles to increase the
/// amount of energy available to a player and provide random effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamwellCardDefinition {
    /// The base card that this card definition is based on.
    pub base_card_id: DreamwellCardId,
    /// Name of this card in the currently-active language.
    pub displayed_name: String,
    /// Energy production added by this card when it resolves.
    pub energy_produced: Energy,
    /// Abilities of this card.
    pub abilities: Vec<Ability>,
    /// Rules text of this card in the currently-active language, formatted for
    /// display.
    pub displayed_rules_text: String,
    /// Prompts to display for this card in the currently-active language,
    /// formatted for display.
    pub displayed_prompts: Vec<String>,
    /// "Phase" of this card, controls the order in which it appears in the
    /// shuffled dreamwell deck. Cards with a lower phase number appear first in
    /// the randomized order.
    pub phase: DreamwellCardPhase,
    /// Image to display for this card.
    pub image: SpriteAddress,
}
