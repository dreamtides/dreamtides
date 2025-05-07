use core_data::numerics::Spark;

use crate::battle::card_id::CharacterId;

#[derive(Clone, Debug)]
pub struct CharacterState {
    pub id: CharacterId,
    pub spark: Option<Spark>,
}
