use core_data::numerics::{Energy, Points};
use core_data::types::PlayerName;
use strum::{Display, EnumDiscriminants};

use crate::battle::card_id::{CardId, CharacterId, HandCardId, StackCardId};
use crate::battle_cards::stack_card_state::StackCardTargets;
use crate::prompt_types::prompt_data::PromptChoiceLabel;

/// Records events during rules engine execution for display as game animations.
#[derive(Clone, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(Display))]
pub enum BattleAnimation {
    StartTurn {
        player: PlayerName,
    },
    Judgment {
        player: PlayerName,
        new_score: Option<Points>,
    },
    DreamwellActivation {
        player: PlayerName,
        dreamwell_card_id: CardId,
        new_energy: Energy,
        new_produced_energy: Energy,
    },
    PlayCardFromHand {
        player: PlayerName,
        card_id: HandCardId,
    },
    DrawCards {
        player: PlayerName,
        cards: Vec<HandCardId>,
    },
    SelectStackCardTargets {
        player: PlayerName,
        source_id: StackCardId,
        targets: StackCardTargets,
    },
    MakeChoice {
        player: PlayerName,
        choice: PromptChoiceLabel,
    },
    ResolveCharacter {
        character_id: CharacterId,
    },
    Negate {
        target_id: StackCardId,
    },
    Dissolve {
        target_id: CharacterId,
    },
}

impl BattleAnimation {
    pub fn discriminant(&self) -> BattleAnimationDiscriminants {
        self.into()
    }
}
