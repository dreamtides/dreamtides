use std::fs;
use std::path::PathBuf;

use llmc::commands::add::create_claude_hook_settings_with_root;
use tempfile::TempDir;

#[test]
fn test_hook_settings_include_llmc_root_env_var() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let worktree_path = temp_dir.path();
    let llmc_root = PathBuf::from("/test/llmc/root");
    create_claude_hook_settings_with_root(worktree_path, "test-worker", &llmc_root)
        .expect("Failed to create hook settings");
    let settings_path = worktree_path.join(".claude").join("settings.json");
    assert!(settings_path.exists(), "Settings file should exist at {}", settings_path.display());
    let content = fs::read_to_string(&settings_path).expect("Failed to read settings file");
    assert!(
        content.contains("LLMC_ROOT="),
        "Hook settings should include LLMC_ROOT= prefix in command. Content:\n{}",
        content
    );
}

#[test]
fn test_hook_settings_preserve_custom_llmc_root() {
    let custom_root = PathBuf::from("/tmp/custom-llmc-test-12345");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let worktree_path = temp_dir.path();
    create_claude_hook_settings_with_root(worktree_path, "test-worker", &custom_root)
        .expect("Failed to create hook settings");
    let settings_path = worktree_path.join(".claude").join("settings.json");
    let content = fs::read_to_string(&settings_path).expect("Failed to read settings file");
    let expected_prefix = format!("LLMC_ROOT={}", custom_root.display());
    assert!(
        content.contains(&expected_prefix),
        "Hook settings should include LLMC_ROOT={} in command. Content:\n{}",
        custom_root.display(),
        content
    );
}

#[test]
fn test_hook_settings_all_hooks_include_llmc_root() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let worktree_path = temp_dir.path();
    let llmc_root = PathBuf::from("/test/llmc/root");
    create_claude_hook_settings_with_root(worktree_path, "test-worker", &llmc_root)
        .expect("Failed to create hook settings");
    let settings_path = worktree_path.join(".claude").join("settings.json");
    let content = fs::read_to_string(&settings_path).expect("Failed to read settings file");
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
            command.starts_with("LLMC_ROOT="),
            "{} hook command should start with 'LLMC_ROOT='. Got: {}",
            hook_type,
            command
        );
    }
}
