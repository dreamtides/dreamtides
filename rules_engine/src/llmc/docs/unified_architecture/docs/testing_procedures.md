---
lattice-id: LB6WQN
name: testing-procedures
description: Unit test additions, integration test script, and manual testing checklists.
parent-id: LBUWQN
created-at: 2026-01-19T05:00:00Z
updated-at: 2026-01-19T05:08:18.132983Z
---

# Testing Procedures

## Unit Test Additions

Add to existing test files:

```rust
// In config.rs tests
#[test]
fn test_worktree_dir_default() {
    let config = Config::from_str(r#"
        [repo]
        source = "/tmp/test-repo"
    "#).unwrap();
    assert_eq!(config.worktree_dir(), PathBuf::from("/tmp/test-repo/.llmc-worktrees"));
}

#[test]
fn test_worktree_dir_override() {
    let config = Config::from_str(r#"
        [repo]
        source = "/tmp/test-repo"
        worktree_dir = "/custom/worktrees"
    "#).unwrap();
    assert_eq!(config.worktree_dir(), PathBuf::from("/custom/worktrees"));
}

// In git.rs tests
#[test]
fn test_is_merge_in_progress() {
    let repo = create_test_repo();
    assert!(!git::is_merge_in_progress(&repo).unwrap());

    // Create MERGE_HEAD file
    fs::write(repo.join(".git/MERGE_HEAD"), "abc123").unwrap();
    assert!(git::is_merge_in_progress(&repo).unwrap());
}
```

## Manual Testing Checklist

### Phase 1 Testing (After Session 1)

```
□ Clean slate test
  rm -rf ~/llmc ~/.llmc-worktrees

□ Init creates correct structure
  llmc init
  ls -la ~/llmc/

  # Should see: config.toml, state.json, logs/, NO .git/

□ Gitignore updated
  cat ~/Documents/GoogleDrive/dreamtides/.gitignore | grep llmc

  # Should see: .llmc-worktrees/

□ Add creates worktree correctly
  llmc add testworker
  ls -la ~/Documents/GoogleDrive/dreamtides/.llmc-worktrees/

  # Should see: testworker/

  git -C ~/Documents/GoogleDrive/dreamtides branch | grep llmc

  # Should see: llmc/testworker

□ Nuke removes everything
  llmc nuke testworker --yes
  ls ~/Documents/GoogleDrive/dreamtides/.llmc-worktrees/

  # Should be empty

```

### Phase 2 Testing (After Session 2)

```
□ Start sends prompt correctly
  llmc up &
  llmc add testworker
  llmc start testworker --prompt "echo 'hello' > test.txt"
  llmc status

  # Should show: working

□ Accept merges to master

  # Wait for worker to complete

  llmc accept testworker
  cat ~/Documents/GoogleDrive/dreamtides/test.txt

  # Should see: hello

□ Accept fails with dirty repo
  echo "dirty" > ~/Documents/GoogleDrive/dreamtides/dirty.txt
  llmc accept testworker

  # Should fail with clear message

□ Accept fails when not on master
  git checkout -b test-branch
  llmc accept testworker

  # Should fail with clear message

```

### Phase 3 Testing (After Session 3)

```
□ Salvage creates patch
  llmc salvage testworker --patch ~/test.patch
  cat ~/test.patch

  # Should contain diff

□ Rescue recovers everything
  echo "invalid" > ~/llmc/state.json
  llmc rescue --yes
  llmc status

  # Should work again

□ Doctor rebuild works
  rm ~/llmc/state.json
  llmc doctor --rebuild
  cat ~/llmc/state.json

  # Should be valid

```
