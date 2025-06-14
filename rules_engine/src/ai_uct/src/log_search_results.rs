use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::RequestContext;
use core_data::types::PlayerName;
use petgraph::dot::Dot;
use petgraph::prelude::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Graph;
use tracing::{debug, error, instrument};

use crate::uct_tree::SearchGraph;

/// Logs search results & best available actions.
///
/// Creates a simplified graph with nodes showing only the total reward values
/// formatted to 1 decimal place, and edges showing the battle actions.
/// The graph is limited to nodes within 3 edges of the root.

#[instrument(
    name = "log_results_diagram",
    level = "debug",
    skip(graph, root, action_taken, request_context)
)]
pub fn log_results_diagram(
    graph: &SearchGraph,
    root: NodeIndex,
    action_taken: BattleAction,
    request_context: &RequestContext,
) {
    // Create a simplified graph for logging
    let logging_graph = graph_for_logging(graph, root, action_taken);
    let dot = Dot::with_config(&logging_graph, &[]);
    let Ok(output_path) = get_dot_file_path(request_context) else {
        error!("Failed to create dot file path");
        return;
    };
    match File::create(&output_path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(format!("{:?}", dot).as_bytes()) {
                error!("Failed to write to dot file: {}", e);
            } else {
                debug!("Search graph written to {}", output_path.display());
            }
        }
        Err(e) => {
            error!("Failed to create dot file: {}", e);
        }
    }
}

fn graph_for_logging(
    graph: &SearchGraph,
    root: NodeIndex,
    action_taken: BattleAction,
) -> Graph<String, String> {
    let mut new_graph = Graph::<String, String>::new();
    let mut node_map = HashMap::new();
    let mut depth_map = HashMap::new();

    let total_reward = graph[root].total_reward.0;
    let formatted_reward = format!("{}@{:.1}", action_taken.battle_action_string(), total_reward);
    let new_root = new_graph.add_node(formatted_reward);
    node_map.insert(root, new_root);
    depth_map.insert(root, 0);

    // Instead of using BFS directly, we'll implement a level-by-level traversal
    // to ensure we don't exceed depth 3 and properly handle all nodes
    let mut current_level = vec![root];
    let mut depth = 0;

    // Process up to depth 3
    while !current_level.is_empty() && depth < 3 {
        let mut next_level = Vec::new();

        for &node in &current_level {
            // Get the corresponding node in the new graph
            // This should always exist since we're processing level by level
            let new_source = *node_map.get(&node).expect("Node should exist in map");

            // Process all outgoing edges
            for edge in graph.edges(node) {
                let target = edge.target();

                // Create a new node in the new graph if it doesn't exist
                let new_target = if let Some(&existing) = node_map.get(&target) {
                    existing
                } else {
                    let total_reward = graph[target].total_reward.0;
                    let formatted_reward = format!(
                        "{}@{:.1}",
                        match graph[target].player {
                            PlayerName::One => "P1",
                            PlayerName::Two => "P2",
                        },
                        total_reward
                    );
                    let new_node = new_graph.add_node(formatted_reward);
                    node_map.insert(target, new_node);
                    next_level.push(target);
                    new_node
                };

                new_graph.add_edge(
                    new_source,
                    new_target,
                    edge.weight().action.battle_action_string(),
                );
            }
        }

        // Move to the next level
        current_level = next_level;
        depth += 1;
    }

    new_graph
}

fn get_dot_file_path(request_context: &RequestContext) -> Result<PathBuf, String> {
    match &request_context.logging_options.log_directory {
        Some(log_dir) => Ok(log_dir.join("search_graph.dot")),
        None => Err("No log directory specified in RequestContext".to_string()),
    }
}
