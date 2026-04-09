use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use chrono::Utc;
use core_data::types::PlayerName;
use serde::Serialize;

#[derive(Serialize)]
pub struct DecisionLogEntryV3 {
    pub timestamp: String,
    pub player: String,
    pub mode: String,
    pub chosen_action: String,
    pub chosen_action_short: String,
    pub chosen_score: f64,
    pub elapsed_ms: u128,
    pub legal_action_count: usize,
    pub kept_candidate_count: usize,
    pub strategic_depth: u8,
    pub min_think_floor_hit: bool,
    pub fallback_used: bool,
    pub candidates: Vec<DecisionCandidateV3>,
}

#[derive(Clone, Serialize)]
pub struct DecisionCandidateV3 {
    pub description: String,
    pub first_action_short: String,
    pub policy_score: f64,
    pub value_score: f64,
    pub combined_score: f64,
}

pub struct WriteDecisionLogV3<'a> {
    pub battle: &'a BattleState,
    pub candidates: Vec<DecisionCandidateV3>,
    pub chosen_action: BattleAction,
    pub chosen_score: f64,
    pub elapsed: Duration,
    pub fallback_used: bool,
    pub legal_action_count: usize,
    pub min_think_floor_hit: bool,
    pub mode: &'a str,
    pub player: PlayerName,
    pub strategic_depth: u8,
}

pub fn write_decision_log(input: WriteDecisionLogV3<'_>) {
    if !input.battle.request_context.logging_options.log_ai_decisions {
        return;
    }
    let Some(log_dir) = &input.battle.request_context.logging_options.log_directory else {
        return;
    };

    let entry = DecisionLogEntryV3 {
        timestamp: Utc::now().to_rfc3339(),
        player: format!("{:?}", input.player),
        mode: input.mode.to_string(),
        chosen_action: format!("{:?}", input.chosen_action),
        chosen_action_short: input.chosen_action.battle_action_string(),
        chosen_score: input.chosen_score,
        elapsed_ms: input.elapsed.as_millis(),
        legal_action_count: input.legal_action_count,
        kept_candidate_count: input.candidates.len(),
        strategic_depth: input.strategic_depth,
        min_think_floor_hit: input.min_think_floor_hit,
        fallback_used: input.fallback_used,
        candidates: input.candidates,
    };
    let Ok(json) = serde_json::to_string(&entry) else {
        return;
    };
    let _ = fs::create_dir_all(log_dir);
    let path = log_dir.join("ai_strategic_v3_decisions.jsonl");
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let _ = writeln!(file, "{json}");
}
