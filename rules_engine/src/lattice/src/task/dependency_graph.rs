use std::collections::{HashMap, HashSet, VecDeque};

use rusqlite::Connection;
use serde::Serialize;
use tracing::{debug, info, warn};

use crate::error::error_types::LatticeError;
use crate::index::document_queries;
use crate::index::document_types::DocumentRow;
use crate::index::link_queries::LinkType;

/// In-memory dependency graph for task blocking relationships.
///
/// The graph is bidirectional: if A blocks B, we store both A→B in `blocks`
/// and B→A in `blocked_by`. This enables efficient queries in both directions.
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    edges: HashMap<String, NodeEdges>,
}

/// Result of cycle detection.
#[derive(Debug, Clone)]
pub struct CycleResult {
    /// Whether a cycle was detected.
    pub has_cycle: bool,
    /// The cycle path if detected, formatted as "A → B → C → A".
    pub cycle_path: Option<String>,
    /// IDs involved in the cycle.
    pub involved_ids: Vec<String>,
}

/// A node in a dependency tree display.
#[derive(Debug, Clone, Serialize)]
pub struct TreeNode {
    /// The Lattice ID of this node.
    pub id: String,
    /// The task name (from frontmatter).
    pub name: Option<String>,
    /// The task state (open/blocked/closed).
    pub state: String,
    /// Children nodes in the tree.
    pub children: Vec<TreeNode>,
}

/// Direction for tree traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeDirection {
    /// Show what this task depends on (upstream).
    Upstream,
    /// Show what depends on this task (downstream).
    Downstream,
}

/// Builds a dependency tree for display.
///
/// The tree shows the dependency hierarchy starting from the given ID,
/// with state information for each node.
pub fn build_dependency_tree(
    conn: &Connection,
    graph: &DependencyGraph,
    root_id: &str,
    direction: TreeDirection,
    max_depth: Option<usize>,
) -> Result<TreeNode, LatticeError> {
    let mut visited = HashSet::new();
    build_tree_node(
        conn,
        graph,
        root_id,
        direction,
        max_depth.unwrap_or(usize::MAX),
        0,
        &mut visited,
    )
}

/// Validates that adding a blocking relationship would not create a cycle.
///
/// Call this before persisting a new blocked-by or blocking link.
pub fn validate_no_cycle_on_add(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
    link_type: LinkType,
) -> Result<(), LatticeError> {
    debug!(source_id, target_id, ?link_type, "Validating no cycle would be created");

    let mut graph = DependencyGraph::build_from_connection(conn)?;
    graph.add_edge(source_id, target_id, link_type);

    let result = graph.detect_cycle();
    if result.has_cycle {
        let cycle = result.cycle_path.unwrap_or_default();
        return Err(LatticeError::CircularDependency { cycle, involved_ids: result.involved_ids });
    }

    debug!("No cycle would be created by this edge");
    Ok(())
}

/// Represents edges in the dependency graph.
///
/// Each task can block multiple tasks and be blocked by multiple tasks.
/// This struct stores both directions for efficient traversal.
#[derive(Debug, Clone, Default)]
struct NodeEdges {
    /// Tasks that this node blocks (downstream dependencies).
    blocks: Vec<String>,
    /// Tasks that block this node (upstream dependencies).
    blocked_by: Vec<String>,
}

impl DependencyGraph {
    /// Creates a new empty dependency graph.
    pub fn new() -> Self {
        Self { edges: HashMap::new() }
    }

    /// Builds a dependency graph from the database.
    ///
    /// Loads all blocking and blocked-by links and constructs the bidirectional
    /// graph structure.
    pub fn build_from_connection(conn: &Connection) -> Result<Self, LatticeError> {
        debug!("Building dependency graph from database");
        let mut graph = Self::new();

        let all_links = load_all_dependency_links(conn)?;
        for (source_id, target_id, link_type) in all_links {
            graph.add_edge(&source_id, &target_id, link_type);
        }

        info!(node_count = graph.edges.len(), "Dependency graph built");
        Ok(graph)
    }

    /// Adds an edge to the graph.
    ///
    /// Maintains bidirectional representation: if A blocks B, we store
    /// A.blocks = [B] and B.blocked_by = [A].
    fn add_edge(&mut self, source_id: &str, target_id: &str, link_type: LinkType) {
        match link_type {
            LinkType::Blocking => {
                self.edges
                    .entry(source_id.to_string())
                    .or_default()
                    .blocks
                    .push(target_id.to_string());
                self.edges
                    .entry(target_id.to_string())
                    .or_default()
                    .blocked_by
                    .push(source_id.to_string());
            }
            LinkType::BlockedBy => {
                self.edges
                    .entry(source_id.to_string())
                    .or_default()
                    .blocked_by
                    .push(target_id.to_string());
                self.edges
                    .entry(target_id.to_string())
                    .or_default()
                    .blocks
                    .push(source_id.to_string());
            }
            _ => {}
        }
    }

    /// Detects cycles in the blocking dependency graph.
    ///
    /// Uses DFS with coloring (white/gray/black) to detect back edges.
    /// Returns the first cycle found, if any.
    pub fn detect_cycle(&self) -> CycleResult {
        debug!("Running cycle detection");
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();
        let mut parent_map: HashMap<String, String> = HashMap::new();

        let mut keys: Vec<&String> = self.edges.keys().collect();
        keys.sort();
        for start_id in keys {
            if visited.contains(start_id) {
                continue;
            }

            if let Some(cycle_ids) =
                self.dfs_cycle(start_id, &mut visited, &mut in_stack, &mut parent_map)
            {
                let cycle_path = format_cycle_path(&cycle_ids);
                warn!(cycle = cycle_path.as_str(), "Cycle detected in dependency graph");
                return CycleResult {
                    has_cycle: true,
                    cycle_path: Some(cycle_path),
                    involved_ids: cycle_ids,
                };
            }
        }

        debug!("No cycles detected");
        CycleResult { has_cycle: false, cycle_path: None, involved_ids: vec![] }
    }

    /// DFS helper for cycle detection.
    ///
    /// Returns Some(cycle_ids) if a cycle is found starting from `node`.
    fn dfs_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        in_stack: &mut HashSet<String>,
        parent_map: &mut HashMap<String, String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        in_stack.insert(node.to_string());

        if let Some(edges) = self.edges.get(node) {
            for neighbor in &edges.blocks {
                if !visited.contains(neighbor) {
                    parent_map.insert(neighbor.clone(), node.to_string());
                    if let Some(cycle) = self.dfs_cycle(neighbor, visited, in_stack, parent_map) {
                        return Some(cycle);
                    }
                } else if in_stack.contains(neighbor) {
                    return Some(extract_cycle(parent_map, node, neighbor));
                }
            }
        }

        in_stack.remove(node);
        None
    }

    /// Returns direct blockers of a task (tasks in blocked-by).
    pub fn get_blockers(&self, id: &str) -> Vec<String> {
        self.edges.get(id).map(|e| e.blocked_by.clone()).unwrap_or_default()
    }

    /// Returns tasks directly blocked by this task (tasks in blocking).
    pub fn get_blocking(&self, id: &str) -> Vec<String> {
        self.edges.get(id).map(|e| e.blocks.clone()).unwrap_or_default()
    }

    /// Returns all blockers transitively (transitive closure).
    pub fn get_all_blockers(&self, id: &str) -> Vec<String> {
        self.transitive_closure(id, TreeDirection::Upstream)
    }

    /// Returns all tasks blocked transitively (transitive closure).
    pub fn get_all_blocking(&self, id: &str) -> Vec<String> {
        self.transitive_closure(id, TreeDirection::Downstream)
    }

    /// Computes transitive closure in the given direction using BFS.
    fn transitive_closure(&self, start_id: &str, direction: TreeDirection) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        visited.insert(start_id.to_string());
        queue.push_back(start_id.to_string());

        while let Some(current) = queue.pop_front() {
            let neighbors = match direction {
                TreeDirection::Upstream => self.get_blockers(&current),
                TreeDirection::Downstream => self.get_blocking(&current),
            };

            for neighbor in neighbors {
                if visited.insert(neighbor.clone()) {
                    result.push(neighbor.clone());
                    queue.push_back(neighbor);
                }
            }
        }

        result
    }

    /// Returns a topological ordering of nodes reachable from the given ID.
    ///
    /// Uses Kahn's algorithm. Returns None if the subgraph contains a cycle.
    pub fn topological_order(&self, start_ids: &[String]) -> Option<Vec<String>> {
        let reachable = self.collect_reachable(start_ids);
        let mut reachable_sorted: Vec<&String> = reachable.iter().collect();
        reachable_sorted.sort();

        let mut in_degree: HashMap<String, usize> = HashMap::new();
        for id in &reachable_sorted {
            in_degree.insert((*id).clone(), 0);
        }

        for id in &reachable_sorted {
            for blocked in self.get_blocking(id) {
                if reachable.contains(&blocked) {
                    *in_degree.entry(blocked).or_default() += 1;
                }
            }
        }

        let mut queue: VecDeque<String> =
            in_degree.iter().filter(|(_, deg)| **deg == 0).map(|(id, _)| id.clone()).collect();
        let mut result = Vec::new();

        while let Some(node) = queue.pop_front() {
            result.push(node.clone());
            for blocked in self.get_blocking(&node) {
                if let Some(deg) = in_degree.get_mut(&blocked) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(blocked);
                    }
                }
            }
        }

        if result.len() == reachable.len() { Some(result) } else { None }
    }

    /// Collects all nodes reachable from start_ids in both directions.
    fn collect_reachable(&self, start_ids: &[String]) -> HashSet<String> {
        let mut visited = HashSet::new();
        let mut queue: VecDeque<String> = start_ids.iter().cloned().collect();

        while let Some(current) = queue.pop_front() {
            if !visited.insert(current.clone()) {
                continue;
            }
            for neighbor in self.get_blockers(&current) {
                queue.push_back(neighbor);
            }
            for neighbor in self.get_blocking(&current) {
                queue.push_back(neighbor);
            }
        }

        visited
    }

    /// Returns true if the given ID has any dependencies.
    pub fn has_dependencies(&self, id: &str) -> bool {
        self.edges.get(id).is_some_and(|e| !e.blocks.is_empty() || !e.blocked_by.is_empty())
    }

    /// Returns the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.edges.len()
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Loads all dependency links from the database.
fn load_all_dependency_links(
    conn: &Connection,
) -> Result<Vec<(String, String, LinkType)>, LatticeError> {
    debug!("Loading all dependency links");
    let mut results = Vec::new();

    let mut stmt = conn
        .prepare("SELECT source_id, target_id, link_type FROM links WHERE link_type IN ('blocking', 'blocked_by')")
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare dependency links query: {e}"),
        })?;

    let rows = stmt
        .query_map([], |row| {
            let source: String = row.get(0)?;
            let target: String = row.get(1)?;
            let link_type_str: String = row.get(2)?;
            Ok((source, target, link_type_str))
        })
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query dependency links: {e}"),
        })?;

    for row in rows {
        let (source, target, link_type_str) = row.map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to read dependency link row: {e}"),
        })?;
        if let Some(link_type) = parse_link_type(&link_type_str) {
            results.push((source, target, link_type));
        }
    }

    debug!(count = results.len(), "Dependency links loaded");
    Ok(results)
}

/// Parses a link type string from the database.
fn parse_link_type(s: &str) -> Option<LinkType> {
    match s {
        "blocked_by" => Some(LinkType::BlockedBy),
        "blocking" => Some(LinkType::Blocking),
        _ => None,
    }
}

/// Extracts cycle path from parent map when a back edge is found.
fn extract_cycle(parent_map: &HashMap<String, String>, end: &str, start: &str) -> Vec<String> {
    let mut cycle = vec![start.to_string()];
    let mut current = end;
    while current != start {
        cycle.push(current.to_string());
        current = parent_map.get(current).map(String::as_str).unwrap_or(start);
    }
    cycle.push(start.to_string());
    cycle.reverse();
    cycle
}

/// Formats cycle IDs into a display string like "A → B → C → A".
fn format_cycle_path(ids: &[String]) -> String {
    ids.join(" → ")
}

/// Recursive helper for tree building.
fn build_tree_node(
    conn: &Connection,
    graph: &DependencyGraph,
    id: &str,
    direction: TreeDirection,
    max_depth: usize,
    current_depth: usize,
    visited: &mut HashSet<String>,
) -> Result<TreeNode, LatticeError> {
    visited.insert(id.to_string());

    let (name, state) = if let Some(doc) = document_queries::lookup_by_id(conn, id)? {
        let state = compute_node_state(conn, graph, &doc)?;
        (Some(doc.name), state)
    } else {
        (None, "unknown".to_string())
    };

    let mut children = Vec::new();
    if current_depth < max_depth {
        let neighbors = match direction {
            TreeDirection::Upstream => graph.get_blockers(id),
            TreeDirection::Downstream => graph.get_blocking(id),
        };

        for neighbor_id in neighbors {
            if !visited.contains(&neighbor_id) {
                let child = build_tree_node(
                    conn,
                    graph,
                    &neighbor_id,
                    direction,
                    max_depth,
                    current_depth + 1,
                    visited,
                )?;
                children.push(child);
            }
        }
    }

    Ok(TreeNode { id: id.to_string(), name, state, children })
}

/// Computes display state for a document.
///
/// Returns "closed" if the document is closed, "blocked" if it has any
/// open blockers, or "open" otherwise.
fn compute_node_state(
    conn: &Connection,
    graph: &DependencyGraph,
    doc: &DocumentRow,
) -> Result<String, LatticeError> {
    if doc.is_closed {
        return Ok("closed".to_string());
    }

    // Check if any blockers are not closed
    for blocker_id in graph.get_blockers(&doc.id) {
        if let Some(blocker) = document_queries::lookup_by_id(conn, &blocker_id)?
            && !blocker.is_closed
        {
            return Ok("blocked".to_string());
        }
    }

    Ok("open".to_string())
}
