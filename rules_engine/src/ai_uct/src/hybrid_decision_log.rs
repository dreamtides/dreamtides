use std::fs::OpenOptions;
use std::io::Write;

use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;
use serde::Serialize;

use crate::decision_log::GameStateSnapshot;

#[derive(Serialize)]
pub struct DecisionLogEntryHybrid {
    pub average_rollout_length: f64,
    pub flat_prior: bool,
    pub game_state: GameStateSnapshot,
    pub legal_action_count: usize,
    pub player: String,
    pub rollout_cutoff_count: u32,
    pub rollout_pass_suppression_count: u32,
    pub root_actions: Vec<RootActionLog>,
    pub timestamp: String,
    pub total_iterations: u32,
}

#[derive(Serialize)]
pub struct RootActionLog {
    pub action: String,
    pub action_short: String,
    pub avg_reward: f64,
    pub draws: u32,
    pub iterations_allocated: u32,
    pub losses: u32,
    pub prior_score: f64,
    pub prior_share: f64,
    pub wins: u32,
}

pub fn write_decision_log(entry: &DecisionLogEntryHybrid, battle: &BattleState) {
    let Some(log_dir) = &battle.request_context.logging_options.log_directory else {
        return;
    };
    let path = log_dir.join("ai_hybrid_decisions.jsonl");
    let Ok(json) = serde_json::to_string(entry) else {
        return;
    };
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) else {
        return;
    };
    let _ = writeln!(file, "{json}");
}

pub fn player_string(player: PlayerName) -> String {
    format!("{player:?}")
}
