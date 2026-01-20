//! Tests for the skill symlink manager.

use std::fs;
use std::os::unix::fs::symlink;

use lattice::document::frontmatter_schema::TaskType;
use lattice::index::document_types::InsertDocument;
use lattice::index::{connection_pool, document_queries, schema_definition};
use lattice::skill::symlink_manager::{SyncResult, sync_symlinks};
use tempfile::TempDir;

/// Creates an in-memory database with the Lattice schema for testing.
fn create_test_db() -> rusqlite::Connection {
    let conn =
        connection_pool::open_memory_connection().expect("Failed to open in-memory connection");
    schema_definition::create_schema(&conn).expect("Failed to create schema");
    conn
}

/// Creates a test document with skill=true.
fn create_skill_document(id: &str, path: &str, name: &str) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        format!("Skill document: {name}"),
        None,
        None,
        None,
        None,
        None,
        "hash123".to_string(),
        100,
        true, // skill = true
    )
}

/// Creates a test document with skill=false.
fn create_non_skill_document(id: &str, path: &str, name: &str) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        format!("Non-skill document: {name}"),
        Some(TaskType::Task),
        Some(2),
        None,
        None,
        None,
        "hash456".to_string(),
        100,
        false, // skill = false
    )
}

/// Sets up a temp directory with the required structure for testing.
fn setup_temp_repo() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    // Create a docs directory to hold skill documents
    fs::create_dir_all(temp_dir.path().join("docs")).expect("Failed to create docs dir");
    temp_dir
}

// =============================================================================
// sync_symlinks - basic functionality
// =============================================================================

#[test]
fn sync_symlinks_creates_skills_directory_if_missing() {
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    let skills_dir = temp_dir.path().join(".claude/skills");
    assert!(!skills_dir.exists(), "Skills dir should not exist initially");

    let result = sync_symlinks(&conn, temp_dir.path());
    assert!(result.is_ok());
    assert!(skills_dir.exists(), "Skills dir should be created");
}

#[test]
fn sync_symlinks_creates_symlink_for_skill_document() {
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    // Create the actual document file
    let doc_path = temp_dir.path().join("docs/my-skill.md");
    fs::write(&doc_path, "# My Skill\nContent here").expect("Failed to write doc");

    // Insert skill document into index
    let doc = create_skill_document("LSKILL", "docs/my-skill.md", "my-skill");
    document_queries::insert(&conn, &doc).expect("Failed to insert document");

    let result = sync_symlinks(&conn, temp_dir.path()).expect("sync_symlinks should succeed");

    assert_eq!(result.created, 1, "Should create one symlink");
    assert_eq!(result.updated, 0);
    assert_eq!(result.removed, 0);

    let symlink_path = temp_dir.path().join(".claude/skills/my-skill.md");
    assert!(symlink_path.is_symlink(), "Symlink should exist");

    // Verify symlink points to correct target
    let target = fs::read_link(&symlink_path).expect("Failed to read symlink");
    assert!(target.to_string_lossy().contains("docs/my-skill.md"));
}

#[test]
fn sync_symlinks_ignores_non_skill_documents() {
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    // Insert non-skill document
    let doc = create_non_skill_document("LTASK1", "docs/task.md", "task-one");
    document_queries::insert(&conn, &doc).expect("Failed to insert document");

    let result = sync_symlinks(&conn, temp_dir.path()).expect("sync_symlinks should succeed");

    assert_eq!(result.created, 0, "Should not create symlinks for non-skill docs");

    let symlink_path = temp_dir.path().join(".claude/skills/task-one.md");
    assert!(!symlink_path.exists(), "No symlink should exist for non-skill doc");
}

#[test]
fn sync_symlinks_creates_multiple_symlinks() {
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    // Create actual document files
    fs::write(temp_dir.path().join("docs/skill-a.md"), "# Skill A").expect("write");
    fs::write(temp_dir.path().join("docs/skill-b.md"), "# Skill B").expect("write");

    // Insert multiple skill documents
    let doc1 = create_skill_document("LSKIL1", "docs/skill-a.md", "skill-a");
    let doc2 = create_skill_document("LSKIL2", "docs/skill-b.md", "skill-b");
    document_queries::insert(&conn, &doc1).expect("insert");
    document_queries::insert(&conn, &doc2).expect("insert");

    let result = sync_symlinks(&conn, temp_dir.path()).expect("sync_symlinks should succeed");

    assert_eq!(result.created, 2, "Should create two symlinks");

    assert!(temp_dir.path().join(".claude/skills/skill-a.md").is_symlink());
    assert!(temp_dir.path().join(".claude/skills/skill-b.md").is_symlink());
}

// =============================================================================
// sync_symlinks - update behavior
// =============================================================================

#[test]
fn sync_symlinks_updates_symlink_when_target_changes() {
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    // Create skills directory and initial symlink pointing to old location
    let skills_dir = temp_dir.path().join(".claude/skills");
    fs::create_dir_all(&skills_dir).expect("create skills dir");

    let new_target = temp_dir.path().join("docs/new-path.md");
    fs::write(&new_target, "# Skill content").expect("write new target");

    // Create symlink pointing to old (non-existent) location
    let symlink_path = skills_dir.join("my-skill.md");
    symlink("../docs/old-path.md", &symlink_path).expect("create initial symlink");

    // Insert document with new path
    let doc = create_skill_document("LSKILL", "docs/new-path.md", "my-skill");
    document_queries::insert(&conn, &doc).expect("insert");

    let result = sync_symlinks(&conn, temp_dir.path()).expect("sync_symlinks should succeed");

    assert_eq!(result.updated, 1, "Should update one symlink");

    // Verify symlink now points to new target
    let target = fs::read_link(&symlink_path).expect("read symlink");
    assert!(target.to_string_lossy().contains("new-path.md"));
}

#[test]
fn sync_symlinks_leaves_unchanged_symlinks_alone() {
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    // Create document file
    let doc_path = temp_dir.path().join("docs/skill.md");
    fs::write(&doc_path, "# Skill").expect("write");

    // Insert skill document
    let doc = create_skill_document("LSKILL", "docs/skill.md", "skill");
    document_queries::insert(&conn, &doc).expect("insert");

    // First sync creates the symlink
    let result1 = sync_symlinks(&conn, temp_dir.path()).expect("first sync");
    assert_eq!(result1.created, 1);

    // Second sync should leave it unchanged
    let result2 = sync_symlinks(&conn, temp_dir.path()).expect("second sync");
    assert_eq!(result2.created, 0, "Should not create new symlinks");
    assert_eq!(result2.updated, 0, "Should not update unchanged symlinks");
}

// =============================================================================
// sync_symlinks - removal behavior
// =============================================================================

#[test]
fn sync_symlinks_removes_stale_symlinks() {
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    // Create skills directory with a stale symlink
    let skills_dir = temp_dir.path().join(".claude/skills");
    fs::create_dir_all(&skills_dir).expect("create skills dir");

    // Create a document file for the stale symlink to point to
    fs::write(temp_dir.path().join("docs/old-skill.md"), "# Old").expect("write");
    let stale_symlink = skills_dir.join("old-skill.md");
    symlink("../docs/old-skill.md", &stale_symlink).expect("create stale symlink");

    // Do NOT insert any skill documents - the stale symlink should be removed
    let result = sync_symlinks(&conn, temp_dir.path()).expect("sync_symlinks should succeed");

    assert_eq!(result.removed, 1, "Should remove stale symlink");
    assert!(!stale_symlink.exists(), "Stale symlink should be removed");
}

#[test]
fn sync_symlinks_cleans_up_orphaned_symlinks() {
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    // Create skills directory with orphaned symlink (points to non-existent file)
    // The symlink name matches a skill document, but the target file was deleted
    let skills_dir = temp_dir.path().join(".claude/skills");
    fs::create_dir_all(&skills_dir).expect("create skills dir");

    // Insert a skill document whose file no longer exists
    // (simulating a file that was deleted but symlink remains)
    let doc = create_skill_document("LORPHN", "docs/deleted.md", "orphan");
    document_queries::insert(&conn, &doc).expect("insert");

    // Create a symlink for this document - but the target file doesn't exist
    let orphan_symlink = skills_dir.join("orphan.md");
    symlink("../docs/deleted.md", &orphan_symlink).expect("create orphan symlink");

    // Also insert a valid skill document
    fs::write(temp_dir.path().join("docs/valid.md"), "# Valid").expect("write");
    let doc = create_skill_document("LVALID", "docs/valid.md", "valid");
    document_queries::insert(&conn, &doc).expect("insert");

    let result = sync_symlinks(&conn, temp_dir.path()).expect("sync_symlinks should succeed");

    assert_eq!(result.orphans_cleaned, 1, "Should clean orphaned symlink");
    assert!(!orphan_symlink.exists(), "Orphaned symlink should be removed");
    assert!(skills_dir.join("valid.md").is_symlink(), "Valid symlink should exist");
}

// =============================================================================
// sync_symlinks - edge cases
// =============================================================================

#[test]
fn sync_symlinks_handles_empty_database() {
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    let result = sync_symlinks(&conn, temp_dir.path()).expect("sync_symlinks should succeed");

    assert_eq!(result.created, 0);
    assert_eq!(result.updated, 0);
    assert_eq!(result.removed, 0);
    assert_eq!(result.orphans_cleaned, 0);
}

#[test]
fn sync_symlinks_handles_skill_in_subdirectory() {
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    // Create nested directory structure
    fs::create_dir_all(temp_dir.path().join("auth/docs")).expect("create nested dir");
    fs::write(temp_dir.path().join("auth/docs/auth-skill.md"), "# Auth Skill").expect("write");

    let doc = create_skill_document("LAUTH1", "auth/docs/auth-skill.md", "auth-skill");
    document_queries::insert(&conn, &doc).expect("insert");

    let result = sync_symlinks(&conn, temp_dir.path()).expect("sync_symlinks should succeed");

    assert_eq!(result.created, 1);

    let symlink_path = temp_dir.path().join(".claude/skills/auth-skill.md");
    assert!(symlink_path.is_symlink());

    // Verify the symlink target is relative and goes up to reach auth/docs
    let target = fs::read_link(&symlink_path).expect("read symlink");
    assert!(target.to_string_lossy().contains("auth/docs/auth-skill.md"));
}

#[test]
fn sync_symlinks_preserves_non_symlink_files() {
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    // Create skills directory with a regular file (not a symlink)
    let skills_dir = temp_dir.path().join(".claude/skills");
    fs::create_dir_all(&skills_dir).expect("create skills dir");

    let regular_file = skills_dir.join("readme.txt");
    fs::write(&regular_file, "This is not a symlink").expect("write regular file");

    let result = sync_symlinks(&conn, temp_dir.path()).expect("sync_symlinks should succeed");

    // Regular files should not be counted or removed
    assert_eq!(result.removed, 0);
    assert!(regular_file.exists(), "Regular file should be preserved");
}

#[test]
fn sync_symlinks_handles_closed_skill_documents() {
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    // Create a closed skill document (in .closed directory)
    fs::create_dir_all(temp_dir.path().join("docs/.closed")).expect("create closed dir");
    fs::write(temp_dir.path().join("docs/.closed/old-skill.md"), "# Old Skill").expect("write");

    // Note: The document path contains .closed, skill=true
    let doc = InsertDocument::new(
        "LCLOSD".to_string(),
        None,
        "docs/.closed/old-skill.md".to_string(),
        "old-skill".to_string(),
        "A closed skill".to_string(),
        None,
        None,
        None,
        None,
        None,
        "hash789".to_string(),
        100,
        true, // skill = true (even though closed)
    );
    document_queries::insert(&conn, &doc).expect("insert");

    // sync_symlinks should still create symlinks for closed skill documents
    // (filtering by closed state is done at query level with DocumentFilter)
    let result = sync_symlinks(&conn, temp_dir.path()).expect("sync_symlinks should succeed");

    assert_eq!(result.created, 1, "Should create symlink even for closed skill docs");
}

// =============================================================================
// SyncResult tests
// =============================================================================

#[test]
fn sync_result_default_is_all_zeros() {
    let result = SyncResult::default();
    assert_eq!(result.created, 0);
    assert_eq!(result.updated, 0);
    assert_eq!(result.removed, 0);
    assert_eq!(result.orphans_cleaned, 0);
}

// =============================================================================
// Concurrency and race condition tests
// =============================================================================

#[test]
fn sync_symlinks_handles_concurrent_sync_operations() {
    // Simulates two lat processes trying to sync at the same time.
    // Both should succeed without corrupting the symlink state.
    let temp_dir = setup_temp_repo();
    let conn1 = create_test_db();
    let conn2 = create_test_db();

    // Create document file
    fs::write(temp_dir.path().join("docs/skill.md"), "# Skill").expect("write");

    // Insert same skill document into both connections
    let doc = create_skill_document("LSKILL", "docs/skill.md", "skill");
    document_queries::insert(&conn1, &doc).expect("insert into conn1");
    document_queries::insert(&conn2, &doc).expect("insert into conn2");

    // Run sync from both "processes" - both should succeed
    let result1 = sync_symlinks(&conn1, temp_dir.path()).expect("first sync should succeed");
    let result2 = sync_symlinks(&conn2, temp_dir.path()).expect("second sync should succeed");

    // First sync creates, second finds it unchanged
    assert_eq!(result1.created, 1, "First sync should create the symlink");
    assert_eq!(result2.created, 0, "Second sync should find symlink already exists");
    assert_eq!(result2.updated, 0, "Second sync should not need to update");

    // Symlink should exist and point to correct target
    let symlink_path = temp_dir.path().join(".claude/skills/skill.md");
    assert!(symlink_path.is_symlink(), "Symlink should exist after concurrent syncs");
}

#[test]
fn sync_symlinks_handles_file_deleted_between_query_and_sync() {
    // Simulates race condition where document file is deleted after
    // querying the index but before creating the symlink.
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    // Create document file
    let doc_path = temp_dir.path().join("docs/skill.md");
    fs::write(&doc_path, "# Skill").expect("write");

    // Insert skill document into index
    let doc = create_skill_document("LSKILL", "docs/skill.md", "skill");
    document_queries::insert(&conn, &doc).expect("insert");

    // Delete the file to simulate race condition
    fs::remove_file(&doc_path).expect("delete file");

    // Sync should still succeed (symlink will be created but point to non-existent
    // file)
    let result = sync_symlinks(&conn, temp_dir.path()).expect("sync should succeed");

    // The symlink is created but points to a non-existent file
    // The cleanup_orphaned_symlinks phase will then clean it up
    // This verifies the sync doesn't panic when the file is missing
    assert!(
        result.created == 1 || result.orphans_cleaned >= 1,
        "Should either create symlink or clean orphan"
    );
}

#[test]
fn sync_symlinks_handles_skill_flag_removed_between_syncs() {
    // Simulates race condition where skill flag is removed after first sync
    // but before second sync starts.
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    // Create document file and insert as skill
    fs::write(temp_dir.path().join("docs/skill.md"), "# Skill").expect("write");
    let doc = create_skill_document("LSKILL", "docs/skill.md", "skill");
    document_queries::insert(&conn, &doc).expect("insert");

    // First sync creates symlink
    let result1 = sync_symlinks(&conn, temp_dir.path()).expect("first sync");
    assert_eq!(result1.created, 1);

    // Simulate removing skill flag: delete document and reinsert without skill
    document_queries::delete_by_id(&conn, "LSKILL").expect("delete");
    let non_skill_doc = create_non_skill_document("LSKILL", "docs/skill.md", "skill");
    document_queries::insert(&conn, &non_skill_doc).expect("reinsert as non-skill");

    // Second sync should remove the now-stale symlink
    let result2 = sync_symlinks(&conn, temp_dir.path()).expect("second sync");
    assert_eq!(result2.removed, 1, "Should remove symlink when skill flag is removed");

    let symlink_path = temp_dir.path().join(".claude/skills/skill.md");
    assert!(!symlink_path.exists(), "Symlink should be removed");
}

#[test]
fn sync_symlinks_handles_document_moved_between_syncs() {
    // Simulates race condition where document is moved to new path
    // between two sync operations.
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    // Create initial document
    fs::create_dir_all(temp_dir.path().join("old")).expect("create old dir");
    fs::write(temp_dir.path().join("old/skill.md"), "# Skill").expect("write");
    let doc = create_skill_document("LSKILL", "old/skill.md", "skill");
    document_queries::insert(&conn, &doc).expect("insert");

    // First sync
    let result1 = sync_symlinks(&conn, temp_dir.path()).expect("first sync");
    assert_eq!(result1.created, 1);

    // Simulate document move: delete and reinsert with new path
    document_queries::delete_by_id(&conn, "LSKILL").expect("delete");
    fs::create_dir_all(temp_dir.path().join("new")).expect("create new dir");
    fs::write(temp_dir.path().join("new/skill.md"), "# Skill").expect("write new");
    let moved_doc = create_skill_document("LSKILL", "new/skill.md", "skill");
    document_queries::insert(&conn, &moved_doc).expect("reinsert with new path");

    // Second sync should update symlink to point to new location
    let result2 = sync_symlinks(&conn, temp_dir.path()).expect("second sync");
    assert_eq!(result2.updated, 1, "Should update symlink to new path");

    let symlink_path = temp_dir.path().join(".claude/skills/skill.md");
    let target = fs::read_link(&symlink_path).expect("read symlink");
    assert!(
        target.to_string_lossy().contains("new/skill.md"),
        "Symlink should point to new location"
    );
}

#[test]
fn sync_symlinks_handles_symlink_manually_deleted_between_syncs() {
    // Simulates external process deleting symlink between sync operations.
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    // Create document and sync
    fs::write(temp_dir.path().join("docs/skill.md"), "# Skill").expect("write");
    let doc = create_skill_document("LSKILL", "docs/skill.md", "skill");
    document_queries::insert(&conn, &doc).expect("insert");

    let result1 = sync_symlinks(&conn, temp_dir.path()).expect("first sync");
    assert_eq!(result1.created, 1);

    // Manually delete the symlink (simulating external process)
    let symlink_path = temp_dir.path().join(".claude/skills/skill.md");
    fs::remove_file(&symlink_path).expect("manually delete symlink");

    // Second sync should recreate the symlink
    let result2 = sync_symlinks(&conn, temp_dir.path()).expect("second sync");
    assert_eq!(result2.created, 1, "Should recreate manually deleted symlink");
    assert!(symlink_path.is_symlink(), "Symlink should be recreated");
}

#[test]
fn sync_symlinks_handles_skills_dir_deleted_between_syncs() {
    // Simulates .claude/skills directory being deleted externally.
    let temp_dir = setup_temp_repo();
    let conn = create_test_db();

    // Create document and sync
    fs::write(temp_dir.path().join("docs/skill.md"), "# Skill").expect("write");
    let doc = create_skill_document("LSKILL", "docs/skill.md", "skill");
    document_queries::insert(&conn, &doc).expect("insert");

    let result1 = sync_symlinks(&conn, temp_dir.path()).expect("first sync");
    assert_eq!(result1.created, 1);

    // Delete the entire skills directory
    let skills_dir = temp_dir.path().join(".claude/skills");
    fs::remove_dir_all(&skills_dir).expect("delete skills dir");

    // Second sync should recreate directory and symlink
    let result2 = sync_symlinks(&conn, temp_dir.path()).expect("second sync");
    assert_eq!(result2.created, 1, "Should recreate symlink after directory deleted");
    assert!(skills_dir.exists(), "Skills directory should be recreated");
}
