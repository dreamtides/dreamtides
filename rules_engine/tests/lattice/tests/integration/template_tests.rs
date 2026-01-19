//! Integration tests for template composition.
//!
//! Tests template composition behavior through multi-command workflows,
//! verifying that context and acceptance criteria are properly composed
//! from ancestor root documents.

use lattice::cli::command_dispatch::{CommandContext, create_context};
use lattice::cli::commands::create_command;
use lattice::cli::commands::show_command::show_executor;
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::task_args::CreateArgs;
use lattice::cli::workflow_args::ShowArgs;
use lattice::document::frontmatter_schema::TaskType;
use lattice::git::client_config::FakeClientIdStore;
use lattice::index::directory_roots::{self, DirectoryRoot};
use lattice::index::document_queries;
use lattice::task::template_composer;
use lattice::test::test_environment::TestEnv;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_context_from_env(env: &TestEnv, global: &GlobalOptions) -> CommandContext {
    let mut context = create_context(env.repo_root(), global).expect("Create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    context
}

fn create_task(env: &TestEnv, parent: &str, description: &str) -> String {
    let args = CreateArgs {
        parent: parent.to_string(),
        description: description.to_string(),
        r#type: Some(TaskType::Task),
        priority: Some(2),
        body_file: None,
        labels: Vec::new(),
        deps: None,
    };

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(env, &global);

    create_command::execute(ctx, args).expect("Create task");

    let docs = document_queries::all_ids(env.conn()).expect("Query IDs");
    docs.into_iter().last().expect("Should have created a document")
}

fn make_root(path: &str, id: &str, parent: Option<&str>, depth: u32) -> DirectoryRoot {
    DirectoryRoot {
        directory_path: path.to_string(),
        root_id: id.to_string(),
        parent_path: parent.map(|s| s.to_string()),
        depth,
    }
}

fn write_root_doc_with_context(env: &TestEnv, dir: &str, id: &str, context_content: &str) {
    let name = dir.split('/').last().unwrap_or(dir);
    let content = format!(
        r#"---
lattice-id: {id}
name: {name}
description: {name} root document
---

# {name}

## [Lattice] Context

{context_content}
"#
    );
    env.write_file(&format!("{dir}/{name}.md"), &content);
    env.fake_git().track_file(format!("{dir}/{name}.md"));
}

fn write_root_doc_with_acceptance(env: &TestEnv, dir: &str, id: &str, acceptance_content: &str) {
    let name = dir.split('/').last().unwrap_or(dir);
    let content = format!(
        r#"---
lattice-id: {id}
name: {name}
description: {name} root document
---

# {name}

## [Lattice] Acceptance Criteria

{acceptance_content}
"#
    );
    env.write_file(&format!("{dir}/{name}.md"), &content);
    env.fake_git().track_file(format!("{dir}/{name}.md"));
}

fn write_root_doc_with_both(
    env: &TestEnv,
    dir: &str,
    id: &str,
    context_content: &str,
    acceptance_content: &str,
) {
    let name = dir.split('/').last().unwrap_or(dir);
    let content = format!(
        r#"---
lattice-id: {id}
name: {name}
description: {name} root document
---

# {name}

## [Lattice] Context

{context_content}

## [Lattice] Acceptance Criteria

{acceptance_content}
"#
    );
    env.write_file(&format!("{dir}/{name}.md"), &content);
    env.fake_git().track_file(format!("{dir}/{name}.md"));
}

// ============================================================================
// Context Composition Tests (general -> specific order)
// ============================================================================

#[test]
fn context_composes_from_single_ancestor() {
    let env = TestEnv::new();
    env.create_dir("project");
    env.create_dir("project/tasks");

    // Create root document with context
    write_root_doc_with_context(&env, "project", "LPROJA", "Project-wide context information.");

    // Register directory root
    directory_roots::upsert(env.conn(), &make_root("project", "LPROJA", None, 0))
        .expect("Insert project root");

    // Create a task under project
    let task_id = create_task(&env, "project/", "Test task");

    // Get task path
    let task = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");

    // Compose templates
    let composed = template_composer::compose_templates(
        env.conn(),
        std::path::Path::new(&task.path),
        env.repo_root(),
    )
    .expect("Compose templates");

    assert!(composed.context.is_some(), "Should have context");
    assert!(
        composed.context.as_ref().unwrap().contains("Project-wide context"),
        "Context should contain project context"
    );
}

#[test]
fn context_composes_in_general_to_specific_order() {
    let env = TestEnv::new();
    env.create_dir("project");
    env.create_dir("project/api");
    env.create_dir("project/api/tasks");

    // Create root documents with context
    write_root_doc_with_context(&env, "project", "LPRJAA", "PROJECT CONTEXT");
    write_root_doc_with_context(&env, "project/api", "LAPIAB", "API CONTEXT");

    // Register directory roots
    directory_roots::upsert(env.conn(), &make_root("project", "LPRJAA", None, 0))
        .expect("Insert project root");
    directory_roots::upsert(env.conn(), &make_root("project/api", "LAPIAB", Some("project"), 1))
        .expect("Insert api root");

    // Create a task under api
    let task_id = create_task(&env, "project/api/", "API task");

    // Get task path
    let task = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");

    // Compose templates
    let composed = template_composer::compose_templates(
        env.conn(),
        std::path::Path::new(&task.path),
        env.repo_root(),
    )
    .expect("Compose templates");

    assert!(composed.context.is_some(), "Should have context");
    let context = composed.context.as_ref().unwrap();

    // PROJECT should come before API (general -> specific)
    let project_pos = context.find("PROJECT CONTEXT").expect("Should contain project context");
    let api_pos = context.find("API CONTEXT").expect("Should contain API context");
    assert!(
        project_pos < api_pos,
        "Project context should come before API context (general -> specific)"
    );
}

#[test]
fn context_composes_through_deep_hierarchy() {
    let env = TestEnv::new();
    env.create_dir("root");
    env.create_dir("root/level1");
    env.create_dir("root/level1/level2");
    env.create_dir("root/level1/level2/tasks");

    // Create root documents at each level
    write_root_doc_with_context(&env, "root", "LRTABC", "ROOT CONTEXT");
    write_root_doc_with_context(&env, "root/level1", "LLVABA", "LEVEL1 CONTEXT");
    write_root_doc_with_context(&env, "root/level1/level2", "LLVBCD", "LEVEL2 CONTEXT");

    // Register directory roots
    directory_roots::upsert(env.conn(), &make_root("root", "LRTABC", None, 0))
        .expect("Insert root");
    directory_roots::upsert(env.conn(), &make_root("root/level1", "LLVABA", Some("root"), 1))
        .expect("Insert level1");
    directory_roots::upsert(
        env.conn(),
        &make_root("root/level1/level2", "LLVBCD", Some("root/level1"), 2),
    )
    .expect("Insert level2");

    // Create a task at the deepest level
    let task_id = create_task(&env, "root/level1/level2/", "Deep task");

    // Get task path
    let task = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");

    // Compose templates
    let composed = template_composer::compose_templates(
        env.conn(),
        std::path::Path::new(&task.path),
        env.repo_root(),
    )
    .expect("Compose templates");

    assert!(composed.context.is_some(), "Should have context");
    let context = composed.context.as_ref().unwrap();

    // Verify order: ROOT < LEVEL1 < LEVEL2
    let root_pos = context.find("ROOT CONTEXT").expect("Should contain root");
    let l1_pos = context.find("LEVEL1 CONTEXT").expect("Should contain level1");
    let l2_pos = context.find("LEVEL2 CONTEXT").expect("Should contain level2");

    assert!(root_pos < l1_pos, "Root should come before level1");
    assert!(l1_pos < l2_pos, "Level1 should come before level2");
}

// ============================================================================
// Acceptance Criteria Composition Tests (specific -> general order)
// ============================================================================

#[test]
fn acceptance_criteria_composes_from_single_ancestor() {
    let env = TestEnv::new();
    env.create_dir("project");
    env.create_dir("project/tasks");

    // Create root document with acceptance criteria
    write_root_doc_with_acceptance(
        &env,
        "project",
        "LPROJB",
        "- [ ] All tests pass\n- [ ] Code reviewed",
    );

    // Register directory root
    directory_roots::upsert(env.conn(), &make_root("project", "LPROJB", None, 0))
        .expect("Insert project root");

    // Create a task under project
    let task_id = create_task(&env, "project/", "Test task");

    // Get task path
    let task = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");

    // Compose templates
    let composed = template_composer::compose_templates(
        env.conn(),
        std::path::Path::new(&task.path),
        env.repo_root(),
    )
    .expect("Compose templates");

    assert!(composed.acceptance_criteria.is_some(), "Should have acceptance criteria");
    assert!(
        composed.acceptance_criteria.as_ref().unwrap().contains("All tests pass"),
        "Acceptance should contain criteria"
    );
}

#[test]
fn acceptance_criteria_composes_in_specific_to_general_order() {
    let env = TestEnv::new();
    env.create_dir("project");
    env.create_dir("project/api");
    env.create_dir("project/api/tasks");

    // Create root documents with acceptance criteria
    write_root_doc_with_acceptance(&env, "project", "LPRJAB", "PROJECT ACCEPTANCE");
    write_root_doc_with_acceptance(&env, "project/api", "LAPICD", "API ACCEPTANCE");

    // Register directory roots
    directory_roots::upsert(env.conn(), &make_root("project", "LPRJAB", None, 0))
        .expect("Insert project root");
    directory_roots::upsert(env.conn(), &make_root("project/api", "LAPICD", Some("project"), 1))
        .expect("Insert api root");

    // Create a task under api
    let task_id = create_task(&env, "project/api/", "API task");

    // Get task path
    let task = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");

    // Compose templates
    let composed = template_composer::compose_templates(
        env.conn(),
        std::path::Path::new(&task.path),
        env.repo_root(),
    )
    .expect("Compose templates");

    assert!(composed.acceptance_criteria.is_some(), "Should have acceptance criteria");
    let acceptance = composed.acceptance_criteria.as_ref().unwrap();

    // API should come before PROJECT (specific -> general)
    let api_pos = acceptance.find("API ACCEPTANCE").expect("Should contain API acceptance");
    let project_pos =
        acceptance.find("PROJECT ACCEPTANCE").expect("Should contain project acceptance");
    assert!(
        api_pos < project_pos,
        "API acceptance should come before project acceptance (specific -> general)"
    );
}

// ============================================================================
// Mixed Context and Acceptance Tests
// ============================================================================

#[test]
fn both_context_and_acceptance_compose_correctly() {
    let env = TestEnv::new();
    env.create_dir("project");
    env.create_dir("project/api");
    env.create_dir("project/api/tasks");

    // Create root documents with both sections
    write_root_doc_with_both(&env, "project", "LPRJCD", "PROJECT CONTEXT", "PROJECT ACCEPTANCE");
    write_root_doc_with_both(&env, "project/api", "LAPICD", "API CONTEXT", "API ACCEPTANCE");

    // Register directory roots
    directory_roots::upsert(env.conn(), &make_root("project", "LPRJCD", None, 0))
        .expect("Insert project root");
    directory_roots::upsert(env.conn(), &make_root("project/api", "LAPICD", Some("project"), 1))
        .expect("Insert api root");

    // Create a task
    let task_id = create_task(&env, "project/api/", "Task with both");

    // Get task path
    let task = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");

    // Compose templates
    let composed = template_composer::compose_templates(
        env.conn(),
        std::path::Path::new(&task.path),
        env.repo_root(),
    )
    .expect("Compose templates");

    // Both should be present
    assert!(composed.context.is_some(), "Should have context");
    assert!(composed.acceptance_criteria.is_some(), "Should have acceptance");

    let context = composed.context.as_ref().unwrap();
    let acceptance = composed.acceptance_criteria.as_ref().unwrap();

    // Context: PROJECT before API (general -> specific)
    let ctx_project = context.find("PROJECT CONTEXT").expect("Context should have project");
    let ctx_api = context.find("API CONTEXT").expect("Context should have API");
    assert!(ctx_project < ctx_api, "Context: project before API");

    // Acceptance: API before PROJECT (specific -> general)
    let acc_api = acceptance.find("API ACCEPTANCE").expect("Acceptance should have API");
    let acc_project =
        acceptance.find("PROJECT ACCEPTANCE").expect("Acceptance should have project");
    assert!(acc_api < acc_project, "Acceptance: API before project");
}

// ============================================================================
// Missing Section Tests
// ============================================================================

#[test]
fn missing_context_returns_none() {
    let env = TestEnv::new();
    env.create_dir("project");
    env.create_dir("project/tasks");

    // Create root document with only acceptance (no context)
    write_root_doc_with_acceptance(&env, "project", "LPRJEF", "- [ ] Some criteria");

    // Register directory root
    directory_roots::upsert(env.conn(), &make_root("project", "LPRJEF", None, 0))
        .expect("Insert project root");

    // Create a task
    let task_id = create_task(&env, "project/", "Task without context");

    let task = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");

    let composed = template_composer::compose_templates(
        env.conn(),
        std::path::Path::new(&task.path),
        env.repo_root(),
    )
    .expect("Compose templates");

    assert!(composed.context.is_none(), "Context should be None when not present");
    assert!(composed.acceptance_criteria.is_some(), "Acceptance should still be present");
}

#[test]
fn missing_acceptance_returns_none() {
    let env = TestEnv::new();
    env.create_dir("project");
    env.create_dir("project/tasks");

    // Create root document with only context (no acceptance)
    write_root_doc_with_context(&env, "project", "LPRJGH", "Some context here.");

    // Register directory root
    directory_roots::upsert(env.conn(), &make_root("project", "LPRJGH", None, 0))
        .expect("Insert project root");

    // Create a task
    let task_id = create_task(&env, "project/", "Task without acceptance");

    let task = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");

    let composed = template_composer::compose_templates(
        env.conn(),
        std::path::Path::new(&task.path),
        env.repo_root(),
    )
    .expect("Compose templates");

    assert!(composed.context.is_some(), "Context should still be present");
    assert!(composed.acceptance_criteria.is_none(), "Acceptance should be None when not present");
}

#[test]
fn no_ancestors_returns_empty_template() {
    let env = TestEnv::new();
    env.create_dir("orphan/tasks");

    // Create a task with no registered root documents
    let task_id = create_task(&env, "orphan/", "Orphan task");

    let task = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");

    let composed = template_composer::compose_templates(
        env.conn(),
        std::path::Path::new(&task.path),
        env.repo_root(),
    )
    .expect("Compose templates");

    assert!(composed.context.is_none(), "Context should be None");
    assert!(composed.acceptance_criteria.is_none(), "Acceptance should be None");
    assert!(composed.contributor_ids.is_empty(), "Contributors should be empty");
}

// ============================================================================
// Partial Hierarchy Tests
// ============================================================================

#[test]
fn partial_context_in_hierarchy() {
    let env = TestEnv::new();
    env.create_dir("project");
    env.create_dir("project/api");
    env.create_dir("project/api/tasks");

    // Project has context, API does not
    write_root_doc_with_context(&env, "project", "LPRJHK", "PROJECT CONTEXT");

    // Create API root without any Lattice sections
    let api_name = "api";
    let api_content = format!(
        r#"---
lattice-id: LAPIKM
name: {api_name}
description: API module
---

# API

Regular content without Lattice sections.
"#
    );
    env.write_file("project/api/api.md", &api_content);
    env.fake_git().track_file("project/api/api.md");

    // Register directory roots
    directory_roots::upsert(env.conn(), &make_root("project", "LPRJHK", None, 0))
        .expect("Insert project root");
    directory_roots::upsert(env.conn(), &make_root("project/api", "LAPIKM", Some("project"), 1))
        .expect("Insert api root");

    // Create a task
    let task_id = create_task(&env, "project/api/", "Task with partial context");

    let task = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");

    let composed = template_composer::compose_templates(
        env.conn(),
        std::path::Path::new(&task.path),
        env.repo_root(),
    )
    .expect("Compose templates");

    // Should have context from project only
    assert!(composed.context.is_some(), "Should have context from project");
    assert!(
        composed.context.as_ref().unwrap().contains("PROJECT CONTEXT"),
        "Context should come from project"
    );
}

#[test]
fn middle_level_only_has_sections() {
    let env = TestEnv::new();
    env.create_dir("root");
    env.create_dir("root/middle");
    env.create_dir("root/middle/tasks");

    // Root has no Lattice sections
    let root_content = r#"---
lattice-id: LRTDEF
name: root
description: Root module
---

# Root

Just regular content.
"#;
    env.write_file("root/root.md", root_content);
    env.fake_git().track_file("root/root.md");

    // Middle has both sections
    write_root_doc_with_both(&env, "root/middle", "LMIDAB", "MIDDLE CONTEXT", "MIDDLE ACCEPTANCE");

    // Register directory roots
    directory_roots::upsert(env.conn(), &make_root("root", "LRTDEF", None, 0))
        .expect("Insert root");
    directory_roots::upsert(env.conn(), &make_root("root/middle", "LMIDAB", Some("root"), 1))
        .expect("Insert middle");

    // Create a task
    let task_id = create_task(&env, "root/middle/", "Task");

    let task = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");

    let composed = template_composer::compose_templates(
        env.conn(),
        std::path::Path::new(&task.path),
        env.repo_root(),
    )
    .expect("Compose templates");

    // Should only have content from middle
    assert!(composed.context.is_some(), "Should have context");
    assert!(composed.acceptance_criteria.is_some(), "Should have acceptance");
    assert_eq!(composed.contributor_ids, vec!["LMIDAB"], "Only middle should contribute");
}

// ============================================================================
// Show Command Integration Tests
// ============================================================================

#[test]
fn show_command_displays_task_with_composed_templates() {
    let env = TestEnv::new();
    env.create_dir("project");
    env.create_dir("project/tasks");

    // Create root document with templates
    write_root_doc_with_both(
        &env,
        "project",
        "LPROJ7",
        "Template context for tasks.",
        "- [ ] Template acceptance criteria",
    );

    // Register root
    directory_roots::upsert(env.conn(), &make_root("project", "LPROJ7", None, 0))
        .expect("Insert project root");

    // Create a task
    let task_id = create_task(&env, "project/", "Task for show");

    let (_temp, context) = env.into_parts();

    // Show the task
    let args = ShowArgs { ids: vec![task_id], short: false, refs: false, peek: false, raw: false };

    let result = show_executor::execute(context, args);
    assert!(result.is_ok(), "Show should succeed with templates: {:?}", result);
}

#[test]
fn show_command_raw_mode_skips_template_composition() {
    let env = TestEnv::new();
    env.create_dir("project");
    env.create_dir("project/tasks");

    // Create root document with templates
    write_root_doc_with_both(
        &env,
        "project",
        "LPRJKM",
        "Context to skip.",
        "- [ ] Acceptance to skip",
    );

    // Register root
    directory_roots::upsert(env.conn(), &make_root("project", "LPRJKM", None, 0))
        .expect("Insert project root");

    // Create a task
    let task_id = create_task(&env, "project/", "Task for raw show");

    let (_temp, context) = env.into_parts();

    // Show in raw mode
    let args = ShowArgs { ids: vec![task_id], short: false, refs: false, peek: false, raw: true };

    let result = show_executor::execute(context, args);
    assert!(result.is_ok(), "Raw show should succeed: {:?}", result);
}
