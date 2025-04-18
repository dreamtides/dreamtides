use std::fmt::{self, Debug};
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

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct CharacterId(pub CardId);

impl CardIdType for CharacterId {
    fn card_id(self) -> CardId {
        self.0
    }
}

impl fmt::Display for CharacterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "C{}", self.0)
    }
}

impl fmt::Debug for CharacterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct VoidCardId(pub CardId);

impl CardIdType for VoidCardId {
    fn card_id(self) -> CardId {
        self.0
    }
}

impl fmt::Display for VoidCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V{}", self.0)
    }
}

impl fmt::Debug for VoidCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct DeckCardId(pub CardId);

impl CardIdType for DeckCardId {
    fn card_id(self) -> CardId {
        self.0
    }
}

impl fmt::Display for DeckCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "D{}", self.0)
    }
}

impl fmt::Debug for DeckCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct HandCardId(pub CardId);

impl CardIdType for HandCardId {
    fn card_id(self) -> CardId {
        self.0
    }
}

impl fmt::Display for HandCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "H{}", self.0)
    }
}

impl fmt::Debug for HandCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct StackCardId(pub CardId);

impl CardIdType for StackCardId {
    fn card_id(self) -> CardId {
        self.0
    }
}

impl fmt::Display for StackCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "S{}", self.0)
    }
}

impl fmt::Debug for StackCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct BanishedCardId(pub CardId);

impl CardIdType for BanishedCardId {
    fn card_id(self) -> CardId {
        self.0
    }
}

impl fmt::Display for BanishedCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "B{}", self.0)
    }
}

impl fmt::Debug for BanishedCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
