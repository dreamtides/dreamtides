use std::collections::{BTreeMap, VecDeque};
use std::fs::OpenOptions;
use std::io::Write;

use battle_queries::battle_card_queries::card;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::{BattleState, RequestContext};
use battle_state::battle::card_id::CharacterId;
use core_data::types::PlayerName;
use petgraph::prelude::NodeIndex;
use petgraph::visit::EdgeRef;
use serde::Serialize;

use crate::uct_tree::SearchGraph;

#[derive(Serialize)]
pub struct DecisionLogEntry {
    pub timestamp: String,
    pub player: String,
    pub chosen_action: String,
    pub chosen_action_short: String,
    pub chosen_avg_reward: f64,
    pub game_state: GameStateSnapshot,
    pub budget: BudgetDetails,
    pub action_results: Vec<ActionResult>,
}

#[derive(Serialize)]
pub struct ActionResult {
    pub action: String,
    pub action_short: String,
    pub total_reward: f64,
    pub visit_count: u32,
    pub avg_reward: f64,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub tree_node_count: usize,
    pub tree_max_depth: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth_stats: Option<Vec<DepthLevelStats>>,
}

#[derive(Serialize)]
pub struct GameStateSnapshot {
    pub turn_id: u32,
    pub active_player: String,
    pub phase: String,
    pub player_one: PlayerSnapshot,
    pub player_two: PlayerSnapshot,
}

#[derive(Serialize)]
pub struct PlayerSnapshot {
    pub points: u32,
    pub points_to_win: u32,
    pub current_energy: u32,
    pub produced_energy: u32,
    pub hand_size: usize,
    pub battlefield: BattlefieldSnapshot,
}

#[derive(Serialize)]
pub struct BattlefieldSnapshot {
    pub front: Vec<Option<CharacterSnapshot>>,
    pub back: Vec<Option<CharacterSnapshot>>,
}

#[derive(Serialize)]
pub struct CharacterSnapshot {
    pub name: String,
    pub spark: u32,
    pub id: usize,
}

#[derive(Serialize)]
pub struct BudgetDetails {
    pub iterations_per_action: u32,
    pub base_iterations: u32,
    pub total_iterations: u32,
    pub num_actions: usize,
    pub multiplier: f64,
    pub multiplier_reason: String,
    pub num_threads: usize,
}

/// Per-depth statistics collected during MCTS tree traversal.
#[derive(Clone, Serialize)]
pub struct DepthLevelStats {
    pub depth: u32,
    pub player: String,
    pub expansions: u32,
    pub selections: u32,
    pub tried_actions: BTreeMap<String, u32>,
}

/// Accumulates tree traversal statistics across iterations.
#[derive(Default)]
pub struct TreeTraversalAccumulator {
    levels: Vec<DepthLevel>,
}

/// Serializes a [DecisionLogEntry] and appends it to `ai_decisions.jsonl`.
pub fn write_decision_log(entry: &DecisionLogEntry, request_context: &RequestContext) {
    let Some(log_dir) = &request_context.logging_options.log_directory else {
        return;
    };
    let path = log_dir.join("ai_decisions.jsonl");
    let Ok(json) = serde_json::to_string(entry) else {
        return;
    };
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) else {
        return;
    };
    let _ = writeln!(file, "{json}");
}

/// Builds a [GameStateSnapshot] from the current battle state.
pub fn build_game_state_snapshot(battle: &BattleState) -> GameStateSnapshot {
    GameStateSnapshot {
        turn_id: battle.turn.turn_id.0,
        active_player: format!("{:?}", battle.turn.active_player),
        phase: format!("{:?}", battle.phase),
        player_one: build_player_snapshot(battle, PlayerName::One),
        player_two: build_player_snapshot(battle, PlayerName::Two),
    }
}

/// Computes the maximum depth of a search tree via BFS.
pub fn compute_tree_depth(graph: &SearchGraph, root: NodeIndex) -> u32 {
    let mut max_depth = 0u32;
    let mut queue = VecDeque::new();
    queue.push_back((root, 0u32));
    while let Some((node, depth)) = queue.pop_front() {
        max_depth = max_depth.max(depth);
        for edge in graph.edges(node) {
            queue.push_back((edge.target(), depth + 1));
        }
    }
    max_depth
}

impl TreeTraversalAccumulator {
    /// Records an expansion (new node created) at the given depth.
    pub fn record_expansion(&mut self, depth: usize, player: PlayerName, action: &BattleAction) {
        self.ensure_depth(depth, player);
        self.levels[depth].expansions += 1;
        *self.levels[depth].expanded_actions.entry(action.battle_action_string()).or_default() += 1;
    }

    /// Records a selection (existing node chosen via best_child) at the given
    /// depth.
    pub fn record_selection(&mut self, depth: usize, player: PlayerName) {
        self.ensure_depth(depth, player);
        self.levels[depth].selections += 1;
    }

    /// Converts accumulated stats into serializable depth level stats.
    pub fn into_depth_stats(self) -> Vec<DepthLevelStats> {
        self.levels
            .into_iter()
            .enumerate()
            .filter(|(_, level)| level.expansions > 0 || level.selections > 0)
            .map(|(i, level)| DepthLevelStats {
                depth: i as u32,
                player: format!("{:?}", level.player.unwrap_or(PlayerName::One)),
                expansions: level.expansions,
                selections: level.selections,
                tried_actions: level.expanded_actions,
            })
            .collect()
    }

    fn ensure_depth(&mut self, depth: usize, player: PlayerName) {
        while self.levels.len() <= depth {
            self.levels.push(DepthLevel::default());
        }
        if self.levels[depth].player.is_none() {
            self.levels[depth].player = Some(player);
        }
    }
}

#[derive(Default)]
struct DepthLevel {
    player: Option<PlayerName>,
    expansions: u32,
    selections: u32,
    expanded_actions: BTreeMap<String, u32>,
}

fn build_player_snapshot(battle: &BattleState, player: PlayerName) -> PlayerSnapshot {
    let player_state = battle.players.player(player);
    let bf = battle.cards.battlefield(player);
    PlayerSnapshot {
        points: player_state.points.0,
        points_to_win: battle.rules_config.points_to_win.0,
        current_energy: player_state.current_energy.0,
        produced_energy: player_state.produced_energy.0,
        hand_size: battle.cards.hand(player).len(),
        battlefield: BattlefieldSnapshot {
            front: bf
                .front
                .iter()
                .map(|slot| build_character_snapshot(battle, player, slot))
                .collect(),
            back: bf
                .back
                .iter()
                .map(|slot| build_character_snapshot(battle, player, slot))
                .collect(),
        },
    }
}

fn build_character_snapshot(
    battle: &BattleState,
    player: PlayerName,
    slot: &Option<CharacterId>,
) -> Option<CharacterSnapshot> {
    let character_id = (*slot)?;
    Some(CharacterSnapshot {
        name: card::get_definition(battle, character_id).displayed_name.clone(),
        spark: battle.cards.spark(player, character_id).map_or(0, |s| s.0),
        id: character_id.0.0,
    })
}
