use core_data::numerics::Spark;

use crate::cards::card_id::ObjectId;

#[derive(Clone, Debug)]
pub struct CardData {
    pub object_id: ObjectId,
    pub spark: Spark,
}
