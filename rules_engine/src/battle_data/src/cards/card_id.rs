use std::fmt::Debug;
use std::hash::Hash;

use core_data::identifiers::CardDataIdentifier;

use crate::cards::all_cards::AllCards;

/// A trait for identifiers which correspond 1:1 with cards.
pub trait CardId: Hash + Eq + PartialEq + Debug + Ord + Copy {
    /// Returns the internal identifier for the card, if this card exists and
    /// this identifier is currently valid.
    ///
    /// Normally it should not be necessary to call this method in rules engine
    /// code.
    fn card_identifier(&self, cards: &AllCards) -> Option<CardDataIdentifier>;
}

/// An identifier for an object while it is in a given zone. A new zone object
/// ID is assigned each time a card changes zones, meaning that it can be
/// used for targeting effects that end when the card changes zones.
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ObjectId(pub u32);

impl CardId for CardDataIdentifier {
    fn card_identifier(&self, _cards: &AllCards) -> Option<CardDataIdentifier> {
        Some(*self)
    }
}

impl CardId for CardObjectId {
    fn card_identifier(&self, cards: &AllCards) -> Option<CardDataIdentifier> {
        if cards.card(self.card_id)?.id().object_id() == self.object_id {
            Some(self.card_id)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct CardObjectId {
    pub object_id: ObjectId,
    pub card_id: CardDataIdentifier,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct CharacterId(pub CardObjectId);

impl CharacterId {
    pub fn new(object_id: ObjectId, card_id: CardDataIdentifier) -> Self {
        Self(CardObjectId { object_id, card_id })
    }
}

impl CardId for CharacterId {
    fn card_identifier(&self, cards: &AllCards) -> Option<CardDataIdentifier> {
        self.0.card_identifier(cards)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct VoidCardId(pub CardObjectId);

impl VoidCardId {
    pub fn new(object_id: ObjectId, card_id: CardDataIdentifier) -> Self {
        Self(CardObjectId { object_id, card_id })
    }
}

impl CardId for VoidCardId {
    fn card_identifier(&self, cards: &AllCards) -> Option<CardDataIdentifier> {
        self.0.card_identifier(cards)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct DeckCardId(pub CardObjectId);

impl DeckCardId {
    pub fn new(object_id: ObjectId, card_id: CardDataIdentifier) -> Self {
        Self(CardObjectId { object_id, card_id })
    }
}

impl CardId for DeckCardId {
    fn card_identifier(&self, cards: &AllCards) -> Option<CardDataIdentifier> {
        self.0.card_identifier(cards)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct HandCardId(pub CardObjectId);

impl HandCardId {
    pub fn new(object_id: ObjectId, card_id: CardDataIdentifier) -> Self {
        Self(CardObjectId { object_id, card_id })
    }
}

impl CardId for HandCardId {
    fn card_identifier(&self, cards: &AllCards) -> Option<CardDataIdentifier> {
        self.0.card_identifier(cards)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct StackCardId(pub CardObjectId);

impl StackCardId {
    pub fn new(object_id: ObjectId, card_id: CardDataIdentifier) -> Self {
        Self(CardObjectId { object_id, card_id })
    }
}

impl CardId for StackCardId {
    fn card_identifier(&self, cards: &AllCards) -> Option<CardDataIdentifier> {
        self.0.card_identifier(cards)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct BanishedCardId(pub CardObjectId);

impl BanishedCardId {
    pub fn new(object_id: ObjectId, card_id: CardDataIdentifier) -> Self {
        Self(CardObjectId { object_id, card_id })
    }
}

impl CardId for BanishedCardId {
    fn card_identifier(&self, cards: &AllCards) -> Option<CardDataIdentifier> {
        self.0.card_identifier(cards)
    }
}
