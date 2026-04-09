use std::collections::BTreeMap;

use serde::Serialize;

pub type FeatureMap = BTreeMap<String, f64>;

#[derive(Serialize)]
pub struct PolicyTrainingRow {
    pub action: String,
    pub action_short: String,
    pub avg_reward: f64,
    pub chosen: bool,
    pub legal_action_count: usize,
    pub player: String,
    pub seed: u64,
    pub state_features: FeatureMap,
    pub action_features: FeatureMap,
    pub turn_id: u32,
    pub visit_count: u32,
    pub visit_fraction: f64,
}
