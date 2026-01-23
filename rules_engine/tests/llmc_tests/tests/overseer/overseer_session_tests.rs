use std::fs;
use std::path::PathBuf;

use llmc::overseer_mode::overseer_session::{
    create_overseer_claude_hooks_with_root, get_overseer_session_name, is_overseer_session,
};
use tempfile::TempDir;

#[test]
fn overseer_session_name_uses_session_prefix() {
    let name = get_overseer_session_name();
    assert!(
        name.ends_with("-overseer"),
        "Overseer session name should end with '-overseer', got: {}",
        name
    );
}

#[test]
fn overseer_session_name_default_is_llmc_overseer() {
    let name = get_overseer_session_name();
    assert_eq!(
        name, "llmc-overseer",
        "Without LLMC_ROOT override, overseer session name should be 'llmc-overseer'"
    );
}

#[test]
fn is_overseer_session_returns_true_for_overseer_name() {
    let overseer_name = get_overseer_session_name();
    let result = is_overseer_session(&overseer_name);

    assert!(result, "Should return true for the overseer session name");
}

#[test]
fn is_overseer_session_returns_false_for_worker_sessions() {
    let worker_sessions = ["llmc-adam", "llmc-baker", "llmc-auto-1", "llmc-console1"];

    for session in worker_sessions {
        let result = is_overseer_session(session);
        assert!(!result, "Should return false for worker session '{}', but got true", session);
    }
}

#[test]
fn is_overseer_session_returns_false_for_similar_names() {
    let similar_names =
        ["llmc-overseer1", "llmc-overseer-backup", "overseer", "LLMC-OVERSEER", "llmc_overseer"];

    for name in similar_names {
        let result = is_overseer_session(name);
        assert!(!result, "Should return false for similar name '{}', but got true", name);
    }
}

#[test]
fn is_overseer_session_returns_false_for_empty_string() {
    let result = is_overseer_session("");

    assert!(!result, "Should return false for empty string");
}

#[test]
fn overseer_hooks_include_llmc_root() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp_dir.path();
    let llmc_root = PathBuf::from("/test/llmc/root");
    create_overseer_claude_hooks_with_root(project_dir, &llmc_root)
        .expect("Failed to create overseer hooks");
    let settings_path = project_dir.join(".claude").join("settings.json");
    assert!(settings_path.exists(), "Settings file should exist at {}", settings_path.display());
    let content = fs::read_to_string(&settings_path).expect("Failed to read settings");
    assert!(
        content.contains("LLMC_ROOT=/test/llmc/root"),
        "Hook settings should include LLMC_ROOT. Content:\n{}",
        content
    );
}

#[test]
fn overseer_hooks_regenerated_when_llmc_root_changes() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp_dir.path();
    let old_root = PathBuf::from("/old/llmc/root");
    let new_root = PathBuf::from("/new/llmc/root");
    create_overseer_claude_hooks_with_root(project_dir, &old_root)
        .expect("Failed to create initial hooks");
    let settings_path = project_dir.join(".claude").join("settings.json");
    let initial_content = fs::read_to_string(&settings_path).expect("Failed to read settings");
    assert!(
        initial_content.contains("LLMC_ROOT=/old/llmc/root"),
        "Initial hooks should have old root. Content:\n{}",
        initial_content
    );
    create_overseer_claude_hooks_with_root(project_dir, &new_root)
        .expect("Failed to create hooks with new root");
    let updated_content = fs::read_to_string(&settings_path).expect("Failed to read settings");
    assert!(
        updated_content.contains("LLMC_ROOT=/new/llmc/root"),
        "Hooks should be regenerated with new LLMC_ROOT. Content:\n{}",
        updated_content
    );
    assert!(
        !updated_content.contains("LLMC_ROOT=/old/llmc/root"),
        "Old LLMC_ROOT should be replaced. Content:\n{}",
        updated_content
    );
}

#[test]
fn overseer_hooks_all_hooks_include_llmc_root() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp_dir.path();
    let llmc_root = PathBuf::from("/test/llmc/root");
    create_overseer_claude_hooks_with_root(project_dir, &llmc_root)
        .expect("Failed to create overseer hooks");
    let settings_path = project_dir.join(".claude").join("settings.json");
    let content = fs::read_to_string(&settings_path).expect("Failed to read settings");
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse settings JSON");
    let hooks = parsed.get("hooks").expect("Settings should have 'hooks' key");
    for hook_type in ["Stop", "SessionStart", "SessionEnd"] {
        let hook_array = hooks.get(hook_type).expect(&format!("Should have {} hook", hook_type));
        let hook_config = hook_array
            .get(0)
            .expect(&format!("{} hook should have at least one config", hook_type));
        let inner_hooks = hook_config
            .get("hooks")
            .expect(&format!("{} hook config should have 'hooks' array", hook_type));
        let hook_entry = inner_hooks
            .get(0)
            .expect(&format!("{} should have at least one hook entry", hook_type));
        let command = hook_entry
            .get("command")
            .and_then(|v| v.as_str())
            .expect(&format!("{} hook should have 'command' string", hook_type));
        assert!(
            command.starts_with("LLMC_ROOT=/test/llmc/root"),
            "{} hook command should start with 'LLMC_ROOT=/test/llmc/root'. Got: {}",
            hook_type,
            command
        );
        assert!(
            command.contains("--worker overseer"),
            "{} hook command should include '--worker overseer'. Got: {}",
            hook_type,
            command
        );
    }
}
