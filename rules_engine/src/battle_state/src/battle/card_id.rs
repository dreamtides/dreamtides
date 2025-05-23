use std::fmt::{self, Debug};
use std::hash::Hash;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
)]
pub struct CardId(pub usize);

/// A trait for identifiers which correspond 1:1 with cards.
pub trait CardIdType: Hash + Eq + PartialEq + Debug + Ord + Copy {
    /// Returns ths associated Card Id for this type.
    fn card_id(self) -> CardId;

    fn from_card_id(card_id: CardId) -> Self;
}

impl CardIdType for CardId {
    fn card_id(self) -> CardId {
        self
    }

    fn from_card_id(card_id: CardId) -> Self {
        card_id
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct CharacterId(pub CardId);

impl CardIdType for CharacterId {
    fn card_id(self) -> CardId {
        self.0
    }

    fn from_card_id(card_id: CardId) -> Self {
        Self(card_id)
    }
}

impl fmt::Display for CharacterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "C{:?}", self.0)
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

    fn from_card_id(card_id: CardId) -> Self {
        Self(card_id)
    }
}

impl fmt::Display for VoidCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V{:?}", self.0)
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

    fn from_card_id(card_id: CardId) -> Self {
        Self(card_id)
    }
}

impl fmt::Display for DeckCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "D{:?}", self.0)
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

    fn from_card_id(card_id: CardId) -> Self {
        Self(card_id)
    }
}

impl fmt::Display for HandCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "H{:?}", self.0)
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

    fn from_card_id(card_id: CardId) -> Self {
        Self(card_id)
    }
}

impl fmt::Display for StackCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "S{:?}", self.0)
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

    fn from_card_id(card_id: CardId) -> Self {
        Self(card_id)
    }
}

impl fmt::Display for BanishedCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "B{:?}", self.0)
    }
}

impl fmt::Debug for BanishedCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
