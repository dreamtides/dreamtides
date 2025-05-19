use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;
use ordered_float::OrderedFloat;
use petgraph::prelude::NodeIndex;
use petgraph::visit::{self, DfsEvent, EdgeRef};
use tracing::debug;

use crate::uct_config::UctConfig;
use crate::uct_search;
use crate::uct_tree::{GraphWithRoot, SearchGraph, SearchNode};

static SEARCH_GRAPH: OnceLock<Mutex<Option<GraphWithRoot>>> = OnceLock::new();

/// Equivalent to [search] for use where there is no previous search graph
/// available.
pub fn search_from_empty(
    initial_battle: &BattleState,
    player: PlayerName,
    config: &UctConfig,
) -> BattleAction {
    let mut graph = SearchGraph::default();
    let root = graph.add_node(SearchNode {
        player,
        total_reward: OrderedFloat(0.0),
        visit_count: 0,
        tried: Vec::new(),
    });
    uct_search::search(initial_battle, player, config, &mut graph, root)
}

/// Equivalent to [search] using a previously saved search graph.
pub fn search_from_saved(
    initial_battle: &BattleState,
    player: PlayerName,
    config: &UctConfig,
) -> BattleAction {
    if let Some(saved) = get_search_graph() {
        uct_search::search(initial_battle, player, config, &mut saved.graph.clone(), saved.root)
    } else {
        search_from_empty(initial_battle, player, config)
    }
}

fn get_search_graph() -> Option<GraphWithRoot> {
    SEARCH_GRAPH.get_or_init(|| Mutex::new(None)).lock().unwrap().clone()
}

pub fn on_action_performed(action: BattleAction) {
    let mut guard = SEARCH_GRAPH.get_or_init(|| Mutex::new(None)).lock().unwrap();
    if let Some(graph_with_root) = guard.as_ref() {
        debug!(?action, "Extracting subtree for action");
        match extract_subtree(&graph_with_root.graph, graph_with_root.root, action) {
            Some(subtree) => {
                *guard = Some(subtree);
            }
            None => {
                debug!("Invalidating search graph");
                *guard = None;
            }
        }
    } else {
        debug!(?action, "No active graph, ignoring action")
    }
}

pub fn on_search_completed(graph: &mut SearchGraph, root: NodeIndex) {
    debug!("Search completed, storing graph");
    let new_graph = graph.clone();
    let with_root = GraphWithRoot { graph: new_graph, root };
    *SEARCH_GRAPH.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(with_root);
}

/// Returns the sub-tree of the tree based at `root` which goes through the
/// edge tagged with the provided `action`.
/// Returns None if no child with the specified action is found.
fn extract_subtree(
    graph: &SearchGraph,
    root: NodeIndex,
    action: BattleAction,
) -> Option<GraphWithRoot> {
    let Some(edge) = graph.edges(root).find(|e| e.weight().action == action) else {
        debug!("Child not found: {action:?}, invalidating search graph");
        return None;
    };
    let target = edge.target();
    let mut new_graph = SearchGraph::new();
    let mut node_map = HashMap::new();
    let new_root = new_graph.add_node(graph[target].clone());
    node_map.insert(target, new_root);

    visit::depth_first_search(graph, Some(target), |event| {
        match event {
            DfsEvent::Discover(node, _) => {
                if node != target && !node_map.contains_key(&node) {
                    let new_node = new_graph.add_node(graph[node].clone());
                    node_map.insert(node, new_node);
                }
            }
            DfsEvent::TreeEdge(source, target) => {
                // Ensure both nodes exist in the node_map
                node_map.entry(source).or_insert_with(|| new_graph.add_node(graph[source].clone()));

                node_map.entry(target).or_insert_with(|| new_graph.add_node(graph[target].clone()));

                // Now we can safely get the corresponding nodes in the new graph
                let new_source = *node_map.get(&source).unwrap();
                let new_target = *node_map.get(&target).unwrap();

                // Find the edge in the original graph
                if let Some(edge_ref) = graph.find_edge(source, target) {
                    // Add the edge with its weight to the new graph
                    new_graph.add_edge(new_source, new_target, graph[edge_ref]);
                }
            }
            _ => {}
        }
    });

    Some(GraphWithRoot { graph: new_graph, root: new_root })
}
