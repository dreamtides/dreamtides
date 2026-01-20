//! Integration tests for skill document workflow.
//!
//! These tests verify the end-to-end flow of skill document management,
//! including document creation, index reconciliation, and symlink
//! synchronization.

use std::fs;

use lattice::index::document_queries;
use lattice::skill::symlink_manager::sync_symlinks;
use lattice::test::test_environment::TestEnv;

// =============================================================================
// Helper functions
// =============================================================================

/// Creates a skill document with the given content directly on the filesystem.
fn create_skill_file(env: &TestEnv, relative_path: &str, id: &str, name: &str, description: &str) {
    let content = format!(
        "---\n\
         lattice-id: {id}\n\
         name: {name}\n\
         description: {description}\n\
         skill: true\n\
         ---\n\n\
         # {name}\n\n\
         Skill documentation body."
    );
    env.write_file(relative_path, &content);
    env.fake_git().track_file(relative_path);
}

/// Creates a non-skill document (task or KB doc).
fn create_non_skill_file(
    env: &TestEnv,
    relative_path: &str,
    id: &str,
    name: &str,
    description: &str,
) {
    let content = format!(
        "---\n\
         lattice-id: {id}\n\
         name: {name}\n\
         description: {description}\n\
         ---\n\n\
         Document body."
    );
    env.write_file(relative_path, &content);
    env.fake_git().track_file(relative_path);
}

/// Inserts a skill document directly into the index.
fn index_skill_document(
    env: &TestEnv,
    id: &str,
    path: &str,
    name: &str,
    description: &str,
    skill: bool,
) {
    use lattice::index::document_types::InsertDocument;

    let doc = InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        description.to_string(),
        None, // task_type
        None, // priority
        None, // created_at
        None, // updated_at
        None, // closed_at
        "hash".to_string(),
        100,
        skill,
    );
    document_queries::insert(env.conn(), &doc).expect("Failed to insert document");
}

// =============================================================================
// End-to-end workflow tests
// =============================================================================

#[test]
fn integration_create_skill_document_then_sync_creates_symlink() {
    let env = TestEnv::new();
    env.create_dir("docs");

    // Create a skill document file
    create_skill_file(&env, "docs/my-skill.md", "LSKL01", "my-skill", "A helpful skill");

    // Index the document (simulating reconciliation)
    index_skill_document(&env, "LSKL01", "docs/my-skill.md", "my-skill", "A helpful skill", true);

    // Run symlink sync (simulating startup)
    let result = sync_symlinks(env.conn(), env.repo_root()).expect("sync should succeed");

    assert_eq!(result.created, 1, "Should create one symlink");

    // Verify symlink exists and points to correct target
    let symlink_path = env.repo_root().join(".claude/skills/my-skill.md");
    assert!(symlink_path.is_symlink(), "Symlink should exist at .claude/skills/my-skill.md");

    let target = fs::read_link(&symlink_path).expect("read symlink");
    assert!(
        target.to_string_lossy().contains("docs/my-skill.md"),
        "Symlink should point to docs/my-skill.md"
    );
}

#[test]
fn integration_remove_skill_flag_then_sync_removes_symlink() {
    let env = TestEnv::new();
    env.create_dir("docs");

    // Create skill document and sync
    create_skill_file(&env, "docs/my-skill.md", "LSKL02", "my-skill", "A skill document");
    index_skill_document(&env, "LSKL02", "docs/my-skill.md", "my-skill", "A skill document", true);
    let result1 = sync_symlinks(env.conn(), env.repo_root()).expect("first sync");
    assert_eq!(result1.created, 1);

    let symlink_path = env.repo_root().join(".claude/skills/my-skill.md");
    assert!(symlink_path.is_symlink(), "Symlink should exist after first sync");

    // Simulate removing skill flag: delete from index and reinsert without skill
    document_queries::delete_by_id(env.conn(), "LSKL02").expect("delete");
    index_skill_document(
        &env,
        "LSKL02",
        "docs/my-skill.md",
        "my-skill",
        "A skill document",
        false, // skill = false now
    );

    // Run sync again
    let result2 = sync_symlinks(env.conn(), env.repo_root()).expect("second sync");
    assert_eq!(result2.removed, 1, "Should remove symlink when skill flag is removed");
    assert!(!symlink_path.exists(), "Symlink should be removed");
}

#[test]
fn integration_move_document_then_sync_updates_symlink() {
    let env = TestEnv::new();
    env.create_dir("old-location");
    env.create_dir("new-location");

    // Create skill document at old location
    create_skill_file(&env, "old-location/my-skill.md", "LSKL03", "my-skill", "A movable skill");
    index_skill_document(
        &env,
        "LSKL03",
        "old-location/my-skill.md",
        "my-skill",
        "A movable skill",
        true,
    );

    let result1 = sync_symlinks(env.conn(), env.repo_root()).expect("first sync");
    assert_eq!(result1.created, 1);

    let symlink_path = env.repo_root().join(".claude/skills/my-skill.md");
    let old_target = fs::read_link(&symlink_path).expect("read symlink");
    assert!(
        old_target.to_string_lossy().contains("old-location"),
        "Should initially point to old location"
    );

    // Move the file and update index
    fs::rename(
        env.repo_root().join("old-location/my-skill.md"),
        env.repo_root().join("new-location/my-skill.md"),
    )
    .expect("move file");

    document_queries::delete_by_id(env.conn(), "LSKL03").expect("delete");
    index_skill_document(
        &env,
        "LSKL03",
        "new-location/my-skill.md",
        "my-skill",
        "A movable skill",
        true,
    );

    // Sync should update symlink
    let result2 = sync_symlinks(env.conn(), env.repo_root()).expect("second sync");
    assert_eq!(result2.updated, 1, "Should update symlink to new path");

    let new_target = fs::read_link(&symlink_path).expect("read updated symlink");
    assert!(
        new_target.to_string_lossy().contains("new-location"),
        "Symlink should point to new location"
    );
}

#[test]
fn integration_delete_skill_document_then_sync_removes_symlink() {
    let env = TestEnv::new();
    env.create_dir("docs");

    // Create and sync skill document
    create_skill_file(&env, "docs/my-skill.md", "LSKL04", "my-skill", "Will be deleted");
    index_skill_document(&env, "LSKL04", "docs/my-skill.md", "my-skill", "Will be deleted", true);

    let result1 = sync_symlinks(env.conn(), env.repo_root()).expect("first sync");
    assert_eq!(result1.created, 1);

    // Delete the document file and remove from index
    fs::remove_file(env.repo_root().join("docs/my-skill.md")).expect("delete file");
    document_queries::delete_by_id(env.conn(), "LSKL04").expect("delete from index");

    // Sync should clean up stale symlink
    let result2 = sync_symlinks(env.conn(), env.repo_root()).expect("second sync");
    assert_eq!(result2.removed, 1, "Should remove symlink for deleted document");

    let symlink_path = env.repo_root().join(".claude/skills/my-skill.md");
    assert!(!symlink_path.exists(), "Symlink should be removed after document deletion");
}

#[test]
fn integration_multiple_skills_partial_update() {
    let env = TestEnv::new();
    env.create_dir("docs");

    // Create three skill documents
    for i in 1..=3 {
        let name = format!("skill-{i}");
        let id = format!("LSKL{i:02}");
        let path = format!("docs/{name}.md");
        create_skill_file(&env, &path, &id, &name, &format!("Skill number {i}"));
        index_skill_document(&env, &id, &path, &name, &format!("Skill number {i}"), true);
    }

    let result1 = sync_symlinks(env.conn(), env.repo_root()).expect("first sync");
    assert_eq!(result1.created, 3, "Should create three symlinks");

    // Remove skill flag from skill-2 only
    document_queries::delete_by_id(env.conn(), "LSKL02").expect("delete");
    index_skill_document(&env, "LSKL02", "docs/skill-2.md", "skill-2", "Skill number 2", false);

    let result2 = sync_symlinks(env.conn(), env.repo_root()).expect("second sync");
    assert_eq!(result2.removed, 1, "Should remove one symlink");
    assert_eq!(result2.created, 0, "Should not create new symlinks");
    assert_eq!(result2.updated, 0, "Should not update existing symlinks");

    // skill-1 and skill-3 symlinks should still exist
    assert!(env.repo_root().join(".claude/skills/skill-1.md").is_symlink());
    assert!(!env.repo_root().join(".claude/skills/skill-2.md").exists());
    assert!(env.repo_root().join(".claude/skills/skill-3.md").is_symlink());
}

#[test]
fn integration_skill_with_validation_issues_still_syncs() {
    // Skills with validation warnings (like name too long) should still sync
    let env = TestEnv::new();
    env.create_dir("docs");

    let long_name = "x".repeat(100); // Exceeds 64 char limit (warning)
    let id = "LSKL05";
    let path = format!("docs/{long_name}.md");

    create_skill_file(&env, &path, id, &long_name, "Skill with long name");
    index_skill_document(&env, id, &path, &long_name, "Skill with long name", true);

    let result = sync_symlinks(env.conn(), env.repo_root()).expect("sync should succeed");
    assert_eq!(result.created, 1, "Should create symlink even with validation warnings");

    let symlink_path = env.repo_root().join(format!(".claude/skills/{long_name}.md"));
    assert!(symlink_path.is_symlink(), "Symlink should exist");
}

#[test]
fn integration_non_skill_document_no_symlink() {
    let env = TestEnv::new();
    env.create_dir("docs");

    // Create a non-skill document
    create_non_skill_file(&env, "docs/regular-doc.md", "LDOC01", "regular-doc", "Not a skill");
    index_skill_document(
        &env,
        "LDOC01",
        "docs/regular-doc.md",
        "regular-doc",
        "Not a skill",
        false,
    );

    let result = sync_symlinks(env.conn(), env.repo_root()).expect("sync should succeed");
    assert_eq!(result.created, 0, "Should not create symlink for non-skill document");

    let symlink_path = env.repo_root().join(".claude/skills/regular-doc.md");
    assert!(!symlink_path.exists(), "No symlink should exist for non-skill document");
}

#[test]
fn integration_convert_document_to_skill() {
    let env = TestEnv::new();
    env.create_dir("docs");

    // Start with a non-skill document
    create_non_skill_file(
        &env,
        "docs/future-skill.md",
        "LDOC02",
        "future-skill",
        "Soon to be skill",
    );
    index_skill_document(
        &env,
        "LDOC02",
        "docs/future-skill.md",
        "future-skill",
        "Soon to be skill",
        false,
    );

    let result1 = sync_symlinks(env.conn(), env.repo_root()).expect("first sync");
    assert_eq!(result1.created, 0, "No symlink for non-skill");

    // Convert to skill
    document_queries::delete_by_id(env.conn(), "LDOC02").expect("delete");
    index_skill_document(
        &env,
        "LDOC02",
        "docs/future-skill.md",
        "future-skill",
        "Now a skill",
        true,
    );

    let result2 = sync_symlinks(env.conn(), env.repo_root()).expect("second sync");
    assert_eq!(result2.created, 1, "Should create symlink when converted to skill");

    let symlink_path = env.repo_root().join(".claude/skills/future-skill.md");
    assert!(symlink_path.is_symlink(), "Symlink should exist after conversion");
}

#[test]
fn integration_skill_in_nested_directory() {
    let env = TestEnv::new();
    env.create_dir("api/auth/docs");

    create_skill_file(
        &env,
        "api/auth/docs/auth-helpers.md",
        "LAUTH1",
        "auth-helpers",
        "Authentication helper skill",
    );
    index_skill_document(
        &env,
        "LAUTH1",
        "api/auth/docs/auth-helpers.md",
        "auth-helpers",
        "Authentication helper skill",
        true,
    );

    let result = sync_symlinks(env.conn(), env.repo_root()).expect("sync should succeed");
    assert_eq!(result.created, 1);

    let symlink_path = env.repo_root().join(".claude/skills/auth-helpers.md");
    assert!(symlink_path.is_symlink());

    // Symlink should resolve to the correct file
    let target = fs::read_link(&symlink_path).expect("read symlink");
    assert!(target.to_string_lossy().contains("api/auth/docs/auth-helpers.md"));
}

#[test]
fn integration_idempotent_sync() {
    let env = TestEnv::new();
    env.create_dir("docs");

    create_skill_file(&env, "docs/stable-skill.md", "LSTAB1", "stable-skill", "A stable skill");
    index_skill_document(
        &env,
        "LSTAB1",
        "docs/stable-skill.md",
        "stable-skill",
        "A stable skill",
        true,
    );

    // Run sync multiple times
    let result1 = sync_symlinks(env.conn(), env.repo_root()).expect("sync 1");
    let result2 = sync_symlinks(env.conn(), env.repo_root()).expect("sync 2");
    let result3 = sync_symlinks(env.conn(), env.repo_root()).expect("sync 3");

    assert_eq!(result1.created, 1, "First sync creates");
    assert_eq!(result2.created, 0, "Second sync is idempotent");
    assert_eq!(result2.updated, 0, "Second sync is idempotent");
    assert_eq!(result3.created, 0, "Third sync is idempotent");
    assert_eq!(result3.updated, 0, "Third sync is idempotent");

    let symlink_path = env.repo_root().join(".claude/skills/stable-skill.md");
    assert!(symlink_path.is_symlink(), "Symlink persists through multiple syncs");
}

// =============================================================================
// Startup behavior simulation tests
// =============================================================================

#[test]
fn integration_fresh_repo_startup_creates_directory_and_symlinks() {
    let env = TestEnv::new();
    env.create_dir("docs");

    // Verify .claude/skills doesn't exist initially
    let skills_dir = env.repo_root().join(".claude/skills");
    assert!(!skills_dir.exists(), "Skills dir should not exist initially");

    // Create skill document
    create_skill_file(
        &env,
        "docs/startup-skill.md",
        "LSTART",
        "startup-skill",
        "Created before startup",
    );
    index_skill_document(
        &env,
        "LSTART",
        "docs/startup-skill.md",
        "startup-skill",
        "Created before startup",
        true,
    );

    // Simulate startup
    let result = sync_symlinks(env.conn(), env.repo_root()).expect("startup sync");

    assert!(skills_dir.exists(), "Skills dir should be created on startup");
    assert_eq!(result.created, 1, "Should create symlink on startup");
}

#[test]
fn integration_startup_with_stale_symlinks_cleans_up() {
    use std::os::unix::fs::symlink;

    let env = TestEnv::new();
    env.create_dir("docs");

    // Create skills directory with stale symlink manually
    let skills_dir = env.repo_root().join(".claude/skills");
    fs::create_dir_all(&skills_dir).expect("create skills dir");

    // Create a stale symlink pointing to a document that doesn't exist in index
    env.write_file("docs/old-skill.md", "# Old Skill");
    symlink("../docs/old-skill.md", skills_dir.join("old-skill.md")).expect("create stale symlink");

    // Create a current skill document
    create_skill_file(&env, "docs/current-skill.md", "LCURR1", "current-skill", "Current skill");
    index_skill_document(
        &env,
        "LCURR1",
        "docs/current-skill.md",
        "current-skill",
        "Current skill",
        true,
    );

    // Startup sync should clean up stale and create current
    let result = sync_symlinks(env.conn(), env.repo_root()).expect("startup sync");

    assert_eq!(result.removed, 1, "Should remove stale symlink");
    assert_eq!(result.created, 1, "Should create current symlink");

    assert!(!skills_dir.join("old-skill.md").exists(), "Stale symlink removed");
    assert!(skills_dir.join("current-skill.md").is_symlink(), "Current symlink exists");
}
