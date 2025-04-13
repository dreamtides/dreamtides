use core_data::identifiers::CardId;
use core_data::numerics::{Energy, Points};
use core_data::types::PlayerName;

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
        dreamwell_card_id: CardId,
        new_energy: Energy,
        new_produced_energy: Energy,
    },
}
