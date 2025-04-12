use core_data::numerics::{Energy, Spark};

#[derive(Clone, Debug)]
pub struct CardProperties {
    /// Current spark value for this card
    pub spark: Option<Spark>,

    /// Cost of this card
    pub cost: Option<Energy>,
}
