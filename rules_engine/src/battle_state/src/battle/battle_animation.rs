use core_data::identifiers::AbilityNumber;
use core_data::numerics::{Energy, Points};
use core_data::types::PlayerName;
use strum::{Display, EnumDiscriminants};

use crate::battle::card_id::{CardId, CharacterId, HandCardId, StackCardId};
use crate::battle_cards::stack_card_state::EffectTargets;
use crate::prompt_types::prompt_data::PromptChoiceLabel;

/// Records events during rules engine execution for display as game animations.
#[derive(Clone, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(Display))]
pub enum BattleAnimation {
    Counterspell {
        target_id: StackCardId,
    },
    Dissolve {
        target_id: CharacterId,
    },
    DrawCards {
        player: PlayerName,
        cards: Vec<HandCardId>,
    },
    DreamwellActivation {
        player: PlayerName,
        dreamwell_card_id: CardId,
        new_energy: Energy,
        new_produced_energy: Energy,
    },
    FireTriggers {
        triggers: Vec<TriggerAnimation>,
    },
    Judgment {
        player: PlayerName,
        new_score: Option<Points>,
    },
    MakeChoice {
        player: PlayerName,
        choice: PromptChoiceLabel,
    },
    PlayCardFromHand {
        player: PlayerName,
        card_id: HandCardId,
    },
    PlayedCardFromHand {
        player: PlayerName,
        card_id: HandCardId,
    },
    ResolveCharacter {
        character_id: CharacterId,
    },
    SelectStackCardTargets {
        player: PlayerName,
        source_id: StackCardId,
        targets: EffectTargets,
    },
    StartTurn {
        player: PlayerName,
    },
}

#[derive(Clone, Debug)]
pub struct TriggerAnimation {
    pub controller: PlayerName,
    pub character_id: CharacterId,
    pub ability_number: AbilityNumber,
}
