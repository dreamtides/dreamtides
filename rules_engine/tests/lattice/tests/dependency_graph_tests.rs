use lattice::document::frontmatter_schema::TaskType;
use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{InsertLink, LinkType};
use lattice::index::{connection_pool, document_queries, link_queries, schema_definition};
use lattice::task::dependency_graph::{
    DependencyGraph, TreeDirection, build_dependency_tree, validate_no_cycle_on_add,
};

fn create_test_db() -> rusqlite::Connection {
    let conn =
        connection_pool::open_memory_connection().expect("Failed to open in-memory connection");
    schema_definition::create_schema(&conn).expect("Failed to create schema");
    conn
}

fn insert_task(conn: &rusqlite::Connection, id: &str, path: &str) {
    let doc = InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        format!("task-{}", id.to_lowercase()),
        format!("Test task {id}"),
        Some(TaskType::Task),
        Some(2),
        None,
        None,
        None,
        format!("hash-{id}"),
        100,
    );
    document_queries::insert(conn, &doc).expect("Failed to insert document");
}

fn add_blocking(conn: &rusqlite::Connection, source: &str, target: &str) {
    let link = InsertLink {
        source_id: source,
        target_id: target,
        link_type: LinkType::Blocking,
        position: 0,
    };
    link_queries::insert_for_document(conn, &[link]).expect("Failed to insert link");
}

fn add_blocked_by(conn: &rusqlite::Connection, source: &str, target: &str) {
    let link = InsertLink {
        source_id: source,
        target_id: target,
        link_type: LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(conn, &[link]).expect("Failed to insert link");
}

#[test]
fn empty_graph_has_no_cycles() {
    let conn = create_test_db();
    let graph = DependencyGraph::build_from_connection(&conn).expect("Should build graph");

    let result = graph.detect_cycle();

    assert!(!result.has_cycle, "Empty graph should have no cycles");
    assert!(result.cycle_path.is_none(), "No cycle path expected");
    assert!(result.involved_ids.is_empty(), "No involved IDs expected");
}

#[test]
fn single_blocking_edge_creates_graph() {
    let conn = create_test_db();
    insert_task(&conn, "LAABCD", "api/tasks/t1.md");
    insert_task(&conn, "LBBCDE", "api/tasks/t2.md");
    add_blocking(&conn, "LAABCD", "LBBCDE");

    let graph = DependencyGraph::build_from_connection(&conn).expect("Should build graph");

    assert_eq!(graph.node_count(), 2, "Graph should have 2 nodes");
    assert_eq!(graph.get_blocking("LAABCD"), vec!["LBBCDE"], "LAABCD should block LBBCDE");
    assert_eq!(graph.get_blockers("LBBCDE"), vec!["LAABCD"], "LBBCDE should be blocked by LAABCD");
}

#[test]
fn blocked_by_edge_creates_inverse_graph() {
    let conn = create_test_db();
    insert_task(&conn, "LCCDEG", "api/tasks/t1.md");
    insert_task(&conn, "LDDEFH", "api/tasks/t2.md");
    add_blocked_by(&conn, "LCCDEG", "LDDEFH");

    let graph = DependencyGraph::build_from_connection(&conn).expect("Should build graph");

    assert_eq!(graph.get_blockers("LCCDEG"), vec!["LDDEFH"], "LCCDEG should be blocked by LDDEFH");
    assert_eq!(graph.get_blocking("LDDEFH"), vec!["LCCDEG"], "LDDEFH should block LCCDEG");
}

#[test]
fn simple_cycle_is_detected() {
    let conn = create_test_db();
    insert_task(&conn, "LEEFGI", "api/tasks/t1.md");
    insert_task(&conn, "LFFGHJ", "api/tasks/t2.md");
    add_blocking(&conn, "LEEFGI", "LFFGHJ");
    add_blocking(&conn, "LFFGHJ", "LEEFGI");

    let graph = DependencyGraph::build_from_connection(&conn).expect("Should build graph");
    let result = graph.detect_cycle();

    assert!(result.has_cycle, "Cycle should be detected");
    assert!(result.cycle_path.is_some(), "Cycle path should be present");
    assert!(!result.involved_ids.is_empty(), "Involved IDs should be present");
}

#[test]
fn three_node_cycle_is_detected() {
    let conn = create_test_db();
    insert_task(&conn, "LGGHIK", "api/tasks/t1.md");
    insert_task(&conn, "LHHIJL", "api/tasks/t2.md");
    insert_task(&conn, "LIIJKM", "api/tasks/t3.md");
    add_blocking(&conn, "LGGHIK", "LHHIJL");
    add_blocking(&conn, "LHHIJL", "LIIJKM");
    add_blocking(&conn, "LIIJKM", "LGGHIK");

    let graph = DependencyGraph::build_from_connection(&conn).expect("Should build graph");
    let result = graph.detect_cycle();

    assert!(result.has_cycle, "Three-node cycle should be detected");
    assert_eq!(result.involved_ids.len(), 4, "Cycle should have 4 IDs (including repeat of first)");
}

#[test]
fn chain_without_cycle_is_valid() {
    let conn = create_test_db();
    insert_task(&conn, "LJJKLN", "api/tasks/t1.md");
    insert_task(&conn, "LKKLMO", "api/tasks/t2.md");
    insert_task(&conn, "LLLMNP", "api/tasks/t3.md");
    add_blocking(&conn, "LJJKLN", "LKKLMO");
    add_blocking(&conn, "LKKLMO", "LLLMNP");

    let graph = DependencyGraph::build_from_connection(&conn).expect("Should build graph");
    let result = graph.detect_cycle();

    assert!(!result.has_cycle, "Linear chain should not have a cycle");
}

#[test]
fn get_all_blockers_returns_transitive_closure() {
    let conn = create_test_db();
    insert_task(&conn, "LMMNOR", "api/tasks/t1.md");
    insert_task(&conn, "LNNOPS", "api/tasks/t2.md");
    insert_task(&conn, "LOOPQT", "api/tasks/t3.md");
    add_blocking(&conn, "LMMNOR", "LNNOPS");
    add_blocking(&conn, "LNNOPS", "LOOPQT");

    let graph = DependencyGraph::build_from_connection(&conn).expect("Should build graph");

    let all_blockers = graph.get_all_blockers("LOOPQT");
    assert_eq!(all_blockers.len(), 2, "LOOPQT should have 2 transitive blockers");
    assert!(all_blockers.contains(&"LMMNOR".to_string()), "Should include LMMNOR");
    assert!(all_blockers.contains(&"LNNOPS".to_string()), "Should include LNNOPS");
}

#[test]
fn get_all_blocking_returns_transitive_closure() {
    let conn = create_test_db();
    insert_task(&conn, "LPPQRU", "api/tasks/t1.md");
    insert_task(&conn, "LQQRSV", "api/tasks/t2.md");
    insert_task(&conn, "LRRSTW", "api/tasks/t3.md");
    add_blocking(&conn, "LPPQRU", "LQQRSV");
    add_blocking(&conn, "LQQRSV", "LRRSTW");

    let graph = DependencyGraph::build_from_connection(&conn).expect("Should build graph");

    let all_blocking = graph.get_all_blocking("LPPQRU");
    assert_eq!(all_blocking.len(), 2, "LPPQRU should transitively block 2 tasks");
    assert!(all_blocking.contains(&"LQQRSV".to_string()), "Should include LQQRSV");
    assert!(all_blocking.contains(&"LRRSTW".to_string()), "Should include LRRSTW");
}

#[test]
fn topological_order_returns_valid_order() {
    let conn = create_test_db();
    insert_task(&conn, "LSSTUX", "api/tasks/t1.md");
    insert_task(&conn, "LTTUVY", "api/tasks/t2.md");
    insert_task(&conn, "LUUVWZ", "api/tasks/t3.md");
    add_blocking(&conn, "LSSTUX", "LTTUVY");
    add_blocking(&conn, "LTTUVY", "LUUVWZ");

    let graph = DependencyGraph::build_from_connection(&conn).expect("Should build graph");

    let order = graph.topological_order(&["LSSTUX".to_string()]);
    assert!(order.is_some(), "Topological order should exist for DAG");
    let order = order.unwrap();
    assert_eq!(order.len(), 3, "Order should have 3 nodes");

    let pos_s = order.iter().position(|id| id == "LSSTUX").unwrap();
    let pos_t = order.iter().position(|id| id == "LTTUVY").unwrap();
    let pos_u = order.iter().position(|id| id == "LUUVWZ").unwrap();
    assert!(pos_s < pos_t, "LSSTUX should come before LTTUVY");
    assert!(pos_t < pos_u, "LTTUVY should come before LUUVWZ");
}

#[test]
fn topological_order_returns_none_for_cycle() {
    let conn = create_test_db();
    insert_task(&conn, "LVVWXA", "api/tasks/t1.md");
    insert_task(&conn, "LWWXYB", "api/tasks/t2.md");
    add_blocking(&conn, "LVVWXA", "LWWXYB");
    add_blocking(&conn, "LWWXYB", "LVVWXA");

    let graph = DependencyGraph::build_from_connection(&conn).expect("Should build graph");

    let order = graph.topological_order(&["LVVWXA".to_string()]);
    assert!(order.is_none(), "Topological order should not exist for cyclic graph");
}

#[test]
fn validate_no_cycle_succeeds_for_valid_edge() {
    let conn = create_test_db();
    insert_task(&conn, "LXXYZC", "api/tasks/t1.md");
    insert_task(&conn, "LYYZAD", "api/tasks/t2.md");

    let result = validate_no_cycle_on_add(&conn, "LXXYZC", "LYYZAD", LinkType::Blocking);

    assert!(result.is_ok(), "Adding edge to acyclic graph should succeed");
}

#[test]
fn validate_no_cycle_fails_for_cycle_creating_edge() {
    let conn = create_test_db();
    insert_task(&conn, "LZZABE", "api/tasks/t1.md");
    insert_task(&conn, "LAABCF", "api/tasks/t2.md");
    add_blocking(&conn, "LZZABE", "LAABCF");

    let result = validate_no_cycle_on_add(&conn, "LAABCF", "LZZABE", LinkType::Blocking);

    assert!(result.is_err(), "Adding cycle-creating edge should fail");
}

#[test]
fn has_dependencies_returns_true_for_connected_node() {
    let conn = create_test_db();
    insert_task(&conn, "LBBCDG", "api/tasks/t1.md");
    insert_task(&conn, "LCCDEH", "api/tasks/t2.md");
    add_blocking(&conn, "LBBCDG", "LCCDEH");

    let graph = DependencyGraph::build_from_connection(&conn).expect("Should build graph");

    assert!(graph.has_dependencies("LBBCDG"), "Node with dependencies should return true");
    assert!(graph.has_dependencies("LCCDEH"), "Node with dependencies should return true");
}

#[test]
fn has_dependencies_returns_false_for_isolated_node() {
    let graph = DependencyGraph::new();

    assert!(!graph.has_dependencies("LXXYZZ"), "Isolated node should return false");
}

#[test]
fn build_dependency_tree_upstream_shows_blockers() {
    let conn = create_test_db();
    insert_task(&conn, "LDDEFI", "api/tasks/t1.md");
    insert_task(&conn, "LEEFGJ", "api/tasks/t2.md");
    insert_task(&conn, "LFFGHK", "api/tasks/t3.md");
    add_blocking(&conn, "LDDEFI", "LEEFGJ");
    add_blocking(&conn, "LEEFGJ", "LFFGHK");

    let graph = DependencyGraph::build_from_connection(&conn).expect("Should build graph");
    let tree =
        build_dependency_tree(&conn, &graph, "LFFGHK", TreeDirection::Upstream, None).unwrap();

    assert_eq!(tree.id, "LFFGHK", "Root should be the requested ID");
    assert_eq!(tree.children.len(), 1, "Should have one direct blocker");
    assert_eq!(tree.children[0].id, "LEEFGJ", "Direct blocker should be LEEFGJ");
    assert_eq!(tree.children[0].children.len(), 1, "LEEFGJ should have one blocker");
    assert_eq!(tree.children[0].children[0].id, "LDDEFI", "Transitive blocker should be LDDEFI");
}

#[test]
fn build_dependency_tree_downstream_shows_blocked() {
    let conn = create_test_db();
    insert_task(&conn, "LGGHIL", "api/tasks/t1.md");
    insert_task(&conn, "LHHIJM", "api/tasks/t2.md");
    insert_task(&conn, "LIIJKN", "api/tasks/t3.md");
    add_blocking(&conn, "LGGHIL", "LHHIJM");
    add_blocking(&conn, "LGGHIL", "LIIJKN");

    let graph = DependencyGraph::build_from_connection(&conn).expect("Should build graph");
    let tree =
        build_dependency_tree(&conn, &graph, "LGGHIL", TreeDirection::Downstream, None).unwrap();

    assert_eq!(tree.id, "LGGHIL", "Root should be the requested ID");
    assert_eq!(tree.children.len(), 2, "Should have two tasks blocked by this task");
}

#[test]
fn build_dependency_tree_respects_depth_limit() {
    let conn = create_test_db();
    insert_task(&conn, "LJJKLO", "api/tasks/t1.md");
    insert_task(&conn, "LKKLMP", "api/tasks/t2.md");
    insert_task(&conn, "LLLMNQ", "api/tasks/t3.md");
    add_blocking(&conn, "LJJKLO", "LKKLMP");
    add_blocking(&conn, "LKKLMP", "LLLMNQ");

    let graph = DependencyGraph::build_from_connection(&conn).expect("Should build graph");
    let tree =
        build_dependency_tree(&conn, &graph, "LLLMNQ", TreeDirection::Upstream, Some(1)).unwrap();

    assert_eq!(tree.id, "LLLMNQ", "Root should be the requested ID");
    assert_eq!(tree.children.len(), 1, "Should have one direct blocker");
    assert!(tree.children[0].children.is_empty(), "Depth limit should prevent deeper traversal");
}

#[test]
fn tree_node_includes_state_information() {
    let conn = create_test_db();
    insert_task(&conn, "LMMNOR", "api/tasks/open.md");

    let closed_doc = InsertDocument::new(
        "LNNOPR".to_string(),
        None,
        "api/tasks/.closed/closed.md".to_string(),
        "closed".to_string(),
        "Closed task".to_string(),
        Some(TaskType::Task),
        Some(2),
        None,
        None,
        None,
        "hash-closed".to_string(),
        100,
    );
    document_queries::insert(&conn, &closed_doc).expect("Failed to insert");

    add_blocking(&conn, "LNNOPR", "LMMNOR");

    let graph = DependencyGraph::build_from_connection(&conn).expect("Should build graph");
    let tree =
        build_dependency_tree(&conn, &graph, "LMMNOR", TreeDirection::Upstream, None).unwrap();

    assert_eq!(tree.state, "open", "Open task should have state 'open'");
    assert_eq!(tree.children[0].state, "closed", "Closed task should have state 'closed'");
}

#[test]
fn tree_node_shows_blocked_state_when_blocker_is_open() {
    let conn = create_test_db();
    // Create an open blocker task
    insert_task(&conn, "LOOPQS", "api/tasks/blocker.md");
    // Create a task blocked by the open task
    insert_task(&conn, "LPPQRT", "api/tasks/blocked.md");
    add_blocking(&conn, "LOOPQS", "LPPQRT");

    let graph = DependencyGraph::build_from_connection(&conn).expect("Should build graph");
    let tree =
        build_dependency_tree(&conn, &graph, "LPPQRT", TreeDirection::Upstream, None).unwrap();

    assert_eq!(tree.state, "blocked", "Task with open blocker should have state 'blocked'");
    assert_eq!(tree.children[0].state, "open", "Open blocker should have state 'open'");
}
