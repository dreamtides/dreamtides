use bon::Builder;
use core_data::identifiers::CardName;
use core_data::numerics::{Energy, Points, Spark};

#[derive(Default, Builder, Clone)]
pub struct TestPlayer {
    #[builder(into)]
    pub points: Option<Points>,
    #[builder(into)]
    pub energy: Option<Energy>,
    #[builder(into)]
    pub produced_energy: Option<Energy>,
    #[builder(into)]
    pub spark_bonus: Option<Spark>,
    #[builder(default)]
    pub hand: Vec<CardName>,
    #[builder(default)]
    pub battlefield: Vec<CardName>,
    #[builder(default)]
    pub void: Vec<CardName>,
}
