use std::fs;

use lattice::index::directory_roots::{DirectoryRoot, upsert};
use lattice::index::schema_definition;
use lattice::task::template_composer::{compose_templates, extract_template_sections};
use rusqlite::Connection;
use tempfile::TempDir;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
    schema_definition::create_schema(&conn).expect("Failed to create schema");
    conn
}

fn make_root(path: &str, id: &str, parent: Option<&str>, depth: u32) -> DirectoryRoot {
    DirectoryRoot {
        directory_path: path.to_string(),
        root_id: id.to_string(),
        parent_path: parent.map(|s| s.to_string()),
        depth,
    }
}

// ============================================================================
// extract_template_sections tests
// ============================================================================

#[test]
fn extract_sections_finds_context_section() {
    let body = r#"# API Module

Some intro text.

## [Lattice] Context

This is the context for API tasks.
It spans multiple lines.

## Other Section

This is not extracted.
"#;

    let sections = extract_template_sections(body);

    assert!(sections.context.is_some(), "Should extract context section");
    let context = sections.context.unwrap();
    assert!(context.contains("This is the context"), "Context should contain expected text");
    assert!(context.contains("multiple lines"), "Context should span multiple lines");
    assert!(!context.contains("Other Section"), "Context should not include next section");
}

#[test]
fn extract_sections_finds_acceptance_criteria_section() {
    let body = r#"# API Module

## [Lattice] Acceptance Criteria

- [ ] Write tests
- [ ] Update docs
- [ ] Review code

## Notes

Some notes here.
"#;

    let sections = extract_template_sections(body);

    assert!(sections.acceptance_criteria.is_some(), "Should extract acceptance criteria section");
    let acceptance = sections.acceptance_criteria.unwrap();
    assert!(acceptance.contains("Write tests"), "Should contain first criterion");
    assert!(acceptance.contains("Review code"), "Should contain last criterion");
    assert!(!acceptance.contains("Notes"), "Should not include next section");
}

#[test]
fn extract_sections_finds_both_sections() {
    let body = r#"# Project Root

Overview text.

## [Lattice] Context

Project-wide context information.

## [Lattice] Acceptance Criteria

- [ ] All tests pass
- [ ] No lint errors

## Other Content

More stuff.
"#;

    let sections = extract_template_sections(body);

    assert!(sections.context.is_some(), "Should find context");
    assert!(sections.acceptance_criteria.is_some(), "Should find acceptance criteria");
    assert!(
        sections.context.unwrap().contains("Project-wide context"),
        "Context should have expected content"
    );
    assert!(
        sections.acceptance_criteria.unwrap().contains("All tests pass"),
        "Acceptance should have expected content"
    );
}

#[test]
fn extract_sections_handles_missing_sections() {
    let body = r#"# Regular Document

Just some regular content.

## Section One

Text here.

## Section Two

More text.
"#;

    let sections = extract_template_sections(body);

    assert!(sections.context.is_none(), "Should return None when no context section");
    assert!(
        sections.acceptance_criteria.is_none(),
        "Should return None when no acceptance criteria section"
    );
}

#[test]
fn extract_sections_handles_different_heading_levels() {
    let body = r#"# Title

### [Lattice] Context

This uses h3.

#### [Lattice] Acceptance Criteria

This uses h4.
"#;

    let sections = extract_template_sections(body);

    assert!(sections.context.is_some(), "Should find h3 context section");
    assert!(sections.acceptance_criteria.is_some(), "Should find h4 acceptance criteria section");
}

#[test]
fn extract_sections_stops_at_same_level_heading() {
    let body = r#"# Root

## [Lattice] Context

Context content here.
More context.

## Another H2

This should not be in context.

## [Lattice] Acceptance Criteria

- [ ] Check something
"#;

    let sections = extract_template_sections(body);

    let context = sections.context.expect("Should have context");
    assert!(context.contains("Context content"), "Should have context content");
    assert!(context.contains("More context"), "Should have more content");
    assert!(!context.contains("Another H2"), "Should stop at same-level heading");
    assert!(!context.contains("not be in context"), "Should not include content after same-level");
}

#[test]
fn extract_sections_includes_lower_level_headings() {
    let body = r#"# Root

## [Lattice] Context

Introduction.

### Subsection

This is a subsection within context.

#### Deep Subsection

Even deeper.

## End Section

Done.
"#;

    let sections = extract_template_sections(body);

    let context = sections.context.expect("Should have context");
    assert!(context.contains("Introduction"), "Should have intro");
    assert!(context.contains("### Subsection"), "Should include h3 subsection");
    assert!(context.contains("within context"), "Should include subsection content");
    assert!(context.contains("#### Deep Subsection"), "Should include h4 subsection");
    assert!(!context.contains("End Section"), "Should stop at same-level heading");
}

#[test]
fn extract_sections_trims_whitespace() {
    let body = r#"## [Lattice] Context


Content with blank lines before.


And after.


## Next
"#;

    let sections = extract_template_sections(body);

    let context = sections.context.expect("Should have context");
    assert!(!context.starts_with('\n'), "Should trim leading newlines");
    assert!(!context.ends_with('\n'), "Should trim trailing newlines");
    assert!(context.starts_with("Content"), "Content should start with actual text");
}

#[test]
fn extract_sections_is_case_insensitive_for_section_names() {
    let body = r#"## [Lattice] CONTEXT

Uppercase context.

## [Lattice] acceptance criteria

Lowercase criteria.
"#;

    let sections = extract_template_sections(body);

    assert!(sections.context.is_some(), "Should match uppercase CONTEXT");
    assert!(sections.acceptance_criteria.is_some(), "Should match lowercase acceptance criteria");
}

// ============================================================================
// compose_templates tests
// ============================================================================

fn create_test_hierarchy(temp_dir: &TempDir) {
    let root = temp_dir.path();

    fs::create_dir_all(root.join("project")).expect("Create project");
    fs::create_dir_all(root.join("project/api")).expect("Create api");
    fs::create_dir_all(root.join("project/api/tasks")).expect("Create tasks");

    fs::write(
        root.join("project/project.md"),
        r#"---
lattice-id: LPRJZA
name: project
description: Project root
---

# Project

## [Lattice] Context

This is project-wide context.
All tasks inherit this.

## [Lattice] Acceptance Criteria

- [ ] All tests pass
- [ ] Code reviewed
"#,
    )
    .expect("Write project.md");

    fs::write(
        root.join("project/api/api.md"),
        r#"---
lattice-id: LAPIZA
name: api
description: API subsystem
---

# API

## [Lattice] Context

API-specific context here.
Handle REST conventions.

## [Lattice] Acceptance Criteria

- [ ] API docs updated
- [ ] Backward compatible
"#,
    )
    .expect("Write api.md");

    fs::write(
        root.join("project/api/tasks/fix_bug.md"),
        r#"---
lattice-id: LBUGZA
name: fix-bug
description: Fix validation bug
task-type: bug
priority: 1
---

# Fix Validation Bug

The validation logic is broken.
"#,
    )
    .expect("Write fix_bug.md");
}

#[test]
fn compose_templates_returns_empty_when_no_ancestors() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Create temp dir");

    fs::create_dir_all(temp_dir.path().join("orphan")).expect("Create orphan");
    fs::write(
        temp_dir.path().join("orphan/task.md"),
        "---\nlattice-id: LZZABC\nname: task\ndescription: Task\n---\n",
    )
    .expect("Write task");

    let result = compose_templates(&conn, "orphan/task.md".as_ref(), temp_dir.path())
        .expect("Should succeed");

    assert!(result.context.is_none(), "Should have no context");
    assert!(result.acceptance_criteria.is_none(), "Should have no acceptance criteria");
    assert!(result.contributor_ids.is_empty(), "Should have no contributors");
}

#[test]
fn compose_templates_extracts_from_single_ancestor() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Create temp dir");
    create_test_hierarchy(&temp_dir);

    upsert(&conn, &make_root("project/api", "LAPIZA", None, 0)).expect("Insert api root");

    let result = compose_templates(&conn, "project/api/tasks/fix_bug.md".as_ref(), temp_dir.path())
        .expect("Should succeed");

    assert!(result.context.is_some(), "Should have context");
    assert!(result.acceptance_criteria.is_some(), "Should have acceptance criteria");
    assert!(result.context.unwrap().contains("API-specific context"), "Context from api root");
    assert!(
        result.acceptance_criteria.unwrap().contains("API docs updated"),
        "Acceptance from api root"
    );
}

#[test]
fn compose_templates_composes_context_general_to_specific() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Create temp dir");
    create_test_hierarchy(&temp_dir);

    upsert(&conn, &make_root("project", "LPRJZA", None, 0)).expect("Insert project root");
    upsert(&conn, &make_root("project/api", "LAPIZA", Some("project"), 1))
        .expect("Insert api root");

    let result = compose_templates(&conn, "project/api/tasks/fix_bug.md".as_ref(), temp_dir.path())
        .expect("Should succeed");

    let context = result.context.expect("Should have composed context");

    let project_pos = context.find("project-wide context").expect("Should have project context");
    let api_pos = context.find("API-specific context").expect("Should have api context");

    assert!(
        project_pos < api_pos,
        "Project context (general) should come before API context (specific). Project at {}, API at {}",
        project_pos,
        api_pos
    );
}

#[test]
fn compose_templates_composes_acceptance_specific_to_general() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Create temp dir");
    create_test_hierarchy(&temp_dir);

    upsert(&conn, &make_root("project", "LPRJZA", None, 0)).expect("Insert project root");
    upsert(&conn, &make_root("project/api", "LAPIZA", Some("project"), 1))
        .expect("Insert api root");

    let result = compose_templates(&conn, "project/api/tasks/fix_bug.md".as_ref(), temp_dir.path())
        .expect("Should succeed");

    let acceptance = result.acceptance_criteria.expect("Should have acceptance criteria");

    let api_pos = acceptance.find("API docs updated").expect("Should have API acceptance");
    let project_pos = acceptance.find("All tests pass").expect("Should have project acceptance");

    assert!(
        api_pos < project_pos,
        "API acceptance (specific) should come before project acceptance (general). API at {}, Project at {}",
        api_pos,
        project_pos
    );
}

#[test]
fn compose_templates_handles_missing_sections() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Create temp dir");

    fs::create_dir_all(temp_dir.path().join("api/tasks")).expect("Create dirs");
    fs::write(
        temp_dir.path().join("api/api.md"),
        r#"---
lattice-id: LAPIZB
name: api
description: API without template sections
---

# API

Just a regular root document with no template sections.
"#,
    )
    .expect("Write api.md");

    fs::write(
        temp_dir.path().join("api/tasks/task.md"),
        "---\nlattice-id: LTSKZB\nname: task\ndescription: Task\ntask-type: task\npriority: 2\n---\n",
    )
    .expect("Write task.md");

    upsert(&conn, &make_root("api", "LAPIZB", None, 0)).expect("Insert api root");

    let result = compose_templates(&conn, "api/tasks/task.md".as_ref(), temp_dir.path())
        .expect("Should succeed");

    assert!(result.context.is_none(), "Should have no context");
    assert!(result.acceptance_criteria.is_none(), "Should have no acceptance criteria");
    assert!(result.contributor_ids.is_empty(), "No contributors when no sections");
}

#[test]
fn compose_templates_tracks_contributor_ids() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Create temp dir");
    create_test_hierarchy(&temp_dir);

    upsert(&conn, &make_root("project", "LPRJZA", None, 0)).expect("Insert project root");
    upsert(&conn, &make_root("project/api", "LAPIZA", Some("project"), 1))
        .expect("Insert api root");

    let result = compose_templates(&conn, "project/api/tasks/fix_bug.md".as_ref(), temp_dir.path())
        .expect("Should succeed");

    assert_eq!(result.contributor_ids.len(), 2, "Should have two contributors");
    assert!(result.contributor_ids.contains(&"LPRJZA".to_string()), "Should include project");
    assert!(result.contributor_ids.contains(&"LAPIZA".to_string()), "Should include api");
}

#[test]
fn compose_templates_skips_missing_ancestor_files() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Create temp dir");

    fs::create_dir_all(temp_dir.path().join("api/tasks")).expect("Create dirs");

    fs::write(
        temp_dir.path().join("api/tasks/task.md"),
        "---\nlattice-id: LTSKZC\nname: task\ndescription: Task\ntask-type: task\npriority: 2\n---\n",
    )
    .expect("Write task.md");

    upsert(&conn, &make_root("api", "LAPIZC", None, 0)).expect("Insert api root");

    let result = compose_templates(&conn, "api/tasks/task.md".as_ref(), temp_dir.path())
        .expect("Should succeed");

    assert!(result.context.is_none(), "Should handle missing file gracefully");
    assert!(result.contributor_ids.is_empty(), "No contributors when file missing");
}

#[test]
fn compose_templates_handles_partial_sections() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Create temp dir");

    fs::create_dir_all(temp_dir.path().join("project/api/tasks")).expect("Create dirs");

    fs::write(
        temp_dir.path().join("project/project.md"),
        r#"---
lattice-id: LPRJZB
name: project
description: Project with only context
---

## [Lattice] Context

Project context only, no acceptance.
"#,
    )
    .expect("Write project.md");

    fs::write(
        temp_dir.path().join("project/api/api.md"),
        r#"---
lattice-id: LAPIZD
name: api
description: API with only acceptance
---

## [Lattice] Acceptance Criteria

- [ ] API acceptance only
"#,
    )
    .expect("Write api.md");

    fs::write(
        temp_dir.path().join("project/api/tasks/task.md"),
        "---\nlattice-id: LTSKZD\nname: task\ndescription: Task\ntask-type: task\npriority: 2\n---\n",
    )
    .expect("Write task.md");

    upsert(&conn, &make_root("project", "LPRJZB", None, 0)).expect("Insert project");
    upsert(&conn, &make_root("project/api", "LAPIZD", Some("project"), 1)).expect("Insert api");

    let result = compose_templates(&conn, "project/api/tasks/task.md".as_ref(), temp_dir.path())
        .expect("Should succeed");

    assert!(result.context.is_some(), "Should have context from project");
    assert!(result.acceptance_criteria.is_some(), "Should have acceptance from api");
    assert_eq!(result.contributor_ids.len(), 2, "Both should contribute");
}

// ============================================================================
// Edge cases
// ============================================================================

#[test]
fn extract_sections_handles_empty_body() {
    let sections = extract_template_sections("");

    assert!(sections.context.is_none());
    assert!(sections.acceptance_criteria.is_none());
}

#[test]
fn extract_sections_handles_body_with_only_headings() {
    let body = "## [Lattice] Context\n## [Lattice] Acceptance Criteria\n";

    let sections = extract_template_sections(body);

    assert!(sections.context.is_some(), "Should find context section");
    assert!(sections.context.unwrap().is_empty(), "Context should be empty");
    assert!(sections.acceptance_criteria.is_some(), "Should find acceptance section");
    assert!(sections.acceptance_criteria.unwrap().is_empty(), "Acceptance should be empty");
}

#[test]
fn extract_sections_requires_lattice_prefix() {
    let body = r#"
## Context

This is not a Lattice section.

## Acceptance Criteria

Neither is this.
"#;

    let sections = extract_template_sections(body);

    assert!(sections.context.is_none(), "Should not match without [Lattice] prefix");
    assert!(sections.acceptance_criteria.is_none(), "Should not match without [Lattice] prefix");
}

#[test]
fn extract_sections_handles_h1_sections() {
    let body = r#"# [Lattice] Context

Top level context.
"#;

    let sections = extract_template_sections(body);

    assert!(sections.context.is_some(), "Should find h1 context");
    assert!(sections.context.unwrap().contains("Top level context"));
}

#[test]
fn extract_sections_handles_h6_sections() {
    let body = r#"###### [Lattice] Context

Deep nested context.
"#;

    let sections = extract_template_sections(body);

    assert!(sections.context.is_some(), "Should find h6 context");
    assert!(sections.context.unwrap().contains("Deep nested context"));
}

#[test]
fn compose_templates_works_with_00_prefixed_root() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Create temp dir");

    fs::create_dir_all(temp_dir.path().join("project/tasks")).expect("Create dirs");

    // Create a 00_-prefixed root document with template sections
    fs::write(
        temp_dir.path().join("project/tasks/00_task_template.md"),
        r#"---
lattice-id: LAATZA
name: 00-task-template
description: Task template for project
---

# Task Template

## [Lattice] Context

This is context from the 00_-prefixed root.
Important guidelines here.

## [Lattice] Acceptance Criteria

- [ ] Follow the template guidelines
- [ ] Complete all checklist items
"#,
    )
    .expect("Write 00_task_template.md");

    // Create a task in the same directory
    fs::write(
        temp_dir.path().join("project/tasks/fix_something.md"),
        r#"---
lattice-id: LTSKZE
name: fix-something
description: Fix something important
task-type: bug
priority: 2
---

# Fix Something

The thing is broken.
"#,
    )
    .expect("Write task.md");

    // Register the directory root with the 00_ root's ID
    upsert(&conn, &make_root("project/tasks", "LAATZA", None, 0)).expect("Insert 00_ root");

    // Insert the 00_ root document into the documents table so lookup_by_id works
    lattice::index::document_queries::insert(
        &conn,
        &lattice::index::document_types::InsertDocument {
            id: "LAATZA".to_string(),
            path: "project/tasks/00_task_template.md".to_string(),
            name: "00-task-template".to_string(),
            description: "Task template for project".to_string(),
            parent_id: None,
            task_type: None,
            is_closed: false,
            priority: None,
            created_at: None,
            updated_at: None,
            closed_at: None,
            body_hash: "test".to_string(),
            content_length: 100,
            is_root: true,
            skill: false,
        },
    )
    .expect("Insert document");

    let result =
        compose_templates(&conn, "project/tasks/fix_something.md".as_ref(), temp_dir.path())
            .expect("Should succeed");

    assert!(
        result.context.is_some(),
        "Should extract context from 00_-prefixed root, got: {result:?}"
    );
    assert!(
        result.acceptance_criteria.is_some(),
        "Should extract acceptance criteria from 00_-prefixed root"
    );

    let context = result.context.unwrap();
    assert!(
        context.contains("00_-prefixed root"),
        "Context should contain the template content, got: {context}"
    );

    let acceptance = result.acceptance_criteria.unwrap();
    assert!(
        acceptance.contains("Follow the template guidelines"),
        "Acceptance should contain template criteria, got: {acceptance}"
    );

    assert!(
        result.contributor_ids.contains(&"LAATZA".to_string()),
        "Should track 00_ root as contributor"
    );
}
