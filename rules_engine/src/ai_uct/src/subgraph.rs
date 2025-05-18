use std::collections::HashMap;

use battle_state::actions::battle_actions::BattleAction;
use petgraph::prelude::NodeIndex;
use petgraph::visit::{self, DfsEvent, EdgeRef};

use crate::uct_tree::SearchGraph;

/// Returns the sub-tree of the tree based at `root` which goes through the
/// edge tagged with the provided `action`.
pub fn extract(
    graph: &SearchGraph,
    root: NodeIndex,
    action: BattleAction,
) -> (SearchGraph, NodeIndex) {
    let Some(edge) = graph.edges(root).find(|e| e.weight().action == action) else {
        panic!("Child not found");
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

    (new_graph, new_root)
}
