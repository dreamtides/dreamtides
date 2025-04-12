use core_data::identifiers::CardDataIdentifier;

use crate::cards::all_cards::AllCards;
use crate::cards::card_id::{
    BanishedCardId, CardId, CharacterId, DeckCardId, HandCardId, ObjectId, StackCardId, VoidCardId,
};
use crate::cards::zone::Zone;

/// An identifier for a card while it is in a given zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CardInstanceId {
    Banished(BanishedCardId),
    Battlefield(CharacterId),
    Deck(DeckCardId),
    Hand(HandCardId),
    Stack(StackCardId),
    Void(VoidCardId),
}

impl CardId for CardInstanceId {
    fn card_identifier(&self, cards: &AllCards) -> Option<CardDataIdentifier> {
        match self {
            Self::Banished(id) => id.card_identifier(cards),
            Self::Battlefield(id) => id.card_identifier(cards),
            Self::Deck(id) => id.card_identifier(cards),
            Self::Hand(id) => id.card_identifier(cards),
            Self::Stack(id) => id.card_identifier(cards),
            Self::Void(id) => id.card_identifier(cards),
        }
    }
}

impl CardInstanceId {
    pub fn zone(&self) -> Zone {
        match self {
            Self::Banished(_) => Zone::Banished,
            Self::Battlefield(_) => Zone::Battlefield,
            Self::Deck(_) => Zone::Deck,
            Self::Hand(_) => Zone::Hand,
            Self::Stack(_) => Zone::Stack,
            Self::Void(_) => Zone::Void,
        }
    }

    pub fn object_id(&self) -> ObjectId {
        match self {
            Self::Banished(id) => id.0.object_id,
            Self::Battlefield(id) => id.0.object_id,
            Self::Deck(id) => id.0.object_id,
            Self::Hand(id) => id.0.object_id,
            Self::Stack(id) => id.0.object_id,
            Self::Void(id) => id.0.object_id,
        }
    }

    /// Returns card identifier for use in UI.
    ///
    /// Note that this bypasses existence checks for the ID.
    pub fn card_identifier_for_display(&self) -> CardDataIdentifier {
        match self {
            Self::Banished(id) => id.0.card_id,
            Self::Battlefield(id) => id.0.card_id,
            Self::Deck(id) => id.0.card_id,
            Self::Hand(id) => id.0.card_id,
            Self::Stack(id) => id.0.card_id,
            Self::Void(id) => id.0.card_id,
        }
    }
}
