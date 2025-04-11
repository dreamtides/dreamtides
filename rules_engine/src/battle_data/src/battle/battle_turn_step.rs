use enum_iterator::Sequence;
use enumset::EnumSetType;

/// Current step within a given battle turn.
#[derive(Debug, Ord, PartialOrd, Hash, EnumSetType, Sequence)]
pub enum BattleTurnStep {
    Judgment,
    Dreamwell,
    Draw,
    Main,
    Ending,
}
