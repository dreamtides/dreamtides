use core_data::numerics::Spark;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CharacterState {
    pub spark: Spark,
}
