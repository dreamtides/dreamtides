use core_data::identifiers::CardIdent;
use core_data::numerics::{Energy, Points};
use core_data::types::PlayerName;

use crate::battle_cards::card_id::HandCardId;

/// Records events during rules engine execution for display as game animations.
#[derive(Clone, Debug)]
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
        dreamwell_card_id: CardIdent,
        new_energy: Energy,
        new_produced_energy: Energy,
    },
    PlayCardFromHand {
        player: PlayerName,
        card_id: HandCardId,
    },
}
