use core_data::identifiers::CardIdentity;
use core_data::types::PlayerName;

use crate::cards::card_instance_id::CardInstanceId;
use crate::cards::card_properties::CardProperties;

#[derive(Clone, Debug)]
pub struct CardData {
    id: CardInstanceId,
    identity: CardIdentity,
    owner: PlayerName,
    properties: CardProperties,
    revealed_to_owner: bool,
    revealed_to_opponent: bool,
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
        }
    }

    /// Unique identifier for this card within a zone.
    pub fn id(&self) -> CardInstanceId {
        self.id
    }

    /// Identifier for the name & rules text for this card.
    pub fn identity(&self) -> CardIdentity {
        self.identity
    }

    /// The owner of this card.
    pub fn owner(&self) -> PlayerName {
        self.owner
    }

    /// Properties of this card.
    pub fn properties(&self) -> &CardProperties {
        &self.properties
    }

    /// Properties of this card.
    pub fn properties_mut(&mut self) -> &mut CardProperties {
        &mut self.properties
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

    pub(crate) fn internal_set_id(&mut self, id: CardInstanceId) {
        self.id = id;
    }
}
