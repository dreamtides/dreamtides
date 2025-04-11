use crate::cards::card_id::{
    BanishedCardId, CharacterId, DeckCardId, HandCardId, ObjectId, StackCardId, VoidCardId,
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
}
