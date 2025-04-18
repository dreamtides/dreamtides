use ability_data::ability::Ability;
use core_data::identifiers::CardId;
use core_data::types::PlayerName;

use crate::battle::turn_data::TurnData;
use crate::battle_cards::card_id::{CharacterId, ObjectId, StackCardId};
use crate::battle_cards::card_properties::CardProperties;
use crate::battle_cards::zone::Zone;

#[derive(Clone, Debug)]
pub struct CardData {
    /// Unique identifier for this card
    pub id: CardId,

    /// The owner of this card.
    pub owner: PlayerName,

    /// The zone this card is currently in.
    pub zone: Zone,

    /// Unique identifier for this card within a zone.
    pub object_id: ObjectId,

    /// Properties of this card.
    pub properties: CardProperties,

    /// Abilities of this card.
    pub abilities: Vec<Ability>,

    /// Whether this card is revealed to its owner.
    pub revealed_to_owner: bool,

    /// Whether this card is revealed to its opponent.
    pub revealed_to_opponent: bool,

    /// The targets of this card while it is on the stack.
    pub targets: Vec<TargetId>,

    /// Turn on which this card last entered its current zone.
    ///
    /// Do not mutate this field directly, use the `move_card` module instead.
    pub turn_entered_current_zone: TurnData,
}

impl CardData {
    /// Creates a new CardData instance
    pub fn new(
        id: CardId,
        owner: PlayerName,
        zone: Zone,
        object_id: ObjectId,
        properties: CardProperties,
        abilities: Vec<Ability>,
    ) -> Self {
        CardData {
            id,
            owner,
            zone,
            object_id,
            properties,
            abilities,
            revealed_to_owner: true,
            revealed_to_opponent: false,
            targets: Vec::new(),
            turn_entered_current_zone: TurnData::default(),
        }
    }

    /// Whether this card is revealed to the given player.
    ///
    /// Note that cards in Dreamtides currently have no concept of "face down"
    /// or "face up", they're simply revealed or not revealed to a given player.
    pub fn is_revealed_to(&self, player_name: PlayerName) -> bool {
        match (self.owner, player_name) {
            (PlayerName::User, PlayerName::User) => self.revealed_to_owner,
            (PlayerName::User, PlayerName::Enemy) => self.revealed_to_opponent,
            (PlayerName::Enemy, PlayerName::User) => self.revealed_to_opponent,
            (PlayerName::Enemy, PlayerName::Enemy) => self.revealed_to_owner,
        }
    }

    /// The controller of this card.
    pub fn controller(&self) -> PlayerName {
        self.owner
    }
}

#[derive(Clone, Debug)]
pub enum TargetId {
    StackCard(StackCardId),
    Character(CharacterId),
}
