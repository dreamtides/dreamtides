use bon::Builder;
use core_data::numerics::{Energy, Points, Spark};

#[derive(Builder, Clone)]
pub struct TestPlayer {
    #[builder(into)]
    #[builder(default = 0)]
    pub points: Points,
    #[builder(into)]
    #[builder(default = 99)]
    pub energy: Energy,
    #[builder(into)]
    #[builder(default = 99)]
    pub produced_energy: Energy,
    #[builder(into)]
    #[builder(default = 0)]
    pub spark_bonus: Spark,
}

impl Default for TestPlayer {
    fn default() -> Self {
        Self {
            points: Points(0),
            energy: Energy(99),
            produced_energy: Energy(99),
            spark_bonus: Spark(0),
        }
    }
}
