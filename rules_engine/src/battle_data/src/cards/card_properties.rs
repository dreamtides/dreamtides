use core_data::numerics::Spark;

#[derive(Clone, Debug)]
pub struct CardProperties {
    /// Current spark value for this card
    pub spark: Spark,
}
