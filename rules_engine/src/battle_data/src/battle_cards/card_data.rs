use core_data::identifiers::CardIdentity;
use core_data::numerics::TurnId;
use core_data::types::PlayerName;

use crate::battle::turn_data::TurnData;
use crate::battle_cards::card_id::{CharacterId, StackCardId};
use crate::battle_cards::card_instance_id::CardInstanceId;
use crate::battle_cards::card_properties::CardProperties;
use crate::battle_cards::zone::Zone;

#[derive(Clone, Debug)]
pub struct CardData {
    /// Unique identifier for this card within a zone.
    pub id: CardInstanceId,

    /// Identifier for the name & rules text for this card.
    pub identity: CardIdentity,

    /// The owner of this card.
    pub owner: PlayerName,

    /// Properties of this card.
    pub properties: CardProperties,

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
    pub fn new(
        id: CardInstanceId,
        identity: CardIdentity,
        owner: PlayerName,
        properties: CardProperties,
    ) -> Self {
        Self {
            id,
            identity,
            owner,
            properties,
            revealed_to_owner: false,
            revealed_to_opponent: false,
            targets: Vec::new(),
            turn_entered_current_zone: TurnData {
                active_player: PlayerName::User,
                turn_id: TurnId(0),
            },
        }
    }

    /// The zone this card is currently in.
    pub fn zone(&self) -> Zone {
        self.id.zone()
    }

    /// Whether this card is revealed to the given player.
    ///
    /// Note that cards in Dreamtides currently have no concept of "face down"
    /// or "face up", they're simply revealed or not revealed to a given player.
    pub fn is_revealed_to(&self, player_name: PlayerName) -> bool {
        match player_name {
            PlayerName::User => self.revealed_to_owner,
            PlayerName::Enemy => self.revealed_to_opponent,
        }
    }

    /// Set whether this card is revealed to the given player.
    pub fn set_revealed_to(&mut self, player_name: PlayerName, revealed: bool) {
        match player_name {
            PlayerName::User => self.revealed_to_owner = revealed,
            PlayerName::Enemy => self.revealed_to_opponent = revealed,
        }
    }

    /// The controller of this card.
    pub fn controller(&self) -> PlayerName {
        self.owner
    }

    pub(crate) fn internal_set_id(&mut self, id: CardInstanceId) {
        self.id = id;
    }
}

#[derive(Clone, Debug)]
pub enum TargetId {
    StackCard(StackCardId),
    Character(CharacterId),
}
