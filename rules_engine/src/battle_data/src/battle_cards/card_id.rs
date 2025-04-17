use std::fmt::Debug;
use std::hash::Hash;

use core_data::identifiers::CardId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A trait for identifiers which correspond 1:1 with cards.
pub trait CardIdType: Hash + Eq + PartialEq + Debug + Ord + Copy {
    /// Returns ths associated Card Id for this type.
    fn card_id(self) -> CardId;
}

/// An identifier for an object while it is in a given zone. A new zone object
/// ID is assigned each time a card changes zones, meaning that it can be
/// used for targeting effects that end when the card changes zones.
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ObjectId(pub u32);

impl CardIdType for CardId {
    fn card_id(self) -> CardId {
        self
    }
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct CharacterId(pub CardId);

impl CardIdType for CharacterId {
    fn card_id(self) -> CardId {
        self.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct VoidCardId(pub CardId);

impl CardIdType for VoidCardId {
    fn card_id(self) -> CardId {
        self.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct DeckCardId(pub CardId);

impl CardIdType for DeckCardId {
    fn card_id(self) -> CardId {
        self.0
    }
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct HandCardId(pub CardId);

impl CardIdType for HandCardId {
    fn card_id(self) -> CardId {
        self.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct StackCardId(pub CardId);

impl CardIdType for StackCardId {
    fn card_id(self) -> CardId {
        self.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct BanishedCardId(pub CardId);

impl CardIdType for BanishedCardId {
    fn card_id(self) -> CardId {
        self.0
    }
}
