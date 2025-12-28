use std::fmt::{self, Debug};
use std::hash::Hash;

use core_data::identifiers::AbilityNumber;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A trait for identifiers which correspond 1:1 with cards.
pub trait CardIdType: Hash + Eq + PartialEq + Debug + Ord + Copy {
    /// Returns ths associated Card Id for this type.
    fn card_id(self) -> CardId;

    fn from_card_id(card_id: CardId) -> Self;
}

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

/// Identifies an ability of a card.
#[derive(
    Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, JsonSchema,
)]
pub struct AbilityId {
    pub card_id: CardId,
    pub ability_number: AbilityNumber,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct CharacterId(pub CardId);

/// Identifies an activated ability of a character.
#[derive(
    Debug, Copy, Clone, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, JsonSchema,
)]
pub struct ActivatedAbilityId {
    pub character_id: CharacterId,
    pub ability_number: AbilityNumber,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct VoidCardId(pub CardId);

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct BattleDeckCardId(pub CardId);

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct HandCardId(pub CardId);

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct StackCardId(pub CardId);

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct BanishedCardId(pub CardId);

impl CardIdType for CardId {
    fn card_id(self) -> CardId {
        self
    }

    fn from_card_id(card_id: CardId) -> Self {
        card_id
    }
}

impl CardIdType for CharacterId {
    fn card_id(self) -> CardId {
        self.0
    }

    fn from_card_id(card_id: CardId) -> Self {
        Self(card_id)
    }
}

impl From<CharacterId> for CardId {
    fn from(value: CharacterId) -> Self {
        value.0
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

impl CardIdType for VoidCardId {
    fn card_id(self) -> CardId {
        self.0
    }

    fn from_card_id(card_id: CardId) -> Self {
        Self(card_id)
    }
}

impl From<VoidCardId> for CardId {
    fn from(value: VoidCardId) -> Self {
        value.0
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

impl CardIdType for BattleDeckCardId {
    fn card_id(self) -> CardId {
        self.0
    }

    fn from_card_id(card_id: CardId) -> Self {
        Self(card_id)
    }
}

impl fmt::Display for BattleDeckCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "D{:?}", self.0)
    }
}

impl fmt::Debug for BattleDeckCardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl CardIdType for HandCardId {
    fn card_id(self) -> CardId {
        self.0
    }

    fn from_card_id(card_id: CardId) -> Self {
        Self(card_id)
    }
}

impl From<HandCardId> for CardId {
    fn from(value: HandCardId) -> Self {
        value.0
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

impl From<StackCardId> for CardId {
    fn from(value: StackCardId) -> Self {
        value.0
    }
}

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

impl From<BanishedCardId> for CardId {
    fn from(value: BanishedCardId) -> Self {
        value.0
    }
}
