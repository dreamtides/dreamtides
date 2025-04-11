use core_data::identifiers::CardIdentity;
use core_data::types::PlayerName;

use crate::cards::card_instance_id::CardInstanceId;
use crate::cards::card_properties::CardProperties;

#[derive(Clone, Debug)]
pub struct CardData {
    /// Unique identifier for this card within a zone.
    ///
    /// Do not modify this field.
    pub id: CardInstanceId,

    /// Identifier for the name & rules text for this card.
    ///
    /// Do not modify this field.
    pub identity: CardIdentity,

    /// The owner of this card.
    pub owner: PlayerName,

    /// Properties of this card
    pub properties: CardProperties,
}
