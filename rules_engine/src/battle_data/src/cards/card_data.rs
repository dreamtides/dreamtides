use core_data::numerics::Spark;
use core_data::types::PlayerName;

use crate::cards::card_instance_id::CardInstanceId;

#[derive(Clone, Debug)]
pub struct CardData {
    /// Unique identifier for this card within a zone.
    ///
    /// Do not modify this field.
    pub id: CardInstanceId,

    /// The owner of this card.
    pub owner: PlayerName,

    /// Current spark value for this card.
    pub spark: Spark,
}
