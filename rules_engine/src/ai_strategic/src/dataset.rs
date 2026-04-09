use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub type FeatureMap = BTreeMap<String, f64>;

#[derive(Clone, Serialize, Deserialize)]
pub struct PolicyCandidateRow {
    pub action: String,
    pub action_short: String,
    pub action_features: FeatureMap,
    pub avg_reward: f64,
    pub chosen: bool,
    pub visit_count: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PolicyTrainingRow {
    pub candidates: Vec<PolicyCandidateRow>,
    pub chosen_action_short: String,
    pub legal_action_count: usize,
    pub player: String,
    pub seed: u64,
    pub state_features: FeatureMap,
    pub turn_id: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ValueTrainingRow {
    pub outcome: f64,
    pub player: String,
    pub seed: u64,
    pub state_features: FeatureMap,
    pub turn_id: u32,
}
