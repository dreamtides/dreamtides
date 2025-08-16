use core_data::identifiers::BaseCardId;

/// Describes a card and the set of modifications applied to it.
///
/// A "base" card is a card that appears in the Tabula database with no
/// modifications. The `CardDescriptor` adds a set of modifications to a base
/// card. A `CardDescriptor` thus represents a `CardIdentity`, and we use the
/// `CardIdentity` in the rules engine as an efficient way to refer back to "a
/// card and its modifications". Card descriptors always map directly to an
/// immutable list of abilities for a card.
///
/// A "deck" of cards is hence always a collection of `CardDescriptor`s.
#[derive(Debug, Clone)]
pub struct CardDescriptor {
    pub base_id: BaseCardId,
    pub is_upgraded: bool,
}
