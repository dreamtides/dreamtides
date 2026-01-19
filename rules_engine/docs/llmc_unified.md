# LLMC Unified Architecture: Technical Design Document

## Executive Summary

This document describes the migration of LLMC from a dual-repository architecture
(separate clone + source repo) to a unified architecture where git worktrees are
created directly from the main repository. This simplification eliminates sync
issues, reduces complexity, and provides a cleaner mental model while maintaining
all existing functionality.

**Key Changes:**
- Remove the intermediate git clone at `~/llmc/`
- Create worktrees directly from the source repository
- Simplify the accept flow to merge directly to master
- Add robust recovery tools for edge cases

**Non-Goals:**
- Changing the worker state machine
- Modifying the TMUX integration
- Altering the patrol system's responsibilities (only its implementation)

---

## Table of Contents

1. [Architecture Comparison](#1-architecture-comparison)
2. [Detailed Design](#2-detailed-design)
3. [Configuration Changes](#3-configuration-changes)
4. [Code Changes by File](#4-code-changes-by-file)
5. [Git Operation Safety](#5-git-operation-safety)
6. [Edge Cases and Failure Modes](#6-edge-cases-and-failure-modes)
7. [Recovery Tools](#7-recovery-tools)
8. [Migration Plan](#8-migration-plan)
9. [Opus Session Specifications](#9-opus-session-specifications)
10. [Testing Procedures](#10-testing-procedures)
11. [Logging Specification](#11-logging-specification)
12. [Rollback Plan](#12-rollback-plan)

---

## 1. Architecture Comparison

### 1.1 Current Architecture (Dual-Repository)

```
~/llmc/                               # LLMC workspace (FULL GIT CLONE)
├── .git/                             # Complete clone of source repo
│   └── config                        # origin = ~/Documents/.../dreamtides
├── .worktrees/                       # Worker worktrees (linked to ~/llmc/.git)
│   ├── adam/
│   │   ├── .git                      # File pointing to ~/llmc/.git/worktrees/adam
│   │   └── ... (working files)
│   └── baker/
├── config.toml
├── state.json
├── llmc.sock
└── logs/

~/Documents/GoogleDrive/dreamtides/   # Source repository (truth)
├── .git/
└── ... (working files)
```

**Current Accept Flow:**
```
1. Worker commits in worktree (~/llmc/.worktrees/adam)
2. Rebase worktree onto origin/master
3. Squash commits
4. Checkout master in ~/llmc/
5. Fast-forward merge worker branch to ~/llmc/ master
6. Fetch new commit FROM ~/llmc/ INTO source repo
7. Reset source repo to new commit
8. Recreate worker worktree
```

**Problems with Current Architecture:**
- Two repositories that can drift out of sync
- Complex accept flow with multiple failure points
- The fetch-from-local step is unusual and error-prone
- Disk space duplication (even with --local, git objects are hard-linked but
  new objects are not shared)
- Mental overhead of understanding two repos

### 1.2 New Architecture (Unified)

```
~/Documents/GoogleDrive/dreamtides/   # Source repository (SINGLE REPO)
├── .git/
│   ├── worktrees/                    # Git's internal worktree tracking
│   │   ├── adam/
│   │   └── baker/
│   └── ...
├── .llmc-worktrees/                  # Worker worktrees (NEW LOCATION)
│   ├── adam/
│   │   ├── .git                      # File pointing to main .git/worktrees/adam
│   │   └── ... (working files)
│   └── baker/
└── ... (normal repo contents)

~/llmc/                               # Metadata only (NO .git/)
├── config.toml
├── state.json
├── llmc.sock
└── logs/
```

**New Accept Flow:**
```
1. Worker commits in worktree
2. Rebase worktree onto master (local ref, not origin/master)
3. Squash commits
4. Fast-forward merge worker branch directly to master
5. Recreate worker worktree
```

**Benefits:**
- Single source of truth
- Simpler accept flow (5 steps vs 8)
- No sync issues possible
- Worker branches visible in main repo (can be useful for debugging)
- Standard git worktree usage pattern

---

## 2. Detailed Design

### 2.1 Repository Concepts

| Concept | Old Architecture | New Architecture |
|---------|------------------|------------------|
| Git operations repo | `~/llmc/` | `config.repo.source` |
| Worktree parent dir | `~/llmc/.worktrees/` | `<source>/.llmc-worktrees/` |
| Metadata dir | `~/llmc/` | `~/llmc/` (unchanged) |
| Remote for fetch | `origin` (source repo) | N/A (no remote needed) |
| Master reference | `origin/master` | `master` |

### 2.2 Key Design Decisions

**Decision 1: Worktree Location**

Worktrees will be stored in `<source>/.llmc-worktrees/` rather than a separate
directory because:
- Git requires worktrees to be accessible relative to the main repo
- Keeping them in the repo directory is conventional
- The `.llmc-worktrees/` prefix clearly identifies them as LLMC-managed
- They can be gitignored to avoid clutter

**Decision 2: Branch Naming**

Worker branches remain `llmc/<worker-name>` (e.g., `llmc/adam`). These branches
will now exist directly in the main repository.

**Decision 3: No Remote Operations**

The new architecture eliminates all remote operations within LLMC. The main repo
may have its own remotes (GitHub, etc.) but LLMC never interacts with them.
All operations are local.

**Decision 4: Master Branch Reference**

All references to `origin/master` become simply `master`. This is a significant
change that affects many files.

### 2.3 State File Changes

The `state.json` format remains unchanged. The `worktree_path` field will now
point to paths under `<source>/.llmc-worktrees/` instead of `~/llmc/.worktrees/`.

### 2.4 Gitignore Updates

The source repository's `.gitignore` should include:
```
# LLMC worktrees
.llmc-worktrees/
```

This will be added automatically by `llmc init`.

---

## 3. Configuration Changes

### 3.1 New Config Schema

```toml
[defaults]
model = "opus"
skip_permissions = true
patrol_interval_secs = 60
sound_on_review = true

[repo]
source = "~/Documents/GoogleDrive/dreamtides"
# NEW: Optional override for worktree location
# worktree_dir = "~/Documents/GoogleDrive/dreamtides/.llmc-worktrees"
# NEW: Optional override for metadata location
# metadata_dir = "~/llmc"

[workers.adam]
model = "opus"
```

### 3.2 Config Resolution Logic

```rust
impl Config {
    /// Returns the directory containing worker worktrees
    pub fn worktree_dir(&self) -> PathBuf {
        self.repo.worktree_dir
            .clone()
            .unwrap_or_else(|| {
                PathBuf::from(&self.repo.source).join(".llmc-worktrees")
            })
    }

    /// Returns the directory containing LLMC metadata (config, state, logs)
    pub fn metadata_dir(&self) -> PathBuf {
        self.repo.metadata_dir
            .clone()
            .unwrap_or_else(config::get_llmc_root)
    }

    /// Returns the path to the git repository for all git operations
    pub fn git_repo(&self) -> PathBuf {
        PathBuf::from(&self.repo.source)
    }
}
```

---

## 4. Code Changes by File

### 4.1 Summary Table

| File | Change Type | Complexity | Description |
|------|-------------|------------|-------------|
| `config.rs` | Modify | Medium | Add `worktree_dir()`, `metadata_dir()`, `git_repo()` methods |
| `commands/init.rs` | Rewrite | High | Remove clone, create metadata dir only, update gitignore |
| `commands/add.rs` | Modify | Medium | Use `config.git_repo()` and `config.worktree_dir()` |
| `commands/accept.rs` | Rewrite | High | Simplify flow, remove fetch-into-source |
| `commands/start.rs` | Modify | Low | Update path references |
| `commands/reset.rs` | Modify | Low | Update path references |
| `commands/nuke.rs` | Modify | Low | Update path references |
| `commands/rebase.rs` | Modify | Medium | Change `origin/master` to `master` |
| `commands/doctor.rs` | Modify | Medium | Update health checks for new architecture |
| `commands/review.rs` | Modify | Low | Update path references |
| `patrol.rs` | Modify | Medium | Change `origin/master` to `master` |
| `git.rs` | Modify | Medium | Add safety checks, update fetch logic |
| `state.rs` | No change | - | Format unchanged |
| `worker.rs` | No change | - | Logic unchanged |

### 4.2 Detailed Changes

#### 4.2.1 `config.rs`

**Add to `RepoConfig` struct:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
    pub source: String,
    #[serde(default)]
    pub worktree_dir: Option<String>,
    #[serde(default)]
    pub metadata_dir: Option<String>,
}
```

**Add helper methods to `Config`:**
```rust
impl Config {
    pub fn worktree_dir(&self) -> PathBuf {
        self.repo.worktree_dir
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.source_repo().join(".llmc-worktrees"))
    }

    pub fn source_repo(&self) -> PathBuf {
        // Expand ~ in path
        let path = &self.repo.source;
        if path.starts_with("~/") {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home).join(&path[2..]);
            }
        }
        PathBuf::from(path)
    }
}
```

#### 4.2.2 `commands/init.rs`

**Complete rewrite.** New implementation:

```rust
pub fn run_init(source: Option<PathBuf>, target: Option<PathBuf>, force: bool) -> Result<()> {
    let metadata_dir = target.unwrap_or_else(config::get_llmc_root);
    let source_dir = resolve_source_dir(source)?;

    validate_source_repo(&source_dir)?;

    if metadata_dir.exists() {
        if force {
            println!("Removing existing metadata directory...");
            fs::remove_dir_all(&metadata_dir)?;
        } else {
            bail!("Metadata directory already exists: {}\nUse --force to overwrite",
                  metadata_dir.display());
        }
    }

    println!("Initializing LLMC...");
    println!("  Source repository: {}", source_dir.display());
    println!("  Metadata directory: {}", metadata_dir.display());

    // Create metadata directory structure
    create_metadata_structure(&metadata_dir)?;

    // Create config file
    create_config_file(&metadata_dir, &source_dir)?;

    // Create initial state
    create_initial_state(&metadata_dir)?;

    // Create worktree directory in source repo
    let worktree_dir = source_dir.join(".llmc-worktrees");
    fs::create_dir_all(&worktree_dir)?;

    // Update .gitignore in source repo
    update_gitignore(&source_dir)?;

    println!("\n✓ LLMC initialized successfully!");
    println!("\nNext steps:");
    println!("  1. Review ~/llmc/config.toml");
    println!("  2. Run 'llmc add <name>' to create workers");
    println!("  3. Run 'llmc up' to start the daemon");

    Ok(())
}

fn update_gitignore(source_dir: &Path) -> Result<()> {
    let gitignore_path = source_dir.join(".gitignore");
    let marker = "# LLMC worktrees";
    let entry = ".llmc-worktrees/";

    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path)?;
        if content.contains(entry) {
            return Ok(()); // Already present
        }
        let new_content = format!("{}\n\n{}\n{}\n", content.trim_end(), marker, entry);
        fs::write(&gitignore_path, new_content)?;
    } else {
        fs::write(&gitignore_path, format!("{}\n{}\n", marker, entry))?;
    }

    println!("  Updated .gitignore to exclude worktrees");
    Ok(())
}
```

#### 4.2.3 `commands/accept.rs`

**Simplified accept flow:**

```rust
pub fn run_accept(worker: Option<String>, force: bool, json: bool) -> Result<()> {
    let config = Config::load(&config::get_config_path())?;
    let source_repo = config.source_repo();

    // ... worker selection logic unchanged ...

    let worktree_path = PathBuf::from(&worker_record.worktree_path);

    // Validate source repo state
    validate_source_repo_state(&source_repo)?;

    // Amend any uncommitted changes
    if git::has_uncommitted_changes(&worktree_path)? {
        git::amend_uncommitted_changes(&worktree_path)?;
    }

    // Rebase onto master (NOT origin/master)
    println!("Rebasing onto master...");
    let rebase_result = git::rebase_onto(&worktree_path, "master")?;

    if !rebase_result.success {
        // Handle conflicts - transition to rebasing state
        // ... existing conflict handling ...
        return Ok(());
    }

    // Squash commits
    println!("Squashing commits...");
    git::squash_commits(&worktree_path, "master")?;

    // Check if there are actually changes to merge
    if !git::has_staged_changes(&worktree_path)? {
        println!("No changes to merge - work already incorporated");
        reset_worker_to_idle(&worker_name, &source_repo, &config, &mut state)?;
        return Ok(());
    }

    // Create the squashed commit
    let commit_message = git::get_commit_message(&worktree_path, "HEAD")?;
    let cleaned_message = git::strip_agent_attribution(&commit_message);
    create_commit(&worktree_path, &cleaned_message)?;

    let new_commit_sha = git::get_head_commit(&worktree_path)?;

    // Fast-forward merge to master (directly in source repo)
    println!("Merging to master...");
    git::fast_forward_merge(&source_repo, &worker_record.branch)?;

    // Verify merge succeeded
    let master_head = git::get_head_commit(&source_repo)?;
    if master_head != new_commit_sha {
        bail!("Merge verification failed: master={}, expected={}",
              master_head, new_commit_sha);
    }

    // Reset worker to idle with fresh worktree
    reset_worker_to_idle(&worker_name, &source_repo, &config, &mut state)?;

    println!("✓ Changes accepted! Commit: {}", &new_commit_sha[..8]);
    Ok(())
}

fn validate_source_repo_state(repo: &Path) -> Result<()> {
    // Check for uncommitted changes in main worktree
    if git::has_uncommitted_changes(repo)? {
        bail!(
            "Source repository has uncommitted changes.\n\
             Please commit or stash changes in {} before accepting work.",
            repo.display()
        );
    }

    // Check for in-progress operations
    if git::is_rebase_in_progress(repo) {
        bail!(
            "Source repository has a rebase in progress.\n\
             Please complete or abort the rebase in {} first.",
            repo.display()
        );
    }

    if git::is_merge_in_progress(repo)? {
        bail!(
            "Source repository has a merge in progress.\n\
             Please complete or abort the merge in {} first.",
            repo.display()
        );
    }

    // Check we're on master branch
    let current_branch = git::get_current_branch(repo)?;
    if current_branch != "master" {
        bail!(
            "Source repository is not on master branch (currently on '{}').\n\
             Please checkout master in {} first.",
            current_branch, repo.display()
        );
    }

    Ok(())
}
```

#### 4.2.4 `git.rs` - New Functions

**Add merge-in-progress check:**
```rust
pub fn is_merge_in_progress(repo: &Path) -> Result<bool> {
    let git_dir = get_git_dir(repo)?;
    Ok(git_dir.join("MERGE_HEAD").exists())
}
```

**Add cherry-pick-in-progress check:**
```rust
pub fn is_cherry_pick_in_progress(repo: &Path) -> Result<bool> {
    let git_dir = get_git_dir(repo)?;
    Ok(git_dir.join("CHERRY_PICK_HEAD").exists())
}
```

**Modify all `origin/master` references:**

The following functions need `origin/master` changed to `master`:
- `pull_rebase()` - remove fetch, just rebase onto master
- `rebase_onto()` - already parameterized, callers change
- `has_commits_ahead_of()` - callers pass `master` instead of `origin/master`

#### 4.2.5 Global Search-Replace Pattern

In all files, apply these changes:

| Old Pattern | New Pattern | Notes |
|-------------|-------------|-------|
| `origin/master` | `master` | All git ref comparisons |
| `git::fetch_origin(&llmc_root)?` | Remove or replace | No longer needed |
| `config::get_llmc_root()` (for git ops) | `config.source_repo()` | Git operations |
| `llmc_root.join(".worktrees")` | `config.worktree_dir()` | Worktree paths |

---

## 5. Git Operation Safety

### 5.1 Pre-Operation Checks

Before any destructive git operation, verify:

```rust
/// Comprehensive safety check before modifying git state
pub fn verify_safe_to_modify(repo: &Path, operation: &str) -> Result<()> {
    // 1. Verify repo exists and is a git repo
    if !repo.join(".git").exists() && !repo.join("HEAD").exists() {
        bail!("Not a git repository: {}", repo.display());
    }

    // 2. Check for lock files
    let git_dir = get_git_dir(repo)?;
    let lock_files = ["index.lock", "HEAD.lock", "config.lock"];
    for lock in &lock_files {
        let lock_path = git_dir.join(lock);
        if lock_path.exists() {
            bail!(
                "Git lock file exists: {}\n\
                 Another git operation may be in progress.\n\
                 If not, remove the lock file manually.",
                lock_path.display()
            );
        }
    }

    // 3. Check for in-progress operations
    if is_rebase_in_progress(repo) {
        bail!("Rebase in progress - cannot perform {}", operation);
    }
    if is_merge_in_progress(repo)? {
        bail!("Merge in progress - cannot perform {}", operation);
    }
    if is_cherry_pick_in_progress(repo)? {
        bail!("Cherry-pick in progress - cannot perform {}", operation);
    }

    Ok(())
}
```

### 5.2 Atomic Operations

All state-modifying operations should be atomic:

```rust
/// Performs an operation with automatic rollback on failure
pub fn with_rollback<F, R>(
    repo: &Path,
    description: &str,
    operation: F,
) -> Result<R>
where
    F: FnOnce() -> Result<R>,
{
    // Save current HEAD for rollback
    let original_head = get_head_commit(repo)?;
    let original_branch = get_current_branch(repo).ok();

    match operation() {
        Ok(result) => Ok(result),
        Err(e) => {
            tracing::error!(
                "Operation '{}' failed, attempting rollback to {}",
                description, original_head
            );

            // Attempt rollback
            if let Err(rollback_err) = reset_to_ref(repo, &original_head) {
                tracing::error!(
                    "Rollback failed: {}. Manual intervention required.",
                    rollback_err
                );
            }

            Err(e.context(format!("Operation '{}' failed", description)))
        }
    }
}
```

### 5.3 Worktree Safety

```rust
/// Safely removes a worktree with verification
pub fn safe_remove_worktree(repo: &Path, worktree: &Path) -> Result<()> {
    // 1. Verify worktree exists
    if !worktree.exists() {
        tracing::warn!("Worktree doesn't exist, skipping removal: {}", worktree.display());
        return Ok(());
    }

    // 2. Verify it's actually a worktree (has .git file, not directory)
    let git_file = worktree.join(".git");
    if git_file.is_dir() {
        bail!(
            "Path appears to be a full git repo, not a worktree: {}\n\
             Refusing to remove to prevent data loss.",
            worktree.display()
        );
    }

    // 3. Check for uncommitted changes
    if has_uncommitted_changes(worktree)? {
        tracing::warn!(
            "Worktree has uncommitted changes that will be lost: {}",
            worktree.display()
        );
    }

    // 4. Remove with force flag
    remove_worktree(repo, worktree, true)
}
```

---

## 6. Edge Cases and Failure Modes

### 6.1 Git State Edge Cases

| Scenario | Detection | Handling |
|----------|-----------|----------|
| Lock file exists | Check `index.lock` | Retry with backoff, then fail with clear message |
| Rebase in progress | Check `.git/rebase-merge/` | Block operation, instruct user to resolve |
| Merge in progress | Check `MERGE_HEAD` | Block operation, instruct user to resolve |
| Detached HEAD in worktree | `git symbolic-ref HEAD` fails | Reattach to branch or reset |
| Worktree on wrong branch | Compare branch name | Reset to correct branch |
| Orphaned worktree (no branch) | Branch doesn't exist | Remove and recreate |
| Corrupted worktree | Various git errors | Remove and recreate |
| Main repo dirty | `has_uncommitted_changes()` | Block accept, inform user |
| Main repo not on master | `get_current_branch()` | Block accept, inform user |

### 6.2 State Inconsistencies

| Scenario | Detection | Handling |
|----------|-----------|----------|
| Worker in state, no worktree | Path doesn't exist | Mark offline, recreate on next start |
| Worktree exists, not in state | Scan `.llmc-worktrees/` | Add to state or remove worktree |
| Branch exists, no worker | List `llmc/*` branches | Delete orphaned branch |
| Worker online, no TMUX session | `session_exists()` returns false | Mark offline |
| Session exists, worker offline | Session check in patrol | Mark online or kill session |
| Commit SHA mismatch | Compare stored vs actual | Update state |

### 6.3 Failure Recovery Matrix

| Failure Point | Symptoms | Automatic Recovery | Manual Recovery |
|---------------|----------|-------------------|-----------------|
| Init interrupted | Partial metadata dir | `llmc init --force` | Remove ~/llmc, reinit |
| Add interrupted | Orphaned branch/worktree | `llmc doctor --repair` | Manual cleanup |
| Accept during rebase | Worker stuck in rebasing | Patrol detects, prompts worker | `llmc reset <worker>` |
| Accept during squash | Partially squashed | `llmc reset <worker>` | Manual git operations |
| Accept during merge | Merge conflict | Should not happen (ff-only) | `git merge --abort` |
| Nuke interrupted | Partial cleanup | `llmc doctor --repair` | Manual cleanup |
| Reset interrupted | Inconsistent state | `llmc doctor --repair` | `llmc nuke` + `llmc add` |

### 6.4 Concurrent Operation Safety

```rust
/// Operations that must not run concurrently
const EXCLUSIVE_OPERATIONS: &[&str] = &[
    "accept",
    "reset",
    "nuke",
    "add",
    "init",
];

/// Acquire exclusive lock for dangerous operations
pub fn acquire_exclusive_lock(operation: &str) -> Result<ExclusiveLock> {
    let lock_path = config::get_llmc_root().join(".llmc-exclusive.lock");

    // Try to acquire lock with timeout
    let start = Instant::now();
    let timeout = Duration::from_secs(30);

    loop {
        match ExclusiveLock::try_acquire(&lock_path, operation) {
            Ok(lock) => return Ok(lock),
            Err(_) if start.elapsed() < timeout => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## 7. Recovery Tools

### 7.1 `llmc doctor` Enhancements

```rust
pub fn run_doctor(repair: bool, rebuild: bool, yes: bool) -> Result<()> {
    let mut issues: Vec<Issue> = Vec::new();

    // Check 1: Metadata directory exists
    check_metadata_dir(&mut issues)?;

    // Check 2: Config file valid
    check_config(&mut issues)?;

    // Check 3: State file valid
    check_state(&mut issues)?;

    // Check 4: Source repo accessible and clean
    check_source_repo(&mut issues)?;

    // Check 5: All worktrees exist and are valid
    check_worktrees(&mut issues)?;

    // Check 6: All branches exist
    check_branches(&mut issues)?;

    // Check 7: TMUX sessions match state
    check_tmux_sessions(&mut issues)?;

    // Check 8: No orphaned resources
    check_orphaned_resources(&mut issues)?;

    // Check 9: State consistency
    check_state_consistency(&mut issues)?;

    // Report findings
    report_issues(&issues);

    if repair && !issues.is_empty() {
        if !yes {
            confirm_repair(&issues)?;
        }
        repair_issues(&issues)?;
    }

    if rebuild {
        rebuild_state_from_filesystem()?;
    }

    Ok(())
}
```

### 7.2 `llmc salvage` - New Command

Recovers work from a broken worker state:

```rust
/// Salvages commits from a worker into a patch file or new branch
pub fn run_salvage(worker: &str, output: SalvageOutput) -> Result<()> {
    let config = Config::load(&config::get_config_path())?;
    let source_repo = config.source_repo();
    let worktree_path = config.worktree_dir().join(worker);

    if !worktree_path.exists() {
        bail!("Worker worktree not found: {}", worktree_path.display());
    }

    // Find commits ahead of master
    let commits = git::list_commits_ahead_of(&worktree_path, "master")?;

    if commits.is_empty() {
        println!("No commits to salvage");
        return Ok(());
    }

    println!("Found {} commit(s) to salvage:", commits.len());
    for commit in &commits {
        println!("  {} {}", &commit.sha[..8], commit.subject);
    }

    match output {
        SalvageOutput::Patch(path) => {
            // Create patch file
            let patch = git::format_patch(&worktree_path, "master")?;
            fs::write(&path, patch)?;
            println!("✓ Saved patch to {}", path.display());
        }
        SalvageOutput::Branch(name) => {
            // Create branch in main repo pointing to current HEAD
            let head = git::get_head_commit(&worktree_path)?;
            git::create_branch_at(&source_repo, &name, &head)?;
            println!("✓ Created branch '{}' at {}", name, &head[..8]);
        }
        SalvageOutput::Stdout => {
            // Print patch to stdout
            let patch = git::format_patch(&worktree_path, "master")?;
            println!("{}", patch);
        }
    }

    Ok(())
}
```

### 7.3 `llmc rescue` - New Command

Rescues a completely broken LLMC installation:

```rust
/// Complete rescue operation - salvages all work and rebuilds
pub fn run_rescue(yes: bool) -> Result<()> {
    println!("LLMC Rescue Mode");
    println!("================");
    println!();
    println!("This will:");
    println!("  1. Salvage all uncommitted work from workers");
    println!("  2. Save patches to ~/llmc-rescue/");
    println!("  3. Completely reset LLMC state");
    println!("  4. Preserve your config.toml");
    println!();

    if !yes {
        confirm("Proceed with rescue?")?;
    }

    let rescue_dir = PathBuf::from(std::env::var("HOME")?).join("llmc-rescue");
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let rescue_subdir = rescue_dir.join(timestamp.to_string());
    fs::create_dir_all(&rescue_subdir)?;

    // Salvage each worker
    let state = State::load(&state::get_state_path()).ok();
    let config = Config::load(&config::get_config_path())?;

    if let Some(state) = state {
        for (name, worker) in &state.workers {
            let worktree = PathBuf::from(&worker.worktree_path);
            if worktree.exists() {
                let patch_path = rescue_subdir.join(format!("{}.patch", name));
                match salvage_to_patch(&worktree, &patch_path) {
                    Ok(true) => println!("✓ Salvaged {} to {}", name, patch_path.display()),
                    Ok(false) => println!("  {} has no changes to salvage", name),
                    Err(e) => println!("✗ Failed to salvage {}: {}", name, e),
                }
            }
        }
    }

    // Backup config
    let config_backup = rescue_subdir.join("config.toml");
    fs::copy(config::get_config_path(), &config_backup)?;
    println!("✓ Backed up config to {}", config_backup.display());

    // Nuke everything
    println!("\nResetting LLMC...");
    run_nuke_all(true)?;

    // Rebuild state
    create_initial_state(&config::get_llmc_root())?;

    println!("\n✓ Rescue complete!");
    println!("  Patches saved to: {}", rescue_subdir.display());
    println!("  Config preserved: {}", config_backup.display());
    println!("\nTo restore a worker's changes:");
    println!("  1. llmc add <worker>");
    println!("  2. cd <worktree>");
    println!("  3. git apply {}<worker>.patch", rescue_subdir.display());

    Ok(())
}
```

### 7.4 Automatic Self-Healing in Patrol

```rust
impl Patrol {
    fn self_heal(&self, state: &mut State, config: &Config) -> Vec<String> {
        let mut healed = Vec::new();

        for (name, worker) in state.workers.iter_mut() {
            let worktree = PathBuf::from(&worker.worktree_path);

            // Heal 1: Worktree missing but worker not offline
            if !worktree.exists() && worker.status != WorkerStatus::Offline {
                tracing::warn!("Healing: Worker {} has no worktree, marking offline", name);
                worker.status = WorkerStatus::Offline;
                healed.push(format!("{}: marked offline (no worktree)", name));
            }

            // Heal 2: Worker idle with stale commits
            if worker.status == WorkerStatus::Idle && worktree.exists() {
                if let Ok(true) = git::has_commits_ahead_of(&worktree, "master") {
                    tracing::warn!("Healing: Idle worker {} has stale commits, resetting", name);
                    if git::reset_to_ref(&worktree, "master").is_ok() {
                        healed.push(format!("{}: reset stale commits", name));
                    }
                }
            }

            // Heal 3: Session exists but worker offline
            if worker.status == WorkerStatus::Offline {
                if session::session_exists(&worker.session_id) {
                    tracing::info!("Healing: Worker {} has session but marked offline", name);
                    // Don't change state - let hook handle it
                }
            }
        }

        healed
    }
}
```

---

## 8. Migration Plan

### 8.1 Prerequisites

Before beginning migration:

1. **All workers must be idle** with no pending work
2. **Backup existing state**: `cp -r ~/llmc ~/llmc.backup`
3. **Verify no TMUX sessions running**: `tmux list-sessions | grep llmc`
4. **Commit any work in source repo**

### 8.2 Migration Steps

```bash
# 1. Stop LLMC daemon
llmc down --kill-consoles

# 2. Verify all workers idle
llmc status  # All should show "idle"

# 3. Salvage any work (paranoid mode)
for worker in $(llmc status --json | jq -r '.workers[].name'); do
    llmc salvage $worker --output ~/llmc-backup/$worker.patch 2>/dev/null || true
done

# 4. Backup current installation
cp -r ~/llmc ~/llmc.backup.$(date +%Y%m%d)

# 5. Nuke all workers
llmc nuke --all --yes

# 6. (After code changes) Reinitialize
llmc init --source ~/Documents/GoogleDrive/dreamtides --force

# 7. Re-add workers
llmc add adam
llmc add baker
# ... etc

# 8. Start daemon
llmc up
```

### 8.3 Verification Checklist

Post-migration verification:

- [ ] `llmc status` shows all workers offline
- [ ] `llmc up` starts daemon successfully
- [ ] `llmc status` shows all workers idle
- [ ] Source repo `.gitignore` contains `.llmc-worktrees/`
- [ ] Worktrees exist under `<source>/.llmc-worktrees/`
- [ ] No `.git/` directory in `~/llmc/`
- [ ] `llmc doctor` reports no issues
- [ ] Test full cycle: start → work → accept

---

## 9. Opus Session Specifications

### 9.1 Session 1: Infrastructure Changes

**Estimated Scope:** ~60% of total work

**Objectives:**
1. Update `config.rs` with new helper methods
2. Rewrite `commands/init.rs` completely
3. Update `commands/add.rs` for new paths
4. Update `commands/nuke.rs` for new paths
5. Update `commands/reset.rs` for new paths
6. Add new git safety functions to `git.rs`
7. Basic testing of init/add/nuke cycle

**Prompt for Session 1:**

```
You are implementing Phase 1 of the LLMC Unified Architecture migration.

CONTEXT:
- Read rules_engine/docs/llmc_unified.md for full design (especially sections 2-4)
- Read rules_engine/docs/llmc.md for current system understanding
- Code is in rules_engine/src/llmc/

YOUR TASKS:
1. Update config.rs:
   - Add worktree_dir and metadata_dir to RepoConfig struct (optional fields)
   - Add helper methods: worktree_dir(), source_repo(), metadata_dir()
   - Ensure ~ expansion works in paths

2. Rewrite commands/init.rs:
   - Remove clone_repository() and all cloning logic
   - Create only metadata directory structure
   - Add .llmc-worktrees/ to source repo's .gitignore
   - Create config pointing to source repo

3. Update commands/add.rs:
   - Use config.source_repo() for git operations
   - Use config.worktree_dir() for worktree paths
   - Remove any origin/master references, use master

4. Update commands/nuke.rs:
   - Update worktree path resolution
   - Update branch deletion to use source repo

5. Update commands/reset.rs:
   - Update all repo path references

6. Add to git.rs:
   - is_merge_in_progress()
   - is_cherry_pick_in_progress()
   - verify_safe_to_modify() - comprehensive pre-operation check

TESTING (do these manually):
1. rm -rf ~/llmc && rm -rf ~/Documents/GoogleDrive/dreamtides/.llmc-worktrees
2. cargo build -p llmc
3. ./target/debug/llmc init --source ~/Documents/GoogleDrive/dreamtides
4. Verify: ~/llmc/ exists with config.toml, state.json, logs/
5. Verify: NO .git/ in ~/llmc/
6. Verify: .llmc-worktrees/ in source repo
7. Verify: .gitignore updated
8. ./target/debug/llmc add testworker
9. Verify: .llmc-worktrees/testworker/ exists
10. Verify: branch llmc/testworker exists in source repo
11. ./target/debug/llmc nuke testworker
12. Verify: worktree and branch removed

IMPORTANT:
- Run 'just fmt' after changes
- Run 'just check' to verify compilation
- DO NOT modify accept.rs, patrol.rs, or start.rs yet (Session 2)
- Log all changes made for handoff to Session 2
```

**Success Criteria for Session 1:**
- [ ] `just check` passes
- [ ] `just fmt` produces no changes
- [ ] `llmc init` creates metadata-only directory
- [ ] `llmc add` creates worktree in source repo
- [ ] `llmc nuke` cleanly removes worker
- [ ] No `origin/master` references in modified files
- [ ] All new git safety functions implemented

### 9.2 Session 2: Accept Flow and Patrol

**Estimated Scope:** ~40% of total work

**Objectives:**
1. Simplify `commands/accept.rs` completely
2. Update `patrol.rs` to use `master` instead of `origin/master`
3. Update `commands/start.rs` path references
4. Update `commands/rebase.rs`
5. Update `commands/review.rs` if needed
6. Update `commands/doctor.rs` for new architecture
7. End-to-end testing

**Prompt for Session 2:**

```
You are implementing Phase 2 of the LLMC Unified Architecture migration.

CONTEXT:
- Read rules_engine/docs/llmc_unified.md for full design (especially sections 4.2.3, 5, 6)
- Session 1 completed: init.rs, add.rs, nuke.rs, reset.rs, config.rs, git.rs updated
- The system now uses worktrees directly from source repo
- config.source_repo() returns the main git repo path
- config.worktree_dir() returns the worktree directory path
- All references to origin/master should become master

YOUR TASKS:
1. Rewrite commands/accept.rs:
   - Remove ALL fetch_origin() calls
   - Remove fetch_from_local() call
   - Remove the "reset source repo" step
   - Change origin/master to master everywhere
   - Add validate_source_repo_state() check before accept
   - Simplify flow: rebase → squash → ff-merge → reset worker

2. Update patrol.rs:
   - Change all origin/master to master
   - Remove any fetch_origin() calls
   - Update rebase_pending_reviews()

3. Update commands/start.rs:
   - Change origin/master to master
   - Remove fetch operations
   - Use config.source_repo() where needed

4. Update commands/rebase.rs:
   - Change origin/master to master
   - Remove fetch operations

5. Update commands/review.rs:
   - Update any path references if needed

6. Update commands/doctor.rs:
   - Remove checks for ~/llmc/.git/
   - Add check for source repo accessibility
   - Add check for .llmc-worktrees/ directory
   - Update orphan detection for new paths

7. Update git.rs:
   - Modify pull_rebase() to not fetch (just rebase onto master)
   - Or rename to just rebase_onto_master()

TESTING (do these manually after each major change):

Test 1 - Basic workflow:
1. llmc up (in background or separate terminal)
2. llmc start testworker --prompt "Create a file called test.txt with 'hello'"
3. Wait for worker to complete (llmc status shows needs_review)
4. llmc review testworker
5. llmc accept testworker
6. Verify: test.txt exists in source repo
7. Verify: testworker is idle with fresh worktree

Test 2 - Accept with dirty source repo (should fail):
1. Create uncommitted change in source repo
2. Start and complete a worker task
3. llmc accept should fail with clear message

Test 3 - Accept with source repo not on master (should fail):
1. git checkout -b test-branch (in source repo)
2. Try llmc accept - should fail

Test 4 - Rebase scenario:
1. Start worker on task
2. While worker is working, make a commit directly to master
3. Worker completes
4. llmc accept should rebase automatically

Test 5 - Patrol functionality:
1. Verify patrol runs without errors (check logs)
2. Verify workers in needs_review get rebased if master advances

IMPORTANT:
- Run 'just fmt' after changes
- Run 'just check' to verify compilation
- Run 'just clippy' for lint checks
- Search for any remaining 'origin/master' strings
- Search for any remaining 'fetch_origin' calls
```

**Success Criteria for Session 2:**
- [ ] `just check` passes
- [ ] `just clippy` has no warnings
- [ ] Full workflow test passes
- [ ] No `origin/master` references remain in codebase
- [ ] No `fetch_origin` calls remain (except maybe in unused code)
- [ ] Accept with dirty repo fails gracefully
- [ ] Patrol runs without errors
- [ ] `llmc doctor` reports system healthy

### 9.3 Session 3: Recovery Tools and Documentation

**Estimated Scope:** Polish and hardening

**Objectives:**
1. Implement `llmc salvage` command
2. Implement `llmc rescue` command
3. Enhance `llmc doctor --repair`
4. Add self-healing to patrol
5. Update `llmc.md` documentation
6. Final testing and edge case handling

**Prompt for Session 3:**

```
You are implementing Phase 3 of the LLMC Unified Architecture migration.

CONTEXT:
- Read rules_engine/docs/llmc_unified.md for full design (especially sections 6, 7)
- Sessions 1-2 completed the core migration
- Now we need recovery tools and documentation

YOUR TASKS:
1. Add commands/salvage.rs:
   - New command: llmc salvage <worker> [--patch <file>] [--branch <name>]
   - Extracts commits from worker as patch or creates branch
   - See section 7.2 in llmc_unified.md for design

2. Add commands/rescue.rs:
   - New command: llmc rescue [--yes]
   - Complete recovery: salvages all work, resets everything
   - See section 7.3 in llmc_unified.md for design

3. Update cli.rs:
   - Add Salvage and Rescue subcommands

4. Enhance commands/doctor.rs:
   - Add --rebuild flag to reconstruct state from filesystem
   - Improve --repair to handle more edge cases
   - Add checks specific to unified architecture

5. Enhance patrol.rs:
   - Add self_heal() method that runs each patrol
   - Automatically fix: orphaned commits, missing worktrees, state drift
   - See section 7.4 in llmc_unified.md

6. Update rules_engine/docs/llmc.md:
   - Update Repository Layout section
   - Update all command documentation
   - Remove references to separate clone
   - Add salvage and rescue commands
   - Update troubleshooting section

7. Edge case testing:
   - Test salvage with no changes
   - Test salvage with multiple commits
   - Test rescue full workflow
   - Test doctor --rebuild
   - Simulate various failure modes

TESTING:

Test 1 - Salvage command:
1. llmc start worker --prompt "Make several commits"
2. (simulate crash or issue)
3. llmc salvage worker --patch ~/test.patch
4. Verify patch contains changes

Test 2 - Rescue command:
1. Create broken state (manually corrupt state.json)
2. llmc rescue --yes
3. Verify: all work salvaged, state reset, config preserved

Test 3 - Doctor rebuild:
1. Delete state.json
2. llmc doctor --rebuild
3. Verify state reconstructed from worktrees

Test 4 - Self-healing patrol:
1. Manually create orphaned worktree
2. Wait for patrol or trigger manually
3. Verify orphan cleaned up

IMPORTANT:
- All new commands need --json support
- Add comprehensive error messages
- Run full test suite after changes
- Update llmc.md to match all changes
```

**Success Criteria for Session 3:**
- [ ] `llmc salvage` works for patches and branches
- [ ] `llmc rescue` recovers from broken state
- [ ] `llmc doctor --rebuild` reconstructs state
- [ ] Patrol self-heals common issues
- [ ] `llmc.md` fully updated
- [ ] All edge cases documented and handled

---

## 10. Testing Procedures

### 10.1 Unit Test Additions

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

### 10.2 Integration Test Script

Create `test_unified_migration.sh`:

```bash
#!/bin/bash
set -e

echo "=== LLMC Unified Architecture Integration Tests ==="

# Setup
TEST_DIR=$(mktemp -d)
SOURCE_REPO="$TEST_DIR/source"
LLMC_ROOT="$TEST_DIR/llmc"
export HOME="$TEST_DIR"

# Create source repo
mkdir -p "$SOURCE_REPO"
cd "$SOURCE_REPO"
git init
echo "initial" > README.md
git add README.md
git commit -m "Initial commit"

# Test 1: Init
echo -e "\n=== Test 1: Init ==="
llmc init --source "$SOURCE_REPO" --target "$LLMC_ROOT"

[[ -f "$LLMC_ROOT/config.toml" ]] || { echo "FAIL: config.toml missing"; exit 1; }
[[ -f "$LLMC_ROOT/state.json" ]] || { echo "FAIL: state.json missing"; exit 1; }
[[ ! -d "$LLMC_ROOT/.git" ]] || { echo "FAIL: .git should not exist"; exit 1; }
[[ -d "$SOURCE_REPO/.llmc-worktrees" ]] || { echo "FAIL: worktree dir missing"; exit 1; }
grep -q ".llmc-worktrees" "$SOURCE_REPO/.gitignore" || { echo "FAIL: gitignore not updated"; exit 1; }
echo "PASS: Init"

# Test 2: Add worker
echo -e "\n=== Test 2: Add Worker ==="
llmc add testworker

[[ -d "$SOURCE_REPO/.llmc-worktrees/testworker" ]] || { echo "FAIL: worktree not created"; exit 1; }
git -C "$SOURCE_REPO" branch | grep -q "llmc/testworker" || { echo "FAIL: branch not created"; exit 1; }
echo "PASS: Add Worker"

# Test 3: Nuke worker
echo -e "\n=== Test 3: Nuke Worker ==="
llmc nuke testworker --yes

[[ ! -d "$SOURCE_REPO/.llmc-worktrees/testworker" ]] || { echo "FAIL: worktree still exists"; exit 1; }
git -C "$SOURCE_REPO" branch | grep -q "llmc/testworker" && { echo "FAIL: branch still exists"; exit 1; }
echo "PASS: Nuke Worker"

# Test 4: Full workflow (requires daemon)
echo -e "\n=== Test 4: Full Workflow ==="
llmc add worker1
# Note: Full workflow test requires running daemon, skipping for unit test

# Cleanup
rm -rf "$TEST_DIR"

echo -e "\n=== All Tests Passed ==="
```

### 10.3 Manual Testing Checklist

#### Phase 1 Testing (After Session 1)

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
  git -C ~/Documents/GoogleDrive/dreamtides branch | grep llmc
  # Should be empty

□ Reset recreates cleanly
  llmc add testworker
  # Make some changes in worktree
  llmc reset testworker --yes
  # Verify clean state
```

#### Phase 2 Testing (After Session 2)

```
□ Start sends prompt correctly
  llmc up &
  llmc add testworker
  llmc start testworker --prompt "echo 'hello' > test.txt && git add test.txt && git commit -m 'test'"
  llmc status
  # Should show: working

□ Accept merges to master
  # Wait for worker to complete
  llmc accept testworker
  cat ~/Documents/GoogleDrive/dreamtides/test.txt
  # Should see: hello
  git -C ~/Documents/GoogleDrive/dreamtides log --oneline -1
  # Should see: test commit

□ Accept fails with dirty repo
  echo "dirty" > ~/Documents/GoogleDrive/dreamtides/dirty.txt
  llmc start testworker --prompt "..."
  # Wait for completion
  llmc accept testworker
  # Should fail with clear message
  rm ~/Documents/GoogleDrive/dreamtides/dirty.txt

□ Accept fails when not on master
  git -C ~/Documents/GoogleDrive/dreamtides checkout -b test-branch
  llmc accept testworker
  # Should fail with clear message
  git -C ~/Documents/GoogleDrive/dreamtides checkout master
  git -C ~/Documents/GoogleDrive/dreamtides branch -d test-branch

□ Rebase on accept works
  llmc start testworker --prompt "..."
  # While working, commit directly to master:
  echo "concurrent" > ~/Documents/GoogleDrive/dreamtides/concurrent.txt
  git -C ~/Documents/GoogleDrive/dreamtides add concurrent.txt
  git -C ~/Documents/GoogleDrive/dreamtides commit -m "concurrent"
  # Wait for worker to complete
  llmc accept testworker
  # Should rebase successfully

□ Patrol runs without errors
  tail -f ~/llmc/logs/llmc.log | grep -i patrol
  # Should see patrol runs with no errors
```

#### Phase 3 Testing (After Session 3)

```
□ Salvage creates patch
  llmc start testworker --prompt "..."
  # Let it make changes but don't accept
  llmc salvage testworker --patch ~/test.patch
  cat ~/test.patch
  # Should contain diff

□ Salvage creates branch
  llmc salvage testworker --branch salvage-test
  git -C ~/Documents/GoogleDrive/dreamtides branch | grep salvage
  # Should see: salvage-test

□ Rescue recovers everything
  # Corrupt state.json
  echo "invalid" > ~/llmc/state.json
  llmc rescue --yes
  ls ~/llmc-rescue/
  # Should see timestamped directory with patches
  llmc status
  # Should work again

□ Doctor rebuild works
  rm ~/llmc/state.json
  llmc doctor --rebuild
  cat ~/llmc/state.json
  # Should be valid

□ Self-healing patrol
  # Manually create orphan:
  mkdir ~/Documents/GoogleDrive/dreamtides/.llmc-worktrees/orphan
  # Wait for patrol
  ls ~/Documents/GoogleDrive/dreamtides/.llmc-worktrees/
  # Orphan should be removed
```

---

## 11. Logging Specification

### 11.1 Log Levels

| Level | Usage |
|-------|-------|
| ERROR | Operation failed, requires attention |
| WARN | Unexpected state, auto-recovered or degraded |
| INFO | Significant state changes, operation completion |
| DEBUG | Detailed operation flow, useful for debugging |
| TRACE | Very verbose, git command output |

### 11.2 Structured Log Fields

All log entries should include:

```rust
tracing::info!(
    operation = "accept",
    worker = %worker_name,
    source_repo = %config.source_repo().display(),
    worktree = %worktree_path.display(),
    duration_ms = elapsed.as_millis(),
    result = "success",
    commit_sha = %new_sha,
    "Accepted worker changes"
);
```

### 11.3 Key Operations to Log

| Operation | Level | Fields |
|-----------|-------|--------|
| Init | INFO | source_repo, metadata_dir |
| Add worker | INFO | worker, branch, worktree_path |
| Nuke worker | INFO | worker, had_uncommitted_changes |
| Start task | INFO | worker, prompt_length, self_review |
| Accept | INFO | worker, commit_sha, rebase_needed |
| Accept fail | ERROR | worker, reason, source_repo_state |
| Rebase | INFO | worker, base_ref, conflicts |
| Patrol run | DEBUG | workers_checked, transitions, errors |
| Self-heal | WARN | worker, issue, action_taken |
| Git operation | DEBUG | operation_type, repo, duration_ms, result |
| Git operation fail | ERROR | operation_type, repo, error, stderr |

### 11.4 Log File Rotation

```rust
// In logging/config.rs
pub fn configure_logging() -> Result<()> {
    let log_dir = config::get_llmc_root().join("logs");

    let file_appender = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .max_log_files(7)  // Keep 1 week
        .filename_prefix("llmc")
        .filename_suffix("log")
        .build(&log_dir)?;

    // ... rest of setup
}
```

---

## 12. Rollback Plan

### 12.1 If Migration Fails Mid-Way

```bash
# 1. Stop daemon
llmc down --force --kill-consoles

# 2. Restore backup
rm -rf ~/llmc
cp -r ~/llmc.backup ~/llmc

# 3. Remove new worktree directory
rm -rf ~/Documents/GoogleDrive/dreamtides/.llmc-worktrees

# 4. Revert code changes
git checkout HEAD -- rules_engine/src/llmc/

# 5. Rebuild
cargo build -p llmc

# 6. Restart
llmc up
```

### 12.2 If Issues Discovered Post-Migration

```bash
# 1. Salvage all work first
mkdir -p ~/llmc-emergency
for worker in $(llmc status --json | jq -r '.workers[].name'); do
    llmc salvage $worker --patch ~/llmc-emergency/$worker.patch || true
done

# 2. Follow rollback steps above

# 3. Apply patches to old system
for patch in ~/llmc-emergency/*.patch; do
    worker=$(basename $patch .patch)
    cd ~/llmc/.worktrees/$worker
    git apply $patch
done
```

### 12.3 Point of No Return

The migration is reversible until:
1. New commits are accepted (merged to master)
2. Old `~/llmc/.git` is deleted

Always keep `~/llmc.backup` until confident in new system.

---

## Appendix A: File Change Summary

```
Modified files:
  rules_engine/src/llmc/src/config.rs        (+50 lines)
  rules_engine/src/llmc/src/git.rs           (+80 lines, -20 lines)
  rules_engine/src/llmc/src/commands/init.rs (-60 lines, +40 lines)
  rules_engine/src/llmc/src/commands/add.rs  (~20 lines changed)
  rules_engine/src/llmc/src/commands/nuke.rs (~10 lines changed)
  rules_engine/src/llmc/src/commands/reset.rs (~10 lines changed)
  rules_engine/src/llmc/src/commands/accept.rs (-100 lines, +60 lines)
  rules_engine/src/llmc/src/commands/start.rs (~15 lines changed)
  rules_engine/src/llmc/src/commands/rebase.rs (~10 lines changed)
  rules_engine/src/llmc/src/commands/doctor.rs (+50 lines, -30 lines)
  rules_engine/src/llmc/src/patrol.rs        (~30 lines changed)
  rules_engine/src/llmc/src/cli.rs           (+20 lines)

New files:
  rules_engine/src/llmc/src/commands/salvage.rs (~100 lines)
  rules_engine/src/llmc/src/commands/rescue.rs  (~80 lines)

Documentation:
  rules_engine/docs/llmc.md                  (major updates)
  rules_engine/docs/llmc_unified.md          (this document)

Estimated total: ~400 lines added, ~200 lines removed, ~100 lines modified
```

---

## Appendix B: Glossary

| Term | Definition |
|------|------------|
| Source repo | The main git repository (`~/Documents/GoogleDrive/dreamtides`) |
| Metadata dir | Directory containing LLMC config and state (`~/llmc`) |
| Worktree dir | Directory containing worker worktrees (`<source>/.llmc-worktrees/`) |
| Worker | A Claude Code session with its own worktree |
| Worktree | A git worktree - separate working directory sharing the same repo |
| Accept | Merging a worker's changes to master |
| Patrol | Background process maintaining system health |

---

## Appendix C: Quick Reference

### Commands After Migration

```bash
# Initialize (first time only)
llmc init --source ~/Documents/GoogleDrive/dreamtides

# Add worker
llmc add <name>

# Start daemon
llmc up

# Assign work
llmc start <worker> --prompt "..."

# Accept work
llmc accept [worker]

# Health check
llmc doctor

# Recovery
llmc salvage <worker> --patch <file>
llmc rescue --yes
llmc doctor --rebuild
```

### Key Paths After Migration

```
~/llmc/config.toml                          # Configuration
~/llmc/state.json                           # Worker state
~/llmc/logs/                                # Log files
~/Documents/GoogleDrive/dreamtides/         # Git repo (all operations)
~/Documents/GoogleDrive/dreamtides/.llmc-worktrees/  # Worker worktrees
```
