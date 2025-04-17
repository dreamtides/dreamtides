use core_data::identifiers::CardId;

use crate::battle_cards::card_id::{
    BanishedCardId, CardIdType, CharacterId, DeckCardId, HandCardId, StackCardId, VoidCardId,
};
use crate::battle_cards::zone::Zone;

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

impl CardIdType for CardInstanceId {
    fn card_id(self) -> CardId {
        match self {
            Self::Banished(id) => id.card_id(),
            Self::Battlefield(id) => id.card_id(),
            Self::Deck(id) => id.card_id(),
            Self::Hand(id) => id.card_id(),
            Self::Stack(id) => id.card_id(),
            Self::Void(id) => id.card_id(),
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
}
