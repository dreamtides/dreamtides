use std::sync::OnceLock;

use serde::Deserialize;
use serde_json::Value;

use crate::hybrid_dataset::FeatureMap;

const POLICY_MODEL_JSON: &str =
    include_str!("../model_artifacts/monte_carlo_hybrid_v1_policy.json");

#[derive(Deserialize)]
struct ModelArtifact {
    feature_names: Vec<String>,
    model: Value,
}

pub struct LoadedModel {
    feature_names: Vec<String>,
    model: Value,
}

static POLICY_MODEL: OnceLock<Option<LoadedModel>> = OnceLock::new();

pub fn load_model() -> Option<&'static LoadedModel> {
    POLICY_MODEL.get_or_init(|| parse_model(POLICY_MODEL_JSON)).as_ref()
}

impl LoadedModel {
    pub fn score(&self, features: &FeatureMap) -> f64 {
        let values: Vec<_> = self
            .feature_names
            .iter()
            .map(|feature_name| *features.get(feature_name).unwrap_or(&0.0))
            .collect();
        self.model["tree_info"]
            .as_array()
            .map(|trees| trees.iter().map(|tree| eval_tree(&tree["tree_structure"], &values)).sum())
            .unwrap_or_default()
    }
}

fn parse_model(json: &str) -> Option<LoadedModel> {
    let artifact: ModelArtifact = serde_json::from_str(json).ok()?;
    if artifact.feature_names.is_empty()
        || artifact.model["tree_info"].as_array().is_none_or(Vec::is_empty)
    {
        return None;
    }
    Some(LoadedModel { feature_names: artifact.feature_names, model: artifact.model })
}

fn eval_tree(node: &Value, values: &[f64]) -> f64 {
    if let Some(leaf_value) = node.get("leaf_value").and_then(Value::as_f64) {
        return leaf_value;
    }

    let feature = node["split_feature"].as_u64().unwrap_or_default() as usize;
    let threshold = node["threshold"].as_f64().unwrap_or_default();
    let decision_type = node["decision_type"].as_str().unwrap_or("<=");
    let value = values.get(feature).copied().unwrap_or_default();
    let go_left = match decision_type {
        "<=" => value <= threshold,
        "<" => value < threshold,
        ">" => value > threshold,
        ">=" => value >= threshold,
        "==" => (value - threshold).abs() < 1e-9,
        _ => value <= threshold,
    };

    if go_left {
        eval_tree(&node["left_child"], values)
    } else {
        eval_tree(&node["right_child"], values)
    }
}
