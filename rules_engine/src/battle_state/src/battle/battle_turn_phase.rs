use enum_iterator::Sequence;
use enumset::EnumSetType;

/// Current phase within a given battle turn.
#[derive(Debug, Ord, PartialOrd, Hash, EnumSetType, Sequence)]
pub enum BattleTurnPhase {
    Starting,
    Judgment,
    Dreamwell,
    Draw,
    Main,
    Ending,
}
