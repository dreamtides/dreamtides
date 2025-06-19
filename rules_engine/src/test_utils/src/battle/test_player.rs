use bon::Builder;
use core_data::identifiers::CardName;
use core_data::numerics::{Energy, Points, Spark};

#[derive(Default, Builder, Clone)]
pub struct TestPlayer {
    pub points: Option<Points>,
    pub energy: Option<Energy>,
    pub produced_energy: Option<Energy>,
    pub spark_bonus: Option<Spark>,
    pub hand: Vec<CardName>,
    pub battlefield: Vec<CardName>,
    pub void: Vec<CardName>,
}
