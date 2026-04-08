use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use chrono::Utc;
use core_data::types::PlayerName;
use serde::Serialize;

use crate::evaluation::EvaluationFeature;

#[derive(Serialize)]
pub struct StrategicDecisionLogEntry {
    pub timestamp: String,
    pub player: String,
    pub mode: String,
    pub chosen_action: String,
    pub chosen_action_short: String,
    pub chosen_score: f64,
    pub elapsed_ms: u128,
    pub transposition_hits: usize,
    pub fallback_used: bool,
    pub candidates: Vec<StrategicCandidateLog>,
    pub evaluation_features: Vec<EvaluationFeature>,
}

#[derive(Clone, Serialize)]
pub struct StrategicCandidateLog {
    pub description: String,
    pub first_action: String,
    pub first_action_short: String,
    pub heuristic_score: f64,
    pub aggregate_score: f64,
    pub samples: usize,
}

pub struct WriteDecisionLog<'a> {
    pub battle: &'a BattleState,
    pub candidates: Vec<StrategicCandidateLog>,
    pub chosen_action: BattleAction,
    pub chosen_score: f64,
    pub elapsed: Duration,
    pub evaluation_features: Vec<EvaluationFeature>,
    pub fallback_used: bool,
    pub mode: &'a str,
    pub player: PlayerName,
    pub transposition_hits: usize,
}

pub fn write_decision_log(input: WriteDecisionLog<'_>) {
    if !input.battle.request_context.logging_options.log_ai_decisions {
        return;
    }
    let Some(log_dir) = &input.battle.request_context.logging_options.log_directory else {
        return;
    };

    let entry = StrategicDecisionLogEntry {
        timestamp: Utc::now().to_rfc3339(),
        player: format!("{:?}", input.player),
        mode: input.mode.to_string(),
        chosen_action: format!("{:?}", input.chosen_action),
        chosen_action_short: input.chosen_action.battle_action_string(),
        chosen_score: input.chosen_score,
        elapsed_ms: input.elapsed.as_millis(),
        transposition_hits: input.transposition_hits,
        fallback_used: input.fallback_used,
        candidates: input.candidates,
        evaluation_features: input.evaluation_features,
    };
    let Ok(json) = serde_json::to_string(&entry) else {
        return;
    };
    let path = log_dir.join("ai_strategic_decisions.jsonl");
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(file, "{json}");
}
