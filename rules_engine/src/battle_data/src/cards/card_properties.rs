use core_data::numerics::{Energy, Spark};

use crate::cards::card_types::CardType;

#[derive(Clone, Debug)]
pub struct CardProperties {
    /// Type & subtype of this card
    pub card_type: CardType,

    /// Current spark value for this card
    pub spark: Option<Spark>,

    /// Cost of this card
    pub cost: Option<Energy>,

    /// Can this card be played during the opponent's turn?
    pub is_fast: bool,
}
