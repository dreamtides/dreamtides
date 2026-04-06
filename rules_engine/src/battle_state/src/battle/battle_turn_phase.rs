use enum_iterator::Sequence;
use enumset::EnumSetType;
use serde::{Deserialize, Serialize};

/// Current phase within a given battle turn.
#[derive(Debug, Ord, PartialOrd, Hash, EnumSetType, Sequence, Serialize, Deserialize)]
pub enum BattleTurnPhase {
    Starting,
    Dreamwell,
    Draw,
    Dawn,
    Main,
    Ending,
    Judgment,
    EndingPhaseFinished,
    FiringEndOfTurnTriggers,
}
